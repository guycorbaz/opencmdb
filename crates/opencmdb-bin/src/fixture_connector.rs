//! `FixtureConnector` — the fixture IS a connector (D19, story 4.4).
//!
//! It replays a committed JSONL stream through the real [`Connector`] trait and passes the same
//! `run_connector_contract` every other connector passes, with no special-casing. *"The punchline:
//! `FixtureConnector` implements `Connector` by replaying JSONL. THE FIXTURE IS A CONNECTOR. Zero
//! mocks. Zero network. If the trait does not allow this, the trait is wrong."*
//!
//! It lives here, in the shipped crate beside `arp_ping.rs`, and not under `tests/` — there it
//! would not face the same compilation gates as its siblings and "zero mocks" would become a
//! slogan (D56).
//!
//! # One stream is one connector
//!
//! A stream carrying an observation attributed to another `connector_id` is refused at load. A
//! multi-source trap — two sources disagreeing about one device — is therefore two
//! `FixtureConnector`s over two files, never one file carrying two ids. The trap families inherit
//! that constraint.
//!
//! # What the format cannot express, recorded rather than worked around
//!
//! `poll` must return a [`PollSummary`] carrying `capabilities` and `scopes_covered`; the fixture
//! format frozen by story 4.1 is a stream of [`Observation`]s and nothing else, so it can express
//! neither. Both are therefore supplied at CONSTRUCTION — which is exactly the "state outside the
//! JSONL" D34 §1 refused when it argued capabilities should travel with the batch.
//!
//! This is recorded, not papered over (`deferred-work.md`, story-4.4 entries). Story 4.5 puts the
//! poll's outcome in the file — *"one JSONL line"* — and owns the capability half; `scopes_covered`
//! follows it by extension and is assigned to nobody yet. `Capabilities.as_of` is the sharpest
//! instance: it is caller-supplied while every observation is dated by the file, so a replay can
//! date its descriptor in a moment its own stream contradicts.
//!
//! **Deriving `capabilities` from the observations would be worse than the gap it closes.** A
//! capability read off what was seen cannot express *"capable of hostnames, saw none"* — the one
//! distinction the descriptor exists for, and the one that stops *"no diverging uplink → same
//! switch → merge"* from merging happily.
//!
//! What construction DOES do is refuse a stream that contradicts the values it was given. The three
//! checks all run in one direction — the file may not exceed the declaration — so *covered and
//! empty* and *capable and unseen* both stay legal, which is precisely the space a derivation would
//! have collapsed.
//!
//! Wired into the running app in a later story; the contract and these tests prove it meanwhile.
#![allow(dead_code)]

use std::collections::HashSet;

use opencmdb_core::connector::{Connector, ConnectorError, ObservationSink, PollSummary};
use opencmdb_core::observation::{Capabilities, ConnectorId, Observation, Scope, Timestamp};
use tokio_util::sync::CancellationToken;

use crate::fixtures::{FixtureError, fixture_path, read_jsonl};

/// Replays a stream of observations as a [`Connector`]. `Debug` because the load invariants are
/// tested through `Result::expect_err`, which requires it on the `Ok` type.
#[derive(Debug)]
pub struct FixtureConnector {
    id: ConnectorId,
    capabilities: Capabilities,
    scopes_covered: Vec<Scope>,
    observations: Vec<Observation>,
}

impl FixtureConnector {
    /// Load a corpus-relative replay stream (e.g. `scenario/replay/minimal.jsonl`).
    ///
    /// The file is read HERE, once. [`Connector::poll`] then performs no I/O at all: a poll that
    /// re-reads the disk is not a replay, it is a race against whatever else is writing.
    pub fn load(
        id: ConnectorId,
        capabilities: Capabilities,
        scopes_covered: Vec<Scope>,
        relative_path: &str,
    ) -> Result<Self, FixtureError> {
        let observations = read_jsonl(&fixture_path(relative_path)?)?;
        Self::from_observations(
            id,
            capabilities,
            scopes_covered,
            relative_path,
            observations,
        )
    }

    /// Build from observations already in memory. `origin` labels them in any error message —
    /// the corpus-relative path for [`Self::load`], a caller-chosen label otherwise.
    ///
    /// Every load invariant is checked HERE, so the file path and the in-memory path cannot
    /// diverge in what they ADMIT. They do differ in what they REPORT for one violation: a
    /// repeated `obs_id` read from a file surfaces as [`FixtureError::DuplicateObservationId`]
    /// (raised by `read_jsonl`, naming both line numbers), and in memory as
    /// [`FixtureError::RepeatedObservationId`] (no lines to name). A caller that wants to handle
    /// "this stream repeats an id" must match BOTH.
    pub fn from_observations(
        id: ConnectorId,
        capabilities: Capabilities,
        scopes_covered: Vec<Scope>,
        origin: &str,
        observations: Vec<Observation>,
    ) -> Result<Self, FixtureError> {
        // `read_jsonl` refuses a repeated `obs_id` for a FILE, naming both lines. A `Vec` never
        // passed through it, so without this check every in-memory construction — this module's
        // tests, and the harnesses of stories 4.6 and 4.7 — would bypass the anchor the whole
        // labelling format rests on ("a trap points at an obs_id, never by line number").
        let mut seen: HashSet<uuid::Uuid> = HashSet::with_capacity(observations.len());
        for observation in &observations {
            let obs_id = observation.obs_id.as_uuid();
            if !seen.insert(obs_id) {
                return Err(FixtureError::RepeatedObservationId {
                    origin: origin.to_string(),
                    obs_id: obs_id.to_string(),
                });
            }

            if observation.connector_id != id {
                return Err(FixtureError::ForeignConnectorId {
                    origin: origin.to_string(),
                    expected: id,
                    found: observation.connector_id,
                    obs_id: obs_id.to_string(),
                });
            }

            // `Scope` is `Copy + Hash + Eq` but NOT `Ord`, so the module's usual `BTreeSet` does
            // not apply. `scopes_covered` is a handful of entries; a linear search is honest.
            if !scopes_covered.contains(&observation.scope) {
                return Err(FixtureError::UncoveredScope {
                    origin: origin.to_string(),
                    l2_domain: observation.scope.l2_domain,
                    vantage: observation.scope.vantage,
                    obs_id: obs_id.to_string(),
                });
            }

            for fact in &observation.facts {
                let kind = fact.kind();
                if !capabilities.can_emit(kind) {
                    return Err(FixtureError::UndeclaredFactKind {
                        origin: origin.to_string(),
                        kind,
                        obs_id: obs_id.to_string(),
                    });
                }
            }
        }

        Ok(Self {
            id,
            capabilities,
            scopes_covered,
            observations,
        })
    }
}

impl Connector for FixtureConnector {
    fn id(&self) -> ConnectorId {
        self.id
    }

    /// `now` is deliberately unused: every observation is dated by the FILE (`observed_at`), and
    /// the engine never touches the clock (D19). That determinism is what makes the corpus an
    /// oracle rather than a snapshot.
    async fn poll(
        &mut self,
        _now: Timestamp,
        sink: &mut dyn ObservationSink,
        cancel: CancellationToken,
    ) -> Result<PollSummary, ConnectorError> {
        // BEFORE any work, not only between emits. A check placed only inside the loop is never
        // reached by an EMPTY stream, so a poll cancelled before it began would return
        // `Ok(PollSummary { scopes_covered })` — a claim to have covered a scope it never touched.
        // `Cancelled` must leave liveness UNCHANGED (NFR7/D34); an `Ok` summary refreshes it.
        // `arp_ping.rs` checks here for the same reason, and `run_connector_contract`'s
        // cancellation clause accepts `Ok`, so it cannot catch the omission.
        if cancel.is_cancelled() {
            return Err(ConnectorError::Cancelled);
        }
        for observation in &self.observations {
            // The cancellation point is BETWEEN emits — never mid-observation. Everything already
            // emitted is true and survives (D34 §2). Nothing is consumed, so a second poll
            // replays the same stream.
            if cancel.is_cancelled() {
                return Err(ConnectorError::Cancelled);
            }
            sink.emit(observation.clone());
        }
        // No check AFTER the loop, deliberately: at this point every observation was emitted and
        // the poll did its whole job. D34 says a cancelled poll KEEPS what it emitted — not that a
        // COMPLETED one must report cancellation. So a token cancelled during the final emit still
        // yields `Ok`. The asymmetry is a decision (story 4.4 review, 2026-07-22), not an oversight.
        Ok(PollSummary {
            capabilities: self.capabilities.clone(),
            scopes_covered: self.scopes_covered.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencmdb_core::connector::VecSink;
    use opencmdb_core::observation::{Fact, FactKind, L2DomainId, MacAddr, ObsId, VantageId};
    use opencmdb_core::testing::run_connector_contract;
    use std::collections::BTreeSet;
    use std::net::Ipv4Addr;
    use uuid::Uuid;

    const MINIMAL: &str = "scenario/replay/minimal.jsonl";

    /// The `connector_id`, scope and capability set the committed `minimal.jsonl` needs.
    ///
    /// These ARE a restatement of the file's ids — a second, independent statement of them, in the
    /// same spirit as `fixtures.rs`'s `expected()`. The consequence is worth knowing: regenerate
    /// the fixture with fresh ids and these tests red with a `ForeignConnectorId`, which names the
    /// mismatch but does not say "the corpus moved". What must NOT be restated here is the
    /// OBSERVATIONS — those come from `read_jsonl`, which is what AC2 rests on.
    fn corpus_id() -> ConnectorId {
        ConnectorId::from_uuid(u("33333333-3333-4333-8333-333333333333"))
    }

    fn corpus_scope() -> Scope {
        Scope {
            l2_domain: L2DomainId::from_uuid(u("11111111-1111-4111-8111-111111111111")),
            vantage: VantageId::from_uuid(u("22222222-2222-4222-8222-222222222222")),
        }
    }

    /// Deliberately WIDER than what `minimal.jsonl` emits: it declares `Uplink` and `DhcpLease`
    /// which the stream never carries. *Capable and unseen* must stay legal — that is the whole
    /// reason the descriptor exists (D34 §1), and a capability set trimmed to what was observed
    /// would be the derivation this story refuses.
    fn corpus_caps() -> Capabilities {
        Capabilities {
            as_of: ts("2026-01-01T00:00:10Z"),
            kinds: BTreeSet::from([
                FactKind::Mac,
                FactKind::IpV4,
                FactKind::Hostname,
                FactKind::OuiVendor,
                FactKind::Rtt,
                FactKind::Uplink,
                FactKind::DhcpLease,
            ]),
        }
    }

    fn u(s: &str) -> Uuid {
        Uuid::parse_str(s).unwrap()
    }

    fn ts(s: &str) -> Timestamp {
        chrono::DateTime::parse_from_rfc3339(s)
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    /// A hand-authored observation for the load-invariant tests. `fixtures.rs`'s own `expected()`
    /// is private to its test module, so these are authored here — and every value is synthetic:
    /// RFC 5737 documentation addresses, locally-administered MACs, invented hostnames. The
    /// repository is public and a real capture in it is disqualifying (D19).
    ///
    /// The invariant tests below put the OFFENDING observation second, behind a valid one. With a
    /// single-element `Vec` an implementation that validated only `observations.first()` would pass
    /// every one of them.
    fn obs(n: u128, id: ConnectorId, scope: Scope, facts: Vec<Fact>) -> Observation {
        Observation {
            obs_id: ObsId::from_uuid(Uuid::from_u128(n)),
            connector_id: id,
            observed_at: ts("2026-01-01T00:00:00Z"),
            scope,
            facts,
            raw: None,
        }
    }

    /// The single list of facts this module hand-authors. Every helper below draws from here, so
    /// the privacy check has exactly one place to look and cannot drift out of date silently.
    fn authored_facts() -> Vec<Fact> {
        vec![a_mac(), an_ip()]
    }

    fn a_mac() -> Fact {
        Fact::Mac {
            addr: MacAddr([0x02, 0x00, 0x5e, 0x00, 0x53, 0x10]),
            locally_administered: true,
        }
    }

    fn an_ip() -> Fact {
        Fact::IpV4 {
            addr: Ipv4Addr::new(192, 0, 2, 20),
        }
    }

    fn mac_only_caps() -> Capabilities {
        Capabilities {
            as_of: ts("2026-01-01T00:00:00Z"),
            kinds: BTreeSet::from([FactKind::Mac]),
        }
    }

    fn other_scope() -> Scope {
        Scope {
            l2_domain: L2DomainId::from_uuid(Uuid::from_u128(0xbeef)),
            vantage: VantageId::from_uuid(Uuid::from_u128(0xcafe)),
        }
    }

    // ── The corpus loads, and replays exactly what the file says ─────────────

    fn corpus_connector() -> FixtureConnector {
        FixtureConnector::load(corpus_id(), corpus_caps(), vec![corpus_scope()], MINIMAL)
            .expect("the committed fixture must load")
    }

    /// AC2: the poll emits the file's observations, in file order.
    ///
    /// What this proves exactly: `poll` neither drops, duplicates nor reorders what was loaded.
    /// Both sides go through the same `read_jsonl` on the same path, so it does NOT independently
    /// re-derive the file's bytes — `fixtures.rs`'s `expected()` is what pins those. The length
    /// assertion is the one tether to the corpus actually holding three observations.
    #[tokio::test]
    async fn a_poll_emits_the_file_in_file_order() {
        let expected = read_jsonl(&fixture_path(MINIMAL).unwrap()).expect("the fixture reads");
        assert_eq!(
            expected.len(),
            3,
            "the corpus fixture is three observations"
        );

        let mut connector = corpus_connector();
        let mut sink = VecSink::default();
        connector
            .poll(
                ts("2000-01-01T00:00:00Z"),
                &mut sink,
                CancellationToken::new(),
            )
            .await
            .expect("a clean poll");

        assert_eq!(sink.observations, expected);
    }

    /// AC4: a replay is idempotent. Nothing is consumed, so the second poll is neither empty nor
    /// a continuation. `run_connector_contract` cannot show this — it builds a FRESH connector for
    /// each of its two cases, so it never polls one instance twice.
    #[tokio::test]
    async fn polling_twice_replays_the_same_stream() {
        let mut connector = corpus_connector();
        let mut first = VecSink::default();
        let mut second = VecSink::default();

        connector
            .poll(
                ts("2000-01-01T00:00:00Z"),
                &mut first,
                CancellationToken::new(),
            )
            .await
            .expect("first poll");
        connector
            .poll(
                ts("2000-01-01T00:00:00Z"),
                &mut second,
                CancellationToken::new(),
            )
            .await
            .expect("second poll");

        assert!(!first.observations.is_empty());
        assert_eq!(first.observations, second.observations);
    }

    /// AC5c: the summary carries back exactly what the constructor was given. This is the only
    /// test that looks at the `PollSummary` at all — `run_connector_contract` ignores it — so
    /// without it the design decision this story turns on would ship with no coverage.
    #[tokio::test]
    async fn the_poll_summary_round_trips_what_construction_declared() {
        let mut connector = corpus_connector();
        let mut sink = VecSink::default();
        let summary = connector
            .poll(
                ts("2000-01-01T00:00:00Z"),
                &mut sink,
                CancellationToken::new(),
            )
            .await
            .expect("a clean poll");

        assert_eq!(summary.capabilities, corpus_caps());
        assert_eq!(summary.scopes_covered, vec![corpus_scope()]);
        // The declared capabilities exceed what the stream emits, and that is the point: a source
        // may be capable and see nothing. Nothing derived them from the observations.
        assert!(summary.capabilities.can_emit(FactKind::Uplink));
        assert!(
            !sink
                .observations
                .iter()
                .flat_map(|o| &o.facts)
                .any(|f| f.kind() == FactKind::Uplink)
        );
    }

    /// AC1/AC9: `id()` is the constructed id, and the corpus is reached through
    /// `crate::fixtures::fixture_path` — this module never writes the corpus path itself.
    #[test]
    fn the_connector_reports_the_id_it_was_built_with() {
        assert_eq!(corpus_connector().id(), corpus_id());
    }

    // ── The contract, driven twice ───────────────────────────────────────────

    /// AC5 (a): the committed fixture, through the same harness every connector faces.
    #[tokio::test]
    async fn the_contract_passes_over_the_committed_fixture() {
        let expected = read_jsonl(&fixture_path(MINIMAL).unwrap()).expect("the fixture reads");
        run_connector_contract(corpus_connector, &expected).await;
    }

    /// AC5 (b): the empty stream. `scopes_covered` is deliberately NON-EMPTY — *covered and
    /// empty* is the meaningful case, because `(connector, scope)` is the blindness unit and a
    /// poll claiming to have covered nothing updates no liveness at all.
    #[tokio::test]
    async fn the_contract_passes_over_an_empty_stream() {
        fn empty() -> FixtureConnector {
            FixtureConnector::from_observations(
                corpus_id(),
                mac_only_caps(),
                vec![corpus_scope()],
                "<empty>",
                vec![],
            )
            .expect("an empty stream must load")
        }
        run_connector_contract(empty, &[]).await;

        let mut connector = empty();
        let mut sink = VecSink::default();
        let summary = connector
            .poll(
                ts("2000-01-01T00:00:00Z"),
                &mut sink,
                CancellationToken::new(),
            )
            .await
            .expect("a clean poll over nothing");
        assert!(sink.observations.is_empty());
        assert_eq!(
            summary.scopes_covered,
            vec![corpus_scope()],
            "covered and empty: the poll still says which scope it covered"
        );
    }

    // ── Cancellation, made non-vacuous ───────────────────────────────────────

    /// A sink that cancels the poll from INSIDE `emit`, after the first observation.
    ///
    /// `poll` contains no `.await`, so its future runs to completion the first time it is polled
    /// and no external task can interleave with it. The sink is the only seam that can observe a
    /// poll mid-stream — and it is enough, because `ObservationSink::emit` is sync by design.
    struct CancelAfterFirst {
        token: CancellationToken,
        observations: Vec<Observation>,
    }

    impl ObservationSink for CancelAfterFirst {
        fn emit(&mut self, observation: Observation) {
            self.observations.push(observation);
            self.token.cancel();
        }
    }

    /// AC5b: the cancellation point exists and is honoured mid-stream, and what was already
    /// emitted survives (D34 §2).
    ///
    /// The contract's own cancellation clause cannot show this: it asserts
    /// `expected.starts_with(&sink.observations)`, which an empty sink satisfies and which a
    /// connector that never reads the token also satisfies. Measured: removing the
    /// `is_cancelled()` checks from `poll` leaves BOTH contract drives green and reds exactly the
    /// two tests written for cancellation — this one and `a_pre_cancelled_poll_emits_nothing`.
    #[tokio::test]
    async fn cancelling_between_emits_keeps_what_was_already_emitted() {
        let token = CancellationToken::new();
        let mut sink = CancelAfterFirst {
            token: token.clone(),
            observations: Vec::new(),
        };
        let mut connector = corpus_connector();

        let err = connector
            .poll(ts("2000-01-01T00:00:00Z"), &mut sink, token)
            .await
            .expect_err("a poll cancelled mid-stream must fail");

        assert_eq!(err, ConnectorError::Cancelled);
        assert!(
            !err.is_blinding(),
            "Cancelled leaves liveness unchanged (NFR7)"
        );
        assert_eq!(
            sink.observations.len(),
            1,
            "the observation emitted before the cancellation point is true and survives"
        );
    }

    /// A pre-cancelled poll emits nothing at all — the token is checked BEFORE the first emit.
    #[tokio::test]
    async fn a_pre_cancelled_poll_emits_nothing() {
        let token = CancellationToken::new();
        token.cancel();
        let mut connector = corpus_connector();
        let mut sink = VecSink::default();

        let err = connector
            .poll(ts("2000-01-01T00:00:00Z"), &mut sink, token)
            .await
            .expect_err("a pre-cancelled poll must fail");
        assert_eq!(err, ConnectorError::Cancelled);
        assert!(sink.observations.is_empty());
    }

    /// The case the code review caught: a pre-cancelled poll over an EMPTY stream.
    ///
    /// With the check only inside the emit loop this returned `Ok(PollSummary { scopes_covered })`
    /// — a poll cancelled before it began, claiming to have covered a scope it never touched.
    /// `Cancelled` must leave liveness UNCHANGED (NFR7/D34); an `Ok` summary refreshes it. The
    /// contract harness cannot catch this: its cancellation block accepts `Ok`, so the empty-stream
    /// drive passed that clause vacuously.
    #[tokio::test]
    async fn a_pre_cancelled_poll_over_an_empty_stream_is_still_cancelled() {
        let mut connector = FixtureConnector::from_observations(
            corpus_id(),
            mac_only_caps(),
            vec![corpus_scope()],
            "<empty>",
            vec![],
        )
        .expect("an empty stream must load");
        let token = CancellationToken::new();
        token.cancel();
        let mut sink = VecSink::default();

        let err = connector
            .poll(ts("2000-01-01T00:00:00Z"), &mut sink, token)
            .await
            .expect_err("a cancelled poll must not report a completed one");
        assert_eq!(err, ConnectorError::Cancelled);
        assert!(sink.observations.is_empty());
    }

    // ── The four load invariants ─────────────────────────────────────────────

    /// AC6: an observation attributed to another connector is refused, naming both ids and the
    /// observation. Emitting it would fabricate provenance.
    #[test]
    fn a_foreign_connector_id_is_refused() {
        let stranger = ConnectorId::from_uuid(Uuid::from_u128(0x99));
        let err = FixtureConnector::from_observations(
            corpus_id(),
            mac_only_caps(),
            vec![corpus_scope()],
            "<foreign>",
            vec![
                obs(1, corpus_id(), corpus_scope(), vec![a_mac()]),
                obs(2, stranger, corpus_scope(), vec![a_mac()]),
            ],
        )
        .expect_err("a stream carrying another connector's observation must be refused");

        match &err {
            FixtureError::ForeignConnectorId {
                expected, found, ..
            } => {
                assert_eq!(*expected, corpus_id());
                assert_eq!(*found, stranger);
            }
            other => panic!("expected a foreign-connector error, got {other:?}"),
        }
        let rendered = err.to_string();
        assert!(rendered.contains(&stranger.to_string()), "{rendered}");
        assert!(rendered.contains(&corpus_id().to_string()), "{rendered}");
        assert!(rendered.contains("<foreign>"), "{rendered}");
        assert!(
            rendered.contains(&Uuid::from_u128(2).to_string()),
            "the message must name the offending observation: {rendered}"
        );
    }

    /// AC7: a scope observed but not covered is refused, naming the scope's two ids.
    #[test]
    fn a_scope_the_poll_does_not_cover_is_refused() {
        let err = FixtureConnector::from_observations(
            corpus_id(),
            mac_only_caps(),
            vec![corpus_scope()],
            "<uncovered>",
            vec![
                obs(3, corpus_id(), corpus_scope(), vec![a_mac()]),
                obs(4, corpus_id(), other_scope(), vec![a_mac()]),
            ],
        )
        .expect_err("observing an uncovered scope must be refused");

        match &err {
            FixtureError::UncoveredScope {
                l2_domain, vantage, ..
            } => {
                assert_eq!(*l2_domain, other_scope().l2_domain);
                assert_eq!(*vantage, other_scope().vantage);
            }
            other => panic!("expected an uncovered-scope error, got {other:?}"),
        }
        assert!(
            err.to_string()
                .contains(&other_scope().l2_domain.to_string()),
            "{err}"
        );
    }

    /// AC7: the reverse stays legitimate. A scope covered with nothing seen in it is a meaningful
    /// answer, not an error — it is what makes an absence mean something.
    #[test]
    fn covering_a_scope_that_was_never_observed_is_legitimate() {
        FixtureConnector::from_observations(
            corpus_id(),
            mac_only_caps(),
            vec![corpus_scope(), other_scope()],
            "<covered-and-empty>",
            vec![obs(10, corpus_id(), corpus_scope(), vec![a_mac()])],
        )
        .expect("covering more than was observed must stay legal");
    }

    /// AC7b: a fact of a kind the capabilities do not declare is refused, naming the kind.
    #[test]
    fn a_fact_kind_the_capabilities_deny_is_refused() {
        let err = FixtureConnector::from_observations(
            corpus_id(),
            mac_only_caps(), // declares Mac only
            vec![corpus_scope()],
            "<undeclared>",
            vec![
                obs(5, corpus_id(), corpus_scope(), vec![a_mac()]),
                obs(6, corpus_id(), corpus_scope(), vec![a_mac(), an_ip()]),
            ],
        )
        .expect_err("emitting an undeclared fact kind must be refused");

        match &err {
            FixtureError::UndeclaredFactKind { kind, .. } => assert_eq!(*kind, FactKind::IpV4),
            other => panic!("expected an undeclared-kind error, got {other:?}"),
        }
        let rendered = err.to_string();
        assert!(rendered.contains("IpV4"), "must name the kind: {rendered}");
        assert!(
            rendered.contains(&Uuid::from_u128(6).to_string()),
            "must name the offending observation: {rendered}"
        );
        assert!(
            rendered.contains("<undeclared>"),
            "must name the origin: {rendered}"
        );
    }

    /// AC7b: the reverse stays legitimate, and it is the entire reason `capabilities` exists —
    /// *capable of hostnames, saw none* must not be expressible as an error.
    #[test]
    fn declaring_a_capability_that_was_never_exercised_is_legitimate() {
        let caps = Capabilities {
            as_of: ts("2026-01-01T00:00:00Z"),
            kinds: BTreeSet::from([FactKind::Mac, FactKind::Hostname, FactKind::Uplink]),
        };
        FixtureConnector::from_observations(
            corpus_id(),
            caps,
            vec![corpus_scope()],
            "<capable-and-unseen>",
            vec![obs(11, corpus_id(), corpus_scope(), vec![a_mac()])],
        )
        .expect("declaring more than was emitted must stay legal");
    }

    /// AC7c: an in-memory stream is held to the same `obs_id` uniqueness a file gets from
    /// `read_jsonl`. Without this, every construction that does not go through a file — this
    /// module's tests, and the harnesses of 4.6 and 4.7 — would bypass the anchor the labelling
    /// format rests on.
    #[test]
    fn an_in_memory_stream_repeating_an_obs_id_is_refused() {
        let err = FixtureConnector::from_observations(
            corpus_id(),
            mac_only_caps(),
            vec![corpus_scope()],
            "<repeated>",
            vec![
                obs(7, corpus_id(), corpus_scope(), vec![a_mac()]),
                obs(8, corpus_id(), corpus_scope(), vec![a_mac()]),
                obs(7, corpus_id(), corpus_scope(), vec![a_mac()]),
            ],
        )
        .expect_err("a repeated obs_id must be refused in memory too");

        match &err {
            FixtureError::RepeatedObservationId { obs_id, .. } => {
                assert_eq!(*obs_id, Uuid::from_u128(7).to_string());
            }
            other => panic!("expected a repeated-obs_id error, got {other:?}"),
        }
        assert!(err.to_string().contains("more than once"), "{err}");
    }

    /// The file path is held to the same rules: `load` funnels into `from_observations`, so a
    /// corpus stream cannot be admitted under looser terms than an in-memory one.
    #[test]
    fn load_applies_the_same_invariants_as_from_observations() {
        let err = FixtureConnector::load(
            ConnectorId::from_uuid(Uuid::from_u128(0x1234)), // not the corpus's id
            corpus_caps(),
            vec![corpus_scope()],
            MINIMAL,
        )
        .expect_err("the committed stream belongs to another connector id");
        assert!(
            matches!(err, FixtureError::ForeignConnectorId { .. }),
            "{err:?}"
        );
        assert!(
            err.to_string().contains(MINIMAL),
            "the origin is the path: {err}"
        );
    }

    /// A path leaving the corpus is refused before anything is read — `load` inherits
    /// `fixture_path`'s containment rule rather than restating it.
    #[test]
    fn load_refuses_a_path_leaving_the_corpus() {
        let err = FixtureConnector::load(
            corpus_id(),
            corpus_caps(),
            vec![corpus_scope()],
            "../../etc/passwd",
        )
        .expect_err("a path leaving the corpus must be refused");
        assert!(matches!(err, FixtureError::OutsideCorpus { .. }), "{err:?}");
    }

    /// Every FACT this module authors is synthetic, and the check walks them rather than naming
    /// two by hand: `authored_facts()` is the single list the helpers draw from, so a fact added
    /// there without a privacy rule fails HERE — and `Fact` is `#[non_exhaustive]` with no `_` arm
    /// below, so a new variant reaching this module must be classified rather than slip past.
    ///
    /// Scope: this covers the module's hand-authored FACTS. The committed corpus is covered by
    /// `fixtures.rs`'s own `the_corpus_carries_no_real_network_data`, and the UUIDs used here are
    /// `Uuid::from_u128` counters and documentation ids — they carry no network data to leak.
    /// The repository is public and a real capture in it is disqualifying (D19).
    #[test]
    fn every_fact_this_module_authors_is_synthetic() {
        let facts = authored_facts();
        assert!(
            !facts.is_empty(),
            "a privacy check over nothing proves nothing"
        );
        for fact in &facts {
            match fact {
                Fact::Mac { addr, .. } => assert!(
                    addr.is_locally_administered(),
                    "{addr} is not locally administered — a real vendor address must never be committed"
                ),
                Fact::IpV4 { addr } => assert!(
                    matches!(
                        [addr.octets()[0], addr.octets()[1], addr.octets()[2]],
                        [192, 0, 2] | [198, 51, 100] | [203, 0, 113]
                    ),
                    "{addr} is not in an RFC 5737 documentation range"
                ),
                Fact::DhcpLease { ip, .. } => assert!(
                    matches!(
                        [ip.octets()[0], ip.octets()[1], ip.octets()[2]],
                        [192, 0, 2] | [198, 51, 100] | [203, 0, 113]
                    ),
                    "{ip} is not in an RFC 5737 documentation range"
                ),
                Fact::Uplink { peer_mac, .. } => {
                    assert!(peer_mac.is_locally_administered(), "{peer_mac}")
                }
                Fact::Hostname { name, .. } => assert!(
                    name.starts_with("doc-"),
                    "hostnames must be invented, not captured: {name}"
                ),
                Fact::OuiVendor { .. } | Fact::Rtt { .. } => {}
                other => {
                    panic!("a new Fact variant reached this module with no privacy rule: {other:?}")
                }
            }
        }
    }
}
