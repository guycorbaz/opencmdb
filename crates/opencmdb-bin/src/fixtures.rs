//! Reading the committed fixture corpus (Story 4.1).
//!
//! The corpus lives at the workspace ROOT, in `fixtures/`, outside every crate (D56): a file
//! under `tests/` reads as the property of the test, and the first reflex of someone refactoring
//! the engine is to edit it until the red goes away. At the root, changing one is a commit that
//! says *"I am changing the spec"*.
//!
//! **An observation line IS an [`Observation`]** (D19), in the domain types' own serde
//! representation — no DTO, no wrapper, no second format to keep in step: *"the fixture is a
//! serialised stream of Observations … write the fixture and the trait falls out."*
//!
//! **A stream is more than its observations, since story 4.5a.** It may also carry CONTROL
//! records, which script the poll's outcome. That is not a departure from D19 but its other half:
//! D34 §1 argued the descriptor must travel with the batch precisely because *"the fixture replays
//! it for free — ONE JSONL LINE reproduces a mid-scan NET_RAW loss, zero mocks; with a separate
//! getter the fixture would need state outside the JSONL."* The architecture sanctions the line
//! and rules on nothing else, so the shape is decided here:
//!
//! - a line carrying `record` is a control record — `failure` (the poll ends with a
//!   [`ConnectorError`]) or `capability` (the descriptor changes and the poll continues);
//! - a line carrying `obs_id` is an [`Observation`], parsed exactly as it always was;
//! - a line carrying neither, or both, is REFUSED by name and line number.
//!
//! **The discriminator is a positive marker, never the absence of `obs_id`** — an absence-based
//! rule routes a line whose `obs_id` is misspelled into the control parser and reports an
//! unknown-field error on a record the author never wrote, which is the opposite of what story 4.1
//! fought for. [`read_jsonl`] still yields observations only, for the callers that want just those.
//!
//! Nothing here reads a clock or mints an id. `obs_id` is stable so truth can point at it, and
//! `observed_at` comes from the file so the engine never touches the clock — determinism is what
//! makes the corpus an oracle rather than a snapshot.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use opencmdb_core::connector::ConnectorError;
use opencmdb_core::observation::{
    Capabilities, ConnectorId, FactKind, L2DomainId, Observation, Timestamp, VantageId,
};
use opencmdb_core::trap::{TrapError, TrapFile};
use serde::Deserialize;

/// The one and only expression of where the corpus lives (D56 path discipline).
///
/// There is no `CARGO_WORKSPACE_DIR`, so the root is reached relatively from this crate's
/// manifest directory. **If this string appears anywhere else in the tree, it is already
/// broken** — take the path from [`fixtures_dir`] instead of writing it again.
const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/");

/// The corpus root, as an owned path.
pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(FIXTURES_DIR)
}

/// Resolve a corpus-relative path (e.g. `scenario/replay/minimal.jsonl`).
///
/// The argument is CONTAINED to the corpus: an absolute path would make `join` discard the root
/// entirely, and a `..` component would climb out of it. The MANIFEST parser already refuses
/// both, so accepting them here would make the reader strictly more permissive than the lock
/// that is supposed to guard it — and any future connector taking a fixture name from
/// configuration would read arbitrary files.
pub fn fixture_path(relative: &str) -> Result<PathBuf, FixtureError> {
    let candidate = Path::new(relative);
    // `CurDir` is refused alongside `ParentDir`: `./x` and `x` name one file but are two cache
    // keys and two MANIFEST spellings, and only one of them is the spelling the lock records.
    if candidate.is_absolute()
        || candidate.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err(FixtureError::OutsideCorpus {
            requested: relative.to_string(),
        });
    }
    Ok(fixtures_dir().join(candidate))
}

/// One line of a replay stream: what the source saw, or what the poll DID.
///
/// The two kinds are not variants of one idea. An observation is a fact; a failure is the end of
/// the poll. Story 4.5b adds a third kind — a capability change — which is neither, because it
/// leaves the poll `Ok` with a different descriptor (D33: *"`CapabilityLost` is an event, not a
/// state — ping-only is an `Ok` with a reduced descriptor, not an error"*).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Record {
    /// What the source saw.
    Observation(Observation),
    /// The poll ends HERE, with this error. Everything emitted before it is still true (D34 §2).
    Failure(ConnectorError),
    /// The source's capability descriptor CHANGES here, and the poll continues.
    ///
    /// Not an error, deliberately: a source that lost NET_RAW is still `Live` — it is talking.
    /// D33 settles it — *"`CapabilityLost` is an **event**, not a state — in steady state ping-only
    /// is an `Ok` with a reduced descriptor, **not an error**"*. Every `ConnectorError` except
    /// `Cancelled` blinds, and blinding a live source is the false-"gone" NFR7 makes impossible.
    ///
    /// The descriptor is DATED BY THE FILE. That is the whole point: D34 §1 argues the descriptor
    /// is *"a dated fact, not a constant"*, and story 4.4 had to record that a caller-supplied
    /// `as_of` could date it in a moment its own stream contradicts.
    Capability(Capabilities),
}

impl Record {
    /// The observation this record carries, if it is one.
    pub fn as_observation(&self) -> Option<&Observation> {
        match self {
            Record::Observation(observation) => Some(observation),
            Record::Failure(_) | Record::Capability(_) => None,
        }
    }
}

/// The on-disk shape of a control record: internally tagged on `record`.
///
/// Internally tagged, not externally: an externally tagged enum would render the line as
/// `{"failure":{…}}`, whose only key is the variant name — leaving no fixed marker to discriminate
/// on before parsing. `record` is that marker, and it is what makes the dispatch in
/// [`read_records`] a positive test rather than a guess.
/// `Serialize` since story 5.1, so the corpus-wide round-trip witness
/// (`every_replay_stream_re_serializes_to_its_committed_bytes`) can render a control record
/// back to its committed line — the two control lines are covered by that witness rather than
/// excluded from it. _(This doc claimed "without it the witness would silently skip every control
/// record" until story 5.1's review: false. `render_record`'s `match` is exhaustive with no `_` arm
/// precisely so a record kind cannot be rendered as nothing — without this derive the witness would
/// not COMPILE. A build error is not a silent skip.)_
#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(tag = "record", rename_all = "snake_case", deny_unknown_fields)]
enum ControlRecord {
    /// `{"record":"failure","error":{"Unreachable":{"detail":"…"}}}`
    Failure { error: ConnectorError },
    /// `{"record":"capability","as_of":"2026-02-01T00:00:07Z","kinds":["IpV4","Mac"]}`
    ///
    /// Flattened, so the line IS a `Capabilities` plus the marker — no wrapper key to learn, and
    /// the domain type stays the single definition of what a descriptor is.
    Capability {
        #[serde(flatten)]
        capabilities: Capabilities,
    },
}

/// Why a fixture could not be read, or why a stream may not CLAIM what it claims. A malformed
/// line names its own 1-indexed number: a corpus that silently skips a line it cannot parse is
/// not an oracle.
///
/// Two shapes live here. The reading variants carry a `path: PathBuf` — they always come from a
/// file. The four replay-admissibility variants added by story 4.4 carry an `origin: String`
/// instead, because a stream handed to `FixtureConnector::from_observations` may never have been
/// on disk; a fabricated `PathBuf::from("<in-memory>")` would be a lie in the type, told only to
/// preserve a habit.
#[derive(Debug)]
pub enum FixtureError {
    /// The file could not be opened or read.
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    /// A line did not deserialize into an [`Observation`].
    Line {
        path: PathBuf,
        lineno: usize,
        source: serde_json::Error,
    },
    /// The requested path would leave the corpus.
    OutsideCorpus { requested: String },
    /// A trap file did not parse.
    Toml {
        path: PathBuf,
        source: toml::de::Error,
    },
    /// A trap file parsed but is not admissible.
    Trap { path: PathBuf, source: TrapError },
    /// A replay stream contains the same `obs_id` twice.
    DuplicateObservationId {
        path: PathBuf,
        obs_id: String,
        first_line: usize,
        second_line: usize,
    },
    /// A trap judges an observation that its replay stream does not contain.
    DanglingObservation {
        path: PathBuf,
        trap: String,
        obs_id: String,
        replay: String,
    },

    // ── Replay admissibility (story 4.4). These carry `origin`, not `path`. ──
    /// A stream carries an observation attributed to a different connector than the one
    /// replaying it. Emitting it would fabricate provenance.
    ForeignConnectorId {
        origin: String,
        expected: ConnectorId,
        found: ConnectorId,
        obs_id: String,
    },
    /// A stream observes a scope the poll does not claim to have covered. The reverse —
    /// covered and empty — stays legitimate: it is what makes an absence meaningful.
    UncoveredScope {
        origin: String,
        l2_domain: L2DomainId,
        vantage: VantageId,
        obs_id: String,
    },
    /// A stream emits a fact of a kind the descriptor IN FORCE AT ITS POSITION denies. The
    /// reverse — capable and unseen — stays legitimate: it is the whole point of the
    /// descriptor (D34 §1). `descriptor` names WHICH descriptor denied it, by its `as_of` — a
    /// capability record has no `obs_id`, and story 4.2 forbids naming anything by line number.
    UndeclaredFactKind {
        origin: String,
        kind: FactKind,
        obs_id: String,
        descriptor: String,
    },
    /// A capability record is dated BEFORE an observation that precedes it in the stream. A
    /// descriptor cannot be dated before facts collected under it (D34 §1: it is a dated fact).
    CapabilityPredatesObservation {
        origin: String,
        as_of: Timestamp,
        observed_at: Timestamp,
        obs_id: String,
    },
    /// Two capability records go backwards in time. The descriptor's history is a timeline, and
    /// a timeline that goes backwards cannot say which descriptor was in force when.
    CapabilityOutOfOrder {
        origin: String,
        as_of: Timestamp,
        previous_as_of: Timestamp,
    },
    /// An in-memory stream repeats an `obs_id`. [`read_jsonl`] already refuses this for a
    /// file, naming both lines; a `Vec` has no lines, so this variant names the id alone.
    RepeatedObservationId { origin: String, obs_id: String },

    // ── Record dispatch (story 4.5a). Read from a file, so `path` + `lineno`. ──
    /// A line is neither an observation nor a control record. `found` says what was there
    /// instead, because "unrecognised" alone sends the author looking in the wrong place.
    UnrecognisedLine {
        path: PathBuf,
        lineno: usize,
        found: String,
    },
    /// A line carries BOTH `obs_id` and `record`. A line that is two things is a line whose
    /// meaning depends on which reader reads it.
    AmbiguousLine { path: PathBuf, lineno: usize },
    /// A control record did not deserialize. Distinct from [`FixtureError::Line`] so the message
    /// says which of the two shapes was being read — the whole point of dispatching first.
    ControlRecordLine {
        path: PathBuf,
        lineno: usize,
        source: serde_json::Error,
    },
    /// A file scripts `ConnectorError::Cancelled`. Cancellation comes from the token, never from
    /// the corpus: it is the only non-blinding variant, so a file able to mint it could claim
    /// liveness was left unchanged when nothing cancelled anything.
    CancellationScripted { path: PathBuf, lineno: usize },
    /// A record follows a terminal failure, and would therefore never be reached. See
    /// [`read_records`] for why an unreachable observation is worse than a missing one.
    RecordAfterTerminalFailure {
        path: PathBuf,
        lineno: usize,
        failure_line: usize,
    },
    /// An in-memory stream scripts `Cancelled`. [`read_records`] refuses this for a file, naming
    /// the line; a `Vec` has no lines. The pairing mirrors
    /// [`FixtureError::DuplicateObservationId`] / [`FixtureError::RepeatedObservationId`]: a
    /// caller that wants to handle "this stream mints a cancellation" must match BOTH.
    CancellationInStream { origin: String },
    /// Two trap files under one corpus root define a trap with the same `id`. `TrapFile::validate`
    /// enforces id-uniqueness within ONE file only, so the metrics harness (story 4.6b) checks it
    /// across the corpus: a `TrapId` is the key an answer is scored against, and one id naming two
    /// traps would score a single outcome twice, in two files. Mirrors the cross-stream `obs_id`
    /// rule for observations.
    DuplicateTrapId {
        trap: String,
        first: PathBuf,
        second: PathBuf,
    },
    /// An answer was supplied for a trap id that no discovered trap carries — a stale, renamed or
    /// typo'd producer id. The gate refuses it rather than silently ignoring the outcome. `count`
    /// is how many such answers there were; `trap` names one of them.
    AnswerForUnknownTrap { trap: String, count: usize },
}

impl std::fmt::Display for FixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixtureError::Io { path, source } => {
                write!(f, "reading fixture {}: {source}", path.display())
            }
            FixtureError::Line {
                path,
                lineno,
                source,
            } => write!(f, "{}:{lineno}: {source}", path.display()),
            FixtureError::OutsideCorpus { requested } => write!(
                f,
                "fixture path `{requested}` leaves the corpus (absolute paths and `..` are refused)"
            ),
            FixtureError::Toml { path, source } => {
                write!(f, "trap file {}: {source}", path.display())
            }
            FixtureError::Trap { path, source } => write!(f, "{}: {source}", path.display()),
            FixtureError::DuplicateObservationId {
                path,
                obs_id,
                first_line,
                second_line,
            } => write!(
                f,
                "{}: observation {obs_id} appears on lines {first_line} and {second_line} — \
                 within one stream an obs_id must name exactly one observation, or a trap \
                 referencing it does not say which",
                path.display()
            ),
            FixtureError::DanglingObservation {
                path,
                trap,
                obs_id,
                replay,
            } => write!(
                f,
                "{}: trap `{trap}` judges observation {obs_id}, which `{replay}` does not contain",
                path.display()
            ),
            FixtureError::ForeignConnectorId {
                origin,
                expected,
                found,
                obs_id,
            } => write!(
                f,
                "{origin}: observation {obs_id} is attributed to connector {found}, but this \
                 replay is connector {expected} — one stream is one connector, and emitting \
                 another's observations would fabricate provenance"
            ),
            FixtureError::UncoveredScope {
                origin,
                l2_domain,
                vantage,
                obs_id,
            } => write!(
                f,
                "{origin}: observation {obs_id} is in scope (l2_domain {l2_domain}, vantage \
                 {vantage}), which this poll does not claim to have covered — a poll may cover \
                 more than it saw, never less"
            ),
            FixtureError::UndeclaredFactKind {
                origin,
                kind,
                obs_id,
                descriptor,
            } => write!(
                f,
                "{origin}: observation {obs_id} emits a {kind:?} fact, which {descriptor} says \
                 the source cannot emit — a source may be capable and see nothing, never the \
                 reverse"
            ),
            FixtureError::CapabilityPredatesObservation {
                origin,
                as_of,
                observed_at,
                obs_id,
            } => write!(
                f,
                "{origin}: a capability record dated {as_of} follows observation {obs_id}, dated \
                 {observed_at} — a descriptor cannot be dated before facts collected under it"
            ),
            FixtureError::CapabilityOutOfOrder {
                origin,
                as_of,
                previous_as_of,
            } => write!(
                f,
                "{origin}: a capability record dated {as_of} follows one dated {previous_as_of} — \
                 the descriptor's history is a timeline, and it may not go backwards"
            ),
            FixtureError::RepeatedObservationId { origin, obs_id } => write!(
                f,
                "{origin}: observation {obs_id} appears more than once — within one stream an \
                 obs_id must name exactly one observation, or a trap referencing it does not \
                 say which"
            ),
            FixtureError::UnrecognisedLine {
                path,
                lineno,
                found,
            } => write!(
                f,
                "{}:{lineno}: {found} — every line must carry either `obs_id` (an observation) \
                 or `record` (a control record)",
                path.display()
            ),
            FixtureError::AmbiguousLine { path, lineno } => write!(
                f,
                "{}:{lineno}: carries both `obs_id` and `record` — a line is one or the other, \
                 never both, or its meaning depends on which reader reads it",
                path.display()
            ),
            FixtureError::ControlRecordLine {
                path,
                lineno,
                source,
            } => write!(f, "{}:{lineno}: control record: {source}", path.display()),
            FixtureError::CancellationScripted { path, lineno } => write!(
                f,
                "{}:{lineno}: a stream may not script `Cancelled` — cancellation comes from the \
                 token, and it is the only error that leaves liveness unchanged, so a file able \
                 to mint it could claim nothing was blinded when nothing cancelled anything",
                path.display()
            ),
            FixtureError::RecordAfterTerminalFailure {
                path,
                lineno,
                failure_line,
            } => write!(
                f,
                "{}:{lineno}: follows the terminal failure on line {failure_line} and could never \
                 be reached — an unreachable observation still satisfies a trap's cross-check, so \
                 it would yield a trap that can never fire",
                path.display()
            ),
            FixtureError::CancellationInStream { origin } => write!(
                f,
                "{origin}: a stream may not script `Cancelled` — cancellation comes from the \
                 token, never from the data"
            ),
            FixtureError::DuplicateTrapId {
                trap,
                first,
                second,
            } => write!(
                f,
                "trap `{trap}` is defined in both {} and {} — a trap id must name exactly one \
                 trap across the whole corpus, or one answer would be scored twice",
                first.display(),
                second.display()
            ),
            FixtureError::AnswerForUnknownTrap { trap, count } => write!(
                f,
                "{count} answer(s) name no discovered trap — the first is `{trap}`; a producer \
                 emitting an outcome the gate cannot place is a mismatch, not a no-op"
            ),
        }
    }
}

impl std::error::Error for FixtureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FixtureError::Io { source, .. } => Some(source),
            FixtureError::Line { source, .. } => Some(source),
            FixtureError::OutsideCorpus { .. } => None,
            FixtureError::Toml { source, .. } => Some(source),
            FixtureError::Trap { source, .. } => Some(source),
            FixtureError::DuplicateObservationId { .. } => None,
            FixtureError::DanglingObservation { .. } => None,
            FixtureError::ForeignConnectorId { .. } => None,
            FixtureError::UncoveredScope { .. } => None,
            FixtureError::UndeclaredFactKind { .. } => None,
            FixtureError::CapabilityPredatesObservation { .. } => None,
            FixtureError::CapabilityOutOfOrder { .. } => None,
            FixtureError::RepeatedObservationId { .. } => None,
            FixtureError::UnrecognisedLine { .. } => None,
            FixtureError::AmbiguousLine { .. } => None,
            FixtureError::ControlRecordLine { source, .. } => Some(source),
            FixtureError::CancellationScripted { .. } => None,
            FixtureError::RecordAfterTerminalFailure { .. } => None,
            FixtureError::CancellationInStream { .. } => None,
            FixtureError::DuplicateTrapId { .. } => None,
            FixtureError::AnswerForUnknownTrap { .. } => None,
        }
    }
}

/// Read a JSONL fixture into its records, in file order.
///
/// Blank lines are skipped; every other line is classified by a positive marker key BEFORE it is
/// parsed, then parsed as what it claims to be, so each shape fails with its own diagnostic and
/// the message story 4.1 froze for an observation is the message an observation still gets.
///
/// Order is preserved because replay order is part of what a trap asserts.
///
/// **Nothing may follow a terminal failure.** A trailing record would never be replayed, and
/// [`read_traps`] cross-checks a trap's `obs_id`s against what the file CONTAINS, not against what
/// is reachable — so an unreachable observation would satisfy the cross-check and yield a trap
/// that can never fire. *"A trap that can never fire would sit in the corpus looking like
/// coverage, and the gate counts traps."* That hole is the one stories 4.1 and 4.2 exist to close.
pub fn read_records(path: &Path) -> Result<Vec<Record>, FixtureError> {
    let text = std::fs::read_to_string(path).map_err(|source| FixtureError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut records = Vec::new();
    let mut seen: std::collections::BTreeMap<uuid::Uuid, usize> = std::collections::BTreeMap::new();
    let mut terminal: Option<usize> = None;
    for (index, line) in text.lines().enumerate() {
        // Only a truly empty line is skipped. A whitespace-only line carries content, and this
        // module's rule is that content it cannot parse is named, never silently dropped.
        if line.is_empty() {
            continue;
        }
        // 1-indexed, counted over the raw lines: a blank line still occupies its number, so the
        // message points at what an editor shows.
        let lineno = index + 1;

        // Classify and parse BEFORE the terminality check. Order matters for the diagnosis: a line
        // that both follows a terminal failure AND is itself inadmissible — a scripted `Cancelled`,
        // a malformed line — must be reported for what it IS, not merely for where it sits.
        // Reporting "unreachable" first costs the author two edit cycles to learn that the line
        // they wrote was never admissible anywhere.
        let value: serde_json::Value =
            serde_json::from_str(line).map_err(|source| FixtureError::Line {
                path: path.to_path_buf(),
                lineno,
                source,
            })?;
        let Some(object) = value.as_object() else {
            return Err(FixtureError::UnrecognisedLine {
                path: path.to_path_buf(),
                lineno,
                found: format!("a JSON {} is not a record", json_kind(&value)),
            });
        };

        match (object.contains_key("record"), object.contains_key("obs_id")) {
            (true, true) => {
                return Err(FixtureError::AmbiguousLine {
                    path: path.to_path_buf(),
                    lineno,
                });
            }
            (false, false) => {
                // Name the keys the author actually wrote. "An object with neither key" alone tells
                // someone who misspelled `obs_id` nothing about which key is wrong.
                let mut keys: Vec<&str> = object.keys().map(String::as_str).collect();
                keys.sort_unstable();
                let found = if keys.is_empty() {
                    "an empty object".to_string()
                } else {
                    format!("an object whose keys are [{}]", keys.join(", "))
                };
                return Err(FixtureError::UnrecognisedLine {
                    path: path.to_path_buf(),
                    lineno,
                    found,
                });
            }
            (true, false) => {
                // Re-parsed from the LINE, not from `value`: the error a control record reports
                // must be the one serde produces for the bytes the author actually wrote.
                let control: ControlRecord = serde_json::from_str(line).map_err(|source| {
                    FixtureError::ControlRecordLine {
                        path: path.to_path_buf(),
                        lineno,
                        source,
                    }
                })?;
                match control {
                    ControlRecord::Failure { error } => {
                        if error == ConnectorError::Cancelled {
                            return Err(FixtureError::CancellationScripted {
                                path: path.to_path_buf(),
                                lineno,
                            });
                        }
                        reject_if_after_terminal(path, lineno, terminal)?;
                        terminal = Some(lineno);
                        records.push(Record::Failure(error));
                    }
                    ControlRecord::Capability { capabilities } => {
                        reject_if_after_terminal(path, lineno, terminal)?;
                        records.push(Record::Capability(capabilities));
                    }
                }
            }
            (false, true) => {
                let observation: Observation =
                    serde_json::from_str(line).map_err(|source| FixtureError::Line {
                        path: path.to_path_buf(),
                        lineno,
                        source,
                    })?;
                reject_if_after_terminal(path, lineno, terminal)?;
                // `obs_id` is the anchor the whole labelling format rests on — a trap points at
                // one "never by line number" (story 4.2). Two lines sharing an id void that
                // guarantee, and a trap referencing it would silently judge whichever one the
                // reader happened to keep.
                let id = observation.obs_id.as_uuid();
                if let Some(first) = seen.insert(id, lineno) {
                    return Err(FixtureError::DuplicateObservationId {
                        path: path.to_path_buf(),
                        obs_id: id.to_string(),
                        first_line: first,
                        second_line: lineno,
                    });
                }
                records.push(Record::Observation(observation));
            }
        }
    }
    Ok(records)
}

/// Refuse an otherwise-admissible record that follows a terminal failure.
///
/// Called AFTER the line has been parsed and found admissible on its own terms, so a line that is
/// both unreachable and inadmissible is reported for what it is first.
fn reject_if_after_terminal(
    path: &Path,
    lineno: usize,
    terminal: Option<usize>,
) -> Result<(), FixtureError> {
    match terminal {
        Some(failure_line) => Err(FixtureError::RecordAfterTerminalFailure {
            path: path.to_path_buf(),
            lineno,
            failure_line,
        }),
        None => Ok(()),
    }
}

/// What a JSON value is, for a message that has to say why a line is not a record.
fn json_kind(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Read a JSONL fixture into its OBSERVATIONS only, in file order.
///
/// Control records are dropped, deliberately and not silently: this is the entry point for callers
/// that only ever wanted the observations — [`read_traps`]'s `obs_id` cross-check and story 4.1's
/// round-trip test. It does not quietly see less **on the file path**: [`read_records`] refuses any
/// record after a terminal failure, so every observation it returns is reachable. That guarantee is
/// file-scoped — `FixtureConnector::from_records` deliberately admits observations after a failure
/// — so do not carry this reasoning to an in-memory stream.
///
/// A caller that needs to know what the poll DID must call [`read_records`].
pub fn read_jsonl(path: &Path) -> Result<Vec<Observation>, FixtureError> {
    Ok(read_records(path)?
        .into_iter()
        .filter_map(|record| match record {
            Record::Observation(observation) => Some(observation),
            // Exhaustive, no `_` arm: a new record kind must break THIS match and force a decision
            // about whether the observations-only view may keep ignoring it.
            Record::Failure(_) | Record::Capability(_) => None,
        })
        .collect())
}

/// Read a trap file, validate it, and check that every observation it judges actually exists in
/// the replay stream it names.
///
/// The cross-check is the point: a trap that points at an `obs_id` absent from its stream is a
/// trap that can never fire, and it would sit in the corpus looking like coverage. The gate
/// counts traps, so a trap that cannot fail is worse than no trap at all.
pub fn read_traps(path: &Path) -> Result<TrapFile, FixtureError> {
    let text = std::fs::read_to_string(path).map_err(|source| FixtureError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let traps: TrapFile = toml::from_str(&text).map_err(|source| FixtureError::Toml {
        path: path.to_path_buf(),
        source,
    })?;
    traps.validate().map_err(|source| FixtureError::Trap {
        path: path.to_path_buf(),
        source,
    })?;

    // One read per distinct replay stream, not one per trap.
    let mut streams: std::collections::BTreeMap<String, std::collections::BTreeSet<uuid::Uuid>> =
        std::collections::BTreeMap::new();
    for trap in &traps.trap {
        if !streams.contains_key(&trap.replay) {
            let stream = read_jsonl(&fixture_path(&trap.replay)?)?;
            // `Uuid`, not `String`: comparing formatted text would couple correctness to
            // `Display` (hyphenation, case) on both sides, and allocate per observation.
            let ids: std::collections::BTreeSet<uuid::Uuid> =
                stream.iter().map(|o| o.obs_id.as_uuid()).collect();
            streams.insert(trap.replay.clone(), ids);
        }
        let known = &streams[&trap.replay];
        for obs_id in &trap.observations {
            if !known.contains(&obs_id.as_uuid()) {
                return Err(FixtureError::DanglingObservation {
                    path: path.to_path_buf(),
                    trap: trap.id.0.clone(),
                    obs_id: obs_id.as_uuid().to_string(),
                    replay: trap.replay.clone(),
                });
            }
        }
    }
    Ok(traps)
}

/// Walk every `.jsonl` under `scenario/replay/`, recursively, refusing symlinks and any other
/// extension, and return how many were visited.
///
/// Recursive on purpose: the trap FAMILIES (4.9+) are what will introduce a subdirectory, and
/// a flat scan would hash them and never read them. Read errors are not swallowed — an
/// unreadable subtree shrinking the search space into a false green was a real defect in 4.1.
///
/// **Test-only, and `pub(crate)` since story 5.1.** It lives outside `mod tests` for one reason:
/// two claims about the corpus are made at two different LAYERS and must walk the same tree, or
/// "every stream" quietly means two different sets. That sentence is the whole reason for the
/// hoist; the caller list is deliberately NOT enumerated here — an inventory in a doc comment has
/// no guard behind it, and the first story to add a caller makes it false silently (story 5.1's
/// review).
///
/// It asserts its own non-emptiness, so a caller cannot pass vacuously by walking nothing. That
/// assertion used to be five verbatim copies at the call sites, whose identical message never said
/// WHICH walk found nothing (story 5.1's review). Proven red by suppressing the `checked`
/// increment below (story 5.1, mutation 11).
///
/// The paths it yields are ABSOLUTE (rooted through [`FIXTURES_DIR`], `..` components included).
/// A caller needing the corpus-RELATIVE spelling that [`fixture_path`] and `MANIFEST.toml` use
/// must derive it with `strip_prefix(fixtures_dir())` — never by writing the `fixtures/` prefix
/// again, which `the_fixtures_path_is_expressed_once` refuses.
#[cfg(test)]
pub(crate) fn walk_replay_streams(visit: &mut dyn FnMut(&Path)) -> usize {
    let root = fixture_path("scenario/replay").unwrap();
    let mut checked = 0usize;
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in
            std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
        {
            let entry = entry.expect("a directory entry must be readable");
            let path = entry.path();
            let file_type = entry.file_type().expect("file type");
            if file_type.is_symlink() {
                panic!(
                    "{}: the corpus must contain its own bytes, not a symlink",
                    path.display()
                );
            }
            // Tooling scratch is not corpus. `fixtures/scenario/replay/.claude/.cc-writes` already
            // exists in a working tree (created 2026-07-26, empty — which is the only reason the
            // suite stayed green), and the assertion below would have accused the CORPUS of a
            // defect the moment any tool wrote a file under it. Skipping dot-entries closes the
            // class rather than that one instance. The cost is named: a `.hidden.jsonl` is no
            // longer walked — acceptable because the corpus never hides an artefact, and
            // `MANIFEST.toml` lists every one by its visible name (story 5.1's review).
            // Observed on BOTH sides (story 5.1, mutation 12): with a file written under
            // `.claude/.cc-writes/`, the walk stays green WITH this skip and reds without it,
            // naming the scratch file — so the skip is what stands between a tooling artefact and
            // a panic accusing the corpus.
            if entry
                .file_name()
                .to_str()
                .is_some_and(|n| n.starts_with('.'))
            {
                continue;
            }
            if file_type.is_dir() {
                stack.push(path);
                continue;
            }
            // `README.md` is exempt at any depth, exactly as the corpus lock's orphan rule
            // exempts it (`xtask/src/main.rs`). Two gates that disagree about what the corpus
            // may contain would make documenting this directory red the test suite.
            if path.file_name().and_then(|n| n.to_str()) == Some("README.md") {
                continue;
            }
            let is_jsonl = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("jsonl"));
            assert!(
                is_jsonl,
                "{}: only .jsonl replay streams and README.md belong under scenario/replay/",
                path.display()
            );
            visit(&path);
            checked += 1;
        }
    }
    assert!(
        checked > 0,
        "no replay stream found under scenario/replay/ — every caller of this walk would \
         otherwise pass by proving nothing"
    );
    checked
}

/// Walk every `.toml` under `scenario/traps/`, recursively, in SORTED order, refusing symlinks
/// and any other extension, and return how many were visited.
///
/// **Test-only, and hoisted by story 5.2** out of `every_trap_file_in_the_corpus_is_valid`, for
/// the reason that hoisted the replay walk: two claims about the trap tree — that every file
/// parses, and that no file's TEXT carries a real address — are made at two layers and must walk
/// the same tree, or "every trap file" quietly means two different sets. A third hand-written walk
/// would have been accidental duplication.
///
/// The production walk `trap_gate::discover_trap_files` is deliberately NOT this function and is
/// not promoted into it: that one is the scoring harness's own discovery path, returns `Result`
/// rather than panicking, and its doc argues the separation. The two are held to the same RULES
/// (dot-entries skipped, `README.md` exempt at any depth, entry symlinks refused, foreign
/// extensions refused, sorted order) — that agreement is the point, not a shared body. **The
/// agreement is not total, and the two exceptions are named rather than implied:** the production
/// walk checks no ROOT symlink and refuses no non-file entry. Both are registered under
/// `deferred-work.md#Deferred from: code review of story-5.2` with an owner.
///
/// It asserts its own non-emptiness, so a caller cannot pass vacuously by walking nothing. That
/// is the shallow half of the vacuity question story 5.1's review raised; the deep half — a scan
/// over eleven files carrying zero ADDRESSES — counting files cannot reach, and is answered by the
/// caller's own coverage assertion.
///
/// Three defects the register recorded against `walk_replay_streams` are closed here rather than
/// inherited (`deferred-work.md#Deferred from: code review of story-5.1`): the ROOT is
/// symlink-checked, not only its entries; a non-file entry (a FIFO named `x.toml` would make
/// `read_to_string` block, with no diagnostic at all) is refused by name; and the yielded order is
/// SORTED, so with two broken files WHICH one panics does not vary per run.
///
/// **Closed HERE means closed in this walk, and the distinction is load-bearing:** the production
/// walk still has no `is_file()` refusal, so a FIFO under `scenario/traps/` continues to hang the
/// SUITE through `discover_trap_files`'s callers — this walk fails by name, and the six
/// `trap_gate` tests that drive the other one block. Measured by story 5.2's code review:
/// `timeout 90 cargo test -p opencmdb-bin` returns 143 with no output even with the guard below in
/// place; only a filtered run surfaces the named failure. Registered, not fixed here (story 5.2's
/// ACs scope it to the test walk).
///
/// The paths it yields are ABSOLUTE, exactly as [`walk_replay_streams`]'s are.
#[cfg(test)]
pub(crate) fn walk_trap_files(visit: &mut dyn FnMut(&Path)) -> usize {
    let root = fixture_path("scenario/traps").unwrap();
    // The root itself, not only what is under it: a walk whose doc says "refuses symlinks" while
    // its own starting point could be one is a doc comment asserting more than the code does.
    let root_meta =
        std::fs::symlink_metadata(&root).unwrap_or_else(|e| panic!("stat {}: {e}", root.display()));
    assert!(
        !root_meta.file_type().is_symlink(),
        "{}: the corpus must contain its own bytes, not a symlink",
        root.display()
    );
    let mut found = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        for entry in
            std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
        {
            let entry = entry.expect("a directory entry must be readable");
            let path = entry.path();
            // `file_type()` does not follow symlinks, so a link can neither smuggle a file in nor
            // be walked out of the corpus — but it must not pass unnoticed either.
            let file_type = entry.file_type().expect("file type");
            if file_type.is_symlink() {
                panic!(
                    "{}: the corpus must contain its own bytes, not a symlink",
                    path.display()
                );
            }
            // Tooling scratch is not corpus — see `discover_trap_files` for the measurement that
            // put this line in both trap walks (story 5.2), and `walk_replay_streams` for the
            // replay-tree twin story 5.1 landed.
            if entry
                .file_name()
                .to_str()
                .is_some_and(|n| n.starts_with('.'))
            {
                continue;
            }
            if file_type.is_dir() {
                stack.push(path);
                continue;
            }
            // `README.md` is exempt at any depth, exactly as the corpus lock's orphan rule exempts
            // it (`xtask/src/main.rs`) and as both sibling walks do — documenting this directory
            // (e.g. the reality-debt register) must not red the test suite.
            if path.file_name().and_then(|n| n.to_str()) == Some("README.md") {
                continue;
            }
            // Neither a file nor a directory: a FIFO named `x.toml` passes the extension check and
            // then blocks `read_to_string` forever, so THIS walk's callers would hang instead of
            // failing. The register calls that "the one failure mode with no diagnostic at all".
            // It refuses the entry here only — `discover_trap_files` has no such guard, so the
            // suite still hangs through its six callers (registered, story 5.2's review).
            assert!(
                file_type.is_file(),
                "{}: only regular files belong under scenario/traps/",
                path.display()
            );
            // Case-insensitive: a `broken.TOML` skipped silently would be hashed by the gate and
            // read by nobody.
            let is_toml = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("toml"));
            assert!(
                is_toml,
                "{}: only .toml trap files belong under scenario/traps/",
                path.display()
            );
            found.push(path);
        }
    }
    // Sorted so that with two broken files, WHICH one panics is the same on every run — the
    // property `discover_trap_files` already has, and the reason the two walks now agree.
    found.sort();
    for path in &found {
        visit(path);
    }
    assert!(
        !found.is_empty(),
        "no trap file found under scenario/traps/ — every caller of this walk would otherwise \
         pass by proving nothing"
    );
    found.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencmdb_core::gap::AbstentionCause;
    use opencmdb_core::identity::blocking::{
        BLOCKING_RECALL_FLOOR_PER_MILLE, CandidatePair, L2CandidatePair, blocking_recall_per_mille,
        candidates, l2_candidates,
    };
    use opencmdb_core::identity::l1::{L1Key, join};
    use opencmdb_core::observation::{
        ConnectorId, Fact, HostnameSource, L2DomainId, MacAddr, ObsId, Scope, Timestamp, VantageId,
    };
    use opencmdb_core::trap::{Expectation, RuleId};
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use uuid::Uuid;

    const MINIMAL: &str = "scenario/replay/minimal.jsonl";

    fn ts(s: &str) -> Timestamp {
        chrono::DateTime::parse_from_rfc3339(s)
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    fn u(s: &str) -> Uuid {
        Uuid::parse_str(s).unwrap()
    }

    /// The corpus, written out in Rust. The test is the second, independent statement of what
    /// the committed bytes mean — if serde's representation ever shifts under us, these two
    /// disagree and the corpus stops silently meaning something else.
    fn expected() -> Vec<Observation> {
        let scope = Scope {
            l2_domain: L2DomainId::from_uuid(u("11111111-1111-4111-8111-111111111111")),
            vantage: VantageId::from_uuid(u("22222222-2222-4222-8222-222222222222")),
        };
        let connector_id = ConnectorId::from_uuid(u("33333333-3333-4333-8333-333333333333"));
        vec![
            Observation {
                obs_id: ObsId::from_uuid(u("aaaaaaaa-0000-4000-8000-000000000001")),
                connector_id,
                observed_at: ts("2026-01-01T00:00:00Z"),
                scope,
                facts: vec![
                    Fact::Mac {
                        addr: MacAddr::from_str("02:00:5e:00:53:01").unwrap(),
                        locally_administered: true,
                    },
                    Fact::IpV4 {
                        addr: Ipv4Addr::new(192, 0, 2, 10),
                    },
                    Fact::Hostname {
                        name: "doc-host-a".into(),
                        source: HostnameSource::Dhcp,
                    },
                ],
                raw: None,
            },
            Observation {
                obs_id: ObsId::from_uuid(u("aaaaaaaa-0000-4000-8000-000000000002")),
                connector_id,
                observed_at: ts("2026-01-01T00:00:05Z"),
                scope,
                facts: vec![
                    Fact::IpV4 {
                        addr: Ipv4Addr::new(192, 0, 2, 11),
                    },
                    Fact::Rtt { millis: 7 },
                ],
                raw: None,
            },
            Observation {
                obs_id: ObsId::from_uuid(u("aaaaaaaa-0000-4000-8000-000000000003")),
                connector_id,
                observed_at: ts("2026-01-01T00:00:10Z"),
                scope,
                facts: vec![
                    Fact::Mac {
                        addr: MacAddr::from_str("02:00:5e:00:53:02").unwrap(),
                        locally_administered: true,
                    },
                    Fact::OuiVendor {
                        vendor: String::new(),
                    },
                ],
                raw: Some("{\"provenance\":\"never read by a decision\"}".into()),
            },
        ]
    }

    #[test]
    fn the_committed_fixture_reads_back_exactly() {
        let observations =
            read_jsonl(&fixture_path(MINIMAL).unwrap()).expect("the fixture must read");
        assert_eq!(observations, expected());
    }

    /// Re-serializing must reproduce the committed bytes. This is what freezes the FORMAT and
    /// not merely the values: a serde rename would still round-trip in memory while silently
    /// changing what every future trap file means.
    ///
    /// **Not a duplicate of `every_replay_stream_re_serializes_to_its_committed_bytes`, and a
    /// future DRY pass must not collapse the two.** This one starts from `expected()` — an
    /// independently authored Rust literal — so it pins the VALUES as well as the shape, over one
    /// file. The corpus-wide witness starts from the FILE, so it pins the shape only, over all
    /// fifteen. Deliberate redundancy of exactly the kind the house DRY rule protects.
    #[test]
    fn re_serializing_reproduces_the_committed_bytes() {
        let path = fixture_path(MINIMAL).unwrap();
        let on_disk = std::fs::read_to_string(&path).expect("the fixture must exist");
        let mut rendered = String::new();
        for observation in expected() {
            rendered.push_str(&serde_json::to_string(&observation).expect("serialize"));
            rendered.push('\n');
        }
        assert_eq!(rendered, on_disk);
    }

    /// Render a parsed [`Record`] back to the line it came from.
    ///
    /// [`Record`] holds a [`ConnectorError`] / [`Capabilities`] directly, NOT a [`ControlRecord`];
    /// the on-disk shape is the latter, so rendering re-wraps. Both inner types are `Clone`.
    fn render_record(record: &Record) -> String {
        match record {
            Record::Observation(observation) => {
                serde_json::to_string(observation).expect("an observation serializes")
            }
            // Exhaustive, no `_` arm: a new record kind must break THIS match rather than be
            // rendered as nothing and silently pass the round-trip.
            Record::Failure(error) => serde_json::to_string(&ControlRecord::Failure {
                error: error.clone(),
            })
            .expect("a failure record serializes"),
            Record::Capability(capabilities) => serde_json::to_string(&ControlRecord::Capability {
                capabilities: capabilities.clone(),
            })
            .expect("a capability record serializes"),
        }
    }

    /// EVERY stream under `scenario/replay/` round-trips to its committed bytes, line by line —
    /// observations and CONTROL records alike.
    ///
    /// "Every stream under `scenario/replay/`", not "every committed stream": the corpus also holds
    /// `scenario/wire/unifi-clients.expected.jsonl`, a committed `.jsonl` of observations that sits
    /// outside every corpus walk on purpose and has no round-trip pin at all (registered under
    /// story 5.1's review, owned by Epic 11's parser). Widening the walk to reach it is not a
    /// tidy-up — it is a scope change.
    ///
    /// Until story 5.1 only `minimal.jsonl` had this, so no other stream and no control record at
    /// all had its exact serialized byte-shape pinned — field order, the `MacAddr` array encoding,
    /// `Uplink`'s field names, the internally-tagged `record` marker (registered under story 4.10's
    /// review). A serde rename anywhere in the domain types would have kept every stream parsing
    /// while changing what the corpus MEANS.
    ///
    /// What it pins is the SHAPE, never the authored values: it starts from the file, so a stream
    /// re-authored with different-but-well-formed values round-trips happily. Value pins are each
    /// family's byte-pin test, and four families still have none (register,
    /// `deferred-work.md#Deferred from: story-5.1`, owned by story 5.2b).
    #[test]
    fn every_replay_stream_re_serializes_to_its_committed_bytes() {
        walk_replay_streams(&mut |path| {
            let text = std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
            // The bytes BETWEEN and AFTER the lines, which the line-by-line comparison below
            // cannot see: `str::lines()` strips a trailing `\r` as well as the `\n`, so without
            // these two a stream re-authored with CRLF endings or with its final newline dropped
            // round-tripped green (story 5.1's review). Only `minimal.jsonl` had whole-file byte
            // equality, via `re_serializing_reproduces_the_committed_bytes`. The sha256 lock is
            // not the backstop for this: the threat model is a DELIBERATE re-authoring, which
            // refreshes `MANIFEST.toml` by definition. Both proven red on `dhcp-churn.jsonl`
            // (story 5.1, mutations 8 and 9) — final newline truncated, then LF converted to CRLF.
            assert!(
                text.ends_with('\n'),
                "{}: a committed stream must be newline-terminated",
                path.display()
            );
            assert!(
                !text.contains('\r'),
                "{}: a committed stream must use LF endings, never CR or CRLF",
                path.display()
            );
            let records = read_records(path).expect("a corpus stream must read");

            // `read_records` discards line numbers, and it SKIPS truly empty lines while still
            // counting them, so a positional zip against `records` would report the wrong number
            // the day a stream carries a blank line (none does today, which is precisely why a
            // positional zip would be invisibly wrong). Re-derive the raw 1-indexed number here.
            // The filter is `trim`-based while `read_records` skips only `is_empty()` lines: the
            // two agree on every file that parses, because a whitespace-only line is not skipped
            // there — it fails to parse and the `expect` above has already panicked.
            let mut committed = text
                .lines()
                .enumerate()
                .filter(|(_, line)| !line.trim().is_empty());

            for record in &records {
                let (index, line) = committed.next().unwrap_or_else(|| {
                    panic!(
                        "{}: more records than non-blank lines — the two iterators must exhaust \
                         together",
                        path.display()
                    )
                });
                assert_eq!(
                    render_record(record),
                    line,
                    "{}:{}: re-serializing does not reproduce the committed bytes",
                    path.display(),
                    index + 1
                );
            }
            if let Some((index, _)) = committed.next() {
                panic!(
                    "{}:{}: more non-blank lines than records — the two iterators must exhaust \
                     together",
                    path.display(),
                    index + 1
                );
            }
            assert!(
                !records.is_empty(),
                "{}: a committed stream with no record proves nothing",
                path.display()
            );
        });
    }

    /// Every value is synthetic: RFC 5737 addresses and locally-administered MACs. A real
    /// capture in a public repository is disqualifying (D19).
    ///
    /// This reads the COMMITTED FILES, not `expected()`. The whole argument of this module is
    /// that the file is the spec — a privacy guard that inspects a Rust literal would stay green
    /// while someone pasted a real MAC into the corpus.
    ///
    /// It WALKS the corpus rather than naming one file. Until story 4.5a it read `minimal.jsonl`
    /// alone, so every other committed stream was locked by sha256 and inspected by nobody — a
    /// privacy rule that cannot see the file it governs is not a rule.
    ///
    /// It reads RECORDS, not observations. `read_jsonl` drops control records, so routing the
    /// privacy rule through it would leave a failure's hand-authored `detail` — free text, and the
    /// obvious place a real hostname or address would land — inspected by nothing. That blind spot
    /// was introduced by the very story that added control records, and found by the review.
    #[test]
    fn the_corpus_carries_no_real_network_data() {
        walk_replay_streams(&mut |path| {
            for record in read_records(path).expect("a corpus stream must read") {
                assert_record_is_synthetic(&record, path);
            }
        });
    }

    /// The privacy rule applied to ONE record, whichever kind it is.
    ///
    /// Extracted from the walk above by story 5.2 so the `raw` guard can drive exactly the code
    /// the corpus walk drives, rather than a re-typed copy of it that could agree with a bug. The
    /// walk's fixed root is what forced the extraction: `walk_replay_streams` hardcodes
    /// `scenario/replay`, that root is load-bearing for story 5.1's callers, and a `scratch_dir`
    /// tree therefore cannot be walked into the rule.
    ///
    /// The `match` stays exhaustive with NO `_` arm, which is the property the extraction had to
    /// preserve: a new `Record` variant must break THIS match and force a privacy decision rather
    /// than slip past. Story 4.5b's capability record did exactly that, and the decision is its
    /// arm below.
    fn assert_record_is_synthetic(record: &Record, path: &Path) {
        match record {
            Record::Observation(observation) => {
                assert_facts_are_synthetic(&observation.facts, path);
                // `raw` is opaque provenance that no decision reads (D19) — which is precisely why
                // it went unscanned until story 5.2: the walk passed only `facts` to the rule, so
                // the one field whose whole purpose is "whatever the source sent" was the one
                // field a pasted capture could land in unseen.
                //
                // **This call site is VACUOUS on today's committed corpus, and saying so is the
                // point.** Across all 15 replay streams and the wire artefact, exactly ONE
                // observation carries a non-null `raw` — `minimal.jsonl` line 3,
                // `{"provenance":"never read by a decision"}` — and it holds no address. So no
                // committed `raw` currently exercises this; deleting this line reds nothing in the
                // corpus. That is exactly why it does not defend itself and ships with a permanent
                // guard, `an_observations_raw_payload_is_scanned`, whose mutation is record-side
                // BECAUSE the corpus has no `raw` to break.
                if let Some(raw) = &observation.raw {
                    assert_text_is_synthetic(raw, path);
                }
            }
            Record::Failure(error) => {
                assert_text_is_synthetic(&error.to_string(), path);
            }
            // A capability record carries a timestamp and a set of `FactKind` enum values —
            // no free text, no address, nothing an author can type a real hostname into.
            // Nothing to scan, stated rather than skipped.
            Record::Capability(_) => {}
        }
    }

    /// The addresses one text scan actually inspected, in the order it met them.
    ///
    /// Returned rather than tallied privately because a scan that finds NOTHING is vacuous and
    /// its caller cannot tell. Counting FILES does not catch it — that is the level story 5.1's
    /// review reached, and the level below it is a scan over eleven files carrying zero addresses.
    /// `walk_trap_files`'s caller asserts on these values.
    #[derive(Default)]
    struct ScannedText {
        /// Every IPv4 address the scan recognised; duplicates kept, so the caller decides
        /// whether it cares about occurrences or distinct values.
        ips: Vec<Ipv4Addr>,
        /// Every MAC the scan recognised; duplicates kept, same reason.
        macs: Vec<MacAddr>,
    }

    /// The longest text an address can occupy: `00:11:22:33:44:55` is 17 bytes, and
    /// `Ipv4Addr::from_str` refuses leading zeros so `255.255.255.255` (15) is its ceiling.
    /// Bounding the longest-match search by it keeps the scan linear in the text rather than
    /// quadratic in a long hex run.
    const LONGEST_ADDRESS: usize = 17;

    /// Free text authored by a fixture author must carry no real address.
    ///
    /// It scans for anything that PARSES as an IPv4 address or a MAC and holds it to the same rule
    /// as a structured fact. That is deliberately narrower than "no private data" — a hostname in
    /// prose cannot be recognised mechanically — so it is a floor, not a proof. The register
    /// carries what it does not cover.
    ///
    /// **The tokenizer is boundary-anchored longest-match** (story 5.2). It normalises `-` to `:`,
    /// splits into maximal runs of `[0-9a-fA-F.:]`, and inside a run tries a candidate only at the
    /// run start or immediately after a `.` or `:`, taking the LONGEST prefix that parses and
    /// resuming after it. That shape is what sees an address wearing punctuation on either side
    /// (`198.18.0.1.`, `00:11:22:33:44:55:`), an INTERIOR separator (`198.18.0.1:8080`) and the
    /// dash form (`00-11-22-33-44-55`) — three evasions the previous split-on-punctuation
    /// tokenizer let through, each observed green before it was closed.
    ///
    /// **Enumerating every SUBSTRING instead reds the committed corpus** — `Ipv4Addr::from_str`
    /// rejects only leading zeros, so `92.0.2.90` parses out of the documentation address
    /// `192.0.2.90`. That is not a citation: dropping the resume below and advancing one byte at a
    /// time was run, and `the_wire_spec_encodes_the_measured_field_behaviours` reds naming
    /// `92.0.2.90` (story 5.2, mutation 6).
    ///
    /// **Which of the two conjuncts earns that, measured rather than assumed:** it is the RESUME
    /// (`i += matched.max(1)`), not the start anchor — with longest-match-and-resume in place, an
    /// interior start inside an address is never reached anyway. Removing the anchor alone was
    /// observed to leave the whole suite green **at the time it was run, 127 tests, before the
    /// blindness guard below existed** (story 5.2, mutation 5); in the delivered tree it reds that
    /// guard and nothing else (mutation 7). The anchor is kept as specified because it is what
    /// bounds the scan to address-shaped positions rather than sliding through arbitrary hex, but
    /// its contribution to CORPUS safety is unfalsifiable and is not claimed.
    ///
    /// **What remains a floor, named rather than elided.** The list is longer than the anchor's own
    /// limit, and a short list read as a complete one is how an owner stops being assigned:
    /// - a hostname in prose still cannot be recognised mechanically;
    /// - an address glued to a HEXDIGIT prefix is invisible — `ab198.18.0.1`, but equally
    ///   `1198.18.0.1`, since the rule is "neither a run start nor preceded by `.` or `:`" and a
    ///   digit is as much a hexdigit as a letter. Pinned by
    ///   `the_text_scanner_is_blind_to_an_address_glued_to_hex`, so the sentence you are reading
    ///   has a check behind it;
    /// - **IPv6 is not scanned at all.** Only `Ipv4Addr` and `MacAddr` are attempted, so an IPv6
    ///   literal — pure hex and colons, collected as a run and then discarded — passes clean. That
    ///   matters most on `Observation.raw`, the field whose whole purpose is "whatever the source
    ///   sent"; and this rule's own multicast argument below invokes real IPv6 interface-identifier
    ///   bytes as the thing worth refusing;
    /// - **zero-padded IPv4** (`010.001.002.003`) is invisible, the mirror image of the
    ///   leading-zero rejection that makes the substring route unusable above;
    /// - **MAC notations other than colon and dash** — the Cisco dotted form (`0011.2233.4455`,
    ///   which every IOS/Aruba/HP CLI emits) and the bare form (`001122334455`) — are the same
    ///   address the dash row closes, in a shape `MacAddr::from_str` cannot read;
    /// - **the resume can swallow a real address adjacent to an accepted one.** Longest-match is
    ///   committed and never backtracks, so `0a:00:11:22:33:44:55` matches the synthetic
    ///   `0a:00:11:22:33:44` and skips the vendor MAC starting three bytes in, and
    ///   `192.0.2.110.0.0.1` matches the documentation `192.0.2.110` and skips `10.0.0.1`. This one
    ///   is a limit of the mechanism story 5.2 introduced, not an inherited one.
    ///
    /// All of the above are registered under `deferred-work.md#Deferred from: code review of
    /// story-5.2`. None is closed here; the story's title is a direction, not a completion claim.
    fn assert_text_is_synthetic(text: &str, path: &Path) -> ScannedText {
        let where_ = path.display();
        let mut seen = ScannedText::default();
        // `-` separates a MAC in the wild but SPLIT tokens here, so the dash form shattered into
        // six two-character fragments before anything could parse it. Normalising in the scanner
        // keeps `MacAddr::from_str` colon-only: widening a domain parser for a test's convenience
        // is a frontier violation (D47) and would change what the shipped connectors accept off
        // the wire.
        let normalised = text.replace('-', ":");
        for run in normalised.split(|c: char| !(c.is_ascii_hexdigit() || c == '.' || c == ':')) {
            // Every byte of a run is ASCII by construction of the predicate above, so byte
            // indices and char boundaries coincide and `run[i..end]` cannot split a character.
            let bytes = run.as_bytes();
            let mut i = 0usize;
            while i < bytes.len() {
                if i > 0 && bytes[i - 1] != b'.' && bytes[i - 1] != b':' {
                    i += 1;
                    continue;
                }
                let mut matched = 0usize;
                let ceiling = bytes.len().min(i + LONGEST_ADDRESS);
                for end in (i + 1..=ceiling).rev() {
                    let candidate = &run[i..end];
                    if let Ok(mac) = MacAddr::from_str(candidate) {
                        assert!(
                            is_synthetic_mac(mac),
                            "{where_}: free text names {mac}, which is {}",
                            mac_refusal_reason(mac)
                        );
                        seen.macs.push(mac);
                        matched = end - i;
                        break;
                    }
                    if let Ok(addr) = candidate.parse::<Ipv4Addr>() {
                        assert_documentation_ip(addr, path);
                        seen.ips.push(addr);
                        matched = end - i;
                        break;
                    }
                }
                i += matched.max(1);
            }
        }
        seen
    }

    /// The one rule saying which MAC bytes may be committed, shared by every site that checks a
    /// MAC (`Fact::Mac`, `Uplink::peer_mac`, free text) — the invariant is byte-level, not
    /// position-level: an allowance on the bytes cannot be a hole in one position and not
    /// another.
    ///
    /// The invariant was never "U/L bit set" — that was the approximation. It is "no committed
    /// byte can identify a real network". Two byte shapes satisfy it: a locally-administered
    /// UNICAST address (the synthetic corpus idiom), and the IANA VRRP IPv4 virtual-router block
    /// `00:00:5e:00:01:xx` — a PROTOCOL address, identical on every VRRP deployment on earth
    /// with that VRID, the MAC analog of an RFC 5737 documentation IP (story 4.14). The list is
    /// CLOSED and 5-octet exact; a new range (HSRP's `00:00:0c:07:ac` is a Cisco OUI, not IANA)
    /// enters only alongside a committed fixture that exercises it, with its own prove-to-red.
    ///
    /// **Multicast is refused whatever its U/L bit says** (story 5.2). A multicast address names
    /// no interface, so "locally administered" tells nothing about whether its bytes came from a
    /// real network — and an IPv6 solicited-node multicast MAC (`33:33:ff:xx:xx:xx`) embeds the
    /// low three bytes of a real IPv6 address, i.e. real interface-identifier bytes, while
    /// wearing a set U/L bit that admitted it. `01:00:5e:…` was already refused and still is by the
    /// same leg as before — its U/L bit is clear, so `&&` short-circuits and the I/G test never
    /// runs on it. What the two now share is the stated REASON, not the branch that refuses them:
    /// `mac_refusal_reason` gives both the multicast sentence. Measured
    /// against every committed MAC before the change — 39 distinct addresses across `Fact::Mac`
    /// and `Uplink::peer_mac` in all 14 committed `.jsonl` files — **not one has the I/G bit set**,
    /// so the tightening reds no committed byte.
    fn is_synthetic_mac(addr: MacAddr) -> bool {
        (addr.is_locally_administered() && !is_multicast_mac(addr))
            || addr.0[..5] == [0, 0, 94, 0, 1]
    }

    /// The I/G bit — bit 0 of the first octet. Set means the address is a group (multicast or
    /// broadcast) address, which names no interface.
    fn is_multicast_mac(addr: MacAddr) -> bool {
        addr.0[0] & 1 == 1
    }

    /// Why a MAC is refused, as the clause following *"which is …"* / *"… is …"*.
    ///
    /// One function so the two refusal sites cannot drift apart, and SPLIT because the tightening
    /// above made the single old sentence false: `33:33:…` IS locally administered and is refused
    /// for being multicast, so saying it is *"neither locally administered nor …"* would be a
    /// message asserting something untrue — held to the same bar as a doc comment. The
    /// non-multicast wording is kept verbatim from before the split, so the guard that pins it
    /// (`the_text_scanner_still_refuses_a_mac_outside_the_block`) still matches and was not
    /// quietly re-pointed.
    fn mac_refusal_reason(addr: MacAddr) -> &'static str {
        if is_multicast_mac(addr) {
            "a MULTICAST address and names no interface — an IPv6 solicited-node multicast MAC \
             embeds real interface-identifier bytes, so a set U/L bit says nothing about it"
        } else {
            "neither locally administered nor in the IANA VRRP virtual-router range \
             00:00:5e:00:01:xx — a real vendor address must never be committed"
        }
    }

    /// The privacy rule itself, applied to one observation's facts. `path` is carried so a
    /// failure names WHICH committed stream broke the rule — with the walk, "a real MAC is in the
    /// corpus" is not actionable unless it says where.
    fn assert_facts_are_synthetic(facts: &[Fact], path: &Path) {
        let where_ = path.display();
        for fact in facts {
            // Exhaustive on purpose — no `_` arm. `Fact` is `#[non_exhaustive]`, so a new
            // variant carrying an address must break THIS test and force a decision, rather
            // than slipping past a catch-all that asserts nothing.
            match fact {
                Fact::IpV4 { addr } => assert_documentation_ip(*addr, path),
                Fact::DhcpLease { ip, .. } => assert_documentation_ip(*ip, path),
                Fact::Mac {
                    addr,
                    locally_administered,
                } => {
                    assert_synthetic_mac(*addr, path);
                    // The serde flag is the connector's READING; the bytes are the ground truth
                    // (`MacAddr::is_locally_administered`'s doc names this cross-check as its
                    // purpose). Until story 4.14 every corpus flag was `true`, so a lying flag
                    // was unobservable — the day the first honest `false` entered, this became
                    // an invariant worth holding.
                    assert_eq!(
                        *locally_administered,
                        addr.is_locally_administered(),
                        "{where_}: {addr}'s authored locally_administered flag contradicts its \
                         own U/L bit"
                    );
                }
                Fact::Uplink { peer_mac, .. } => assert_synthetic_mac(*peer_mac, path),
                // An EMPTY name is trivially synthetic — it identifies nothing — and it is one
                // of the two shapes the measured source actually produces for "no hostname"
                // (MISSING and empty, never null), so the corpus must be able to commit it
                // (story 4.17). Whitespace-only names stay refused, deliberately: the
                // measurement records `""`, not padding.
                Fact::Hostname { name, .. } => assert!(
                    name.is_empty() || name.starts_with("doc-"),
                    "{where_}: hostnames must be invented (doc-…) or honestly empty, not \
                     captured: {name}"
                ),
                Fact::OuiVendor { .. } | Fact::Rtt { .. } => {}
                other => panic!(
                    "{where_}: a new Fact variant reached the corpus with no privacy rule: {other:?}"
                ),
            }
        }
    }

    /// RFC 5737 reserves three ranges for documentation. Accepting only one and blaming the
    /// standard in the message would send a future author looking for a defect that is not there.
    fn assert_documentation_ip(addr: Ipv4Addr, path: &Path) {
        let where_ = path.display();
        let o = addr.octets();
        let documentation = matches!(
            [o[0], o[1], o[2]],
            [192, 0, 2] | [198, 51, 100] | [203, 0, 113]
        );
        assert!(
            documentation,
            "{where_}: {addr} is not in an RFC 5737 documentation range \
             (192.0.2.0/24, 198.51.100.0/24, 203.0.113.0/24)"
        );
    }

    /// The structured-fact refusal site. It shares `mac_refusal_reason` with the free-text
    /// scanner so the two can never disagree about WHY a byte pattern is out.
    fn assert_synthetic_mac(addr: MacAddr, path: &Path) {
        let where_ = path.display();
        assert!(
            is_synthetic_mac(addr),
            "{where_}: {addr} is {}",
            mac_refusal_reason(addr)
        );
    }

    /// A line that cannot be parsed is named, never skipped: a corpus that drops what it does
    /// not understand cannot be an oracle.
    #[test]
    fn a_malformed_line_names_its_line_number() {
        let dir = scratch_dir("malformed");
        let path = dir.join("broken.jsonl");
        let good = serde_json::to_string(&expected()[0]).unwrap();
        std::fs::write(&path, format!("{good}\n\n{{ not json\n")).unwrap();

        let err = read_jsonl(&path).expect_err("a malformed line must fail the read");
        match &err {
            FixtureError::Line { lineno, .. } => {
                assert_eq!(*lineno, 3, "blank lines do not shift the count")
            }
            other => panic!("expected a line error, got {other:?}"),
        }
        // The message must carry the file and the underlying reason, not just a number.
        let rendered = err.to_string();
        assert!(rendered.contains("broken.jsonl"), "{rendered}");
        assert!(std::error::Error::source(&err).is_some());
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A whitespace-only line carries content, so it must be named rather than skipped.
    #[test]
    fn a_whitespace_only_line_is_not_silently_skipped() {
        let dir = scratch_dir("whitespace");
        let path = dir.join("spaces.jsonl");
        let good = serde_json::to_string(&expected()[0]).unwrap();
        std::fs::write(&path, format!("{good}\n   \n")).unwrap();

        match read_jsonl(&path).expect_err("a whitespace-only line must fail the read") {
            FixtureError::Line { lineno, .. } => assert_eq!(lineno, 2),
            other => panic!("expected a line error, got {other:?}"),
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A missing file is an `Io` error that names the path — the variant the reader's own
    /// acceptance criterion promised and nothing exercised.
    #[test]
    fn a_missing_file_is_an_io_error_naming_the_path() {
        let path = scratch_dir("missing").join("absent.jsonl");
        let err = read_jsonl(&path).expect_err("a missing file must fail the read");
        assert!(matches!(err, FixtureError::Io { .. }), "{err:?}");
        assert!(err.to_string().contains("absent.jsonl"), "{err}");
        assert!(std::error::Error::source(&err).is_some());
    }

    /// The reader must not be more permissive than the lock that guards it: the MANIFEST parser
    /// refuses absolute paths and `..`, so the reader refuses them too.
    #[test]
    fn a_path_leaving_the_corpus_is_refused() {
        for escaping in [
            "/etc/passwd",
            "../../etc/passwd",
            "scenario/../../outside.jsonl",
        ] {
            let err = fixture_path(escaping)
                .expect_err("a path leaving the corpus must be refused: {escaping}");
            assert!(matches!(err, FixtureError::OutsideCorpus { .. }), "{err:?}");
        }
        assert!(fixture_path("scenario/replay/minimal.jsonl").is_ok());
    }

    /// A private scratch directory per test. A shared constant path races between concurrent
    /// `cargo test` runs and panics as a parser failure when it is owned by another user.
    /// Who claimed a scratch tag. Two helpers share one namespace and one of them deletes.
    ///
    /// 🔴 **A REPRODUCED CANDIDATE CAUSE FOR ISSUE #38, and the guard the original refutation
    /// never ran.** `read_scratch` and `write_traps` both key their directory on
    /// `(pid, tag)` — so the same tag from the two of them is the SAME directory — and
    /// a test that cleans up finish with `remove_dir_all`. Run in parallel test threads, that
    /// cleanup can land between `write_traps`' `create_dir_all` and its `fs::write`, giving
    /// `Os { code: 2, kind: NotFound }` at the write. Story 6b.12's validation reproduced it once
    /// in ten full-suite runs — the rate issue #38 itself records — with the tag `"both"` claimed
    /// by both helpers.
    ///
    /// ⚠️ **`CLAUDE.md` files that hypothesis as *"raised and refuted"*, and the refutation
    /// checked that *the six `write_traps` tags are distinct from each other*** — it never
    /// compared a `write_traps` tag with a `read_scratch` one. ***It measured one half of the
    /// population.*** This map is the other half: a tag may be claimed by exactly ONE helper, and
    /// a second claimant panics naming both. Together with the distinctness the original
    /// refutation did establish, the pair covers the namespace.
    ///
    /// ⚠️ **This does not close issue #38.** One reproduced occurrence establishes *a* cause,
    /// never *the* cause — this project forbids naming a cause without the check that would
    /// refute it, and the issue stays open with the measurement attached.
    fn scratch_owners() -> &'static std::sync::Mutex<std::collections::BTreeMap<String, String>> {
        static OWNERS: std::sync::OnceLock<
            std::sync::Mutex<std::collections::BTreeMap<String, String>>,
        > = std::sync::OnceLock::new();
        OWNERS.get_or_init(|| std::sync::Mutex::new(std::collections::BTreeMap::new()))
    }

    #[track_caller]
    fn scratch_dir(tag: &str) -> PathBuf {
        // 🔴 **The owner is the CALL SITE, and the first form used the HELPER'S NAME — which the
        // code review's edge layer measured insufficient by REPRODUCING the race through it.**
        // With `"read_scratch"` as the owner, two different tests reusing one tag through that
        // one helper claimed the same directory, the first one's cleanup deleted it, and the bare
        // `Os { code: 2, kind: NotFound }` came back **with the registry silent** — the very
        // failure it exists to name. Its doc promised *"a tag may be claimed by exactly ONE"*,
        // which was true only ACROSS helpers. `#[track_caller]` here and on both helpers makes
        // the owner a file and a line, so two tests cannot share a tag by any route.
        let caller = std::panic::Location::caller();
        let owner = format!("{}:{}", caller.file(), caller.line());
        {
            // ⚠️ Poison is RECOVERED rather than propagated: the panic below is itself a
            // poisoning event, and an `expect` here turned one real defect into EIGHTEEN failing
            // tests — measured. A guard that multiplies one finding by ten is noise around it.
            let mut owners = scratch_owners()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match owners.get(tag) {
                Some(first) if *first != owner => panic!(
                    "the scratch tag {tag:?} is claimed by BOTH {first} and {owner} — they \
                     resolve to one directory keyed on (pid, tag), and a test that cleans up \
                     deletes it under the other. Give one of them a tag of its own."
                ),
                _ => {
                    owners.insert(tag.to_string(), owner.clone());
                }
            }
        }
        let dir =
            std::env::temp_dir().join(format!("opencmdb-fixtures-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    // ── Record dispatch and the failure record (story 4.5a) ──────────────────

    /// Write a scratch stream and read it back as records.
    #[track_caller]
    fn read_scratch(tag: &str, body: &str) -> (PathBuf, Result<Vec<Record>, FixtureError>) {
        let dir = scratch_dir(tag);
        let path = dir.join("stream.jsonl");
        std::fs::write(&path, body).unwrap();
        let result = read_records(&path);
        (dir, result)
    }

    /// One valid observation line, so every dispatch test can put its offending line SECOND.
    /// With the offender first, a reader that stopped after line 1 would pass every one of them.
    fn good_line() -> String {
        serde_json::to_string(&expected()[0]).unwrap()
    }

    const UNREACHABLE: &str = r#"{"record":"failure","error":{"Unreachable":{"detail":"no route to the documentation net"}}}"#;

    /// A failure record is a record, and `read_jsonl` drops it for the callers that only wanted
    /// observations.
    #[test]
    fn a_failure_record_ends_the_stream_and_read_jsonl_ignores_it() {
        let (dir, result) = read_scratch("failure", &format!("{}\n{UNREACHABLE}\n", good_line()));
        let records = result.expect("a trailing failure record is admissible");
        assert_eq!(records.len(), 2);
        assert!(matches!(records[0], Record::Observation(_)));
        match &records[1] {
            Record::Failure(ConnectorError::Unreachable { detail }) => {
                assert!(detail.contains("no route"), "{detail}")
            }
            other => panic!("expected an Unreachable failure, got {other:?}"),
        }
        let observations = read_jsonl(&dir.join("stream.jsonl")).expect("observations read");
        assert_eq!(observations, vec![expected()[0].clone()]);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// The message story 4.1 froze must be the message an observation still gets. This is the
    /// case an absence-based discriminator inverts: no `obs_id` key would route this line to the
    /// control parser and blame a record the author never wrote.
    #[test]
    fn an_observation_line_with_a_misspelled_field_still_reports_its_own_error() {
        let typo = r#"{"obs_id":"aaaaaaaa-0000-4000-8000-0000000000ff","connector_id":"33333333-3333-4333-8333-333333333333","observed_at":"2026-01-01T00:00:00Z","scope":{"l2_domain":"11111111-1111-4111-8111-111111111111","vantage":"22222222-2222-4222-8222-222222222222"},"factz":[],"raw":null}"#;
        let (dir, result) = read_scratch("obs-typo", &format!("{}\n{typo}\n", good_line()));
        let err = result.expect_err("a misspelled field must be refused");
        match &err {
            FixtureError::Line { lineno, .. } => assert_eq!(*lineno, 2),
            other => panic!("expected an observation line error, got {other:?}"),
        }
        assert!(
            err.to_string().contains("factz"),
            "must name the field: {err}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// …and a control record with a misspelled field is just as precise, on the control record.
    #[test]
    fn a_control_record_with_a_misspelled_field_names_the_control_record() {
        let typo = r#"{"record":"failure","errro":{"Timeout":null}}"#;
        let (dir, result) = read_scratch("ctl-typo", &format!("{}\n{typo}\n", good_line()));
        let err = result.expect_err("a misspelled field must be refused");
        match &err {
            FixtureError::ControlRecordLine { lineno, .. } => assert_eq!(*lineno, 2),
            other => panic!("expected a control-record error, got {other:?}"),
        }
        let rendered = err.to_string();
        assert!(
            rendered.contains("errro"),
            "must name the field: {rendered}"
        );
        assert!(
            rendered.contains("control record"),
            "must say which shape failed: {rendered}"
        );
        assert!(std::error::Error::source(&err).is_some());
        std::fs::remove_dir_all(&dir).ok();
    }

    /// An unknown `record` value is refused, never ignored: story 4.5b adds `capability`, and a
    /// reader that skipped what it did not recognise would replay a downgrade as a clean poll.
    #[test]
    fn an_unknown_record_kind_is_refused() {
        let (dir, result) = read_scratch(
            "ctl-kind",
            &format!("{}\n{{\"record\":\"reboot\"}}\n", good_line()),
        );
        let err = result.expect_err("an unknown record kind must be refused");
        assert!(
            matches!(err, FixtureError::ControlRecordLine { lineno: 2, .. }),
            "{err:?}"
        );
        assert!(err.to_string().contains("reboot"), "must name it: {err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A line carrying neither marker has no defined meaning, and must not acquire one by
    /// falling through to whichever parser is tried first.
    ///
    /// The message must name the keys the author actually WROTE. Asserting only the template's own
    /// literals (`obs_id`, `record`) would be true of every `UnrecognisedLine` ever produced, and
    /// would not tell a correct message from a useless one.
    #[test]
    fn a_line_carrying_neither_marker_names_the_keys_it_found() {
        let (dir, result) = read_scratch(
            "neither",
            &format!("{}\n{{\"hello\":\"world\",\"aardvark\":1}}\n", good_line()),
        );
        let err = result.expect_err("a line that is neither shape must be refused");
        match &err {
            FixtureError::UnrecognisedLine { lineno, found, .. } => {
                assert_eq!(*lineno, 2);
                assert!(found.contains("hello"), "must name the key found: {found}");
                assert!(found.contains("aardvark"), "must name every key: {found}");
            }
            other => panic!("expected an unrecognised-line error, got {other:?}"),
        }
        let rendered = err.to_string();
        assert!(rendered.contains("obs_id"), "{rendered}");
        assert!(rendered.contains("record"), "{rendered}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// The case Decision 2 is ARGUED from: an author misspells `obs_id` itself.
    ///
    /// Under an absence-based discriminator this line would be handed to the control-record parser
    /// and blamed for a record nobody wrote. Under the positive marker it is an `UnrecognisedLine`
    /// that NAMES the misspelling, which is the outcome the whole design exists to produce — and
    /// until the review, the rationale's own example had no test.
    #[test]
    fn a_misspelled_obs_id_names_the_misspelling() {
        let typo = r#"{"obs_di":"aaaaaaaa-0000-4000-8000-0000000000ff","facts":[]}"#;
        let (dir, result) = read_scratch("obs-id-typo", &format!("{}\n{typo}\n", good_line()));
        let err = result.expect_err("a misspelled obs_id must be refused");
        match &err {
            FixtureError::UnrecognisedLine { lineno, found, .. } => {
                assert_eq!(*lineno, 2);
                assert!(
                    found.contains("obs_di"),
                    "must name the misspelling: {found}"
                );
            }
            other => panic!("expected an unrecognised-line error, got {other:?}"),
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A line that both follows a terminal failure AND is inadmissible on its own terms is
    /// reported for what it IS. Reporting "unreachable" first would cost the author two edit
    /// cycles to discover that the line they wrote was never admissible anywhere.
    #[test]
    fn an_inadmissible_line_after_a_failure_is_diagnosed_for_itself() {
        let cancelled = r#"{"record":"failure","error":"Cancelled"}"#;
        let (dir, result) = read_scratch(
            "cancel-after-failure",
            &format!("{}\n{UNREACHABLE}\n{cancelled}\n", good_line()),
        );
        let err = result.expect_err("a scripted cancellation must be refused");
        assert!(
            matches!(err, FixtureError::CancellationScripted { lineno: 3, .. }),
            "the cancellation must win over the unreachability: {err:?}"
        );

        // A malformed line after a failure is likewise a parse error, not "unreachable".
        let (dir2, result2) = read_scratch(
            "malformed-after-failure",
            &format!("{}\n{UNREACHABLE}\n{{ not json\n", good_line()),
        );
        assert!(
            matches!(
                result2.expect_err("a malformed line must be refused"),
                FixtureError::Line { lineno: 3, .. }
            ),
            "a malformed line is a parse error, not an unreachable record"
        );
        std::fs::remove_dir_all(&dir).ok();
        std::fs::remove_dir_all(&dir2).ok();
    }

    /// Valid JSON that is not an object at all: `42`, `[]`, `"x"`, `null`. Each must be named for
    /// what it is — "unrecognised" alone sends the author looking in the wrong place.
    #[test]
    fn a_line_that_is_not_an_object_is_named_for_what_it_is() {
        for (body, kind) in [
            ("42", "number"),
            ("[]", "array"),
            ("\"x\"", "string"),
            ("null", "null"),
            ("true", "boolean"),
        ] {
            let (dir, result) = read_scratch("not-object", &format!("{}\n{body}\n", good_line()));
            let err = result.expect_err("a non-object line must be refused");
            match &err {
                FixtureError::UnrecognisedLine { lineno, found, .. } => {
                    assert_eq!(*lineno, 2, "for {body}");
                    assert!(found.contains(kind), "{found} must name `{kind}`");
                }
                other => panic!("expected an unrecognised-line error for {body}, got {other:?}"),
            }
            std::fs::remove_dir_all(&dir).ok();
        }
    }

    /// A line that is two things is a line whose meaning depends on which reader reads it.
    #[test]
    fn a_line_carrying_both_markers_is_refused() {
        let both = r#"{"record":"failure","obs_id":"aaaaaaaa-0000-4000-8000-0000000000ff"}"#;
        let (dir, result) = read_scratch("both", &format!("{}\n{both}\n", good_line()));
        let err = result.expect_err("an ambiguous line must be refused");
        assert!(
            matches!(err, FixtureError::AmbiguousLine { lineno: 2, .. }),
            "{err:?}"
        );
        assert!(err.to_string().contains("both"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// `Cancelled` is the ONE variant that leaves liveness unchanged. A file able to mint it
    /// could assert that nothing was blinded when nothing cancelled anything.
    #[test]
    fn a_stream_may_not_script_cancellation() {
        let cancelled = r#"{"record":"failure","error":"Cancelled"}"#;
        let (dir, result) = read_scratch("cancel", &format!("{}\n{cancelled}\n", good_line()));
        let err = result.expect_err("a scripted cancellation must be refused");
        assert!(
            matches!(err, FixtureError::CancellationScripted { lineno: 2, .. }),
            "{err:?}"
        );
        assert!(err.to_string().contains("token"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Nothing may follow a terminal failure. An unreachable observation still satisfies a trap's
    /// cross-check, so it would yield a trap that can never fire — the hole 4.1/4.2 exist to close.
    #[test]
    fn nothing_may_follow_a_terminal_failure() {
        let (dir, result) = read_scratch(
            "after-failure",
            &format!("{}\n{UNREACHABLE}\n{}\n", good_line(), good_line()),
        );
        let err = result.expect_err("a record after a terminal failure must be refused");
        match &err {
            FixtureError::RecordAfterTerminalFailure {
                lineno,
                failure_line,
                ..
            } => assert_eq!((*lineno, *failure_line), (3, 2), "both lines are named"),
            other => panic!("expected a record-after-failure error, got {other:?}"),
        }
        assert!(err.to_string().contains("never be reached"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Every replay stream in the corpus is DISCOVERED by walking and read as records — the same
    /// treatment `scenario/traps/` has had since story 4.1. Without it, a committed `.jsonl` would
    /// be hashed by the gate and parsed by nobody.
    /// The first `obs_id` that appears in two DIFFERENT streams, if any — pure, so it can be
    /// proven to red on hand-built input instead of a scratch corpus.
    ///
    /// `streams` is `(stream label, its obs_ids)`. An `obs_id` is the anchor a trap points at
    /// (story 4.2, never a line number), so across the whole corpus it must name exactly one
    /// observation — or a failure report naming it cannot say which stream it meant. `read_records`
    /// enforces this WITHIN a stream; this closes it ACROSS streams (the item open since the 4.1
    /// review). A real collision existed until 2026-07-22 — `partial-then-failed.jsonl` reused
    /// `example-traps.jsonl`'s ids — which is exactly why this guard exists.
    fn first_cross_stream_obs_id(
        streams: &[(String, Vec<Uuid>)],
    ) -> Option<(Uuid, String, String)> {
        let mut seen: std::collections::BTreeMap<Uuid, String> = std::collections::BTreeMap::new();
        for (label, ids) in streams {
            for id in ids {
                if let Some(first) = seen.get(id) {
                    // Skip a repeat WITHIN one stream — `read_records` already refuses that, and
                    // this guard is only about two DIFFERENT streams.
                    if first != label {
                        return Some((*id, first.clone(), label.clone()));
                    }
                } else {
                    seen.insert(*id, label.clone());
                }
            }
        }
        None
    }

    #[test]
    fn the_cross_stream_obs_id_detector_finds_a_collision_and_ignores_a_within_stream_repeat() {
        let a = Uuid::from_u128(0xa);
        let b = Uuid::from_u128(0xb);
        // Two DIFFERENT streams sharing `a` -> a collision naming both.
        let collide = vec![
            ("first.jsonl".to_string(), vec![a, b]),
            ("second.jsonl".to_string(), vec![Uuid::from_u128(0xc), a]),
        ];
        let (id, first, second) = first_cross_stream_obs_id(&collide).expect("a collision");
        assert_eq!(id, a);
        assert_eq!(
            (first.as_str(), second.as_str()),
            ("first.jsonl", "second.jsonl")
        );

        // Distinct ids across streams -> none.
        let clean = vec![
            ("first.jsonl".to_string(), vec![a, b]),
            ("second.jsonl".to_string(), vec![Uuid::from_u128(0xc)]),
        ];
        assert!(first_cross_stream_obs_id(&clean).is_none());

        // A repeat WITHIN one stream is NOT this guard's business (read_records owns it).
        let within = vec![("only.jsonl".to_string(), vec![a, a])];
        assert!(first_cross_stream_obs_id(&within).is_none());
    }

    /// No committed replay stream shares an `obs_id` with another — the corpus-wide anchor
    /// uniqueness the 4.1 review left open. Walks the real corpus and runs the pure detector.
    #[test]
    fn no_obs_id_is_shared_across_replay_streams() {
        let mut streams: Vec<(String, Vec<Uuid>)> = Vec::new();
        walk_replay_streams(&mut |path| {
            let ids = read_records(path)
                .unwrap_or_else(|e| {
                    panic!("corpus replay stream {} is invalid: {e}", path.display())
                })
                .into_iter()
                .filter_map(|r| r.as_observation().map(|o| o.obs_id.as_uuid()))
                .collect();
            streams.push((path.display().to_string(), ids));
        });
        if let Some((id, first, second)) = first_cross_stream_obs_id(&streams) {
            panic!(
                "obs_id {id} appears in both {first} and {second} — a trap pointing at it could not say which"
            );
        }
    }

    #[test]
    fn every_replay_stream_in_the_corpus_is_valid() {
        walk_replay_streams(&mut |path| {
            let records = read_records(path).unwrap_or_else(|e| {
                panic!("corpus replay stream {} is invalid: {e}", path.display())
            });
            // A stream that parses to nothing is a file the gate hashes and the engine cannot
            // use — the same vacuity the fixtures gate carried until story 4.1.
            assert!(
                !records.is_empty(),
                "{}: a committed replay stream must carry at least one record",
                path.display()
            );
        });
    }

    // ── Trap files (story 4.2) ───────────────────────────────────────────────

    /// `obs_id` is the anchor the labelling format rests on: a trap points at one, "never by
    /// line number". Two lines sharing an id void that guarantee — a trap would silently judge
    /// whichever the reader happened to keep.
    #[test]
    fn a_stream_repeating_an_obs_id_is_refused() {
        let dir = scratch_dir("dup-obs");
        let path = dir.join("dup.jsonl");
        let line = serde_json::to_string(&expected()[0]).unwrap();
        std::fs::write(&path, format!("{line}\n{line}\n")).unwrap();

        let err = read_jsonl(&path).expect_err("a repeated obs_id must be refused");
        match &err {
            FixtureError::DuplicateObservationId {
                first_line,
                second_line,
                ..
            } => {
                assert_eq!((*first_line, *second_line), (1, 2));
            }
            other => panic!("expected a duplicate-id error, got {other:?}"),
        }
        assert!(err.to_string().contains("appears on lines"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Every trap file in the corpus must parse, validate and cross-check — DISCOVERED by
    /// WALKING, not by listing one directory. Trap FAMILIES (story 4.9 onward) are exactly what
    /// will introduce a subdirectory, and a non-recursive scan would hash them and never read
    /// them — reintroducing the hole this test exists to close.
    ///
    /// The walk itself moved to `walk_trap_files` in story 5.2, so this test and the privacy scan
    /// below cannot disagree about which files "every trap file" names.
    #[test]
    fn every_trap_file_in_the_corpus_is_valid() {
        walk_trap_files(&mut |path| {
            read_traps(path)
                .unwrap_or_else(|e| panic!("corpus trap file {} is invalid: {e}", path.display()));
        });
    }

    /// The privacy rule reaches trap-file TEXT, comments included (story 5.2).
    ///
    /// It reads the raw bytes with `read_to_string` and scans them BEFORE `toml::from_str` — the
    /// point of the whole AC. A parse-then-inspect rule cannot see a header comment, and a header
    /// comment is exactly where an author narrating a real capture would paste it. That idiom is
    /// not invented here: story 4.18 already reads and scans `scenario/wire/unifi-clients.json`
    /// the same way, because that directory sits outside every corpus walk.
    ///
    /// Until this test, `assert_text_is_synthetic` reached NO committed trap text — its corpus
    /// call sites were the `Record::Failure` arm of the replay walk and the wire body. Story
    /// 4.14's own "no octets in a trap reason" rule was held by code review, not by a gate; the
    /// register carried that from 4.14's review to here.
    ///
    /// **It asserts its own COVERAGE, not just that it ran.** `walk_trap_files` already refuses to
    /// pass over zero files, but eleven files carrying zero addresses would be just as vacuous and no
    /// file count can see it — the level below the one story 5.1's review reached. Measured on the
    /// committed corpus at `e846836`: 4 distinct MACs (`00:00:5e:00:01:0a`, `02:00:5e:00:53:10`,
    /// `02:00:5e:00:53:20`, `02:00:5e:00:53:78`) and 3 distinct IPs (`192.0.2.1`, `192.0.2.120`,
    /// `192.0.2.121`). ⚠️ **Story 5.13b raised the MAC floor from 4 to 5 and the reason is worth
    /// keeping**: its `blinded-source.toml` reasons cite `02:00:5e:00:56:01`, a FIFTH MAC — and a
    /// floor left at 4 would from then on have tolerated the loss of one previously-pinned value
    /// while still reading as a pass. **A floor is only a guard while it equals what is there**;
    /// growing the corpus without raising it converts the guard into slack, silently. Its code
    /// review measured this (the scan reports 5, the floor said 4), and the story's own count sweep
    /// had edited the doc comment two lines above and left the measurement and the floor untouched. Those values live in `reason` strings AND in header COMMENTS —
    /// `00:00:5e:00:01:0a` is a comment in `vrrp-virtual-mac.toml` and appears nowhere else in that
    /// file — which is the AC working: the scan reads bytes, so a comment counts.
    ///
    /// **What the floor below actually guarantees, stated no wider than it is.** It is a GLOBAL
    /// count over distinct values, so it catches a re-authoring that drops addresses (story 5.2b
    /// touches these very families) and does NOT catch one that drops some while adding others
    /// elsewhere. The distribution is uneven — `dhcp-churn.toml` 1 MAC + 2 IPs, `example.toml`
    /// 1 MAC, `randomized-mac.toml` 1 MAC, `vrrp-virtual-mac.toml` 1 MAC + 1 IP,
    /// `blinded-source.toml` 1 MAC, the other six files nothing — and that sentence is an inventory
    /// with no guard behind it, which is exactly what the register warns about, and which story
    /// 5.13b's addition demonstrated by falsifying it. Pinning per file is the stronger check and is not taken here;
    /// the floor is the cheap half that stops the scan going silently empty.
    #[test]
    fn the_committed_trap_text_carries_no_real_network_data() {
        let mut macs = std::collections::BTreeSet::new();
        let mut ips = std::collections::BTreeSet::new();
        walk_trap_files(&mut |path| {
            let raw = std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
            let seen = assert_text_is_synthetic(&raw, path);
            macs.extend(seen.macs);
            ips.extend(seen.ips);
        });
        assert!(
            macs.len() >= 5 && ips.len() >= 3,
            "the trap-text scan inspected {} distinct MAC(s) and {} distinct IP(s); it was \
             measured at 5 and 3, so a scan finding fewer has stopped exercising the rule it \
             claims to enforce rather than proving the corpus clean",
            macs.len(),
            ips.len()
        );
    }

    const EXAMPLE_TRAPS: &str = "scenario/traps/example.toml";

    /// The committed example must parse, validate, and point only at observations that exist.
    #[test]
    fn the_committed_trap_file_reads_and_cross_checks() {
        let traps = read_traps(&fixture_path(EXAMPLE_TRAPS).unwrap()).expect("the example reads");
        // Coverage, not order: reordering the `[[trap]]` blocks is a no-op and must stay one.
        let columns: std::collections::BTreeSet<&str> =
            traps.trap.iter().map(|t| t.expect.column()).collect();
        assert_eq!(
            columns,
            ["must-abstain", "must-merge", "must-not-merge"]
                .into_iter()
                .collect(),
            "the example must exercise all three of D18's columns"
        );
        // Every decision names a rule, and it is not blank — the premise of `(verdict, rule)`.
        for trap in &traps.trap {
            if let Some(rule) = trap.expect.rule() {
                assert!(
                    !rule.0.trim().is_empty(),
                    "trap {:?} names no rule",
                    trap.id
                );
            }
            assert!(!trap.reason.trim().is_empty());
        }
        // The traps span two streams: a trap names the stream it judges, and nothing assumes one.
        let streams: std::collections::BTreeSet<&str> =
            traps.trap.iter().map(|t| t.replay.as_str()).collect();
        assert_eq!(
            streams.len(),
            2,
            "the example must exercise more than one stream"
        );
    }

    /// The scratch registry refuses a tag two call sites claim — issue #38's shape.
    ///
    /// 🔴 **The registry shipped with NO test, and the code review's blind layer found it from
    /// the diff alone.** A prove-to-red run during development is not a guard in the suite, and
    /// this project's own rule says so: *"it does not excuse a new guard shipping without a test
    /// that reds when it is removed."* ⚠️ **And the first registry would have PASSED this test
    /// while still being wrong** — it keyed on the helper's name, so two calls from one test body
    /// were one owner. It reds here only because the owner is the call site.
    ///
    /// The tag is deliberately unused elsewhere: the registry is process-global, so a tag this
    /// test poisons would be unavailable to any other.
    #[test]
    #[should_panic(expected = "is claimed by BOTH")]
    fn a_scratch_tag_two_call_sites_claim_is_refused() {
        let _first = scratch_dir("registry-guard");
        let _second = scratch_dir("registry-guard");
    }

    #[track_caller]
    fn write_traps(tag: &str, body: &str) -> PathBuf {
        let path = scratch_dir(tag).join("traps.toml");
        std::fs::write(&path, body).unwrap();
        path
    }

    /// The oracle rests on the author's reason. Its absence must stop the read, not warn.
    #[test]
    fn a_trap_without_a_reason_is_refused() {
        let path = write_traps(
            "no-reason",
            r#"
[[trap]]
id = "nameless"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001"]
reason = "   "
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        );
        let err = read_traps(&path).expect_err("a reasonless trap must be refused");
        assert!(matches!(err, FixtureError::Trap { .. }), "{err:?}");
        let rendered = err.to_string();
        assert!(
            rendered.contains("nameless"),
            "must name the trap: {rendered}"
        );
        // Assert the CAUSE, not a constant of the message: an earlier version checked for the
        // literal "must-abstain", which every reason error contains and which therefore could not
        // tell a correct message from a wrong one.
        assert!(
            matches!(
                err,
                FixtureError::Trap {
                    source: opencmdb_core::trap::TrapError::ReasonMissing { .. },
                    ..
                }
            ),
            "an empty reason is ReasonMissing, not another reason error: {err:?}"
        );
    }

    /// An ABSENT `reason` key must be refused by a message that names the TRAP — not by serde,
    /// which can only name the field. That is why `reason` carries `#[serde(default)]`.
    #[test]
    fn a_trap_whose_reason_key_is_absent_names_the_trap() {
        let path = write_traps(
            "absent-reason",
            r#"
[[trap]]
id = "keyless"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001"]
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        );
        let err = read_traps(&path).expect_err("an absent reason must be refused");
        assert!(matches!(err, FixtureError::Trap { .. }), "{err:?}");
        assert!(
            err.to_string().contains("keyless"),
            "must name the trap, not just the field: {err}"
        );
    }

    /// A `./` prefix names the same file under a spelling the MANIFEST never records.
    #[test]
    fn a_dot_slash_replay_is_refused() {
        assert!(matches!(
            fixture_path("./scenario/replay/minimal.jsonl")
                .expect_err("a `./` prefix must be refused"),
            FixtureError::OutsideCorpus { .. }
        ));
    }

    /// A misspelled field must fail loudly rather than be ignored — the rule story 4.1
    /// established for observations, applied to the labelling.
    #[test]
    fn an_unknown_field_in_a_trap_file_is_refused() {
        let path = write_traps(
            "unknown-field",
            r#"
[[trap]]
id = "typo"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001"]
reason = "a reason long enough to state something about this trap"
resaon = "the misspelling that motivates deny_unknown_fields"
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        );
        let err = read_traps(&path).expect_err("an unknown field must be refused");
        assert!(matches!(err, FixtureError::Toml { .. }), "{err:?}");
        let rendered = err.to_string();
        assert!(
            rendered.contains("resaon"),
            "must name the field: {rendered}"
        );
        assert!(
            rendered.contains("trap file"),
            "must say what failed: {rendered}"
        );
    }

    /// A decision carrying an abstention cause is not merely invalid, it is unrepresentable —
    /// so the parse fails rather than a validator catching it later.
    #[test]
    fn a_decision_carrying_an_abstention_cause_is_refused() {
        // ⚠️ `both-cause`, not `both`: `read_scratch` claims `"both"` at :1792, and the two
        // resolve to one directory that the other one deletes. Issue #38's reproduced cause.
        let path = write_traps(
            "both-cause",
            r#"
[[trap]]
id = "confused"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001"]
reason = "a reason long enough to state something about this trap"
expect = { must-merge = { rule = "r", cause = "NoObservedValue" } }
"#,
        );
        let err = read_traps(&path).expect_err("a column must carry only its own payload");
        assert!(matches!(err, FixtureError::Toml { .. }), "{err:?}");
        assert!(
            err.to_string().contains("cause"),
            "must name the offending key: {err}"
        );
    }

    /// A trap pointing at an observation its stream does not contain can never fire, and would
    /// sit in the corpus looking like coverage. The gate counts traps.
    #[test]
    fn a_trap_judging_an_absent_observation_is_refused() {
        let path = write_traps(
            "dangling",
            r#"
[[trap]]
id = "points-at-nothing"
replay = "scenario/replay/minimal.jsonl"
observations = ["ffffffff-0000-4000-8000-00000000dead"]
reason = "this observation is deliberately absent from the stream it names"
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        );
        let err = read_traps(&path).expect_err("a dangling reference must be refused");
        match &err {
            FixtureError::DanglingObservation { trap, replay, .. } => {
                assert_eq!(trap, "points-at-nothing");
                assert_eq!(replay, "scenario/replay/minimal.jsonl");
            }
            other => panic!("expected a dangling-observation error, got {other:?}"),
        }
        assert!(err.to_string().contains("does not contain"), "{err}");
    }

    /// A trap file may not reach outside the corpus through its `replay` field either.
    #[test]
    fn a_trap_replaying_outside_the_corpus_is_refused() {
        let path = write_traps(
            "escape",
            r#"
[[trap]]
id = "escapes"
replay = "../../etc/passwd"
observations = ["aaaaaaaa-0000-4000-8000-000000000001"]
reason = "a reason long enough to state something about this trap"
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        );
        assert!(matches!(
            read_traps(&path).expect_err("a replay path leaving the corpus must be refused"),
            FixtureError::OutsideCorpus { .. }
        ));
    }

    /// D56: the corpus path is "a single constant, in one module, never copied. If it appears
    /// more than once in the tree, it is already broken." That is checkable, so it is checked.
    ///
    /// Three things this deliberately does NOT do, each of which was a real defect:
    /// it does not follow directory symlinks (a link to an ancestor made the walk loop forever);
    /// it does not swallow read errors (an unreadable subtree used to shrink the search space
    /// into a false green); and it does not walk the whole workspace (a `git worktree` or a
    /// vendored copy under the root produced a red that blamed path discipline for nothing).
    #[test]
    fn the_fixtures_path_is_expressed_once() {
        const NEEDLE: &str = "/../../fixtures";
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        // The source roots this rule governs — not an unbounded walk of everything present.
        let mut roots = vec![workspace.join("xtask/src")];
        for crate_dir in std::fs::read_dir(workspace.join("crates"))
            .expect("crates/ must be readable")
            .flatten()
        {
            roots.push(crate_dir.path().join("src"));
        }

        let mut occurrences = 0usize;
        let mut files = Vec::new();
        while let Some(dir) = roots.pop() {
            if !dir.exists() {
                continue;
            }
            for entry in
                std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
            {
                let entry = entry.expect("a directory entry must be readable");
                // `file_type()` does NOT follow symlinks: a link is neither descended into nor
                // read, so a cycle cannot exist and a link cannot smuggle in a second copy.
                let file_type = entry.file_type().expect("a file type must be readable");
                let path = entry.path();
                if file_type.is_dir() {
                    roots.push(path);
                } else if file_type.is_file() && path.extension().is_some_and(|e| e == "rs") {
                    let text = std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
                    // Count OCCURRENCES, not files: the constant written twice in one file is
                    // exactly as broken as the same constant written in two files.
                    let n = text.matches(NEEDLE).count();
                    if n > 0 {
                        occurrences += n;
                        files.push((path, n));
                    }
                }
            }
        }

        // Two expected occurrences, both in THIS file: the constant itself, and NEEDLE above.
        assert_eq!(
            occurrences, 2,
            "the corpus path must be expressed once (plus this test's own needle); found {files:?}"
        );
    }

    /// Pin the `obs_id` ↔ OBSERVATION-ORDER binding of a committed stream, and its length: the
    /// stream carries exactly `expected_len` observations, and the `n`-th of them (0-indexed)
    /// carries `{prefix}-0000-4000-8000-{n+1:012}`.
    ///
    /// **Observation order, NOT the file line.** The slice comes from [`read_jsonl`], which DROPS
    /// control records, so the two diverge the moment a stream carries one: in
    /// `capability-downgrade.jsonl` the `capability` record is on file line 3, which puts `obs_id`
    /// `…0003` on file LINE 4. This doc said "line `n`" until story 5.1's review measured it. A
    /// byte-pin that needs the file line must read the file, not this slice.
    ///
    /// **`expected_len` is not ceremony.** Without it the helper asserts only the ids of whatever it
    /// was handed, so an empty or truncated slice passes: `assert_obs_ids(&[], "afafafaf", 0)` is
    /// the only way to say "nothing here" out loud. The four inline loops this replaced iterated a
    /// FIXED index list and panicked on a short stream; folding them into a bare `enumerate()` lost
    /// that, and every call site happened to assert its own length beside the call — a guard living
    /// at the call site, which the four families story 5.2b points here have no sibling to inherit
    /// (story 5.1's review). Proven red by handing it a two-observation slice of the
    /// three-observation `dhcp-churn` stream (story 5.1, mutation 7).
    ///
    /// The call sites keep their own `observations.len()` assertion, and that redundancy is
    /// DELIBERATE — do not collapse it. Several of them index `observations[n]` with a fixed list
    /// BEFORE reaching this helper, so their length assertion is what turns a truncated stream into
    /// a named failure instead of an out-of-bounds panic. This one makes the helper self-sufficient
    /// for a caller that has no such loop.
    ///
    /// The traps judge by `obs_id` while every byte-pin reads by INDEX. Without this, a deliberate
    /// re-authoring that swaps two lines' ids (with a re-hashed manifest) would invert what a
    /// family's traps judge while every byte-level assertion stayed green — registered under story
    /// 4.15's review, closed by story 5.1.
    ///
    /// **Why folding six call sites into one is DRY and not a lost oracle.** The house rule protects
    /// redundancy a test pins on purpose — `expected()` restates the corpus VALUES in Rust and must
    /// survive any DRY pass. The loops folded here restate nothing: all four that already existed
    /// COMPUTED their ids with `format!("…{suffix:012}")`, so what is removed is mechanical
    /// duplication of one loop, not a second statement of anything.
    ///
    /// What this helper does encode, in one place, are two CORPUS CONVENTIONS: the fixed
    /// `-0000-4000-8000-` middle segment, and sequential numbering from 1 rendered in DECIMAL into
    /// a hexadecimal field (invisible until a stream passes nine observations — the longest today
    /// carries six). All three hold for the 15 streams under `scenario/replay/` and for the wire
    /// artefact, the 16th, which is this helper's sixth call site. A future stream numbered
    /// otherwise gets its OWN assertion — it is not re-authored to satisfy this helper.
    fn assert_obs_ids(observations: &[Observation], prefix: &str, expected_len: usize) {
        assert_eq!(
            observations.len(),
            expected_len,
            "the stream must carry exactly {expected_len} observations"
        );
        for (n, observation) in observations.iter().enumerate() {
            assert_eq!(
                observation.obs_id.to_string(),
                format!("{prefix}-0000-4000-8000-{:012}", n + 1),
                "observation {n} carries its authored obs_id"
            );
        }
    }

    /// Pin ONE trap's binding: which observations it judges, **in order**, and under which
    /// column and rule — story 5.2b's AC4b.
    ///
    /// **Why this exists, measured.** Every byte-pin in this module lives on the `.jsonl`, but a
    /// family's premise is stated across TWO files: the stream holds the values, the `.toml`
    /// declares WHICH pair of `obs_id`s is judged and under which column. Nothing asserted the
    /// second half. `read_traps` cross-checks only that a trap's `obs_id`s EXIST in the stream
    /// (a `BTreeSet` membership test), and `trap_gate`'s completeness check only asks that both
    /// poles of a family are present — which any EXCHANGE of the two poles' observation vectors
    /// preserves. Measured during this story's validation pass: exchanging the two `observations`
    /// vectors in `fixtures/scenario/traps/cloned-mac.toml` — three characters, no stream byte
    /// touched — makes the corpus DEMAND the false merge (`doc-host-echo` + `doc-host-foxtrot`
    /// under `must-merge`/`l1-exact-mac`), and the whole workspace suite stayed green.
    ///
    /// The sha256 lock is not the backstop: this corpus's stated threat model is *"a DELIBERATE
    /// re-authoring, which refreshes `MANIFEST.toml` by definition"*.
    ///
    /// ORDER is pinned, not membership — but **not** because a set comparison would miss the
    /// exchange above. It would not: `cloned-mac`'s poles are `[001, 002]` and `[001, 003]`, which
    /// differ AS SETS, so a set comparison reds on that exchange too. *(An earlier draft of this
    /// doc claimed the opposite and its own example refuted it — corrected on this story's code
    /// review, in the house idiom of preferring the weaker true sentence.)* The narrower true
    /// reason: a set comparison is blind exactly when two poles judge the SAME pair in a different
    /// order — `[001, 002]` against `[002, 001]` — which is a judgement no engine can tell apart
    /// today, but which a later `observations`-order-sensitive rule would. Comparing the vector
    /// costs nothing and keeps the pin honest ahead of that.
    ///
    /// `family` is pinned too, and that is measured: deleting BOTH `family` lines from a trap file
    /// left the whole suite green while silently exempting the family from `incomplete_families`
    /// (`family` is `Option`, and a family-less trap is *"exempt from the completeness check"* by
    /// design) — after which the family could be reduced to ONE pole with the gate still green.
    /// Deleting only ONE line already reds `trap_gate`; deleting both did not, until this.
    ///
    /// What is deliberately NOT pinned here: `replay` and `reason`. `replay` is reached indirectly
    /// — each family's `obs_id` prefix is unique to its stream and
    /// `no_obs_id_is_shared_across_replay_streams` holds that — and `reason` is prose, whose
    /// mechanical tie to the values it cites is registered as open, not claimed here.
    fn assert_trap_binds(
        traps: &TrapFile,
        id: &str,
        observations: &[&str],
        expect: Expectation,
        family: Option<&str>,
    ) {
        let trap = traps
            .trap
            .iter()
            .find(|t| t.id.0 == id)
            .unwrap_or_else(|| panic!("the trap file must declare `{id}`"));
        let judged: Vec<String> = trap.observations.iter().map(|o| o.to_string()).collect();
        let authored: Vec<String> = observations.iter().map(|s| (*s).to_owned()).collect();
        assert_eq!(
            judged, authored,
            "trap `{id}` judges exactly these observations, in this order"
        );
        assert_eq!(
            trap.expect, expect,
            "trap `{id}` judges under exactly this column and rule"
        );
        assert_eq!(
            trap.family.as_ref().map(|f| f.0.as_str()),
            family,
            "trap `{id}` declares exactly its authored family — a family-less trap is EXEMPT from \
             the completeness check, so losing the key is not a cosmetic edit"
        );
    }

    /// The `must-merge` / `must-not-merge` shorthands the bindings below read with. They take the
    /// rule id as a `&str` so a call site states the rule verbatim, the way the `.toml` does.
    fn merge(rule: &str) -> Expectation {
        Expectation::MustMerge {
            rule: RuleId(rule.to_owned()),
        }
    }

    /// The refusal pole — see [`merge`].
    fn not_merge(rule: &str) -> Expectation {
        Expectation::MustNotMerge {
            rule: RuleId(rule.to_owned()),
        }
    }

    /// Story 4.13's byte-pin — the second, independent oracle over `dhcp-churn.jsonl`, in the
    /// spirit of `expected()`: the two holders of the recycled address `192.0.2.120` are
    /// separated by NOTHING but time (D19 — DHCP churn is tested by replaying timestamps; the
    /// engine never touches the clock). Nothing else in the harness validates timestamps, so
    /// their strict increase in this stream is pinned here or nowhere.
    #[test]
    fn the_dhcp_churn_stream_moves_the_address_only_through_observed_at() {
        let observations = read_jsonl(&fixture_path("scenario/replay/dhcp-churn.jsonl").unwrap())
            .expect("the dhcp-churn stream must read");
        assert_eq!(observations.len(), 3, "three authored presences, exactly");
        // Exactly 3 facts per line. Without this, `find()` below takes the FIRST match of each
        // kind and a duplicated or extra fact would pass every assertion unnoticed; with it, the
        // one-of-each extraction pins the fact list exactly.
        for (n, observation) in observations.iter().enumerate() {
            assert_eq!(
                observation.facts.len(),
                3,
                "observation {n} carries exactly its three facts"
            );
        }
        // This test predates the `obs_id` ↔ line rule (story 4.15's review) and read purely by
        // index; story 5.1 back-fills the binding, since `dhcp-churn.toml` judges by `obs_id`.
        assert_obs_ids(&observations, "adadadad", 3);

        let fact = |n: usize, pick: fn(&Fact) -> bool| {
            observations[n]
                .facts
                .iter()
                .find(|f| pick(f))
                .unwrap_or_else(|| panic!("observation {n} must carry the fact"))
                .clone()
        };
        let mac = |n| fact(n, |f| matches!(f, Fact::Mac { .. }));
        let ip = |n| fact(n, |f| matches!(f, Fact::IpV4 { .. }));
        let hostname = |n| fact(n, |f| matches!(f, Fact::Hostname { .. }));

        // N1 and N3 wear the recycled address — byte-identical `IpV4` facts, compared parsed.
        assert_eq!(
            ip(0),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 120)
            },
            "N1 holds the recycled address"
        );
        assert_eq!(ip(2), ip(0), "N3 holds the SAME address two hours later");

        // N1 and N2 are one host — identical Mac AND Hostname; only the lease moved, and the
        // address it moved TO is pinned: were N2 still holding `.120`, the must-merge pair would
        // be a same-address re-sighting, not a moved lease — the family's premise.
        assert_eq!(mac(0), mac(1), "N2 carries N1's exact MAC");
        assert_eq!(hostname(0), hostname(1), "N2 carries N1's exact hostname");

        // Story 5.2b (AC5) — the three authored values the trap `reason`s CITE, pinned by value
        // rather than only relationally. The assertions above are relational (`mac(0) == mac(1)`,
        // `hostname(2) != hostname(0)`) and stay green under a wholesale re-authoring that gives
        // this stream different synthetic values — at which point both reasons cite constants the
        // bytes no longer hold and the family's prose is stranded. Registered under story 4.13's
        // review; closed here.
        //
        // Counted on the committed file, because the epic (`epics.md:1391`) and the register
        // bullet both say "both `reason` strings cite" all three and that is FALSE: the MAC
        // appears in ONE reason (`dhcp-churn.toml:39`), `doc-host-hotel` in ONE (`:28`), and
        // `doc-host-golf` in BOTH. The union of the two reasons cites all three; neither reason
        // does. The conclusion is unchanged — all three are cited by prose and asserted by no
        // test until now.
        assert_eq!(
            mac(0),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 120]),
                locally_administered: true,
            },
            "N1 wears the MAC 02:00:5e:00:53:78 that the must-merge reason cites"
        );
        // N2 and N3's MACs too, added on this story's code review: the closure note claims all five
        // families "assert their authored MACs by value", and pinning only line 0 made that
        // sentence wider than the code. The COLLAPSE is already caught by `assert_ne!(mac(2),
        // mac(0))` below — what these reach is a wholesale re-authoring to a DIFFERENT distinct
        // pair, which leaves every relational assertion green.
        assert_eq!(
            mac(1),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 120]),
                locally_administered: true,
            },
            "N2 wears N1's exact authored MAC — same box, moved lease"
        );
        assert_eq!(
            mac(2),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 121]),
                locally_administered: true,
            },
            "N3 wears 02:00:5e:00:53:79 — the distinct MAC the must-not-merge pole rests on"
        );
        assert_eq!(
            hostname(0),
            Fact::Hostname {
                name: "doc-host-golf".into(),
                source: HostnameSource::Dhcp,
            },
            "N1 answers to doc-host-golf, the name BOTH reasons cite"
        );
        assert_eq!(
            hostname(2),
            Fact::Hostname {
                name: "doc-host-hotel".into(),
                source: HostnameSource::Dhcp,
            },
            "N3 answers to doc-host-hotel, the name the must-not-merge reason cites"
        );
        assert_eq!(
            ip(1),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 121)
            },
            "N2 holds the moved lease"
        );

        // N3 is a different box — it shares ONLY the IpV4 bytes with N1.
        assert_ne!(mac(2), mac(0), "N3's MAC differs from N1's");
        assert_ne!(hostname(2), hostname(0), "N3's hostname differs from N1's");

        // The churn lives in `observed_at` alone: the three instants are exactly the three
        // authored values, in strictly increasing order.
        let instants: Vec<Timestamp> = observations.iter().map(|o| o.observed_at).collect();
        assert_eq!(
            instants,
            vec![
                ts("2026-01-06T00:00:00Z"),
                ts("2026-01-06T01:00:00Z"),
                ts("2026-01-06T02:00:00Z"),
            ],
            "the reassignment happens BETWEEN observations, as authored time"
        );

        // Story 5.2b (AC4b) — the TOML side of the same premise. Everything above pins the
        // stream; this pins which pair each pole judges. Exchanging the two vectors would pair
        // the two doc-host-golf presences under `must-not-merge` and the golf/hotel pair under
        // `must-merge`, inverting the family without moving a stream byte.
        let traps = read_traps(&fixture_path("scenario/traps/dhcp-churn.toml").unwrap())
            .expect("the dhcp-churn trap file must read");
        assert_eq!(
            traps.trap.len(),
            2,
            "the family declares exactly its two poles"
        );
        assert_trap_binds(
            &traps,
            "dhcp-churn-must-not-merge",
            &[
                "adadadad-0000-4000-8000-000000000001",
                "adadadad-0000-4000-8000-000000000003",
            ],
            not_merge("l1-distinct-mac"),
            Some("dhcp-churn"),
        );
        assert_trap_binds(
            &traps,
            "dhcp-churn-must-merge",
            &[
                "adadadad-0000-4000-8000-000000000001",
                "adadadad-0000-4000-8000-000000000002",
            ],
            merge("l1-exact-mac"),
            Some("dhcp-churn"),
        );
    }

    /// Story 4.14's byte-pin — the second, independent oracle over `vrrp-virtual-mac.jsonl`, in
    /// the spirit of `expected()`: one IANA VRRP virtual MAC (the corpus's only
    /// universally-administered byte pattern, admitted by name in `is_synthetic_mac`) carries one
    /// VIP across two sightings whose uplink moves from router A's switch to router B's — while
    /// the two routers themselves keep their own MACs, hostnames and addresses. The harness
    /// validates no uplink geometry anywhere else, so the failover (a DIFFERENT peer switch, not
    /// merely a different port — multi-nic's must-merge treats same-switch-different-port as an
    /// uplink that AGREES) is pinned here or nowhere.
    #[test]
    fn the_vrrp_stream_shares_one_virtual_mac_and_moves_its_uplink() {
        let observations =
            read_jsonl(&fixture_path("scenario/replay/vrrp-virtual-mac.jsonl").unwrap())
                .expect("the vrrp-virtual-mac stream must read");
        assert_eq!(observations.len(), 4, "four authored presences, exactly");
        // Exact fact counts per line: V1/V2 are ARP-shaped sightings (Mac, IpV4, Uplink — no
        // Hostname: a VIP answers, nobody resolves it), A/B are full router presences. Without
        // these, `find()` below takes the FIRST match of each kind and an extra or duplicated
        // fact would pass unnoticed.
        for (n, expected_len) in [(0, 3), (1, 4), (2, 4), (3, 3)] {
            assert_eq!(
                observations[n].facts.len(),
                expected_len,
                "observation {n} carries exactly its facts"
            );
        }
        // This test predates the `obs_id` ↔ line rule (story 4.15's review) and read purely by
        // index; story 5.1 back-fills the binding, since `vrrp-virtual-mac.toml` judges by `obs_id`.
        assert_obs_ids(&observations, "aeaeaeae", 4);

        let fact = |n: usize, pick: fn(&Fact) -> bool| {
            observations[n]
                .facts
                .iter()
                .find(|f| pick(f))
                .unwrap_or_else(|| panic!("observation {n} must carry the fact"))
                .clone()
        };
        let mac = |n| fact(n, |f| matches!(f, Fact::Mac { .. }));
        let ip = |n| fact(n, |f| matches!(f, Fact::IpV4 { .. }));
        let hostname = |n| fact(n, |f| matches!(f, Fact::Hostname { .. }));
        let uplink = |n| fact(n, |f| matches!(f, Fact::Uplink { .. }));

        // V1 and V2 are the virtual gateway — byte-identical virtual Mac (flag honestly
        // `false`: the IANA range is universally administered) and byte-identical VIP.
        assert_eq!(
            mac(0),
            Fact::Mac {
                addr: MacAddr([0, 0, 94, 0, 1, 10]),
                locally_administered: false,
            },
            "V1 wears the authentic IANA VRRP MAC 00:00:5e:00:01:0a"
        );
        assert_eq!(mac(3), mac(0), "V2 wears V1's exact virtual MAC");
        assert_eq!(
            ip(0),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 1)
            },
            "V1 answers for the VIP"
        );
        assert_eq!(ip(3), ip(0), "V2 answers for the SAME VIP an hour later");

        // A and B are two real routers — distinct locally-administered MACs (flags honestly
        // `true`), distinct hostnames, distinct addresses.
        for n in [1, 2] {
            match mac(n) {
                Fact::Mac {
                    addr,
                    locally_administered,
                } => {
                    assert!(
                        addr.is_locally_administered() && locally_administered,
                        "observation {n}'s physical MAC is locally administered, flag and bytes"
                    );
                }
                other => panic!("observation {n} must carry a Mac, got {other:?}"),
            }
        }
        assert_ne!(mac(1), mac(2), "the two routers keep their own MACs");
        // The routers' own addresses are pinned too: the bearers' reason claims "distinct
        // addresses", and the primary's premise needs the VIP to be nobody's own address — a
        // drift giving A the VIP would change both traps' geometry while staying green here.
        assert_eq!(
            ip(1),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 2)
            },
            "A owns its own address"
        );
        assert_eq!(
            ip(2),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 3)
            },
            "B owns its own address"
        );
        assert_eq!(
            hostname(1),
            Fact::Hostname {
                name: "doc-rtr-alpha".into(),
                source: HostnameSource::Dhcp,
            },
            "A is doc-rtr-alpha"
        );
        assert_eq!(
            hostname(2),
            Fact::Hostname {
                name: "doc-rtr-bravo".into(),
                source: HostnameSource::Dhcp,
            },
            "B is doc-rtr-bravo"
        );

        // The failover, in uplink geometry: V1 sits where A sits; V2 sits where B sits — and B
        // hangs off the SECOND switch, so the move crosses switches (the committed shape of
        // `l2-different-switch`), not merely ports.
        assert_eq!(uplink(0), uplink(1), "V1 shares A's exact uplink");
        assert_eq!(uplink(3), uplink(2), "V2 shares B's exact uplink");
        // BOTH ends of the failover are pinned octet-exact (the 4.13 review's lesson, applied
        // by this story's own review): without V1's pin, a re-authored stream putting V1/A on
        // the second switch with a different port would stay green while dissolving the
        // "different switch, not merely port" premise the must-merge depends on.
        assert_eq!(
            uplink(0),
            Fact::Uplink {
                peer_mac: MacAddr([2, 0, 94, 0, 96, 10]),
                peer_port: "swport-11".into(),
            },
            "V1 sits on the first switch, at A's port"
        );
        assert_eq!(
            uplink(3),
            Fact::Uplink {
                peer_mac: MacAddr([2, 0, 94, 0, 96, 11]),
                peer_port: "swport-12".into(),
            },
            "the failover moved the VIP to the second switch's port"
        );
        assert_ne!(
            uplink(0),
            uplink(3),
            "the two VIP sightings disagree on uplink — the contradiction the must-merge overcomes"
        );

        // The four instants are exactly the authored values, strictly increasing: the failover
        // happens BETWEEN observations, as authored time.
        let instants: Vec<Timestamp> = observations.iter().map(|o| o.observed_at).collect();
        assert_eq!(
            instants,
            vec![
                ts("2026-01-08T00:00:00Z"),
                ts("2026-01-08T00:05:00Z"),
                ts("2026-01-08T00:10:00Z"),
                ts("2026-01-08T01:00:00Z"),
            ],
            "the failover happens BETWEEN observations, as authored time"
        );

        // Story 5.2b (AC4b), extended to this family by its code review — the TOML side. Everything
        // above pins the stream; this pins which pair each pole judges. Three poles here, and the
        // two `must-not-merge`s name DIFFERENT rules, so an exchange between them would swap the
        // virtual-MAC-prefix refusal for the hostname one without moving a byte.
        let traps = read_traps(&fixture_path("scenario/traps/vrrp-virtual-mac.toml").unwrap())
            .expect("the vrrp-virtual-mac trap file must read");
        assert_eq!(
            traps.trap.len(),
            3,
            "the family declares exactly its three traps"
        );
        assert_trap_binds(
            &traps,
            "vrrp-virtual-mac-must-not-merge-master",
            &[
                "aeaeaeae-0000-4000-8000-000000000001",
                "aeaeaeae-0000-4000-8000-000000000002",
            ],
            not_merge("l2-virtual-mac-prefix"),
            Some("vrrp-virtual-mac"),
        );
        assert_trap_binds(
            &traps,
            "vrrp-virtual-mac-must-not-merge-bearers",
            &[
                "aeaeaeae-0000-4000-8000-000000000002",
                "aeaeaeae-0000-4000-8000-000000000003",
            ],
            not_merge("l2-different-hostname"),
            Some("vrrp-virtual-mac"),
        );
        assert_trap_binds(
            &traps,
            "vrrp-virtual-mac-must-merge",
            &[
                "aeaeaeae-0000-4000-8000-000000000001",
                "aeaeaeae-0000-4000-8000-000000000004",
            ],
            merge("l1-exact-mac"),
            Some("vrrp-virtual-mac"),
        );
    }

    /// The closed allowlist's edges (story 4.14). The VRRP block is admitted 5-octet exact;
    /// every neighbour stays out: the block below it, VRRP IPv6 above it (NOT admitted until a
    /// fixture commits it), a first-octet near-miss, and a plain vendor-style address. Widening
    /// the helper's match to the 3-octet `00:00:5e` prefix reds this test (the recorded
    /// mutation).
    #[test]
    fn the_vrrp_allowance_is_five_octets_exact() {
        assert!(
            is_synthetic_mac(MacAddr([0, 0, 94, 0, 1, 10])),
            "00:00:5e:00:01:0a is the admitted IANA VRRP range"
        );
        assert!(
            is_synthetic_mac(MacAddr([0, 0, 94, 0, 1, 0])),
            "the last octet is the VRID field — every byte value there is equally protocol-\
             defined and privacy-safe (VRID 0 is not a deployable VRID, which is exactly why \
             nothing real can wear it)"
        );
        assert!(
            !is_synthetic_mac(MacAddr([0, 0, 94, 0, 0, 10])),
            "00:00:5e:00:00:0a sits below the block and stays out"
        );
        assert!(
            !is_synthetic_mac(MacAddr([0, 0, 94, 0, 2, 10])),
            "00:00:5e:00:02:0a is VRRP IPv6 — not admitted until a fixture commits it"
        );
        assert!(
            !is_synthetic_mac(MacAddr([0, 0, 95, 0, 1, 10])),
            "00:00:5f:00:01:0a is outside the IANA OUI and stays out"
        );
        assert!(
            !is_synthetic_mac(MacAddr([0, 0x11, 0x22, 0x33, 0x44, 0x55])),
            "a plain universally-administered vendor-style address stays out"
        );
        // Story 5.2. This row is the only place the tightened rule's TWO conjuncts can be pinned
        // independently of the tokenizer: `0x33` has the U/L bit set, so the old
        // locally-administered leg admitted it, and only the I/G test refuses it. Dropping
        // `&& !is_multicast_mac(addr)` from `is_synthetic_mac` reds exactly this assertion (the
        // recorded mutation).
        assert!(
            !is_synthetic_mac(MacAddr([0x33, 0x33, 0xff, 0, 0x60, 0x0a])),
            "33:33:ff:00:60:0a is locally administered AND multicast — an IPv6 solicited-node \
             MAC embeds real interface-identifier bytes and names no interface"
        );
    }

    /// The free-text scanner's WIRING onto the shared helper, proven directly and in isolation:
    /// one address, one rule, no corpus in the way — so a scanner whose MAC leg drifted from
    /// `assert_synthetic_mac`'s would red HERE, naming the address, rather than somewhere inside a
    /// walk over eleven files.
    ///
    /// **It is no longer the only thing that would red, and saying so is the honest version.**
    /// This doc claimed, for one story, that *"no COMMITTED text exercises the VRRP allowance
    /// through the scanner"* — false the moment it was written, and false BECAUSE of story 5.2:
    /// `scenario/traps/vrrp-virtual-mac.toml` carries `00:00:5e:00:01:0a` in a header comment, and
    /// that story's own `the_committed_trap_text_carries_no_real_network_data` reads and scans it.
    /// `0x00 & 0x02 == 0`, so the address is admitted ONLY by the VRRP leg: dropping
    /// `|| addr.0[..5] == [0, 0, 94, 0, 1]` reds that test too (measured, story 5.2's code review).
    /// This test keeps its value as the ISOLATED pin, not as the sole one.
    ///
    /// Before that, the doc said the scanner's *"only call site is the `Record::Failure` walk"* —
    /// true when story 4.14 wrote it, false from 4.18 onward. Twice now an inventory in a doc
    /// comment has rotted, both times within one story of being written; the lesson recorded here
    /// is to state what THIS test proves and let the register count call sites.
    #[test]
    fn the_text_scanner_admits_the_vrrp_range() {
        assert_text_is_synthetic(
            "the virtual gateway answers as 00:00:5e:00:01:0a on the shared VIP",
            Path::new("in-memory"),
        );
    }

    /// The scanner kept its teeth: a universally-administered MAC OUTSIDE the admitted block
    /// still panics in free text.
    #[test]
    #[should_panic(expected = "neither locally administered nor in the IANA VRRP")]
    fn the_text_scanner_still_refuses_a_mac_outside_the_block() {
        assert_text_is_synthetic(
            "a stray note naming 00:00:5e:00:00:0a must never be committed",
            Path::new("in-memory"),
        );
    }

    // ── The three named evasions, closed (story 5.2, AC3) ────────────────────
    //
    // Six rows, one test each, every `expected` naming the EXACT address the row asserts rather
    // than the generic tail of the message: a tokenizer that finds `98.18.0.1` inside
    // `198.18.0.1` also panics, and a loose substring would credit that bug with a pass.
    //
    // Rows (a)-(d) were observed GREEN before the fix — the scanner did NOT panic, so each of
    // these tests failed with *"test did not panic as expected"*. That two-sided observation is
    // what makes "the hole existed" a measurement rather than a claim (story 5.1, mutation 12).

    /// Row (a) — a trailing sentence period does not hide an address.
    #[test]
    #[should_panic(expected = "198.18.0.1 is not in an RFC 5737")]
    fn the_text_scanner_sees_an_ip_followed_by_a_period() {
        assert_text_is_synthetic("the host was seen at 198.18.0.1.", Path::new("in-memory"));
    }

    /// Row (b) — a trailing colon does not hide an address.
    #[test]
    #[should_panic(expected = "names 00:11:22:33:44:55, which is neither")]
    fn the_text_scanner_sees_a_mac_followed_by_a_colon() {
        assert_text_is_synthetic(
            "the port answers as 00:11:22:33:44:55: see the note below",
            Path::new("in-memory"),
        );
    }

    /// Row (c) — INTERIOR punctuation does not hide an address. This is the row an edge-trim
    /// does not reach: `198.18.0.1:8080` keeps its `:` inside the token.
    #[test]
    #[should_panic(expected = "198.18.0.1 is not in an RFC 5737")]
    fn the_text_scanner_sees_an_ip_carrying_a_port() {
        assert_text_is_synthetic(
            "the service is reachable at 198.18.0.1:8080 from the lab",
            Path::new("in-memory"),
        );
    }

    /// Row (d) — the dash form of a MAC is the same address. Closed by normalising `-` to `:`
    /// INSIDE the scanner: `MacAddr::from_str` stays colon-only (D47 — widening a domain parser
    /// for a test's convenience would change what the shipped connectors accept off the wire).
    #[test]
    #[should_panic(expected = "names 00:11:22:33:44:55, which is neither")]
    fn the_text_scanner_sees_a_dash_separated_mac() {
        assert_text_is_synthetic(
            "the port learned 00-11-22-33-44-55 on vlan 12",
            Path::new("in-memory"),
        );
    }

    /// `Observation.raw` is scanned by the same rule as everything else (story 5.2, AC2).
    ///
    /// This guard is PERMANENT rather than a recorded mutation, and the difference matters. The
    /// call site it defends is vacuous on the committed corpus — one non-null `raw` corpus-wide,
    /// carrying no address — so deleting that call site after merge would red nothing, and a
    /// mutation record is not a guard. The same reasoning put
    /// `the_text_scanner_admits_the_vrrp_range` in the tree for story 4.14.
    ///
    /// It drives `assert_record_is_synthetic`, the function the corpus walk drives, rather than
    /// re-typing the rule: a hand-built record walked through a hand-built copy of the check could
    /// agree with a bug in both. The record is hand-built because it MUST be — the walk's root is
    /// hardcoded and must stay so (story 5.1's callers depend on it), and `scratch_dir` produces a
    /// path under `std::env::temp_dir()` that no corpus walk can be pointed at.
    #[test]
    #[should_panic(expected = "198.18.0.1 is not in an RFC 5737")]
    fn an_observations_raw_payload_is_scanned() {
        let mut observation = expected()[0].clone();
        observation.raw = Some(r#"{"provenance":"seen at 198.18.0.1"}"#.into());
        assert_record_is_synthetic(&Record::Observation(observation), Path::new("in-memory"));
    }

    /// The floor the boundary anchor leaves, stated as a CHECK rather than as a doc sentence.
    ///
    /// `ab198.18.0.1` hides a non-documentation address from the scanner: `1` is neither a run
    /// start nor preceded by `.` or `:`, so no candidate ever begins there. This test is the only
    /// place the anchor's behaviour is observable — dropping the anchor reds it (the scanner would
    /// then see `198.18.0.1` and panic), while dropping it changes nothing else in the suite. An
    /// admitted limit with a guard behind it cannot rot into a false claim of coverage.
    ///
    /// **The control address is what makes this a check rather than a shrug.** A test that only
    /// says "this does not panic" is satisfied by a scanner that does nothing at all — the vacuity
    /// this story's own review found in it. `192.0.2.1` sits in the same string, unglued, and the
    /// assertion below says the scan saw THAT one and only that one: the glued address is invisible
    /// *while the scanner is demonstrably working*.
    #[test]
    fn the_text_scanner_is_blind_to_an_address_glued_to_hex() {
        let seen = assert_text_is_synthetic(
            "the label reads ab198.18.0.1 on the port, next to 192.0.2.1",
            Path::new("in-memory"),
        );
        assert_eq!(
            seen.ips,
            vec![Ipv4Addr::new(192, 0, 2, 1)],
            "the scan must have found the unglued control address and nothing else — finding \
             nothing would mean the tokenizer is dead, not that the anchor is working"
        );
    }

    /// Row (e) — an IPv6 solicited-node multicast MAC is locally administered (`0x33` has the
    /// U/L bit set) and was therefore ADMITTED. Its low THREE bytes are the low three bytes of a
    /// real IPv6 address, i.e. real interface-identifier bytes (the fourth from the end is the
    /// constant `ff`), so admitting it was a hole.
    #[test]
    #[should_panic(expected = "names 33:33:ff:00:60:0a, which is a MULTICAST address")]
    fn the_text_scanner_refuses_an_ipv6_multicast_mac() {
        assert_text_is_synthetic(
            "the capture shows 33:33:ff:00:60:0a on the segment",
            Path::new("in-memory"),
        );
    }

    /// Row (f) — an IPv4 multicast MAC was ALREADY refused (its U/L bit is clear), but for an
    /// unrelated reason. Only the stated reason changes here, so the `expected` substring names
    /// the NEW multicast sentence: matched against the old message this test proves nothing.
    #[test]
    #[should_panic(expected = "names 01:00:5e:00:00:0a, which is a MULTICAST address")]
    fn the_text_scanner_refuses_an_ipv4_multicast_mac_for_the_stated_reason() {
        assert_text_is_synthetic(
            "the capture shows 01:00:5e:00:00:0a on the segment",
            Path::new("in-memory"),
        );
    }

    /// Story 4.15's byte-pin — the second, independent oracle over `hostname-collision.jsonl`:
    /// one factory-default hostname worn by two distinct boxes (the collision, H1/H2) and by
    /// the same box re-seen an hour later (the plain re-sighting, H1/H3). Every value the two
    /// reasons cite is pinned here (the 4.13/4.14 review lesson, applied up front): the shared
    /// name, both MACs, both addresses, and the authored instants as an exact vector — a bare
    /// increase check would leave the one-hour re-sighting gap unpinned.
    #[test]
    fn the_hostname_collision_stream_shares_one_name_across_two_boxes() {
        let observations =
            read_jsonl(&fixture_path("scenario/replay/hostname-collision.jsonl").unwrap())
                .expect("the hostname-collision stream must read");
        assert_eq!(observations.len(), 3, "three authored presences, exactly");
        // Exactly 3 facts per line. Without this, `find()` below takes the FIRST match of each
        // kind and a duplicated or extra fact would pass every assertion unnoticed (4.13's
        // lesson, carried forward).
        for (n, observation) in observations.iter().enumerate() {
            assert_eq!(
                observation.facts.len(),
                3,
                "observation {n} carries exactly its three facts"
            );
        }
        // The obs_id ↔ line binding is pinned too (this story's review): the traps judge by
        // obs_id, this test reads by index — without these pins, swapping two lines' obs_ids
        // (with a re-hashed manifest) would silently invert what each trap judges while every
        // byte-level assertion stayed green.
        assert_obs_ids(&observations, "afafafaf", 3);

        let fact = |n: usize, pick: fn(&Fact) -> bool| {
            observations[n]
                .facts
                .iter()
                .find(|f| pick(f))
                .unwrap_or_else(|| panic!("observation {n} must carry the fact"))
                .clone()
        };
        let mac = |n| fact(n, |f| matches!(f, Fact::Mac { .. }));
        let ip = |n| fact(n, |f| matches!(f, Fact::IpV4 { .. }));
        let hostname = |n| fact(n, |f| matches!(f, Fact::Hostname { .. }));

        // The collision: one factory default on all three lines, value-pinned once and
        // equality-carried to the others.
        assert_eq!(
            hostname(0),
            Fact::Hostname {
                name: "doc-printer".into(),
                source: HostnameSource::Dhcp,
            },
            "H1 wears the factory default"
        );
        assert_eq!(
            hostname(1),
            hostname(0),
            "H2 wears the SAME name — the collision"
        );
        assert_eq!(
            hostname(2),
            hostname(0),
            "H3 wears the same name — the re-sighting"
        );

        // H1 and H2 are two real boxes: both ends value-pinned, and distinct.
        assert_eq!(
            mac(0),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 160]),
                locally_administered: true,
            },
            "H1's own MAC"
        );
        assert_eq!(
            mac(1),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 161]),
                locally_administered: true,
            },
            "H2's own MAC"
        );
        assert_ne!(mac(1), mac(0), "the colliding boxes keep their own MACs");
        assert_eq!(
            ip(0),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 50)
            },
            "H1's own address"
        );
        assert_eq!(
            ip(1),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 51)
            },
            "H2's own address"
        );
        assert_ne!(ip(1), ip(0), "and their own addresses");

        // H3 is H1 re-seen: byte-identical MAC and address — nothing moved, nothing opposes.
        assert_eq!(mac(2), mac(0), "H3 carries H1's exact MAC");
        assert_eq!(ip(2), ip(0), "H3 holds H1's exact address");

        // The instants, as an exact vector (pins the values AND the strict increase): the
        // re-sighting is one full hour later — a plain re-sighting, not a moved lease.
        let instants: Vec<Timestamp> = observations.iter().map(|o| o.observed_at).collect();
        assert_eq!(
            instants,
            vec![
                ts("2026-01-09T00:00:00Z"),
                ts("2026-01-09T00:05:00Z"),
                ts("2026-01-09T01:00:00Z"),
            ],
            "the collision and the re-sighting happen at the authored instants"
        );

        // Story 5.2b (AC4b), extended to this family by its code review — and this is the family
        // the review MEASURED. Exchanging the two `observations` vectors here left the whole
        // workspace suite green (135 + 86 + 42) while the corpus DEMANDED `must-merge`/`l1-exact-mac`
        // on two DIFFERENT MACs — two physically distinct boxes that merely share a hostname. That
        // is D10's catastrophic direction, and it was reachable from the `.toml` alone.
        let traps = read_traps(&fixture_path("scenario/traps/hostname-collision.toml").unwrap())
            .expect("the hostname-collision trap file must read");
        assert_eq!(
            traps.trap.len(),
            2,
            "the family declares exactly its two poles"
        );
        assert_trap_binds(
            &traps,
            "hostname-collision-must-not-merge",
            &[
                "afafafaf-0000-4000-8000-000000000001",
                "afafafaf-0000-4000-8000-000000000002",
            ],
            not_merge("l1-distinct-mac"),
            Some("hostname-collision"),
        );
        assert_trap_binds(
            &traps,
            "hostname-collision-must-merge",
            &[
                "afafafaf-0000-4000-8000-000000000001",
                "afafafaf-0000-4000-8000-000000000003",
            ],
            merge("l1-exact-mac"),
            Some("hostname-collision"),
        );
    }

    /// Story 4.14's flag-vs-bytes guard, red-proven in memory: a locally-administered byte
    /// pattern whose authored flag lies (`false`) must panic. The mis-paired fact is built
    /// here and never committed — the committed corpus is walked by
    /// `the_corpus_carries_no_real_network_data`, where this same assertion holds every stream.
    #[test]
    #[should_panic(expected = "authored locally_administered flag contradicts its own U/L bit")]
    fn a_mac_whose_flag_contradicts_its_bytes_is_refused() {
        assert_facts_are_synthetic(
            &[Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 99]),
                locally_administered: false,
            }],
            Path::new("in-memory"),
        );
    }

    /// Story 4.17's boundary on the amended hostname rule: an empty name is admitted (the
    /// measured EMPTY form), but a non-empty, non-`doc-` name still reds — the expected
    /// substring guards against a pass-for-the-wrong-panic. In-memory only, never committed.
    #[test]
    #[should_panic(expected = "hostnames must be invented")]
    fn a_captured_looking_hostname_is_still_refused() {
        assert_facts_are_synthetic(
            &[Fact::Hostname {
                name: "printer-salon".into(),
                source: HostnameSource::Dhcp,
            }],
            Path::new("in-memory"),
        );
    }

    /// Story 4.16's byte-pin — the second, independent oracle over `docker-veth.jsonl`: a
    /// docker host whose container veth appears (E2), vanishes, and is SUCCEEDED by a new veth
    /// wearing the recycled container address (E4), while the host itself is re-seen unchanged
    /// (E3 — the authored evidence that the window stayed open and the first veth failed to
    /// reappear; NFR7 forbids an absence fact, so disappearance can only be authored this way).
    /// Every value the two reasons cite is pinned, the obs_id ↔ line binding included (4.15's
    /// rule), and E3 is pinned VALUE-identical to E1 fact-by-fact (parsed equality per kind —
    /// fact order and raw bytes are the corpus lock's business) so it cannot drift into a
    /// third device.
    #[test]
    fn the_docker_veth_stream_replaces_its_veth_without_replacing_its_host() {
        let observations = read_jsonl(&fixture_path("scenario/replay/docker-veth.jsonl").unwrap())
            .expect("the docker-veth stream must read");
        assert_eq!(observations.len(), 4, "four authored presences, exactly");
        // Exact fact counts per line (the `find()` guard, 4.13's lesson): the host lines carry
        // Mac+IpV4+Hostname+Uplink, the veth lines Mac+IpV4+Uplink — a veth answers ARP, it
        // resolves no name.
        for (n, expected_len) in [(0, 4), (1, 3), (2, 4), (3, 3)] {
            assert_eq!(
                observations[n].facts.len(),
                expected_len,
                "observation {n} carries exactly its facts"
            );
        }
        // The obs_id ↔ line binding (4.15's review rule): the traps judge by obs_id.
        assert_obs_ids(&observations, "babababa", 4);

        let fact = |n: usize, pick: fn(&Fact) -> bool| {
            observations[n]
                .facts
                .iter()
                .find(|f| pick(f))
                .unwrap_or_else(|| panic!("observation {n} must carry the fact"))
                .clone()
        };
        let mac = |n| fact(n, |f| matches!(f, Fact::Mac { .. }));
        let ip = |n| fact(n, |f| matches!(f, Fact::IpV4 { .. }));
        let hostname = |n| fact(n, |f| matches!(f, Fact::Hostname { .. }));
        let uplink = |n| fact(n, |f| matches!(f, Fact::Uplink { .. }));

        // E1, the docker host — all four facts value-pinned.
        assert_eq!(
            mac(0),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 180]),
                locally_administered: true,
            },
            "the host's stable MAC"
        );
        assert_eq!(
            ip(0),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 60)
            },
            "the host's own address"
        );
        assert_eq!(
            hostname(0),
            Fact::Hostname {
                name: "doc-dockerhost".into(),
                source: HostnameSource::Dhcp,
            },
            "the host's name"
        );
        assert_eq!(
            uplink(0),
            Fact::Uplink {
                peer_mac: MacAddr([2, 0, 94, 0, 96, 10]),
                peer_port: "swport-21".into(),
            },
            "the host's own switch port"
        );

        // E3 is the host RE-SEEN — value-identical fact-by-fact, so the unreferenced
        // observation cannot drift into a third device.
        assert_eq!(mac(2), mac(0), "E3 carries the host's exact MAC");
        assert_eq!(ip(2), ip(0), "E3 holds the host's exact address");
        assert_eq!(
            hostname(2),
            hostname(0),
            "E3 resolves the host's exact name"
        );
        assert_eq!(uplink(2), uplink(0), "E3 sits on the host's exact port");

        // E2, the first veth — its own MAC, the container address, the HOST's uplink (bridged
        // traffic exits through the host's own port).
        assert_eq!(
            mac(1),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 181]),
                locally_administered: true,
            },
            "the first veth's fresh MAC"
        );
        assert_eq!(
            ip(1),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 61)
            },
            "the container address"
        );
        assert_eq!(
            uplink(1),
            uplink(0),
            "the veth shares the host's exact uplink"
        );

        // E4, the successor veth — a NEW MAC, the RECYCLED container address, the same uplink.
        assert_eq!(
            mac(3),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 182]),
                locally_administered: true,
            },
            "the successor's fresh MAC"
        );
        assert_ne!(
            mac(3),
            mac(1),
            "the successor is not the first veth re-seen"
        );
        assert_eq!(
            ip(3),
            ip(1),
            "the successor wears the recycled container address"
        );
        assert_eq!(
            uplink(3),
            uplink(1),
            "the successor exits through the same port"
        );

        // The instants, as an exact vector: the first veth appears at 00:05 and by 01:00 the
        // host is re-seen WITHOUT it — the disappearance happens inside an observably open
        // window; the successor arrives an hour after the first.
        let instants: Vec<Timestamp> = observations.iter().map(|o| o.observed_at).collect();
        assert_eq!(
            instants,
            vec![
                ts("2026-01-10T00:00:00Z"),
                ts("2026-01-10T00:05:00Z"),
                ts("2026-01-10T01:00:00Z"),
                ts("2026-01-10T01:05:00Z"),
            ],
            "the succession happens at the authored instants"
        );

        // Story 5.2b (AC4b), extended to this family by its code review — the TOML side. This
        // family's two poles do NOT share an endpoint (`[001,002]` vs `[002,004]`), which is why
        // the ORDER pin matters less here than the vector pin: an exchange moves both endpoints.
        let traps = read_traps(&fixture_path("scenario/traps/docker-veth.toml").unwrap())
            .expect("the docker-veth trap file must read");
        assert_eq!(
            traps.trap.len(),
            2,
            "the family declares exactly its two poles"
        );
        assert_trap_binds(
            &traps,
            "docker-veth-must-merge",
            &[
                "babababa-0000-4000-8000-000000000001",
                "babababa-0000-4000-8000-000000000002",
            ],
            merge("l2-uplink-agrees"),
            Some("docker-veth"),
        );
        assert_trap_binds(
            &traps,
            "docker-veth-must-not-merge",
            &[
                "babababa-0000-4000-8000-000000000002",
                "babababa-0000-4000-8000-000000000004",
            ],
            not_merge("l1-distinct-mac"),
            Some("docker-veth"),
        );
    }

    /// Story 4.17's byte-pin — the second, independent oracle over `hostname-absence.jsonl`:
    /// the two shapes the measured source actually produces for "no hostname" — a byte-present
    /// EMPTY name and a MISSING `Hostname` fact — and never `null`, which `Fact::Hostname`'s
    /// `String` cannot even represent. Three pairs pin one equivalence: `"" == ""` is not
    /// agreement (G1/G2), empty counts as no-observed-value (G3/G4), and a name that fell
    /// silent opposes nothing (G5/G6). Every value the reasons cite is pinned, the MISSING
    /// form is an assertion (not an accident), and the obs_id ↔ line binding holds.
    #[test]
    fn the_hostname_absence_stream_encodes_empty_and_missing_and_never_null() {
        let observations =
            read_jsonl(&fixture_path("scenario/replay/hostname-absence.jsonl").unwrap())
                .expect("the hostname-absence stream must read");
        assert_eq!(observations.len(), 6, "six authored presences, exactly");
        // Exact fact counts per line (the `find()` guard): together with the per-kind
        // extractions below, each line's fact multiset is fully determined.
        for (n, expected_len) in [(0, 3), (1, 3), (2, 4), (3, 3), (4, 3), (5, 2)] {
            assert_eq!(
                observations[n].facts.len(),
                expected_len,
                "observation {n} carries exactly its facts"
            );
        }
        // The obs_id ↔ line binding (the standing rule): the traps judge by obs_id.
        assert_obs_ids(&observations, "bcbcbcbc", 6);

        let fact = |n: usize, pick: fn(&Fact) -> bool| {
            observations[n]
                .facts
                .iter()
                .find(|f| pick(f))
                .unwrap_or_else(|| panic!("observation {n} must carry the fact"))
                .clone()
        };
        let mac = |n| fact(n, |f| matches!(f, Fact::Mac { .. }));
        let ip = |n| fact(n, |f| matches!(f, Fact::IpV4 { .. }));
        let hostname = |n| fact(n, |f| matches!(f, Fact::Hostname { .. }));
        let uplink = |n| fact(n, |f| matches!(f, Fact::Uplink { .. }));
        let has_no_hostname = |n: usize| {
            observations[n]
                .facts
                .iter()
                .all(|f| !matches!(f, Fact::Hostname { .. }))
        };

        // The EMPTY form, three times: value-pinned once, equality-carried to the others.
        assert_eq!(
            hostname(0),
            Fact::Hostname {
                name: "".into(),
                source: HostnameSource::Dhcp,
            },
            "G1 reports the honestly empty name"
        );
        assert_eq!(hostname(1), hostname(0), "G2 reports the SAME empty name");
        assert_eq!(hostname(2), hostname(0), "G3 reports the same empty name");

        // The MISSING form is an assertion, not an accident.
        assert!(has_no_hostname(3), "G4 carries NO Hostname fact — MISSING");
        assert!(
            has_no_hostname(5),
            "G6 carries NO Hostname fact — the name fell silent"
        );

        // G1/G2 — the false-agreement pair: both ends value-pinned, and distinct.
        assert_eq!(
            mac(0),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 200]),
                locally_administered: true,
            },
            "G1's own MAC"
        );
        assert_eq!(
            mac(1),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 201]),
                locally_administered: true,
            },
            "G2's own MAC"
        );
        assert_ne!(
            mac(1),
            mac(0),
            "the two empty-named boxes keep their own MACs"
        );
        assert_eq!(
            ip(0),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 70)
            },
            "G1's own address"
        );
        assert_eq!(
            ip(1),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 71)
            },
            "G2's own address"
        );
        assert_ne!(ip(1), ip(0), "and their own addresses");

        // G3/G4 — the equivalence pair: MACs and IPs pinned and distinct too (an accidental
        // shared MAC would collapse the abstain pair into an exact-MAC pair), and the WHOLE
        // Uplink fact identical and pinned — the shared port is what makes the hostname the
        // needed discriminator.
        assert_eq!(
            mac(2),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 202]),
                locally_administered: true,
            },
            "G3's own MAC"
        );
        assert_eq!(
            mac(3),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 203]),
                locally_administered: true,
            },
            "G4's own MAC"
        );
        assert_ne!(mac(3), mac(2), "the abstain pair is two boxes, not one");
        assert_eq!(
            ip(2),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 72)
            },
            "G3's own address"
        );
        assert_eq!(
            ip(3),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 73)
            },
            "G4's own address"
        );
        assert_ne!(ip(3), ip(2), "and their own addresses");
        assert_eq!(
            uplink(2),
            Fact::Uplink {
                peer_mac: MacAddr([2, 0, 94, 0, 96, 10]),
                peer_port: "swport-31".into(),
            },
            "G3 sits behind the shared port"
        );
        assert_eq!(uplink(3), uplink(2), "G4 sits behind the SAME port");

        // G5/G6 — the silence pair: the named box, then the same box with no name at all.
        assert_eq!(
            mac(4),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 204]),
                locally_administered: true,
            },
            "G5's own MAC"
        );
        assert_eq!(
            ip(4),
            Fact::IpV4 {
                addr: Ipv4Addr::new(192, 0, 2, 74)
            },
            "G5's own address"
        );
        assert_eq!(
            hostname(4),
            Fact::Hostname {
                name: "doc-host-india".into(),
                source: HostnameSource::Dhcp,
            },
            "G5 resolves its name"
        );
        assert_eq!(mac(5), mac(4), "G6 carries G5's exact MAC");
        assert_eq!(ip(5), ip(4), "G6 holds G5's exact address");

        // The instants, as an exact vector: the silence arrives forty minutes after the name.
        let instants: Vec<Timestamp> = observations.iter().map(|o| o.observed_at).collect();
        assert_eq!(
            instants,
            vec![
                ts("2026-01-11T00:00:00Z"),
                ts("2026-01-11T00:05:00Z"),
                ts("2026-01-11T00:10:00Z"),
                ts("2026-01-11T00:15:00Z"),
                ts("2026-01-11T00:20:00Z"),
                ts("2026-01-11T01:00:00Z"),
            ],
            "the absences happen at the authored instants"
        );

        // Story 5.2b (AC4b), extended to this family by its code review — the TOML side. Three
        // DISJOINT pairs here (G1/G2, G3/G4, G5/G6), one per column, so an exchange between any two
        // poles re-points a whole pair at the wrong equivalence: the false-agreement pair judged as
        // an abstention, or the honestly-silent pair judged as a refusal.
        let traps = read_traps(&fixture_path("scenario/traps/hostname-absence.toml").unwrap())
            .expect("the hostname-absence trap file must read");
        assert_eq!(
            traps.trap.len(),
            3,
            "the family declares exactly its three traps"
        );
        assert_trap_binds(
            &traps,
            "hostname-absence-must-not-merge",
            &[
                "bcbcbcbc-0000-4000-8000-000000000001",
                "bcbcbcbc-0000-4000-8000-000000000002",
            ],
            not_merge("l1-distinct-mac"),
            Some("hostname-absence"),
        );
        assert_trap_binds(
            &traps,
            "hostname-absence-must-abstain",
            &[
                "bcbcbcbc-0000-4000-8000-000000000003",
                "bcbcbcbc-0000-4000-8000-000000000004",
            ],
            Expectation::MustAbstain {
                cause: AbstentionCause::NoObservedValue,
            },
            Some("hostname-absence"),
        );
        assert_trap_binds(
            &traps,
            "hostname-absence-must-merge",
            &[
                "bcbcbcbc-0000-4000-8000-000000000005",
                "bcbcbcbc-0000-4000-8000-000000000006",
            ],
            merge("l1-exact-mac"),
            Some("hostname-absence"),
        );
    }

    /// Story 5.13b's byte-pin over the blinded-source TWIN PAIR — the family whose stream carries
    /// a `capability` control record, and whose two poles both STRADDLE it.
    ///
    /// # ⚠️ What reds without this pin, and what does NOT
    ///
    /// Story 5.2b's argument — *"exchanging the two `observations` vectors leaves the whole suite
    /// green while making the corpus demand a false merge"* — was measured TWICE and is **stale for
    /// this family**. Since story 5.7 the committed corpus is scored by the real engine, and here
    /// the swap would ask `l1-exact-mac` to merge two DISTINCT MACs, which the engine refuses: the
    /// trap gate reds on its own, before any pin is consulted. So this pin is a **second oracle,
    /// not the sole carrier**, and saying otherwise would be a claim outrunning its measurement —
    /// the defect this corpus exists to catch.
    ///
    /// It still earns its place: it pins WHICH pair each pole judges (the gate only knows that
    /// *some* answer disagreed), and it pins `family`, whose loss is measured to exempt the family
    /// from `incomplete_families` in silence.
    #[test]
    fn the_blinded_source_pair_pins_its_twin_relation_and_its_two_poles() {
        // The stream side: the twins carry the same MACs, and the faulted one carries FEWER facts.
        // `fault_injection`'s twin guard is what proves the derivation; this states the premise
        // that guard rests on, in `expected()`'s idiom — read from the bytes, not from the guard.
        let clean = read_jsonl(&fixture_path("scenario/replay/blinded-source.jsonl").unwrap())
            .expect("the clean twin must read");
        let faulted =
            read_jsonl(&fixture_path("scenario/replay/blinded-source-blinded.jsonl").unwrap())
                .expect("the faulted twin must read");

        assert_eq!(clean.len(), 4, "the clean twin carries four observations");
        assert_eq!(
            faulted.len(),
            clean.len(),
            "and so does the faulted one — the blinding removes FACTS, never an observation"
        );
        assert_eq!(
            (
                clean.iter().map(|o| o.facts.len()).sum::<usize>(),
                faulted.iter().map(|o| o.facts.len()).sum::<usize>()
            ),
            (12, 10),
            "twelve facts against ten: the two observations after the capability record lose their \
             Rtt, and nothing else moves"
        );
        let mac = |o: &Observation| {
            o.facts
                .iter()
                .find_map(|f| match f {
                    Fact::Mac { addr, .. } => Some(*addr),
                    _ => None,
                })
                .expect("every observation in this pair carries a MAC")
        };
        assert_eq!(
            mac(&faulted[0]),
            mac(&faulted[2]),
            "the must-merge pole's pair shares its MAC ACROSS the capability record — the premise \
             of the whole family"
        );
        assert_ne!(
            mac(&faulted[0]),
            mac(&faulted[3]),
            "and the must-not-merge pole's pair does not, which is what OPPOSES that merge"
        );

        // The TOML side: which pair each pole judges.
        let traps = read_traps(&fixture_path("scenario/traps/blinded-source.toml").unwrap())
            .expect("the blinded-source trap file must read");
        assert_eq!(
            traps.trap.len(),
            2,
            "the family declares exactly its two poles"
        );
        assert_trap_binds(
            &traps,
            "blinded-source-must-merge",
            &[
                "bebebebe-0000-4000-8000-000000000001",
                "bebebebe-0000-4000-8000-000000000003",
            ],
            merge("l1-exact-mac"),
            Some("blinded-source"),
        );
        assert_trap_binds(
            &traps,
            "blinded-source-must-not-merge",
            &[
                "bebebebe-0000-4000-8000-000000000001",
                "bebebebe-0000-4000-8000-000000000004",
            ],
            not_merge("l1-distinct-mac"),
            Some("blinded-source"),
        );
    }

    /// Story 5.2b's byte-pin over `randomized-mac.jsonl` — the family whose entire
    /// discrimination rests on ONE octet, in the spirit of `expected()`. Its `must-merge` pole
    /// (`l1-exact-mac`, judging 001+002) and its `must-not-merge` pole (`l1-distinct-mac`,
    /// judging 001+003) differ by the last byte of one MAC and nothing else.
    ///
    /// **Why every MAC is pinned by VALUE and not by `assert_eq!(mac(0), mac(1))`.** A relational
    /// pin stays green if BOTH N1 and N2 are re-authored to `…:21` — at which point the
    /// `must-not-merge` pair becomes a same-MAC pair, the corpus demands the opposite decision,
    /// and nothing says so. That is the failure this test exists to stop, and it is why story
    /// 4.13's relational-only pin was an open register item.
    ///
    /// Before this test the stream was named by no value test at all: its only mention was the
    /// per-stream context table story 5.1 added (`fixture_connector.rs`), which states the
    /// stream's declared CONTEXT and asserts nothing about its contents, while `read_traps`
    /// checks only that a trap's `obs_id`s EXIST — never which line they name.
    #[test]
    fn the_randomized_mac_stream_rests_on_one_octet() {
        let observations =
            read_jsonl(&fixture_path("scenario/replay/randomized-mac.jsonl").unwrap())
                .expect("the randomized-mac stream must read");
        assert_eq!(observations.len(), 3, "three authored presences, exactly");
        // Exactly 2 facts per line (the `find()` guard): uniform here, which is what lets the
        // one-of-each extraction below pin each line's fact multiset exactly. Without it, a
        // duplicated or extra fact would pass every assertion unnoticed.
        for (n, observation) in observations.iter().enumerate() {
            assert_eq!(
                observation.facts.len(),
                2,
                "observation {n} carries exactly its two facts"
            );
        }
        // The obs_id ↔ line binding (the standing rule): the traps judge by obs_id.
        assert_obs_ids(&observations, "eeeeeeee", 3);

        let fact = |n: usize, pick: fn(&Fact) -> bool| {
            observations[n]
                .facts
                .iter()
                .find(|f| pick(f))
                .unwrap_or_else(|| panic!("observation {n} must carry the fact"))
                .clone()
        };
        let mac = |n| fact(n, |f| matches!(f, Fact::Mac { .. }));
        let ip = |n| fact(n, |f| matches!(f, Fact::IpV4 { .. }));

        // The one octet, pinned on EACH of the three lines — not pairwise. The shared constant is
        // the MAC N1 and N2 wear BEFORE the re-randomization; N3 is the re-randomized one, which
        // is why it is named for what it holds rather than for the event.
        let before_re_randomization = Fact::Mac {
            addr: MacAddr([2, 0, 94, 0, 83, 32]),
            locally_administered: true,
        };
        assert_eq!(
            mac(0),
            before_re_randomization,
            "N1 wears 02:00:5e:00:53:20, the MAC the must-merge reason cites"
        );
        assert_eq!(
            mac(1),
            before_re_randomization,
            "N2 wears the byte-identical 02:00:5e:00:53:20 an hour later"
        );
        assert_eq!(
            mac(2),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 33]),
                locally_administered: true,
            },
            "N3 wears 02:00:5e:00:53:21 — the ONE octet the must-not-merge pole rests on"
        );

        // Three distinct leases, value-pinned.
        for (n, last) in [(0, 30u8), (1, 31), (2, 32)] {
            assert_eq!(
                ip(n),
                Fact::IpV4 {
                    addr: Ipv4Addr::new(192, 0, 2, last)
                },
                "observation {n} holds its authored address 192.0.2.{last}"
            );
        }

        // The authored instants, an hour apart. Strictly increasing HERE — unlike the three
        // families below, where two NICs seen in one sweep deliberately share an instant.
        let instants: Vec<Timestamp> = observations.iter().map(|o| o.observed_at).collect();
        assert_eq!(
            instants,
            vec![
                ts("2026-01-02T00:00:00Z"),
                ts("2026-01-02T01:00:00Z"),
                ts("2026-01-02T02:00:00Z"),
            ],
            "the re-randomization happens BETWEEN observations, as authored time"
        );

        // AC4b — the TOML side: which pair each pole judges, and under which column and rule.
        let traps = read_traps(&fixture_path("scenario/traps/randomized-mac.toml").unwrap())
            .expect("the randomized-mac trap file must read");
        assert_eq!(
            traps.trap.len(),
            2,
            "the family declares exactly its two poles"
        );
        assert_trap_binds(
            &traps,
            "randomized-mac-must-not-merge",
            &[
                "eeeeeeee-0000-4000-8000-000000000001",
                "eeeeeeee-0000-4000-8000-000000000003",
            ],
            not_merge("l1-distinct-mac"),
            Some("randomized-mac"),
        );
        assert_trap_binds(
            &traps,
            "randomized-mac-must-merge",
            &[
                "eeeeeeee-0000-4000-8000-000000000001",
                "eeeeeeee-0000-4000-8000-000000000002",
            ],
            merge("l1-exact-mac"),
            Some("randomized-mac"),
        );
    }

    /// Story 5.2b's byte-pin over `multi-nic.jsonl` — the family whose premise is entirely
    /// geometric, and which the harness validated NOWHERE. The VRRP byte-pin's own doc says
    /// uplink geometry is pinned *"here or nowhere"*, and that was true precisely because VRRP
    /// had a byte-pin; this stream had none.
    ///
    /// **BOTH halves of the `Uplink` fact are pinned on every line.** The two poles are
    /// `must-merge`/`l2-uplink-agrees` on 001+002 (same switch, DIFFERENT port — an uplink that
    /// AGREES) and `must-not-merge`/`l2-different-switch` on 001+003. Pinning only `peer_mac`
    /// would let *"same switch, different port = agrees"* and *"different switch = opposes"* be
    /// silently exchanged. And collapsing M2's port onto M1's must RED: that edit turns this
    /// family into the `shared-hardware-vm` shape, where an identical uplink is exactly what does
    /// NOT discriminate.
    #[test]
    fn the_multi_nic_stream_pins_both_halves_of_its_uplink() {
        let observations = read_jsonl(&fixture_path("scenario/replay/multi-nic.jsonl").unwrap())
            .expect("the multi-nic stream must read");
        assert_eq!(observations.len(), 3, "three authored presences, exactly");
        // Exactly 3 facts per line (the `find()` guard) — see the randomized-mac pin.
        for (n, observation) in observations.iter().enumerate() {
            assert_eq!(
                observation.facts.len(),
                3,
                "observation {n} carries exactly its three facts"
            );
        }
        assert_obs_ids(&observations, "ffffffff", 3);

        let fact = |n: usize, pick: fn(&Fact) -> bool| {
            observations[n]
                .facts
                .iter()
                .find(|f| pick(f))
                .unwrap_or_else(|| panic!("observation {n} must carry the fact"))
                .clone()
        };
        let mac = |n| fact(n, |f| matches!(f, Fact::Mac { .. }));
        let ip = |n| fact(n, |f| matches!(f, Fact::IpV4 { .. }));
        let uplink = |n| fact(n, |f| matches!(f, Fact::Uplink { .. }));

        // The geometry, both halves, on each line. M1 and M2 share the access switch and differ
        // by PORT — that pair is the `must-merge` pole. M3 hangs off another switch entirely.
        assert_eq!(
            uplink(0),
            Fact::Uplink {
                peer_mac: MacAddr([2, 0, 94, 0, 96, 10]),
                peer_port: "swport-1".into(),
            },
            "M1 hangs off 02:00:5e:00:60:0a port swport-1"
        );
        assert_eq!(
            uplink(1),
            Fact::Uplink {
                peer_mac: MacAddr([2, 0, 94, 0, 96, 10]),
                peer_port: "swport-2".into(),
            },
            "M2 hangs off the SAME switch on a DIFFERENT port — the uplink that agrees"
        );
        assert_eq!(
            uplink(2),
            Fact::Uplink {
                peer_mac: MacAddr([2, 0, 94, 0, 96, 11]),
                peer_port: "swport-7".into(),
            },
            "M3 hangs off 02:00:5e:00:60:0b — a different switch, which OPPOSES the L2 join"
        );

        // The three host NICs keep their own distinct MACs — L1 is right to hold them apart, and
        // the false split this family catches lives at L2.
        for (n, last) in [(0, 64u8), (1, 65), (2, 66)] {
            assert_eq!(
                mac(n),
                Fact::Mac {
                    addr: MacAddr([2, 0, 94, 0, 83, last]),
                    locally_administered: true,
                },
                "observation {n} wears its own authored MAC"
            );
        }
        for (n, last) in [(0, 40u8), (1, 41), (2, 42)] {
            assert_eq!(
                ip(n),
                Fact::IpV4 {
                    addr: Ipv4Addr::new(192, 0, 2, last)
                },
                "observation {n} holds its authored address 192.0.2.{last}"
            );
        }

        // The authored instants — M1 and M2 SHARE one, deliberately: two NICs of one host seen in
        // the same sweep should. Pin the vector as authored; do NOT assert strict increase, which
        // is dhcp-churn's assertion and specific to a family whose churn lives in time alone.
        let instants: Vec<Timestamp> = observations.iter().map(|o| o.observed_at).collect();
        assert_eq!(
            instants,
            vec![
                ts("2026-01-03T00:00:00Z"),
                ts("2026-01-03T00:00:00Z"),
                ts("2026-01-03T00:05:00Z"),
            ],
            "M1 and M2 are seen in one sweep, as authored"
        );

        // AC4b — the TOML side.
        let traps = read_traps(&fixture_path("scenario/traps/multi-nic.toml").unwrap())
            .expect("the multi-nic trap file must read");
        assert_eq!(
            traps.trap.len(),
            2,
            "the family declares exactly its two poles"
        );
        assert_trap_binds(
            &traps,
            "multi-nic-must-merge",
            &[
                "ffffffff-0000-4000-8000-000000000001",
                "ffffffff-0000-4000-8000-000000000002",
            ],
            merge("l2-uplink-agrees"),
            Some("multi-nic"),
        );
        assert_trap_binds(
            &traps,
            "multi-nic-must-not-merge",
            &[
                "ffffffff-0000-4000-8000-000000000001",
                "ffffffff-0000-4000-8000-000000000003",
            ],
            not_merge("l2-different-switch"),
            Some("multi-nic"),
        );
    }

    /// Story 5.2b's byte-pin over `shared-hardware-vm.jsonl` — the family whose trap header
    /// declares the uplink *"shared by construction (the same `peer_mac` and `peer_port` on every
    /// observation)"*, prose that no test asserted.
    ///
    /// **Pinning the identical uplink is what keeps the discriminator the HOSTNAME.** Were the
    /// uplink allowed to drift, the `must-merge` pole could start passing for a topological
    /// reason this family explicitly denies — it is distinguished by hostname, and the shared
    /// hypervisor uplink is the temptation, not the evidence.
    ///
    /// **W4's ABSENT hostname is an assertion, not an accident** — story 4.17's idiom, asserted
    /// directly rather than inferred from a fact count. W4 is the `must-abstain` pole: no
    /// observed value distinguishes a second NIC of `doc-vm-alpha` from a new co-tenant VM.
    ///
    /// ⚠️ The fact count here is NOT uniform — 4, 4, 4, 3 — so the per-line vector is asserted
    /// exactly. `>= 3` would be vacuous, and that is measured: adding a SECOND, contradicting
    /// `Uplink` to W4 reds nothing under it, because the one-of-each `find()` returns the FIRST
    /// match — the authored one — so the value pin passes while *"shared by construction"* is
    /// false on the very pole that depends on it. The non-uniform count is the REASON the exact
    /// vector is needed, not a reason to relax it.
    #[test]
    fn the_shared_hardware_vm_stream_shares_one_uplink_and_falls_silent_on_the_abstain_pole() {
        let observations =
            read_jsonl(&fixture_path("scenario/replay/shared-hardware-vm.jsonl").unwrap())
                .expect("the shared-hardware-vm stream must read");
        assert_eq!(observations.len(), 4, "four authored presences, exactly");
        // Exact fact counts per line: W1–W3 are full VM presences (Mac, IpV4, Hostname, Uplink),
        // W4 is the hostless abstain pole. A blanket `assert_eq!(len, 4)` REDS on W4 and reads as
        // a corpus defect; it is not one, it is the premise.
        for (n, expected_len) in [(0, 4), (1, 4), (2, 4), (3, 3)] {
            assert_eq!(
                observations[n].facts.len(),
                expected_len,
                "observation {n} carries exactly its facts"
            );
        }
        assert_obs_ids(&observations, "abababab", 4);

        let fact = |n: usize, pick: fn(&Fact) -> bool| {
            observations[n]
                .facts
                .iter()
                .find(|f| pick(f))
                .unwrap_or_else(|| panic!("observation {n} must carry the fact"))
                .clone()
        };
        let mac = |n| fact(n, |f| matches!(f, Fact::Mac { .. }));
        let ip = |n| fact(n, |f| matches!(f, Fact::IpV4 { .. }));
        let hostname = |n| fact(n, |f| matches!(f, Fact::Hostname { .. }));
        let uplink = |n| fact(n, |f| matches!(f, Fact::Uplink { .. }));

        // The MISSING form is an assertion, not an accident (story 4.17's idiom). It is asserted
        // FIRST, ahead of the uplink loop below, on purpose: W4's silence is what makes this
        // family's third column exist, and the loop's one-of-each `find()` PANICS on a line whose
        // `Uplink` was replaced — so with the loop first, the mutation that removes W4's uplink in
        // favour of a hostname would red the uplink pin and never reach this one. Ordered this
        // way, that mutation reds THIS assertion and nothing else, which is what proves it has
        // teeth (story 5.2b, AC6).
        assert!(
            observations[3]
                .facts
                .iter()
                .all(|f| !matches!(f, Fact::Hostname { .. })),
            "W4 carries NO Hostname fact — it is the must-abstain pole, and its silence is the premise"
        );

        // The hypervisor's uplink, byte-identical on ALL FOUR lines — shared by construction.
        let hypervisor = Fact::Uplink {
            peer_mac: MacAddr([2, 0, 94, 0, 96, 10]),
            peer_port: "swport-1".into(),
        };
        for n in 0..4 {
            assert_eq!(
                uplink(n),
                hypervisor,
                "observation {n} hangs off the shared hypervisor uplink, by construction"
            );
        }

        // The discriminator: two VMs, one of them dual-homed.
        let alpha = Fact::Hostname {
            name: "doc-vm-alpha".into(),
            source: HostnameSource::Dhcp,
        };
        assert_eq!(hostname(0), alpha, "W1 answers to doc-vm-alpha");
        assert_eq!(
            hostname(1),
            alpha,
            "W2 answers to the SAME doc-vm-alpha — the must-merge pole"
        );
        assert_eq!(
            hostname(2),
            Fact::Hostname {
                name: "doc-vm-beta".into(),
                source: HostnameSource::Dhcp,
            },
            "W3 answers to doc-vm-beta — the co-tenant the must-not-merge pole names"
        );

        // Four distinct virtual MACs and four distinct addresses. TWO loops, not one: the MAC's
        // final octet (0x50..0x53 = 80..83) and the address's (.80..83) coincide here only by
        // accident of authoring — every other family in this module has them differ. Driving both
        // from one variable would encode a relationship the corpus does not promise, and these
        // pins are a second oracle over the bytes, not a restatement of one number.
        for (n, last) in [(0, 80u8), (1, 81), (2, 82), (3, 83)] {
            assert_eq!(
                mac(n),
                Fact::Mac {
                    addr: MacAddr([2, 0, 94, 0, 83, last]),
                    locally_administered: true,
                },
                "observation {n} wears its own authored virtual MAC"
            );
        }
        for (n, last) in [(0, 80u8), (1, 81), (2, 82), (3, 83)] {
            assert_eq!(
                ip(n),
                Fact::IpV4 {
                    addr: Ipv4Addr::new(192, 0, 2, last)
                },
                "observation {n} holds its authored address 192.0.2.{last}"
            );
        }

        // The authored instants — W1 and W2 share one: one VM's two NICs, one sweep.
        let instants: Vec<Timestamp> = observations.iter().map(|o| o.observed_at).collect();
        assert_eq!(
            instants,
            vec![
                ts("2026-01-04T00:00:00Z"),
                ts("2026-01-04T00:00:00Z"),
                ts("2026-01-04T00:05:00Z"),
                ts("2026-01-04T00:10:00Z"),
            ],
            "W1 and W2 are seen in one sweep, as authored"
        );

        // AC4b — the TOML side. This is the corpus's first three-column family: both poles keep
        // it complete, and the abstain rides along on the SAME scenario (DR1).
        let traps = read_traps(&fixture_path("scenario/traps/shared-hardware-vm.toml").unwrap())
            .expect("the shared-hardware-vm trap file must read");
        assert_eq!(
            traps.trap.len(),
            3,
            "the family declares exactly its three traps"
        );
        assert_trap_binds(
            &traps,
            "shared-hardware-vm-must-merge",
            &[
                "abababab-0000-4000-8000-000000000001",
                "abababab-0000-4000-8000-000000000002",
            ],
            merge("l2-hostname-agrees"),
            Some("shared-hardware-vm"),
        );
        assert_trap_binds(
            &traps,
            "shared-hardware-vm-must-not-merge",
            &[
                "abababab-0000-4000-8000-000000000001",
                "abababab-0000-4000-8000-000000000003",
            ],
            not_merge("l2-different-hostname"),
            Some("shared-hardware-vm"),
        );
        assert_trap_binds(
            &traps,
            "shared-hardware-vm-must-abstain",
            &[
                "abababab-0000-4000-8000-000000000001",
                "abababab-0000-4000-8000-000000000004",
            ],
            Expectation::MustAbstain {
                cause: AbstentionCause::NoObservedValue,
            },
            Some("shared-hardware-vm"),
        );
    }

    /// Story 5.2b's byte-pin over `cloned-mac.jsonl` — the INVERSE family, and the corpus's ONLY
    /// pre-release guard against the false MERGE. D21 refuses a unique index on
    /// `interface.mac_canon` deliberately (*"a UNIQUE would turn the exact case we must ABSTAIN
    /// on into a 500"*), so the schema cannot be that guard; D10 calls the false merge
    /// catastrophic and asymmetric.
    ///
    /// **The one MAC is pinned on each of the three lines, not pairwise.** A one-octet edit to
    /// any line would turn the `must-not-merge` pole into a tautology any engine passes — a
    /// pairwise pin cannot reach that, a per-line one does.
    ///
    /// **And the second inversion is reached from the TOML, where nothing reached it before.**
    /// Exchanging the two poles' `observations` vectors makes the corpus DEMAND the false merge:
    /// the echo/foxtrot pair — two real hosts, one wearing a clone of the other's MAC — becomes
    /// `must-merge`/`l1-exact-mac`, and the two genuine `doc-host-echo` presences become
    /// `must-not-merge`. Measured green across the whole workspace before [`assert_trap_binds`]
    /// existed. See that helper.
    #[test]
    fn the_cloned_mac_stream_wears_one_mac_on_every_line() {
        let observations = read_jsonl(&fixture_path("scenario/replay/cloned-mac.jsonl").unwrap())
            .expect("the cloned-mac stream must read");
        assert_eq!(observations.len(), 3, "three authored presences, exactly");
        // Exactly 3 facts per line (the `find()` guard) — see the randomized-mac pin.
        for (n, observation) in observations.iter().enumerate() {
            assert_eq!(
                observation.facts.len(),
                3,
                "observation {n} carries exactly its three facts"
            );
        }
        assert_obs_ids(&observations, "acacacac", 3);

        let fact = |n: usize, pick: fn(&Fact) -> bool| {
            observations[n]
                .facts
                .iter()
                .find(|f| pick(f))
                .unwrap_or_else(|| panic!("observation {n} must carry the fact"))
                .clone()
        };
        let mac = |n| fact(n, |f| matches!(f, Fact::Mac { .. }));
        let ip = |n| fact(n, |f| matches!(f, Fact::IpV4 { .. }));
        let hostname = |n| fact(n, |f| matches!(f, Fact::Hostname { .. }));

        // The clone, on EVERY line — pinned by value three times, never `mac(1) == mac(0)`.
        let cloned = Fact::Mac {
            addr: MacAddr([2, 0, 94, 0, 83, 112]),
            locally_administered: true,
        };
        for n in 0..3 {
            assert_eq!(
                mac(n),
                cloned,
                "observation {n} wears 02:00:5e:00:53:70 — the one cloned MAC, byte-identical"
            );
        }

        // The opposing signal: K1 and K3 are one host re-seen, K2 is the impostor.
        let echo = Fact::Hostname {
            name: "doc-host-echo".into(),
            source: HostnameSource::Dhcp,
        };
        assert_eq!(hostname(0), echo, "K1 answers to doc-host-echo");
        assert_eq!(
            hostname(1),
            Fact::Hostname {
                name: "doc-host-foxtrot".into(),
                source: HostnameSource::Dhcp,
            },
            "K2 answers to doc-host-foxtrot — the hostname that OPPOSES the tempting merge"
        );
        assert_eq!(
            hostname(2),
            echo,
            "K3 answers to the SAME doc-host-echo an hour later — the must-merge pole"
        );

        // Three distinct addresses, value-pinned.
        for (n, last) in [(0, 112u8), (1, 113), (2, 114)] {
            assert_eq!(
                ip(n),
                Fact::IpV4 {
                    addr: Ipv4Addr::new(192, 0, 2, last)
                },
                "observation {n} holds its authored address 192.0.2.{last}"
            );
        }

        // The authored instants — K1 and K2 share one: the clone answers in the same sweep.
        let instants: Vec<Timestamp> = observations.iter().map(|o| o.observed_at).collect();
        assert_eq!(
            instants,
            vec![
                ts("2026-01-05T00:00:00Z"),
                ts("2026-01-05T00:00:00Z"),
                ts("2026-01-05T01:00:00Z"),
            ],
            "the impostor answers in K1's own sweep, as authored"
        );

        // AC4b — the TOML side, and the measured one: this is the exchange that makes the corpus
        // demand the false merge without moving a stream byte.
        let traps = read_traps(&fixture_path("scenario/traps/cloned-mac.toml").unwrap())
            .expect("the cloned-mac trap file must read");
        assert_eq!(
            traps.trap.len(),
            2,
            "the family declares exactly its two poles"
        );
        assert_trap_binds(
            &traps,
            "cloned-mac-must-not-merge",
            &[
                "acacacac-0000-4000-8000-000000000001",
                "acacacac-0000-4000-8000-000000000002",
            ],
            not_merge("l2-different-hostname"),
            Some("cloned-mac"),
        );
        assert_trap_binds(
            &traps,
            "cloned-mac-must-merge",
            &[
                "acacacac-0000-4000-8000-000000000001",
                "acacacac-0000-4000-8000-000000000003",
            ],
            merge("l1-exact-mac"),
            Some("cloned-mac"),
        );
    }

    /// Story 5.2b's byte-pin over `example-traps.jsonl` — the SIXTH committed stream named by no
    /// value test, surfaced by this story's validation pass and closed here rather than
    /// registered.
    ///
    /// It is not a family (its traps declare no `family` and are exempt from the completeness
    /// check), but it carries exactly the shape story 4.13's register entry is about:
    /// `example.toml`'s first reason cites `02:00:5e:00:53:10` and its second cites *"their MACs
    /// differ in the final octet"* — claims no test asserted. That file's own header records why
    /// this matters in its own words: *"the first version of this file claimed two observations
    /// shared a MAC when the committed bytes said otherwise, and a reader caught it precisely
    /// because the claim was written down."* A reader caught it once; nothing would catch it
    /// twice.
    ///
    /// **Not a duplicate of `the_committed_trap_file_reads_and_cross_checks`**, which asserts that
    /// the example exercises all three of D18's columns and more than one stream — that the FORMAT
    /// is exercised. This one asserts the VALUES and the bindings. The two answer different
    /// questions over one file, which is the deliberate redundancy the house rule protects, not
    /// the accidental duplication it forbids.
    #[test]
    fn the_example_trap_stream_carries_the_values_its_reasons_cite() {
        let observations =
            read_jsonl(&fixture_path("scenario/replay/example-traps.jsonl").unwrap())
                .expect("the example-traps stream must read");
        assert_eq!(observations.len(), 3, "three authored presences, exactly");
        // Exactly 2 facts per line (the `find()` guard) — see the randomized-mac pin.
        for (n, observation) in observations.iter().enumerate() {
            assert_eq!(
                observation.facts.len(),
                2,
                "observation {n} carries exactly its two facts"
            );
        }
        assert_obs_ids(&observations, "bbbbbbbb", 3);

        let fact = |n: usize, pick: fn(&Fact) -> bool| {
            observations[n]
                .facts
                .iter()
                .find(|f| pick(f))
                .unwrap_or_else(|| panic!("observation {n} must carry the fact"))
                .clone()
        };
        let mac = |n| fact(n, |f| matches!(f, Fact::Mac { .. }));
        let ip = |n| fact(n, |f| matches!(f, Fact::IpV4 { .. }));

        // The MAC the first reason CITES, on both lines it judges — and the final octet the
        // second reason claims differs.
        let cited = Fact::Mac {
            addr: MacAddr([2, 0, 94, 0, 83, 16]),
            locally_administered: true,
        };
        assert_eq!(
            mac(0),
            cited,
            "E1 wears 02:00:5e:00:53:10, the MAC the must-merge reason cites"
        );
        assert_eq!(
            mac(1),
            cited,
            "E2 wears the byte-identical 02:00:5e:00:53:10 an hour later"
        );
        assert_eq!(
            mac(2),
            Fact::Mac {
                addr: MacAddr([2, 0, 94, 0, 83, 17]),
                locally_administered: true,
            },
            "E3 differs in the FINAL octet, exactly as the must-not-merge reason claims"
        );

        for (n, last) in [(0, 20u8), (1, 21), (2, 22)] {
            assert_eq!(
                ip(n),
                Fact::IpV4 {
                    addr: Ipv4Addr::new(192, 0, 2, last)
                },
                "observation {n} holds its authored address 192.0.2.{last}"
            );
        }

        // "an hour apart" is the merge reason's own words.
        let instants: Vec<Timestamp> = observations.iter().map(|o| o.observed_at).collect();
        assert_eq!(
            instants,
            vec![
                ts("2026-01-02T00:00:00Z"),
                ts("2026-01-02T01:00:00Z"),
                ts("2026-01-02T02:00:00Z"),
            ],
            "the sightings are an hour apart, as the merge reason states"
        );

        // AC4b — the TOML side. The third trap deliberately judges ANOTHER stream (`minimal`,
        // whose bytes `expected()` already pins), because a trap names the stream it judges and
        // nothing assumes there is one.
        let traps = read_traps(&fixture_path(EXAMPLE_TRAPS).unwrap())
            .expect("the example trap file must read");
        assert_eq!(
            traps.trap.len(),
            3,
            "the example declares exactly its three"
        );
        assert_trap_binds(
            &traps,
            "example-must-merge",
            &[
                "bbbbbbbb-0000-4000-8000-000000000001",
                "bbbbbbbb-0000-4000-8000-000000000002",
            ],
            merge("l1-exact-mac"),
            None,
        );
        assert_trap_binds(
            &traps,
            "example-must-not-merge",
            &[
                "bbbbbbbb-0000-4000-8000-000000000001",
                "bbbbbbbb-0000-4000-8000-000000000003",
            ],
            not_merge("l1-distinct-mac"),
            None,
        );
        assert_trap_binds(
            &traps,
            "example-must-abstain",
            &["aaaaaaaa-0000-4000-8000-000000000002"],
            Expectation::MustAbstain {
                cause: AbstentionCause::NoObservedValue,
            },
            None,
        );
    }

    /// Story 4.18's shape guard — the wire spec's only test until Epic 11's harness runs it
    /// under the real parser. It holds three things: the synthetic BODY to the measured field
    /// behaviours (lowercase-colon mac, 10-digit seconds epoch, no vlan key, len-24 single
    /// network_id, hostname present/empty/missing and null never, oui mostly empty, is_wired
    /// both ways, sw_port on wired only — the unmeasured wireless case deliberately absent);
    /// the EXPECTED Observations to their authored values (ids, context UUIDs, facts,
    /// instants); and the DERIVATION between them (mac string → Mac bytes, ip → IpV4, oui →
    /// OuiVendor, hostname key state mirrored, last_seen → observed_at) — the spec as
    /// executable connection, so the pair cannot drift apart while both halves stay
    /// internally consistent. This directory sits outside every corpus walk, so this test is
    /// also its PRIVACY coverage: the raw body text and the expected facts are routed through
    /// the corpus privacy rules here.
    #[test]
    fn the_wire_spec_encodes_the_measured_field_behaviours() {
        let body_path = fixture_path("scenario/wire/unifi-clients.json").unwrap();
        let raw = std::fs::read_to_string(&body_path).expect("the wire body must read");
        assert_text_is_synthetic(&raw, &body_path);
        let body: serde_json::Value = serde_json::from_str(&raw).expect("the body must parse");
        let data = body["data"].as_array().expect("the data array");
        assert_eq!(data.len(), 4, "four authored clients, exactly");

        let mut network_ids = std::collections::BTreeSet::new();
        let mut empty_ouis = 0;
        let mut wired = [false; 4];
        for (n, client) in data.iter().enumerate() {
            let obj = client.as_object().expect("a client is an object");

            // mac: lowercase colon-separated, 100% (the measurement's closed trap).
            let mac = obj["mac"].as_str().expect("mac is a string");
            assert!(
                mac.len() == 17
                    && mac.split(':').count() == 6
                    && mac
                        .chars()
                        .all(|c| c == ':' || c.is_ascii_digit() || c.is_ascii_lowercase()),
                "client {n}: mac must be lowercase colon-separated, got {mac}"
            );

            // last_seen: a 10-digit SECONDS epoch, not milliseconds.
            let last_seen = obj["last_seen"].as_i64().expect("last_seen is an integer");
            assert!(
                (1_000_000_000..10_000_000_000).contains(&last_seen),
                "client {n}: last_seen must be a 10-digit seconds epoch, got {last_seen}"
            );

            // vlan: MISSING, 100%.
            assert!(
                !obj.contains_key("vlan"),
                "client {n}: the measured payload never carries a vlan key"
            );

            // network_id: fixed-length 24, one distinct value across entries.
            let network_id = obj["network_id"].as_str().expect("network_id is a string");
            assert_eq!(
                network_id.chars().count(),
                24,
                "client {n}: network_id is fixed-length 24"
            );
            network_ids.insert(network_id.to_owned());

            // hostname: present, empty and missing all occur — null NEVER (walked per entry:
            // if the key exists its value is a string, never Value::Null).
            if let Some(hostname) = obj.get("hostname") {
                assert!(
                    hostname.is_string(),
                    "client {n}: a present hostname is a string, never null"
                );
            }

            // oui: present on all.
            let oui = obj["oui"].as_str().expect("oui is a string on every entry");
            if oui.is_empty() {
                empty_ouis += 1;
            }

            // is_wired: bool, 100%; sw_port present iff wired (the wireless presence rate was
            // never measured — the certain case only).
            wired[n] = obj["is_wired"].as_bool().expect("is_wired is a bool");
            match obj.get("sw_port") {
                Some(port) if wired[n] => {
                    let port = port.as_i64().expect("sw_port is an integer");
                    assert!(
                        (1..100).contains(&port),
                        "client {n}: sw_port is 1-2 digits, got {port}"
                    );
                }
                None if !wired[n] => {}
                other => panic!(
                    "client {n}: sw_port must be present iff wired (wired={}, sw_port={other:?})",
                    wired[n]
                ),
            }
        }
        assert_eq!(
            network_ids.len(),
            1,
            "one distinct network_id across entries"
        );
        assert_eq!(
            empty_ouis, 3,
            "oui is empty on the measured large share (3 of 4)"
        );
        assert!(
            wired.contains(&true) && wired.contains(&false),
            "both is_wired values occur"
        );
        let hostname_states: Vec<Option<&str>> = data
            .iter()
            .map(|c| c.get("hostname").map(|h| h.as_str().unwrap()))
            .collect();
        assert!(
            hostname_states
                .iter()
                .any(|s| matches!(s, Some(h) if !h.is_empty()))
                && hostname_states.iter().any(|s| matches!(s, Some("")))
                && hostname_states.iter().any(|s| s.is_none()),
            "hostname present, empty and missing all occur"
        );

        // The expected Observations — the parser's output contract, authored before the
        // parser (D19: the fixture schema IS the Observation schema).
        let expected_path = fixture_path("scenario/wire/unifi-clients.expected.jsonl").unwrap();
        let observations = read_jsonl(&expected_path).expect("the expected stream must read");
        assert_eq!(observations.len(), 4, "one expected observation per client");
        let scope = Scope {
            l2_domain: L2DomainId::from_uuid(u("11111111-1111-4111-8111-111111111111")),
            vantage: VantageId::from_uuid(u("22222222-2222-4222-8222-222222222222")),
        };
        let connector_id = ConnectorId::from_uuid(u("33333333-3333-4333-8333-333333333333"));
        // The `obs_id` ↔ line binding — hoisted out of the loop below into the shared helper
        // (story 5.1); the placeholder-context pins that were fused with it stay where they are.
        assert_obs_ids(&observations, "bdbdbdbd", 4);
        for (n, expected_len) in [(0usize, 4usize), (1, 4), (2, 3), (3, 4)] {
            let obs = &observations[n];
            assert_eq!(
                obs.facts.len(),
                expected_len,
                "observation {n}'s fact count"
            );
            // The context UUIDs are PLACEHOLDERS (harness context, not expectations) — pinned
            // so they cannot drift silently either.
            assert_eq!(
                obs.connector_id, connector_id,
                "line {n}'s placeholder connector"
            );
            assert_eq!(obs.scope, scope, "line {n}'s placeholder scope");
            assert_facts_are_synthetic(&obs.facts, &expected_path);
        }

        // The DERIVATION, per index: the expected facts re-derived from the body's strings.
        for (n, client) in data.iter().enumerate() {
            let obs = &observations[n];
            let find = |pick: fn(&Fact) -> bool| obs.facts.iter().find(|f| pick(f)).cloned();

            let wire_mac = MacAddr::from_str(client["mac"].as_str().unwrap()).unwrap();
            assert_eq!(
                find(|f| matches!(f, Fact::Mac { .. })),
                Some(Fact::Mac {
                    addr: wire_mac,
                    locally_administered: wire_mac.is_locally_administered(),
                }),
                "line {n}: the expected Mac is the wire mac, flag from its own U/L bit"
            );

            let wire_ip: Ipv4Addr = client["ip"].as_str().unwrap().parse().unwrap();
            assert_eq!(
                find(|f| matches!(f, Fact::IpV4 { .. })),
                Some(Fact::IpV4 { addr: wire_ip }),
                "line {n}: the expected IpV4 is the wire ip"
            );

            // hostname: wire "" -> fact with "", wire key MISSING -> no fact (4.17's
            // committed doctrine; the source attribution Dhcp is a charter-named hole).
            let expected_hostname = client.get("hostname").map(|h| Fact::Hostname {
                name: h.as_str().unwrap().into(),
                source: HostnameSource::Dhcp,
            });
            assert_eq!(
                find(|f| matches!(f, Fact::Hostname { .. })),
                expected_hostname,
                "line {n}: the expected hostname mirrors the wire key's state and value"
            );

            // oui: mapped by the same doctrine ("" stays an empty fact) — a charter-named
            // recorded-bump candidate.
            assert_eq!(
                find(|f| matches!(f, Fact::OuiVendor { .. })),
                Some(Fact::OuiVendor {
                    vendor: client["oui"].as_str().unwrap().into(),
                }),
                "line {n}: the expected OuiVendor is the wire oui"
            );

            // observed_at = last_seen read as epoch SECONDS, exactly.
            let expected_at =
                chrono::DateTime::from_timestamp(client["last_seen"].as_i64().unwrap(), 0).unwrap();
            assert_eq!(
                obs.observed_at, expected_at,
                "line {n}: observed_at is the wire last_seen in seconds"
            );

            // No Uplink anywhere: sw_mac was never measured — the hole is an omission, not an
            // accident.
            assert!(
                obs.facts.iter().all(|f| !matches!(f, Fact::Uplink { .. })),
                "line {n}: no Uplink may be expected — sw_mac was never measured"
            );
        }

        // The instants restated as authored values (the second oracle over the derivation).
        let instants: Vec<Timestamp> = observations.iter().map(|o| o.observed_at).collect();
        assert_eq!(
            instants,
            vec![
                ts("2026-01-12T00:00:00Z"),
                ts("2026-01-12T00:05:00Z"),
                ts("2026-01-12T00:10:00Z"),
                ts("2026-01-12T00:15:00Z"),
            ],
            "the four last_seen epochs are the four authored instants"
        );
    }

    // ---- The blocker, measured against the committed corpus (story 5.6) ----
    //
    // The blocker itself lives in `opencmdb_core::identity::blocking`. These assertions live HERE
    // and not beside it because D47 forbids the domain crate to touch the filesystem, and the truth
    // set is the corpus: Epic 4 froze it BEFORE the engine on purpose — *"a metric written after
    // the engine is bent to fit the engine"* — so a recall floor measured against a truth set the
    // engine's own author writes today would be the mirror D13 refuses for weights, applied to
    // blocking.
    //
    // Nothing above `#[cfg(test)]` changes, and no new `pub` item appears in this crate.

    /// What the committed traps say about pairs, read once and reused by the assertions below.
    struct CorpusPairs {
        /// How many traps the corpus holds, across every trap file.
        traps: usize,
        /// The traps that name exactly two observations, as `(trap id, replay, the pair)`.
        pairs: Vec<(String, String, CandidatePair)>,
        /// The traps that name fewer than two observations, by id.
        without_a_pair: Vec<String>,
        /// The traps that name more than two observations, by id.
        beyond_a_pair: Vec<String>,
        /// The `must-merge` pairs — the recall truth set.
        required: std::collections::BTreeSet<CandidatePair>,
        /// The candidate universe of each replay stream a trap names, keyed by the stream.
        universes: std::collections::BTreeMap<String, std::collections::BTreeSet<CandidatePair>>,
    }

    impl CorpusPairs {
        /// The union of the per-stream universes.
        ///
        /// A cross-stream pair is meaningless — `candidates` never sees two streams at once — but
        /// `required` draws its pairs from ten different streams, so the only recall call that
        /// typechecks compares against this union. What makes the union HONEST is
        /// `the_blocker_proposes_every_required_pair_within_its_own_stream`: it proves each required
        /// pair sits in its OWN stream's universe, so the union can only add pairs and can never
        /// explain a miss away.
        fn union(&self) -> std::collections::BTreeSet<CandidatePair> {
            self.universes.values().flatten().copied().collect()
        }
    }

    /// Walk every committed trap file, and build the pairs and universes it implies.
    fn corpus_pairs() -> CorpusPairs {
        let mut found = CorpusPairs {
            traps: 0,
            pairs: Vec::new(),
            without_a_pair: Vec::new(),
            beyond_a_pair: Vec::new(),
            required: std::collections::BTreeSet::new(),
            universes: std::collections::BTreeMap::new(),
        };
        walk_trap_files(&mut |path| {
            let file = read_traps(path)
                .unwrap_or_else(|e| panic!("corpus trap file {} is invalid: {e}", path.display()));
            for trap in &file.trap {
                found.traps += 1;
                if !found.universes.contains_key(&trap.replay) {
                    let stream = read_jsonl(&fixture_path(&trap.replay).unwrap())
                        .unwrap_or_else(|e| panic!("reading {}: {e}", trap.replay));
                    found
                        .universes
                        .insert(trap.replay.clone(), candidates(&stream));
                }
                match trap.observations.as_slice() {
                    [a, b] => {
                        // `CandidatePair::new` returns `Option`, and a trap naming one id twice
                        // must fail LOUDLY rather than vanish out of the truth set.
                        let pair = CandidatePair::new(*a, *b)
                            .expect("a trap names two distinct observations");
                        found
                            .pairs
                            .push((trap.id.0.clone(), trap.replay.clone(), pair));
                        if matches!(trap.expect, Expectation::MustMerge { .. }) {
                            found.required.insert(pair);
                        }
                    }
                    fewer if fewer.len() < 2 => found.without_a_pair.push(trap.id.0.clone()),
                    _ => found.beyond_a_pair.push(trap.id.0.clone()),
                }
            }
        });
        found
    }

    /// The architecture's own ratified test name [architecture.md:2954] — D13's
    /// `blocking_recall >= 0.999`, expressed in the milli-units D13's corollary demands.
    ///
    /// It computes ONLY the recall. The per-trap containment assertion is a separate test on
    /// purpose: with both in one function, a missing pair panics inside the loop and the recall is
    /// never computed at all, so the value a narrowed blocker would score could not be observed.
    #[test]
    fn blocking_recall_above_999() {
        let corpus = corpus_pairs();

        assert_eq!(
            corpus.required.len(),
            11,
            "the truth set is the committed `must-merge` traps; a denominator that shrinks in \
             silence is a gate that quietly stops testing"
        );

        let recall = blocking_recall_per_mille(&corpus.union(), &corpus.required)
            .expect("the truth set is not empty, so the recall is defined");

        // ⚠️ The FLOOR first, then the exact value — the order is load-bearing and it was the wrong
        // way round until this story's code review. `assert_eq!(recall, 1000)` is strictly stronger,
        // so with it first the floor comparison could never be the assertion that fails: a narrowed
        // blocker redded on a bare equality and D13's threshold said nothing. Measured then:
        // changing `>=` to `>` left the whole workspace green. This way the realistic failure —
        // a blocker that hides a required pair — reds with the floor's own message, and the equality
        // below still pins that the committed corpus is at full recall rather than merely above it.
        assert!(
            recall >= BLOCKING_RECALL_FLOOR_PER_MILLE,
            "recall is {recall} per-mille, below the floor of {BLOCKING_RECALL_FLOOR_PER_MILLE}"
        );
        assert_eq!(
            recall, 1000,
            "the blocker proposes every pair the corpus requires"
        );
    }

    /// Each required pair is in the universe of ITS OWN stream — what makes the union above honest.
    ///
    /// Backstop, measured on the committed bytes rather than guaranteed by code: the `obs_id`s
    /// across the streams the traps name are all distinct, so no coincidental cross-stream hit
    /// exists today. That is a property of the corpus and could change; this assertion is the one
    /// that would still hold if it did.
    #[test]
    fn the_blocker_proposes_every_required_pair_within_its_own_stream() {
        let corpus = corpus_pairs();
        let mut checked = 0usize;
        for (id, replay, pair) in &corpus.pairs {
            if !corpus.required.contains(pair) {
                continue;
            }
            assert!(
                corpus.universes[replay].contains(pair),
                "{id}: the blocker never proposes the pair this trap requires, in {replay}"
            );
            checked += 1;
        }
        assert_eq!(
            checked, 11,
            "every `must-merge` trap must have been checked, or this test passes by walking past \
             the traps it exists to cover"
        );
    }

    /// Coverage, NOT recall — and the two are deliberately not given one name.
    ///
    /// D13's recall metric is about the merge pairs. This asserts something wider and weaker: every
    /// trap pair, `must-not-merge` and `must-abstain` included, is in the universe. A pair outside
    /// the universe can never be answered by anything, so the trap runner could never score it.
    #[test]
    fn every_trap_pair_is_in_the_universe() {
        let corpus = corpus_pairs();

        assert_eq!(
            corpus.pairs.len(),
            25,
            "25 of the committed traps name a pair; a count that drifts in silence is a scan that \
             has stopped covering the corpus"
        );
        for (id, replay, pair) in &corpus.pairs {
            assert!(
                corpus.universes[replay].contains(pair),
                "{id}: the pair it judges is outside the candidate universe of {replay}, so no \
                 rule could ever be asked about it"
            );
        }
    }

    // ---- The L2 blocker, measured against the committed corpus (story 6.6) ----
    //
    // The L1 walk above keys on `obs_id`. This one keys on the INTERFACE — `join`'s
    // `(l2_domain, mac)` — because that is what an L2 rule judges. The two truth sets are
    // DIFFERENT SETS and the difference is the point: eight of the eleven L1 `must-merge` pairs
    // expect `l1-exact-mac` and are two sightings of ONE interface, which is no L2 pair at all.

    /// What the committed traps say about INTERFACE pairs, for the traps that name an `l2-*` rule.
    struct L2Corpus {
        /// The L2 recall truth set — `must-merge` traps whose expected rule is `l2-*`.
        required: std::collections::BTreeSet<L2CandidatePair>,
        /// The L2 candidate universe of each replay stream, keyed by the stream.
        universes: std::collections::BTreeMap<String, std::collections::BTreeSet<L2CandidatePair>>,
        /// Traps naming an `l2-*` rule that resolve to a real pair, as `(id, replay, pair)`.
        pairs: Vec<(String, String, L2CandidatePair)>,
        /// Traps naming an `l2-*` rule whose two observations COLLAPSE onto ONE interface.
        collapsed: Vec<String>,
        /// Traps naming an `l2-*` rule where a named observation carries NO L1 key.
        interfaceless: Vec<String>,
        /// Traps naming an `l2-*` rule that do not name exactly TWO observations.
        ///
        /// 🔴 Separate from `interfaceless` since story 6.6's code review, which found the two
        /// folded together: a future `l2-*` trap naming three observations would have been reported
        /// as *"names an observation without an L1 key"*, **a red accusing the wrong cause**. Empty
        /// today — every one of the eight `l2-*` traps names a pair — so this is a naming hole and
        /// not a coverage hole, and it is closed before the corpus grows into it.
        wrong_arity: Vec<String>,
        /// Traps naming an `l2-*` rule where a named observation carries TWO OR MORE L1 keys.
        multi_homed: Vec<String>,
    }

    /// Walk every committed trap file and resolve the `l2-*` traps to INTERFACE pairs.
    ///
    /// The three residue buckets are not defensive padding: an observation may carry zero MACs or
    /// several, so `join` may give a named observation no key or many. Dropping either case silently
    /// is how a trap leaves the corpus without anyone noticing — and `collapsed` is what found the
    /// story's §0j.
    fn l2_corpus() -> L2Corpus {
        let mut found = L2Corpus {
            required: std::collections::BTreeSet::new(),
            universes: std::collections::BTreeMap::new(),
            pairs: Vec::new(),
            collapsed: Vec::new(),
            interfaceless: Vec::new(),
            wrong_arity: Vec::new(),
            multi_homed: Vec::new(),
        };
        walk_trap_files(&mut |path| {
            let file = read_traps(path)
                .unwrap_or_else(|e| panic!("corpus trap file {} is invalid: {e}", path.display()));
            for trap in &file.trap {
                let Some(rule) = trap.expect.rule() else {
                    continue;
                };
                if !rule.0.starts_with("l2-") {
                    continue;
                }
                let stream = read_jsonl(&fixture_path(&trap.replay).unwrap())
                    .unwrap_or_else(|e| panic!("reading {}: {e}", trap.replay));
                let groups = join(&stream);
                if !found.universes.contains_key(&trap.replay) {
                    let interfaces: Vec<L1Key> = groups.keys().copied().collect();
                    found
                        .universes
                        .insert(trap.replay.clone(), l2_candidates(&interfaces));
                }
                // Invert the join: which interfaces does each named observation land on?
                let keys_of = |wanted: &ObsId| -> Vec<L1Key> {
                    groups
                        .iter()
                        .filter(|(_, members)| members.contains(wanted))
                        .map(|(key, _)| *key)
                        .collect()
                };
                let [a, b] = trap.observations.as_slice() else {
                    found.wrong_arity.push(trap.id.0.clone());
                    continue;
                };
                let (ka, kb) = (keys_of(a), keys_of(b));
                if ka.is_empty() || kb.is_empty() {
                    found.interfaceless.push(trap.id.0.clone());
                    continue;
                }
                if ka.len() > 1 || kb.len() > 1 {
                    found.multi_homed.push(trap.id.0.clone());
                    continue;
                }
                match L2CandidatePair::new(ka[0], kb[0]) {
                    None => found.collapsed.push(trap.id.0.clone()),
                    Some(pair) => {
                        found
                            .pairs
                            .push((trap.id.0.clone(), trap.replay.clone(), pair));
                        if matches!(trap.expect, Expectation::MustMerge { .. }) {
                            found.required.insert(pair);
                        }
                    }
                }
            }
        });
        found
    }

    /// D13's `blocking_recall >= 0.999`, at L2 — in the milli-units its corollary demands.
    ///
    /// It computes ONLY the recall; the containment assertion is a separate test on the same
    /// reasoning `blocking_recall_above_999` states for L1: with both in one function a missing pair
    /// panics inside the loop and the value a narrowed blocker would score is never observed.
    ///
    /// ⚠️ **At a denominator of three, `>= 999` is ZERO-TOLERANCE**: one miss scores 666 and the
    /// floor reds. It is the binary form NFR4 demands, not a statistical tolerance.
    #[test]
    fn l2_blocking_recall_above_999() {
        let corpus = l2_corpus();

        assert_eq!(
            corpus.required.len(),
            3,
            "the L2 truth set is the `must-merge` traps whose expected rule is `l2-*`; a \
             denominator that shrinks in silence is a gate that quietly stops testing"
        );

        let union: std::collections::BTreeSet<L2CandidatePair> =
            corpus.universes.values().flatten().copied().collect();
        let recall = blocking_recall_per_mille(&union, &corpus.required)
            .expect("the truth set is not empty, so the recall is defined");

        assert!(
            recall >= BLOCKING_RECALL_FLOOR_PER_MILLE,
            "L2 recall is {recall} per-mille, below the floor of {BLOCKING_RECALL_FLOOR_PER_MILLE}"
        );
        assert_eq!(
            recall, 1000,
            "the committed corpus is at FULL L2 recall, which is stronger than the floor and is \
             what the total universe gives by construction"
        );
    }

    /// Every required L2 pair is proposed WITHIN ITS OWN STREAM, so the union above can only add
    /// pairs and can never explain a miss away.
    #[test]
    fn the_l2_blocker_proposes_every_required_pair_within_its_own_stream() {
        let corpus = l2_corpus();
        let mut checked = 0usize;
        for (id, replay, pair) in &corpus.pairs {
            if !corpus.required.contains(pair) {
                continue;
            }
            assert!(
                corpus.universes[replay].contains(pair),
                "{id}: the L2 blocker never proposes the interface pair this trap requires, in \
                 {replay}"
            );
            checked += 1;
        }
        assert_eq!(
            checked, 3,
            "every L2 `must-merge` trap must have been checked, or this test passes by walking \
             past the traps it exists to cover"
        );
    }

    /// COVERAGE, not recall — and this is the assertion that found story 6.6's §0j.
    ///
    /// Wider and weaker than the recall metric: every trap naming an `l2-*` rule, `must-not-merge`
    /// included, is accounted for — either as an interface pair in the universe, or by NAME in one
    /// of the three residue buckets. A pair outside the universe can never be answered by anything.
    ///
    /// ⚠️ **What carries this test is the BUCKETS, not the containment loop at the end.** The loop
    /// is a COROLLARY of totality and cannot red on its own: `pair` is built from `groups`, and the
    /// universe is built from the keys of that same `groups`, so a total `l2_candidates` contains it
    /// by construction. 🔑 **Its L1 twin `every_trap_pair_is_in_the_universe` is NOT a corollary** —
    /// there the pair comes from the `obs_id`s a trap NAMES, which need not appear in the stream at
    /// all. This story inherited the shape without inheriting the property, and the loop is kept as
    /// a deliberate second oracle rather than deleted: it would red the day `l2_candidates` stopped
    /// being total, which `l2_the_universe_is_total_over_distinct_interfaces` also covers.
    /// _(Found by story 6.6's blind review layer, from the diff alone.)_
    ///
    /// 🔴 **One committed trap is EXCLUDED BY NAME, and the exclusion is the finding.**
    /// `cloned-mac-must-not-merge` names `l2-different-hostname` and has **no L2 pair**: its two
    /// observations carry the SAME `MacAddr` in the same `l2_domain`, so `join` collapses them onto
    /// ONE interface. The trap file says so itself, citing D21 — *"a cloned MAC = two real
    /// interfaces, same MAC"* — which is exactly what `join`'s key makes unrepresentable. It is
    /// therefore **unanswerable at L2**, and the unanswerable bucket that stories 6.7–6.11 leave
    /// behind is **4, not the 3 `epics.md`'s Epic 6 constraint (2) states**. Registered; owner
    /// story 6.15.
    #[test]
    fn every_l2_trap_pair_is_in_the_universe() {
        let corpus = l2_corpus();

        assert_eq!(
            corpus.collapsed,
            vec!["cloned-mac-must-not-merge".to_string()],
            "exactly one `l2-*` trap collapses onto a single interface, and it is NAMED here so \
             the residue cannot grow unnoticed — a second one would be a second trap silently \
             leaving the corpus"
        );
        assert!(
            corpus.interfaceless.is_empty(),
            "no `l2-*` trap names an observation without an L1 key today: {:?}",
            corpus.interfaceless
        );
        assert!(
            corpus.wrong_arity.is_empty(),
            "every `l2-*` trap names exactly two observations today; one that did not would need a \
             decision about WHICH interface pair it judges, not a silent skip — and it must not be \
             reported as an observation without an L1 key, which is a different cause: {:?}",
            corpus.wrong_arity
        );
        assert!(
            corpus.multi_homed.is_empty(),
            "no `l2-*` trap names an observation carrying two or more MACs today; one that did \
             would need a decision about WHICH interface pair it judges, not a silent skip: {:?}",
            corpus.multi_homed
        );
        assert_eq!(
            corpus.pairs.len(),
            7,
            "seven of the eight `l2-*` traps resolve to an interface pair; a count that drifts in \
             silence is a scan that has stopped covering the corpus"
        );
        for (id, replay, pair) in &corpus.pairs {
            assert!(
                corpus.universes[replay].contains(pair),
                "{id}: the interface pair it judges is outside the L2 candidate universe of \
                 {replay}, so no rule could ever be asked about it"
            );
        }
    }

    /// Exactly one committed trap names fewer than two observations, and none names more.
    ///
    /// The residue is asserted rather than quoted: the two pair-based tests above skip these traps,
    /// and a skip that can grow silently is how a gate quietly stops testing.
    #[test]
    fn exactly_one_trap_names_fewer_than_two_observations() {
        let corpus = corpus_pairs();

        assert_eq!(corpus.traps, 26, "the corpus holds 26 traps");
        assert_eq!(
            corpus.without_a_pair,
            vec!["example-must-abstain".to_string()],
            "one trap judges a single observation, and it is named here so the residue cannot grow \
             unnoticed"
        );
        assert!(
            corpus.beyond_a_pair.is_empty(),
            "no trap names more than two observations today; one that did would need a decision \
             about what pair it requires, not a silent skip: {:?}",
            corpus.beyond_a_pair
        );
        assert_eq!(
            corpus.traps,
            corpus.pairs.len() + corpus.without_a_pair.len() + corpus.beyond_a_pair.len(),
            "every trap lands in exactly one of the three buckets"
        );
    }
}
