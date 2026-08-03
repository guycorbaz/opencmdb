//! The MariaDB adapter for the persistence contract (D49). This is the only place SQL against
//! the domain tables is written, and the only place a `sqlx::Error` becomes a `RepositoryError`
//! (D47) — the query bodies are free functions generic over `sqlx::Executor` that both the read
//! side and a unit of work delegate to, so the query is written once.
//!
//! _(This doc claimed to be "the ONLY place `sqlx` appears" until story 5.9. Measured: `sqlx` is
//! also used by `main.rs`, `page.rs` and `dburl.rs`. The weaker sentence above is the true one.)_
//!
//! Skeleton (D49 story-1 bar): it COMPILES and is proven by a `transact` round-trip test.
//! The running app wires it in from Story 3.5 (ingestion) onward — hence `allow(dead_code)`.
#![allow(dead_code)]

use opencmdb_core::identity::cascade::{Conclusion, Decision, IdentityAbstentionCause};
use opencmdb_core::observation::{InterfaceId, L2DomainId, LinkId, MacAddr, ObsId, Timestamp};
use opencmdb_core::repo::{BoxFuture, ReadRepository, RepositoryError, WriteRepository, WriteUnit};
use sqlx::{Executor, MySql, MySqlConnection, MySqlPool};

/// The write side, over a MariaDB pool.
pub struct MariaRepository {
    pool: MySqlPool,
}

impl MariaRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

/// A unit of work: a mutable borrow of the transaction's connection. Holding `&'u mut Conn`
/// (not a `Transaction<'u>` by value) keeps the unit COVARIANT in `'u`, which is what lets the
/// `for<'u> FnOnce(&'u mut Unit<'u>)` closure (an HRTB over the GAT) type-check without erasure.
pub struct MariaUnit<'u> {
    conn: &'u mut MySqlConnection,
}

impl MariaUnit<'_> {
    /// Lend the unit's connection as a sqlx `Executor` to the query bodies (read-your-own-writes).
    pub(crate) fn executor(&mut self) -> &mut MySqlConnection {
        self.conn
    }
}

impl WriteUnit for MariaUnit<'_> {}

impl WriteRepository for MariaRepository {
    type Unit<'u>
        = MariaUnit<'u>
    where
        Self: 'u;

    async fn transact<F, T>(&self, f: F) -> Result<T, RepositoryError>
    where
        F: for<'u> FnOnce(&'u mut Self::Unit<'u>) -> BoxFuture<'u, Result<T, RepositoryError>>
            + Send,
        T: Send,
    {
        let mut tx = self.pool.begin().await.map_err(classify)?;
        // The closure borrows the unit (and thus the connection) for its whole future; once it
        // resolves, the borrow ends and we own `tx` again to commit or roll back.
        let result = {
            let mut unit = MariaUnit { conn: &mut tx };
            f(&mut unit).await
        };
        match result {
            Ok(value) => {
                tx.commit().await.map_err(classify)?;
                Ok(value)
            }
            Err(error) => {
                let _ = tx.rollback().await; // best-effort; the original error is what matters
                Err(error)
            }
        }
    }
}

/// The read side, over the pool (`&self`, D21).
pub struct MariaReadRepository {
    pool: MySqlPool,
}

impl MariaReadRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Count declared attributes via the read pool — delegates to the shared query body.
    pub async fn count_declared_attributes(&self) -> Result<i64, RepositoryError> {
        count_declared_attributes(&self.pool)
            .await
            .map_err(classify)
    }
}

impl ReadRepository for MariaReadRepository {}

// ── The query bodies: written once, generic over `sqlx::Executor` (D49) ──────

/// `SELECT COUNT(*) FROM declared_attribute`. The read side calls it with the pool; a unit of
/// work calls it with its transaction connection (read-your-own-writes).
pub async fn count_declared_attributes<'e, E>(executor: E) -> Result<i64, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    // Static SQL — no `AssertSqlSafe` needed.
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM declared_attribute")
        .fetch_one(executor)
        .await?;
    Ok(count)
}

/// Insert one manually-authored declared attribute. Static SQL, bound values (D48).
pub async fn insert_declared_attribute<'e, E>(
    executor: E,
    entity_id: &str,
    attr_key: &str,
    attr_value: &str,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    sqlx::query(
        "INSERT INTO declared_attribute \
         (entity_id, attr_key, attr_value, origin, actor_id, updated_at) \
         VALUES (?, ?, ?, 'manual', 'operator', NOW(6))",
    )
    .bind(entity_id)
    .bind(attr_key)
    .bind(attr_value)
    .execute(executor)
    .await?;
    Ok(())
}

/// Insert one observation (immutable, linked-never-merged, FR11). `facts` serialize to JSON —
/// the engine deserializes and compares in Rust; SQL never compares (D10). All values are bound
/// as Strings (D48); `observed_at` as a MariaDB datetime literal.
pub async fn insert_observation<'e, E>(
    executor: E,
    observation: &opencmdb_core::observation::Observation,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let facts =
        serde_json::to_string(&observation.facts).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;
    let observed_at = observation
        .observed_at
        .format("%Y-%m-%d %H:%M:%S%.6f")
        .to_string();
    sqlx::query(
        "INSERT INTO observation_record \
         (id, connector_id, observed_at, l2_domain, vantage, facts, raw) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(observation.obs_id.to_string())
    .bind(observation.connector_id.to_string())
    .bind(observed_at)
    .bind(observation.scope.l2_domain.to_string())
    .bind(observation.scope.vantage.to_string())
    .bind(facts)
    .bind(observation.raw.clone())
    .execute(executor)
    .await?;
    Ok(())
}

/// Count observation records via any executor.
pub async fn count_observations<'e, E>(executor: E) -> Result<i64, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM observation_record")
        .fetch_one(executor)
        .await?;
    Ok(count)
}

/// Load every declared attribute as `(entity_id, attr_key, attr_value)`, ordered so a page groups
/// them deterministically. Static SQL; the page reconciles in Rust (SQL never compares — D10).
pub async fn load_declared_attributes<'e, E>(
    executor: E,
) -> Result<Vec<(String, String, String)>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT entity_id, attr_key, attr_value FROM declared_attribute \
         ORDER BY entity_id, attr_key",
    )
    .fetch_all(executor)
    .await?;
    Ok(rows)
}

/// Load each observation's `facts` JSON, deserialized into `Vec<Fact>` (oldest first). The engine
/// compares the facts in Rust — the JSON never round-trips through SQL comparison (D10).
pub async fn load_observation_facts<'e, E>(
    executor: E,
) -> Result<Vec<Vec<opencmdb_core::observation::Fact>>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT facts FROM observation_record ORDER BY observed_at")
            .fetch_all(executor)
            .await?;
    let mut out = Vec::with_capacity(rows.len());
    for (facts,) in rows {
        let parsed: Vec<opencmdb_core::observation::Fact> =
            serde_json::from_str(&facts).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
        out.push(parsed);
    }
    Ok(out)
}

// ── Identity persistence: interfaces, links and their candidates (story 5.9) ──────

/// The `valid_to` of a link that is still current.
///
/// D21 writes this sentinel `OPEN_END = '9999-12-31T23:59:59.999Z'` [architecture.md:1467] — an
/// ISO-8601 TEXT literal from the two-engine era, when dates were stored as text. D64 made MariaDB
/// the only engine and the column a `DATETIME(6)`, so the same instant is written the way MariaDB
/// writes instants. **This is a transposition, not a contradiction.**
///
/// It is a sentinel rather than `NULL` because the uniqueness key contains this column, and
/// MariaDB holds NULLs distinct: with a NULL here `identity_link_one_current` would never fire and
/// "exactly one current link" would be decorative — D21's trap [architecture.md:1462-1468].
/// [`ABSTAINED_SUBJECT`] closes the same trap on the other column of the same key.
pub const OPEN_END: &str = "9999-12-31 23:59:59.999999";

/// The `link_subject` of a link that names no interface — an abstention.
///
/// The nil UUID, standing for "no interface". Same reasoning as [`OPEN_END`], one column over:
/// `interface_id` is NULL for an abstention, the uniqueness key contains it, and MariaDB holds
/// NULLs distinct — so two current abstentions for one observation would both insert and the
/// abstention half of the constraint would be decorative. The DDL's
/// `identity_link_subject_matches` CHECK is what stops the sentinel drifting from what it stands
/// for.
///
/// It is **never** an `interface.id`: `interface_id` keeps its foreign key and stays NULL.
pub const ABSTAINED_SUBJECT: &str = "00000000-0000-0000-0000-000000000000";

/// The persisted token for a [`Conclusion`], by an exhaustive `match`.
///
/// No `#[derive(Serialize)]`, deliberately: a derived variant name is a wire format nobody chose,
/// and renaming a variant would silently rewrite stored bytes — the *"silent data migration, the
/// worst kind"* D14 names about `ruleset_version`. [`Conclusion`] is also deliberately not
/// `#[non_exhaustive]`, so a new variant produces `error[E0004]` here. **No `_` arm** — the `_` is
/// what turns that compile error into a silent mis-classification.
fn outcome_token(conclusion: &Conclusion) -> &'static str {
    match conclusion {
        Conclusion::Match { .. } => "match",
        Conclusion::NoMatch { .. } => "no_match",
        Conclusion::Abstained { .. } => "abstained",
    }
}

/// The persisted token for an [`IdentityAbstentionCause`], by an exhaustive `match`.
///
/// Same refusal and same reason as [`outcome_token`].
fn cause_token(cause: &IdentityAbstentionCause) -> &'static str {
    match cause {
        IdentityAbstentionCause::Ambiguous => "ambiguous",
        IdentityAbstentionCause::AbsenceOfProof => "absence_of_proof",
    }
}

/// Who decided a link. `decided_by` is not optional: story 5.10 deletes the engine's links by it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecidedBy {
    /// The identity engine derived it.
    Engine,
    /// A human asserted it.
    Operator,
}

impl DecidedBy {
    /// The persisted token — exhaustive `match`, no `_` arm, same refusal as [`outcome_token`].
    fn token(self) -> &'static str {
        match self {
            Self::Engine => "ENGINE",
            Self::Operator => "OPERATOR",
        }
    }
}

/// Insert one interface.
///
/// `first_seen_at` and `last_seen_at` are **parameters** and must be derived from the observations
/// on the interface (their earliest and latest `observed_at`), never read from the clock: *"the
/// engine never touches the clock"* [architecture.md:3364], and story 5.10 replays the engine and
/// compares bit for bit. `insert_declared_attribute`'s `NOW(6)` is a DECLARED row authored by a
/// human and is not a precedent for an engine-derived one.
///
/// `mac_canon` is [`MacAddr`]'s `Display` — lowercase, colon-separated. There is no second
/// canonicalisation.
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`].
pub async fn insert_interface<'e, E>(
    executor: E,
    id: InterfaceId,
    l2_domain: L2DomainId,
    mac_canon: &MacAddr,
    first_seen_at: Timestamp,
    last_seen_at: Timestamp,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    sqlx::query(
        "INSERT INTO interface \
         (id, l2_domain, mac_canon, first_seen_at, last_seen_at) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(l2_domain.to_string())
    .bind(mac_canon.to_string())
    .bind(datetime_literal(first_seen_at))
    .bind(datetime_literal(last_seen_at))
    .execute(executor)
    .await?;
    Ok(())
}

/// Format an instant the way MariaDB writes one. The single formatting site; do not invent a
/// second format string.
fn datetime_literal(at: Timestamp) -> String {
    at.format("%Y-%m-%d %H:%M:%S%.6f").to_string()
}

/// Insert one identity link, deriving every decision-shaped column from the [`Decision`] itself.
///
/// The derivation is ONE `match` over the conclusion, so a single call site cannot get the
/// rule-XOR-cause pairing wrong. That is what makes the DDL's `identity_link_rule_xor_cause` a
/// second line of defence rather than the only one.
///
/// `interface` is `None` exactly for an abstention, and the DDL says so too. `valid_from` is a
/// **parameter**, never `NOW(6)` — see [`insert_interface`] for why.
///
/// The `verdict_vector` is **not stored**: D14's list of what a link carries does not include it,
/// and storing it would mean deriving a wire format for four domain types to serve no reader. The
/// consequence, stated rather than discovered: a persisted link cannot be turned back into a
/// `Decision`.
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`].
#[allow(clippy::too_many_arguments)]
pub async fn insert_identity_link<'e, E>(
    executor: E,
    id: LinkId,
    observation_id: ObsId,
    interface: Option<InterfaceId>,
    decision: &Decision,
    evidence: &[ObsId],
    decided_by: DecidedBy,
    valid_from: Timestamp,
    valid_to: Timestamp,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    // One match: the outcome, the rule and the cause are derived together or not at all.
    let (rule_id, abstention_cause) = match &decision.conclusion {
        Conclusion::Match { rule } | Conclusion::NoMatch { rule } => (Some(rule.0.clone()), None),
        Conclusion::Abstained { cause } => (None, Some(cause_token(cause))),
    };
    let evidence_json =
        serde_json::to_string(evidence).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;
    sqlx::query(
        "INSERT INTO identity_link \
         (id, observation_id, interface_id, link_subject, outcome, rule_id, abstention_cause, \
          evidence, ruleset_version, decided_by, valid_from, valid_to) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(observation_id.to_string())
    .bind(interface.map(|i| i.to_string()))
    .bind(link_subject_of(interface))
    .bind(outcome_token(&decision.conclusion))
    .bind(rule_id)
    .bind(abstention_cause)
    .bind(evidence_json)
    .bind(decision.ruleset_version.0)
    .bind(decided_by.token())
    .bind(datetime_literal(valid_from))
    .bind(datetime_literal(valid_to))
    .execute(executor)
    .await?;
    Ok(())
}

/// The `link_subject` for a link pointing at `interface` — the interface, or [`ABSTAINED_SUBJECT`]
/// when there is none. The single derivation site, which is what keeps it from drifting.
fn link_subject_of(interface: Option<InterfaceId>) -> String {
    interface.map_or_else(|| ABSTAINED_SUBJECT.to_string(), |i| i.to_string())
}

/// Close a current link by stamping its `valid_to` — an SCD2 supersede is this plus an append.
///
/// The old row stays readable with its old `valid_to`: *"a bad link is UNLINKED, never erased"*
/// [architecture.md:1016-1017]. `closed_at` is a parameter, never the clock.
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`].
pub async fn close_identity_link<'e, E>(
    executor: E,
    id: LinkId,
    closed_at: Timestamp,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    sqlx::query("UPDATE identity_link SET valid_to = ? WHERE id = ?")
        .bind(datetime_literal(closed_at))
        .bind(id.to_string())
        .execute(executor)
        .await?;
    Ok(())
}

/// Insert one candidate interface of an abstained link, with the evidence that made it a candidate.
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`].
pub async fn insert_link_candidate<'e, E>(
    executor: E,
    link_id: LinkId,
    interface_id: InterfaceId,
    evidence: &[ObsId],
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let evidence_json =
        serde_json::to_string(evidence).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;
    sqlx::query("INSERT INTO link_candidate (link_id, interface_id, evidence) VALUES (?, ?, ?)")
        .bind(link_id.to_string())
        .bind(interface_id.to_string())
        .bind(evidence_json)
        .execute(executor)
        .await?;
    Ok(())
}

/// One persisted link, as it was read back. Rows, not a reconstructed `Decision`.
///
/// A `Decision` cannot be rebuilt from this: the `verdict_vector` is not stored (see
/// [`insert_identity_link`]), so no constructor bypassing `decide` is written here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedLink {
    /// The link's own id.
    pub id: String,
    /// The interface it places the observation on — `None` for an abstention.
    pub interface_id: Option<String>,
    /// The persisted outcome token.
    pub outcome: String,
    /// The rule that settled it — `None` for an abstention.
    pub rule_id: Option<String>,
    /// Why it abstained — `None` unless it did.
    pub abstention_cause: Option<String>,
    /// The observations that justified it, as stored.
    pub evidence: Vec<ObsId>,
    /// The ruleset that produced it (D14).
    pub ruleset_version: u32,
    /// Who decided it, as stored.
    pub decided_by: String,
}

/// Load the CURRENT links of one observation — plural, because one observation can sit on several
/// interfaces at once.
///
/// That is not a hypothetical: the L1 join inserts an observation under **every** key it carries,
/// so a multi-MAC observation legitimately holds one current link per interface. A singular
/// accessor would encode a constraint the schema deliberately does not have.
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`].
pub async fn load_current_links_for_observation<'e, E>(
    executor: E,
    observation_id: ObsId,
) -> Result<Vec<PersistedLink>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(
        String,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
        String,
        u32,
        String,
    )> = sqlx::query_as(
        "SELECT id, interface_id, outcome, rule_id, abstention_cause, evidence, \
                ruleset_version, decided_by \
         FROM identity_link WHERE observation_id = ? AND valid_to = ? \
         ORDER BY link_subject",
    )
    .bind(observation_id.to_string())
    .bind(OPEN_END)
    .fetch_all(executor)
    .await?;
    rows.into_iter()
        .map(|r| {
            Ok(PersistedLink {
                id: r.0,
                interface_id: r.1,
                outcome: r.2,
                rule_id: r.3,
                abstention_cause: r.4,
                evidence: serde_json::from_str(&r.5)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                ruleset_version: r.6,
                decided_by: r.7,
            })
        })
        .collect()
}

/// Load one link by its id, current or superseded, with its `valid_to` as stored.
///
/// This is what proves a superseded link is still readable (D14).
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`].
pub async fn load_link_valid_to<'e, E>(
    executor: E,
    id: LinkId,
) -> Result<Option<String>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    // `valid_to` is a DATETIME(6). `sqlx` is built here without its `chrono` feature, so it has no
    // Rust type to decode one into; `CAST(… AS CHAR)` renders it in MariaDB's own datetime shape,
    // which is exactly [`OPEN_END`]'s. That is TRANSPORT, not comparison — D10 forbids SQL to
    // descend into a domain value, and an instant's encoding on the wire is not one. The write
    // path is already symmetric: it renders in Rust and binds a string.
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT CAST(valid_to AS CHAR) FROM identity_link WHERE id = ?")
            .bind(id.to_string())
            .fetch_all(executor)
            .await?;
    Ok(rows.into_iter().next().map(|r| r.0))
}

/// Load the candidates of one link, each with its evidence, ordered so a page renders them
/// deterministically.
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`].
pub async fn load_link_candidates<'e, E>(
    executor: E,
    link_id: LinkId,
) -> Result<Vec<(String, Vec<ObsId>)>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT interface_id, evidence FROM link_candidate WHERE link_id = ? ORDER BY interface_id",
    )
    .bind(link_id.to_string())
    .fetch_all(executor)
    .await?;
    rows.into_iter()
        .map(|(interface_id, evidence)| {
            let parsed =
                serde_json::from_str(&evidence).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            Ok((interface_id, parsed))
        })
        .collect()
}

/// Count identity links via any executor.
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`].
pub async fn count_identity_links<'e, E>(executor: E) -> Result<i64, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM identity_link")
        .fetch_one(executor)
        .await?;
    Ok(count)
}

/// Classify a `sqlx::Error` into the closed `RepositoryError` (D47) — the ONLY translation of
/// a backend error in this crate.
pub fn classify(error: sqlx::Error) -> RepositoryError {
    if let sqlx::Error::RowNotFound = error {
        return RepositoryError::NotFound;
    }
    if let Some(db) = error.as_database_error() {
        // MariaDB: 1213 = deadlock, 1205 = lock wait timeout → retryable contention (NFR15).
        match db.code().as_deref() {
            Some("1213") | Some("1205") => return RepositoryError::Contention,
            _ if db.is_unique_violation() => {
                return RepositoryError::Constraint("unique");
            }
            _ if db.is_foreign_key_violation() => {
                return RepositoryError::Constraint("foreign_key");
            }
            _ if db.is_check_violation() => {
                return RepositoryError::Constraint("check");
            }
            _ => {}
        }
    }
    RepositoryError::Backend(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencmdb_core::repo::WriteRepository;

    /// A `transact` round-trip against a real MariaDB: the closure inserts a declared attribute
    /// through the unit and reads its own write back; after commit, the read side sees it.
    /// Gated on `DATABASE_URL` (CI's MariaDB service; a local container in dev).
    #[tokio::test]
    async fn transact_writes_and_reads_its_own_write() {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping repo round-trip: DATABASE_URL unset");
            return;
        };
        let _guard = crate::DB_TEST_LOCK.lock().await; // serialize DB tests (see the static)
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        // Isolate this test run.
        sqlx::query("DELETE FROM declared_attribute")
            .execute(&pool)
            .await
            .expect("clean");

        let repo = MariaRepository::new(pool.clone());
        let entity = "00000000-0000-0000-0000-000000000001";
        let count_in_tx = repo
            .transact(move |unit| {
                Box::pin(async move {
                    insert_declared_attribute(unit.executor(), entity, "hostname", "nas")
                        .await
                        .map_err(classify)?;
                    // read-your-own-writes: the count sees the uncommitted insert
                    count_declared_attributes(unit.executor())
                        .await
                        .map_err(classify)
                })
            })
            .await
            .expect("transact");
        assert_eq!(
            count_in_tx, 1,
            "read-your-own-writes inside the transaction"
        );

        // After commit, the read side sees the row.
        let read = MariaReadRepository::new(pool);
        assert_eq!(read.count_declared_attributes().await.unwrap(), 1);
    }

    /// Ingest a synthetic observation and read it back — the observed side round-trips (FR11).
    #[tokio::test]
    async fn ingest_observation_round_trip() {
        use opencmdb_core::observation::{
            ConnectorId, Fact, L2DomainId, MacAddr, ObsId, Observation, Scope, VantageId,
        };
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping ingest round-trip: DATABASE_URL unset");
            return;
        };
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        sqlx::query("DELETE FROM observation_record")
            .execute(&pool)
            .await
            .expect("clean");

        let obs = Observation {
            obs_id: ObsId::from_uuid(uuid::Uuid::now_v7()),
            connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()),
            observed_at: chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
            scope: Scope {
                l2_domain: L2DomainId::from_uuid(uuid::Uuid::nil()),
                vantage: VantageId::from_uuid(uuid::Uuid::nil()),
            },
            facts: vec![
                Fact::Mac {
                    addr: MacAddr([0, 1, 2, 3, 4, 5]),
                    locally_administered: false,
                },
                Fact::Rtt { millis: 3 },
            ],
            raw: None,
        };

        let repo = MariaRepository::new(pool.clone());
        repo.transact(move |unit| {
            let obs = obs.clone();
            Box::pin(async move {
                insert_observation(unit.executor(), &obs)
                    .await
                    .map_err(classify)
            })
        })
        .await
        .expect("ingest");

        assert_eq!(count_observations(&pool).await.unwrap(), 1);
    }

    // ── Identity persistence (story 5.9) ──────
    //
    // Every test below is gated on DATABASE_URL and serialized under DB_TEST_LOCK. ⚠️ With
    // DATABASE_URL unset they PASS BY RETURNING and the whole suite stays green — six of this
    // story's seven mutations are pure schema behaviour and red nothing without a database.

    use opencmdb_core::identity::l1::CURRENT_RULESET_VERSION;
    use opencmdb_core::trap::RuleId;

    /// An instant that is a parameter, not the clock — every timestamp this module stores is.
    fn at(secs: i64) -> Timestamp {
        chrono::DateTime::from_timestamp(secs, 0).expect("in range")
    }

    /// `OPEN_END` as a `Timestamp`, so a test can pass it where a link's `valid_to` is a parameter.
    fn open_end() -> Timestamp {
        use chrono::TimeZone;
        chrono::Utc
            .with_ymd_and_hms(9999, 12, 31, 23, 59, 59)
            .single()
            .expect("the sentinel instant")
            + chrono::Duration::microseconds(999_999)
    }

    fn a_match(rule: &str) -> Decision {
        Decision {
            conclusion: Conclusion::Match {
                rule: RuleId(rule.to_string()),
            },
            verdict_vector: vec![],
            ruleset_version: CURRENT_RULESET_VERSION,
        }
    }

    fn an_abstention(cause: IdentityAbstentionCause) -> Decision {
        Decision {
            conclusion: Conclusion::Abstained { cause },
            verdict_vector: vec![],
            ruleset_version: CURRENT_RULESET_VERSION,
        }
    }

    /// Connect, migrate and empty the three identity tables plus `observation_record`.
    ///
    /// The `DELETE`s are one static statement per table, not a loop over table names: sqlx 0.9
    /// rejects `sqlx::query(&format!(…))` at compile time. Children before parents (FKs).
    async fn identity_fixture() -> Option<MySqlPool> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping identity persistence test: DATABASE_URL unset");
            return None;
        };
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        sqlx::query("DELETE FROM link_candidate")
            .execute(&pool)
            .await
            .expect("clean candidates");
        sqlx::query("DELETE FROM identity_link")
            .execute(&pool)
            .await
            .expect("clean links");
        sqlx::query("DELETE FROM interface")
            .execute(&pool)
            .await
            .expect("clean interfaces");
        Some(pool)
    }

    async fn an_interface(pool: &MySqlPool, l2: L2DomainId, mac: [u8; 6]) -> InterfaceId {
        let id = InterfaceId::from_uuid(uuid::Uuid::now_v7());
        insert_interface(
            pool,
            id,
            l2,
            &MacAddr(mac),
            at(1_700_000_000),
            at(1_700_000_100),
        )
        .await
        .map_err(classify)
        .expect("insert interface");
        id
    }

    /// AC2 — a match link round-trips through one `transact`, read back as current.
    #[tokio::test]
    async fn a_match_link_round_trips() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let obs = ObsId::from_uuid(uuid::Uuid::now_v7());
        let iface = an_interface(&pool, l2, [0, 1, 2, 3, 4, 5]).await;
        let link = LinkId::from_uuid(uuid::Uuid::now_v7());
        let evidence = vec![obs, ObsId::from_uuid(uuid::Uuid::now_v7())];

        let repo = MariaRepository::new(pool.clone());
        let ev = evidence.clone();
        repo.transact(move |unit| {
            let ev = ev.clone();
            Box::pin(async move {
                insert_identity_link(
                    unit.executor(),
                    link,
                    obs,
                    Some(iface),
                    &a_match("l1-exact-mac"),
                    &ev,
                    DecidedBy::Engine,
                    at(1_700_000_000),
                    open_end(),
                )
                .await
                .map_err(classify)
            })
        })
        .await
        .expect("write the link");

        let links = load_current_links_for_observation(&pool, obs)
            .await
            .map_err(classify)
            .expect("read back");
        assert_eq!(links.len(), 1, "exactly one current link was written");
        let got = &links[0];
        assert_eq!(got.outcome, "match");
        assert_eq!(got.rule_id.as_deref(), Some("l1-exact-mac"));
        assert_eq!(got.abstention_cause, None);
        assert_eq!(
            got.interface_id.as_deref(),
            Some(iface.to_string().as_str())
        );
        assert_eq!(got.decided_by, "ENGINE");
        assert_eq!(got.ruleset_version, CURRENT_RULESET_VERSION.0);
        // AC5's other half: the evidence survives byte-identically, order included.
        assert_eq!(got.evidence, evidence, "evidence round-trips as Vec<ObsId>");
    }

    /// AC2 — SCD2: superseding appends and closes; the superseded row is STILL READABLE.
    #[tokio::test]
    async fn superseding_a_link_leaves_the_old_row_readable() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let obs = ObsId::from_uuid(uuid::Uuid::now_v7());
        let iface = an_interface(&pool, l2, [0, 1, 2, 3, 4, 6]).await;
        let first = LinkId::from_uuid(uuid::Uuid::now_v7());
        let second = LinkId::from_uuid(uuid::Uuid::now_v7());

        write_link(
            &pool,
            first,
            obs,
            Some(iface),
            &a_match("l1-exact-mac"),
            open_end(),
        )
        .await
        .expect("first link");
        close_identity_link(&pool, first, at(1_700_000_500))
            .await
            .map_err(classify)
            .expect("close the first");
        write_link(
            &pool,
            second,
            obs,
            Some(iface),
            &a_match("l1-exact-mac"),
            open_end(),
        )
        .await
        .expect("the superseding link");

        let current = load_current_links_for_observation(&pool, obs)
            .await
            .map_err(classify)
            .expect("read current");
        assert_eq!(
            current.len(),
            1,
            "exactly one link is current after a supersede"
        );
        assert_eq!(
            current[0].id,
            second.to_string(),
            "the current one is the new one"
        );

        // "A bad link is UNLINKED, never erased."
        let old = load_link_valid_to(&pool, first)
            .await
            .map_err(classify)
            .expect("read the superseded link");
        assert_eq!(
            old.as_deref(),
            Some(datetime_literal(at(1_700_000_500)).as_str()),
            "the superseded row is still readable, carrying its closing stamp"
        );
        assert_ne!(
            old.as_deref(),
            Some(OPEN_END),
            "and it is no longer current"
        );
    }

    /// Helper: write one link outside a transaction, classified.
    async fn write_link(
        pool: &MySqlPool,
        id: LinkId,
        obs: ObsId,
        interface: Option<InterfaceId>,
        decision: &Decision,
        valid_to: Timestamp,
    ) -> Result<(), RepositoryError> {
        insert_identity_link(
            pool,
            id,
            obs,
            interface,
            decision,
            &[obs],
            DecidedBy::Engine,
            at(1_700_000_000),
            valid_to,
        )
        .await
        .map_err(classify)
    }

    /// AC3 half 1 — a second current link for the same (observation, interface) is REFUSED.
    #[tokio::test]
    async fn a_second_current_link_for_one_placement_is_refused() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let obs = ObsId::from_uuid(uuid::Uuid::now_v7());
        let iface = an_interface(&pool, l2, [0, 1, 2, 3, 4, 7]).await;

        write_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::now_v7()),
            obs,
            Some(iface),
            &a_match("l1-exact-mac"),
            open_end(),
        )
        .await
        .expect("the first current link");

        let second = write_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::now_v7()),
            obs,
            Some(iface),
            &a_match("l1-exact-mac"),
            open_end(),
        )
        .await;
        assert_eq!(
            second,
            Err(RepositoryError::Constraint("unique")),
            "opening a second current link without closing the first must be refused"
        );
    }

    /// AC3 half 3 — and after that refusal, exactly ONE is current. This is the assertion that
    /// COUNTS: the `Constraint("unique")` shape above panics at `expect_err` before any count
    /// exists, so it cannot carry M5's red. Measured at validation: without the sentinel this
    /// reds `left: 2, right: 1`.
    #[tokio::test]
    async fn exactly_one_link_is_current_per_placement() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let obs = ObsId::from_uuid(uuid::Uuid::now_v7());
        let iface = an_interface(&pool, l2, [0, 1, 2, 3, 4, 8]).await;

        for _ in 0..2 {
            // Deliberately NOT `expect_err`: the count below is the assertion under test, and a
            // panic here would run before it exists.
            let _ = write_link(
                &pool,
                LinkId::from_uuid(uuid::Uuid::now_v7()),
                obs,
                Some(iface),
                &a_match("l1-exact-mac"),
                open_end(),
            )
            .await;
        }

        let current = load_current_links_for_observation(&pool, obs)
            .await
            .map_err(classify)
            .expect("read current");
        assert_eq!(
            current.len(),
            1,
            "exactly one link is current per observation and interface"
        );
    }

    /// AC3 half 2 — and the key does NOT over-fire: a multi-MAC observation legitimately holds one
    /// current link per interface. The L1 join inserts an observation under EVERY key it carries,
    /// and `multi-nic` is a committed trap family — the narrower `(observation_id, valid_to)` key
    /// was measured refusing this exact write.
    #[tokio::test]
    async fn one_observation_holds_a_current_link_on_each_of_its_interfaces() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let obs = ObsId::from_uuid(uuid::Uuid::now_v7());
        let nic_a = an_interface(&pool, l2, [0, 1, 2, 3, 4, 9]).await;
        let nic_b = an_interface(&pool, l2, [0, 1, 2, 3, 4, 10]).await;

        write_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::now_v7()),
            obs,
            Some(nic_a),
            &a_match("l1-exact-mac"),
            open_end(),
        )
        .await
        .expect("the first NIC's link");

        let second = write_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::now_v7()),
            obs,
            Some(nic_b),
            &a_match("l1-exact-mac"),
            open_end(),
        )
        .await;
        assert_eq!(
            second,
            Ok(()),
            "a multi-MAC observation must be linkable to each of its interfaces at once"
        );

        let current = load_current_links_for_observation(&pool, obs)
            .await
            .map_err(classify)
            .expect("read current");
        assert_eq!(current.len(), 2, "both placements are current");
    }

    /// Decision 9's other half — two current ABSTENTIONS for one observation are refused. Without
    /// the `link_subject` sentinel both `interface_id`s are NULL, MariaDB holds NULLs distinct,
    /// and the constraint would be decorative for exactly the half FR16 exists to display.
    #[tokio::test]
    async fn a_second_current_abstention_for_one_observation_is_refused() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let obs = ObsId::from_uuid(uuid::Uuid::now_v7());
        let abstention = an_abstention(IdentityAbstentionCause::Ambiguous);

        write_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::now_v7()),
            obs,
            None,
            &abstention,
            open_end(),
        )
        .await
        .expect("the first abstention");

        let second = write_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::now_v7()),
            obs,
            None,
            &abstention,
            open_end(),
        )
        .await;
        assert_eq!(
            second,
            Err(RepositoryError::Constraint("unique")),
            "a NULL interface_id must not make the uniqueness key decorative"
        );
    }

    /// AC4 — an ambiguous outcome is a LINK with its candidates, never an absence. The link row's
    /// presence is asserted by COUNT, not by `.expect()`ing its write: the `.expect()` form lets
    /// the candidates' foreign key carry the red instead of this assertion.
    #[tokio::test]
    async fn an_ambiguity_is_a_link_with_its_candidates() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let obs = ObsId::from_uuid(uuid::Uuid::now_v7());
        let one = an_interface(&pool, l2, [0, 1, 2, 3, 4, 11]).await;
        let two = an_interface(&pool, l2, [0, 1, 2, 3, 4, 12]).await;
        let link = LinkId::from_uuid(uuid::Uuid::now_v7());
        let ev_one = vec![obs];
        let ev_two = vec![obs, ObsId::from_uuid(uuid::Uuid::now_v7())];

        let repo = MariaRepository::new(pool.clone());
        let (e1, e2) = (ev_one.clone(), ev_two.clone());
        let written = repo
            .transact(move |unit| {
                let (e1, e2) = (e1.clone(), e2.clone());
                Box::pin(async move {
                    insert_identity_link(
                        unit.executor(),
                        link,
                        obs,
                        None,
                        &an_abstention(IdentityAbstentionCause::Ambiguous),
                        &e1,
                        DecidedBy::Engine,
                        at(1_700_000_000),
                        open_end(),
                    )
                    .await
                    .map_err(classify)?;
                    insert_link_candidate(unit.executor(), link, one, &e1)
                        .await
                        .map_err(classify)?;
                    insert_link_candidate(unit.executor(), link, two, &e2)
                        .await
                        .map_err(classify)
                })
            })
            .await;
        // Not `.expect(…)`: a failure here must not pre-empt the assertions below.
        let _ = written;

        let links = load_current_links_for_observation(&pool, obs)
            .await
            .map_err(classify)
            .expect("read current");
        assert_eq!(
            links.len(),
            1,
            "the ambiguity is DATA, not a hole — an abstention IS a link row"
        );
        assert_eq!(links[0].outcome, "abstained");
        assert_eq!(links[0].abstention_cause.as_deref(), Some("ambiguous"));
        assert_eq!(links[0].rule_id, None, "an abstention names no rule");
        assert_eq!(links[0].interface_id, None);

        let candidates = load_link_candidates(&pool, link)
            .await
            .map_err(classify)
            .expect("read candidates");
        assert_eq!(candidates.len(), 2, "both candidates are readable");
        let mut expected = vec![(one.to_string(), ev_one), (two.to_string(), ev_two)];
        expected.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(candidates, expected, "each candidate carries its evidence");
    }

    /// AC2 — the DDL CHECKs fire. Each is the DDL-level echo of a type-level property.
    #[tokio::test]
    async fn the_ddl_checks_refuse_incoherent_links() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let obs = ObsId::from_uuid(uuid::Uuid::now_v7());

        // A match with no interface — `interface_id IS NULL` iff abstained.
        let no_interface = write_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::now_v7()),
            obs,
            None,
            &a_match("l1-exact-mac"),
            open_end(),
        )
        .await;
        assert_eq!(
            no_interface,
            Err(RepositoryError::Constraint("check")),
            "a match must name the interface it placed the observation on"
        );

        // An abstention that names an interface — the same CHECK from the other side.
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let iface = an_interface(&pool, l2, [0, 1, 2, 3, 4, 13]).await;
        let abstained_with_interface = write_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::now_v7()),
            obs,
            Some(iface),
            &an_abstention(IdentityAbstentionCause::AbsenceOfProof),
            open_end(),
        )
        .await;
        assert_eq!(
            abstained_with_interface,
            Err(RepositoryError::Constraint("check")),
            "an abstention places the observation nowhere"
        );

        // ⚠️ rule-XOR-cause can only be reached by going AROUND the adapter, and that is the
        // point. `insert_identity_link` derives the rule and the cause from one `match`, so it
        // cannot emit an incoherent pair — which makes the CHECK a second line of defence against
        // a future writer. Until these two inserts existed nothing measured it: dropping the
        // constraint left all 378 tests GREEN. `Decision::rule()` returns None exactly for an
        // abstention; this attacks that property from both sides.
        let abstained_naming_a_rule = sqlx::query(
            "INSERT INTO identity_link \
             (id, observation_id, interface_id, link_subject, outcome, rule_id, abstention_cause, \
              evidence, ruleset_version, decided_by, valid_from, valid_to) \
             VALUES (?, ?, NULL, ?, 'abstained', 'l1-exact-mac', 'ambiguous', '[]', 1, 'ENGINE', ?, ?)",
        )
        .bind(uuid::Uuid::now_v7().to_string())
        .bind(obs.to_string())
        .bind(ABSTAINED_SUBJECT)
        .bind(datetime_literal(at(1_700_000_000)))
        .bind(OPEN_END)
        .execute(&pool)
        .await
        .map_err(classify);
        assert_eq!(
            abstained_naming_a_rule.err(),
            Some(RepositoryError::Constraint("check")),
            "an abstention took no decision, so it names no rule"
        );

        let deciding_without_a_rule = sqlx::query(
            "INSERT INTO identity_link \
             (id, observation_id, interface_id, link_subject, outcome, rule_id, abstention_cause, \
              evidence, ruleset_version, decided_by, valid_from, valid_to) \
             VALUES (?, ?, ?, ?, 'match', NULL, NULL, '[]', 1, 'ENGINE', ?, ?)",
        )
        .bind(uuid::Uuid::now_v7().to_string())
        .bind(obs.to_string())
        .bind(iface.to_string())
        .bind(iface.to_string())
        .bind(datetime_literal(at(1_700_000_000)))
        .bind(OPEN_END)
        .execute(&pool)
        .await
        .map_err(classify);
        assert_eq!(
            deciding_without_a_rule.err(),
            Some(RepositoryError::Constraint("check")),
            "a decision names the rule that settled it"
        );

        // decided_by is a closed set.
        let bad_actor = insert_identity_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::now_v7()),
            obs,
            Some(iface),
            &a_match("l1-exact-mac"),
            &[obs],
            DecidedBy::Engine,
            at(1_700_000_000),
            open_end(),
        )
        .await;
        assert!(bad_actor.is_ok(), "the ENGINE token is accepted");
        let raw_bad_actor = sqlx::query(
            "INSERT INTO identity_link \
             (id, observation_id, interface_id, link_subject, outcome, rule_id, abstention_cause, \
              evidence, ruleset_version, decided_by, valid_from, valid_to) \
             VALUES (?, ?, ?, ?, 'match', 'l1-exact-mac', NULL, '[]', 1, 'SCANNER', ?, ?)",
        )
        .bind(uuid::Uuid::now_v7().to_string())
        .bind(obs.to_string())
        .bind(iface.to_string())
        .bind(iface.to_string())
        .bind("2023-11-14 22:13:20.000000")
        .bind("2023-11-14 22:13:20.000000")
        .execute(&pool)
        .await
        .map_err(classify);
        assert_eq!(
            raw_bad_actor.err(),
            Some(RepositoryError::Constraint("check")),
            "decided_by is ENGINE or OPERATOR — a scanner never decides identity"
        );
    }

    /// AC5 — `interface (l2_domain, mac_canon)` is NOT unique. A cloned MAC is two real interfaces
    /// sharing one address, and a UNIQUE would turn the case the engine must ABSTAIN on into a 500.
    /// Asserted, not `.expect()`ed: the assertion form is what carries M2's red.
    #[tokio::test]
    async fn two_interfaces_may_share_one_l1_key() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let mac = MacAddr([0xde, 0xad, 0xbe, 0xef, 0, 1]);

        insert_interface(
            &pool,
            InterfaceId::from_uuid(uuid::Uuid::now_v7()),
            l2,
            &mac,
            at(1_700_000_000),
            at(1_700_000_100),
        )
        .await
        .map_err(classify)
        .expect("the first interface");

        let second = insert_interface(
            &pool,
            InterfaceId::from_uuid(uuid::Uuid::now_v7()),
            l2,
            &mac,
            at(1_700_000_000),
            at(1_700_000_100),
        )
        .await
        .map_err(classify);
        assert_eq!(
            second,
            Ok(()),
            "a cloned MAC is two real interfaces — a UNIQUE here would 500 on the abstain case"
        );
    }

    /// The persisted tokens are pinned, every one of them. No database needed — this is the only
    /// mutation of the seven that reds without one.
    #[test]
    fn every_persisted_token_is_pinned() {
        assert_eq!(
            outcome_token(&Conclusion::Match {
                rule: RuleId("r".into())
            }),
            "match"
        );
        assert_eq!(
            outcome_token(&Conclusion::NoMatch {
                rule: RuleId("r".into())
            }),
            "no_match"
        );
        assert_eq!(
            outcome_token(&Conclusion::Abstained {
                cause: IdentityAbstentionCause::Ambiguous
            }),
            "abstained"
        );
        assert_eq!(
            cause_token(&IdentityAbstentionCause::Ambiguous),
            "ambiguous"
        );
        assert_eq!(
            cause_token(&IdentityAbstentionCause::AbsenceOfProof),
            "absence_of_proof"
        );
        assert_eq!(DecidedBy::Engine.token(), "ENGINE");
        assert_eq!(DecidedBy::Operator.token(), "OPERATOR");
    }

    /// The two sentinels are what they claim to be, and `link_subject` is derived from one place.
    #[test]
    fn the_two_sentinels_are_pinned() {
        assert_eq!(OPEN_END, "9999-12-31 23:59:59.999999");
        assert_eq!(ABSTAINED_SUBJECT, "00000000-0000-0000-0000-000000000000");
        assert_eq!(
            datetime_literal(open_end()),
            OPEN_END,
            "OPEN_END is reachable as a Timestamp"
        );
        let iface = InterfaceId::from_uuid(uuid::Uuid::now_v7());
        assert_eq!(link_subject_of(Some(iface)), iface.to_string());
        assert_eq!(link_subject_of(None), ABSTAINED_SUBJECT);
    }
}
