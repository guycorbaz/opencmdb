//! The `Clock` port — the only way time enters the domain.
//!
//! The engine is a pure function of its inputs (D19/D25): `observed_at` comes from the source,
//! and any "now" the engine needs is bound from a `Clock` at the composition root. core's
//! `chrono` has its `clock` feature OFF, so `Utc::now()` is not callable here — reading the
//! wall clock is a composition-root privilege, and a fixture/replay substitutes a different
//! `Clock` to make behaviour deterministic.

use crate::observation::Timestamp;

/// A source of the current instant. The one seam through which time reaches the domain.
pub trait Clock: Send + Sync {
    /// The current instant, as decided by whoever wired this clock in.
    fn now(&self) -> Timestamp;
}
