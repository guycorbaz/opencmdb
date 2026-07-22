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

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

use crate::observation::{Capabilities, ConnectorId, Observation, Scope};

/// Why a poll produced no (or incomplete) observations. Closed by design (D33): every cause
/// blinds the source EXCEPT [`ConnectorError::Cancelled`], which writes nothing. There is no
/// `Other(String)` — that would be `anyhow` in disguise and would make FR5/FR8/FR19
/// inexpressible. The discriminant is machine-readable (the engine matches on it); the
/// payload is human-readable (never matched on).
///
/// NOTE: a `scope` field (D33's "every variant carries scope") is deferred to Epic 13, where
/// the liveness-blindness scope of D34 §3 is built and `source_state` exists; the scheduler
/// (which polls per scope, D34) knows the scope until then. Adding it later is additive.
///
/// **Serde (story 4.5a).** A fixture scripts a poll's failure by naming a variant in the file, so
/// the taxonomy needs an on-disk representation. It is EXTERNALLY tagged (serde's default), which
/// renders unit variants as bare strings (`"Timeout"`) and struct variants as one-key objects
/// (`{"Unreachable":{"detail":"…"}}`) — a heterogeneous shape a fixture author must expect.
/// `deny_unknown_fields` because the corpus is an oracle: a line that parses while meaning
/// something other than it says is worse than a line that does not parse at all.
///
/// Deserializability does NOT make every variant scriptable: [`ConnectorError::Cancelled`] is
/// refused by the fixture reader, because cancellation comes from the token and a file able to
/// mint it could claim liveness was left unchanged when nothing cancelled anything.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

// ── The connector contract (D34) ────────────────────────────────────────────

/// Where a connector emits its observations, one at a time, as they are produced.
///
/// `emit` is SYNC on purpose: it keeps the trait object-safe, so a connector's `poll` can
/// take `&mut dyn ObservationSink` and stay agnostic to what the sink does (collect, forward
/// to a channel, persist). Incremental emission is what lets a timed-out or cancelled poll
/// keep the observations it already produced (D34 §2) — the sink holds them, not a `Vec`
/// returned only on success.
pub trait ObservationSink {
    /// Record one observation the moment the connector produces it.
    fn emit(&mut self, observation: Observation);
}

/// A trivial [`ObservationSink`] that collects into a `Vec`. For tests and simple callers.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct VecSink {
    pub observations: Vec<Observation>,
}

impl ObservationSink for VecSink {
    fn emit(&mut self, observation: Observation) {
        self.observations.push(observation);
    }
}

/// What a successful poll established this cycle: the dated capability descriptor (the poll
/// is the authority on capabilities, D34 §1) and which scopes it actually covered. The
/// observations themselves went to the [`ObservationSink`], not here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollSummary {
    pub capabilities: Capabilities,
    pub scopes_covered: Vec<Scope>,
}

/// A source of observations — UniFi, an ARP/ping scan, a replayed fixture, anything.
///
/// `poll` emits observations INCREMENTALLY through `sink` and checks `cancel` at its own
/// cancellation points (between probes, never mid-probe): a poll cut short by a timeout or a
/// shutdown keeps everything already emitted, because those observations are TRUE — their
/// `observed_at` is the source's, they do not expire because the poll ended early (D34 §2).
/// A `ConnectorError::Cancelled` leaves liveness unchanged (Story 2.2), so cancellation never
/// fabricates a gap (NFR7).
///
/// **Dispatch:** a native `async fn` in a trait is not object-safe, so there is no
/// `Box<dyn Connector>` — connectors are consumed generically (or via an enum of the known
/// kinds). `ObservationSink` is the `dyn` seam. `#[allow(async_fn_in_trait)]`: the `Send`-ness
/// of the returned future (for spawning across a multi-thread runtime) is deferred to the
/// scheduler; if required it becomes a return-position `impl Future + Send`, an additive
/// change that does not pull in `async-trait`.
#[allow(async_fn_in_trait)]
pub trait Connector {
    /// The connector's stable id.
    fn id(&self) -> ConnectorId;

    /// Poll the source once, at logical time `now`, emitting observations into `sink` as they
    /// are produced and honouring `cancel` at the connector's own cancellation points.
    async fn poll(
        &mut self,
        now: crate::observation::Timestamp,
        sink: &mut dyn ObservationSink,
        cancel: CancellationToken,
    ) -> Result<PollSummary, ConnectorError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::{
        Capabilities, Fact, FactKind, L2DomainId, MacAddr, ObsId, Observation, Scope, Timestamp,
        VantageId,
    };
    use std::collections::BTreeSet;
    use uuid::Uuid;

    fn ts() -> Timestamp {
        chrono::DateTime::parse_from_rfc3339("2026-07-20T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    fn a_scope() -> Scope {
        Scope {
            l2_domain: L2DomainId::from_uuid(Uuid::nil()),
            vantage: VantageId::from_uuid(Uuid::nil()),
        }
    }

    fn caps() -> Capabilities {
        Capabilities {
            as_of: ts(),
            kinds: BTreeSet::from([FactKind::Mac]),
        }
    }

    fn obs(n: u8) -> Observation {
        Observation {
            obs_id: ObsId::from_uuid(Uuid::nil()),
            connector_id: ConnectorId::from_uuid(Uuid::nil()),
            observed_at: ts(),
            scope: a_scope(),
            facts: vec![Fact::Mac {
                addr: MacAddr([n, 0, 0, 0, 0, 0]),
                locally_administered: false,
            }],
            raw: None,
        }
    }

    /// The variant's name, by an EXHAUSTIVE match with no `_` arm.
    ///
    /// **This function IS the closed-taxonomy guardrail.** Adding a variant to [`ConnectorError`]
    /// stops it compiling, which forces a decision about how the new variant is written on disk
    /// before it can reach the fixture corpus. Every use of it below is there to make that
    /// compile-time break unavoidable.
    ///
    /// It exists because the guardrail was previously believed to live in [`one_of_each`], and did
    /// not: that function is a `vec![…]` of constructor expressions, so a new variant compiles
    /// straight past it. Measured during the story-4.5a code review by adding a variant — the
    /// workspace built and all core tests passed.
    fn variant_name(e: &ConnectorError) -> &'static str {
        match e {
            ConnectorError::Unauthorized { .. } => "Unauthorized",
            ConnectorError::Unreachable { .. } => "Unreachable",
            ConnectorError::SchemaMismatch { .. } => "SchemaMismatch",
            ConnectorError::Timeout => "Timeout",
            ConnectorError::Cancelled => "Cancelled",
            ConnectorError::RemoteFault { .. } => "RemoteFault",
            ConnectorError::Misconfigured { .. } => "Misconfigured",
        }
    }

    /// Every variant once, for exhaustive iteration in tests.
    ///
    /// **This list does NOT stop compiling when a variant is added** — it is a `vec!` of
    /// constructor expressions, not a `match`. [`variant_name`] is what breaks, and
    /// `every_variant_is_listed_once` is what makes a variant missing from this list fail.
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

    /// Every variant is in [`one_of_each`], exactly once.
    ///
    /// Together with [`variant_name`] — which stops compiling when a variant is added — this is
    /// what closes the taxonomy against the fixture format: the compiler forces the new variant to
    /// be named, and this test forces it to be exercised.
    #[test]
    fn every_variant_is_listed_once() {
        let names: std::collections::BTreeSet<&'static str> =
            one_of_each().iter().map(variant_name).collect();
        assert_eq!(
            names.len(),
            one_of_each().len(),
            "one_of_each lists a variant twice"
        );
        // Update BOTH this number and `variant_name` when the taxonomy changes — `variant_name`
        // will not compile until you do, which is the point.
        assert_eq!(names.len(), 7, "a variant is missing from one_of_each");
    }

    /// Every variant round-trips through JSON, and its serialized form NAMES it.
    ///
    /// The `variant_name` call is not decoration: it makes this test depend on the exhaustive
    /// match, so a variant added to `ConnectorError` cannot reach the fixture format without
    /// someone deciding how it is written.
    #[test]
    fn every_variant_round_trips_through_json() {
        for e in one_of_each() {
            let json = serde_json::to_string(&e).expect("every variant must serialize");
            assert!(
                json.contains(variant_name(&e)),
                "the serialized form must name the variant: {json}"
            );
            let back: ConnectorError =
                serde_json::from_str(&json).expect("every variant must deserialize");
            assert_eq!(back, e, "round-trip changed the variant: {json}");
        }
        // The two shapes a fixture author will meet, pinned so a serde change is visible here
        // rather than in a corpus file that silently stops meaning what it said.
        assert_eq!(
            serde_json::to_string(&ConnectorError::Timeout).unwrap(),
            r#""Timeout""#
        );
        assert_eq!(
            serde_json::to_string(&ConnectorError::Unreachable {
                detail: "connection refused".into()
            })
            .unwrap(),
            r#"{"Unreachable":{"detail":"connection refused"}}"#
        );
    }

    /// A misspelled field must fail, not be ignored: the corpus is an oracle (story 4.1's rule,
    /// applied to the error taxonomy now that it is authored in files).
    #[test]
    fn an_unknown_field_on_a_variant_is_refused() {
        let json = r#"{"Unreachable":{"detail":"x","detial":"y"}}"#;
        assert!(
            serde_json::from_str::<ConnectorError>(json).is_err(),
            "an unknown field must be refused"
        );
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

    #[test]
    fn observation_sink_is_object_safe() {
        let mut sink = VecSink::default();
        let dyn_sink: &mut dyn ObservationSink = &mut sink;
        dyn_sink.emit(obs(0));
        dyn_sink.emit(obs(1));
        assert_eq!(sink.observations.len(), 2);
    }

    /// Emits `emit_count` observations, checking `cancel` between each (its cancellation point).
    struct TinyConnector {
        emit_count: u8,
    }

    impl Connector for TinyConnector {
        fn id(&self) -> ConnectorId {
            ConnectorId::from_uuid(Uuid::nil())
        }

        async fn poll(
            &mut self,
            _now: Timestamp,
            sink: &mut dyn ObservationSink,
            cancel: CancellationToken,
        ) -> Result<PollSummary, ConnectorError> {
            for i in 0..self.emit_count {
                if cancel.is_cancelled() {
                    return Err(ConnectorError::Cancelled);
                }
                sink.emit(obs(i));
            }
            Ok(PollSummary {
                capabilities: caps(),
                scopes_covered: vec![a_scope()],
            })
        }
    }

    #[tokio::test]
    async fn poll_emits_incrementally_and_returns_summary() {
        let mut c = TinyConnector { emit_count: 2 };
        let mut sink = VecSink::default();
        let summary = c
            .poll(ts(), &mut sink, CancellationToken::new())
            .await
            .expect("clean poll");
        assert_eq!(sink.observations.len(), 2);
        assert_eq!(summary.capabilities, caps());
        assert_eq!(summary.scopes_covered, vec![a_scope()]);
    }

    #[tokio::test]
    async fn cancelled_before_poll_emits_nothing() {
        let mut c = TinyConnector { emit_count: 3 };
        let mut sink = VecSink::default();
        let token = CancellationToken::new();
        token.cancel();
        let err = c.poll(ts(), &mut sink, token).await.unwrap_err();
        assert_eq!(err, ConnectorError::Cancelled);
        assert!(!err.is_blinding()); // liveness unchanged — no gap (NFR7)
        assert!(sink.observations.is_empty());
    }

    /// Emits one observation, then is cancelled mid-poll — the emitted one must survive.
    struct CancelsMidway;

    impl Connector for CancelsMidway {
        fn id(&self) -> ConnectorId {
            ConnectorId::from_uuid(Uuid::nil())
        }

        async fn poll(
            &mut self,
            _now: Timestamp,
            sink: &mut dyn ObservationSink,
            cancel: CancellationToken,
        ) -> Result<PollSummary, ConnectorError> {
            sink.emit(obs(0)); // this observation is TRUE and must not be lost
            cancel.cancel(); // something cancels us mid-poll
            if cancel.is_cancelled() {
                return Err(ConnectorError::Cancelled);
            }
            sink.emit(obs(1)); // never reached
            Ok(PollSummary {
                capabilities: caps(),
                scopes_covered: vec![a_scope()],
            })
        }
    }

    #[tokio::test]
    async fn cancel_midway_keeps_already_emitted() {
        let mut c = CancelsMidway;
        let mut sink = VecSink::default();
        let err = c
            .poll(ts(), &mut sink, CancellationToken::new())
            .await
            .unwrap_err();
        assert_eq!(err, ConnectorError::Cancelled);
        // D34 §2: the observation emitted before the cancellation point survives.
        assert_eq!(sink.observations.len(), 1);
    }
}
