//! opencmdb-core — the domain layer.
//!
//! The engine that compares the *observed* state against the *declared* state; the
//! gap between them is the product. This crate holds the domain and NOTHING that
//! reaches the outside world — no SQL, no HTTP, no clock, no `anyhow` (D47).
//!
//! Subdomains land here by folder, but the folder is not the frontier: visibility is
//! (`pub(in ...)` → `E0603`, D54). The identity engine, the verdict algebra, the gap
//! predicate, and `http_status(&DomainError) -> u16` (D53) live under here as the work
//! of story 1 onward. The identity engine now decides at L1: `identity::l1` joins
//! observations on `(l2_domain, mac)` and answers a candidate pair through
//! `identity::cascade::decide`, and `identity::blocking` is what defines which pairs
//! could ever be asked about. Its L2 half, and the persistence of what it decides,
//! are still ahead.

#![forbid(unsafe_code)]
// Documentation is a project rule (CLAUDE.md): every public item — structs, enums, fields,
// variants and functions — carries a doc comment. This crate does NOT yet carry
// `#![deny(missing_docs)]`: ~70 field/variant docs are outstanding (mostly in `observation`), and
// the CI clippy gate runs `-D warnings`, which would promote a `#![warn]` straight to an error. The
// lint lands here once that sweep is done — bin and xtask, already clean, deny it today.

pub mod clock;
pub mod connector;
pub mod gap;
pub mod identity;
pub mod observation;
pub mod repo;
pub mod score;
pub mod trap;

/// Scripted in-memory connector + the contract test harness. Compiled for this crate's own
/// tests and for consumers that enable the `test-support` feature; never in the shipped build.
#[cfg(any(test, feature = "test-support"))]
pub mod testing;

#[cfg(any(test, feature = "test-support"))]
pub use testing::{FixedClock, ScriptedConnector, ScriptedOutcome, run_connector_contract};

pub use clock::Clock;
pub use connector::{Connector, ConnectorError, ObservationSink, PollSummary, VecSink};
pub use gap::{AbstentionCause, Gap, Reconciliation, reconcile};
pub use identity::cascade::{
    Conclusion, Decision, IdentityAbstentionCause, RuleVerdict, RulesetVersion, Verdict, decide,
};
// `identity::l1::join` is deliberately NOT flat-re-exported: `opencmdb_core::join` is a very
// generic root-level name for an L1-specific function. ⚠️ The reason recorded here used to be that
// its consumer, the candidate generator, did not exist yet — the generator now exists and does NOT
// consume `join`, so that reason is retired rather than left standing. `join`'s INTENDED consumer
// is still the trap runner (story 5.7), which has not crossed the crate frontier and does not call
// it today. The generic-name argument is the one
// that survives, and it is the whole reason. Reach it through `identity::l1::join`.
// `verdict_for_pair` is `pub(crate)` — see its doc for why exposing it beside `decide` would
// falsify a claim in `cascade`.
//
// `identity::blocking` follows the same precedent, for the same reason and more strongly:
// `opencmdb_core::candidates` names nothing on its own, and neither does a bare `recall`. Reach
// them through `identity::blocking::{candidates, blocking_recall_per_mille, CandidatePair,
// BLOCKING_RECALL_FLOOR_PER_MILLE}`.
pub use identity::l1::{
    CURRENT_RULESET_VERSION, L1_DISTINCT_MAC, L1_EXACT_MAC, L1Key, decide_pair,
};
pub use observation::{
    Capabilities, ConnectorId, Fact, FactKind, HostnameSource, L2DomainId, MacAddr, MacParseError,
    ObsId, Observation, Scope, Timestamp, VantageId,
};
pub use repo::{BoxFuture, ReadRepository, RepositoryError, WriteRepository, WriteUnit};
pub use score::{
    Column, Outcome, RecordComparison, RunComparison, Score, ScoredRecord, SourceState, Tally,
    TrapVerdict, VerdictVectorEntry, compare_records, compare_runs, run_trap, score,
};
