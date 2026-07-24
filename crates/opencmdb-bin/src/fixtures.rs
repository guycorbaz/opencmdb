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
#[derive(Debug, Clone, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use opencmdb_core::observation::{
        ConnectorId, Fact, HostnameSource, L2DomainId, MacAddr, ObsId, Scope, Timestamp, VantageId,
    };
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
        let checked = walk_replay_streams(&mut |path| {
            for record in read_records(path).expect("a corpus stream must read") {
                match record {
                    Record::Observation(observation) => {
                        assert_facts_are_synthetic(&observation.facts, path)
                    }
                    // Exhaustive on purpose — no `_` arm. A new record kind must break THIS match
                    // and force a privacy decision rather than slip past. Story 4.5b's capability
                    // record did exactly that, and the decision is the arm below.
                    Record::Failure(error) => assert_text_is_synthetic(&error.to_string(), path),
                    // A capability record carries a timestamp and a set of `FactKind` enum values —
                    // no free text, no address, nothing an author can type a real hostname into.
                    // Nothing to scan, stated rather than skipped.
                    Record::Capability(_) => {}
                }
            }
        });
        assert!(checked > 0, "no replay stream found under scenario/replay/");
    }

    /// Free text authored by a fixture author must carry no real address.
    ///
    /// It scans for anything that PARSES as an IPv4 address or a MAC and holds it to the same rule
    /// as a structured fact. That is deliberately narrower than "no private data" — a hostname in
    /// prose cannot be recognised mechanically — so it is a floor, not a proof. The register
    /// carries what it does not cover.
    fn assert_text_is_synthetic(text: &str, path: &Path) {
        let where_ = path.display();
        for token in text.split(|c: char| !(c.is_ascii_hexdigit() || c == '.' || c == ':')) {
            if let Ok(addr) = token.parse::<Ipv4Addr>() {
                assert_documentation_ip(addr, path);
            }
            if let Ok(mac) = MacAddr::from_str(token) {
                assert!(
                    mac.is_locally_administered(),
                    "{where_}: free text names {mac}, which is not locally administered — \
                     a real vendor address must never be committed"
                );
            }
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
                Fact::Mac { addr, .. } => assert_synthetic_mac(*addr, path),
                Fact::Uplink { peer_mac, .. } => assert_synthetic_mac(*peer_mac, path),
                Fact::Hostname { name, .. } => assert!(
                    name.starts_with("doc-"),
                    "{where_}: hostnames must be invented, not captured: {name}"
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

    fn assert_synthetic_mac(addr: MacAddr, path: &Path) {
        let where_ = path.display();
        assert!(
            addr.is_locally_administered(),
            "{where_}: {addr} is not locally administered — a real vendor address must never be \
             committed"
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
    fn scratch_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("opencmdb-fixtures-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    // ── Record dispatch and the failure record (story 4.5a) ──────────────────

    /// Write a scratch stream and read it back as records.
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
        let checked = walk_replay_streams(&mut |path| {
            let ids = read_records(path)
                .unwrap_or_else(|e| {
                    panic!("corpus replay stream {} is invalid: {e}", path.display())
                })
                .into_iter()
                .filter_map(|r| r.as_observation().map(|o| o.obs_id.as_uuid()))
                .collect();
            streams.push((path.display().to_string(), ids));
        });
        assert!(checked > 0, "no replay stream found under scenario/replay/");
        if let Some((id, first, second)) = first_cross_stream_obs_id(&streams) {
            panic!(
                "obs_id {id} appears in both {first} and {second} — a trap pointing at it could not say which"
            );
        }
    }

    #[test]
    fn every_replay_stream_in_the_corpus_is_valid() {
        let checked = walk_replay_streams(&mut |path| {
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
        assert!(checked > 0, "no replay stream found under scenario/replay/");
    }

    /// Walk every `.jsonl` under `scenario/replay/`, recursively, refusing symlinks and any other
    /// extension, and return how many were visited.
    ///
    /// Recursive on purpose: the trap FAMILIES (4.9+) are what will introduce a subdirectory, and
    /// a flat scan would hash them and never read them. Read errors are not swallowed — an
    /// unreadable subtree shrinking the search space into a false green was a real defect in 4.1.
    fn walk_replay_streams(visit: &mut dyn FnMut(&Path)) -> usize {
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
        checked
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
    #[test]
    fn every_trap_file_in_the_corpus_is_valid() {
        let traps_dir = fixture_path("scenario/traps").unwrap();
        let mut checked = 0usize;
        let mut stack = vec![traps_dir.clone()];
        while let Some(dir) = stack.pop() {
            for entry in
                std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()))
            {
                let entry = entry.expect("a directory entry must be readable");
                let path = entry.path();
                // `file_type()` does not follow symlinks, so a link can neither smuggle a file
                // in nor be walked out of the corpus — but it must not pass unnoticed either.
                let file_type = entry.file_type().expect("file type");
                if file_type.is_symlink() {
                    panic!(
                        "{}: the corpus must contain its own bytes, not a symlink",
                        path.display()
                    );
                }
                if file_type.is_dir() {
                    stack.push(path);
                    continue;
                }
                // `README.md` is exempt at any depth, exactly as the corpus lock's orphan rule
                // exempts it (`xtask/src/main.rs`) and the production walk `discover_trap_files`
                // does — documenting this directory (e.g. the reality-debt register) must not red
                // the test suite. Kept identical to the replay walk's exemption above.
                if path.file_name().and_then(|n| n.to_str()) == Some("README.md") {
                    continue;
                }
                // Case-insensitive: a `broken.TOML` skipped silently would be hashed by the
                // gate and read by nobody.
                let is_toml = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e.eq_ignore_ascii_case("toml"));
                assert!(
                    is_toml,
                    "{}: only .toml trap files belong under scenario/traps/",
                    path.display()
                );
                read_traps(&path).unwrap_or_else(|e| {
                    panic!("corpus trap file {} is invalid: {e}", path.display())
                });
                checked += 1;
            }
        }
        // A discovery test that finds nothing must not pass silently — the vacuity the fixtures
        // gate carried from Epic 1 until story 4.1 put a file on disk.
        assert!(checked > 0, "no trap file found in {}", traps_dir.display());
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
        let path = write_traps(
            "both",
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
}
