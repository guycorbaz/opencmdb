//! The connector contract's error taxonomy.
//!
//! Per D33, this is not a taxonomy of "things that can go wrong" — it is a taxonomy of the
//! QUESTIONS a source did not answer. A variant exists only when it produces a
//! `(source_state, operator action)` pair no other variant produces; DNS failure, connection
//! refused, and a network timeout are ONE variant (`Unreachable`) because they share one
//! operator action. It is a CLOSED taxonomy — deliberately NOT `#[non_exhaustive]` — because
//! exhaustiveness is the guardrail: adding a variant must break every downstream `match`, so
//! each is forced to decide blind-vs-gap. "`anyhow::Error` wakes nobody; a non-exhaustive
//! `match` does not compile."
//!
//! The `Connector` trait, `ObservationSink`, and `PollSummary` join this module in Story 2.3.

use thiserror::Error;

/// Why a poll produced no (or incomplete) observations. Closed by design (D33): every cause
/// blinds the source EXCEPT [`ConnectorError::Cancelled`], which writes nothing. There is no
/// `Other(String)` — that would be `anyhow` in disguise and would make FR5/FR8/FR19
/// inexpressible. The discriminant is machine-readable (the engine matches on it); the
/// payload is human-readable (never matched on).
///
/// NOTE: a `scope` field (D33's "every variant carries scope") is deferred to Epic 13, where
/// the liveness-blindness scope of D34 §3 is built and `source_state` exists; the scheduler
/// (which polls per scope, D34) knows the scope until then. Adding it later is additive.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConnectorError {
    /// Authentication was rejected — the source responds, but gives us nothing.
    /// Operator action: rotate the source's API key (Journey 4).
    #[error("authentication rejected: {detail}")]
    Unauthorized { detail: String },

    /// The source could not be reached at all: DNS failure, connection refused, or a network
    /// timeout — one variant, because they share one operator action: check connectivity.
    #[error("source unreachable: {detail}")]
    Unreachable { detail: String },

    /// The response could not be parsed against the expected schema. Aimed at the MAINTAINER
    /// (NFR8), not the operator; carries an OS-version hint when known.
    #[error("response did not match the expected schema: {detail}")]
    SchemaMismatch {
        detail: String,
        os_version_hint: Option<String>,
    },

    /// The poll exceeded its time budget (FR6). A metric, not an alert.
    #[error("poll exceeded its time budget")]
    Timeout,

    /// The poll was cancelled cooperatively. The ONLY variant that writes nothing and leaves
    /// liveness unchanged — a clean shutdown must not blind every source (else FR19 would
    /// suppress everything at restart).
    #[error("poll was cancelled")]
    Cancelled,

    /// The source answered with a server-side fault (a 5xx). Blinds after repeated failures.
    #[error("remote fault (status {status}): {detail}")]
    RemoteFault { status: u16, detail: String },

    /// The source cannot start — bad or missing configuration. Surfaced at startup, not at
    /// 3 a.m.
    #[error("source is misconfigured: {detail}")]
    Misconfigured { detail: String },
}

impl ConnectorError {
    /// The NFR7 safe default: every cause blinds the source EXCEPT a clean cancellation, which
    /// writes nothing. A future non-blinding variant must justify itself here before NFR7 —
    /// and because the enum is closed, the compiler will make that choice unavoidable.
    pub fn is_blinding(&self) -> bool {
        !matches!(self, ConnectorError::Cancelled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant once, for exhaustive iteration in tests. If a variant is added, this
    /// array stops compiling — the closed-taxonomy guardrail, exercised.
    fn one_of_each() -> Vec<ConnectorError> {
        vec![
            ConnectorError::Unauthorized {
                detail: "401".into(),
            },
            ConnectorError::Unreachable {
                detail: "connection refused".into(),
            },
            ConnectorError::SchemaMismatch {
                detail: "unexpected field".into(),
                os_version_hint: Some("UniFi OS 4.0".into()),
            },
            ConnectorError::Timeout,
            ConnectorError::Cancelled,
            ConnectorError::RemoteFault {
                status: 503,
                detail: "service unavailable".into(),
            },
            ConnectorError::Misconfigured {
                detail: "missing base URL".into(),
            },
        ]
    }

    #[test]
    fn only_cancelled_is_non_blinding() {
        for e in one_of_each() {
            let expected = !matches!(e, ConnectorError::Cancelled);
            assert_eq!(
                e.is_blinding(),
                expected,
                "wrong blinding verdict for {e:?}"
            );
        }
        assert!(!ConnectorError::Cancelled.is_blinding());
        assert!(ConnectorError::Timeout.is_blinding());
    }

    #[test]
    fn display_is_meaningful_for_every_variant() {
        for e in one_of_each() {
            assert!(!e.to_string().is_empty());
        }
        assert!(
            ConnectorError::Unauthorized {
                detail: "401".into()
            }
            .to_string()
            .contains("authentication")
        );
        assert!(ConnectorError::Cancelled.to_string().contains("cancelled"));
    }

    #[test]
    fn schema_mismatch_surfaces_the_os_hint() {
        let e = ConnectorError::SchemaMismatch {
            detail: "x".into(),
            os_version_hint: Some("UniFi OS 4.0".into()),
        };
        // The hint is carried on the variant for the maintainer; the message names the cause.
        match &e {
            ConnectorError::SchemaMismatch {
                os_version_hint, ..
            } => {
                assert_eq!(os_version_hint.as_deref(), Some("UniFi OS 4.0"));
            }
            _ => unreachable!(),
        }
        assert!(e.to_string().contains("schema"));
    }

    #[test]
    fn implements_std_error() {
        // Compile-level: a ConnectorError is a std::error::Error.
        let e: ConnectorError = ConnectorError::Timeout;
        let _dyn: &dyn std::error::Error = &e;
    }
}
