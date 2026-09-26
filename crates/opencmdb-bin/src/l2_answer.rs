//! The operator's answer to an L2 question — *the same machine* or *distinct machines* (story 6.14b).
//!
//! Story 6.14 put the question on `/triage`: one Ambigu row per GROUP of interfaces the engine would
//! not call one machine or several. This module is the gesture that lifts the doubt, the glossary's
//! `resolve`/*résoudre*: one POST, one transaction, one OPERATOR row per question-pair.
//!
//! # What an answer writes — Guy's arbitrations of 2026-09-25 (story 6.14b, §0.8 and §0.11)
//!
//! - **The pairs that ARE the question, and none elsewhere** (§0.11(1a)): every current ENGINE
//!   `abstained`/`ambiguous` row among the group's interfaces is CLOSED at the instant the operator was
//!   shown, and one OPERATOR row is opened on the same pair. A pair with no row and an ENGINE `no_match`
//!   are not written — the second would override a verdict the software took (case one of Guy's
//!   taxonomy). ⚠️ A chain's `match` rows can still join, by union-find in story 6.14c, two interfaces an
//!   ENGINE `no_match` separates; that is 6.14c's question, registered, and not decided here.
//! - **The row's shape** (§0.8 B): `rule_id = 'operator'`, no cause, and the closed row's verdict vector
//!   and ruleset version COPIED — the evidence the operator was shown is kept with the answer. No
//!   `Decision` is forged: an answer is not a `decide` output (story 5.9b refused the struct literal).
//! - **The instant is the one the operator was SHOWN** (§0.8 C1): the newest freshness of the group's
//!   candidates, carried by the form. It is the house rule for derived instants and it doubles as the
//!   stale-page guard, which a clock could not give. Bounded on BOTH sides: an ENGINE row reached LATER
//!   than it means the question changed under the page (409), and an instant later than anything the
//!   store shows is forged or garbled (422) — a forged `9999-12-31 23:59:59.999998` otherwise passes
//!   the stale check and the interval CHECK alike (measured by PR #219's review).
//! - **No switch** (§0.8 F1): an answer writes an identity INPUT, never a declared value, so the
//!   authorship hazard `OPENCMDB_DOCUMENT_ENABLED` guards is absent here.
//!
//! # 🔴 What is locked, and why not more
//!
//! A PLAIN read names the question and recomputes its group; then ONLY the question's own pairs are
//! locked, one by one in `(low, high)` order, through the unique key, and re-verified (Guy, 2026-09-26, PR
//! #220's review). The first version read every open question `FOR UPDATE`: `EXPLAIN` gave `type=ALL`,
//! and an answer on one question was MEASURED waiting on — and deadlocking with — a sweep holding
//! another question's row, with the SWEEP rolled back as the victim. Under REPEATABLE READ a plain read
//! returns a snapshot, which is why the pairs themselves are read `FOR UPDATE`: that reads the latest
//! committed row and holds it until commit.
//!
//! ⚠️ **What serialises two answers to ONE question is the lock on its pairs AND the close that follows**
//! — two carriers, measured by mutation M2 (dropping the lock left the second answer refused anyway, its
//! close finding no row). And a sweep and an answer that lock the SAME pairs in opposite orders can still
//! deadlock; that residual is registered, and an answer that loses one says *nothing was written*.
//!
//! # The state holds a PORT and no pool
//!
//! [`AnswerState`] carries `Arc<dyn AnswerPort>`, so a handler cannot extract `State<MySqlPool>`
//! (story 6.1's M4 carrier, on `document.rs`'s and `ipam_write.rs`'s precedent).

use std::collections::BTreeSet;
use std::sync::Arc;

use axum::extract::State;
use axum::extract::rejection::FormRejection;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Form, Router};
use opencmdb_core::observation::Timestamp;
use opencmdb_core::repo::{BoxFuture, RepositoryError};
use serde::Deserialize;
use sqlx::{MySqlConnection, MySqlPool};

use crate::ipam_write::Refusal;
use crate::repo::{OPEN_END, classify, datetime_literal};

/// Where an answer posts. A constant of the product, never operator input.
pub(crate) const ANSWER_PATH: &str = "/triage/answer";

/// Every path this sub-router carries, and the list [`router_with`] is BUILT from — so the auth
/// route-table test in `main.rs` walks what is mounted (story 14.2b's AC3 shape).
pub(crate) const PATHS: &[&str] = &[ANSWER_PATH];

/// The token an operator's answer writes in `rule_id` — outside both `l1-` and `l2-`, so no trap
/// runner can mistake it for a rule, and refused on an ENGINE row by `0013`.
pub(crate) const OPERATOR_RULE: &str = "operator";

/// The two answers of the one gesture, `resolve`/*résoudre*.
///
/// ⚠️ **Two labels of ONE gesture, not two gestures** (story 6.14b §0.3): neither is `attach` — no
/// device record exists to attach to, and the answer is symmetric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Answer {
    /// The candidates are one machine — an OPERATOR `match` on each question-pair.
    SameMachine,
    /// No candidate is the other — an OPERATOR `no_match` on each question-pair.
    DistinctMachines,
}

impl Answer {
    /// Both answers, in the order the bar shows them.
    pub(crate) const ALL: [Answer; 2] = [Answer::SameMachine, Answer::DistinctMachines];

    /// The form's spelling — what a button posts and what `?answered=` carries back.
    pub(crate) const fn token(self) -> &'static str {
        match self {
            Answer::SameMachine => "same",
            Answer::DistinctMachines => "distinct",
        }
    }

    /// The outcome it writes in `l2_pair_decision`.
    pub(crate) const fn outcome(self) -> &'static str {
        match self {
            Answer::SameMachine => "match",
            Answer::DistinctMachines => "no_match",
        }
    }

    /// The answer a form value names, if any. Exact: a typo is not an answer.
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|answer| answer.token() == raw)
    }

    /// The answer an OUTCOME records, for reading an answer back.
    pub(crate) fn of_outcome(outcome: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|answer| answer.outcome() == outcome)
    }

    /// The key of its label on the bar.
    pub(crate) const fn label_key(self) -> &'static str {
        match self {
            Answer::SameMachine => "triage.answer.same",
            Answer::DistinctMachines => "triage.answer.distinct",
        }
    }
}

/// What a recorded answer wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Answered {
    /// The answer.
    pub(crate) answer: Answer,
    /// How many question-pairs it closed and answered.
    pub(crate) pairs: usize,
}

/// Why an answer was not recorded. Each maps to ONE keyed refusal at the route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AnswerRefused {
    /// An ENGINE row of the question was reached LATER than the instant the operator was shown: the
    /// question changed under the page. 409.
    Stale,
    /// The group the store holds now is not the group the page showed — grown OR shrunk. 409.
    GroupChanged,
    /// No open question holds these interfaces: already answered, gone, or never asked. Also what a
    /// close finding no row and an insert refused by the one-current key mean inside the transaction —
    /// a concurrent answer or sweep got there first. 409.
    NotOpen,
    /// The instant is later than anything the store shows for the group: forged or garbled. 422.
    Forged,
    /// *The same machine* on a group holding a pair the operator already answered *distinct machines*
    /// — it would make them one by transitivity (Guy, 2026-09-26, PR #220's review). 409.
    Contradicts,
    /// The store chose this answer as a deadlock victim, or gave up waiting on a lock: the transaction
    /// was rolled back, so nothing was written — unlike the budget's timeout, which cannot say. 503.
    Contended,
    /// The store failed.
    Store(RepositoryError),
}

impl From<RepositoryError> for AnswerRefused {
    fn from(error: RepositoryError) -> Self {
        match error {
            // 🔴 Both are a concurrent writer getting there first, and the operator's sentence is the
            // same: the question is no longer open. Never a 500 (PR #219's review, edge layer).
            RepositoryError::NotFound | RepositoryError::Constraint("unique") => Self::NotOpen,
            RepositoryError::Contention => Self::Contended,
            other => Self::Store(other),
        }
    }
}

/// One current ENGINE question-pair, as the PLAIN read that names the question returns it:
/// `(interface_low, interface_high, id, verdicts, ruleset_version, valid_from)`.
type QuestionRow = (String, String, String, String, u32, String);

/// One pair's CURRENT row as the locking read returns it:
/// `(id, decided_by, outcome, abstention_cause, verdicts, ruleset_version, valid_from)`.
type LockedRow = (String, String, String, Option<String>, String, u32, String);

/// Record an answer on the group `members`, as the operator was shown it at `shown`.
///
/// Must run inside a transaction the caller commits: the question's pairs it locks it holds until then,
/// and every row it writes is the answer's.
///
/// # Errors
///
/// An [`AnswerRefused`] naming the rule broken; nothing is written on any of them once the caller rolls
/// back.
pub(crate) async fn record_operator_answer(
    conn: &mut MySqlConnection,
    members: &BTreeSet<String>,
    answer: Answer,
    shown: Timestamp,
) -> Result<Answered, AnswerRefused> {
    // 🔑 A PLAIN read NAMES the question: every open ENGINE question, from which the group is recomputed
    // by the reader the screen uses — never trusted from the form. It locks nothing (Guy, 2026-09-26, PR
    // #220's review: a locking read here scanned the whole table, `type=ALL`, and an answer on one
    // question waited on — and was MEASURED deadlocking with — a sweep holding another question's row,
    // the sweep being the victim). The question's own pairs are locked below, one by one.
    let rows: Vec<QuestionRow> = sqlx::query_as(
        "SELECT interface_low, interface_high, id, verdicts, ruleset_version, \
         DATE_FORMAT(valid_from, '%Y-%m-%d %H:%i:%s.%f') \
         FROM l2_pair_decision WHERE is_current = 1 AND decided_by = 'ENGINE' \
         AND outcome = 'abstained' AND abstention_cause = 'ambiguous'",
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(classify)?;
    let pairs: Vec<(String, String, String)> = rows
        .iter()
        .map(|(low, high, _, verdicts, _, _)| (low.clone(), high.clone(), verdicts.clone()))
        .collect();
    let Some(group) = crate::ambiguity_view::groups(&pairs)
        .into_iter()
        .find(|group| group.interfaces.iter().any(|id| members.contains(id)))
    else {
        return Err(AnswerRefused::NotOpen);
    };
    let now: BTreeSet<String> = group.interfaces.iter().cloned().collect();
    if &now != members {
        return Err(AnswerRefused::GroupChanged);
    }

    // 🔴 Guy, 2026-09-26 (PR #220's review): *the same machine* on a group holding a pair the operator
    // already answered DISTINCT would make those two one by transitivity — refused, and the pane does
    // not offer it.
    if answer == Answer::SameMachine {
        let distinct: Vec<(String, String)> = sqlx::query_as(
            "SELECT interface_low, interface_high FROM l2_pair_decision \
             WHERE is_current = 1 AND decided_by = 'OPERATOR' AND outcome = 'no_match'",
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(classify)?;
        if distinct
            .iter()
            .any(|(low, high)| members.contains(low) && members.contains(high))
        {
            return Err(AnswerRefused::Contradicts);
        }
    }

    let shown_literal = datetime_literal(shown);
    let newest = newest_placement(conn, members).await?;
    match newest {
        // The page offers no answer on a group with no current placement (AC5), so a POST for one
        // arrived after the placement went: the question changed under the page.
        None => return Err(AnswerRefused::Stale),
        Some(newest) if shown_literal > newest => return Err(AnswerRefused::Forged),
        Some(_) => {}
    }

    // The pairs that ARE the question — both ends in the group — in `(low, high)` order, the order every
    // answer locks them in.
    let mut question: Vec<&QuestionRow> = rows
        .iter()
        .filter(|(low, high, ..)| members.contains(low) && members.contains(high))
        .collect();
    question.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    // 🔑 LOCK the question's pairs and nothing else: a lookup by the unique key's prefix, `FOR UPDATE`,
    // which reads the LATEST committed row (under REPEATABLE READ a plain read would return the snapshot
    // taken above). A row that is no longer the one the question was named on means someone got there
    // first: an operator (not open), or a sweep that re-judged the pair (the question changed).
    let mut locked: Vec<(&QuestionRow, LockedRow)> = Vec::with_capacity(question.len());
    for row in &question {
        let current: Option<LockedRow> = sqlx::query_as(
            "SELECT id, decided_by, outcome, abstention_cause, verdicts, ruleset_version, \
             DATE_FORMAT(valid_from, '%Y-%m-%d %H:%i:%s.%f') FROM l2_pair_decision \
             WHERE interface_low = ? AND interface_high = ? AND is_current = 1 FOR UPDATE",
        )
        .bind(&row.0)
        .bind(&row.1)
        .fetch_optional(&mut *conn)
        .await
        .map_err(classify)?;
        match current {
            Some(current) if current.0 == row.2 => locked.push((row, current)),
            Some(current)
                if current.1 == "ENGINE"
                    && current.2 == "abstained"
                    && current.3.as_deref() == Some("ambiguous") =>
            {
                return Err(AnswerRefused::Stale);
            }
            _ => return Err(AnswerRefused::NotOpen),
        }
    }
    if locked.iter().any(|(_, current)| current.6 > shown_literal) {
        return Err(AnswerRefused::Stale);
    }
    for ((low, high, ..), (id, _, _, _, verdicts, ruleset_version, _)) in &locked {
        crate::l2_repo::close_l2_decision(&mut *conn, id, shown).await?;
        sqlx::query(
            "INSERT INTO l2_pair_decision \
             (id, interface_low, interface_high, outcome, rule_id, abstention_cause, verdicts, \
              ruleset_version, decided_by, valid_from, valid_to, is_current) \
             VALUES (?, ?, ?, ?, ?, NULL, ?, ?, 'OPERATOR', ?, ?, 1)",
        )
        .bind(uuid::Uuid::now_v7().to_string())
        .bind(low)
        .bind(high)
        .bind(answer.outcome())
        .bind(OPERATOR_RULE)
        .bind(verdicts)
        .bind(ruleset_version)
        .bind(&shown_literal)
        .bind(OPEN_END)
        .execute(&mut *conn)
        .await
        .map_err(classify)?;
    }
    Ok(Answered {
        answer,
        pairs: locked.len(),
    })
}

/// The newest `observed_at` among the members' CURRENT placements, rendered — the instant the page
/// shows as the group's freshness, recomputed from the store. `None` when no member is placed.
///
/// ⚠️ One query per member rather than an `IN` list assembled at runtime, which is a query nobody can
/// read in the source (`l2_repo`'s batch read gives the same reason); a group is two to a handful of
/// interfaces.
async fn newest_placement(
    conn: &mut MySqlConnection,
    members: &BTreeSet<String>,
) -> Result<Option<String>, AnswerRefused> {
    let mut newest: Option<String> = None;
    for member in members {
        let latest: Option<String> = sqlx::query_scalar(
            "SELECT DATE_FORMAT(MAX(o.observed_at), '%Y-%m-%d %H:%i:%s.%f') \
             FROM identity_link l JOIN observation_record o ON o.id = l.observation_id \
             WHERE l.interface_id = ? AND l.outcome = 'match' AND l.valid_to = ?",
        )
        .bind(member)
        .bind(OPEN_END)
        .fetch_one(&mut *conn)
        .await
        .map_err(classify)?;
        newest = newest.max(latest);
    }
    Ok(newest)
}

/// The answer as the route hands it to the store.
pub(crate) trait AnswerPort: Send + Sync {
    /// Record `answer` on the group `members`, shown at `shown`, in one transaction.
    ///
    /// # Errors
    ///
    /// An [`AnswerRefused`]; nothing is written.
    fn answer(
        &self,
        members: BTreeSet<String>,
        answer: Answer,
        shown: Timestamp,
    ) -> BoxFuture<'_, Result<Answered, AnswerRefused>>;
}

/// The production wiring: the answer over a MariaDB pool, which lives HERE and not on the state.
pub(crate) struct StoreAnswer {
    /// The pool, unreachable from the handler by type.
    pool: MySqlPool,
}

impl AnswerPort for StoreAnswer {
    fn answer(
        &self,
        members: BTreeSet<String>,
        answer: Answer,
        shown: Timestamp,
    ) -> BoxFuture<'_, Result<Answered, AnswerRefused>> {
        Box::pin(async move {
            let mut conn = self.pool.acquire().await.map_err(classify)?;
            let mut tx = sqlx::Connection::begin(&mut *conn)
                .await
                .map_err(classify)?;
            match record_operator_answer(&mut tx, &members, answer, shown).await {
                Ok(answered) => {
                    tx.commit().await.map_err(classify)?;
                    Ok(answered)
                }
                Err(refused) => {
                    if let Err(error) = tx.rollback().await {
                        tracing::warn!(%error, "rolling back a refused answer failed");
                    }
                    Err(refused)
                }
            }
        })
    }
}

/// The sub-router's state — deliberately NO pool field.
#[derive(Clone)]
pub(crate) struct AnswerState {
    /// The gesture. A port, never a pool.
    port: Arc<dyn AnswerPort>,
}

/// The production sub-router, wired to the store-backed port.
pub(crate) fn router(pool: MySqlPool) -> Router {
    router_with(Arc::new(StoreAnswer { pool }))
}

/// The sub-router over an explicit port — the seam tests drive the route through.
pub(crate) fn router_with(port: Arc<dyn AnswerPort>) -> Router {
    let mut router = Router::new();
    for path in PATHS {
        router = router.route(path, post(answer_question));
    }
    router.with_state(AnswerState { port })
}

/// The request: what the answer's button posts through `hx-vals`.
#[derive(Debug, Deserialize)]
pub(crate) struct AnswerRequest {
    /// The queue row's id, `ambigu:{smallest member}`.
    group: String,
    /// The member interface ids, comma-separated.
    members: String,
    /// `same` or `distinct`.
    answer: String,
    /// The instant the page showed as the group's freshness, RFC 3339 with microseconds.
    shown: String,
}

/// How long the answer may hold the browser — the figure every write and screen here uses, for the
/// same want of a measurement justifying another ([`crate::page::PAGE_STORE_BUDGET`]).
const ANSWER_BUDGET: std::time::Duration = crate::page::PAGE_STORE_BUDGET;

/// `POST /triage/answer` — the operator answers an L2 question.
///
/// The CSRF check is decided first (story 6.2 §5): a cross-site request is refused before anything the
/// form says is consulted.
async fn answer_question(
    State(state): State<AnswerState>,
    headers: HeaderMap,
    form: Result<Form<AnswerRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return Refusal::new(StatusCode::FORBIDDEN, "triage.answer.refused.cross_site")
            .into_response();
    }
    let Some((members, answer, shown)) = form.ok().and_then(|Form(request)| parse(&request)) else {
        return Refusal::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "triage.answer.refused.malformed",
        )
        .into_response();
    };
    let work = state.port.answer(members, answer, shown);
    let outcome = match tokio::time::timeout(ANSWER_BUDGET, work).await {
        Ok(outcome) => outcome,
        Err(_elapsed) => {
            tracing::error!("an answer did not finish within its budget");
            return Refusal::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "triage.answer.refused.busy",
            )
            .into_response();
        }
    };
    match outcome {
        Ok(answered) => {
            tracing::info!(
                answer = answered.answer.token(),
                pairs = answered.pairs,
                "an L2 question was answered"
            );
            (
                StatusCode::CREATED,
                [(
                    axum::http::HeaderName::from_static("hx-redirect"),
                    format!("/triage?answered={}", answered.answer.token()),
                )],
                rust_i18n::t!(done_key(answered.answer)).to_string(),
            )
                .into_response()
        }
        Err(refused) => refusal(&refused).into_response(),
    }
}

/// The confirmation key for an answer — shown after the redirect, from `?answered=`.
pub(crate) const fn done_key(answer: Answer) -> &'static str {
    match answer {
        Answer::SameMachine => "triage.answered.same",
        Answer::DistinctMachines => "triage.answered.distinct",
    }
}

/// The keyed refusal each [`AnswerRefused`] earns — exhaustive, so a new cause cannot ship without a
/// sentence.
pub(crate) fn refusal(refused: &AnswerRefused) -> Refusal {
    match refused {
        AnswerRefused::Stale => Refusal::new(StatusCode::CONFLICT, "triage.answer.refused.stale"),
        AnswerRefused::GroupChanged => {
            Refusal::new(StatusCode::CONFLICT, "triage.answer.refused.changed")
        }
        AnswerRefused::NotOpen => {
            Refusal::new(StatusCode::CONFLICT, "triage.answer.refused.not_open")
        }
        AnswerRefused::Contradicts => {
            Refusal::new(StatusCode::CONFLICT, "triage.answer.refused.contradicts")
        }
        AnswerRefused::Contended => Refusal::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "triage.answer.refused.contended",
        ),
        AnswerRefused::Forged => Refusal::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "triage.answer.refused.malformed",
        ),
        AnswerRefused::Store(error) => {
            tracing::error!(%error, "an answer failed at the backend");
            Refusal::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "triage.answer.refused.store",
            )
        }
    }
}

/// The form, parsed — or `None`, which is the malformed refusal.
///
/// The members are UUIDs, never the nil sentinel, at least two, and the group id must be the row id the
/// screen mints from them (`ambigu:{smallest}`), so a form stitched from two questions is not a form.
fn parse(request: &AnswerRequest) -> Option<(BTreeSet<String>, Answer, Timestamp)> {
    let mut members = BTreeSet::new();
    for raw in request.members.split(',') {
        let id = uuid::Uuid::parse_str(raw.trim()).ok()?;
        if id.is_nil() {
            return None;
        }
        members.insert(id.to_string());
    }
    if members.len() < 2 {
        return None;
    }
    let smallest = members.iter().next()?;
    // Compared case-insensitively, as the members are (they are normalised by `Uuid::to_string`).
    if !request
        .group
        .trim()
        .eq_ignore_ascii_case(&format!("ambigu:{smallest}"))
    {
        return None;
    }
    let answer = Answer::parse(request.answer.trim())?;
    let shown = chrono::DateTime::parse_from_rfc3339(request.shown.trim())
        .ok()?
        .with_timezone(&chrono::Utc);
    // 🔴 chrono ACCEPTS a leap second (`…23:59:60.5Z`) and represents it with nanoseconds ≥ 10⁹; MariaDB
    // refuses the rendered `:60` and the answer answered 500 (PR #220's review, edge layer, measured).
    // The page never renders one, so it is a malformed form.
    if shown.timestamp_subsec_nanos() >= 1_000_000_000 {
        return None;
    }
    Some((members, answer, shown))
}

/// Tests for the answer: the adapter against a store, the route over a port.
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, header};
    use tower::ServiceExt;

    // ---- pure ----

    #[test]
    fn the_two_answers_have_distinct_tokens_outcomes_and_labels() {
        let tokens: BTreeSet<&str> = Answer::ALL.iter().map(|a| a.token()).collect();
        let outcomes: BTreeSet<&str> = Answer::ALL.iter().map(|a| a.outcome()).collect();
        let labels: BTreeSet<&str> = Answer::ALL.iter().map(|a| a.label_key()).collect();
        assert_eq!((tokens.len(), outcomes.len(), labels.len()), (2, 2, 2));
        for answer in Answer::ALL {
            assert_eq!(Answer::parse(answer.token()), Some(answer));
            assert_eq!(Answer::of_outcome(answer.outcome()), Some(answer));
        }
        assert_eq!(Answer::parse("Same"), None, "a typo is not an answer");
    }

    const A: &str = "00000000-0000-0000-0000-0000000000b1";
    const B: &str = "00000000-0000-0000-0000-0000000000b2";

    fn request(group: &str, members: &str, answer: &str, shown: &str) -> AnswerRequest {
        AnswerRequest {
            group: group.to_string(),
            members: members.to_string(),
            answer: answer.to_string(),
            shown: shown.to_string(),
        }
    }

    #[test]
    fn a_form_is_read_only_when_every_part_is_the_screens() {
        // PR #220's review: an uppercase group is the same group, and a leap second is no instant.
        let upper = request(
            &format!("AMBIGU:{}", A.to_uppercase()),
            &format!("{},{B}", A.to_uppercase()),
            "same",
            "2026-09-26T10:00:00Z",
        );
        assert!(parse(&upper).is_some(), "case is not a different group");
        let leap = request(
            &format!("ambigu:{A}"),
            &format!("{A},{B}"),
            "same",
            "2026-09-26T23:59:60.5Z",
        );
        assert!(
            parse(&leap).is_none(),
            "a leap second is refused as malformed, never a 500"
        );
        let shown = "2026-09-26T10:00:00.123456Z";
        let good = request(&format!("ambigu:{A}"), &format!("{B},{A}"), "same", shown);
        let (members, answer, at) = parse(&good).expect("the screen's own form");
        assert_eq!(members.len(), 2);
        assert_eq!(answer, Answer::SameMachine);
        assert_eq!(
            datetime_literal(at),
            "2026-09-26 10:00:00.123456",
            "microseconds survive"
        );
        for (why, bad) in [
            (
                "a group id not the smallest member",
                request(&format!("ambigu:{B}"), &format!("{A},{B}"), "same", shown),
            ),
            (
                "one member is no question",
                request(&format!("ambigu:{A}"), A, "same", shown),
            ),
            (
                "the nil sentinel",
                request(
                    "ambigu:00000000-0000-0000-0000-000000000000",
                    &format!("00000000-0000-0000-0000-000000000000,{A}"),
                    "same",
                    shown,
                ),
            ),
            (
                "not a uuid",
                request(&format!("ambigu:{A}"), &format!("{A},x"), "same", shown),
            ),
            (
                "no such answer",
                request(&format!("ambigu:{A}"), &format!("{A},{B}"), "merge", shown),
            ),
            (
                "no instant",
                request(&format!("ambigu:{A}"), &format!("{A},{B}"), "same", ""),
            ),
        ] {
            assert!(parse(&bad).is_none(), "{why}");
        }
    }

    /// Every cause earns its own key and the status the story names (AC5); a lost race is a 409.
    #[test]
    fn every_refusal_is_keyed_and_a_lost_race_is_not_a_500() {
        let cases = [
            (AnswerRefused::Stale, 409, "triage.answer.refused.stale"),
            (
                AnswerRefused::GroupChanged,
                409,
                "triage.answer.refused.changed",
            ),
            (
                AnswerRefused::NotOpen,
                409,
                "triage.answer.refused.not_open",
            ),
            (
                AnswerRefused::Forged,
                422,
                "triage.answer.refused.malformed",
            ),
            (
                AnswerRefused::from(RepositoryError::NotFound),
                409,
                "triage.answer.refused.not_open",
            ),
            (
                AnswerRefused::from(RepositoryError::Constraint("unique")),
                409,
                "triage.answer.refused.not_open",
            ),
            (
                AnswerRefused::from(RepositoryError::Contention),
                503,
                "triage.answer.refused.contended",
            ),
            (
                AnswerRefused::from(RepositoryError::Backend("x".into())),
                500,
                "triage.answer.refused.store",
            ),
        ];
        for (refused, status, key) in cases {
            let refusal = refusal(&refused);
            assert_eq!(
                (refusal.status().as_u16(), refusal.key()),
                (status, key),
                "{refused:?}"
            );
            for locale in ["en", "fr"] {
                let text = rust_i18n::t!(key, locale = locale).to_string();
                assert!(!text.trim().is_empty() && text != key, "{key} in {locale}");
            }
        }
    }

    // ---- the route over a port ----

    struct Fixed(fn() -> Result<Answered, AnswerRefused>);
    impl AnswerPort for Fixed {
        fn answer(
            &self,
            _: BTreeSet<String>,
            _: Answer,
            _: Timestamp,
        ) -> BoxFuture<'_, Result<Answered, AnswerRefused>> {
            let outcome = (self.0)();
            Box::pin(async move { outcome })
        }
    }

    fn body() -> String {
        format!(
            "group=ambigu%3A{A}&members={A}%2C{B}&answer=distinct&shown=2026-09-26T10%3A00%3A00.000001Z"
        )
    }

    async fn post(port: Fixed, origin: &str, body: String) -> Response {
        router_with(Arc::new(port))
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(ANSWER_PATH)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header(header::HOST, "nas:8080")
                    .header(header::ORIGIN, origin)
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn an_answer_redirects_to_the_queue_carrying_its_confirmation() {
        let response = post(
            Fixed(|| {
                Ok(Answered {
                    answer: Answer::DistinctMachines,
                    pairs: 1,
                })
            }),
            "http://nas:8080",
            body(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            response
                .headers()
                .get("hx-redirect")
                .and_then(|v| v.to_str().ok()),
            Some("/triage?answered=distinct")
        );
    }

    #[tokio::test]
    async fn a_cross_site_answer_is_refused_before_the_store() {
        let response = post(
            Fixed(|| unreachable!("never asked")),
            "http://evil.example",
            body(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let response = post(
            Fixed(|| unreachable!("never asked")),
            "http://nas:8080",
            "x=1".into(),
        )
        .await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    // ---- the adapter against a store ----

    /// A store holding one question between two placed interfaces, `obelix`'s shape, built through the
    /// shipped path — the resolver — so the rows are the engine's own.
    pub(crate) mod fixture {
        use opencmdb_core::observation::{
            ConnectorId, Fact, HostnameSource, L2DomainId, MacAddr, ObsId, Observation, Scope,
            Timestamp, VantageId,
        };
        use opencmdb_core::repo::WriteRepository;
        use sqlx::MySqlPool;

        pub(crate) fn at_micros(micros: i64) -> Timestamp {
            chrono::DateTime::from_timestamp_micros(micros).expect("in range")
        }

        /// One sighting of NIC `last`, answering to `name`, at `micros` — the shipped connector's shape
        /// since PR #163 (`l2_pass`'s test helper, one MAC per observation).
        pub(crate) fn sighting(id: u128, last: u8, name: Option<&str>, micros: i64) -> Observation {
            let mut facts = vec![Fact::Mac {
                addr: MacAddr([0x00, 0x11, 0x22, 0x33, 0x55, last]),
                locally_administered: false,
            }];
            if let Some(name) = name {
                facts.push(Fact::Hostname {
                    name: name.to_string(),
                    source: HostnameSource::Dns,
                });
            }
            Observation {
                obs_id: ObsId::from_uuid(uuid::Uuid::from_u128(0x6140_0000 + id)),
                connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()),
                observed_at: at_micros(micros),
                scope: Scope {
                    l2_domain: L2DomainId::from_uuid(uuid::Uuid::from_u128(1)),
                    vantage: VantageId::from_uuid(uuid::Uuid::nil()),
                },
                facts,
                raw: None,
            }
        }

        /// A migrated, emptied store, or `None` without `DATABASE_URL`.
        pub(crate) async fn store() -> Option<MySqlPool> {
            let Ok(url) = std::env::var("DATABASE_URL") else {
                eprintln!("skipping l2_answer test: DATABASE_URL unset");
                return None;
            };
            let pool = MySqlPool::connect(&url).await.expect("connect");
            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("migrate");
            for statement in [
                "DELETE FROM l2_pair_decision",
                "DELETE FROM link_candidate",
                "DELETE FROM identity_link",
                "DELETE FROM interface",
                "DELETE FROM observation_record",
            ] {
                sqlx::query(statement).execute(&pool).await.expect("clean");
            }
            Some(pool)
        }

        /// Ingest and resolve one sweep through the shipped path, in one transaction as `scan_pass` does.
        pub(crate) async fn sweep(pool: &MySqlPool, observations: Vec<Observation>) {
            for observation in &observations {
                crate::repo::insert_observation(pool, observation)
                    .await
                    .expect("insert observation");
            }
            crate::repo::MariaRepository::new(pool.clone())
                .transact(move |unit| {
                    let observations = observations.clone();
                    Box::pin(async move {
                        crate::resolver::resolve(unit.executor(), &observations).await
                    })
                })
                .await
                .expect("resolve");
        }

        /// The same sighting, answering at `address` too — so the triage queue has `Nouveau` rows.
        pub(crate) fn at_address(mut observation: Observation, address: &str) -> Observation {
            observation.facts.push(Fact::IpV4 {
                addr: address.parse().expect("an address"),
            });
            observation
        }

        /// As [`sweep`], returning the pass's counts.
        pub(crate) async fn counted_sweep(
            pool: &MySqlPool,
            observations: Vec<Observation>,
        ) -> crate::resolver::Resolution {
            for observation in &observations {
                crate::repo::insert_observation(pool, observation)
                    .await
                    .expect("insert observation");
            }
            crate::repo::MariaRepository::new(pool.clone())
                .transact(move |unit| {
                    let observations = observations.clone();
                    Box::pin(async move {
                        crate::resolver::resolve(unit.executor(), &observations).await
                    })
                })
                .await
                .expect("resolve")
        }

        /// The interface ids of the store, sorted.
        pub(crate) async fn interfaces(pool: &MySqlPool) -> Vec<String> {
            sqlx::query_scalar("SELECT id FROM interface ORDER BY id")
                .fetch_all(pool)
                .await
                .expect("read")
        }

        /// Every current row as `(low, high, outcome, rule_id, decided_by, verdicts, valid_from)`.
        pub(crate) async fn current(
            pool: &MySqlPool,
        ) -> Vec<(
            String,
            String,
            String,
            Option<String>,
            String,
            String,
            String,
        )> {
            sqlx::query_as(
                "SELECT interface_low, interface_high, outcome, rule_id, decided_by, verdicts, \
                 DATE_FORMAT(valid_from, '%Y-%m-%d %H:%i:%s.%f') \
                 FROM l2_pair_decision WHERE is_current = 1 ORDER BY interface_low, interface_high",
            )
            .fetch_all(pool)
            .await
            .expect("read")
        }
    }

    use fixture::*;

    /// Record one answer in its own committed transaction — what the port does.
    async fn answer_on(
        pool: &MySqlPool,
        members: &BTreeSet<String>,
        answer: Answer,
        shown: Timestamp,
    ) -> Result<Answered, AnswerRefused> {
        StoreAnswer { pool: pool.clone() }
            .answer(members.clone(), answer, shown)
            .await
    }

    /// AC2: an answer closes every ENGINE question-pair at the instant shown and opens one OPERATOR row
    /// per pair — copying the vector — and writes NO other pair. The chain's third pair carries an
    /// ENGINE `no_match` (a virtual-router address) and is asserted untouched.
    #[tokio::test]
    async fn an_answer_writes_the_question_and_nothing_else() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let t = 1_790_000_000_123_456;
        sweep(
            &pool,
            vec![
                sighting(1, 1, Some("obelix"), t),
                sighting(2, 2, Some("obelix"), t),
            ],
        )
        .await;
        let before = current(&pool).await;
        assert_eq!(before.len(), 1, "premise: one ENGINE question {before:?}");
        let members: BTreeSet<String> = interfaces(&pool).await.into_iter().collect();
        // A third interface with an ENGINE `no_match` against one member, written around the pass so
        // the chain's shape exists whatever the rules do.
        let third = "ffffffff-0000-0000-0000-000000000001";
        crate::repo::insert_interface(
            &pool,
            opencmdb_core::observation::InterfaceId::from_uuid(
                uuid::Uuid::parse_str(third).unwrap(),
            ),
            opencmdb_core::observation::L2DomainId::from_uuid(uuid::Uuid::from_u128(1)),
            &opencmdb_core::observation::MacAddr([0x00, 0x00, 0x5e, 0x00, 0x01, 0x0a]),
            at_micros(t),
            at_micros(t),
        )
        .await
        .expect("interface");
        let first = members.iter().next().unwrap().clone();
        sqlx::query(
            "INSERT INTO l2_pair_decision (id, interface_low, interface_high, outcome, rule_id, \
             abstention_cause, verdicts, ruleset_version, decided_by, valid_from, valid_to, is_current) \
             VALUES (UUID(), ?, ?, 'no_match', 'l2-virtual-mac-prefix', NULL, \
             'l2-virtual-mac-prefix=disqualifying', 1, 'ENGINE', '2026-01-01 00:00:00', ?, 1)",
        )
        .bind(&first)
        .bind(third)
        .bind(OPEN_END)
        .execute(&pool)
        .await
        .expect("the engine's refusal");

        let answered = answer_on(&pool, &members, Answer::SameMachine, at_micros(t))
            .await
            .expect("the answer");
        assert_eq!(answered.pairs, 1);
        let after = current(&pool).await;
        assert_eq!(after.len(), 2, "{after:?}");
        let operator: Vec<_> = after.iter().filter(|r| r.4 == "OPERATOR").collect();
        assert_eq!(operator.len(), 1, "one OPERATOR row per question-pair");
        let row = operator[0];
        assert_eq!(
            (row.2.as_str(), row.3.as_deref()),
            ("match", Some("operator"))
        );
        assert_eq!(
            row.5, before[0].5,
            "the vector the operator was shown is kept"
        );
        assert_eq!(
            row.6, "2026-09-21 14:13:20.123456",
            "valid_from is the instant shown"
        );
        let untouched = after.iter().find(|r| r.1 == third).expect("the no_match");
        assert_eq!(
            (untouched.2.as_str(), untouched.4.as_str()),
            ("no_match", "ENGINE"),
            "an ENGINE no_match is never overridden"
        );
        let closed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM l2_pair_decision WHERE decided_by = 'ENGINE' \
             AND outcome = 'abstained' AND is_current IS NULL AND valid_to = ?",
        )
        .bind("2026-09-21 14:13:20.123456")
        .fetch_one(&pool)
        .await
        .expect("read");
        assert_eq!(
            closed, 1,
            "the ENGINE row is CLOSED at the instant shown, never deleted"
        );
    }

    /// Story 6.14b, AC4 and AC7, end to end: the form is read off the view production renders, POSTED
    /// THROUGH THE ROUTE over the store-backed port, and then —
    /// - the question has left the queue, and each of its addresses offers *Ajouter* again (AC7);
    /// - later sweeps leave the answered pair alone: `operator_held` counts it, a second sweep writes
    ///   nothing, a sweep where a member loses its name vacates nothing, and the question never returns
    ///   (AC4).
    #[tokio::test]
    async fn an_answer_posted_through_the_route_settles_the_question() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        sqlx::query("DELETE FROM declared_attribute")
            .execute(&pool)
            .await
            .expect("clean");
        let t = 1_790_000_000_000_010;
        counted_sweep(
            &pool,
            vec![
                at_address(sighting(1, 1, Some("obelix"), t), "192.0.2.8"),
                at_address(sighting(2, 2, Some("obelix"), t), "192.0.2.9"),
            ],
        )
        .await;
        let (view, _) = crate::page::triage_view(&pool, None, false, true)
            .await
            .unwrap_or_else(|_| panic!("the view"));
        let group = view
            .rows
            .iter()
            .find(|r| r.id.starts_with("ambigu:"))
            .expect("the question is in the queue")
            .id
            .clone();
        let (view, _) = crate::page::triage_view(&pool, Some(&group), false, true)
            .await
            .unwrap_or_else(|_| panic!("the view"));
        let form = view
            .selected
            .and_then(|pane| pane.question)
            .expect("the pane carries the question's form");
        let body = format!(
            "group={}&members={}&answer=same&shown={}",
            crate::triage_view::url_escape(&form.group),
            crate::triage_view::url_escape(&form.members),
            crate::triage_view::url_escape(&form.shown),
        );
        let body_again = body.clone();
        let response = router(pool.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(ANSWER_PATH)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header(header::HOST, "nas:8080")
                    .header(header::ORIGIN, "http://nas:8080")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        // AC5, through the ROUTE (PR #220's review): the same answer posted again — a second tab, a
        // double press — is the keyed 409 *no longer open*, never a 500.
        let again = router(pool.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(ANSWER_PATH)
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header(header::HOST, "nas:8080")
                    .header(header::ORIGIN, "http://nas:8080")
                    .body(Body::from(body_again))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(again.status(), StatusCode::CONFLICT);

        // AC7: the question is gone, and *Ajouter* is back on each address.
        let (view, _) = crate::page::triage_view(&pool, None, false, true)
            .await
            .unwrap_or_else(|_| panic!("the view"));
        assert!(
            view.rows.iter().all(|r| !r.id.starts_with("ambigu:")),
            "question gone: {:?}",
            view.rows.iter().map(|r| &r.id).collect::<Vec<_>>()
        );
        for address in ["192.0.2.8", "192.0.2.9"] {
            let (view, _) =
                crate::page::triage_view(&pool, Some(&format!("nouveau:{address}")), false, true)
                    .await
                    .unwrap_or_else(|_| panic!("the view"));
            let pane = view.selected.expect("the address's pane");
            assert!(
                matches!(
                    pane.gestures[0].nature,
                    crate::triage_view::GestureRender::Live(_)
                ),
                "Ajouter present on {address}"
            );
            assert_eq!(pane.open_question, None);
        }

        // AC4: the engine never asks again.
        let again = counted_sweep(
            &pool,
            vec![
                at_address(sighting(11, 1, Some("obelix"), t + 1), "192.0.2.8"),
                at_address(sighting(12, 2, Some("obelix"), t + 1), "192.0.2.9"),
            ],
        )
        .await;
        assert_eq!(
            (again.l2.operator_held, again.l2.written),
            (1, 0),
            "{:?}",
            again.l2
        );
        let nameless = counted_sweep(
            &pool,
            vec![
                at_address(sighting(21, 1, Some("obelix"), t + 2), "192.0.2.8"),
                at_address(sighting(22, 2, None, t + 2), "192.0.2.9"),
            ],
        )
        .await;
        assert_eq!(
            (nameless.l2.operator_held, nameless.l2.vacated),
            (1, 0),
            "{:?}",
            nameless.l2
        );
        let rows = current(&pool).await;
        assert_eq!(rows.len(), 1);
        assert_eq!(
            (rows[0].2.as_str(), rows[0].4.as_str()),
            ("match", "OPERATOR")
        );
        let (view, identity) = crate::page::triage_view(&pool, None, false, true)
            .await
            .unwrap_or_else(|_| panic!("the view"));
        assert!(
            view.rows.iter().all(|r| !r.id.starts_with("ambigu:")),
            "never returns"
        );
        assert_eq!(
            identity.answered_questions, 1,
            "AC8: the answer stays visible"
        );
    }

    /// Story 6.14b (Guy, 2026-09-26): a group that RE-FORMS around an answered pair — A–B answered, then
    /// a third NIC D answering to the same name — asks only about D: the answer writes D's two pairs and
    /// leaves A–B's earlier answer as it was; and an answer that would CONTRADICT it is refused.
    #[tokio::test]
    async fn a_re_formed_group_writes_only_the_pairs_it_did_not_cover() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let t = 1_790_000_000_000_020;
        sweep(
            &pool,
            vec![
                sighting(1, 1, Some("obelix"), t),
                sighting(2, 2, Some("obelix"), t),
            ],
        )
        .await;
        let ab: BTreeSet<String> = interfaces(&pool).await.into_iter().collect();
        answer_on(&pool, &ab, Answer::DistinctMachines, at_micros(t))
            .await
            .expect("A–B answered");
        sweep(
            &pool,
            vec![
                sighting(11, 1, Some("obelix"), t + 1),
                sighting(12, 2, Some("obelix"), t + 1),
                sighting(13, 3, Some("obelix"), t + 1),
            ],
        )
        .await;
        let abd: BTreeSet<String> = interfaces(&pool).await.into_iter().collect();
        // 🔴 Guy's decision (2026-09-26, PR #220's review): *the same machine* on a group holding a pair
        // the operator already called DISTINCT would make A=B by transitivity — it is refused.
        assert_eq!(
            answer_on(&pool, &abd, Answer::SameMachine, at_micros(t + 1)).await,
            Err(AnswerRefused::Contradicts)
        );
        let answered = answer_on(&pool, &abd, Answer::DistinctMachines, at_micros(t + 1))
            .await
            .expect("the re-formed question, answered consistently");
        assert_eq!(answered.pairs, 2, "D's two pairs, and not A–B again");
        let outcomes: Vec<(String, String)> = current(&pool)
            .await
            .into_iter()
            .map(|r| (r.2, r.4))
            .collect();
        assert_eq!(outcomes.len(), 3);
        assert_eq!(
            outcomes.iter().filter(|(o, _)| o == "no_match").count(),
            3,
            "A–B's earlier answer stands and D is distinct from both: {outcomes:?}"
        );
        assert!(outcomes.iter().all(|(_, who)| who == "OPERATOR"));
    }

    /// 🔴 PR #220's review (edge layer, measured): with the question filter replaced by `|_| true` the
    /// whole suite stayed green while one click answered EVERY open question. Two open questions here —
    /// `obelix` and `asterix` — and answering one leaves the other exactly as the engine wrote it.
    #[tokio::test]
    async fn an_answer_answers_its_own_question_and_no_other() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let t = 1_790_000_000_000_030;
        sweep(
            &pool,
            vec![
                sighting(1, 1, Some("obelix"), t),
                sighting(2, 2, Some("obelix"), t),
                sighting(3, 3, Some("asterix"), t),
                sighting(4, 4, Some("asterix"), t),
            ],
        )
        .await;
        let questions = current(&pool).await;
        assert_eq!(
            questions.len(),
            2,
            "premise: two open questions {questions:?}"
        );
        let first: BTreeSet<String> = [questions[0].0.clone(), questions[0].1.clone()].into();
        let answered = answer_on(&pool, &first, Answer::DistinctMachines, at_micros(t))
            .await
            .expect("the first question");
        assert_eq!(answered.pairs, 1);
        let after = current(&pool).await;
        let other = after
            .iter()
            .find(|r| r.0 == questions[1].0 && r.1 == questions[1].1)
            .expect("the other question");
        assert_eq!(
            (other.2.as_str(), other.4.as_str()),
            ("abstained", "ENGINE"),
            "the other question is untouched"
        );
    }

    /// 🔴 PR #220's review (edge layer, MEASURED a deadlock whose victim was the sweep): the answer's
    /// locking read scanned the whole table (`type=ALL`), so an answer on one question waited on — and
    /// could deadlock with — a sweep closing ANY other question's row. Guy's decision (2026-09-26): only
    /// the question's own pairs are locked.
    ///
    /// ⚠️ **What this does NOT show, measured while writing it**: the answer can still wait on the pair
    /// that is its NEIGHBOUR in the unique key `(interface_low, interface_high, is_current)`. The answer's
    /// INSERT checks that key for duplicates, finds its own just-closed entry, and InnoDB then takes a
    /// shared lock on the next entry — which a sweep may hold. With two questions minted in one sweep they
    /// ARE neighbours, and the first version of this test waited on exactly that (the process list showed
    /// the INSERT in `Update`, and `INNODB_LOCKS` read empty). So three questions: answering the FIRST in key
    /// order does not wait on the THIRD, which the whole-table read did. The neighbour is registered.
    #[tokio::test]
    async fn an_answer_does_not_wait_on_another_questions_rows() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let t = 1_790_000_000_000_040;
        sweep(
            &pool,
            vec![
                sighting(1, 1, Some("obelix"), t),
                sighting(2, 2, Some("obelix"), t),
                sighting(3, 3, Some("asterix"), t),
                sighting(4, 4, Some("asterix"), t),
                sighting(5, 5, Some("idefix"), t),
                sighting(6, 6, Some("idefix"), t),
            ],
        )
        .await;
        // `current` is ordered by `(interface_low, interface_high)` — the unique key's order.
        let questions = current(&pool).await;
        assert_eq!(
            questions.len(),
            3,
            "premise: three open questions {questions:?}"
        );
        // The holder does what a SWEEP does to the LAST question's row: closes it by primary key through
        // `close_l2_decision`, and keeps its transaction open.
        let last_id: String = sqlx::query_scalar(
            "SELECT id FROM l2_pair_decision WHERE interface_low = ? AND interface_high = ? \
             AND is_current = 1",
        )
        .bind(&questions[2].0)
        .bind(&questions[2].1)
        .fetch_one(&pool)
        .await
        .expect("the last question's row");
        let mut holder = pool.acquire().await.expect("connection");
        let mut held = sqlx::Connection::begin(&mut *holder).await.expect("tx");
        crate::l2_repo::close_l2_decision(&mut *held, &last_id, at_micros(t + 5))
            .await
            .expect("a sweep closes the last question's row");
        let first: BTreeSet<String> = [questions[0].0.clone(), questions[0].1.clone()].into();
        let outcome = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            answer_on(&pool, &first, Answer::SameMachine, at_micros(t)),
        )
        .await;
        held.rollback().await.expect("release");
        assert!(
            matches!(outcome, Ok(Ok(_))),
            "the answer must not wait on a question that is not its key neighbour: {outcome:?}"
        );
    }

    /// PR #220's review (edge layer, measured): the store-side refusal of a group with no current
    /// placement was carried by no test — `None => {}` left the suite green and skipped the forged-
    /// instant bound too. The page offers no answer there (AC5); a POST that arrives anyway is refused.
    #[tokio::test]
    async fn a_question_with_no_placement_is_refused_by_the_store() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let t = 1_790_000_000_000_050;
        sweep(
            &pool,
            vec![
                sighting(1, 1, Some("obelix"), t),
                sighting(2, 2, Some("obelix"), t),
            ],
        )
        .await;
        let members: BTreeSet<String> = interfaces(&pool).await.into_iter().collect();
        // Every placement closed: the interfaces are in the question and on the network nowhere now.
        sqlx::query(
            "UPDATE identity_link SET valid_to = valid_from, current_subject = NULL \
             WHERE interface_id IS NOT NULL",
        )
        .execute(&pool)
        .await
        .expect("close the placements");
        assert_eq!(
            answer_on(&pool, &members, Answer::SameMachine, at_micros(t)).await,
            Err(AnswerRefused::Stale)
        );
    }

    /// AC5: stale, changed, not open and forged each refuse, and each writes nothing.
    #[tokio::test]
    async fn every_refusal_writes_nothing() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let t = 1_790_000_000_000_001;
        sweep(
            &pool,
            vec![
                sighting(1, 1, Some("obelix"), t),
                sighting(2, 2, Some("obelix"), t),
            ],
        )
        .await;
        let members: BTreeSet<String> = interfaces(&pool).await.into_iter().collect();
        let before = current(&pool).await;

        // Stale: the page was drawn before the ENGINE row was reached.
        assert_eq!(
            answer_on(&pool, &members, Answer::SameMachine, at_micros(t - 1)).await,
            Err(AnswerRefused::Stale)
        );
        // Forged: later than anything the store shows.
        assert_eq!(
            answer_on(
                &pool,
                &members,
                Answer::SameMachine,
                at_micros(253_402_300_799_999_998)
            )
            .await,
            Err(AnswerRefused::Forged)
        );
        // Changed: grown — a member the store does not group.
        let mut grown = members.clone();
        grown.insert("ffffffff-0000-0000-0000-00000000000f".to_string());
        assert_eq!(
            answer_on(&pool, &grown, Answer::SameMachine, at_micros(t)).await,
            Err(AnswerRefused::GroupChanged)
        );
        // Not open: interfaces no question holds.
        let strangers: BTreeSet<String> = [
            "ffffffff-0000-0000-0000-00000000000e".to_string(),
            "ffffffff-0000-0000-0000-00000000000f".to_string(),
        ]
        .into();
        assert_eq!(
            answer_on(&pool, &strangers, Answer::SameMachine, at_micros(t)).await,
            Err(AnswerRefused::NotOpen)
        );
        assert_eq!(current(&pool).await, before, "no refusal wrote anything");

        // And equality is accepted, microseconds included (§0.9(D)) — then the question is gone.
        answer_on(&pool, &members, Answer::DistinctMachines, at_micros(t))
            .await
            .expect("equality is the ordinary case");
        assert_eq!(
            answer_on(&pool, &members, Answer::DistinctMachines, at_micros(t)).await,
            Err(AnswerRefused::NotOpen),
            "already answered"
        );
    }

    /// AC5: a group that SHRANK under the page — three candidates shown, one lost its name — is refused
    /// rather than recorded for two (§0.9(E), measured writing ONE row before this check).
    #[tokio::test]
    async fn a_group_that_shrank_under_the_page_is_refused() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let t = 1_790_000_000_000_000;
        sweep(
            &pool,
            (1..=3)
                .map(|n| sighting(n, n as u8, Some("obelix"), t))
                .collect(),
        )
        .await;
        let three: BTreeSet<String> = interfaces(&pool).await.into_iter().collect();
        assert_eq!(three.len(), 3);
        // The third loses its name: the next sweep vacates its two pairs.
        sweep(
            &pool,
            vec![
                sighting(11, 1, Some("obelix"), t + 1),
                sighting(12, 2, Some("obelix"), t + 1),
                sighting(13, 3, None, t + 1),
            ],
        )
        .await;
        assert_eq!(
            answer_on(&pool, &three, Answer::SameMachine, at_micros(t + 1)).await,
            Err(AnswerRefused::GroupChanged)
        );
    }

    /// AC5: two answers to one question at once — exactly one current OPERATOR row per pair, and the
    /// loser is the keyed 409, never a 500. Two carriers serialise them — the lock on the pairs and the
    /// close that follows — which mutation M2 measured by removing the first and staying green.
    #[tokio::test]
    async fn two_concurrent_answers_leave_one_and_the_loser_is_told_it_is_no_longer_open() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else { return };
        let t = 1_790_000_000_000_002;
        sweep(
            &pool,
            vec![
                sighting(1, 1, Some("obelix"), t),
                sighting(2, 2, Some("obelix"), t),
            ],
        )
        .await;
        let members: BTreeSet<String> = interfaces(&pool).await.into_iter().collect();
        // The first answer holds its transaction open while the second starts.
        let mut first = pool.acquire().await.expect("connection");
        let mut first_tx = sqlx::Connection::begin(&mut *first).await.expect("tx");
        record_operator_answer(&mut first_tx, &members, Answer::SameMachine, at_micros(t))
            .await
            .expect("the first answer");
        let second = {
            let pool = pool.clone();
            let members = members.clone();
            tokio::spawn(async move {
                answer_on(&pool, &members, Answer::DistinctMachines, at_micros(t)).await
            })
        };
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        first_tx.commit().await.expect("commit");
        assert_eq!(second.await.expect("join"), Err(AnswerRefused::NotOpen));
        let rows = current(&pool).await;
        assert_eq!(rows.len(), 1);
        assert_eq!(
            (rows[0].2.as_str(), rows[0].4.as_str()),
            ("match", "OPERATOR")
        );
    }
}
