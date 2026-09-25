//! The adapter for `l2_pair_decision` — the engine's L2 decision about a PAIR of interfaces
//! (story 6.12, migration `0012`).
//!
//! It lives here and not in `repo.rs` because `repo.rs` is at 1789 of the 2000 code lines the
//! `file-size` rule allows, and a gate blind to that file (story 6.5's registered row) is no reason to
//! grow it: `sighting_repo.rs` and `ipam_repo.rs` set the precedent.
//!
//! # What a row says, and what it deliberately does not
//!
//! A row is a decision the L2 cascade reached about two interfaces: `NoMatch` (a rule excluded the
//! pair — today only `l2-virtual-mac-prefix`) or `Abstained { Ambiguous }` (the pair looks alike and
//! the engine will not guess — `obelix`'s shape). **It never says `Match`** — no L2 rule is `Decisive`
//! — **and never `AbsenceOfProof`**, which is not persisted (Guy's arbitration, 2026-09-24). Both
//! refusals are made here before a row is written and again by `0012`'s CHECKs.
//!
//! **The evidence is the vector, not observation ids** (the same arbitration, F): a row stores each
//! rule's verdict, which stays the same while the network does, and its subject is the pair. The
//! observation ids a rule read are minted afresh at every sweep, so storing them would make an
//! unchanged network write at every sweep.

use opencmdb_core::identity::cascade::{Conclusion, Decision, IdentityAbstentionCause, Verdict};
use opencmdb_core::observation::{InterfaceId, Timestamp};
use opencmdb_core::repo::RepositoryError;
use sqlx::{Executor, MySql};

use crate::repo::{DecidedBy, OPEN_END, cause_token, classify, datetime_literal, outcome_token};

/// The persisted spelling of one verdict. Exhaustive, so a sixth verdict cannot be written until
/// someone says how it reads back.
pub(crate) fn verdict_token(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Decisive => "decisive",
        Verdict::Supports => "supports",
        Verdict::Neutral => "neutral",
        Verdict::Opposes => "opposes",
        Verdict::Disqualifying => "disqualifying",
    }
}

/// A decision's verdict vector as `0012` stores it: `rule=verdict` pairs joined by `;`, in the order
/// the vector carries them. Evidence is deliberately left out — see the module doc.
pub(crate) fn verdicts_of(decision: &Decision) -> String {
    decision
        .verdict_vector
        .iter()
        .map(|verdict| format!("{}={}", verdict.rule.0, verdict_token(verdict.verdict)))
        .collect::<Vec<_>>()
        .join(";")
}

/// Whether a decision is one this table holds.
///
/// `Ok(true)` for `NoMatch` and `Abstained { Ambiguous }`; `Ok(false)` for `Abstained { AbsenceOfProof }`,
/// which is never persisted, by decision.
///
/// # Errors
///
/// [`RepositoryError::Constraint`]`("l2_match_not_persisted")` for a `Match`: no L2 rule produces one
/// today, and a match between two interfaces is a DEVICE, which this story does not mint. Refused by
/// name rather than dropped, so the day an L2 `Decisive` arrives its first pass fails loudly instead of
/// silently discarding the first grouping this product ever made.
pub(crate) fn is_persisted(decision: &Decision) -> Result<bool, RepositoryError> {
    match &decision.conclusion {
        Conclusion::Match { .. } => Err(RepositoryError::Constraint("l2_match_not_persisted")),
        Conclusion::NoMatch { .. } => Ok(true),
        Conclusion::Abstained {
            cause: IdentityAbstentionCause::Ambiguous,
        } => Ok(true),
        Conclusion::Abstained {
            cause: IdentityAbstentionCause::AbsenceOfProof,
        } => Ok(false),
    }
}

/// `id, outcome, rule_id, abstention_cause, verdicts, ruleset_version, valid_from, decided_by`, as
/// decoded.
#[cfg(test)]
type CurrentRow = (
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    u32,
    String,
    String,
);

/// `interface_low, interface_high` followed by a [`CurrentRow`], as the batch read decodes it.
type CurrentPairRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    u32,
    String,
    String,
);

/// The current version of one pair's decision, as stored — the engine's or an operator's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CurrentL2Decision {
    /// The row id — a v7 UUID, a row identifier and not part of the decision.
    pub(crate) id: String,
    /// `no_match` or `abstained`.
    pub(crate) outcome: String,
    /// The rule that excluded the pair, for a `no_match`.
    pub(crate) rule_id: Option<String>,
    /// `ambiguous`, for an abstention.
    pub(crate) abstention_cause: Option<String>,
    /// The stored verdict vector.
    pub(crate) verdicts: String,
    /// The ruleset that reached it.
    pub(crate) ruleset_version: u32,
    /// Its `valid_from`, rendered — compared, never decoded (`sqlx` is built without `chrono`).
    pub(crate) valid_from: String,
    /// `ENGINE` or `OPERATOR`. The pass leaves an operator's pair alone (Guy, 2026-09-25).
    pub(crate) decided_by: String,
}

impl CurrentL2Decision {
    /// Whether a human wrote this version. D14: an operator's row is an INPUT, which the engine
    /// neither adopts nor supersedes.
    pub(crate) fn is_operators(&self) -> bool {
        self.decided_by == "OPERATOR"
    }

    /// Whether this stored version carries the decision a pass is about to write.
    ///
    /// Every decision-bearing column, and nothing minted per sweep: that is what makes an unchanged
    /// network write nothing. `valid_from` is excluded — it is when the decision was reached, not what it
    /// is — and so is the id.
    pub(crate) fn carries(&self, decision: &Decision) -> bool {
        let (rule_id, abstention_cause) = match &decision.conclusion {
            Conclusion::Match { rule } | Conclusion::NoMatch { rule } => {
                (Some(rule.0.as_str()), None)
            }
            Conclusion::Abstained { cause } => (None, Some(cause_token(cause))),
        };
        self.outcome == outcome_token(&decision.conclusion)
            && self.rule_id.as_deref() == rule_id
            && self.abstention_cause.as_deref() == abstention_cause
            && self.verdicts == verdicts_of(decision)
            && self.ruleset_version == decision.ruleset_version.0
    }
}

/// Every CURRENT decision — the engine's AND an operator's — keyed by `(interface_low,
/// interface_high)`: ONE static query for a whole sweep.
///
/// 🔴 **One query, not one per pair, and the reason is measured.** The pass first read each pair's row
/// separately: at 300 interfaces that is 44 850 round trips inside the sweep's transaction, which the
/// code review measured adding ~2.8 s per sweep and taking the reference-scale test from 139–382 ms to
/// 3.7–9.8 s. It reads the WHOLE current set rather than filtering on the sweep's interfaces because the
/// table holds only `Ambiguous` and `NoMatch` pairs — a small set by Guy's decision G — and because a
/// filter would need an `IN` list assembled at runtime, which is a query nobody can read in the source.
///
/// 🔴 **Operator rows are READ, not filtered out.** Filtering them made the engine blind to a human's
/// row, insert beside it, collide on `l2_pair_decision_one_current` and roll the whole sweep back — L1
/// included, at every sweep (measured by the review's edge layer). The caller decides what an operator's
/// row means; see `l2_pass`.
///
/// # Errors
///
/// Any database error.
pub(crate) async fn load_current_l2_decisions<'e, E>(
    executor: E,
) -> Result<std::collections::BTreeMap<(String, String), CurrentL2Decision>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<CurrentPairRow> = sqlx::query_as(
        "SELECT interface_low, interface_high, id, outcome, rule_id, abstention_cause, verdicts, \
         ruleset_version, DATE_FORMAT(valid_from, '%Y-%m-%d %H:%i:%s.%f'), decided_by \
         FROM l2_pair_decision WHERE is_current = 1",
    )
    .fetch_all(executor)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(
                low,
                high,
                id,
                outcome,
                rule_id,
                abstention_cause,
                verdicts,
                ruleset_version,
                valid_from,
                decided_by,
            )| {
                (
                    (low, high),
                    CurrentL2Decision {
                        id,
                        outcome,
                        rule_id,
                        abstention_cause,
                        verdicts,
                        ruleset_version,
                        valid_from,
                        decided_by,
                    },
                )
            },
        )
        .collect())
}

/// Every current ENGINE `Ambiguous` pair, as `(interface_low, interface_high, verdicts)` — the
/// questions story 6.14 shows on `/triage`.
///
/// ⚠️ **ENGINE rows only, by decision**: an OPERATOR row on a pair is an answer, not a question
/// (Guy, 2026-09-25, at story 6.12's code review), and a `no_match` is a decision the software took
/// (case one of Guy's taxonomy) — neither is shown as a doubt to lift.
///
/// # Errors
///
/// Any database error.
pub(crate) async fn load_current_ambiguous_pairs<'e, E>(
    executor: E,
) -> Result<Vec<(String, String, String)>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    sqlx::query_as(
        "SELECT interface_low, interface_high, verdicts FROM l2_pair_decision \
         WHERE is_current = 1 AND decided_by = 'ENGINE' AND outcome = 'abstained' \
         AND abstention_cause = 'ambiguous'",
    )
    .fetch_all(executor)
    .await
}

/// For every interface in a current ENGINE `Ambiguous` pair: its id, its hardware address, and the
/// observation of its LATEST current placement — what the candidates pane shows *as seen now*.
///
/// 🔑 **Per-interface latest, in SQL**, and the reason is a projection: every observation's placement
/// link stays current for ever, so reading the whole `identity_link` table per render would grow by
/// about one row per host per sweep (reasoned by story 6.14's validation, ≈13k a day at the reference
/// cadence). Several observations tied at the same instant all come back; the caller keeps one.
///
/// # Errors
///
/// Any database error.
pub(crate) async fn load_ambiguous_interface_sightings<'e, E>(
    executor: E,
) -> Result<Vec<(String, String, String)>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    sqlx::query_as(
        "SELECT i.id, i.mac_canon, l.observation_id \
         FROM interface i \
         JOIN identity_link l ON l.interface_id = i.id AND l.outcome = 'match' \
              AND l.valid_to = ? \
         JOIN observation_record o ON o.id = l.observation_id \
         WHERE i.id IN ( \
             SELECT interface_low FROM l2_pair_decision WHERE is_current = 1 \
                 AND decided_by = 'ENGINE' AND abstention_cause = 'ambiguous' \
             UNION \
             SELECT interface_high FROM l2_pair_decision WHERE is_current = 1 \
                 AND decided_by = 'ENGINE' AND abstention_cause = 'ambiguous') \
         AND o.observed_at = ( \
             SELECT MAX(o2.observed_at) FROM identity_link l2 \
             JOIN observation_record o2 ON o2.id = l2.observation_id \
             WHERE l2.interface_id = i.id AND l2.outcome = 'match' AND l2.valid_to = ?) \
         ORDER BY i.id, l.observation_id",
    )
    .bind(OPEN_END)
    .bind(OPEN_END)
    .fetch_all(executor)
    .await
}

/// The current decision about `(low, high)`, if any — the engine's or an operator's.
///
/// # Errors
///
/// Any database error.
#[cfg(test)]
pub(crate) async fn load_current_l2_decision<'e, E>(
    executor: E,
    low: InterfaceId,
    high: InterfaceId,
) -> Result<Option<CurrentL2Decision>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let row: Option<CurrentRow> = sqlx::query_as(
        "SELECT id, outcome, rule_id, abstention_cause, verdicts, ruleset_version, \
         DATE_FORMAT(valid_from, '%Y-%m-%d %H:%i:%s.%f'), decided_by \
         FROM l2_pair_decision \
         WHERE interface_low = ? AND interface_high = ? AND is_current = 1",
    )
    .bind(low.to_string())
    .bind(high.to_string())
    .fetch_optional(executor)
    .await?;
    Ok(row.map(
        |(
            id,
            outcome,
            rule_id,
            abstention_cause,
            verdicts,
            ruleset_version,
            valid_from,
            decided_by,
        )| {
            CurrentL2Decision {
                id,
                outcome,
                rule_id,
                abstention_cause,
                verdicts,
                ruleset_version,
                valid_from,
                decided_by,
            }
        },
    ))
}

/// Insert a CURRENT version of one pair's decision.
///
/// # Errors
///
/// [`RepositoryError::Constraint`] when the decision is not one this table holds (see
/// [`is_persisted`]) or when `low` does not sort before `high`; any database error, classified.
pub(crate) async fn insert_l2_decision<'e, E>(
    executor: E,
    id: uuid::Uuid,
    low: InterfaceId,
    high: InterfaceId,
    decision: &Decision,
    decided_by: DecidedBy,
    valid_from: Timestamp,
) -> Result<(), RepositoryError>
where
    E: Executor<'e, Database = MySql>,
{
    if !is_persisted(decision)? {
        return Err(RepositoryError::Constraint(
            "l2_absence_of_proof_not_persisted",
        ));
    }
    if low.to_string() >= high.to_string() {
        return Err(RepositoryError::Constraint("l2_pair_decision_ordered"));
    }
    let (rule_id, abstention_cause) = match &decision.conclusion {
        Conclusion::Match { rule } | Conclusion::NoMatch { rule } => (Some(rule.0.clone()), None),
        Conclusion::Abstained { cause } => (None, Some(cause_token(cause))),
    };
    sqlx::query(
        "INSERT INTO l2_pair_decision \
         (id, interface_low, interface_high, outcome, rule_id, abstention_cause, verdicts, \
          ruleset_version, decided_by, valid_from, valid_to, is_current) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1)",
    )
    .bind(id.to_string())
    .bind(low.to_string())
    .bind(high.to_string())
    .bind(outcome_token(&decision.conclusion))
    .bind(rule_id)
    .bind(abstention_cause)
    .bind(verdicts_of(decision))
    .bind(decision.ruleset_version.0)
    .bind(decided_by.token())
    .bind(datetime_literal(valid_from))
    .bind(OPEN_END)
    .execute(executor)
    .await
    .map_err(classify)?;
    Ok(())
}

/// Close one current version at `closed_at`.
///
/// # Errors
///
/// [`RepositoryError::Constraint`]`("check")` when `closed_at` IS the sentinel — closing there would
/// leave the row reading as current (story 5.9's resurrection defect, one table over);
/// [`RepositoryError::NotFound`] when no CURRENT row carries that id.
pub(crate) async fn close_l2_decision<'e, E>(
    executor: E,
    id: &str,
    closed_at: Timestamp,
) -> Result<(), RepositoryError>
where
    E: Executor<'e, Database = MySql>,
{
    let closed_at_literal = datetime_literal(closed_at);
    if closed_at_literal == OPEN_END {
        return Err(RepositoryError::Constraint("check"));
    }
    let result = sqlx::query(
        "UPDATE l2_pair_decision SET valid_to = ?, is_current = NULL \
         WHERE id = ? AND is_current = 1",
    )
    .bind(&closed_at_literal)
    .bind(id)
    .execute(executor)
    .await
    .map_err(classify)?;
    if result.rows_affected() == 0 {
        return Err(RepositoryError::NotFound);
    }
    Ok(())
}

/// Delete every ENGINE row — story 5.10's purge, at L2. `purge_engine_links` purges `identity_link`
/// only, and this table is not a child of it.
///
/// # Errors
///
/// Any database error.
#[cfg(test)]
pub(crate) async fn purge_engine_l2_decisions<'e, E>(executor: E) -> Result<u64, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let result = sqlx::query("DELETE FROM l2_pair_decision WHERE decided_by = 'ENGINE'")
        .execute(executor)
        .await?;
    Ok(result.rows_affected())
}

/// One CURRENT row's decision-bearing columns, with no id — what a purge-and-replay must reproduce.
///
/// The pair is compared: under Guy's arbitration E the pair IS the candidate set of an ambiguity, so
/// comparing it is what story 5.10's snapshot was blind to at L1 (`link_candidate`, registered).
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct L2Snapshot {
    /// The lower interface id.
    pub(crate) interface_low: String,
    /// The higher interface id.
    pub(crate) interface_high: String,
    /// `no_match` or `abstained`.
    pub(crate) outcome: String,
    /// The rule, for a `no_match`.
    pub(crate) rule_id: Option<String>,
    /// The cause, for an abstention.
    pub(crate) abstention_cause: Option<String>,
    /// The stored verdict vector.
    pub(crate) verdicts: String,
    /// The ruleset.
    pub(crate) ruleset_version: u32,
    /// Its `valid_from`, rendered.
    pub(crate) valid_from: String,
}

/// `interface_low, interface_high, outcome, rule_id, abstention_cause, verdicts, ruleset_version,
/// valid_from`, as decoded.
#[cfg(test)]
type SnapshotRow = (
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    String,
    u32,
    String,
);

/// Every CURRENT engine row, sorted, without ids.
///
/// # Errors
///
/// Any database error.
#[cfg(test)]
pub(crate) async fn snapshot_l2_decisions<'e, E>(
    executor: E,
) -> Result<Vec<L2Snapshot>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<SnapshotRow> = sqlx::query_as(
        "SELECT interface_low, interface_high, outcome, rule_id, abstention_cause, verdicts, \
         ruleset_version, DATE_FORMAT(valid_from, '%Y-%m-%d %H:%i:%s.%f') \
         FROM l2_pair_decision WHERE is_current = 1 AND decided_by = 'ENGINE'",
    )
    .fetch_all(executor)
    .await?;
    // Sorted in Rust, not by `ORDER BY`: a sort inside a function used on BOTH sides of a comparison
    // is unreddenable (story 5.10's finding), and this one is pinned by its own test.
    let mut snapshot: Vec<L2Snapshot> = rows
        .into_iter()
        .map(
            |(
                interface_low,
                interface_high,
                outcome,
                rule_id,
                abstention_cause,
                verdicts,
                ruleset_version,
                valid_from,
            )| L2Snapshot {
                interface_low,
                interface_high,
                outcome,
                rule_id,
                abstention_cause,
                verdicts,
                ruleset_version,
                valid_from,
            },
        )
        .collect();
    snapshot.sort();
    Ok(snapshot)
}

/// How many rows the table holds, current and closed.
///
/// # Errors
///
/// Any database error.
#[cfg(test)]
pub(crate) async fn count_l2_decisions<'e, E>(executor: E) -> Result<i64, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    sqlx::query_scalar("SELECT COUNT(*) FROM l2_pair_decision")
        .fetch_one(executor)
        .await
}

/// Tests for the adapter and for `0012`'s refusals.
///
/// The CHECK probes go AROUND the adapter with raw SQL, on story 5.9's M3 lesson: a guard the adapter
/// cannot violate is reachable only by a write the adapter would never make.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo::insert_interface;
    use opencmdb_core::identity::cascade::{RuleVerdict, RulesetVersion, decide};
    use opencmdb_core::observation::{L2DomainId, MacAddr};
    use opencmdb_core::trap::RuleId;
    use sqlx::MySqlPool;

    const LOW: &str = "00000000-0000-0000-0000-0000000000a1";
    const HIGH: &str = "00000000-0000-0000-0000-0000000000a2";

    fn verdict(rule: &str, verdict: Verdict) -> RuleVerdict {
        RuleVerdict {
            rule: RuleId(rule.to_string()),
            verdict,
            evidence: Vec::new(),
        }
    }

    /// `obelix`'s decision, as `l2::decide_pair` builds it.
    fn ambiguous() -> Decision {
        decide(
            vec![
                verdict("l2-different-hostname", Verdict::Neutral),
                verdict("l2-hostname-agrees", Verdict::Supports),
                verdict("l2-virtual-mac-prefix", Verdict::Neutral),
            ],
            RulesetVersion(1),
        )
    }

    fn at(secs: i64) -> Timestamp {
        chrono::DateTime::from_timestamp(secs, 0).expect("in range")
    }

    fn stored(decision: &Decision) -> CurrentL2Decision {
        let (rule_id, abstention_cause) = match &decision.conclusion {
            Conclusion::Match { rule } | Conclusion::NoMatch { rule } => {
                (Some(rule.0.clone()), None)
            }
            Conclusion::Abstained { cause } => (None, Some(cause_token(cause).to_string())),
        };
        CurrentL2Decision {
            id: "x".to_string(),
            outcome: outcome_token(&decision.conclusion).to_string(),
            rule_id,
            abstention_cause,
            verdicts: verdicts_of(decision),
            ruleset_version: decision.ruleset_version.0,
            valid_from: "1970-01-01 00:01:40.000000".to_string(),
            decided_by: "ENGINE".to_string(),
        }
    }

    // ---- pure: what counts as the SAME decision, one column at a time ----

    #[test]
    fn a_stored_version_carries_the_decision_it_was_written_from() {
        assert!(stored(&ambiguous()).carries(&ambiguous()));
    }

    #[test]
    fn each_decision_bearing_column_is_compared() {
        let decision = ambiguous();
        let mut outcome = stored(&decision);
        outcome.outcome = "no_match".to_string();
        let mut rule = stored(&decision);
        rule.rule_id = Some("l2-virtual-mac-prefix".to_string());
        let mut cause = stored(&decision);
        cause.abstention_cause = Some("absence_of_proof".to_string());
        let mut verdicts = stored(&decision);
        verdicts.verdicts = "l2-hostname-agrees=supports".to_string();
        let mut ruleset = stored(&decision);
        ruleset.ruleset_version = 2;
        for (column, row) in [
            ("outcome", outcome),
            ("rule_id", rule),
            ("abstention_cause", cause),
            ("verdicts", verdicts),
            ("ruleset_version", ruleset),
        ] {
            assert!(!row.carries(&decision), "{column} must be compared");
        }
    }

    /// `valid_from` and the id are NOT decision-bearing: a version found later is the same decision.
    #[test]
    fn when_a_decision_was_reached_is_not_what_it_is() {
        let decision = ambiguous();
        let mut later = stored(&decision);
        later.valid_from = "2026-09-24 00:00:00.000000".to_string();
        later.id = "y".to_string();
        assert!(later.carries(&decision));
    }

    #[test]
    fn the_verdict_vector_is_stored_in_order_with_no_evidence() {
        assert_eq!(
            verdicts_of(&ambiguous()),
            "l2-different-hostname=neutral;l2-hostname-agrees=supports;l2-virtual-mac-prefix=neutral"
        );
    }

    #[test]
    fn every_verdict_has_a_distinct_token() {
        let tokens: std::collections::BTreeSet<&str> = [
            Verdict::Decisive,
            Verdict::Supports,
            Verdict::Neutral,
            Verdict::Opposes,
            Verdict::Disqualifying,
        ]
        .into_iter()
        .map(verdict_token)
        .collect();
        assert_eq!(tokens.len(), 5);
    }

    /// `Match` is refused by name and `AbsenceOfProof` is not persisted — Guy's arbitration G.
    #[test]
    fn only_no_match_and_ambiguity_are_persisted() {
        let no_match = decide(
            vec![verdict("l2-virtual-mac-prefix", Verdict::Disqualifying)],
            RulesetVersion(1),
        );
        assert_eq!(is_persisted(&no_match), Ok(true));
        assert_eq!(is_persisted(&ambiguous()), Ok(true));
        let absence = decide(vec![], RulesetVersion(1));
        assert_eq!(is_persisted(&absence), Ok(false));
        let hand_built_match = decide(
            vec![verdict("l2-hypothetical", Verdict::Decisive)],
            RulesetVersion(1),
        );
        assert_eq!(
            is_persisted(&hand_built_match),
            Err(RepositoryError::Constraint("l2_match_not_persisted"))
        );
    }

    // ---- against a store ----

    async fn store() -> Option<MySqlPool> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping l2_repo test: DATABASE_URL unset");
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
        ] {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }
        for (id, last) in [(LOW, 1), (HIGH, 2)] {
            insert_interface(
                &pool,
                InterfaceId::from_uuid(uuid::Uuid::parse_str(id).expect("uuid")),
                L2DomainId::from_uuid(uuid::Uuid::from_u128(1)),
                &MacAddr([0x00, 0x11, 0x22, 0x33, 0x44, last]),
                at(100),
                at(100),
            )
            .await
            .expect("interface");
        }
        Some(pool)
    }

    /// A raw insert with every column settable; the defaults are a VALID row.
    async fn raw(
        pool: &MySqlPool,
        low: &str,
        high: &str,
        (outcome, rule_id, cause): (&str, Option<&str>, Option<&str>),
        valid_to: &str,
        is_current: Option<u8>,
    ) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO l2_pair_decision (id, interface_low, interface_high, outcome, rule_id, \
             abstention_cause, verdicts, ruleset_version, decided_by, valid_from, valid_to, \
             is_current) VALUES (UUID(), ?, ?, ?, ?, ?, 'r=neutral', 1, 'ENGINE', \
             '2026-01-01 00:00:00', ?, ?)",
        )
        .bind(low)
        .bind(high)
        .bind(outcome)
        .bind(rule_id)
        .bind(cause)
        .bind(valid_to)
        .bind(is_current)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
    }

    /// The constraint is matched WITH the backticks MariaDB prints around it (`` CONSTRAINT `name`
    /// failed ``). ⚠️ Unbounded, `l2_pair_decision_current` is a substring of
    /// `l2_pair_decision_current_value`, so the needle could not tell the two refusals apart — found by
    /// mutation M6 contradicting its prediction, not by reading.
    fn refused_by(result: Result<(), String>, constraint: &str) {
        let error = result.expect_err("the schema must refuse this row");
        let needle = format!("`{constraint}`");
        assert!(error.contains(&needle), "{needle} in {error}");
    }

    /// The control: a valid ambiguity row is accepted — so each refusal below is about its one column.
    #[tokio::test]
    async fn a_valid_row_is_accepted() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        raw(
            &pool,
            LOW,
            HIGH,
            ("abstained", None, Some("ambiguous")),
            OPEN_END,
            Some(1),
        )
        .await
        .expect("valid");
    }

    /// 🔴 `identity_link`'s UNKNOWN defect, NOT inherited: a current `valid_to` with a NULL marker is
    /// refused by name, where `identity_link` accepts the same shape (registered since story 5.14).
    #[tokio::test]
    async fn a_current_interval_without_its_marker_is_refused() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        refused_by(
            raw(
                &pool,
                LOW,
                HIGH,
                ("abstained", None, Some("ambiguous")),
                OPEN_END,
                None,
            )
            .await,
            "l2_pair_decision_current",
        );
        // And the mirror: a closed interval still marked current.
        refused_by(
            raw(
                &pool,
                LOW,
                HIGH,
                ("abstained", None, Some("ambiguous")),
                "2026-02-01 00:00:00",
                Some(1),
            )
            .await,
            "l2_pair_decision_current",
        );
    }

    #[tokio::test]
    async fn a_marker_other_than_one_is_refused() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        refused_by(
            raw(
                &pool,
                LOW,
                HIGH,
                ("abstained", None, Some("ambiguous")),
                OPEN_END,
                Some(2),
            )
            .await,
            "l2_pair_decision_current_value",
        );
    }

    #[tokio::test]
    async fn an_unordered_pair_is_refused() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        refused_by(
            raw(
                &pool,
                HIGH,
                LOW,
                ("abstained", None, Some("ambiguous")),
                OPEN_END,
                Some(1),
            )
            .await,
            "l2_pair_decision_ordered",
        );
    }

    /// `match` and `absence_of_proof` are refused by the schema as well as by the adapter.
    #[tokio::test]
    async fn a_match_or_an_absence_of_proof_is_refused_by_the_schema() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        refused_by(
            raw(
                &pool,
                LOW,
                HIGH,
                ("match", Some("l2-x"), None),
                OPEN_END,
                Some(1),
            )
            .await,
            "l2_pair_decision_outcome",
        );
        refused_by(
            raw(
                &pool,
                LOW,
                HIGH,
                ("abstained", None, Some("absence_of_proof")),
                OPEN_END,
                Some(1),
            )
            .await,
            "l2_pair_decision_rule_xor_cause",
        );
        refused_by(
            raw(
                &pool,
                LOW,
                HIGH,
                ("no_match", Some(""), None),
                OPEN_END,
                Some(1),
            )
            .await,
            "l2_pair_decision_rule_xor_cause",
        );
    }

    /// One CURRENT decision per pair; a CLOSED one does not occupy the slot.
    #[tokio::test]
    async fn one_current_decision_per_pair_and_history_does_not_count() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        raw(
            &pool,
            LOW,
            HIGH,
            ("abstained", None, Some("ambiguous")),
            OPEN_END,
            Some(1),
        )
        .await
        .expect("first");
        let second = raw(
            &pool,
            LOW,
            HIGH,
            ("abstained", None, Some("ambiguous")),
            OPEN_END,
            Some(1),
        )
        .await;
        assert!(second.expect_err("duplicate").contains("Duplicate"));
        raw(
            &pool,
            LOW,
            HIGH,
            ("abstained", None, Some("ambiguous")),
            "2026-02-01 00:00:00",
            None,
        )
        .await
        .expect("a closed version beside the current one");
    }

    /// The adapter's own refusals, before any SQL: an unpersisted decision and an unordered pair.
    #[tokio::test]
    async fn the_writer_refuses_what_the_table_would_refuse() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let low = InterfaceId::from_uuid(uuid::Uuid::parse_str(LOW).expect("uuid"));
        let high = InterfaceId::from_uuid(uuid::Uuid::parse_str(HIGH).expect("uuid"));
        let absence = decide(vec![], RulesetVersion(1));
        assert_eq!(
            insert_l2_decision(
                &pool,
                uuid::Uuid::now_v7(),
                low,
                high,
                &absence,
                DecidedBy::Engine,
                at(100)
            )
            .await,
            Err(RepositoryError::Constraint(
                "l2_absence_of_proof_not_persisted"
            ))
        );
        assert_eq!(
            insert_l2_decision(
                &pool,
                uuid::Uuid::now_v7(),
                high,
                low,
                &ambiguous(),
                DecidedBy::Engine,
                at(100)
            )
            .await,
            Err(RepositoryError::Constraint("l2_pair_decision_ordered"))
        );
        insert_l2_decision(
            &pool,
            uuid::Uuid::now_v7(),
            low,
            high,
            &ambiguous(),
            DecidedBy::Engine,
            at(100),
        )
        .await
        .expect("valid");
        let current = load_current_l2_decision(&pool, low, high)
            .await
            .expect("read")
            .expect("current");
        assert!(current.carries(&ambiguous()), "{current:?}");
        assert_eq!(current.valid_from, "1970-01-01 00:01:40.000000");
    }

    /// Closing: never AT the sentinel (it would still read as current), and never an unknown id.
    #[tokio::test]
    async fn closing_refuses_the_sentinel_and_an_unknown_row() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let low = InterfaceId::from_uuid(uuid::Uuid::parse_str(LOW).expect("uuid"));
        let high = InterfaceId::from_uuid(uuid::Uuid::parse_str(HIGH).expect("uuid"));
        insert_l2_decision(
            &pool,
            uuid::Uuid::now_v7(),
            low,
            high,
            &ambiguous(),
            DecidedBy::Engine,
            at(100),
        )
        .await
        .expect("valid");
        let current = load_current_l2_decision(&pool, low, high)
            .await
            .expect("read")
            .expect("current");
        assert_eq!(
            close_l2_decision(&pool, &current.id, crate::repo::open_end()).await,
            Err(RepositoryError::Constraint("check"))
        );
        assert_eq!(
            close_l2_decision(&pool, "no-such-row", at(200)).await,
            Err(RepositoryError::NotFound)
        );
        close_l2_decision(&pool, &current.id, at(200))
            .await
            .expect("close");
        assert_eq!(
            close_l2_decision(&pool, &current.id, at(300)).await,
            Err(RepositoryError::NotFound),
            "a closed version cannot be closed again"
        );
    }
}
