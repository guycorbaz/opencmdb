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
//! # The poll's outcome comes from the file (story 4.5a)
//!
//! A stream is not only observations. A `failure` control record ends the poll with a
//! [`ConnectorError`] named in the file, and everything emitted before it stays in the sink —
//! those observations are TRUE, and they do not expire because the poll was cut (D34 §2). That is
//! D35's layer A: *"`FixtureConnector` replays `Result`s (401, timeout, partial). Tests the
//! engine."*
//!
//! Two rules the READER enforces, neither of which this module depends on: a stream may not script
//! `Cancelled` (re-checked here independently, because an in-memory stream never passes the
//! reader), and nothing may follow a terminal failure in a FILE (`poll` returns at the first
//! failure regardless of what follows, so its behaviour is the same either way). Those rules exist
//! for `read_traps` and the committed corpus — see [`FixtureConnector::from_records`].
//!
//! # The descriptor travels with the batch (story 4.5b)
//!
//! A `capability` control record changes the source's [`Capabilities`] mid-stream and the poll
//! CONTINUES, returning `Ok` with the descriptor in force when the stream ended. That closes what
//! story 4.4 had to record as a gap: the descriptor is now **dated by the file**, which is D34 §1's
//! actual demand — *"a capability IS an observation: 'NET_RAW absent' is a fact dated by the
//! source"*, and *"the fixture replays it for free — one JSONL line reproduces a mid-scan NET_RAW
//! loss, zero mocks"*.
//!
//! The constructor's `capabilities` is now the **initial** descriptor — the one in force before any
//! record — not the descriptor for the whole stream.
//!
//! **An UPGRADE is legal, and so is an empty `kinds` set.** A source can regain NET_RAW (FR5 treats
//! capability as an axis that moves both ways), and a source capable of nothing is still `Live`:
//! under FR19 that suppresses divergences on every field while leaving liveness untouched. Neither
//! is a bug; both have a test.
//!
//! **Containment is POSITIONAL, and that supersedes story 4.4's global rule rather than dropping
//! it.** 4.4 required that the file never exceed the constructor's declaration. Once the descriptor
//! comes from the file, the file IS the authority (D34 §1: *"the connector is no longer the
//! authority — the poll is"*), so each observation is instead checked against the descriptor in
//! force AT ITS OWN POSITION. That is stronger where it counts — emitting what you just declared
//! yourself blind to becomes impossible, which the global check could not express — and weaker only
//! where the authority moved. Evaluating every observation against the FINAL descriptor would be
//! the D13 bug D34 §1 names: *"the past would change status."*
//!
//! **Deriving `capabilities` from the observations would still be worse than any gap.** A capability
//! read off what was seen cannot express *"capable of hostnames, saw none"* — the one distinction
//! the descriptor exists for, and the one that stops *"no diverging uplink → same switch → merge"*
//! from merging happily. Nothing here derives anything.
//!
//! Still NOT in the file: `scopes_covered`, which stays constructor-supplied and is assigned to no
//! story. And `poll` returns a [`PollSummary`] only on `Ok`, so a poll that degraded and then failed
//! reports no descriptor at all — recorded, not fixed (`deferred-work.md`).
//!
//! Wired into the running app in a later story; the contract and these tests prove it meanwhile.
#![allow(dead_code)]

use std::collections::HashSet;

use opencmdb_core::connector::{Connector, ConnectorError, ObservationSink, PollSummary};
use opencmdb_core::observation::{Capabilities, ConnectorId, Observation, Scope, Timestamp};

/// Name the descriptor that denied a fact kind, for a message that cannot use a line number
/// (story 4.2) and has no `obs_id` to use either — a capability record carries neither.
fn describe(in_force: &Capabilities, initial: &Capabilities) -> String {
    if std::ptr::eq(in_force, initial) {
        "the initial descriptor".to_string()
    } else {
        // RFC 3339, the format the file itself uses: naming a descriptor in a different
        // spelling than the author typed sends them looking for a record they cannot grep for.
        format!("the descriptor dated {}", in_force.as_of.to_rfc3339())
    }
}
use tokio_util::sync::CancellationToken;

use crate::fixtures::{FixtureError, Record, fixture_path, read_records};

/// Replays a stream of records as a [`Connector`]. `Debug` because the load invariants are
/// tested through `Result::expect_err`, which requires it on the `Ok` type.
#[derive(Debug)]
pub struct FixtureConnector {
    id: ConnectorId,
    capabilities: Capabilities,
    scopes_covered: Vec<Scope>,
    records: Vec<Record>,
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
        let records = read_records(&fixture_path(relative_path)?)?;
        Self::from_records(id, capabilities, scopes_covered, relative_path, records)
    }

    /// Build from observations already in memory — the shape every caller had before story 4.5a,
    /// kept unchanged so no existing call site moves. A stream built this way scripts no outcome.
    pub fn from_observations(
        id: ConnectorId,
        capabilities: Capabilities,
        scopes_covered: Vec<Scope>,
        origin: &str,
        observations: Vec<Observation>,
    ) -> Result<Self, FixtureError> {
        Self::from_records(
            id,
            capabilities,
            scopes_covered,
            origin,
            observations.into_iter().map(Record::Observation).collect(),
        )
    }

    /// Build from records already in memory. `origin` labels them in any error message — the
    /// corpus-relative path for [`Self::load`], a caller-chosen label otherwise.
    ///
    /// Every load invariant is checked HERE, so the two paths cannot diverge in what they admit
    /// **except for one rule, deliberately** — see the paragraph below on terminal failures. They
    /// also differ in what they REPORT for two violations, because
    /// a `Vec` has no line numbers to name: a repeated `obs_id` is
    /// [`FixtureError::DuplicateObservationId`] from a file and
    /// [`FixtureError::RepeatedObservationId`] in memory; a scripted cancellation is
    /// [`FixtureError::CancellationScripted`] from a file and
    /// [`FixtureError::CancellationInStream`] in memory. A caller that wants to handle either
    /// condition must match BOTH of its variants.
    ///
    /// **One rule is deliberately NOT enforced here: "nothing may follow a terminal failure".**
    /// Its entire rationale is the committed corpus — `read_traps` cross-checks a trap's `obs_id`s
    /// against what a FILE contains, so an unreachable observation there would yield a trap that
    /// can never fire. An in-memory stream is judged by no trap file, and a caller needs to build
    /// exactly that shape to prove a faulted replay emits a strict PREFIX of the clean one
    /// (D35(a)). Enforcing it here would forbid the test that proves the story's own criterion.
    pub fn from_records(
        id: ConnectorId,
        capabilities: Capabilities,
        scopes_covered: Vec<Scope>,
        origin: &str,
        records: Vec<Record>,
    ) -> Result<Self, FixtureError> {
        // `read_records` refuses a repeated `obs_id` for a FILE, naming both lines. A `Vec` never
        // passed through it, so without this check every in-memory construction — this module's
        // tests, and the harnesses of stories 4.6 and 4.7 — would bypass the anchor the whole
        // labelling format rests on ("a trap points at an obs_id, never by line number").
        let mut seen: HashSet<uuid::Uuid> = HashSet::with_capacity(records.len());
        // The descriptor IN FORCE, walked positionally. It starts as the constructor's — the one
        // in force before any record — and each capability record replaces it from that point on.
        let mut in_force = &capabilities;
        // The latest observation seen so far, for AC2's "a descriptor cannot be dated before facts
        // collected under it". Tracked as a MAX, not as the previous line: a stream is not required
        // to be sorted by `observed_at`, and the rule is about every preceding observation.
        let mut latest_observation: Option<(Timestamp, String)> = None;
        let mut previous_as_of: Option<Timestamp> = None;
        for record in &records {
            let observation = match record {
                Record::Observation(observation) => observation,
                Record::Failure(error) => {
                    // Cancellation comes from the token, never from the data. It is the ONE
                    // variant that leaves liveness unchanged, so a stream able to mint it could
                    // assert that nothing was blinded when nothing cancelled anything.
                    if *error == ConnectorError::Cancelled {
                        return Err(FixtureError::CancellationInStream {
                            origin: origin.to_string(),
                        });
                    }
                    continue;
                }
                Record::Capability(next) => {
                    if let Some((observed_at, obs_id)) = &latest_observation
                        && next.as_of < *observed_at
                    {
                        return Err(FixtureError::CapabilityPredatesObservation {
                            origin: origin.to_string(),
                            as_of: next.as_of,
                            observed_at: *observed_at,
                            obs_id: obs_id.clone(),
                        });
                    }
                    if let Some(previous) = previous_as_of
                        && next.as_of < previous
                    {
                        return Err(FixtureError::CapabilityOutOfOrder {
                            origin: origin.to_string(),
                            as_of: next.as_of,
                            previous_as_of: previous,
                        });
                    }
                    previous_as_of = Some(next.as_of);
                    // An UPGRADE is legal, and so is an empty `kinds` set — see the module doc.
                    // Nothing here compares the new descriptor to the old one.
                    in_force = next;
                    continue;
                }
            };
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

            // POSITIONAL: checked against the descriptor in force HERE, not against one set for
            // the whole file. D34 §1's own doctrine — evaluate against the batch's descriptor, or
            // "an observation ingested when NET_RAW existed would be re-evaluated later with a
            // ping-only descriptor and become retroactively SourceNotCapable. The past would
            // change status."
            for fact in &observation.facts {
                let kind = fact.kind();
                if !in_force.can_emit(kind) {
                    return Err(FixtureError::UndeclaredFactKind {
                        origin: origin.to_string(),
                        kind,
                        obs_id: obs_id.to_string(),
                        descriptor: describe(in_force, &capabilities),
                    });
                }
            }

            if latest_observation
                .as_ref()
                .is_none_or(|(at, _)| observation.observed_at > *at)
            {
                latest_observation = Some((observation.observed_at, obs_id.to_string()));
            }
        }

        Ok(Self {
            id,
            capabilities,
            scopes_covered,
            records,
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
        // Recomputed EVERY poll, never written back onto `self`: an implementation that persisted
        // the reduced descriptor would pass the first poll and start the second already degraded.
        // A replay is idempotent, and the trap runner polls more than once.
        let mut in_force = &self.capabilities;
        for record in &self.records {
            // The cancellation point is BETWEEN records — never mid-observation. Everything
            // already emitted is true and survives (D34 §2). Nothing is consumed, so a second poll
            // replays the same stream.
            //
            // The token is checked BEFORE the record is examined, so a pre-cancelled poll over a
            // stream scripting a failure reports `Cancelled`, not the scripted error. That is the
            // safe direction and it is deliberate: cancellation is the runtime's business and
            // leaves liveness UNCHANGED, while every scriptable variant blinds the source
            // (`is_blinding`). Reporting the file's error for a poll that was shut down would
            // blind a source because the process was stopping.
            if cancel.is_cancelled() {
                return Err(ConnectorError::Cancelled);
            }
            match record {
                Record::Observation(observation) => sink.emit(observation.clone()),
                // The poll ends HERE. Everything already emitted stays in the sink: those
                // observations are true, and they do not expire because the poll was cut (D34 §2).
                Record::Failure(error) => return Err(error.clone()),
                // The source's capability changed and the poll CONTINUES. A source that lost
                // NET_RAW is still `Live` (D33) — reporting an error here would blind it.
                Record::Capability(next) => in_force = next,
            }
        }
        // No check AFTER the loop, deliberately: at this point every observation was emitted and
        // the poll did its whole job. D34 says a cancelled poll KEEPS what it emitted — not that a
        // COMPLETED one must report cancellation. So a token cancelled during the final emit still
        // yields `Ok`. The asymmetry is a decision (story 4.4 review, 2026-07-22), not an oversight.
        Ok(PollSummary {
            capabilities: in_force.clone(),
            scopes_covered: self.scopes_covered.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{Record, read_jsonl, read_records};
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

    // ── The scripted failure (story 4.5a) ────────────────────────────────────

    const PARTIAL: &str = "scenario/replay/partial-then-failed.jsonl";

    fn partial_id() -> ConnectorId {
        ConnectorId::from_uuid(u("44444444-4444-4444-8444-444444444444"))
    }

    fn partial_scope() -> Scope {
        Scope {
            l2_domain: L2DomainId::from_uuid(u("55555555-5555-4555-8555-555555555555")),
            vantage: VantageId::from_uuid(u("66666666-6666-4666-8666-666666666666")),
        }
    }

    fn partial_caps() -> Capabilities {
        Capabilities {
            as_of: ts("2026-02-01T00:00:20Z"),
            kinds: BTreeSet::from([
                FactKind::Mac,
                FactKind::IpV4,
                FactKind::Rtt,
                FactKind::Hostname,
            ]),
        }
    }

    /// The committed stream's observations, read from the FILE. Nothing here restates its bytes.
    fn partial_observations() -> Vec<Observation> {
        read_jsonl(&fixture_path(PARTIAL).unwrap()).expect("the committed stream reads")
    }

    fn partial_connector_from(records: Vec<Record>) -> FixtureConnector {
        FixtureConnector::from_records(
            partial_id(),
            partial_caps(),
            vec![partial_scope()],
            PARTIAL,
            records,
        )
        .expect("the records must load")
    }

    async fn drain(connector: &mut FixtureConnector) -> (Vec<Observation>, Option<ConnectorError>) {
        let mut sink = VecSink::default();
        let outcome = connector
            .poll(
                ts("2000-01-01T00:00:00Z"),
                &mut sink,
                CancellationToken::new(),
            )
            .await;
        (sink.observations, outcome.err())
    }

    /// AC3: the poll ends with the scripted variant, and everything emitted before it survives —
    /// those observations are true and do not expire because the poll was cut (D34 §2).
    #[tokio::test]
    async fn a_scripted_failure_ends_the_poll_and_keeps_what_was_emitted() {
        let mut connector =
            FixtureConnector::load(partial_id(), partial_caps(), vec![partial_scope()], PARTIAL)
                .expect("the committed stream must load");
        let (emitted, error) = drain(&mut connector).await;

        match error.expect("the scripted failure must surface") {
            ConnectorError::Unreachable { detail } => assert!(
                detail.contains("mid-sweep"),
                "the payload comes from the FILE: {detail}"
            ),
            other => panic!("expected the scripted Unreachable, got {other:?}"),
        }
        // Not rolled back: the four observations preceding the record are in the sink.
        assert_eq!(emitted.len(), 4);
        // Both sides come from the same reader, so this proves `poll` neither drops nor reorders
        // what it loaded — NOT that the reader parsed the file correctly, which
        // `fixtures.rs`'s own byte-for-byte test is what pins.
        assert_eq!(
            emitted,
            read_jsonl(&fixture_path(PARTIAL).unwrap()).expect("the stream reads"),
            "`poll` emits what was loaded, in file order"
        );
        // The obs_ids are stated independently of the reader, so a reader that silently reordered
        // or dropped a line would be caught here rather than agreeing with itself.
        let ids: Vec<String> = emitted.iter().map(|o| o.obs_id.to_string()).collect();
        assert_eq!(
            ids,
            vec![
                "bbbbbbbb-0000-4000-8000-000000000001",
                "bbbbbbbb-0000-4000-8000-000000000002",
                "bbbbbbbb-0000-4000-8000-000000000003",
                "bbbbbbbb-0000-4000-8000-000000000004",
            ]
        );
    }

    /// AC7 / D35(a) / NFR8(a): **a fault may only REMOVE knowledge, never ADD an assertion.**
    ///
    /// The assertion is a STRICT prefix, and that is the whole point. With the failure record
    /// last — the only shape a committed stream may have — the faulted run emits exactly what the
    /// clean run emits, `starts_with` holds trivially, and a `poll` that ignored the record
    /// entirely would pass. So the record is moved INTO the middle here, which
    /// `from_records` permits precisely so this proof can exist.
    #[tokio::test]
    async fn a_faulted_replay_emits_a_strict_prefix_of_the_clean_one() {
        let committed = read_records(&fixture_path(PARTIAL).unwrap()).expect("the stream reads");
        let observations: Vec<Observation> = committed
            .iter()
            .filter_map(|r| r.as_observation().cloned())
            .collect();
        // Matched explicitly, NOT as "the record that is not an observation": story 4.5b adds a
        // capability record, and the negative formulation would silently start injecting THAT as
        // the fault while still claiming to prove D35(a).
        let failure = committed
            .iter()
            .find(|r| matches!(r, Record::Failure(_)))
            .expect("the committed stream scripts a failure")
            .clone();

        // The clean run: the same stream with the record removed.
        let clean_records: Vec<Record> = observations
            .iter()
            .cloned()
            .map(Record::Observation)
            .collect();
        let (clean, clean_error) = drain(&mut partial_connector_from(clean_records)).await;
        assert!(clean_error.is_none(), "the clean run completes");
        assert_eq!(clean.len(), 4);

        // The faulted run: the same observations, with the fault injected in the MIDDLE.
        let mut faulted_records: Vec<Record> = Vec::new();
        faulted_records.push(Record::Observation(observations[0].clone()));
        faulted_records.push(Record::Observation(observations[1].clone()));
        faulted_records.push(failure);
        faulted_records.extend(observations[2..].iter().cloned().map(Record::Observation));
        let (faulted, faulted_error) = drain(&mut partial_connector_from(faulted_records)).await;

        assert!(faulted_error.is_some(), "the faulted run fails");
        assert!(
            clean.starts_with(&faulted),
            "a fault may only remove knowledge, never add an assertion"
        );
        assert!(
            faulted.len() < clean.len(),
            "the prefix must be STRICT ({} vs {}), or this assertion cannot fail",
            faulted.len(),
            clean.len()
        );
    }

    /// A poll whose FIRST record is a failure emits NOTHING and fails.
    ///
    /// epics.md names three outcomes — "clean, **failed**, or partial-then-failed" — and this is
    /// the plain "failed" one. It is the only shape where the emit loop returns before `sink.emit`
    /// is ever called, which is why it needs its own test rather than being assumed covered by the
    /// partial case.
    #[tokio::test]
    async fn a_failure_as_the_first_record_emits_nothing_and_fails() {
        let mut connector = partial_connector_from(vec![
            Record::Failure(ConnectorError::Timeout),
            Record::Observation(partial_observations()[0].clone()),
        ]);
        let (emitted, error) = drain(&mut connector).await;
        assert_eq!(error, Some(ConnectorError::Timeout));
        assert!(
            emitted.is_empty(),
            "nothing precedes the failure, so nothing is emitted"
        );
        // And it still blinds — only `Cancelled` does not (NFR7).
        assert!(error.expect("an error").is_blinding());
    }

    /// AC9: cancellation is the runtime's business and leaves liveness UNCHANGED, while every
    /// scriptable variant blinds the source. So the token wins over the file — reporting the
    /// scripted error would blind a source because the process was shutting down.
    #[tokio::test]
    async fn a_pre_cancelled_poll_reports_cancellation_not_the_scripted_failure() {
        let mut connector =
            FixtureConnector::load(partial_id(), partial_caps(), vec![partial_scope()], PARTIAL)
                .expect("the committed stream must load");
        let mut sink = VecSink::default();
        let token = CancellationToken::new();
        token.cancel();
        let err = connector
            .poll(ts("2000-01-01T00:00:00Z"), &mut sink, token)
            .await
            .expect_err("a pre-cancelled poll must fail");
        assert_eq!(err, ConnectorError::Cancelled);
        assert!(!err.is_blinding(), "Cancelled leaves liveness unchanged");
        assert!(sink.observations.is_empty());
    }

    /// The in-memory mirror of the reader's refusal. Without it, every construction that does not
    /// go through a file — this module's tests, and the harnesses of 4.6 and 4.7 — could mint the
    /// one error that asserts nothing was blinded.
    #[test]
    fn an_in_memory_stream_may_not_script_cancellation() {
        let err = FixtureConnector::from_records(
            partial_id(),
            partial_caps(),
            vec![partial_scope()],
            "<in-memory>",
            // The offender is SECOND, behind a valid observation: with a one-element vector a
            // `from_records` inspecting only `.first()` would pass this test untouched.
            vec![
                Record::Observation(partial_observations()[0].clone()),
                Record::Failure(ConnectorError::Cancelled),
            ],
        )
        .expect_err("a scripted cancellation must be refused");
        match &err {
            FixtureError::CancellationInStream { origin } => assert_eq!(origin, "<in-memory>"),
            other => panic!("expected CancellationInStream, got {other:?}"),
        }
        assert!(err.to_string().contains("token"), "{err}");
        // A non-cancellation failure is admissible, so the check is about `Cancelled` and not
        // about failures in general.
        FixtureConnector::from_records(
            partial_id(),
            partial_caps(),
            vec![partial_scope()],
            "<in-memory>",
            vec![
                Record::Observation(partial_observations()[0].clone()),
                Record::Failure(ConnectorError::Timeout),
            ],
        )
        .expect("a Timeout is scriptable");
    }

    // ── The capability record (story 4.5b) ───────────────────────────────────

    const DOWNGRADE: &str = "scenario/replay/capability-downgrade.jsonl";

    fn downgrade_id() -> ConnectorId {
        ConnectorId::from_uuid(u("77777777-7777-4777-8777-777777777777"))
    }

    fn downgrade_scope() -> Scope {
        Scope {
            l2_domain: L2DomainId::from_uuid(u("88888888-8888-4888-8888-888888888888")),
            vantage: VantageId::from_uuid(u("99999999-9999-4999-8999-999999999999")),
        }
    }

    /// The INITIAL descriptor — the one in force before any record. It declares `Rtt`, which the
    /// committed capability record later drops.
    fn downgrade_initial_caps() -> Capabilities {
        Capabilities {
            as_of: ts("2026-03-01T00:00:00Z"),
            kinds: BTreeSet::from([
                FactKind::Mac,
                FactKind::IpV4,
                FactKind::Rtt,
                FactKind::Hostname,
            ]),
        }
    }

    fn downgrade_connector() -> FixtureConnector {
        FixtureConnector::load(
            downgrade_id(),
            downgrade_initial_caps(),
            vec![downgrade_scope()],
            DOWNGRADE,
        )
        .expect("the committed downgrade stream must load")
    }

    fn downgrade_records() -> Vec<Record> {
        read_records(&fixture_path(DOWNGRADE).unwrap()).expect("the stream reads")
    }

    fn downgrade_from(records: Vec<Record>) -> Result<FixtureConnector, FixtureError> {
        FixtureConnector::from_records(
            downgrade_id(),
            downgrade_initial_caps(),
            vec![downgrade_scope()],
            "<in-memory>",
            records,
        )
    }

    /// AC1/AC3: one line changes the descriptor, the poll CONTINUES, and the summary reports the
    /// descriptor in force at the end — read from the FILE, not from the constructor.
    #[tokio::test]
    async fn a_capability_record_changes_the_descriptor_and_the_poll_continues() {
        let mut connector = downgrade_connector();
        let mut sink = VecSink::default();
        let summary = connector
            .poll(
                ts("2000-01-01T00:00:00Z"),
                &mut sink,
                CancellationToken::new(),
            )
            .await
            .expect("a capability change is not an error — the source is still Live (D33)");

        assert_eq!(sink.observations.len(), 4, "every observation is replayed");
        // The descriptor comes from the FILE: its date and its kinds are the record's, not the
        // constructor's. This is what closes 4.4's "as_of is unrelated to the file" finding.
        assert_eq!(summary.capabilities.as_of, ts("2026-03-01T00:00:07Z"));
        assert_ne!(summary.capabilities, downgrade_initial_caps());
        assert!(
            !summary.capabilities.can_emit(FactKind::Rtt),
            "the record dropped Rtt"
        );
        assert!(summary.capabilities.can_emit(FactKind::Hostname));
    }

    /// AC4, the LEGAL direction — and the one a careless implementation breaks.
    ///
    /// The committed stream emits `Rtt` BEFORE the record that drops it. An implementation that
    /// checked every observation against the FINAL descriptor would refuse this stream at load,
    /// retroactively invalidating facts that were legitimate when they were collected — D34 §1's
    /// "the past would change status". That this fixture loads at all is the assertion.
    #[test]
    fn a_fact_emitted_before_the_descriptor_dropped_it_stays_legal() {
        let connector = downgrade_connector();
        drop(connector);
        // Stated explicitly so the reason this test exists cannot be mistaken for a smoke test.
        let records = downgrade_records();
        let rtt_before = records.iter().any(|r| {
            r.as_observation()
                .is_some_and(|o| o.facts.iter().any(|f| f.kind() == FactKind::Rtt))
        });
        assert!(rtt_before, "the fixture must emit Rtt before the downgrade");
    }

    /// AC4, the REFUSED direction: the same fact kind, after the record that denied it.
    ///
    /// The offending observation is placed SECOND among the post-record observations, behind a
    /// valid one, so an implementation validating only the first record after a descriptor change
    /// would still be caught.
    #[test]
    fn a_fact_emitted_after_the_descriptor_dropped_it_is_refused() {
        let records = downgrade_records();
        let mut mutated = records.clone();
        // Give the LAST observation an Rtt fact — it sits after the capability record that
        // dropped Rtt, and behind another post-record observation.
        let last = mutated.len() - 1;
        match &mut mutated[last] {
            Record::Observation(observation) => observation.facts.push(Fact::Rtt { millis: 3 }),
            other => panic!("expected the last record to be an observation, got {other:?}"),
        }
        let err = downgrade_from(mutated).expect_err("a denied fact kind must be refused");
        match &err {
            FixtureError::UndeclaredFactKind {
                kind,
                obs_id,
                descriptor,
                ..
            } => {
                assert_eq!(*kind, FactKind::Rtt);
                assert!(obs_id.ends_with("004"), "names the observation: {obs_id}");
                assert!(
                    descriptor.contains("2026-03-01T00:00:07"),
                    "names the DENYING descriptor by its as_of: {descriptor}"
                );
            }
            other => panic!("expected UndeclaredFactKind, got {other:?}"),
        }
    }

    /// When no record is yet in force, the message must say so rather than naming an `as_of` that
    /// no record authored (AC5).
    #[test]
    fn the_initial_descriptor_is_named_as_such() {
        let mut records = downgrade_records();
        records.truncate(1);
        match &mut records[0] {
            Record::Observation(observation) => observation.facts.push(Fact::Uplink {
                peer_mac: MacAddr([0x02, 0x00, 0x5e, 0x00, 0x55, 0x01]),
                peer_port: "doc-port-1".into(),
            }),
            other => panic!("expected an observation, got {other:?}"),
        }
        let err = downgrade_from(records).expect_err("Uplink is not in the initial descriptor");
        match &err {
            FixtureError::UndeclaredFactKind { descriptor, .. } => assert_eq!(
                descriptor, "the initial descriptor",
                "no record is in force yet, so no as_of may be named"
            ),
            other => panic!("expected UndeclaredFactKind, got {other:?}"),
        }
    }

    /// AC2, rule 1: a descriptor cannot be dated before facts collected under it.
    #[test]
    fn a_capability_record_predating_a_preceding_observation_is_refused() {
        let mut records = downgrade_records();
        // The record sits at index 2, after two observations dated :00 and :05.
        match &mut records[2] {
            Record::Capability(capabilities) => {
                capabilities.as_of = ts("2026-03-01T00:00:01Z");
            }
            other => panic!("expected the capability record at index 2, got {other:?}"),
        }
        let err = downgrade_from(records).expect_err("a backdated descriptor must be refused");
        match &err {
            FixtureError::CapabilityPredatesObservation {
                as_of,
                observed_at,
                obs_id,
                ..
            } => {
                assert_eq!(*as_of, ts("2026-03-01T00:00:01Z"));
                // The MAX preceding observation, not merely the previous line.
                assert_eq!(*observed_at, ts("2026-03-01T00:00:05Z"));
                assert!(obs_id.ends_with("002"), "{obs_id}");
            }
            other => panic!("expected CapabilityPredatesObservation, got {other:?}"),
        }
    }

    /// AC2, rule 2: the descriptor's history is a timeline and may not go backwards.
    #[test]
    fn capability_records_going_backwards_in_time_are_refused() {
        let mut records = downgrade_records();
        // INSERTED directly after the committed record (index 2), not appended: the two AC2 rules
        // interact, and appending would put this record after an observation dated :15, where the
        // "predates an observation" rule fires first and this one is never reached. Measured — the
        // first version of this test got `CapabilityPredatesObservation` instead.
        records.insert(
            3,
            Record::Capability(Capabilities {
                as_of: ts("2026-03-01T00:00:06Z"),
                kinds: BTreeSet::from([FactKind::Mac, FactKind::IpV4, FactKind::Hostname]),
            }),
        );
        let err = downgrade_from(records).expect_err("a backwards timeline must be refused");
        match &err {
            FixtureError::CapabilityOutOfOrder {
                as_of,
                previous_as_of,
                ..
            } => {
                assert_eq!(*as_of, ts("2026-03-01T00:00:06Z"));
                assert_eq!(*previous_as_of, ts("2026-03-01T00:00:07Z"));
            }
            other => panic!("expected CapabilityOutOfOrder, got {other:?}"),
        }
        assert!(err.to_string().contains("backwards"), "{err}");
    }

    /// AC6: an UPGRADE is legal — a source can regain what it lost — and so is an empty `kinds`
    /// set, which is a source capable of nothing while still `Live`.
    #[test]
    fn an_upgrade_and_an_empty_descriptor_are_both_legal() {
        let mut upgraded = downgrade_records();
        upgraded.push(Record::Capability(Capabilities {
            as_of: ts("2026-03-01T00:00:20Z"),
            // Wider than the constructor's — the file is the authority now (D34 §1).
            kinds: BTreeSet::from([
                FactKind::Mac,
                FactKind::IpV4,
                FactKind::Rtt,
                FactKind::Uplink,
            ]),
        }));
        downgrade_from(upgraded).expect("an upgrade is legal");

        let mut emptied = downgrade_records();
        emptied.push(Record::Capability(Capabilities {
            as_of: ts("2026-03-01T00:00:20Z"),
            kinds: BTreeSet::new(),
        }));
        downgrade_from(emptied).expect("a source capable of nothing is a state, not an error");
    }

    /// AC7: the descriptor is recomputed per poll and never written back onto `self`.
    ///
    /// The trap: an implementation that persisted the reduced descriptor passes the FIRST poll and
    /// starts the second already degraded. `polling_twice_replays_the_same_stream` never looks at
    /// the summary, so this needs its own assertion.
    #[tokio::test]
    async fn polling_twice_yields_the_same_final_descriptor() {
        let mut connector = downgrade_connector();
        let mut first_sink = VecSink::default();
        let first = connector
            .poll(
                ts("2000-01-01T00:00:00Z"),
                &mut first_sink,
                CancellationToken::new(),
            )
            .await
            .expect("first poll");
        let mut second_sink = VecSink::default();
        let second = connector
            .poll(
                ts("2000-01-01T00:00:00Z"),
                &mut second_sink,
                CancellationToken::new(),
            )
            .await
            .expect("second poll");

        assert_eq!(first.capabilities, second.capabilities);
        assert_eq!(first_sink.observations, second_sink.observations);
        assert!(!second.capabilities.can_emit(FactKind::Rtt));
        // And the connector itself was not mutated: its initial descriptor still declares Rtt.
        assert!(connector.capabilities.can_emit(FactKind::Rtt));
    }

    /// AC8: a stream may degrade AND then fail — and the descriptor dies with the `Err`.
    ///
    /// This is the concrete instance of the open contradiction recorded for story 4.6: `poll`
    /// returns `PollSummary` only on `Ok`, so a poll that degraded and then failed reports no
    /// descriptor at all, while 4.6 needs a `capability_snapshot` on every scored record.
    #[tokio::test]
    async fn a_degraded_then_failed_poll_reports_no_descriptor_at_all() {
        let mut records = downgrade_records();
        records.push(Record::Failure(ConnectorError::Unreachable {
            detail: "the documentation subnet stopped answering".into(),
        }));
        let mut connector = downgrade_from(records).expect("degrade-then-fail is admissible");
        let mut sink = VecSink::default();
        let outcome = connector
            .poll(
                ts("2000-01-01T00:00:00Z"),
                &mut sink,
                CancellationToken::new(),
            )
            .await;

        assert!(matches!(outcome, Err(ConnectorError::Unreachable { .. })));
        assert_eq!(
            sink.observations.len(),
            4,
            "what was emitted is kept (D34 §2)"
        );
        // `outcome` is an `Err`, so there is NO `PollSummary` and therefore no capability snapshot
        // — the descriptor the stream established is unreachable from the trait's return type.
        assert!(outcome.is_err());
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
