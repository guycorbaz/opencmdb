//! The L2 pass: judge every pair of interfaces a sweep carried, and persist what the engine concluded
//! (story 6.12).
//!
//! [`crate::resolver::resolve_within`] calls [`judge`] after the L1 pass, inside the SAME transaction,
//! handing it `join`'s groups and the interface each key landed on. D13's order holds one level up:
//! the blocker ([`l2_candidates`]) is called ONCE over every key before any verdict is asked for, and
//! [`l2::decide_pair`] judges each proposed pair with the L2 rules and nothing else.
//!
//! # What reaches the table — Guy's arbitration of 2026-09-24 (story 6.12, §0.6)
//!
//! - **`NoMatch` and `Abstained { Ambiguous }` are written; `Abstained { AbsenceOfProof }` never is.**
//!   On a 46-interface sweep ~1000 pairs conclude that nothing can be said, and writing them would
//!   record silence at the cost of the transaction cap.
//! - **`Match` cannot arrive** — no L2 rule is `Decisive` — and if one ever does, the writer refuses it
//!   by name ([`crate::l2_repo::is_persisted`]) and the pass fails loudly: a match is a DEVICE, and no
//!   story has decided how one is minted without a read-then-insert window.
//! - **An unchanged network writes nothing**: a row stores the verdict vector, not the observation ids
//!   minted afresh at every sweep, so the comparison sees the same decision.
//! - **A pair is only ever judged when BOTH its interfaces are in this sweep** — the universe is built
//!   from this slice's keys — so an interface that missed one sweep keeps every decision it had (story
//!   5.14 measured the alternative erasing a host that missed a single scan). A pair judged again and
//!   now concluding `AbsenceOfProof` is CLOSED, with no successor.
//!
//! 🔑 **What this does NOT do, and says so: it never groups two interfaces into a device.** That is the
//! story's headline finding: `decide` reaches `Match` only through a `Decisive`, and no L2 rule emits
//! one. `obelix`'s two NICs answering to one name persist as ONE `Ambiguous` pair — a question for the
//! operator, which story 6.14 shows.
//!
//! # 🔴 Why an `Ambiguous` must not go through `resolver::guard_decision`
//!
//! That guard refuses an `Ambiguous` carrying no candidates, and it runs in the sweep's one
//! transaction: routed through it, `obelix` rolled back every L1 link of the sweep (measured by the
//! validation). At L2 the candidate set IS the pair, which this table always carries in two NOT NULL
//! columns — so the invariant the guard protects holds by construction here, and the guard is not called.

use std::collections::{BTreeMap, BTreeSet};

use opencmdb_core::identity::blocking::{L2CandidatePair, l2_candidates};
use opencmdb_core::identity::l1::L1Key;
use opencmdb_core::identity::l2::{self, L2Side};
use opencmdb_core::observation::{InterfaceId, ObsId, Observation, Timestamp};
use opencmdb_core::repo::RepositoryError;
use sqlx::MySqlConnection;

use crate::l2_repo::{
    close_l2_decision, insert_l2_decision, is_persisted, load_current_l2_decision,
};
use crate::repo::{DecidedBy, classify, datetime_literal};

/// What one L2 pass did, in counts a test can also read back from the database.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct L2Resolution {
    /// Pairs in the universe this pass was handed. [`judge`] supplies every pair of this sweep's keys,
    /// `n(n-1)/2` — which is what makes *"the blocker is not narrowed at its call site"* assertable
    /// (story 6.6's registered row: the corpus is blind to an uplink narrowing).
    pub candidate_pairs: usize,
    /// Versions written, first versions and replacements together.
    pub written: usize,
    /// Current versions closed because the pair now concludes differently and is still persisted.
    pub superseded: usize,
    /// Pairs whose current row already carried this decision, left alone.
    pub unchanged: usize,
    /// Current versions closed with NO successor, because the pair now concludes `AbsenceOfProof`.
    pub vacated: usize,
}

/// Judge every pair of this sweep's interfaces.
///
/// `groups` is `join`'s output for the sweep and `interfaces` the interface each key landed on; both
/// come from the L1 pass that ran just before, in the same transaction.
///
/// # Errors
///
/// Any [`RepositoryError`] a write produces, and [`RepositoryError::InstantRegressed`] when a pair is
/// re-judged at an instant EARLIER than its current version's.
pub async fn judge(
    conn: &mut MySqlConnection,
    groups: &BTreeMap<L1Key, BTreeSet<ObsId>>,
    by_id: &BTreeMap<ObsId, &Observation>,
    interfaces: &BTreeMap<L1Key, InterfaceId>,
) -> Result<L2Resolution, RepositoryError> {
    let keys: Vec<L1Key> = groups.keys().copied().collect();
    let universe = l2_candidates(&keys);
    judge_within(conn, groups, by_id, interfaces, &universe).await
}

/// Judge against a universe the caller supplies — the seam [`judge`] delegates through.
///
/// It exists so that *"only what the blocker proposed is judged"* is FALSIFIABLE: [`l2_candidates`]
/// is total, so with the universe computed internally, dropping the containment would change nothing
/// any test could see (story 5.9b's measurement, one level up).
///
/// # Errors
///
/// As [`judge`].
pub async fn judge_within(
    conn: &mut MySqlConnection,
    groups: &BTreeMap<L1Key, BTreeSet<ObsId>>,
    by_id: &BTreeMap<ObsId, &Observation>,
    interfaces: &BTreeMap<L1Key, InterfaceId>,
    universe: &BTreeSet<L2CandidatePair>,
) -> Result<L2Resolution, RepositoryError> {
    let mut summary = L2Resolution {
        candidate_pairs: universe.len(),
        ..L2Resolution::default()
    };
    for pair in universe {
        let (Some(low_group), Some(high_group)) =
            (groups.get(&pair.low()), groups.get(&pair.high()))
        else {
            // A pair the caller proposed over a key this sweep did not carry: nothing to judge it on.
            continue;
        };
        let side =
            |group: &BTreeSet<ObsId>| L2Side::new(group.iter().map(|id| by_id[id]).collect());
        let decision = l2::decide_pair(pair, &side(low_group), &side(high_group));

        let (Some(a), Some(b)) = (interfaces.get(&pair.low()), interfaces.get(&pair.high())) else {
            return Err(RepositoryError::Backend(
                "an L2 pair names a key the L1 pass placed on no interface".to_string(),
            ));
        };
        // The table orders the pair by INTERFACE id; the candidate pair is ordered by L1 key.
        let (low, high) = if a.to_string() < b.to_string() {
            (*a, *b)
        } else {
            (*b, *a)
        };
        let reached_at = latest_instant(low_group.iter().chain(high_group), by_id);
        let persisted = is_persisted(&decision)?;
        let current = load_current_l2_decision(&mut *conn, low, high)
            .await
            .map_err(classify)?;

        match current {
            None if persisted => {
                insert(conn, low, high, &decision, reached_at).await?;
                summary.written += 1;
            }
            None => {}
            Some(current) if persisted && current.carries(&decision) => {
                summary.unchanged += 1;
            }
            Some(current) => {
                if datetime_literal(reached_at) < current.valid_from {
                    return Err(RepositoryError::InstantRegressed);
                }
                close_l2_decision(&mut *conn, &current.id, reached_at).await?;
                if persisted {
                    insert(conn, low, high, &decision, reached_at).await?;
                    summary.written += 1;
                    summary.superseded += 1;
                } else {
                    summary.vacated += 1;
                }
            }
        }
    }
    Ok(summary)
}

async fn insert(
    conn: &mut MySqlConnection,
    low: InterfaceId,
    high: InterfaceId,
    decision: &opencmdb_core::identity::cascade::Decision,
    reached_at: Timestamp,
) -> Result<(), RepositoryError> {
    insert_l2_decision(
        &mut *conn,
        uuid::Uuid::now_v7(),
        low,
        high,
        decision,
        DecidedBy::Engine,
        reached_at,
    )
    .await
}

/// The latest `observed_at` among the observations a pair was judged on — the instant the decision was
/// reached, derived and never read from the clock.
///
/// # Panics
///
/// Never: both groups come from `join`, which produces no empty group.
fn latest_instant<'a>(
    ids: impl Iterator<Item = &'a ObsId>,
    by_id: &BTreeMap<ObsId, &Observation>,
) -> Timestamp {
    ids.map(|id| by_id[id].observed_at)
        .max()
        .expect("join never produces an empty group")
}

/// Tests for the L2 pass, driven through [`crate::resolver::resolve`] — the path the shipped binary
/// takes — and, for the seam, through [`judge_within`] directly.
///
/// Every count is read back from the database; the summary is asserted too, and the two agreeing is
/// itself the assertion (the resolver's idiom).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::l2_repo::{count_l2_decisions, purge_engine_l2_decisions, snapshot_l2_decisions};
    use crate::repo::{MariaRepository, classify, count_engine_reach, insert_interface};
    use crate::resolver::{Resolution, resolve};
    use opencmdb_core::identity::l1::join;
    use opencmdb_core::observation::{
        ConnectorId, Fact, HostnameSource, L2DomainId, MacAddr, Scope, VantageId,
    };
    use opencmdb_core::repo::WriteRepository;
    use sqlx::MySqlPool;

    fn domain() -> L2DomainId {
        L2DomainId::from_uuid(uuid::Uuid::from_u128(1))
    }

    fn nic(last: u8) -> MacAddr {
        MacAddr([0x00, 0x11, 0x22, 0x33, 0x44, last])
    }

    /// The IANA virtual-router address, VRID 10 — the corpus's own.
    fn vrrp() -> MacAddr {
        MacAddr([0x00, 0x00, 0x5e, 0x00, 0x01, 0x0a])
    }

    fn at(secs: i64) -> Timestamp {
        chrono::DateTime::from_timestamp(secs, 0).expect("in range")
    }

    /// One sighting of one NIC, answering to `name` when it is `Some` — the shipped connector's shape
    /// since PR #163: one MAC per observation, a PTR name when reverse DNS answers.
    fn sighting(id: u128, mac: MacAddr, name: Option<&str>, seen: i64) -> Observation {
        let mut facts = vec![Fact::Mac {
            addr: mac,
            locally_administered: false,
        }];
        if let Some(name) = name {
            facts.push(Fact::Hostname {
                name: name.to_string(),
                source: HostnameSource::Dns,
            });
        }
        Observation {
            obs_id: ObsId::from_uuid(uuid::Uuid::from_u128(id)),
            connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()),
            observed_at: at(seen),
            scope: Scope {
                l2_domain: domain(),
                vantage: VantageId::from_uuid(uuid::Uuid::nil()),
            },
            facts,
            raw: None,
        }
    }

    /// Connect, migrate and empty the identity tables. `l2_pair_decision` empties by CASCADE from
    /// `interface` — which is also what keeps the twenty other `DELETE FROM interface` cleanups working.
    async fn store() -> Option<MySqlPool> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping l2_pass test: DATABASE_URL unset");
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
            "DELETE FROM address_sighting",
        ] {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }
        Some(pool)
    }

    /// Record the observations (the identity link's foreign key needs them) and run one pass in one
    /// transaction, exactly as `scan_pass` does.
    async fn sweep(
        pool: &MySqlPool,
        observations: Vec<Observation>,
    ) -> Result<Resolution, RepositoryError> {
        for observation in &observations {
            crate::repo::insert_observation(pool, observation)
                .await
                .map_err(classify)
                .expect("insert observation");
        }
        MariaRepository::new(pool.clone())
            .transact(move |unit| {
                let observations = observations.clone();
                Box::pin(async move { resolve(unit.executor(), &observations).await })
            })
            .await
    }

    /// `(outcome, rule_id, abstention_cause, verdicts)` of every CURRENT row.
    async fn current(pool: &MySqlPool) -> Vec<(String, Option<String>, Option<String>, String)> {
        sqlx::query_as(
            "SELECT outcome, rule_id, abstention_cause, verdicts FROM l2_pair_decision \
             WHERE is_current = 1 ORDER BY outcome, verdicts",
        )
        .fetch_all(pool)
        .await
        .expect("read")
    }

    async fn links(pool: &MySqlPool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM identity_link")
            .fetch_one(pool)
            .await
            .expect("count")
    }

    /// AC4 — `obelix`: two NICs, one name. ONE `Ambiguous` pair is persisted, and the sweep's L1 links
    /// commit with it. 🔴 The second half is the one that matters: routed through
    /// `resolver::guard_decision(&[])` the same decision rolled back every L1 link of the sweep.
    #[tokio::test]
    async fn obelix_persists_one_ambiguous_pair_and_the_sweeps_l1_links_commit_with_it() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let resolution = sweep(
            &pool,
            vec![
                sighting(1, nic(8), Some("obelix.home.arpa"), 100),
                sighting(2, nic(9), Some("obelix.home.arpa"), 100),
            ],
        )
        .await
        .expect("the sweep must commit");

        assert_eq!(links(&pool).await, 2, "both L1 placements committed");
        assert_eq!(
            current(&pool).await,
            vec![(
                "abstained".to_string(),
                None,
                Some("ambiguous".to_string()),
                "l2-different-hostname=neutral;l2-hostname-agrees=supports;\
                 l2-virtual-mac-prefix=neutral"
                    .to_string(),
            )]
        );
        let pair: (String, String) =
            sqlx::query_as("SELECT interface_low, interface_high FROM l2_pair_decision")
                .fetch_one(&pool)
                .await
                .expect("the pair");
        let mut interfaces: Vec<String> = sqlx::query_scalar("SELECT id FROM interface")
            .fetch_all(&pool)
            .await
            .expect("interfaces");
        interfaces.sort();
        assert_eq!(
            vec![pair.0, pair.1],
            interfaces,
            "the pair IS the candidate set: both interfaces, ordered"
        );
        assert_eq!(resolution.l2.written, 1);
        assert_eq!(resolution.l2.candidate_pairs, 1);
    }

    /// Guy's arbitration E, on the validation's measurement: three NICs sharing one name are THREE
    /// pairs, and a per-interface key collided on them and rolled the sweep back.
    #[tokio::test]
    async fn three_nics_sharing_a_name_persist_three_pairs_without_collision() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let resolution = sweep(
            &pool,
            (1..=3)
                .map(|n| sighting(n, nic(n as u8), Some("obelix"), 100))
                .collect(),
        )
        .await
        .expect("the sweep must commit");
        assert_eq!(current(&pool).await.len(), 3);
        assert_eq!(count_l2_decisions(&pool).await.expect("count"), 3);
        assert_eq!(resolution.l2.written, 3);
        assert_eq!(links(&pool).await, 3);
    }

    /// A virtual-router address refuses every pair it is in, naming the reading — and those `NoMatch`
    /// rows sit beside `obelix`'s ambiguity without colliding with it.
    #[tokio::test]
    async fn a_virtual_router_address_persists_a_no_match_naming_the_reading() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        sweep(
            &pool,
            vec![
                sighting(1, nic(8), Some("obelix"), 100),
                sighting(2, nic(9), Some("obelix"), 100),
                sighting(3, vrrp(), Some("gw"), 100),
            ],
        )
        .await
        .expect("the sweep must commit");
        let rows = current(&pool).await;
        let no_match: Vec<_> = rows.iter().filter(|row| row.0 == "no_match").collect();
        assert_eq!(no_match.len(), 2, "the VIP against each NIC");
        assert!(
            no_match
                .iter()
                .all(|row| row.1.as_deref() == Some("l2-virtual-mac-prefix")),
            "{no_match:?}"
        );
        assert_eq!(rows.len(), 3, "and obelix's ambiguity beside them");
    }

    /// Guy's arbitration G: `AbsenceOfProof` is never persisted — two names that differ, and no name at
    /// all, write nothing. The universe is still judged in full.
    #[tokio::test]
    async fn absence_of_proof_is_judged_and_never_persisted() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let resolution = sweep(
            &pool,
            vec![
                sighting(1, nic(1), Some("asterix"), 100),
                sighting(2, nic(2), Some("obelix"), 100),
                sighting(3, nic(3), None, 100),
            ],
        )
        .await
        .expect("the sweep must commit");
        assert_eq!(resolution.l2.candidate_pairs, 3);
        assert_eq!(resolution.l2.written, 0);
        assert_eq!(count_l2_decisions(&pool).await.expect("count"), 0);
    }

    /// Guy's arbitration F: an UNCHANGED network writes nothing on its next sweep — with FRESH
    /// observation ids, which is what the shipped connector produces (`arp_ping.rs` mints one per
    /// sighting). 🔴 The premise is asserted first: the validation measured *"a second pass writes
    /// nothing"* green under a writer that never writes at all.
    #[tokio::test]
    async fn an_unchanged_network_writes_nothing_on_its_next_sweep() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let first = sweep(
            &pool,
            vec![
                sighting(1, nic(8), Some("obelix"), 100),
                sighting(2, nic(9), Some("obelix"), 100),
            ],
        )
        .await
        .expect("first sweep");
        assert_eq!(
            first.l2.written, 1,
            "premise: the first sweep wrote a decision"
        );

        let second = sweep(
            &pool,
            vec![
                sighting(11, nic(8), Some("obelix"), 400),
                sighting(12, nic(9), Some("obelix"), 400),
            ],
        )
        .await
        .expect("second sweep");
        assert_eq!(second.l2.written, 0, "an unchanged network writes nothing");
        assert_eq!(second.l2.unchanged, 1);
        assert_eq!(count_l2_decisions(&pool).await.expect("count"), 1);
    }

    /// Guy's arbitration D: an interface absent from a sweep keeps its pair's decision — the pair is not
    /// judged at all, because the universe is this sweep's keys.
    #[tokio::test]
    async fn an_interface_missing_from_a_sweep_keeps_its_pairs_decision() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        sweep(
            &pool,
            vec![
                sighting(1, nic(8), Some("obelix"), 100),
                sighting(2, nic(9), Some("obelix"), 100),
            ],
        )
        .await
        .expect("first sweep");
        let second = sweep(&pool, vec![sighting(11, nic(8), Some("obelix"), 400)])
            .await
            .expect("second sweep");
        assert_eq!(second.l2.candidate_pairs, 0);
        assert_eq!(
            current(&pool).await.len(),
            1,
            "the ambiguity is still current"
        );
    }

    /// The control for D: both interfaces present, and the pair now concludes `AbsenceOfProof` — the
    /// current row is CLOSED, with no successor (G).
    #[tokio::test]
    async fn a_pair_that_decays_to_absence_of_proof_is_closed_with_no_successor() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        sweep(
            &pool,
            vec![
                sighting(1, nic(8), Some("obelix"), 100),
                sighting(2, nic(9), Some("obelix"), 100),
            ],
        )
        .await
        .expect("first sweep");
        let second = sweep(
            &pool,
            vec![
                sighting(11, nic(8), Some("obelix"), 400),
                sighting(12, nic(9), None, 400),
            ],
        )
        .await
        .expect("second sweep");
        assert_eq!(second.l2.vacated, 1);
        assert_eq!(second.l2.written, 0);
        assert!(current(&pool).await.is_empty(), "no current decision");
        assert_eq!(
            count_l2_decisions(&pool).await.expect("count"),
            1,
            "the closed version stays — a decision is closed, never erased"
        );
    }

    /// A pair re-judged at an EARLIER instant than its current version is refused by name rather than
    /// written with a history running backwards (story 5.11's rule, one level up).
    #[tokio::test]
    async fn a_pair_rejudged_in_the_past_is_refused() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        sweep(
            &pool,
            vec![
                sighting(1, nic(8), Some("obelix"), 400),
                sighting(2, nic(9), Some("obelix"), 400),
            ],
        )
        .await
        .expect("first sweep");
        let earlier = sweep(
            &pool,
            vec![
                sighting(11, nic(8), Some("obelix"), 100),
                sighting(12, nic(9), None, 100),
            ],
        )
        .await;
        assert!(
            matches!(earlier, Err(RepositoryError::InstantRegressed)),
            "{earlier:?}"
        );
    }

    /// Story 6.6's registered row: the production caller hands the blocker EVERY key of the sweep. An
    /// uplink narrowing at the call site left the whole suite green; this counts the universe.
    #[tokio::test]
    async fn the_production_universe_is_every_pair_of_the_sweeps_interfaces() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let resolution = sweep(
            &pool,
            (1..=5)
                .map(|n| sighting(n, nic(n as u8), None, 100))
                .collect(),
        )
        .await
        .expect("the sweep must commit");
        assert_eq!(
            resolution.l2.candidate_pairs, 10,
            "n(n-1)/2 over five interfaces"
        );
    }

    /// AC5 — purge the engine's L2 rows, replay, and every decision-bearing column comes back.
    #[tokio::test]
    async fn purge_and_replay_reproduces_every_l2_decision() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let observations = vec![
            sighting(1, nic(8), Some("obelix"), 100),
            sighting(2, nic(9), Some("obelix"), 100),
            sighting(3, vrrp(), Some("gw"), 100),
        ];
        sweep(&pool, observations.clone())
            .await
            .expect("first sweep");
        let before = snapshot_l2_decisions(&pool).await.expect("snapshot");
        assert_eq!(before.len(), 3, "premise: there is something to reproduce");

        assert_eq!(purge_engine_l2_decisions(&pool).await.expect("purge"), 3);
        MariaRepository::new(pool.clone())
            .transact(move |unit| {
                let observations = observations.clone();
                Box::pin(async move { resolve(unit.executor(), &observations).await })
            })
            .await
            .expect("replay");
        assert_eq!(
            snapshot_l2_decisions(&pool).await.expect("snapshot"),
            before
        );
    }

    /// AC8 — the reach section's query reads `identity_link` only, so an L2 decision changes no screen:
    /// after `obelix`'s sweep it reports exactly the two placements L1 made.
    #[tokio::test]
    async fn an_l2_decision_changes_nothing_the_reach_section_counts() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        sweep(
            &pool,
            vec![
                sighting(1, nic(8), Some("obelix"), 100),
                sighting(2, nic(9), Some("obelix"), 100),
            ],
        )
        .await
        .expect("the sweep");
        assert_eq!(
            count_l2_decisions(&pool).await.expect("count"),
            1,
            "premise"
        );
        let reach = count_engine_reach(&pool).await.expect("reach");
        assert_eq!(reach.len(), 1, "{reach:?}");
        assert_eq!((reach[0].outcome.as_str(), reach[0].count), ("match", 2));
    }

    /// AC2 — the seam. A universe WITHHOLDING the obelix pair — a pair that would otherwise be
    /// persisted, which is what the validation's M1 showed the naive version lacked — writes no row;
    /// the control, the full universe, writes one.
    #[tokio::test]
    async fn only_the_pairs_the_blocker_proposed_are_judged() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let observations = vec![
            sighting(1, nic(8), Some("obelix"), 100),
            sighting(2, nic(9), Some("obelix"), 100),
        ];
        let groups = join(&observations);
        let by_id: BTreeMap<ObsId, &Observation> =
            observations.iter().map(|o| (o.obs_id, o)).collect();
        let mut interfaces = BTreeMap::new();
        for (n, key) in groups.keys().enumerate() {
            let id = InterfaceId::from_uuid(uuid::Uuid::from_u128(0xA0 + n as u128));
            insert_interface(&pool, id, key.0, &key.1, at(100), at(100))
                .await
                .expect("interface");
            interfaces.insert(*key, id);
        }
        let keys: Vec<L1Key> = groups.keys().copied().collect();
        let full = l2_candidates(&keys);

        let mut conn = pool.acquire().await.expect("connection");
        let withheld = judge_within(&mut conn, &groups, &by_id, &interfaces, &BTreeSet::new())
            .await
            .expect("judge");
        assert_eq!(withheld.written, 0);
        assert_eq!(count_l2_decisions(&pool).await.expect("count"), 0);

        let control = judge_within(&mut conn, &groups, &by_id, &interfaces, &full)
            .await
            .expect("judge");
        assert_eq!(
            control.written, 1,
            "the same pair IS persisted under the full universe"
        );
        assert_eq!(count_l2_decisions(&pool).await.expect("count"), 1);
    }
}
