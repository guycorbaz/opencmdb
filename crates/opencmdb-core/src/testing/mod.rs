//! Test support — a hand-scripted in-memory [`Connector`] for exercising the contract.
//!
//! Behind the `test-support` feature (and available to this crate's own `cfg(test)` build).
//! It exists so the [`Connector`] contract (Story 2.3) can be driven before any real source
//! or the committed JSONL fixture exists. It is NOT the `FixtureConnector` of Epic 4 (which
//! replays committed JSONL and is the oracle) — this one is scripted in Rust, zero mocks,
//! zero I/O, and it never ships (`opencmdb-bin` builds without the feature).

use tokio_util::sync::CancellationToken;

use crate::connector::{Connector, ConnectorError, ObservationSink, PollSummary, VecSink};
use crate::observation::{Capabilities, ConnectorId, Observation, Scope, Timestamp};

/// What a scripted poll does AFTER emitting its observations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptedOutcome {
    /// Return `Ok(PollSummary)` — a clean, complete poll.
    Complete,
    /// Return `Err(e)` — the "partial emission then error" case. Any observations scripted
    /// before it are still emitted first (they are true; D34 §2).
    Fail(ConnectorError),
}

/// A [`Connector`] whose behaviour is fully scripted in memory: emit these observations,
/// cover these scopes, report these capabilities, then complete or fail.
#[derive(Debug, Clone)]
pub struct ScriptedConnector {
    id: ConnectorId,
    observations: Vec<Observation>,
    capabilities: Capabilities,
    scopes_covered: Vec<Scope>,
    outcome: ScriptedOutcome,
}

impl ScriptedConnector {
    /// A connector that emits nothing and completes cleanly, reporting `capabilities` over
    /// `scopes_covered`. Build the interesting cases from here.
    pub fn new(id: ConnectorId, capabilities: Capabilities, scopes_covered: Vec<Scope>) -> Self {
        Self {
            id,
            observations: Vec::new(),
            capabilities,
            scopes_covered,
            outcome: ScriptedOutcome::Complete,
        }
    }

    /// Script the observations this connector emits (in order) on each poll.
    pub fn with_observations(mut self, observations: Vec<Observation>) -> Self {
        self.observations = observations;
        self
    }

    /// Script the poll to FAIL with `error` after emitting its observations.
    pub fn failing_with(mut self, error: ConnectorError) -> Self {
        self.outcome = ScriptedOutcome::Fail(error);
        self
    }
}

impl Connector for ScriptedConnector {
    fn id(&self) -> ConnectorId {
        self.id
    }

    async fn poll(
        &mut self,
        _now: Timestamp,
        sink: &mut dyn ObservationSink,
        cancel: CancellationToken,
    ) -> Result<PollSummary, ConnectorError> {
        for observation in &self.observations {
            // The cancellation point is BETWEEN emits — never mid-observation. Everything
            // already emitted is true and survives (D34 §2).
            if cancel.is_cancelled() {
                return Err(ConnectorError::Cancelled);
            }
            sink.emit(observation.clone());
        }
        match &self.outcome {
            ScriptedOutcome::Complete => Ok(PollSummary {
                capabilities: self.capabilities.clone(),
                scopes_covered: self.scopes_covered.clone(),
            }),
            ScriptedOutcome::Fail(error) => Err(error.clone()),
        }
    }
}

// ── The consumer-driven contract test (Story 2.5) ───────────────────────────

/// Drive any [`Connector`] through the runtime-agnostic contract invariants. `make` yields a
/// FRESH connector that emits `expected` (in order) then completes cleanly. A future connector
/// plugs in with a single call:
///
/// ```ignore
/// run_connector_contract(|| MyConnector::new(/* … */), &expected).await;
/// ```
///
/// Covered here (universal, using only `tokio-util`; the caller supplies the runtime):
/// - **clean completion** — emits exactly `expected`, returns `Ok`;
/// - **empty stream** — the same, with `expected == []`;
/// - **missing field** — the same, with observations whose `facts` omit kinds (valid, not "gone");
/// - **cancellation** — a pre-cancelled poll returns cleanly, emits only a prefix of `expected`,
///   and any error is a non-blinding `Cancelled` (NFR7).
///
/// The two connector-*scripted* cases — **partial-then-error** and **timeout** — are exercised
/// against a concrete connector (see this module's tests), because they cannot be expressed
/// through an "emit `expected` then complete" factory.
pub async fn run_connector_contract<C, F>(make: F, expected: &[Observation])
where
    C: Connector,
    F: Fn() -> C,
{
    // A clean, uncancelled poll emits exactly the expected observations and returns Ok.
    {
        let mut c = make();
        let mut sink = VecSink::default();
        c.poll(contract_now(), &mut sink, CancellationToken::new())
            .await
            .expect("contract: a clean, uncancelled poll must return Ok");
        assert_eq!(
            sink.observations, expected,
            "contract: a clean poll must emit exactly the expected observations"
        );
    }

    // A pre-cancelled poll must not panic; it emits only a PREFIX of expected; any error is a
    // non-blinding Cancelled (liveness unchanged — NFR7).
    {
        let mut c = make();
        let mut sink = VecSink::default();
        let token = CancellationToken::new();
        token.cancel();
        if let Err(e) = c.poll(contract_now(), &mut sink, token).await {
            assert_eq!(
                e,
                ConnectorError::Cancelled,
                "contract: a cancelled poll must return Cancelled, got {e:?}"
            );
            assert!(
                !e.is_blinding(),
                "contract: Cancelled must leave liveness unchanged (NFR7)"
            );
        }
        assert!(
            expected.starts_with(&sink.observations),
            "contract: a cancelled poll must emit only a prefix of the expected observations"
        );
    }
}

/// A fixed instant for the contract harness. The domain has no clock (D19); a test harness
/// parses a literal — it never reads the wall clock.
fn contract_now() -> Timestamp {
    chrono::DateTime::parse_from_rfc3339("2000-01-01T00:00:00Z")
        .expect("valid literal timestamp")
        .with_timezone(&chrono::Utc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::VecSink;
    use crate::observation::{Fact, FactKind, L2DomainId, MacAddr, ObsId, VantageId};
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

    fn scripted() -> ScriptedConnector {
        ScriptedConnector::new(ConnectorId::from_uuid(Uuid::nil()), caps(), vec![a_scope()])
    }

    #[tokio::test]
    async fn complete_emits_all_and_returns_summary() {
        let mut c = scripted().with_observations(vec![obs(0), obs(1)]);
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
    async fn empty_batch_completes_with_no_observations() {
        let mut c = scripted();
        let mut sink = VecSink::default();
        let summary = c
            .poll(ts(), &mut sink, CancellationToken::new())
            .await
            .expect("clean poll");
        assert!(sink.observations.is_empty());
        assert_eq!(summary.scopes_covered, vec![a_scope()]);
    }

    #[tokio::test]
    async fn partial_then_error_keeps_emitted_observation() {
        let err = ConnectorError::SchemaMismatch {
            detail: "unexpected field".into(),
            os_version_hint: None,
        };
        let mut c = scripted()
            .with_observations(vec![obs(0)])
            .failing_with(err.clone());
        let mut sink = VecSink::default();
        let got = c
            .poll(ts(), &mut sink, CancellationToken::new())
            .await
            .unwrap_err();
        assert_eq!(got, err);
        assert_eq!(sink.observations.len(), 1); // emitted before the error, and it survives
    }

    #[tokio::test]
    async fn cancelled_before_poll_emits_nothing() {
        let mut c = scripted().with_observations(vec![obs(0), obs(1)]);
        let mut sink = VecSink::default();
        let token = CancellationToken::new();
        token.cancel();
        let got = c.poll(ts(), &mut sink, token).await.unwrap_err();
        assert_eq!(got, ConnectorError::Cancelled);
        assert!(!got.is_blinding());
        assert!(sink.observations.is_empty());
    }

    // ── the reusable contract harness, driven against ScriptedConnector ──────

    /// An observation carrying ONLY a Mac fact — a source blind to hostnames emits this; it is
    /// valid, NOT a "gone" (NFR7). Exercises the missing-field case.
    fn sparse_obs() -> Observation {
        Observation {
            facts: vec![Fact::Mac {
                addr: MacAddr([9, 0, 0, 0, 0, 0]),
                locally_administered: false,
            }],
            ..obs(0)
        }
    }

    #[tokio::test]
    async fn contract_passes_for_a_normal_batch() {
        let expected = vec![obs(0), obs(1)];
        let e = expected.clone();
        run_connector_contract(|| scripted().with_observations(e.clone()), &expected).await;
    }

    #[tokio::test]
    async fn contract_passes_for_the_empty_stream() {
        run_connector_contract(scripted, &[]).await;
    }

    #[tokio::test]
    async fn contract_passes_with_a_missing_field() {
        let expected = vec![sparse_obs()];
        let e = expected.clone();
        run_connector_contract(|| scripted().with_observations(e.clone()), &expected).await;
    }

    // ── the two connector-scripted cases ─────────────────────────────────────

    #[tokio::test]
    async fn contract_partial_then_error_keeps_emitted() {
        let err = ConnectorError::SchemaMismatch {
            detail: "unexpected field".into(),
            os_version_hint: None,
        };
        let mut c = scripted()
            .with_observations(vec![obs(0)])
            .failing_with(err.clone());
        let mut sink = VecSink::default();
        let got = c
            .poll(ts(), &mut sink, CancellationToken::new())
            .await
            .unwrap_err();
        assert_eq!(got, err);
        assert_eq!(sink.observations.len(), 1); // emitted before the error survives (D34 §2)
    }

    /// Emits observations with a yield point between them, so a `timeout` can drop the future
    /// mid-poll — proving the already-emitted observations survive because the sink is external.
    struct YieldingConnector {
        count: u8,
    }

    impl Connector for YieldingConnector {
        fn id(&self) -> ConnectorId {
            ConnectorId::from_uuid(Uuid::nil())
        }

        async fn poll(
            &mut self,
            _now: Timestamp,
            sink: &mut dyn ObservationSink,
            _cancel: CancellationToken,
        ) -> Result<PollSummary, ConnectorError> {
            for i in 0..self.count {
                sink.emit(obs(i));
                tokio::task::yield_now().await; // a preemption/cancellation point
            }
            Ok(PollSummary {
                capabilities: caps(),
                scopes_covered: vec![a_scope()],
            })
        }
    }

    #[tokio::test]
    async fn contract_timeout_keeps_already_emitted() {
        let mut c = YieldingConnector { count: 3 };
        let mut sink = VecSink::default();
        // A zero-duration timeout fires at the first yield point: the poll future is dropped,
        // but whatever reached the (external) sink survives — no total loss (D34 §2).
        let _ = tokio::time::timeout(
            std::time::Duration::ZERO,
            c.poll(ts(), &mut sink, CancellationToken::new()),
        )
        .await;
        assert!(
            sink.observations.len() <= 3,
            "the sink holds a valid prefix, never more than was scripted"
        );
    }
}
