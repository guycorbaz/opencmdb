//! Test support — a hand-scripted in-memory [`Connector`] for exercising the contract.
//!
//! Behind the `test-support` feature (and available to this crate's own `cfg(test)` build).
//! It exists so the [`Connector`] contract (Story 2.3) can be driven before any real source
//! or the committed JSONL fixture exists. It is NOT the `FixtureConnector` of Epic 4 (which
//! replays committed JSONL and is the oracle) — this one is scripted in Rust, zero mocks,
//! zero I/O, and it never ships (`opencmdb-bin` builds without the feature).

use tokio_util::sync::CancellationToken;

use crate::connector::{Connector, ConnectorError, ObservationSink, PollSummary};
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
}
