//! The seam between a scan and the identity pass — poll, ingest, resolve.
//!
//! Story 5.14. Until it, `resolver::resolve` had no production caller at all: the engine had been
//! writing identity links since story 5.9b, in tests only, and `main.rs` had never called it. So a
//! reach counter would have read ZERO in the shipped product, permanently and greenly — decoration
//! in D18's sense.
//!
//! # 🔑 Why this is a SEAM and not three lines inside `spawn_startup_scan`
//!
//! `spawn_startup_scan` is a `std::thread::spawn` whose `JoinHandle` is dropped, and its body is
//! inseparable from a live ICMP poll. **Nothing can ever assert on what happens inside it.** The
//! story's first arbitration on this said "the helper is the seam, and the last link is carried by
//! nothing" — true, and incomplete: its validation then measured that a seam GENERIC over
//! [`Connector`] lets a test drive poll → ingest → resolve with the already-committed
//! `FixtureConnector`, at which point the uncarried region shrinks from *the whole wiring* to
//! **three lines** — build the connector, connect the pool, call this function.
//!
//! 🔑 *Recording an unavoidable GREEN is honest; recording it without measuring how much it covers
//! is not.* The three lines are still carried by nothing, and that is stated rather than implied.
//!
//! # 🔴 Two transaction units, and the pass sees only what LANDED
//!
//! The ingest writes one transaction PER OBSERVATION (it always did — FR11 makes an observation
//! immutable and independently true, and D34 §2 says everything emitted before a failure is still
//! true). The pass runs in its own, second unit.
//!
//! And it is handed **only the observations that were actually written**, which is not a nicety:
//! `identity_link.observation_id` is a foreign key onto `observation_record`
//! (`0003_resolver_guards.sql`), so handing the pass an observation the database refused fails the
//! WHOLE pass and costs **every other observation its link**. Measured before this filter existed:
//! one refused observation beside one good one gave `resolution = None` and zero links.
//!
//! # ⚠️ What this seam does NOT make true
//!
//! The only connector `main.rs` reaches emits no MAC, ever, so every scanned observation abstains
//! (see `arp_ping`'s pins). And nothing supersedes an abstention across scans, so the population
//! accumulates. Both are pinned by tests here and in `resolver`; neither is fixed here. See
//! [`counted_current_engine_links`] and the accumulation test below.

use opencmdb_core::connector::{Connector, VecSink};
use opencmdb_core::observation::{Observation, Timestamp};
use opencmdb_core::repo::WriteRepository;
use sqlx::MySqlPool;
use tokio_util::sync::CancellationToken;

use crate::repo::{MariaRepository, classify, insert_observation};
use crate::resolver::{Resolution, resolve};

/// What one scan-and-resolve pass did.
#[derive(Debug)]
pub(crate) struct ScanOutcome {
    /// Observations the database accepted.
    pub(crate) ingested: usize,
    /// Observations the database refused. Each is logged; none is fatal.
    pub(crate) failed: usize,
    /// The identity pass's own outcome, or `None` if the poll failed, nothing landed, or the pass
    /// was refused. A `None` here is always accompanied by a log line naming why.
    pub(crate) resolution: Option<Resolution>,
}

/// Poll `connector`, ingest what it emits, and run the identity pass over what landed.
///
/// Best-effort throughout, exactly as the startup scan has always been: a poll failure, an ingest
/// failure and a refused pass are each logged and none is fatal. The page still serves whatever is
/// already persisted.
///
/// # Errors
///
/// None — every failure is reported through [`ScanOutcome`] and the log. A caller that needs to
/// distinguish them reads `ingested`, `failed` and `resolution`.
pub(crate) async fn poll_ingest_resolve<C: Connector>(
    connector: &mut C,
    now: Timestamp,
    pool: &MySqlPool,
) -> ScanOutcome {
    let mut sink = VecSink::default();
    if let Err(error) = connector
        .poll(now, &mut sink, CancellationToken::new())
        .await
    {
        tracing::warn!(?error, "scan failed");
        return ScanOutcome {
            ingested: 0,
            failed: 0,
            resolution: None,
        };
    }

    let repo = MariaRepository::new(pool.clone());
    let mut landed: Vec<Observation> = Vec::new();
    let mut failed = 0usize;
    for observation in sink.observations {
        let stored = observation.clone();
        let result = repo
            .transact(move |unit| {
                let observation = observation.clone();
                Box::pin(async move {
                    insert_observation(unit.executor(), &observation)
                        .await
                        .map_err(classify)
                })
            })
            .await;
        match result {
            Ok(()) => landed.push(stored),
            Err(error) => {
                failed += 1;
                tracing::warn!(?error, "ingesting a scanned observation failed");
            }
        }
    }

    let ingested = landed.len();
    if landed.is_empty() {
        return ScanOutcome {
            ingested,
            failed,
            resolution: None,
        };
    }

    // The SECOND unit. A refused pass must not reach back and undo the ingest.
    let resolution = match repo
        .transact(move |unit| {
            let landed = landed.clone();
            Box::pin(async move { resolve(unit.executor(), &landed).await })
        })
        .await
    {
        Ok(resolution) => Some(resolution),
        Err(error) => {
            // ⚠️ `error!`, and the refusal NAMED. `InstantRegressed` and
            // `ContradictoryObservation` are the two the resolver can raise; a silent skip would
            // make any downstream count lie by omission.
            tracing::error!(?error, "the identity pass was refused");
            None
        }
    };

    ScanOutcome {
        ingested,
        failed,
        resolution,
    }
}

/// How many CURRENT links the ENGINE holds.
///
/// # 🔴 Why this read exists at all
///
/// [`Resolution`] is per-PASS: after two passes it reports `links_written: 1`, never *"two current
/// links"*. The accumulation this story pins is a property of the STORE across passes, so it is
/// visible only through the database. An earlier draft of the story claimed it added no read and
/// was contradicted by its own acceptance criterion.
///
/// # ⚠️ Both predicates are load-bearing, and each is carried by a row a test creates
///
/// Measured on this story's validation: dropping `decided_by = 'ENGINE'`, dropping
/// `current_subject IS NOT NULL`, and dropping both each left the whole suite GREEN — the fifth
/// recurrence of that family in this project. A `WHERE` no test can red is decoration, so the tests
/// below plant an OPERATOR row and a SUPERSEDED row and assert this function excludes each.
///
/// `count_identity_links` cannot serve here: it is an unfiltered `SELECT COUNT(*)`, so it would
/// agree with this one only by accident and diverge the first time a link is superseded.
///
/// # Errors
///
/// Propagates the database error.
pub(crate) async fn counted_current_engine_links(pool: &MySqlPool) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(sqlx::AssertSqlSafe(
        "SELECT COUNT(*) FROM identity_link \
         WHERE decided_by = 'ENGINE' AND current_subject IS NOT NULL",
    ))
    .fetch_one(pool)
    .await?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture_connector::FixtureConnector;
    use opencmdb_core::observation::{
        Capabilities, ConnectorId, Fact, FactKind, L2DomainId, MacAddr, ObsId, Scope, VantageId,
    };
    use std::collections::BTreeSet;

    fn at(secs: i64) -> Timestamp {
        chrono::DateTime::from_timestamp(secs, 0).expect("in range")
    }

    fn scope() -> Scope {
        Scope {
            l2_domain: L2DomainId::from_uuid(uuid::Uuid::from_u128(0x5140)),
            vantage: VantageId::from_uuid(uuid::Uuid::nil()),
        }
    }

    fn connector_id() -> ConnectorId {
        ConnectorId::from_uuid(uuid::Uuid::from_u128(0x5141))
    }

    /// An observation carrying an address and a round-trip time and NO MAC — the shape the shipped
    /// ARP/ping connector actually produces (see its own pins).
    fn mac_less(id: u128, secs: i64) -> Observation {
        Observation {
            obs_id: ObsId::from_uuid(uuid::Uuid::from_u128(id)),
            connector_id: connector_id(),
            observed_at: at(secs),
            scope: scope(),
            facts: vec![
                Fact::IpV4 {
                    addr: "203.0.113.7".parse().expect("a documentation address"),
                },
                Fact::Rtt { millis: 3 },
            ],
            raw: None,
        }
    }

    /// The same, with a MAC — so a pass CAN mint an interface. Used only to show the difference.
    fn with_mac(id: u128, secs: i64, last: u8) -> Observation {
        let mut o = mac_less(id, secs);
        o.facts.push(Fact::Mac {
            addr: MacAddr([0x02, 0x00, 0x5e, 0x00, 0x57, last]),
            locally_administered: true,
        });
        o
    }

    fn kinds() -> Capabilities {
        Capabilities {
            as_of: at(1_700_000_000),
            kinds: BTreeSet::from([FactKind::IpV4, FactKind::Rtt, FactKind::Mac]),
        }
    }

    fn connector(observations: Vec<Observation>) -> FixtureConnector {
        FixtureConnector::from_observations(
            connector_id(),
            kinds(),
            vec![scope()],
            "story 5.14 scan seam",
            observations,
        )
        .expect("the in-memory stream must load")
    }

    /// Connect, migrate and empty every table. Unlike `resolver`'s fixture this inserts NOTHING —
    /// the whole point of this seam is that it does the ingesting.
    async fn empty_pool() -> Option<MySqlPool> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping scan-pass test: DATABASE_URL unset");
            return None;
        };
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        for statement in [
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM interface",
            "DELETE FROM observation_record",
        ] {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }
        Some(pool)
    }

    /// **AC1** — the seam really polls, ingests and resolves, driven end to end by a test.
    ///
    /// 🔴 This is what arbitration 8 bought. Deleting the `resolve` call inside
    /// [`poll_ingest_resolve`] reds THIS test; deleting the call to `poll_ingest_resolve` inside
    /// `spawn_startup_scan` reds nothing, and the module doc says so. The difference between the two
    /// is the measured size of what remains uncarried: three lines.
    #[tokio::test]
    async fn the_seam_polls_ingests_and_resolves() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = empty_pool().await else {
            return;
        };
        let mut source = connector(vec![with_mac(1, 1_700_000_100, 1)]);

        let outcome = poll_ingest_resolve(&mut source, at(1_700_000_100), &pool).await;

        assert_eq!(outcome.ingested, 1, "the observation was written");
        assert_eq!(outcome.failed, 0, "and none was refused");
        let resolution = outcome
            .resolution
            .expect("the pass ran — a None here means it was refused, and the log names why");
        assert_eq!(
            resolution.links_written, 1,
            "the identity pass wrote its link: this is the assertion that dies if the `resolve` \
             call is removed from the seam"
        );
        assert_eq!(
            counted_current_engine_links(&pool).await.expect("count"),
            1,
            "and the store holds it"
        );
    }

    /// **AC2** — a refused ingest is bounded to its own row (arbitration 7).
    ///
    /// Before the pass was handed only what LANDED, one observation the database refused cost
    /// EVERY other observation its link: `identity_link.observation_id` is a foreign key onto
    /// `observation_record`, so the whole pass failed and `resolution` came back `None`.
    ///
    /// ⚠️ The database count is asserted FIRST on purpose. An earlier ordering died on a preceding
    /// assertion and this test's explanatory message never printed — the assertion-order family
    /// this project has now caught three times.
    #[tokio::test]
    async fn a_refused_ingest_is_bounded_to_its_own_row() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = empty_pool().await else {
            return;
        };
        // Year ~11500: outside MariaDB's `DATETIME` range, so this row is refused at the insert.
        let doomed = with_mac(2, 300_000_000_000, 2);
        let good = with_mac(3, 1_700_000_200, 3);
        let mut source = connector(vec![doomed, good]);

        let outcome = poll_ingest_resolve(&mut source, at(1_700_000_200), &pool).await;

        assert_eq!(
            counted_current_engine_links(&pool).await.expect("count"),
            1,
            "the good observation still got its link — a row the database refuses must cost only \
             itself, never the identity of the whole sweep"
        );
        assert_eq!(outcome.failed, 1, "exactly one row was refused");
        assert_eq!(outcome.ingested, 1, "exactly one landed");
        assert!(
            outcome.resolution.is_some(),
            "and the pass ran over what landed rather than failing on what did not"
        );
    }

    /// **AC4** — the structural zero, on the pass's own outcome.
    ///
    /// ⚠️ On a FIRST pass over fresh ids. `Resolution::record`'s doc records that an IDEMPOTENT
    /// pass over the same observation reports `abstentions = 0, links_unchanged = 1`, so this
    /// assertion is false on a second run and must not be written as if it held generally.
    #[tokio::test]
    async fn a_mac_less_slice_mints_no_interface_and_abstains_throughout() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = empty_pool().await else {
            return;
        };
        let slice = vec![mac_less(4, 1_700_000_300), mac_less(5, 1_700_000_301)];
        let mut source = connector(slice.clone());

        let outcome = poll_ingest_resolve(&mut source, at(1_700_000_300), &pool).await;
        let resolution = outcome.resolution.expect("the pass ran");

        assert_eq!(
            resolution.interfaces_minted, 0,
            "no MAC, no interface: `join` keys on (l2_domain, mac), and the shipped connector emits \
             neither — this is the structural zero the wiring does NOT remove"
        );
        assert_eq!(
            resolution.abstentions,
            slice.len(),
            "every observation abstained, on this FIRST pass over fresh ids"
        );
    }

    /// **AC5** — the accumulation, pinned and named as a DEFECT.
    ///
    /// # 🔴 This assertion describes a defect, not a specification
    ///
    /// Two scans of ONE unplaceable address leave TWO current abstention links, because each scan
    /// mints fresh `obs_id`s and nothing supersedes an abstention across passes. At a five-minute
    /// scan interval that one host reads ~105 000 after a year: **a counter built on this measures
    /// uptime, not reach.**
    ///
    /// ⚠️ **Do not repair this number.** The production change that would fix it — widening the
    /// vacate pass to close engine slots belonging to observations it never saw — **erases a host
    /// that missed a single scan**, which is a worse defect than accumulating. Collapsing sightings
    /// of one unplaceable thing means deciding what makes two sightings the same thing WITHOUT an
    /// identity, and that is grouping: story 5.14b and Epic 6 own it. Take the number there.
    ///
    /// The sentence is in this doc comment as well as in the assertion message deliberately: a
    /// message is read only when the test FAILS, and the reader who mistakes a pin for a
    /// specification is reading it while it passes.
    #[tokio::test]
    async fn two_scans_of_one_address_leave_two_current_links_and_that_is_the_defect() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = empty_pool().await else {
            return;
        };
        let mut first = connector(vec![mac_less(6, 1_700_000_400)]);
        poll_ingest_resolve(&mut first, at(1_700_000_400), &pool).await;
        let mut second = connector(vec![mac_less(7, 1_700_000_500)]);
        poll_ingest_resolve(&mut second, at(1_700_000_500), &pool).await;

        assert_eq!(
            counted_current_engine_links(&pool).await.expect("count"),
            2,
            "TWO current links for ONE unplaceable address, one per scan. This is a DEFECT the \
             story PINS rather than fixes — do not repair this number, take it to story 5.14b / \
             Epic 6, which own the denominator. Fixing it here by over-vacating would erase a host \
             that missed a single scan"
        );
    }

    /// **AC6** — `decided_by = 'ENGINE'` is carried by a row that would otherwise be counted.
    ///
    /// Measured on this story's validation: dropping the clause left the whole suite GREEN, because
    /// no test created an operator row. Fifth recurrence of that family in this project.
    #[tokio::test]
    async fn an_operator_link_is_not_counted() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = empty_pool().await else {
            return;
        };
        let mut source = connector(vec![with_mac(8, 1_700_000_600, 8)]);
        poll_ingest_resolve(&mut source, at(1_700_000_600), &pool).await;
        let before = counted_current_engine_links(&pool).await.expect("count");

        sqlx::query("UPDATE identity_link SET decided_by = 'OPERATOR' WHERE decided_by = 'ENGINE'")
            .execute(&pool)
            .await
            .expect("re-attribute the link to an operator");

        assert_eq!(before, 1, "the premise: the engine held one current link");
        assert_eq!(
            counted_current_engine_links(&pool).await.expect("count"),
            0,
            "and an OPERATOR row is not the engine's reach — without this clause the counter would \
             report a human's decision as the engine's"
        );
    }

    /// **AC6** — `current_subject IS NOT NULL` is carried by a superseded row.
    ///
    /// A superseded link keeps its history and drops out of the key (story 5.9's second
    /// arbitration); counting it would make the number grow with every re-scan.
    #[tokio::test]
    async fn a_superseded_link_is_not_counted() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = empty_pool().await else {
            return;
        };
        let mut source = connector(vec![with_mac(9, 1_700_000_700, 9)]);
        poll_ingest_resolve(&mut source, at(1_700_000_700), &pool).await;
        let before = counted_current_engine_links(&pool).await.expect("count");

        sqlx::query("UPDATE identity_link SET current_subject = NULL")
            .execute(&pool)
            .await
            .expect("supersede the link by hand");

        assert_eq!(before, 1, "the premise: one current link");
        assert_eq!(
            counted_current_engine_links(&pool).await.expect("count"),
            0,
            "a superseded link is history, not reach — without this clause the number would grow \
             with every re-scan even where the engine settled"
        );
    }
}
