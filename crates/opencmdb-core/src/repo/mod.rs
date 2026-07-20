//! The persistence contract — abstract, and free of `sqlx` (D47/D49).
//!
//! The write side lends a *unit of work* through a closure (`transact`); it never exposes a
//! raw transaction handle, and the unit has no `commit()` — an identity decision cannot be
//! split across two transactions because the method does not exist. On a deadlock, `transact`
//! fails `Contention` and the caller replays the whole closure: one retry path (NFR15).
//!
//! There is deliberately NO single `Reads` trait: `ReadRepository` is `&self`, a `WriteUnit`
//! is `&mut self`, and this crate cannot name `sqlx::Executor` — so the read query bodies live
//! in the adapter as free functions generic over `Executor`, and both sides delegate to them.

use std::future::Future;
use std::pin::Pin;

use thiserror::Error;

/// A boxed, `Send` future. Defined with `std` so `opencmdb-core` needs no `futures` crate.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// What can go wrong at the persistence boundary. Closed (D47): `sqlx::Error` is classified
/// into this in the adapter and dies there — core never names sqlx. `Contention` is the one
/// retryable case (NFR15); `Backend` is terminal and opaque BY DESIGN.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepositoryError {
    /// A lock-wait timeout or deadlock — the actor retries the transaction.
    #[error("contention — retry the transaction")]
    Contention,
    /// A database constraint was violated (the `&'static str` names which invariant).
    #[error("constraint violated: {0}")]
    Constraint(&'static str),
    /// A row that was required was not found.
    #[error("not found")]
    NotFound,
    /// Any other backend failure — terminal, non-retryable, opaque by design.
    #[error("backend error: {0}")]
    Backend(String),
}

/// A unit of work inside a transaction. It reads its own writes, and it has **no `commit()`**:
/// the transaction is committed (or rolled back) by `transact`, not by its user.
pub trait WriteUnit: Send {}

/// The write side of the repository. `transact` lends a `Unit` to a closure for the duration
/// of one transaction and returns its result; the transaction commits iff the closure returns
/// `Ok`. The closure is `for<'u> …` so it works for whatever transaction lifetime the adapter
/// chooses — an HRTB over the `Unit<'u>` GAT (D49).
#[allow(async_fn_in_trait)]
pub trait WriteRepository {
    /// The adapter's unit of work — opaque: no `sqlx::Transaction`, no `sqlx::Error` escapes.
    type Unit<'u>: WriteUnit + Send
    where
        Self: 'u;

    /// Run `f` inside a fresh transaction. Commit on `Ok`, roll back on `Err`. A `Contention`
    /// result means the caller should replay `f`.
    async fn transact<F, T>(&self, f: F) -> Result<T, RepositoryError>
    where
        F: for<'u> FnOnce(&'u mut Self::Unit<'u>) -> BoxFuture<'u, Result<T, RepositoryError>>
            + Send,
        T: Send;
}

/// The read side — a `&self` pool serving reads (the API, D21). A distinct type from
/// [`WriteRepository`], so the writer actor is constructed with the write side only and
/// cannot reach the read pool: read-your-own-writes as a constructor signature.
pub trait ReadRepository {}
