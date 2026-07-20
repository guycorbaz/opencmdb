//! The MariaDB adapter for the persistence contract (D49). This is the ONLY place `sqlx`
//! appears: `sqlx::Error` is classified into `RepositoryError` and dies here (D47), and the
//! read query bodies are free functions generic over `sqlx::Executor` that both the read side
//! and a unit of work delegate to — the query is written once.
//!
//! Skeleton (D49 story-1 bar): it COMPILES and is proven by a `transact` round-trip test.
//! The running app wires it in from Story 3.5 (ingestion) onward — hence `allow(dead_code)`.
#![allow(dead_code)]

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

/// Classify a `sqlx::Error` into the closed `RepositoryError` (D47) — the ONLY translation of
/// a backend error, and the only place a MariaDB error code is named.
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
}
