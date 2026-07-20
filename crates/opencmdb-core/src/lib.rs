//! opencmdb-core — the domain layer.
//!
//! The engine that compares the *observed* state against the *declared* state; the
//! gap between them is the product. This crate holds the domain and NOTHING that
//! reaches the outside world — no SQL, no HTTP, no clock, no `anyhow` (D47).
//!
//! Subdomains land here by folder, but the folder is not the frontier: visibility is
//! (`pub(in ...)` → `E0603`, D54). The identity engine, the verdict algebra, the gap
//! predicate, and `http_status(&DomainError) -> u16` (D53) live under here as the work
//! of story 1 onward. This is the walking-skeleton placeholder: it compiles, and it
//! asserts nothing about identity yet.

#![forbid(unsafe_code)]

pub mod connector;
pub mod observation;

/// Scripted in-memory connector + the contract test harness. Compiled for this crate's own
/// tests and for consumers that enable the `test-support` feature; never in the shipped build.
#[cfg(any(test, feature = "test-support"))]
pub mod testing;

pub use connector::{Connector, ConnectorError, ObservationSink, PollSummary, VecSink};
pub use observation::{
    Capabilities, ConnectorId, Fact, FactKind, HostnameSource, L2DomainId, MacAddr, MacParseError,
    ObsId, Observation, Scope, Timestamp, VantageId,
};
