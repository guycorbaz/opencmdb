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
    /// An instant the caller supplied is EARLIER than one already stored for the same subject, so
    /// honouring it would run a version's interval backwards.
    ///
    /// It is its own variant rather than a [`Self::Constraint`] because the database's answer to
    /// this is an anonymous `CHECK` failure that names no cause a reader could act on — and because
    /// the same regression is SILENT on the branch where the decision did not change, so the guard
    /// has to exist above the DDL to give one condition one answer. Story 5.11 measured both
    /// branches.
    #[error("the supplied instant precedes the stored one")]
    InstantRegressed,
    /// One `ObsId` was supplied TWICE in a single pass carrying different decision-bearing content.
    ///
    /// An `ObsId` identifies an immutable observation, so two different contents under one id is a
    /// self-contradictory input and a caller bug. It is refused rather than resolved: the grouping
    /// walks the whole slice while the by-id lookup keeps the LAST copy, so silently picking one
    /// would let an observation be placed on an interface derived from a MAC its winning copy does
    /// not carry — and which copy wins would depend on arrival order, which is exactly what story
    /// 5.11b exists to rule out.
    ///
    /// It is its own variant rather than a [`Self::Constraint`] for the reason story 5.11
    /// established with [`Self::InstantRegressed`]: `Constraint` means *"a database constraint was
    /// violated"* by its own doc, and no database was consulted here — the input contradicts itself
    /// before any statement runs.
    ///
    /// ⚠️ A repeated IDENTICAL observation stays LEGAL. Callers do supply one, and two existing
    /// tests depend on it.
    #[error("one observation id was supplied twice with different content")]
    ContradictoryObservation,
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
