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
//!   story has decided how one is minted without a read-then-insert window. ⚠️ *Loudly means the WHOLE
//!   sweep rolls back, L1 included* — the shape the section below rejects for `guard_decision`. The
//!   difference is WHERE it lands: a `Match` needs a new `Decisive` rule, whose own story's tests drive
//!   `obelix`-shaped sweeps through this pass and so red in CI before any deployment runs it, while
//!   `guard_decision` refused a decision the SHIPPED rules produce on a real network every five minutes.
//! - **An unchanged network writes nothing**: a row stores the verdict vector, not the observation ids
//!   minted afresh at every sweep, so the comparison sees the same decision. ⚠️ *For a network whose
//!   ANSWERS are stable*: a reverse-DNS answer that flaps closes the pair when the name is missing and
//!   re-opens it when it returns — two history rows per flap, registered.
//! - **A pair holding a current OPERATOR row is left alone** (Guy, 2026-09-25): a human's decision is an
//!   input the engine neither adopts nor supersedes (D14). Filtering operator rows out of the read — the
//!   first version — made the engine insert beside one and roll the sweep back at every sweep.
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
    close_l2_decision, insert_l2_decision, is_persisted, load_current_l2_decisions,
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
    /// Pairs left alone because an OPERATOR's row is current on them (Guy, 2026-09-25). D14: a
    /// human's decision is an INPUT, which the engine neither adopts nor supersedes.
    pub operator_held: usize,
}

/// Judge every pair of this sweep's interfaces.
///
/// `groups` is `join`'s output for the sweep and `interfaces` the interface each key landed on; both
/// come from the L1 pass that ran just before, in the same transaction.
///
/// # Errors
///
/// Any [`RepositoryError`] a write produces, and [`RepositoryError::InstantRegressed`] when a pair whose
/// decision CHANGED is re-judged at an instant earlier than its current version's. ⚠️ *Only then*: an
/// unchanged decision reached at an earlier instant writes nothing and so has no history to run
/// backwards — this doc promised the refusal for any earlier re-judgement until the code review.
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
    let mut current_rows = load_current_l2_decisions(&mut *conn)
        .await
        .map_err(classify)?;
    for pair in universe {
        let (Some(low_group), Some(high_group)) =
            (groups.get(&pair.low()), groups.get(&pair.high()))
        else {
            // A pair the caller proposed over a key this sweep did not carry: nothing to judge it on.
            continue;
        };
        // ⚠️ A side is the whole `join` group — every observation carrying the key — including any L1
        // declined to PLACE under a narrowed universe. Unreachable in production (`resolve` hands L1 a
        // total universe) and reachable through `resolve_within`'s seam; stated at the code review.
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
        let current = current_rows.remove(&(low.to_string(), high.to_string()));
        if current.as_ref().is_some_and(|row| row.is_operators()) {
            summary.operator_held += 1;
            continue;
        }

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
                match close_l2_decision(&mut *conn, &current.id, reached_at).await {
                    Ok(()) => {}
                    // 🔴 Story 6.14b, AC6 (Guy's 2a): the row this sweep's SNAPSHOT shows was closed by
                    // someone else — an operator answering the question while the sweep judged it. The
                    // slot is re-read with a LOCKING read: under REPEATABLE READ a plain read returns the
                    // snapshot, still showing the ENGINE row, and the skip would never fire (measured by
                    // PR #219's review). An OPERATOR row found there is a human's answer, left alone, and
                    // the sweep commits its L1 placements instead of rolling back for a click. Anything
                    // else, the original refusal stands.
                    Err(RepositoryError::NotFound)
                        if current_is_operators_for_update(&mut *conn, low, high).await? =>
                    {
                        summary.operator_held += 1;
                        continue;
                    }
                    Err(error) => return Err(error),
                }
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

/// Whether the pair's CURRENT row is an operator's, read with a LOCKING read (`FOR UPDATE`) — the
/// latest committed version, never this transaction's snapshot. Story 6.14b's AC6.
///
/// # Errors
///
/// Any database error, classified.
async fn current_is_operators_for_update(
    conn: &mut MySqlConnection,
    low: InterfaceId,
    high: InterfaceId,
) -> Result<bool, RepositoryError> {
    let decided_by: Option<String> = sqlx::query_scalar(
        "SELECT decided_by FROM l2_pair_decision \
         WHERE interface_low = ? AND interface_high = ? AND is_current = 1 FOR UPDATE",
    )
    .bind(low.to_string())
    .bind(high.to_string())
    .fetch_optional(&mut *conn)
    .await
    .map_err(classify)?;
    Ok(decided_by.as_deref() == Some("OPERATOR"))
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

    /// Mint an interface per key of `observations`, in key order, and return what `judge_within` takes.
    async fn seam(
        pool: &MySqlPool,
        observations: &[Observation],
    ) -> (
        BTreeMap<L1Key, BTreeSet<ObsId>>,
        BTreeMap<L1Key, InterfaceId>,
        Vec<L1Key>,
    ) {
        let groups = join(observations);
        let mut interfaces = BTreeMap::new();
        for (n, key) in groups.keys().enumerate() {
            let id = InterfaceId::from_uuid(uuid::Uuid::from_u128(0xA0 + n as u128));
            insert_interface(pool, id, key.0, &key.1, at(100), at(100))
                .await
                .expect("interface");
            interfaces.insert(*key, id);
        }
        let keys: Vec<L1Key> = groups.keys().copied().collect();
        (groups, interfaces, keys)
    }

    /// AC2 — the seam. Three NICs sharing a name are three persisted pairs; a universe WITHHOLDING ONE of
    /// them writes the other two. 🔴 *The first version withheld the only pair of a one-pair fixture, so
    /// "withheld" was the EMPTY universe and could not tell per-pair containment from an emptiness
    /// shortcut* (blind review layer). The control is the full universe.
    #[tokio::test]
    async fn only_the_pairs_the_blocker_proposed_are_judged() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let observations: Vec<Observation> = (1..=3)
            .map(|n| sighting(n, nic(n as u8), Some("obelix"), 100))
            .collect();
        let (groups, interfaces, keys) = seam(&pool, &observations).await;
        let by_id: BTreeMap<ObsId, &Observation> =
            observations.iter().map(|o| (o.obs_id, o)).collect();
        let full = l2_candidates(&keys);
        let withheld = L2CandidatePair::new(keys[0], keys[1]).expect("two keys");
        let narrowed: BTreeSet<L2CandidatePair> = full
            .iter()
            .copied()
            .filter(|pair| *pair != withheld)
            .collect();

        let mut conn = pool.acquire().await.expect("connection");
        let partial = judge_within(&mut conn, &groups, &by_id, &interfaces, &narrowed)
            .await
            .expect("judge");
        assert_eq!(partial.written, 2, "every proposed pair, and only those");
        let (a, b) = (interfaces[&keys[0]], interfaces[&keys[1]]);
        let (low, high) = if a.to_string() < b.to_string() {
            (a, b)
        } else {
            (b, a)
        };
        assert!(
            crate::l2_repo::load_current_l2_decision(&pool, low, high)
                .await
                .expect("read")
                .is_none(),
            "the withheld pair has no row"
        );

        let control = judge_within(&mut conn, &groups, &by_id, &interfaces, &full)
            .await
            .expect("judge");
        assert_eq!(
            control.written, 1,
            "the withheld pair IS persisted under the full universe"
        );
        assert_eq!(count_l2_decisions(&pool).await.expect("count"), 3);
    }

    /// 🔴 The pair is ordered by INTERFACE id, and that order diverges from L1-key order as soon as the
    /// higher-keyed NIC was minted in an EARLIER sweep — the normal case on a live network and on any
    /// upgraded store. The edge review layer measured the swap `if true` leaving the whole suite green:
    /// every test minted its interfaces in key order within one sweep. Without the swap this sweep is
    /// refused (`l2_pair_decision_ordered`) and rolls back, L1 included.
    #[tokio::test]
    async fn a_pair_whose_higher_key_was_minted_first_is_persisted() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        sweep(&pool, vec![sighting(1, nic(9), Some("obelix"), 100)])
            .await
            .expect("the first NIC, alone");
        let second = sweep(
            &pool,
            vec![
                sighting(11, nic(8), Some("obelix"), 400),
                sighting(12, nic(9), Some("obelix"), 400),
            ],
        )
        .await
        .expect("the sweep must commit");
        assert_eq!(second.l2.written, 1);
        assert_eq!(current(&pool).await.len(), 1);
    }

    /// The supersede branch — a persisted decision replaced by a DIFFERENT persisted one. Unreachable
    /// through today's rules for one pair (a VRRP address stays VRRP), so it is driven from a stored row
    /// carrying another vector, as a new ruleset would leave behind. Close + append, one current row.
    #[tokio::test]
    async fn a_changed_persisted_decision_supersedes_the_current_one() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let observations = || {
            vec![
                sighting(1, nic(8), Some("obelix"), 100),
                sighting(2, nic(9), Some("obelix"), 100),
            ]
        };
        sweep(&pool, observations()).await.expect("first sweep");
        sqlx::query("UPDATE l2_pair_decision SET verdicts = 'l2-older-rule=supports'")
            .execute(&pool)
            .await
            .expect("age the row");
        let second = MariaRepository::new(pool.clone())
            .transact(move |unit| {
                let observations = observations();
                Box::pin(async move { resolve(unit.executor(), &observations).await })
            })
            .await
            .expect("second sweep");
        assert_eq!((second.l2.superseded, second.l2.written), (1, 1));
        assert_eq!(count_l2_decisions(&pool).await.expect("count"), 2);
        let rows = current(&pool).await;
        assert_eq!(rows.len(), 1);
        assert!(
            rows[0].3.contains("l2-hostname-agrees=supports"),
            "{rows:?}"
        );
    }

    /// Guy's decision of 2026-09-25: a pair holding a current OPERATOR row is LEFT ALONE — the row
    /// survives, the sweep commits, and the L1 links are written. 🔴 Before, the engine read ENGINE rows
    /// only, inserted beside the human's, collided on the uniqueness key and rolled every sweep back.
    #[tokio::test]
    async fn a_pair_an_operator_decided_is_left_to_the_operator() {
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
        // Story 6.14b, AC4: the operator's row is written by its REAL producer — the answer's adapter —
        // where this test forged it with a raw `UPDATE` until then.
        answer(&pool, crate::l2_answer::Answer::DistinctMachines, 100).await;
        let links_before = links(&pool).await;
        let second = sweep(
            &pool,
            vec![
                sighting(11, nic(8), Some("obelix"), 400),
                sighting(12, nic(9), None, 400),
            ],
        )
        .await
        .expect("the sweep must commit");
        assert_eq!(second.l2.operator_held, 1);
        assert_eq!((second.l2.written, second.l2.vacated), (0, 0));
        assert_eq!(
            links(&pool).await,
            links_before + 2,
            "the sweep's L1 links are written"
        );
        let who: Vec<String> =
            sqlx::query_scalar("SELECT decided_by FROM l2_pair_decision WHERE is_current = 1")
                .fetch_all(&pool)
                .await
                .expect("read");
        assert_eq!(who, vec!["OPERATOR".to_string()]);
    }

    /// Answer the one open question of the store through the adapter, shown at `seen` seconds — what
    /// `POST /triage/answer` does, without the route.
    async fn answer(pool: &MySqlPool, answer: crate::l2_answer::Answer, seen: i64) {
        let members: BTreeSet<String> = sqlx::query_scalar("SELECT id FROM interface")
            .fetch_all(pool)
            .await
            .expect("read")
            .into_iter()
            .collect();
        let mut conn = pool.acquire().await.expect("connection");
        let mut tx = sqlx::Connection::begin(&mut *conn).await.expect("tx");
        crate::l2_answer::record_operator_answer(&mut tx, &members, answer, at(seen))
            .await
            .expect("the answer");
        tx.commit().await.expect("commit");
    }

    /// 🔴 Story 6.14b, AC6: an answer committed WHILE a sweep judges the same pair does not cost the
    /// sweep. The sweep's snapshot still shows the ENGINE row; its decision CHANGED (one NIC lost its
    /// name), so it goes to close that row, finds the operator already closed it, re-reads the slot —
    /// with a LOCKING read, since a plain one returns the snapshot under REPEATABLE READ (measured by
    /// PR #219's review) — finds the OPERATOR row, and skips the pair. It commits its L1 links.
    ///
    /// ⚠️ The changed decision is load-bearing: an UNCHANGED pair is counted `unchanged` from the
    /// snapshot and never reaches the close, so a test on that branch would measure nothing.
    #[tokio::test]
    async fn an_answer_that_races_a_sweep_does_not_cost_the_sweep() {
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
        let second_sweep = vec![
            sighting(11, nic(8), Some("obelix"), 400),
            sighting(12, nic(9), None, 400),
        ];
        for observation in &second_sweep {
            crate::repo::insert_observation(&pool, observation)
                .await
                .expect("insert observation");
        }
        // The sweep's transaction opens and TAKES ITS SNAPSHOT first — the ENGINE row is current in it.
        let mut sweep_conn = pool.acquire().await.expect("connection");
        let mut sweep_tx = sqlx::Connection::begin(&mut *sweep_conn).await.expect("tx");
        let seen: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM l2_pair_decision")
            .fetch_one(&mut *sweep_tx)
            .await
            .expect("the snapshot");
        assert_eq!(seen, 1, "premise: the question is in the sweep's snapshot");
        // Then the operator answers, and commits.
        answer(&pool, crate::l2_answer::Answer::SameMachine, 100).await;
        let links_before = links(&pool).await;
        let resolution = resolve(&mut sweep_tx, &second_sweep)
            .await
            .expect("the sweep must not fail on the operator's answer");
        sweep_tx.commit().await.expect("the sweep commits");
        assert_eq!(resolution.l2.operator_held, 1, "{resolution:?}");
        assert_eq!((resolution.l2.written, resolution.l2.vacated), (0, 0));
        assert_eq!(
            links(&pool).await,
            links_before + 2,
            "the sweep's L1 links are written"
        );
        let who: Vec<(String, String)> =
            sqlx::query_as("SELECT decided_by, outcome FROM l2_pair_decision WHERE is_current = 1")
                .fetch_all(&pool)
                .await
                .expect("read");
        assert_eq!(who, vec![("OPERATOR".to_string(), "match".to_string())]);
    }

    /// Story 6.6's registered hazard is an UPLINK narrowing at the call site. 🔴 The first guard counted
    /// a population carrying no uplink, where *"keep pairs whose uplinks do not disagree"* keeps every
    /// pair (blind review layer). Here every interface reports a DIFFERENT uplink, so that filter would
    /// keep none, and the count still says `n(n-1)/2`.
    #[tokio::test]
    async fn interfaces_on_different_uplinks_are_still_all_paired() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let observations: Vec<Observation> = (1..=4)
            .map(|n| {
                let mut o = sighting(n, nic(n as u8), None, 100);
                o.facts.push(Fact::Uplink {
                    peer_mac: MacAddr([0x02, 0, 0, 0, 0, n as u8]),
                    peer_port: format!("port-{n}"),
                });
                o
            })
            .collect();
        let resolution = sweep(&pool, observations).await.expect("the sweep");
        assert_eq!(resolution.l2.candidate_pairs, 6);
    }

    /// The two guard branches of `judge_within`, reachable only through the seam: a proposed pair over a
    /// key this sweep did not carry is skipped, and a key the L1 pass placed on no interface is refused.
    #[tokio::test]
    async fn a_pair_over_an_absent_key_is_skipped_and_an_unplaced_key_is_refused() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let observations = vec![
            sighting(1, nic(8), Some("obelix"), 100),
            sighting(2, nic(9), Some("obelix"), 100),
        ];
        let (groups, mut interfaces, keys) = seam(&pool, &observations).await;
        let by_id: BTreeMap<ObsId, &Observation> =
            observations.iter().map(|o| (o.obs_id, o)).collect();
        let absent: L1Key = (domain(), nic(77));
        let stray: BTreeSet<L2CandidatePair> =
            [L2CandidatePair::new(keys[0], absent).expect("pair")].into();
        let mut conn = pool.acquire().await.expect("connection");
        let skipped = judge_within(&mut conn, &groups, &by_id, &interfaces, &stray)
            .await
            .expect("an absent key is skipped, not an error");
        assert_eq!(skipped.written, 0);

        interfaces.remove(&keys[1]);
        let refused = judge_within(
            &mut conn,
            &groups,
            &by_id,
            &interfaces,
            &l2_candidates(&keys),
        )
        .await;
        assert!(
            matches!(refused, Err(RepositoryError::Backend(_))),
            "{refused:?}"
        );
    }

    /// Story 6.14: the screen reads current ENGINE `Ambiguous` pairs ONLY — never a `NoMatch` (the
    /// software decided) and never an OPERATOR row (an answer, not a question). 🔴 The validation's M2
    /// (drop the ENGINE filter) was green before a test of this shape existed.
    #[tokio::test]
    async fn the_screen_reads_engine_ambiguities_and_nothing_else() {
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
        .expect("the sweep");
        assert_eq!(
            current(&pool).await.len(),
            3,
            "premise: one ambiguity and two refusals"
        );
        let pairs = crate::l2_repo::load_current_ambiguous_pairs(&pool)
            .await
            .expect("read");
        assert_eq!(
            pairs.len(),
            1,
            "the ambiguity, and not the two `NoMatch`: {pairs:?}"
        );
        // Story 6.14b, AC4: answered through the adapter, where this test forged the row by `UPDATE`.
        let obelix: BTreeSet<String> = sqlx::query_scalar(
            "SELECT interface_low FROM l2_pair_decision WHERE outcome = 'abstained' \
             UNION SELECT interface_high FROM l2_pair_decision WHERE outcome = 'abstained'",
        )
        .fetch_all(&pool)
        .await
        .expect("read")
        .into_iter()
        .collect();
        let mut conn = pool.acquire().await.expect("connection");
        let mut tx = sqlx::Connection::begin(&mut *conn).await.expect("tx");
        crate::l2_answer::record_operator_answer(
            &mut tx,
            &obelix,
            crate::l2_answer::Answer::DistinctMachines,
            at(100),
        )
        .await
        .expect("a human answers");
        tx.commit().await.expect("commit");
        assert!(
            crate::l2_repo::load_current_ambiguous_pairs(&pool)
                .await
                .expect("read")
                .is_empty(),
            "an operator's row is an answer, not a question"
        );
        assert!(
            crate::l2_repo::load_ambiguous_interface_sightings(&pool)
                .await
                .expect("read")
                .is_empty(),
            "and the sightings reader reads the SAME pairs — its ENGINE filter was tested by nothing"
        );
    }

    /// One observation at a given address — `sighting` plus an `IpV4`.
    fn at_address(mut observation: Observation, address: &str) -> Observation {
        observation.facts.push(Fact::IpV4 {
            addr: address.parse().expect("an address"),
        });
        observation
    }

    /// 🔴 An interface answering at TWO addresses in ONE sweep: both observations carry the sweep's
    /// instant, so both are its latest, and the reader returns BOTH (the first version kept one).
    #[tokio::test]
    async fn two_sightings_tied_at_one_instant_are_both_read() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        sweep(
            &pool,
            vec![
                at_address(sighting(1, nic(8), Some("obelix"), 100), "192.0.2.8"),
                at_address(sighting(2, nic(8), Some("obelix"), 100), "192.0.2.18"),
                at_address(sighting(3, nic(9), Some("obelix"), 100), "192.0.2.9"),
            ],
        )
        .await
        .expect("the sweep");
        let sightings = crate::l2_repo::load_ambiguous_interface_sightings(&pool)
            .await
            .expect("read");
        assert_eq!(
            sightings.len(),
            3,
            "two for nic 8, one for nic 9: {sightings:?}"
        );
    }

    /// Story 6.14, decision F: each interface's LATEST placed sighting is what the pane shows — read in
    /// SQL, one row per interface, and the later sweep wins.
    #[tokio::test]
    async fn the_candidates_are_read_from_each_interfaces_latest_sighting() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        sweep(
            &pool,
            vec![
                at_address(sighting(1, nic(8), Some("obelix"), 100), "192.0.2.8"),
                sighting(2, nic(9), Some("obelix"), 100),
            ],
        )
        .await
        .expect("first sweep");
        sweep(
            &pool,
            vec![
                // The LATER sweep sees nic 8 at a DIFFERENT address — the only shape that separates
                // *as seen now* from *as first seen* on a real store (story 6.14's review).
                at_address(sighting(11, nic(8), Some("obelix"), 400), "192.0.2.88"),
                sighting(12, nic(9), Some("obelix"), 400),
            ],
        )
        .await
        .expect("second sweep");
        let sightings = crate::l2_repo::load_ambiguous_interface_sightings(&pool)
            .await
            .expect("read");
        let (view, _) = crate::page::triage_view(&pool, Some("__none__"), false, false)
            .await
            .unwrap_or_else(|response| panic!("the triage view builds: {}", response.status()));
        let question = view
            .rows
            .iter()
            .find(|row| row.id.starts_with("ambigu:"))
            .expect("one question");
        assert!(
            question.entity.contains("192.0.2.88") && !question.entity.contains("192.0.2.8 "),
            "the screen shows the address the LATER sweep saw: {}",
            question.entity
        );
        let observations: std::collections::BTreeSet<String> =
            sightings.iter().filter_map(|(_, _, o)| o.clone()).collect();
        let expected: std::collections::BTreeSet<String> = [11_u128, 12]
            .into_iter()
            .map(|n| ObsId::from_uuid(uuid::Uuid::from_u128(n)).to_string())
            .collect();
        assert_eq!(sightings.len(), 2, "one row per interface: {sightings:?}");
        assert_eq!(observations, expected, "the LATER sweep's observations");
    }
}
