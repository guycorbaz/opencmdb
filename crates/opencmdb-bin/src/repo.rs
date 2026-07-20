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
    fn executor(&mut self) -> &mut MySqlConnection {
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
}
