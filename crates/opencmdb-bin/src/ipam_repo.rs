//! The addressing plan's adapter — the four refusals a `CHECK` cannot express, and the store.
//!
//! # 🔴 Why this module exists at all, and why it is not in `repo.rs`
//!
//! `repo.rs` is at 1743 code lines of a 2000 ceiling — 87 %, not the 190 % this project's own
//! `CLAUDE.md` claimed until 2026-09-11 — and **the `file-size` gate cannot see it**: it stops at
//! the first `#[cfg(test)]`, which in `repo.rs` is a test-only helper at line 184, so the gate reads
//! 183 where 1743 are. ⚠️ The gate's own doc claims a file with test code above the marker *"would
//! over-count itself, never under-count"*; `repo.rs` falsifies that by 1560 lines.
//!
//! 🔑 *A gate that cannot measure a file is not permission to grow it.* A separate module costs
//! nothing, so the instruction is unconditional rather than resting on a number.
//!
//! # ✅ `#![allow(dead_code)]`, and there is none left
//!
//! Story 14.1 shipped this module with a blanket one because it had NO producer by its own
//! criterion; story 14.2 narrowed it to six item-level attributes when the READ path gained one;
//! **story 14.2b removed the last of them**, each as its route came to call the item it sat on.
//! There is now **no `allow(dead_code)` in this file at all**, and `clippy --workspace
//! --all-targets -D warnings` is green without one.
//!
//! 🔴 **The reason they went one at a time is not tidiness.** An attribute's `reason` said *"the
//! write path has no producer until story 14.2b"*, and the moment a route called the item, that
//! sentence was FALSE — *an allow whose reason is untrue is a false doc, not a deferral*. So each
//! left with the route that falsified it rather than in one sweep at the end.
//!
//! 🔑 **What the attribute was a trade about, kept because the next module will face it.** On
//! 2026-09-10 the OPPOSITE conclusion was taken one file over, correctly: `arp_ping.rs` had a
//! blanket `allow(dead_code)` deleted, because with it standing, severing the product's MAC read
//! left `neighbours()` and `neighbour::table()` completely unreferenced **with clippy still
//! green** — the compiler was the only thing watching that wiring, and the attribute blinded it.
//!
//! *An `allow` is a trade between "the compiler cannot see a producer that does not exist yet" and
//! "the compiler is the only thing watching this wiring". Say which one you are in.* This module
//! was in the first and has left it.
//!
//! ⚠️ **The counting history is kept because every retelling of it was wrong.** The register said
//! *eleven items*; a correction said *ten warnings covering fourteen*; measured on `38da035` it is
//! **eleven warnings covering fifteen items**, identically under `cargo build` and
//! `clippy --all-targets`. Story 14.2 left **six** attributes — and the first draft of that
//! sentence said *seven*, inside the paragraph correcting two other wrong counts of the same
//! figure. *A unit-sensitive sentence is where an unqualified number does the most damage.*

use std::net::Ipv4Addr;

use opencmdb_core::ipam::{IpPolicy, IpamError};
use sqlx::{Executor, MySql};

use opencmdb_core::repo::RepositoryError;

use crate::repo::classify;

/// How many characters the canonical IPv4 form occupies. `LENGTH()` on this is the family
/// discriminant the schema's `ip_range_same_family` CHECK compares — see `0007`'s header.
pub(crate) const IPV4_CANONICAL_LEN: usize = 15;

/// Render an address in this store's canonical form: `192.0.2.9` becomes `192.000.002.009`.
///
/// 🔑 **Lexicographic order is numeric order**, which is the whole reason the padding exists, and
/// it is D10's precedent applied to addresses. The registered defect at `inventory_view.rs:261` —
/// *"the address compares as a STRING, so `192.0.2.9` follows `192.0.2.10`"* — is caused by the
/// ABSENCE of padding, not by text.
pub(crate) fn canonical(addr: Ipv4Addr) -> String {
    let [a, b, c, d] = addr.octets();
    format!("{a:03}.{b:03}.{c:03}.{d:03}")
}

/// Read an address back out of its canonical form.
///
/// # Errors
///
/// [`IpamError::MalformedAddress`] when the value is not exactly this store's spelling — an
/// unpadded quad, a short octet, a trailing space. The DDL refuses those too (`0007`'s anchored
/// `RLIKE`); this is the second carrier, because a value can reach here from a backfill that went
/// around the adapter.
pub(crate) fn from_canonical(text: &str) -> Result<Ipv4Addr, IpamError> {
    if text.len() != IPV4_CANONICAL_LEN {
        return Err(IpamError::MalformedAddress);
    }
    let mut octets = [0_u8; 4];
    let mut parts = text.split('.');
    for octet in &mut octets {
        let part = parts.next().ok_or(IpamError::MalformedAddress)?;
        // Exactly three ASCII digits — `u8::from_str_radix` would take a sign and `parse` would
        // take `9` where `009` is the only spelling this store holds.
        if part.len() != 3 || !part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(IpamError::MalformedAddress);
        }
        *octet = part.parse().map_err(|_| IpamError::MalformedAddress)?;
    }
    if parts.next().is_some() {
        return Err(IpamError::MalformedAddress);
    }
    Ok(Ipv4Addr::from(octets))
}

/// A subnet as the plan declares it: a base address and a prefix length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Subnet {
    /// The NETWORK address — not any address inside the subnet.
    base: Ipv4Addr,
    /// How many leading bits the prefix fixes.
    prefix_len: u8,
}

impl Subnet {
    /// Refuse a subnet the arithmetic cannot answer questions about.
    ///
    /// # Errors
    ///
    /// - [`IpamError::PrefixLengthNotInFamily`] when the prefix cannot belong to IPv4.
    ///   🔴 The validation measured why this is a refusal rather than a lint: a `/64` accepted on an
    ///   IPv4 base makes [`Subnet::last`] compute `32 - 64` and **panic on subtract-with-overflow**.
    ///   *A schema that admits an impossible pair hands the arithmetic an impossible input* — and
    ///   `0007` cannot refuse it, because binding the prefix to the family needs the base's length.
    /// - [`IpamError::BaseIsNotTheNetworkAddress`] when the base carries host bits. `192.0.2.5/24`
    ///   is refused: containment is arithmetic on the base, and a base that is not the network
    ///   address makes every answer wrong in a way nothing downstream can detect.
    pub(crate) fn new(base: Ipv4Addr, prefix_len: u8) -> Result<Self, IpamError> {
        if prefix_len > 32 {
            return Err(IpamError::PrefixLengthNotInFamily);
        }
        let candidate = Subnet { base, prefix_len };
        if candidate.network() != base {
            return Err(IpamError::BaseIsNotTheNetworkAddress);
        }
        Ok(candidate)
    }

    /// The mask this prefix fixes. `prefix_len` is `<= 32` by construction ([`Subnet::new`]).
    fn mask(&self) -> u32 {
        if self.prefix_len == 0 {
            0
        } else {
            u32::MAX << (32 - self.prefix_len)
        }
    }

    /// The network address the base ought to be.
    fn network(&self) -> Ipv4Addr {
        Ipv4Addr::from(u32::from(self.base) & self.mask())
    }

    /// The last address the subnet covers — broadcast included, because the PLAN covers it and
    /// `infrastructure` is what names it.
    fn last(&self) -> Ipv4Addr {
        Ipv4Addr::from(u32::from(self.base) | !self.mask())
    }

    /// Whether an address falls inside this subnet.
    ///
    /// 🔑 **In Rust, never in SQL** (D10: *"all value comparison and normalization happens in
    /// Rust"*). `architecture.md:4847` (F57) asks that SQL-side comparison be costed before any
    /// epic reintroduces it; this story does not reintroduce it.
    pub(crate) fn contains(&self, addr: Ipv4Addr) -> bool {
        addr >= self.base && addr <= self.last()
    }

    /// Every address of the subnet, network and broadcast included, in numeric order.
    ///
    /// 🔑 **256 addresses, 254 hosts, and the two are not the same number** — story 6b.7 found the
    /// reference mock conflating them: it looped `0..256`, drew `.0` and `.255` as ordinary free
    /// cells, and its *next free address* panel then named the NETWORK address. The grid draws
    /// every address because the PLAN covers every address; what may be OFFERED is a separate
    /// question, answered by `CellState::offerable`.
    pub(crate) fn addresses(&self) -> impl Iterator<Item = Ipv4Addr> + use<> {
        let first = u32::from(self.network());
        let last = u32::from(self.last());
        (first..=last).map(Ipv4Addr::from)
    }

    /// How many addresses the subnet holds, network and broadcast included.
    ///
    /// 🔑 It is a `u64` because a `/0` holds 2³² addresses and a `u32` cannot say so. The count is
    /// what lets a caller refuse to DRAW a subnet before it has paid for drawing it — see
    /// `ipam_page::MAX_DRAWN_ADDRESSES`.
    pub(crate) fn size(&self) -> u64 {
        u64::from(u32::from(self.last()) - u32::from(self.network())) + 1
    }

    /// The subnet in CIDR notation, as an operator writes it — `192.0.2.0/24`.
    ///
    /// ⚠️ NOT the stored spelling: the store holds `192.000.002.000` so that lexicographic order is
    /// numeric order, and that padding is an implementation of ordering, never something to show.
    /// *A canonical form imposed for the machine is not a form to render.*
    pub(crate) fn cidr(&self) -> String {
        format!("{}/{}", self.base, self.prefix_len)
    }

    /// Whether this address is the subnet's network or broadcast address.
    ///
    /// 🔴 **A `/31` AND A `/32` HAVE NO EDGES, and the first draft said the comparison handled them
    /// "without a special arm" — it did, and the answer was WRONG.** Measured by the code review's
    /// edge layer: on a `/31` both addresses compare equal to a bound, so a point-to-point link
    /// declared `static` offered NEITHER of its two addresses, and a `/32` host route offered
    /// nothing at all — the two shapes an operator declares for exactly the addresses they mean to
    /// assign. **RFC 3021** says both addresses of a `/31` are usable, and a `/32` is a single host.
    ///
    /// 🔑 The prefix length is what settles it, so the arm is explicit rather than implied: there is
    /// no network/broadcast pair to reserve when the subnet is too small to hold one.
    pub(crate) fn is_edge(&self, addr: Ipv4Addr) -> bool {
        if self.prefix_len >= 31 {
            return false;
        }
        addr == self.network() || addr == self.last()
    }

    /// Whether any address of this subnet falls inside the closed interval `first..=last`.
    ///
    /// 🔑 An INTERVAL comparison and never a walk: a `/8` holds 16 777 216 addresses, and the one
    /// caller asks this question while rendering a page the ceiling refuses to draw.
    pub(crate) fn overlaps(&self, first: Ipv4Addr, last: Ipv4Addr) -> bool {
        last >= self.network() && first <= self.last()
    }
}

/// Prefix one of the plan's write statements with its lock-wait cap.
///
/// 🔴 **The server's own default is 50 s, and the handler's budget is 5 s** — story 14.2b's second
/// review measured a write dropped by its budget staying in `LOCK WAIT` on the server, holding its
/// pool connection, until whatever held the lock let go. So every statement of the plan that can
/// wait on a lock waits FOUR seconds at most: the server gives up first, the handler receives a real
/// 1205 as `Contention`, and the connection goes back to the pool (Guy, 2026-09-15). `SET STATEMENT`
/// scopes the cap to the one statement, so a pooled connection carries nothing to its next user.
macro_rules! capped {
    ($sql:literal) => {
        concat!("SET STATEMENT innodb_lock_wait_timeout=4 FOR ", $sql)
    };
}

/// Insert one subnet of the addressing plan.
///
/// # Errors
///
/// [`RepositoryError::Backend`] carrying an [`IpamError`]'s sentence when the subnet is not one the
/// arithmetic can answer for, or the `sqlx::Error` classified by [`classify`].
pub(crate) async fn insert_subnet<'e, E>(
    executor: E,
    id: &str,
    subnet: Subnet,
    label: &str,
) -> Result<(), RepositoryError>
where
    E: Executor<'e, Database = MySql>,
{
    // 🔴 The comment here read *"`Subnet` can only be built through `new`"* and it was FALSE: both
    // fields were `pub(crate)`, so any module in this crate could write the literal and reach
    // `contains` with `prefix_len = 64`, which panics on `32 - 64` — **the exact panic
    // `PrefixLengthNotInFamily` is documented to exist for**, measured by the review. And the silent
    // half: a `192.0.2.5/24` literal reports `192.0.2.1` as OUTSIDE the subnet.
    // The fields are private now (story 5.6's precedent: closed in the TYPE, not in a sentence),
    // which costs nothing — every use was already inside this module.
    // The re-validation below is kept for the caller holding a `Subnet` read back from the store.
    Subnet::new(subnet.base, subnet.prefix_len).map_err(ipam)?;
    sqlx::query(capped!(
        "INSERT INTO ip_subnet (id, base, prefix_len, label) \
         VALUES (?, ?, ?, ?)"
    ))
    .bind(id)
    .bind(canonical(subnet.base))
    .bind(subnet.prefix_len)
    .bind(label)
    .execute(executor)
    .await
    .map_err(classify)?;
    Ok(())
}

/// Read a subnet back.
///
/// # Errors
///
/// [`RepositoryError::NotFound`] when no such subnet exists; a backend error otherwise.
pub(crate) async fn load_subnet<'e, E>(executor: E, id: &str) -> Result<Subnet, RepositoryError>
where
    E: Executor<'e, Database = MySql>,
{
    let (base, prefix_len): (String, u8) =
        sqlx::query_as("SELECT base, prefix_len FROM ip_subnet WHERE id = ?")
            .bind(id)
            .fetch_optional(executor)
            .await
            .map_err(classify)?
            .ok_or(RepositoryError::NotFound)?;
    subnet_from_row(&base, prefix_len)
}

/// Insert one range, after the three refusals no `CHECK` can express.
///
/// # Errors
///
/// A [`RepositoryError::Backend`] carrying [`IpamError::RangeOutsideSubnet`] or
/// [`IpamError::RangeOverlapsAnother`], or the classified `sqlx::Error`.
///
/// ⚠️ **Raw SQL does NOT measure these two guards — it BYPASSES them.** Story 5.9's M3 (*"a guard
/// the adapter cannot violate is measured by raw SQL or by nothing"*) governs `0007`'s CHECKs and
/// not these: both rules compare two TABLES, a `CHECK` referencing another table is `ERROR 1901` on
/// MariaDB 10.11, and a raw insert simply succeeds. They are measured THROUGH this function, with a
/// raw insert as the control that shows the DDL does not refuse it.
pub(crate) async fn insert_range(
    conn: &mut sqlx::MySqlConnection,
    id: &str,
    subnet_id: &str,
    first: Ipv4Addr,
    last: Ipv4Addr,
    policy: IpPolicy,
    label: &str,
) -> Result<(), RepositoryError> {
    insert_range_pausing(
        conn,
        id,
        subnet_id,
        first,
        last,
        policy,
        label,
        std::future::ready(()),
    )
    .await
}

/// [`insert_range`] with a seam between the deciding read and the insert.
///
/// 🔑 **The seam is a FUTURE and not a flag**, so production passes `std::future::ready(())` and
/// the seam cannot rot for want of a caller. AC5's harness passes a 400 ms sleep and drives two of
/// these at once: without it the window is too short to observe, and *a race nobody can reproduce
/// is a race nobody can prove closed*.
#[allow(
    clippy::too_many_arguments,
    reason = "the seam is one argument on an already-wide row"
)]
async fn insert_range_pausing(
    conn: &mut sqlx::MySqlConnection,
    id: &str,
    subnet_id: &str,
    first: Ipv4Addr,
    last: Ipv4Addr,
    policy: IpPolicy,
    label: &str,
    after_the_deciding_read: impl std::future::Future<Output = ()>,
) -> Result<(), RepositoryError> {
    // 🔴 BEFORE the overlap scan, and the order is the finding: an inverted range inside a populated
    // subnet was refused as `RangeOverlapsAnother`. An empty interval overlaps nothing, so the
    // reason was wrong wherever a sibling happened to be in the way — and story 14.2 renders these
    // sentences to the operator. *A refusal that names the wrong rule is one nobody can act on.*
    // It is also before any statement, so a plainly impossible range costs no transaction.
    if last < first {
        return Err(ipam(IpamError::RangeBoundsInverted));
    }
    // 🔴 **A DEADLOCK VICTIM IS REPLAYED, and the reviews measured why it is owed.** Two ranges
    // defined at once in two DIFFERENT, empty subnets deadlocked 2 runs of 3: both subnet ids fall in
    // one gap of the non-unique `ip_range_subnet` index, the parent locks are different rows and
    // serialise nothing between them, and each transaction then needs an insert-intention lock the
    // other's gap lock blocks. InnoDB has already rolled the victim back, so replaying the whole
    // attempt is exactly NFR15's *"the caller replays the whole closure"*.
    // 🔴 **ONE replay was the first answer and the second review refuted it**: four writers in four
    // subnets left 11 writes of 40 answered `Contention`, the replay deadlocking again in lock-step
    // with the other victims. So up to [`MAX_DEADLOCK_REPLAYS`], each after a short RANDOM pause that
    // breaks the lock-step (Guy, 2026-09-15).
    // ⚠️ On 1213 alone: a lock-wait TIMEOUT (1205) is not replayed — it has already spent its wait —
    // and the seam is not replayed either: it exists to open ONE window for AC5's harness.
    let mut outcome = range_attempt(
        conn,
        id,
        subnet_id,
        first,
        last,
        policy,
        label,
        after_the_deciding_read,
    )
    .await;
    for replay in 1..=MAX_DEADLOCK_REPLAYS {
        if !matches!(outcome, Err(RangeAttempt::Deadlocked)) {
            break;
        }
        tracing::warn!(
            subnet = subnet_id,
            replay,
            "a range write was chosen as a deadlock victim — replaying it"
        );
        tokio::time::sleep(replay_pause(replay)).await;
        outcome = range_attempt(
            conn,
            id,
            subnet_id,
            first,
            last,
            policy,
            label,
            std::future::ready(()),
        )
        .await;
    }
    match outcome {
        Ok(()) => Ok(()),
        Err(RangeAttempt::Refused(error)) => Err(error),
        Err(RangeAttempt::Deadlocked) => Err(RepositoryError::Contention),
    }
}

/// How many times a range write chosen as a deadlock victim is replayed before `Contention`.
const MAX_DEADLOCK_REPLAYS: u32 = 3;

/// The pause before a replay: random, and growing with the replay.
///
/// ⚠️ **The randomness is DEFENSIVE and carried by no test, measured rather than assumed.** It was
/// added on the argument that victims replaying in lock-step meet again — but the four-writer
/// measurement that argued for more replays was of ONE replay, and mutation M2-2 (the pause set to
/// zero) left `four_ranges_at_once_in_four_subnets_all_land` GREEN: three immediate replays were
/// enough there. It is kept as Guy decided (2026-09-15), because a larger burst or a busier store
/// may lock-step where five rounds of four writers did not, and this doc says so instead of
/// crediting a measurement that never measured it. At most 49 ms × the replay number, so three
/// replays cost under 300 ms of a 5 s budget; the noise is `RandomState`'s per-instance keys, so no
/// dependency is taken for it.
fn replay_pause(replay: u32) -> std::time::Duration {
    use std::hash::{BuildHasher, Hasher};
    let noise = std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish();
    std::time::Duration::from_millis((10 + noise % 40) * u64::from(replay))
}

/// Why one attempt at a range write did not land.
enum RangeAttempt {
    /// InnoDB chose this transaction as a deadlock victim (1213) and rolled it back — the one
    /// outcome worth replaying.
    Deadlocked,
    /// Anything else: a domain refusal, a constraint, a lock-wait timeout, a backend fault.
    Refused(RepositoryError),
}

impl From<RepositoryError> for RangeAttempt {
    fn from(error: RepositoryError) -> Self {
        Self::Refused(error)
    }
}

/// Classify a `sqlx::Error` met inside a range attempt, keeping a deadlock apart from the rest.
fn attempt_error(error: sqlx::Error) -> RangeAttempt {
    if crate::repo::is_deadlock(&error) {
        RangeAttempt::Deadlocked
    } else {
        RangeAttempt::Refused(classify(error))
    }
}

/// One transaction of [`insert_range`]: lock, decide, write, commit.
#[allow(
    clippy::too_many_arguments,
    reason = "the seam is one argument on an already-wide row"
)]
async fn range_attempt(
    conn: &mut sqlx::MySqlConnection,
    id: &str,
    subnet_id: &str,
    first: Ipv4Addr,
    last: Ipv4Addr,
    policy: IpPolicy,
    label: &str,
    after_the_deciding_read: impl std::future::Future<Output = ()>,
) -> Result<(), RangeAttempt> {
    // 🔑 THE FUNCTION OWNS ITS TRANSACTION, and that is what makes the locks below mean anything:
    // in autocommit a `FOR UPDATE` is released at the end of its own statement, so read-decide-write
    // would still race with every lock in place. ⚠️ It must not be called from inside another
    // transaction: sqlx would open a SAVEPOINT there, so this `commit` would only release it and the
    // locks would live as long as the OUTER transaction chose. No caller does.
    let mut tx = sqlx::Connection::begin(&mut *conn)
        .await
        .map_err(attempt_error)?;
    // 🔴 THE PARENT ROW FIRST, THEN THE DECIDING READ, and the pair is Guy's arbitration of
    // 2026-09-12 taken on a measured matrix of five strategies (§1(d)). The two locks are NOT
    // redundant, which is why the same option was refused the day before as belt-and-braces:
    //   · the deciding read alone DEADLOCKS, 20 observations of 20 — `WHERE subnet_id = ?` is a
    //     non-unique index scan, so both transactions take compatible gap locks and each then needs
    //     an insert-intention lock. One row survives either way, and the loser is told *deadlock,
    //     retry* where the product knows the range overlaps;
    //   · the parent row alone is defeated by ONE ORDINARY DRY LINE — a non-locking `load_subnet`
    //     placed first fixes this transaction's REPEATABLE READ snapshot, and a locking read does
    //     not refresh it, so the sibling scan reads a world without the other range and both
    //     commit.
    // ⚠️ So the rule is not *"take a lock"* but *"take the parent row's lock BEFORE any read of
    // this subnet at all"*. `load_subnet_locked` exists so no caller can accidentally read first.
    //
    // 🔴 **NEITHER LOCK IS MEASURABLE ON ITS OWN, and the mutation pass is what found it.** Remove
    // the deciding read's `FOR UPDATE` and AC5's harness stays GREEN, because the parent row alone
    // serialises entry; add the DRY line with both locks in place and it stays green too, because
    // the range lock catches it. Only the COMPOSITE — the DRY line together with an unlocked
    // deciding read — reds, with `[Ok(()), Ok(())]`. *Two guards that each mask the other's
    // mutation are two guards nothing measures*, so the pair is carried behaviourally by that
    // composite and each half is named individually by
    // `both_reads_of_a_subnet_under_write_take_their_lock`, a source guard that names the cause
    // where the composite names only the symptom (story 6b.11's AC5, as amended).
    let decided: Result<(), RangeAttempt> = async {
        let subnet = load_subnet_locked(&mut tx, subnet_id).await?;
        if !subnet.contains(first) || !subnet.contains(last) {
            return Err(ipam(IpamError::RangeOutsideSubnet).into());
        }
        let siblings: Vec<(String, String)> = sqlx::query_as(capped!(
            "SELECT first_addr, last_addr FROM ip_range WHERE subnet_id = ? FOR UPDATE"
        ))
        .bind(subnet_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(attempt_error)?;
        after_the_deciding_read.await;
        for (sibling_first, sibling_last) in siblings {
            let sibling_first = from_canonical(&sibling_first).map_err(ipam)?;
            let sibling_last = from_canonical(&sibling_last).map_err(ipam)?;
            // Two closed intervals overlap unless one ends before the other begins.
            if first <= sibling_last && sibling_first <= last {
                return Err(ipam(IpamError::RangeOverlapsAnother).into());
            }
        }
        sqlx::query(capped!(
            "INSERT INTO ip_range (id, subnet_id, first_addr, last_addr, policy, label) \
         VALUES (?, ?, ?, ?, ?, ?)"
        ))
        .bind(id)
        .bind(subnet_id)
        .bind(canonical(first))
        .bind(canonical(last))
        .bind(policy.as_str())
        .bind(label)
        .execute(&mut *tx)
        .await
        .map_err(attempt_error)?;
        Ok(())
    }
    .await;
    match decided {
        Ok(()) => tx.commit().await.map_err(attempt_error),
        // ⚠️ ROLLED BACK HERE, EXPLICITLY — and the second review sized what that buys. A refused
        // attempt left to `Drop` only QUEUES its rollback; sqlx pings a connection returned to the
        // pool, which flushes it, so on the handler path the locks go with the request either way
        // (measured through `StoreIpamWrite`: the next write on the subnet answered in 2.56 ms). What
        // the explicit rollback changes is a HELD connection — the race test keeps both of its own,
        // and there the queued rollback kept the loser's locks for 50 s (MR1). The first repair
        // called that a production defect; it was a property of the holder.
        // 🔴 **And a FAILED rollback ends the replays**: the connection's transaction state is then
        // unknown, so a deadlock is answered as `Contention` rather than replayed on it (the second
        // review's blind layer).
        Err(refused) => match tx.rollback().await {
            Ok(()) => Err(refused),
            Err(error) => {
                tracing::warn!(
                    %error,
                    "rolling back a refused range write failed — not replaying on this connection"
                );
                Err(match refused {
                    RangeAttempt::Deadlocked => RangeAttempt::Refused(RepositoryError::Contention),
                    other @ RangeAttempt::Refused(_) => other,
                })
            }
        },
    }
}

/// [`load_subnet`], but taking the subnet ROW's lock — the entry point every write to a subnet's
/// children must pass through first.
///
/// 🔑 It is a second SQL literal rather than `load_subnet` with four characters appended, and the
/// redundancy is DELIBERATE in this codebase's sense: the two differ by the one thing that matters,
/// and a shared helper taking a `lock: bool` would let a caller pass `false` at the exact place
/// where `false` is the defect. What IS shared is the decoding, in [`subnet_from_row`].
///
/// 🔴 **It answers a [`RangeAttempt`], not a `RepositoryError`**: its `sqlx::Error` went through
/// `classify`, so a deadlock on THIS read was wrapped as a refusal and never replayed, while the
/// comment above the replays promised *"on 1213"* for the whole attempt (the second review's blind
/// layer). It goes through `attempt_error` like every other statement of the attempt.
async fn load_subnet_locked(
    tx: &mut sqlx::MySqlConnection,
    id: &str,
) -> Result<Subnet, RangeAttempt> {
    let row: Option<(String, u8)> = sqlx::query_as(capped!(
        "SELECT base, prefix_len FROM ip_subnet WHERE id = ? FOR UPDATE"
    ))
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(attempt_error)?;
    let (base, prefix_len) = row.ok_or(RangeAttempt::Refused(RepositoryError::NotFound))?;
    Ok(subnet_from_row(&base, prefix_len)?)
}

/// Turn a stored `(base, prefix_len)` pair into the subnet the arithmetic can answer for.
fn subnet_from_row(base: &str, prefix_len: u8) -> Result<Subnet, RepositoryError> {
    let base = from_canonical(base).map_err(ipam)?;
    Subnet::new(base, prefix_len).map_err(ipam)
}

/// Insert one individually defined address.
///
/// # Errors
///
/// [`IpamError::AddressOutsideSubnet`] through [`RepositoryError::Backend`], or the classified
/// `sqlx::Error`. ⚠️ The same instrument note as [`insert_range`] applies: raw SQL bypasses this.
pub(crate) async fn insert_address(
    conn: &mut sqlx::MySqlConnection,
    id: &str,
    subnet_id: &str,
    addr: Ipv4Addr,
    label: &str,
) -> Result<(), RepositoryError> {
    let subnet = load_subnet(&mut *conn, subnet_id).await?;
    if !subnet.contains(addr) {
        return Err(ipam(IpamError::AddressOutsideSubnet));
    }
    sqlx::query(capped!(
        "INSERT INTO ip_address (id, subnet_id, addr, label) VALUES (?, ?, ?, ?)"
    ))
    .bind(id)
    .bind(subnet_id)
    .bind(canonical(addr))
    .bind(label)
    .execute(&mut *conn)
    .await
    .map_err(classify)?;
    Ok(())
}

/// Delete one defined address, by the id the plan gave it.
///
/// 🔑 **It takes no lock of its OWN and computes no refusal, and that is a property of the SCHEMA
/// rather than a convenience**: `ip_address` is a leaf — nothing references it — so removing a row
/// can orphan nothing. Every other destructive gesture of this story has a parent to protect; this
/// one has none, and saying so is what stops a later reader adding a lock *"for symmetry"*.
///
/// ⚠️ **The statement still takes locks, which is why it is CAPPED** — the review found the two
/// sentences reading as a contradiction four lines apart, and the first one was the loose half. A
/// `DELETE` takes an exclusive lock on the row it matches and a shared one on the `ip_subnet` parent
/// it references, so it can wait behind a concurrent write of that subnet — uncapped, for the
/// server's 50 s default, with the handler's connection in hand.
///
/// # Errors
///
/// [`RepositoryError::NotFound`] when no row carries that id — an id that names nothing is the
/// operator pressing a control for a record someone else has already removed, and answering `Ok`
/// there would tell them a deletion happened that did not. Otherwise the classified `sqlx::Error`.
///
/// ⚠️ **"No row carries that id" is true MODULO TRAILING SPACES, and this holds for every
/// id-addressed statement in this module** — the five writes all match on `WHERE id = ?`, and the
/// plan's id columns are `ascii_bin`, which is a **PAD SPACE** collation: `'abc'` and `'abc   '`
/// compare equal (measured at the review). The row reached is still the intended one, so nothing
/// wrong is deleted; what is weaker than it reads is the *uniqueness* of the spelling, on five
/// routes that take the id from an operator-submitted form. Story 14.1 met the same collation on the
/// policy token and closed it with an INTEGER length comparison; here the consequence is benign and
/// the sentence is corrected rather than the comparison.
pub(crate) async fn delete_address(
    conn: &mut sqlx::MySqlConnection,
    id: &str,
) -> Result<(), RepositoryError> {
    let done = sqlx::query(capped!("DELETE FROM ip_address WHERE id = ?"))
        .bind(id)
        .execute(&mut *conn)
        .await
        .map_err(classify)?;
    if done.rows_affected() == 0 {
        return Err(RepositoryError::NotFound);
    }
    Ok(())
}

/// Delete one subnet — refused by the DATABASE while it still holds ranges or addresses.
///
/// 🔴 **THE REFUSAL IS A FOREIGN KEY AND NOT ARITHMETIC, which is what makes this route cheap**
/// (Guy's decision 2, 2026-09-16). `ip_range_subnet_fk` and `ip_address_subnet_fk` both reference
/// `ip_subnet(id)`, so MariaDB answers `ERROR 1451` on its own: no count to take, no lock to hold
/// while taking it, and therefore **no race** — where a range's refusal must be computed and
/// serialised ([`delete_range`]).
///
/// ⚠️ **The caller must not serve that refusal as *"no such subnet"*.** `classify` maps a foreign-key
/// violation to `Constraint("foreign_key")`, and `ipam_write`'s `constraint_refusal` mapped every one
/// of those to a 404 *unknown subnet* — the wrong sentence for *this subnet still holds ranges*. The
/// route splits it; this function only reports what the database said.
///
/// # Errors
///
/// [`RepositoryError::NotFound`] when no row carries that id; `Constraint("foreign_key")` when the
/// subnet still has children; otherwise the classified `sqlx::Error`.
pub(crate) async fn delete_subnet(
    conn: &mut sqlx::MySqlConnection,
    id: &str,
) -> Result<(), RepositoryError> {
    let done = sqlx::query(capped!("DELETE FROM ip_subnet WHERE id = ?"))
        .bind(id)
        .execute(&mut *conn)
        .await
        .map_err(classify)?;
    if done.rows_affected() == 0 {
        return Err(RepositoryError::NotFound);
    }
    Ok(())
}

/// Correct one defined address — its address, its label, or both.
///
/// 🔑 **The containment rule is RE-VALIDATED, because an edit can leave the subnet where an insert
/// could not.** `insert_address` checks `subnet.contains(addr)` once, at birth; moving an address
/// afterwards is the same rule asked again, and skipping it here would let an edit write what the
/// insert refuses — *the shape story 6.3 named: a rule held on one path and not on its twin.*
///
/// 🔴 **The parent row is locked BEFORE THE READ THAT DECIDES** ([`load_subnet_locked`]'s rule,
/// story 14.2b's arbitration of 2026-09-12): that read must not run before the lock, or a concurrent
/// write of the same subnet fixes this transaction's REPEATABLE READ snapshot and the decision is
/// taken against a world that has already moved.
///
/// ⚠️ **It is NOT locked first outright, and the review caught three sites claiming it was.** The
/// child row is locked first of all — and it has to be, because the parent's identity is only known
/// once this row has been read: `subnet_id` lives on the child. So a strict *parent first* is
/// unreachable here, and the weaker sentence above is the one that is true.
///
/// ⚠️ **The residual is declared rather than removed**: [`insert_range`] takes the parent and then
/// locks sibling ranges, while this path takes a child row and then waits for the parent — a
/// lock-order inversion between two shipped gestures on one subnet. It degrades to
/// [`RepositoryError::Contention`] rather than to corruption (`repo.rs` maps `1213`/`1205`), so the
/// operator meets *try again* and never a wrong answer; closing it properly means re-deciding where
/// each gesture takes its first lock, which is more than a correction may take on itself.
///
/// # Errors
///
/// [`RepositoryError::NotFound`] when the id names no address; [`IpamError::AddressOutsideSubnet`]
/// through [`RepositoryError::Ipam`] when the new address leaves its subnet — ⚠️ the review found
/// this line naming `Backend`, which `ipam` never produces and which would have sent the route to a
/// backend sentence where this module's whole contract is *refused BY NAME*; a `Constraint` when the
/// subnet already defines that address; otherwise the classified `sqlx::Error`.
pub(crate) async fn update_address(
    conn: &mut sqlx::MySqlConnection,
    id: &str,
    addr: Ipv4Addr,
    label: &str,
) -> Result<(), RepositoryError> {
    let mut tx = sqlx::Connection::begin(&mut *conn)
        .await
        .map_err(classify)?;
    let written = async {
        let subnet_id: String = sqlx::query_as(capped!(
            "SELECT subnet_id FROM ip_address WHERE id = ? FOR UPDATE"
        ))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(classify)?
        .map(|(subnet_id,): (String,)| subnet_id)
        .ok_or(RepositoryError::NotFound)?;
        // The parent's lock, through the one entry point that cannot be called without taking it.
        let subnet = load_subnet_locked(&mut tx, &subnet_id)
            .await
            .map_err(refused)?;
        if !subnet.contains(addr) {
            return Err(ipam(IpamError::AddressOutsideSubnet));
        }
        sqlx::query(capped!(
            "UPDATE ip_address SET addr = ?, label = ? WHERE id = ?"
        ))
        .bind(canonical(addr))
        .bind(label)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(classify)?;
        Ok(())
    }
    .await;
    settle_plan_write(tx, written, "address edit").await
}

/// Translate a [`RangeAttempt`] into the error a caller that does not replay can answer.
///
/// 🔑 [`load_subnet_locked`] speaks `RangeAttempt` because [`insert_range`] replays a deadlock once;
/// the corrections of story 14.4 do not replay, so a deadlock reaches the operator as `Contention` —
/// *nothing about the request is wrong, try again* — rather than as a backend failure that leaks the
/// driver's vocabulary. One translation, three callers: this is not the deliberate kind of
/// redundancy, and three copies of a `match` is where the fourth one drifts.
fn refused(attempt: RangeAttempt) -> RepositoryError {
    match attempt {
        RangeAttempt::Refused(error) => error,
        RangeAttempt::Deadlocked => RepositoryError::Contention,
    }
}

/// Delete one range — **refused by name while it still holds addresses defined inside it**.
///
/// 🔴 **THE REFUSAL IS COMPUTED, AND IT RACES, WHICH IS WHY THE PARENT ROW IS LOCKED FIRST.**
/// `ip_address` carries a foreign key to `ip_subnet` and NONE to `ip_range` (`0007:153`): an address
/// falls inside a range by ARITHMETIC, so MariaDB raises nothing here — unlike [`delete_subnet`],
/// whose refusal is a key. Story 14.4's validation measured the gap between counting and deleting:
/// with a lock held, a concurrent `insert_address` waits **1404.8 ms** and the two acts serialise;
/// with NO lock at all the inserter returns in single-digit milliseconds and **an address lands
/// inside the range the same transaction is deleting**. ⚠️ **That pair was first written here as
/// *"without the parent row's lock"*, and the mutation pass below refuted the attribution**: removing
/// that one lock changes nothing, because a second carrier holds the property up. The figures are
/// real; what they separate is *locked* from *unlocked*, never one lock from the other.
///
/// ⚠️ **THE SERIALISATION HAS TWO INDEPENDENT CARRIERS, AND EITHER ALONE SUFFICES — measured, after
/// this sentence first claimed only one.** [`insert_address`] takes no lock of its own (*"NO LOCK
/// HERE"*, its own doc), and what makes it wait is BOTH of: the parent row's exclusive lock, which
/// its foreign key on `ip_subnet` must share; AND this function's address scan, whose next-key/gap
/// locks over that subnet's index range block an `INSERT` there with no foreign key involved.
/// 🔴 Removing either one alone reds NOTHING (M-T3, M-T3b); removing both takes the inserter from
/// over a second to **2.86 ms** while the deleter's 400 ms pause runs alone (M-T3c). *A property
/// with two carriers is one a mutation of either measures nothing about* — story 6.5's M6 and
/// 6.4b's P3. So an address write that stopped touching `ip_subnet` would NOT lose this in silence
/// while the scan stands, which is what the earlier single-mechanism sentence got wrong.
///
/// 🔑 **The containment comparison is in RUST** (D10), over the addresses the locked read returns —
/// the padded canonical form makes a SQL `BETWEEN` tempting and D10 is what keeps the comparison
/// where the domain can be tested without a store.
///
/// # Errors
///
/// [`RepositoryError::NotFound`] when the id names no range; [`IpamError::RangeStillHoldsAddresses`]
/// through [`RepositoryError::Ipam`] when it still holds one; [`RepositoryError::Contention`] on a
/// deadlock (this path does not replay); otherwise the classified `sqlx::Error`.
pub(crate) async fn delete_range(
    conn: &mut sqlx::MySqlConnection,
    id: &str,
) -> Result<(), RepositoryError> {
    delete_range_pausing(conn, id, std::future::ready(())).await
}

/// [`delete_range`] with a seam between the deciding read and the `DELETE`.
///
/// 🔴 **ITS OWN SEAM, and `insert_range_pausing` is deliberately NOT the instrument.** That one is a
/// private seam inside [`insert_range`]'s body, parameterised on THAT function's decide-then-write;
/// a delete decides something else (containment, in Rust) and writes something else. Reusing it
/// would have meant widening a function until it served two gestures — and this module has already
/// measured what that costs: *one ordinary DRY line brings the race back*.
///
/// 🔑 **The seam sits exactly in the window §1(c) measured**: after the addresses have been counted
/// and before the range is removed. That is where a concurrent `insert_address` slips a row into the
/// space being deleted — **2.86 ms** with both locks gone and the address lands, against over a
/// second while either one stands (M-T3c against M-T3/M-T3b). A seam anywhere else would open a
/// window nothing writes through. ⚠️ This sentence read *"once the parent row is locked"* until the
/// review; that attributes to one lock what two carry.
///
/// ⚠️ **No replay, unlike the insert's.** A deadlock victim here answers [`RepositoryError::Contention`]
/// through [`refused`]: the insert replays because a deadlock between two definitions in two empty
/// subnets is an artefact of gap locking that costs the operator nothing to retry invisibly, while a
/// deletion is an act they asked for once and must be told about rather than re-attempted silently.
///
/// # Errors
///
/// As [`delete_range`].
async fn delete_range_pausing(
    conn: &mut sqlx::MySqlConnection,
    id: &str,
    after_the_deciding_read: impl std::future::Future<Output = ()>,
) -> Result<(), RepositoryError> {
    let mut tx = sqlx::Connection::begin(&mut *conn)
        .await
        .map_err(classify)?;
    let written = async {
        let (subnet_id, first, last): (String, String, String) = sqlx::query_as(capped!(
            "SELECT subnet_id, first_addr, last_addr FROM ip_range WHERE id = ? FOR UPDATE"
        ))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(classify)?
        .ok_or(RepositoryError::NotFound)?;
        let first = from_canonical(&first).map_err(ipam)?;
        let last = from_canonical(&last).map_err(ipam)?;
        // The parent row BEFORE THE DECIDING READ — `range_attempt`'s rule, and for its reason: a
        // read of this subnet taken before the lock fixes the transaction's snapshot and the
        // decision is then taken against a world that has already moved. ⚠️ Not *first outright*:
        // the range row above had to be read to learn which subnet this is.
        load_subnet_locked(&mut tx, &subnet_id)
            .await
            .map_err(refused)?;
        let held: Vec<(String,)> = sqlx::query_as(capped!(
            "SELECT addr FROM ip_address WHERE subnet_id = ? FOR UPDATE"
        ))
        .bind(&subnet_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(classify)?;
        // ⚠️ The scan is over the whole SUBNET, so a row this build cannot read ANYWHERE in it stops
        // a range delete. Failing closed is right; the refusal must not then name a rule about an
        // address the operator never touched, so the unreadable row is named in the log and the
        // gesture refuses as a backend fault rather than as `MalformedAddress`.
        for (addr,) in held {
            let Ok(addr) = from_canonical(&addr) else {
                tracing::warn!(
                    stored_addr = %addr,
                    range_id = %id,
                    "an ip_address row this build cannot read blocks a range delete in its subnet"
                );
                return Err(RepositoryError::Backend(
                    "an address row of this subnet cannot be read back".to_string(),
                ));
            };
            if first <= addr && addr <= last {
                return Err(ipam(IpamError::RangeStillHoldsAddresses));
            }
        }
        // 🔴 THE WINDOW. Production passes a future that is already ready, so this costs nothing and
        // cannot rot for want of a caller; the harness passes a sleep, because *a race nobody can
        // reproduce is a race nobody can prove closed*.
        after_the_deciding_read.await;
        sqlx::query(capped!("DELETE FROM ip_range WHERE id = ?"))
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(classify)?;
        Ok(())
    }
    .await;
    settle_plan_write(tx, written, "range delete").await
}

/// Correct one range — its bounds, its policy, its label.
///
/// 🔴 **IT CANNOT REUSE `insert_range`'s SIBLING SCAN, and the validation measured why.** That scan
/// reads every range of the subnet and compares each against the proposed interval; for an edit the
/// row being edited is IN that result, so it overlaps itself and **every legal widening is refused**
/// (measured: widening `.010–.099` to `.010–.120` returns `r1` with `overlaps = 1`). The clause
/// `id <> ?` is what an insert cannot have — it has no id in the table yet — so this is a second
/// literal rather than a parameter on the first.
///
/// 🔑 **A second SQL literal, deliberately**, on [`load_subnet_locked`]'s own precedent: a shared
/// helper taking `exclude: Option<&str>` would let a caller pass `None` at exactly the place where
/// `None` is the defect. What is shared is the overlap ARITHMETIC, which is three lines of Rust and
/// is the part a test can hold.
///
/// ⚠️ **The refusals are the insert's, re-asked**: an edit can move a range out of its subnet or
/// invert its bounds just as a creation can, and a rule held on one path and not on its twin is the
/// shape story 6.3 named. The parent row is locked before the deciding read, for [`delete_range`]'s
/// reason.
///
/// 🔴 **AND THE DELETE'S REFUSAL IS RE-ASKED TOO — the review found that it was not, and all three
/// layers reached it** (Guy's arbitration, 2026-09-16). Asking only *"which of the INSERT's refusals
/// apply to an edit?"* left the door shut and the window open: [`delete_range`] refuses while the
/// range holds a defined address, and an edit that moved the range off that address answered `Ok`,
/// leaving it under no range at all — after which the delete the product had just refused succeeded.
/// MEASURED by probe: control refuses, `update_range .10-.20 → .30-.40` returns `Ok`, the address is
/// orphaned. *The product refused the honest gesture and permitted the discreet one.*
///
/// 🔑 **The rule is the smallest one that closes it**: an address that was inside the OLD bounds must
/// still be inside the NEW ones. So **widening is always legal** (nothing is abandoned), a policy or
/// label correction is always legal (the ground does not move), and only a move or a shrink that
/// would leave an address behind is refused — with [`delete_range`]'s own sentence, because it is
/// the same rule. ⚠️ Refusing every edit of an occupied range was REFUSED as the coarser reading: it
/// would make correcting a label impossible on any range holding an address, a cost the arbitration
/// did not ask for.
///
/// # Errors
///
/// [`RepositoryError::NotFound`] when the id names no range; [`IpamError::RangeBoundsInverted`],
/// [`IpamError::RangeOutsideSubnet`] or [`IpamError::RangeOverlapsAnother`] through
/// [`RepositoryError::Ipam`]; [`RepositoryError::Contention`] on a deadlock; otherwise the classified
/// `sqlx::Error`.
pub(crate) async fn update_range(
    conn: &mut sqlx::MySqlConnection,
    id: &str,
    first: Ipv4Addr,
    last: Ipv4Addr,
    policy: IpPolicy,
    label: &str,
) -> Result<(), RepositoryError> {
    let mut tx = sqlx::Connection::begin(&mut *conn)
        .await
        .map_err(classify)?;
    let written = async {
        // 🔑 The row's CURRENT bounds travel with its subnet id, because the orphan rule below needs
        // to know what this range holds TODAY — the new bounds alone cannot say what is being
        // abandoned.
        let (subnet_id, held_first, held_last): (String, String, String) = sqlx::query_as(capped!(
            "SELECT subnet_id, first_addr, last_addr FROM ip_range WHERE id = ? FOR UPDATE"
        ))
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(classify)?
        .ok_or(RepositoryError::NotFound)?;
        let held_first = from_canonical(&held_first).map_err(ipam)?;
        let held_last = from_canonical(&held_last).map_err(ipam)?;
        let subnet = load_subnet_locked(&mut tx, &subnet_id)
            .await
            .map_err(refused)?;
        if last < first {
            return Err(ipam(IpamError::RangeBoundsInverted));
        }
        if !subnet.contains(first) || !subnet.contains(last) {
            return Err(ipam(IpamError::RangeOutsideSubnet));
        }
        // 🔴 `id <> ?` — the one clause an insert cannot carry. Without it a range overlaps itself.
        let siblings: Vec<(String, String)> = sqlx::query_as(capped!(
            "SELECT first_addr, last_addr FROM ip_range WHERE subnet_id = ? AND id <> ? FOR UPDATE"
        ))
        .bind(&subnet_id)
        .bind(id)
        .fetch_all(&mut *tx)
        .await
        .map_err(classify)?;
        for (sibling_first, sibling_last) in siblings {
            let sibling_first = from_canonical(&sibling_first).map_err(ipam)?;
            let sibling_last = from_canonical(&sibling_last).map_err(ipam)?;
            // Two closed intervals overlap unless one ends before the other begins.
            if first <= sibling_last && sibling_first <= last {
                return Err(ipam(IpamError::RangeOverlapsAnother));
            }
        }
        // 🔴 **THE DELETE'S REFUSAL, RE-ASKED** (Guy, 2026-09-16). An address inside the OLD bounds
        // that the NEW ones do not hold is abandoned, which is exactly what `delete_range` refuses —
        // so it earns the same sentence. ⚠️ Decided AFTER the overlap scan on purpose: an edit that
        // collides with a neighbour has a more basic fault, and the operator should be told that one.
        let held: Vec<(String,)> = sqlx::query_as(capped!(
            "SELECT addr FROM ip_address WHERE subnet_id = ? FOR UPDATE"
        ))
        .bind(&subnet_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(classify)?;
        for (addr,) in held {
            let Ok(addr) = from_canonical(&addr) else {
                tracing::warn!(
                    stored_addr = %addr,
                    range_id = %id,
                    "an ip_address row this build cannot read blocks a range edit in its subnet"
                );
                return Err(RepositoryError::Backend(
                    "an address row of this subnet cannot be read back".to_string(),
                ));
            };
            let was_held = held_first <= addr && addr <= held_last;
            let still_held = first <= addr && addr <= last;
            if was_held && !still_held {
                return Err(ipam(IpamError::RangeStillHoldsAddresses));
            }
        }
        sqlx::query(capped!(
            "UPDATE ip_range SET first_addr = ?, last_addr = ?, policy = ?, label = ? WHERE id = ?"
        ))
        .bind(canonical(first))
        .bind(canonical(last))
        .bind(policy.as_str())
        .bind(label)
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(classify)?;
        Ok(())
    }
    .await;
    settle_plan_write(tx, written, "range edit").await
}

/// Commit a correction, or roll it back naming what it was.
///
/// 🔑 `settle` in `ipam_write.rs` does this for the write ROUTES (a private item there, so it is
/// named rather than linked — an intra-doc link to it is one `cargo doc` cannot resolve, and this
/// project already carries a registered backlog of exactly those); this is its adapter-side twin,
/// and the rollback is EXPLICIT for the reason story 14.2b measured: a dropped `Transaction` only
/// QUEUES its rollback, so on a HELD connection the loser's locks outlive the refusal.
///
/// ⚠️ **The commit-failure path issues no rollback, and the review asked why.** A `COMMIT` that fails
/// has already ended the transaction at the server — there is nothing left to roll back, and `tx` is
/// consumed by value here, so no second gesture is available or owed. What the caller loses is the
/// distinction between *refused and clean* and *the commit itself failed*: both arrive as an `Err`,
/// the second classified from the driver. Said rather than left to be inferred from an absence.
async fn settle_plan_write(
    tx: sqlx::Transaction<'_, MySql>,
    written: Result<(), RepositoryError>,
    what: &str,
) -> Result<(), RepositoryError> {
    match written {
        Ok(()) => tx.commit().await.map_err(classify),
        Err(refusal) => {
            if let Err(error) = tx.rollback().await {
                tracing::warn!(%error, %what, "rolling back a refused plan correction failed");
            }
            Err(refusal)
        }
    }
}

// ⚠️ **`addresses_in` — the per-subnet defined-address read — was REMOVED at story 14.3b's code
// review, and this note is here so the next reader knows it was a decision.** The grid is derived
// from the PLAN-WIDE ranges and addresses since then (decision 2 applied to the cells, not only to
// the offer), so the function had no production caller at all and clippy said so under
// `-D warnings`. Kept only for its own test, it would have been SQL nothing runs, verified by a
// guard where the defect cannot occur — this epic's dominant class. Story 14.4 may need a
// per-subnet read for release; writing it then is cheaper than keeping this one warm for a use it
// had not got (`ipam_write.rs`'s own rule about `WriteRoute::paths`). Registered.

/// Every subnet in the plan, in numeric order of its base address.
///
/// 🔑 The order is the STORE's: the padded spelling makes lexicographic order numeric, so no caller
/// sorts and no caller can forget to.
///
/// # Errors
///
/// The classified `sqlx::Error`, or [`IpamError`] when a stored row is not this store's canonical
/// spelling or does not describe a subnet — reachable only by a write that went around this module.
pub(crate) async fn list_subnets<'e, E>(
    executor: E,
) -> Result<Vec<(String, Subnet, String)>, RepositoryError>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(String, String, u8, String)> = sqlx::query_as(
        "SELECT id, base, prefix_len, label FROM ip_subnet ORDER BY base, prefix_len",
    )
    .fetch_all(executor)
    .await
    .map_err(classify)?;
    // 🔴 **ONE UNREADABLE ROW MUST NOT TAKE THE WHOLE SCREEN WITH IT, and it did.** `0007`
    // deliberately admits `prefix_len` up to 128 (IPv6 forward-compat) and cannot check that a base
    // is its own network address — both are same-row rules the ADAPTER owns. So a single
    // schema-legal row (`prefix_len = 64`, or a base carrying host bits) made this function return
    // `Err`, and `/ipam` answered **500 for every subnet, including the healthy ones** — measured
    // by the code review's edge layer, with a log that named no id, a body reading *"the data
    // behind it is intact; the fault is in the display"* (backwards here), and a `/diagnostic` that
    // keeps no error history. *A row nobody can find, breaking a page nobody can read.*
    //
    // 🔑 The unreadable row is now SKIPPED AND NAMED: the screen draws what it can, and the log
    // carries the id and the stored spelling so the row can be found and fixed. ⚠️ The trade is
    // stated: a silent skip would hide a real defect, so it is a `warn` with the id in it, never a
    // `debug`. A row that cannot be read is a row the operator never declared through the product.
    let mut subnets = Vec::with_capacity(rows.len());
    for (id, base, prefix_len, label) in rows {
        match from_canonical(&base).and_then(|base| Subnet::new(base, prefix_len)) {
            Ok(subnet) => subnets.push((id, subnet, label)),
            Err(error) => tracing::warn!(
                subnet_id = %id,
                stored_base = %base,
                stored_prefix_len = prefix_len,
                %error,
                "skipping an ip_subnet row this build cannot read — the rest of the plan is drawn"
            ),
        }
    }
    Ok(subnets)
}

/// Every range of the plan, in every subnet, as `(first, last, policy)`.
///
/// 🔑 **Plan-wide, for story 14.3b's decision 2**: the most protective range decides across
/// overlapping ranges and NESTED subnets, so the audit of one subnet reads the ranges of all of them.
///
/// # Errors
///
/// [`RepositoryError`] on a backend failure or a row this build cannot read — the same contract as
/// [`ranges_in`].
pub(crate) async fn plan_ranges<'e, E>(
    executor: E,
) -> Result<Vec<(Ipv4Addr, Ipv4Addr, IpPolicy)>, RepositoryError>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(String, String, String)> =
        sqlx::query_as("SELECT first_addr, last_addr, policy FROM ip_range ORDER BY first_addr")
            .fetch_all(executor)
            .await
            .map_err(classify)?;
    // 🔴 **ONE UNREADABLE ROW MUST NOT TAKE EVERY PAGE WITH IT — `list_subnets`'s finding, one table
    // over, and this read is WORSE placed because it is PLAN-WIDE.** Measured by the code review's
    // blind layer: a single `ip_range` row a raw write left unreadable — a non-canonical bound, or a
    // policy token `ascii_bin`'s PAD SPACE collation accepts (`'static '`) — made this return `Err`,
    // and with it `/ipam` answered 500 for EVERY subnet and the address check answered 500 for every
    // address. *A row in one subnet breaking the page of all the others.*
    //
    // 🔑 Skipped and NAMED, exactly as `list_subnets` does it: the screen draws what it can and the
    // log carries the stored spelling so the row can be found. ⚠️ A `warn`, never a `debug` — a
    // silent skip would hide a real defect, and the audit is then reading an INCOMPLETE plan: an
    // address a skipped `reserved` range protected can be offered, which is why this must be loud.
    let mut ranges = Vec::with_capacity(rows.len());
    for (first, last, policy) in rows {
        match read_range_row(&first, &last, &policy) {
            Ok(range) => ranges.push(range),
            Err(error) => tracing::warn!(
                stored_first = %first,
                stored_last = %last,
                stored_policy = %policy,
                %error,
                "skipping an ip_range row this build cannot read — the rest of the plan is drawn"
            ),
        }
    }
    Ok(ranges)
}

/// Read one `ip_range` row into the plan's vocabulary.
///
/// # Errors
///
/// [`IpamError`] behind a [`RepositoryError`] when a bound is not canonical, or a backend-shaped
/// error naming the policy token when it is not one this build knows.
fn read_range_row(
    first: &str,
    last: &str,
    policy: &str,
) -> Result<(Ipv4Addr, Ipv4Addr, IpPolicy), RepositoryError> {
    Ok((
        from_canonical(first).map_err(ipam)?,
        from_canonical(last).map_err(ipam)?,
        policy_from_token(policy)?,
    ))
}

/// Every address an `ip_address` row names, in every subnet — decision 2's plan-wide half.
///
/// # Errors
///
/// [`RepositoryError`] on a backend failure or a row this build cannot read.
pub(crate) async fn plan_addresses<'e, E>(executor: E) -> Result<Vec<Ipv4Addr>, RepositoryError>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(String,)> = sqlx::query_as("SELECT addr FROM ip_address ORDER BY addr")
        .fetch_all(executor)
        .await
        .map_err(classify)?;
    // Skipped and NAMED, for [`plan_ranges`]'s reason: this read is plan-wide, so one unreadable
    // row took every `/ipam` page and every address check down with it. ⚠️ The consequence here is
    // the mirror one: a skipped `ip_address` row is an address the plan holds and the offer no
    // longer knows about, so it can be proposed — loud on purpose.
    let mut defined = Vec::with_capacity(rows.len());
    for (addr,) in rows {
        match from_canonical(&addr) {
            Ok(parsed) => defined.push(parsed),
            Err(error) => tracing::warn!(
                stored_addr = %addr,
                %error,
                "skipping an ip_address row this build cannot read — the rest of the plan is drawn"
            ),
        }
    }
    Ok(defined)
}

/// Every range defined in one subnet, in numeric order of its first address.
///
/// # Errors
///
/// The classified `sqlx::Error`, or [`IpamError`] when a stored bound is not canonical or the
/// stored policy token is not one this build knows — the second is reachable by a raw write, the
/// `ascii_bin` PAD SPACE collation accepting `'static '` where [`IpPolicy::as_str`] does not.
pub(crate) async fn ranges_in<'e, E>(
    executor: E,
    subnet_id: &str,
) -> Result<Vec<(Ipv4Addr, Ipv4Addr, IpPolicy, String)>, RepositoryError>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(String, String, String, String)> = sqlx::query_as(
        "SELECT first_addr, last_addr, policy, label FROM ip_range WHERE subnet_id = ? \
         ORDER BY first_addr",
    )
    .bind(subnet_id)
    .fetch_all(executor)
    .await
    .map_err(classify)?;
    rows.into_iter()
        .map(|(first, last, policy, label)| {
            let first = from_canonical(&first).map_err(ipam)?;
            let last = from_canonical(&last).map_err(ipam)?;
            let policy = policy_from_token(&policy)?;
            Ok((first, last, policy, label))
        })
        .collect()
}

/// The subnet's ranges, each with the id a correction names.
///
/// 🔑 **The id is the whole difference from [`ranges_in`], and it is what a CONTROL needs.** A list
/// the operator can only read needs bounds, a policy and a label; a list they can edit or delete
/// needs to say WHICH row — so this reader exists for the rail's controls and `ranges_in` stays as
/// it is for everything that only displays. ⚠️ Two readers over one table is the kind of redundancy
/// this project keeps deliberately only when something pins it: what pins this one is that the
/// display path must not carry ids into pages that offer no gesture.
///
/// ⚠️ **NOT `capped!`, and that is a decision rather than an omission** — the review found it
/// unwritten. Every capped statement in this module either writes or takes a lock, and can therefore
/// wait behind one; this is a plain consistent read under REPEATABLE READ, which takes no row lock
/// and waits for nothing. It is also why the guard that counts *"the plan's lockable statements"*
/// does not see these two: they are not lockable. The day either grows a `FOR UPDATE`, it earns the
/// cap and the count in the same act.
///
/// # Errors
///
/// A row this build cannot read back — a non-canonical address or an unknown policy token — or the
/// classified `sqlx::Error` when the query itself fails, which is the likelier of the two and was
/// missing from this list.
pub(crate) async fn correctable_ranges_in<'e, E>(
    executor: E,
    subnet_id: &str,
) -> Result<Vec<(String, Ipv4Addr, Ipv4Addr, IpPolicy, String)>, RepositoryError>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, first_addr, last_addr, policy, label FROM ip_range WHERE subnet_id = ? \
         ORDER BY first_addr",
    )
    .bind(subnet_id)
    .fetch_all(executor)
    .await
    .map_err(classify)?;
    rows.into_iter()
        .map(|(id, first, last, policy, label)| {
            let first = from_canonical(&first).map_err(ipam)?;
            let last = from_canonical(&last).map_err(ipam)?;
            let policy = policy_from_token(&policy)?;
            Ok((id, first, last, policy, label))
        })
        .collect()
}

/// The subnet's defined addresses, each with the id a correction names.
///
/// ⚠️ **THIS IS NOT THE `addresses_in` STORY 14.3b REMOVED, and the difference is the CONSUMER.**
/// That one fed the AUDIT, which compares what the network shows against what the plan claims — and
/// it was removed because the audit reads the sighting summary and the plan's whole address set,
/// never one subnet's. This one feeds the RAIL: the list of records the operator can correct on the
/// screen they are looking at. The note left at the removal site warns against re-adding the reader
/// for the audit's sake, and that warning stands.
///
/// ⚠️ **It is a CONVENTION, not a barrier, and the review was right to press the difference.** This
/// function is `pub(crate)`, its name does not say *rail only*, and nothing stops the audit path
/// calling it — the discipline is two comments, which is a tripwire against the ordinary gesture and
/// not a constraint. Story 5.12's precedent: say which of the two it is.
///
/// ⚠️ **Not `capped!`**, for [`correctable_ranges_in`]'s stated reason.
///
/// # Errors
///
/// A row whose address is not in this store's canonical spelling, or the classified `sqlx::Error`
/// when the query itself fails.
pub(crate) async fn correctable_addresses_in<'e, E>(
    executor: E,
    subnet_id: &str,
) -> Result<Vec<(String, Ipv4Addr, String)>, RepositoryError>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(String, String, String)> =
        sqlx::query_as("SELECT id, addr, label FROM ip_address WHERE subnet_id = ? ORDER BY addr")
            .bind(subnet_id)
            .fetch_all(executor)
            .await
            .map_err(classify)?;
    rows.into_iter()
        .map(|(id, addr, label)| Ok((id, from_canonical(&addr).map_err(ipam)?, label)))
        .collect()
}

/// The one reading of a stored policy token, and it is EXACT.
///
/// 🔴 It compares against [`IpPolicy::as_str`] with no trimming and no case folding, because
/// `ascii_bin` is a PAD SPACE collation: the schema's `IN (...)` CHECK accepts `'static '`, and so
/// would `= TRIM(...)`. The schema's own defence is an INTEGER comparison
/// (`LENGTH(policy) = LENGTH(TRIM(policy))`, `0007:142`); this is the second carrier, on the
/// reasoning `from_canonical` states — *a value can reach here from a backfill that went around
/// the adapter*.
///
/// # Errors
///
/// [`IpamError::MalformedAddress`] is deliberately NOT reused; an unknown policy is a row this
/// build cannot render, so it surfaces as a backend failure naming the token.
fn policy_from_token(token: &str) -> Result<IpPolicy, RepositoryError> {
    IpPolicy::ALL
        .into_iter()
        .find(|policy| policy.as_str() == token)
        .ok_or_else(|| {
            RepositoryError::Backend(format!(
                "stored policy token is not one this build knows: {token:?}"
            ))
        })
}

/// Carry an [`IpamError`] across the frontier.
///
/// ✅ **The seam D47 made awkward on purpose is now closed.** Story 14.1 carried these refusals as
/// `RepositoryError::Backend(String)` and said so here, naming story 14.2 as the one that would
/// earn the variant; this is that story, and [`RepositoryError::Ipam`] is that variant.
///
/// ⚠️ **What the variant does NOT buy, stated so nobody reads more into it than it gives**: adding
/// it produced **zero** compiler errors outside this module's own tests — `RepositoryError` is not
/// `#[non_exhaustive]` and no exhaustive `match` traverses it — so nothing forces a handler to
/// distinguish these refusals. The obligation is carried by a test over what a handler can
/// RECEIVE, which is deliberately larger than this enum.
fn ipam(error: IpamError) -> RepositoryError {
    RepositoryError::Ipam(error)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use sqlx::MySqlPool;

    /// Connect and migrate. `None` when `DATABASE_URL` is unset.
    ///
    /// 🔴 **Every caller takes [`crate::DB_TEST_LOCK`] first.** Story 6.5's review measured the cost
    /// of not doing it: seven new tests without the lock gave **6/6 RED on a virgin store where the
    /// parent commit was 6/6 green** — the shape CI's own `Tests` step runs — through a `Dirty(2)`
    /// from concurrent `migrate!` calls. *The per-test ids are a remedy for a symptom; the lock is
    /// the remedy for the cause.*
    ///
    /// ⚠️ **The suite reports the same counts with and without a store, and the clock is the only
    /// tell.** Every figure this story records names the store it was taken against.
    pub(crate) async fn ipam_fixture() -> Option<MySqlPool> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping ipam test: DATABASE_URL unset");
            return None;
        };
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        Some(pool)
    }

    /// Remove one subnet and everything that points at it, so a test can be re-run against a store
    /// that kept the last run's rows. Children first — the foreign keys point that way.
    pub(crate) async fn forget_subnet(pool: &MySqlPool, id: &str) {
        for statement in [
            "DELETE FROM ip_address WHERE subnet_id = ?",
            "DELETE FROM ip_range WHERE subnet_id = ?",
            "DELETE FROM ip_subnet WHERE id = ?",
        ] {
            sqlx::query(statement)
                .bind(id)
                .execute(pool)
                .await
                .expect("forget the subnet");
        }
    }

    /// 🔴 **Each store-backed test owns its own CIDR, and the review measured why.** Three of them
    /// built `192.0.2.0/24` under different ids; a test that panics skips its trailing cleanup, its
    /// subnet survives, and the NEXT test's `insert_subnet` then dies on `ip_subnet_cidr` — not on
    /// the thing under test. Measured: one mutation reported **red 3** in a full run and **red 1**
    /// when each test ran alone on a virgin store, so the recorded carrier counts were inflated by
    /// collateral; and a single leftover row made a PRISTINE tree red, with *which* test reddening
    /// depending on run order — which is what makes it read as flakiness.
    /// ⚠️ Not claimed as the cause of issue #38 or of Epic 6's registered non-determinism: it is a
    /// named, reproducible cause of non-determinism in THIS story's tests, and nothing more.
    fn v4(text: &str) -> Ipv4Addr {
        text.parse().expect("a literal address")
    }

    // ── The canonical form, pure ────────────────────────────────────────────────────────────────

    /// 🔑 **The property the whole representation was chosen for**, asserted without a store: the
    /// canonical order is the numeric order, and the UNPADDED form is the control that fails.
    /// `inventory_view.rs:261` registers that defect for the product's other address column.
    #[test]
    fn the_canonical_form_sorts_numerically_and_the_unpadded_one_does_not() {
        let addresses = ["192.0.2.9", "192.0.2.10", "192.0.2.100", "192.0.2.1"];

        let mut canonical_order: Vec<String> = addresses.iter().map(|a| canonical(v4(a))).collect();
        canonical_order.sort();
        assert_eq!(
            canonical_order,
            vec![
                "192.000.002.001",
                "192.000.002.009",
                "192.000.002.010",
                "192.000.002.100"
            ],
            "the padded form sorts as the numbers do — this is the reason the padding exists"
        );

        // THE CONTROL. Without it this test says nothing: a sort that happens to be right proves
        // nothing about why the padding is there.
        let mut raw: Vec<&str> = addresses.to_vec();
        raw.sort();
        assert_eq!(
            raw,
            vec!["192.0.2.1", "192.0.2.10", "192.0.2.100", "192.0.2.9"],
            "and the unpadded form puts .9 LAST — the registered defect, reproduced on purpose"
        );
    }

    /// One address has ONE spelling, and this half refuses every other one.
    #[test]
    fn only_the_canonical_spelling_reads_back() {
        assert_eq!(from_canonical("192.000.002.009"), Ok(v4("192.0.2.9")));
        for wrong in [
            "192.0.2.9",        // unpadded — the ordinary mistake
            "192.000.002.009 ", // a trailing space, which `ascii_bin` compares as equal
            "192.000.002.09",   // a short octet
            "192.000.002.0090", // a long one
            "19a.000.002.009",  // a letter
            "192.000.002",      // three octets
            "+92.000.002.009",  // a sign, which `parse` on a padded octet would not take anyway
            "",
        ] {
            assert_eq!(
                from_canonical(wrong),
                Err(IpamError::MalformedAddress),
                "{wrong:?} is not this store's spelling"
            );
        }
    }

    /// 🔴 **The refusal the validation measured as a PANIC.** A `/64` on an IPv4 base makes the
    /// containment arithmetic compute `32 - 64`; `0007` cannot refuse it, because binding the
    /// prefix to the family needs the base's length.
    #[test]
    fn a_prefix_that_cannot_belong_to_the_family_is_refused() {
        assert_eq!(
            Subnet::new(v4("192.0.2.0"), 64),
            Err(IpamError::PrefixLengthNotInFamily)
        );
        assert!(Subnet::new(v4("192.0.2.0"), 24).is_ok());
        assert!(
            Subnet::new(v4("0.0.0.0"), 0).is_ok(),
            "a /0 is arithmetic, not an error"
        );
    }

    /// A base carrying host bits is refused: containment is arithmetic on the base, and a wrong
    /// base makes every answer wrong in a way nothing downstream can detect.
    #[test]
    fn a_base_that_is_not_the_network_address_is_refused() {
        assert_eq!(
            Subnet::new(v4("192.0.2.5"), 24),
            Err(IpamError::BaseIsNotTheNetworkAddress)
        );
        assert!(Subnet::new(v4("192.0.2.0"), 24).is_ok());
    }

    /// Containment covers the whole plan, broadcast included — `infrastructure` is what names it.
    #[test]
    fn containment_covers_the_network_and_broadcast_addresses() {
        let subnet = Subnet::new(v4("192.0.2.0"), 24).expect("a subnet");
        assert!(
            subnet.contains(v4("192.0.2.0")),
            "the network address is in the plan"
        );
        assert!(subnet.contains(v4("192.0.2.255")), "and so is broadcast");
        assert!(subnet.contains(v4("192.0.2.9")));
        assert!(
            !subnet.contains(v4("192.0.3.0")),
            "and the next subnet is not"
        );
        assert!(!subnet.contains(v4("192.0.1.255")));
    }

    // ── The schema ──────────────────────────────────────────────────────────────────────────────

    /// AC3 — the domain and the schema name the same SET.
    ///
    /// 🔴 Story 6.5's **M8** is why the word SET is load-bearing: comparing COUNTS left two variants
    /// pointing at one token green. ***A count is not a set.***
    #[tokio::test]
    async fn the_policy_domain_agrees_with_the_schema() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        let (clause,): (String,) = sqlx::query_as(
            "SELECT CHECK_CLAUSE FROM information_schema.CHECK_CONSTRAINTS \
             WHERE CONSTRAINT_SCHEMA = DATABASE() AND CONSTRAINT_NAME = 'ip_range_policy_domain'",
        )
        .fetch_one(&pool)
        .await
        .expect("the CHECK exists and is readable");

        let named: std::collections::BTreeSet<&str> =
            IpPolicy::ALL.iter().map(IpPolicy::as_str).collect();
        assert_eq!(
            named.len(),
            IpPolicy::ALL.len(),
            "two `IpPolicy` variants persist as the same token — one of them can never be read \
             back, and a count of four would have hidden it"
        );
        let quoted: std::collections::BTreeSet<&str> =
            clause.split('\'').skip(1).step_by(2).collect();
        assert_eq!(
            quoted, named,
            "the schema's CHECK and `IpPolicy` name different sets — a value the schema accepts and \
             no variant names is a value nothing can read back, and the reverse is a write that \
             fails in production. Clause: {clause}"
        );
    }

    /// AC5's DDL half, and AC4b — measured by RAW SQL, because the adapter cannot produce any of
    /// these. Story 5.9's **M3**: *a guard the adapter cannot violate is measured by raw SQL or by
    /// nothing.* Every row here carries a CONTROL that is accepted.
    #[tokio::test]
    async fn the_schema_refuses_what_the_adapter_can_never_send() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        // 🔴 **CLEAN AT THE START, and every probe row has an id of its OWN.** The review measured
        // what the first version did: all six refusals inserted under the literal id `'t-x'`, the
        // cleanup ran only for `t-ddl`, and a panic skips the trailing cleanup entirely. So over a
        // BROKEN schema this test reddened ONCE — and on the next run the accepted row was already
        // there, the primary key refused the retry, `is_err()` could not tell *the CHECK refused
        // this* from *the PK refused this*, and the test went green for ever on the same store.
        // 🔑 *A guard that heals itself into a pass is worse than no guard*: CI is safe (a fresh
        // database per run) and the LOCAL reading is not, which is where this project measures its
        // mutations.
        // ⚠️ The three statements are spelled out rather than built with `format!`: sqlx 0.9's
        // `SqlSafeStr` is implemented for `&'static str` only, which refuses a dynamically
        // assembled query at compile time. A good refusal, met here for a cleanup.
        for probe in ["t-x1", "t-x2", "t-x3", "t-x4", "t-x5", "t-x6"] {
            for statement in [
                "DELETE FROM ip_address WHERE id = ?",
                "DELETE FROM ip_range WHERE id = ?",
                "DELETE FROM ip_subnet WHERE id = ?",
            ] {
                sqlx::query(statement)
                    .bind(probe)
                    .execute(&pool)
                    .await
                    .expect("clear any residue an earlier failed run left behind");
            }
        }
        forget_subnet(&pool, "t-ddl").await;
        sqlx::query("INSERT INTO ip_subnet (id, base, prefix_len, label) VALUES (?,?,?,?)")
            .bind("t-ddl")
            .bind("192.000.002.000")
            .bind(24_u8)
            .bind("the control subnet")
            .execute(&pool)
            .await
            .expect("a canonical subnet is accepted — the control for every refusal below");

        let refusals: [(&str, &str, &str); 6] = [
            (
                "an unpadded base",
                "INSERT INTO ip_subnet (id, base, prefix_len, label) VALUES ('t-x1','192.0.2.0',24,'x')",
                "the canonical form is imposed, not merely intended",
            ),
            (
                "a prefix beyond any family",
                "INSERT INTO ip_subnet (id, base, prefix_len, label) VALUES ('t-x2','010.000.000.000',200,'x')",
                "128 is the only bound this shape admits",
            ),
            (
                "bounds out of order",
                "INSERT INTO ip_range (id,subnet_id,first_addr,last_addr,policy,label) \
              VALUES ('t-x3','t-ddl','192.000.002.100','192.000.002.010','static','x')",
                "a range that ends before it begins",
            ),
            (
                "a policy the domain does not name",
                "INSERT INTO ip_range (id,subnet_id,first_addr,last_addr,policy,label) \
              VALUES ('t-x4','t-ddl','192.000.002.010','192.000.002.100','STATIC','x')",
                "`ascii_bin` is case-sensitive and the token set is closed",
            ),
            (
                "a policy with a trailing space",
                "INSERT INTO ip_range (id,subnet_id,first_addr,last_addr,policy,label) \
              VALUES ('t-x5','t-ddl','192.000.002.010','192.000.002.100','static ','x')",
                "🔴 `ascii_bin` is PAD SPACE, so `IN (...)` and `= TRIM(...)` both ACCEPT this — the \
              carrier is the INTEGER comparison of lengths, which is why it looks redundant",
            ),
            (
                "an address with a trailing space",
                "INSERT INTO ip_address (id,subnet_id,addr,label) \
              VALUES ('t-x6','t-ddl','192.000.002.009 ','x')",
                "the same PAD SPACE trap, closed by the RLIKE's `$` anchor",
            ),
        ];
        for (what, statement, why) in refusals {
            let outcome = sqlx::query(statement).execute(&pool).await;
            assert!(outcome.is_err(), "the schema accepted {what} — {why}");
        }

        // THE CONTROL: a legal range and a legal address both land.
        sqlx::query(
            "INSERT INTO ip_range (id,subnet_id,first_addr,last_addr,policy,label) \
             VALUES ('t-ddl-r','t-ddl','192.000.002.010','192.000.002.100','dhcp-pool','pool')",
        )
        .execute(&pool)
        .await
        .expect(
            "a legal range is accepted — without this the six refusals above could all be one \
                 broken table",
        );
        forget_subnet(&pool, "t-ddl").await;
    }

    /// 🔴 **The two holes the first pattern had, both INSIDE AC4b's promise, closed and pinned.**
    ///
    /// Two review layers reached them independently and a third deduced the second's consequence
    /// from the diff alone:
    ///
    /// - **`$` is not end-of-string in MariaDB's `RLIKE`** — it matches before a final newline. A
    ///   trailing `\n` was a SECOND accepted spelling of every address, and neither UNIQUE key
    ///   refused the pair: two rows, one address, lengths 15 and 16. It also INVERTED the ordering
    ///   this representation was chosen for (`0x0A` sorts before a space), and one poisoned row made
    ///   the defined-address read return `Err` for the whole subnet, losing the good rows with the
    ///   bad. ⚠️ That read was `addresses_in`, removed at story 14.3b's code review; the plan-wide
    ///   [`plan_addresses`] inherited the hazard and answers it the other way — it SKIPS the
    ///   unreadable row and names it, because one poisoned row was taking down every `/ipam` page.
    /// - **`[0-9]{3}` bounds the SHAPE and not the VALUE**, so `999.999.999.999` was storable raw —
    ///   and unreadable back, which is the same blinding.
    ///
    /// ⚠️ A trailing line terminator is how a bulk import or a shell `$(…)` poisons a text column:
    /// the ordinary gesture, and the exact population this CHECK exists for.
    #[tokio::test]
    async fn one_address_has_exactly_one_spelling_in_the_store() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        forget_subnet(&pool, "t-spell").await;
        sqlx::query("INSERT INTO ip_subnet (id, base, prefix_len, label) VALUES (?,?,?,?)")
            .bind("t-spell")
            .bind("192.000.002.000")
            .bind(24_u8)
            .execute(&pool)
            .await
            .ok();
        sqlx::query("INSERT INTO ip_subnet (id, base, prefix_len, label) VALUES (?,?,?,'spell')")
            .bind("t-spell")
            .bind("192.000.002.000")
            .bind(24_u8)
            .execute(&pool)
            .await
            .expect("the canonical subnet — the control for every refusal below");

        for (what, spelling) in [
            (
                "a trailing NEWLINE, which `$` accepts and `\\z` refuses",
                "192.000.002.009\n",
            ),
            (
                "a trailing space, which `ascii_bin` compares as equal",
                "192.000.002.009 ",
            ),
            (
                "an octet out of range, which bounds the shape and not the value",
                "999.999.999.999",
            ),
            ("an octet just over the top", "256.000.000.000"),
            ("the unpadded form", "192.0.2.9"),
            ("a leading newline", "\n192.000.002.009"),
        ] {
            let outcome =
                sqlx::query("INSERT INTO ip_address (id, subnet_id, addr, label) VALUES (?,?,?,?)")
                    .bind(format!("sp-{}", spelling.len()))
                    .bind("t-spell")
                    .bind(spelling)
                    .bind("second spelling")
                    .execute(&pool)
                    .await;
            assert!(
                outcome.is_err(),
                "the store accepted {what} — one address would then have two spellings, and the \
                 UNIQUE key does not refuse the pair"
            );
        }

        // THE CONTROL, and without it the six refusals above could all be one broken table.
        sqlx::query("INSERT INTO ip_address (id, subnet_id, addr, label) VALUES (?,?,?,?)")
            .bind("sp-ok")
            .bind("t-spell")
            .bind("192.000.002.009")
            .bind("the one spelling")
            .execute(&pool)
            .await
            .expect("the canonical form is still accepted");
        // …and the two extremes of the octet range, which the alternation must not have clipped.
        for edge in ["000.000.000.000", "255.255.255.255"] {
            sqlx::query(
                "INSERT INTO ip_subnet (id, base, prefix_len, label) VALUES (?,?,0,'edge')",
            )
            .bind(format!("edge-{edge}"))
            .bind(edge)
            .execute(&pool)
            .await
            .expect("the alternation accepts both ends of the octet range");
            sqlx::query("DELETE FROM ip_subnet WHERE id = ?")
                .bind(format!("edge-{edge}"))
                .execute(&pool)
                .await
                .ok();
        }
        forget_subnet(&pool, "t-spell").await;
    }

    /// ⚠️ **`ip_range_same_family` is VACUOUS TODAY, and this test is what says so out loud.**
    ///
    /// All three review layers reached it: the canonical pattern admits exactly ONE width, so every
    /// row the two canonical CHECKs accept already satisfies the family check, and **no mutation can
    /// be built that reds it**. *A guard placed where the defect cannot occur reads as coverage and
    /// is none.*
    ///
    /// 🔑 It is kept rather than deleted because the rule becomes real the day FR25 adds a
    /// 39-character alternative — and THIS test is what will red on that day, so the constraint
    /// stops being decoration at the moment it stops being vacuous, rather than when someone
    /// remembers.
    #[tokio::test]
    async fn the_family_check_is_implied_until_a_second_width_exists() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        let (clause,): (String,) = sqlx::query_as(
            "SELECT CHECK_CLAUSE FROM information_schema.CHECK_CONSTRAINTS \
             WHERE CONSTRAINT_SCHEMA = DATABASE() AND CONSTRAINT_NAME = 'ip_range_first_canonical'",
        )
        .fetch_one(&pool)
        .await
        .expect("the canonical CHECK exists and is readable");

        // One alternation group per octet and no second width: every accepted value is 15 long.
        assert_eq!(
            clause.matches("25[0-5]").count(),
            2,
            "the canonical pattern is expected to hold ONE octet alternation reused for the tail \
             ({clause}). If this changed, read the next assertion — the family check may have just \
             become load-bearing."
        );
        assert!(
            !clause.contains("[0-9a-f]") && !clause.contains(':'),
            "the canonical pattern admits a SECOND address family, so `ip_range_same_family` is no \
             longer implied by it — it has become a real guard, and it now needs a test of its own \
             and a row in the refusal array. Clause: {clause}"
        );
    }

    /// AC4 — the ORDER the store itself returns, which is the padding's whole purpose.
    ///
    /// ⚠️ It read back through `addresses_in` until story 14.3b's code review removed that function
    /// with its last production caller; it reads the PLAN-WIDE `plan_addresses` now, which is what
    /// the grid and the offer both use. The property is the store's `ORDER BY` and is unchanged —
    /// *the test follows the reader the product has, not the one it used to have*.
    #[tokio::test]
    async fn the_store_returns_addresses_in_numeric_order() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        forget_subnet(&pool, "t-order").await;
        let mut conn = pool.acquire().await.expect("a connection");
        insert_subnet(
            &mut *conn,
            "t-order",
            Subnet::new(v4("198.51.100.0"), 24).unwrap(),
            "order",
        )
        .await
        .expect("the subnet");
        for (id, addr) in [
            ("o1", "198.51.100.100"),
            ("o2", "198.51.100.9"),
            ("o3", "198.51.100.10"),
        ] {
            insert_address(&mut conn, id, "t-order", v4(addr), "n")
                .await
                .expect("the address");
        }
        let read = plan_addresses(&mut *conn).await.expect("read back");
        assert_eq!(
            read,
            vec![
                v4("198.51.100.9"),
                v4("198.51.100.10"),
                v4("198.51.100.100")
            ],
            "the STORE's own ORDER BY is the numeric order, so no caller has to sort and no caller \
             can forget to"
        );
        drop(conn);
        forget_subnet(&pool, "t-order").await;
    }

    /// 🔴 **ONE UNREADABLE ROW MUST NOT TAKE EVERY `/ipam` PAGE WITH IT** — the code review's blind
    /// layer, reasoning from the code: [`plan_ranges`] and [`plan_addresses`] are PLAN-WIDE, so a
    /// single row this build cannot read made both return `Err`, and with them every subnet's page
    /// and every address check answered 500. They skip and NAME it now, as [`list_subnets`] does.
    ///
    /// ⚠️ **AND THE STORE CANNOT PRODUCE SUCH A ROW TODAY, which is said rather than implied.** Every
    /// spelling below is refused by `0007`'s own CHECKs: the canonical pattern admits exactly ONE
    /// address family (pinned by `the_family_check_is_implied_until_a_second_width_exists`, whose
    /// own subject is that vacuity) so a stored bound always parses, and
    /// `LENGTH(policy) = LENGTH(TRIM(policy))` closes the PAD SPACE door the `IN (...)` list leaves
    /// open. ⚠️ The address half of the decision is also pinned by
    /// `only_the_canonical_spelling_reads_back`; it is asserted here too because this test is about
    /// what the PLAN-WIDE readers refuse, and that redundancy is deliberate. So the skip is **defensive code whose trigger is out of
    /// reach until FR25 widens that CHECK for IPv6** — at which point a v6 bound in a v4 build is
    /// exactly this case. 🔑 *What is testable here is the DECISION — which spellings this build
    /// calls unreadable — and that is what this pins; the loop around it is one `match`.*
    /// Registering the unreachability is the honest half: a guard nobody can red is a guard whose
    /// reach must be written down (story 14.1's `ip_range_same_family`, same shape).
    #[test]
    fn an_unreadable_plan_row_is_refused_by_the_reader_the_skip_consults() {
        let canonical_first = canonical(v4("192.0.2.1"));
        let canonical_last = canonical(v4("192.0.2.40"));
        assert!(
            read_range_row(&canonical_first, &canonical_last, "static").is_ok(),
            "the control: a row this build CAN read is read"
        );
        assert!(
            read_range_row("192.0.2.1", &canonical_last, "static").is_err(),
            "an unpadded bound is a row this build cannot read — it would also sort wrongly, which \
             is what the padding exists for"
        );
        assert!(
            read_range_row(&canonical_first, &canonical_last, "static ").is_err(),
            "and so is a policy token the `ascii_bin` PAD SPACE collation would accept in an \
             `IN (...)` comparison"
        );
        assert!(
            from_canonical("192.0.2.9").is_err(),
            "the address half of the same decision"
        );
    }

    /// 🔴 **AC5's adapter half, and the instrument is the opposite of the DDL half's.**
    ///
    /// These two rules compare two TABLES, so a `CHECK` cannot express them (`ERROR 1901`). Raw SQL
    /// therefore does NOT measure them — **it bypasses them**, and a test written that way passes
    /// over a deleted guard. Each refusal is exercised THROUGH the adapter, with a raw insert as
    /// the CONTROL showing the DDL really does let it through.
    #[tokio::test]
    async fn the_adapter_refuses_what_no_check_can_express() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        forget_subnet(&pool, "t-adapter").await;
        let mut conn = pool.acquire().await.expect("a connection");
        insert_subnet(
            &mut *conn,
            "t-adapter",
            Subnet::new(v4("203.0.113.0"), 24).unwrap(),
            "office",
        )
        .await
        .expect("the subnet");

        // (1) a range that leaves its subnet
        let outside = insert_range(
            &mut conn,
            "t-a1",
            "t-adapter",
            v4("203.0.113.10"),
            v4("203.0.114.10"),
            IpPolicy::Static,
            "escapes",
        )
        .await;
        assert!(
            outside.is_err(),
            "a range must not leave the subnet it belongs to"
        );

        // (2) an address that leaves its subnet
        let stray = insert_address(&mut conn, "t-a2", "t-adapter", v4("8.8.8.8"), "stray").await;
        assert!(stray.is_err(), "nor may an individually defined address");

        // (3) two ranges that overlap
        insert_range(
            &mut conn,
            "t-a3",
            "t-adapter",
            v4("203.0.113.100"),
            v4("203.0.113.199"),
            IpPolicy::DhcpPool,
            "pool",
        )
        .await
        .expect("the first range");
        let overlapping = insert_range(
            &mut conn,
            "t-a4",
            "t-adapter",
            v4("203.0.113.150"),
            v4("203.0.113.250"),
            IpPolicy::Static,
            "overlaps",
        )
        .await;
        assert!(
            overlapping.is_err(),
            "two ranges must not claim one address"
        );
        // …and a range that merely ABUTS the first is legal, which is what makes (3) a boundary
        // test rather than a coincidence.
        insert_range(
            &mut conn,
            "t-a5",
            "t-adapter",
            v4("203.0.113.200"),
            v4("203.0.113.250"),
            IpPolicy::Static,
            "abuts",
        )
        .await
        .expect("abutting is not overlapping");

        // (4) 🔴 an INVERTED range, and the finding was the REASON rather than the acceptance. It
        // was refused as `RangeOverlapsAnother` — an empty interval overlaps nothing — because the
        // overlap scan ran first and a sibling happened to be in the way. Story 14.2 renders these
        // sentences to the operator, so a refusal naming the wrong rule is one nobody can act on.
        let inverted = insert_range(
            &mut conn,
            "t-a7",
            "t-adapter",
            v4("203.0.113.150"),
            v4("203.0.113.120"),
            IpPolicy::Static,
            "ends before it begins",
        )
        .await;
        assert_eq!(
            inverted,
            Err(opencmdb_core::repo::RepositoryError::Ipam(
                IpamError::RangeBoundsInverted
            )),
            "an inverted range is refused BY NAME — asserting `is_err()` here would have passed \
             over the wrong reason, which is the defect the review found. Story 14.2 turned the \
             name from a SENTENCE into a VARIANT: `Backend(String)` forced a caller that wanted to \
             render this refusal to match on prose, which is what D47 forbids"
        );

        // 🔴 THE CONTROL, and it is what makes the three assertions mean anything: the DDL does NOT
        // refuse any of them. A raw insert of the escaping range succeeds.
        let raw = sqlx::query(
            "INSERT INTO ip_range (id,subnet_id,first_addr,last_addr,policy,label) \
             VALUES ('t-a6','t-adapter','203.000.113.010','203.000.114.010','static','raw')",
        )
        .execute(&mut *conn)
        .await;
        assert!(
            raw.is_ok(),
            "the control failed, so the three assertions above may be measuring the DDL rather \
             than the adapter — which is exactly the mistake AC5 was rewritten to prevent"
        );

        drop(conn);
        forget_subnet(&pool, "t-adapter").await;
    }
    /// **AC5 — two operators define overlapping ranges at the same time, and exactly one is told
    /// why.** A PERMANENT test, not a one-off measurement.
    ///
    /// 🔴 **It asserts the REFUSAL and not merely that one row survives**, and that is the whole
    /// design: §1(d)'s matrix of five locking strategies leaves ONE row in four of them, and three
    /// of those four tell the operator something false. Counting rows would have passed on:
    ///
    /// | strategy | loser is told | rows |
    /// |---|---|---|
    /// | no lock | `Ok` — its range is simply not there | **2** |
    /// | parent row only | `RangeOverlapsAnother` | 1 |
    /// | parent row + a non-locking `load_subnet` first (the DRY line) | `Ok` | **2** |
    /// | the deciding read only | `Backend("…1213 (40001): Deadlock found…")` | 1 |
    /// | **parent row THEN the deciding read** | **`RangeOverlapsAnother`** | 1 |
    ///
    /// ⚠️ **The 400 ms pause is what makes the race observable**, and it is a FUTURE handed to the
    /// production function rather than a flag inside it — production passes
    /// `std::future::ready(())`, so the seam has a live caller and cannot rot.
    ///
    /// ⚠️ Its own `/25`, its own ids: `DB_TEST_LOCK` serialises store tests, but a row left by a
    /// previous run is refused by `ip_subnet_cidr` and would read as *the guard fired* (story
    /// 14.1's self-healing guard, and 14.2's four tests reddened by a shared CIDR).
    #[tokio::test]
    async fn two_overlapping_ranges_at_once_leave_one_row_and_one_named_refusal() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        for statement in [
            "DELETE FROM ip_range WHERE subnet_id = 't-race'",
            "DELETE FROM ip_subnet WHERE id = 't-race'",
        ] {
            sqlx::query(statement)
                .execute(&pool)
                .await
                .expect("the probe's own rows are its own to remove");
        }
        let subnet = Subnet::new(v4("198.51.100.128"), 25).expect("a subnet of its own");
        let mut setup = pool.acquire().await.expect("a connection");
        insert_subnet(&mut *setup, "t-race", subnet, "the race")
            .await
            .expect("the parent row");
        drop(setup);

        let mut left = pool.acquire().await.expect("a connection");
        let mut right = pool.acquire().await.expect("a connection");
        let pause = || tokio::time::sleep(std::time::Duration::from_millis(400));
        let (first, second) = tokio::join!(
            insert_range_pausing(
                &mut left,
                "t-race-a",
                "t-race",
                v4("198.51.100.130"),
                v4("198.51.100.140"),
                IpPolicy::Static,
                "left",
                pause(),
            ),
            insert_range_pausing(
                &mut right,
                "t-race-b",
                "t-race",
                v4("198.51.100.135"),
                v4("198.51.100.150"),
                IpPolicy::Static,
                "right",
                pause(),
            )
        );

        let outcomes = [&first, &second];
        let won = outcomes.iter().filter(|r| r.is_ok()).count();
        assert_eq!(
            won, 1,
            "exactly one of the two writes may win: {outcomes:?}"
        );
        let loser = outcomes
            .iter()
            .find_map(|r| r.as_ref().err())
            .expect("the other one was refused");
        assert!(
            matches!(
                loser,
                RepositoryError::Ipam(IpamError::RangeOverlapsAnother)
            ),
            "the loser must be told the RULE it broke. A deadlock, a backend sentence or a silent \
             `Ok` all leave one row too, and none of them is an answer the operator can act on. \
             Got: {loser:?}"
        );

        let (rows,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM ip_range WHERE subnet_id = 't-race'")
                .fetch_one(&pool)
                .await
                .expect("counting the survivors");
        assert_eq!(rows, 1, "the refused write must have left nothing behind");
        // 🔴 Cleaned at the END as well as at the start — the review found its `/25` left behind.
        forget_subnet(&pool, "t-race").await;
    }

    /// 🔴 **Two ranges defined at once in two DIFFERENT subnets both land** — the review's Edge Case
    /// Hunter measured them DEADLOCKING, 2 runs of 3, the loser answered in 0.42 s: two empty subnets
    /// fall in one gap of the non-unique `ip_range_subnet` index, the parent-row locks are two
    /// different rows and serialise nothing between them, and each transaction then needs an
    /// insert-intention lock the other's gap lock blocks. The operator was told *another change was
    /// in flight* about a write that conflicted with nothing. Guy, 2026-09-15: replay a deadlock
    /// victim ONCE.
    ///
    /// ⚠️ **FIVE ROUNDS, because one round is a coin toss** — and what carries the replay is not an
    /// odds sentence but two measurements: MR3 (`is_deadlock` never true) and the second review's M1
    /// (replay removed) both red, 3 runs of 3. The rate a deadlock occurs at depends on the gap
    /// locking and the ids around them, so the oracle `a.is_ok() && b.is_ok()` would also be green
    /// where no deadlock happens at all; the mutations are what say it does happen here. (This read
    /// *"under half a percent"* until the second review called the figure an assumption.)
    #[tokio::test]
    async fn two_ranges_at_once_in_two_subnets_both_land() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        for id in ["t-dl-a", "t-dl-b"] {
            forget_subnet(&pool, id).await;
        }
        let mut setup = pool.acquire().await.expect("a connection");
        for (id, base) in [("t-dl-a", "100.64.10.0"), ("t-dl-b", "100.64.20.0")] {
            let subnet = Subnet::new(v4(base), 24).expect("a subnet of its own");
            insert_subnet(&mut *setup, id, subnet, "deadlock probe")
                .await
                .expect("the parent row");
        }
        drop(setup);

        for round in 0..5 {
            for id in ["t-dl-a", "t-dl-b"] {
                sqlx::query("DELETE FROM ip_range WHERE subnet_id = ?")
                    .bind(id)
                    .execute(&pool)
                    .await
                    .expect("the probe's own ranges");
            }
            let mut left = pool.acquire().await.expect("a connection");
            let mut right = pool.acquire().await.expect("a connection");
            let pause = || tokio::time::sleep(std::time::Duration::from_millis(400));
            let (left_id, right_id) = (format!("t-dl-a-{round}"), format!("t-dl-b-{round}"));
            let (a, b) = tokio::join!(
                insert_range_pausing(
                    &mut left,
                    &left_id,
                    "t-dl-a",
                    v4("100.64.10.10"),
                    v4("100.64.10.20"),
                    IpPolicy::Static,
                    "left",
                    pause(),
                ),
                insert_range_pausing(
                    &mut right,
                    &right_id,
                    "t-dl-b",
                    v4("100.64.20.10"),
                    v4("100.64.20.20"),
                    IpPolicy::Static,
                    "right",
                    pause(),
                )
            );
            assert!(
                a.is_ok() && b.is_ok(),
                "round {round}: two ranges in two DIFFERENT subnets conflict with nothing, so both \
                 must land — a deadlock victim is replayed once. Got {a:?} / {b:?}"
            );
        }
        for id in ["t-dl-a", "t-dl-b"] {
            forget_subnet(&pool, id).await;
        }
    }

    /// 🔴 **THE CLAUSE AN INSERT CANNOT CARRY, measured before it was written.** Re-running
    /// [`insert_range`]'s sibling scan verbatim for a widened range (`.010–.099` → `.010–.120`)
    /// returns **the row being edited, with `overlaps = 1`**: `range_attempt`'s scan has no
    /// `id <> ?`, because an insert has no id in the table yet, so **an edit that reuses it refuses
    /// every legal widening**.
    ///
    /// 🔴 **AND THE SENTENCE THAT STOOD HERE WAS WRONG ABOUT ITS OWN CARRIER, which the blind review
    /// layer found from the diff alone.** It called the third assertion — re-applying a range's OWN
    /// CURRENT BOUNDS — *"the one that carries the exclusion clause"*, on the grounds that the
    /// widening above *"could still be argued about"*. Neither half holds: under a scan without
    /// `id <> ?` the range overlaps ITSELF, so the FIRST `expect` panics and the third assertion is
    /// never reached; and the third call repeats the first byte for byte, against a scan that
    /// excludes the edited row, so its decision inputs are identical and it cannot fail alone.
    ///
    /// 🔑 **What the third call really buys is the IDEMPOTENCE of a correction** — applying the same
    /// values twice is accepted — which is worth keeping and is not what the clause needs. The
    /// clause's carrier is the FIRST call, and the doc now says so.
    ///
    /// ⚠️ Its own `/24` and its own ids, per this module's measured rule: a row left by a previous
    /// run is refused by `ip_subnet_cidr` and would read as *the guard fired*.
    #[tokio::test]
    async fn a_range_edit_widens_into_free_space_and_onto_its_own_ground() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        forget_subnet(&pool, "t-edit").await;
        let subnet = Subnet::new(v4("100.66.10.0"), 24).expect("a subnet of its own");
        let mut conn = pool.acquire().await.expect("a connection");
        insert_subnet(&mut *conn, "t-edit", subnet, "the edit")
            .await
            .expect("the parent row");
        for (id, first, last, label) in [
            ("t-edit-a", "100.66.10.10", "100.66.10.20", "a"),
            ("t-edit-b", "100.66.10.30", "100.66.10.40", "b"),
        ] {
            insert_range(
                &mut conn,
                id,
                "t-edit",
                v4(first),
                v4(last),
                IpPolicy::Static,
                label,
            )
            .await
            .expect("the two neighbours");
        }

        // A legal widening: `.20` → `.25` still stops short of `b`'s `.30`.
        update_range(
            &mut conn,
            "t-edit-a",
            v4("100.66.10.10"),
            v4("100.66.10.25"),
            IpPolicy::Reserved,
            "a, widened",
        )
        .await
        .expect("a widening that touches no neighbour is legal");

        // Read it BACK. An `Ok` says the statement ran, never that the row moved.
        let after = ranges_in(&pool, "t-edit").await.expect("the plan");
        assert_eq!(
            after,
            vec![
                (
                    v4("100.66.10.10"),
                    v4("100.66.10.25"),
                    IpPolicy::Reserved,
                    "a, widened".to_string()
                ),
                (
                    v4("100.66.10.30"),
                    v4("100.66.10.40"),
                    IpPolicy::Static,
                    "b".to_string()
                ),
            ],
            "the edit carries the bounds, the policy AND the label, and leaves the neighbour alone"
        );

        // 🔴 THE CARRIER OF `id <> ?`: a range must be allowed to keep the ground it already holds.
        update_range(
            &mut conn,
            "t-edit-a",
            v4("100.66.10.10"),
            v4("100.66.10.25"),
            IpPolicy::Reserved,
            "a, widened",
        )
        .await
        .expect(
            "a range overlaps itself by definition — without the exclusion clause this is refused, \
             and every edit in the product becomes impossible",
        );

        drop(conn);
        forget_subnet(&pool, "t-edit").await;
    }

    /// 🔴 **A refused edit must move NOTHING**, and the second half is what makes this test worth
    /// writing: an `Err` proves the decision, never the rollback. The neighbours are read back and
    /// compared whole.
    ///
    /// 🔑 **The insert's refusals are re-asked on the edit path** — an edit can push a range out of
    /// its subnet or invert its bounds exactly as a creation can, and *a rule held on one path and
    /// not on its twin* is the shape story 6.3 named. `NotFound` is asserted too, because the id
    /// comes from the operator's own page and a stale one must say so rather than write nothing and
    /// report success.
    #[tokio::test]
    async fn a_range_edit_that_would_overlap_a_neighbour_is_refused_and_moves_nothing() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        forget_subnet(&pool, "t-edit2").await;
        let subnet = Subnet::new(v4("100.66.11.0"), 24).expect("a subnet of its own");
        let mut conn = pool.acquire().await.expect("a connection");
        insert_subnet(&mut *conn, "t-edit2", subnet, "the refused edit")
            .await
            .expect("the parent row");
        for (id, first, last, label) in [
            ("t-edit2-a", "100.66.11.10", "100.66.11.20", "a"),
            ("t-edit2-b", "100.66.11.30", "100.66.11.40", "b"),
        ] {
            insert_range(
                &mut conn,
                id,
                "t-edit2",
                v4(first),
                v4(last),
                IpPolicy::Static,
                label,
            )
            .await
            .expect("the two neighbours");
        }
        let before = ranges_in(&pool, "t-edit2").await.expect("the plan");

        for (first, last, expected, what) in [
            (
                "100.66.11.10",
                "100.66.11.35",
                IpamError::RangeOverlapsAnother,
                "a widening that reaches into the neighbour",
            ),
            (
                "100.66.11.10",
                "100.66.12.5",
                IpamError::RangeOutsideSubnet,
                "an edit that walks out of the subnet",
            ),
            (
                "100.66.11.30",
                "100.66.11.20",
                IpamError::RangeBoundsInverted,
                "an edit whose last address precedes its first",
            ),
        ] {
            let refusal = update_range(
                &mut conn,
                "t-edit2-a",
                v4(first),
                v4(last),
                IpPolicy::Static,
                "a",
            )
            .await
            .expect_err(what);
            assert!(
                matches!(&refusal, RepositoryError::Ipam(error) if *error == expected),
                "{what} must be refused BY NAME — the operator can act on a rule and not on a \
                 backend sentence. Got: {refusal:?}"
            );
        }

        assert!(
            matches!(
                update_range(
                    &mut conn,
                    "t-edit2-nope",
                    v4("100.66.11.50"),
                    v4("100.66.11.60"),
                    IpPolicy::Static,
                    "ghost",
                )
                .await,
                Err(RepositoryError::NotFound)
            ),
            "an id the plan does not carry is NotFound — never a silent success over zero rows"
        );

        // The half an `Err` cannot prove: the transaction really rolled back.
        let after = ranges_in(&pool, "t-edit2").await.expect("the plan");
        assert_eq!(
            after, before,
            "four refused edits and the plan is byte-for-byte what it was — a refusal that leaves \
             a half-applied row behind is worse than no refusal at all"
        );

        drop(conn);
        forget_subnet(&pool, "t-edit2").await;
    }

    /// 🔴 **THE COMPUTED REFUSAL RACES, AND THIS IS THE MEASUREMENT** (§1(c)). Between counting the
    /// addresses a range holds and removing the range there is a window, and an `insert_address`
    /// running in it lands a row inside the space being deleted. The validation measured both sides
    /// with the inserter started 0.6 s into a 2 s window: with no lock the insert returned in
    /// single-digit milliseconds and the address landed; with one, it waited **1404.8 ms** and the
    /// two acts serialised. ⚠️ Those figures were first written here as *"without the parent row's
    /// lock"*; the mutation pass refuted that attribution, and the sentence now separates *locked*
    /// from *unlocked* rather than one lock from the other.
    ///
    /// 🔴 **WHAT THIS TEST CANNOT TELL YOU, AND THE MUTATIONS THAT ESTABLISHED IT.** Its wait is
    /// carried TWICE over: by the parent row's lock (which `insert_address`'s foreign key on
    /// `ip_subnet` must share — it takes no lock of its own, *"NO LOCK HERE"*), and by this
    /// function's address scan, whose next-key/gap locks over the subnet's index range block an
    /// `INSERT` there directly. Drop either and the test stays **green** (M-T3, M-T3b); drop both
    /// and the inserter returns in **2.86 ms** against the pair's 406.87 ms (M-T3c).
    /// ⚠️ *So a green here names no mechanism* — it is the pair that is load-bearing, and an
    /// attribution to one of them is exactly the *cause without a check* this project forbids.
    ///
    /// ⚠️ **What is asserted is the SERIALISATION, not a wall-clock figure.** A threshold well under
    /// the pause is what makes this a guard rather than a benchmark: the pause is 400 ms and the
    /// floor is 150 ms, so the test says *the inserter waited for the deleter* without failing on a
    /// slow machine. It is the direction that is load-bearing, and the mutation that removes BOTH
    /// carriers takes the wait to single-digit milliseconds — removing either one alone does not.
    ///
    /// 🔴 **AND THE THRESHOLD ALONE CANNOT TELL "WAITED" FROM "WAS SLOW"**, which the blind review
    /// layer caught: any change making `insert_address` take 150 ms for an unrelated reason would
    /// pass it with the serialisation broken. So the ORDER is asserted directly as well — the
    /// deleter's commit instant against the inserter's — and the threshold is kept beside it because
    /// the two fail differently: the order says *the wrong thing happened*, the threshold says
    /// *nothing waited at all*.
    ///
    /// ⚠️ **The name says more than the body measures, and is corrected rather than defended**: this
    /// test does not establish that an address CANNOT land inside a range being deleted — the insert
    /// is expected to SUCCEED. What it establishes is that it lands AFTER, never inside the window.
    #[tokio::test]
    async fn an_address_cannot_land_inside_a_range_while_it_is_being_deleted() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        forget_subnet(&pool, "t-del").await;
        let subnet = Subnet::new(v4("100.66.12.0"), 24).expect("a subnet of its own");
        let mut setup = pool.acquire().await.expect("a connection");
        insert_subnet(&mut *setup, "t-del", subnet, "the raced delete")
            .await
            .expect("the parent row");
        insert_range(
            &mut setup,
            "t-del-r",
            "t-del",
            v4("100.66.12.10"),
            v4("100.66.12.20"),
            IpPolicy::Static,
            "the doomed range",
        )
        .await
        .expect("the range to delete");
        drop(setup);

        let mut deleter = pool.acquire().await.expect("a connection");
        let mut inserter = pool.acquire().await.expect("a connection");
        let started = std::time::Instant::now();
        let (deleted, inserted) = tokio::join!(
            async {
                let outcome = delete_range_pausing(
                    &mut deleter,
                    "t-del-r",
                    tokio::time::sleep(std::time::Duration::from_millis(400)),
                )
                .await;
                (outcome, std::time::Instant::now())
            },
            async {
                // Into the window, not before it: the deleter must already have counted.
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                let at = std::time::Instant::now();
                let outcome = insert_address(
                    &mut inserter,
                    "t-del-a",
                    "t-del",
                    v4("100.66.12.15"),
                    "the interloper",
                )
                .await;
                (outcome, at.elapsed(), std::time::Instant::now())
            }
        );
        let (deleted, delete_finished) = deleted;
        let (inserted, insert_took, insert_finished) = inserted;
        drop(deleter);
        drop(inserter);

        deleted.expect("the range held no address, so the delete is legal");
        inserted.expect("the address is legal too — what is at stake is WHEN it lands");
        // 🔴 **THE ORDER FIRST, because it is what the property actually is.** The threshold below
        // cannot tell *waited for the deleter* from *was slow for its own reasons* — the review's
        // blind layer caught the oracle admitting any 150 ms insert — so the instants are compared
        // directly: the interloper may only complete once the deletion has.
        assert!(
            insert_finished >= delete_finished,
            "the address completed BEFORE the deletion did, so it landed inside the window the \
             deletion had already counted as empty — the two acts did not serialise"
        );
        assert!(
            insert_took >= std::time::Duration::from_millis(150),
            "nothing made the inserter wait at all: it returned in {insert_took:?} (the pair took \
             {:?}). This fails where the order assertion cannot — if both acts are instantaneous \
             the order can hold by accident, and a wait well inside the 400 ms pause is what says a \
             lock was met. With BOTH carriers removed the inserter returns in 2.86 ms.",
            started.elapsed()
        );

        forget_subnet(&pool, "t-del").await;
    }

    /// 🔴 **A REFUSED DELETE MOVES NOTHING, and the neighbours are what say so.** The refusal is
    /// computed in Rust over the addresses the locked read returns, so the arithmetic is the only
    /// thing standing between *this range holds an address* and *this range does not* — and an `Err`
    /// proves the decision, never the rollback.
    ///
    /// 🔑 **The two controls are what give it meaning**: an address just OUTSIDE the range does not
    /// forbid the delete, and a range that holds nothing is removed while its neighbour and that
    /// neighbour's address stand still. Without them a rule refusing everything would pass.
    #[tokio::test]
    async fn a_range_still_holding_an_address_is_refused_and_its_neighbours_stand_still() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        forget_subnet(&pool, "t-del2").await;
        let subnet = Subnet::new(v4("100.66.13.0"), 24).expect("a subnet of its own");
        let mut conn = pool.acquire().await.expect("a connection");
        insert_subnet(&mut *conn, "t-del2", subnet, "the refused delete")
            .await
            .expect("the parent row");
        for (id, first, last, label) in [
            ("t-del2-held", "100.66.13.10", "100.66.13.20", "held"),
            ("t-del2-free", "100.66.13.30", "100.66.13.40", "free"),
        ] {
            insert_range(
                &mut conn,
                id,
                "t-del2",
                v4(first),
                v4(last),
                IpPolicy::Static,
                label,
            )
            .await
            .expect("the two ranges");
        }
        for (id, addr) in [
            ("t-del2-in", "100.66.13.15"),  // inside the first range
            ("t-del2-out", "100.66.13.99"), // inside neither — the control
        ] {
            insert_address(&mut conn, id, "t-del2", v4(addr), "an address")
                .await
                .expect("the two addresses");
        }

        let refusal = delete_range(&mut conn, "t-del2-held")
            .await
            .expect_err("a range holding a defined address may not be removed");
        assert!(
            matches!(
                &refusal,
                RepositoryError::Ipam(IpamError::RangeStillHoldsAddresses)
            ),
            "the refusal must NAME the rule: no foreign key can raise this one, so a backend \
             sentence here would mean the arithmetic never ran. Got: {refusal:?}"
        );

        // CONTROL ONE: the address outside every range forbids nothing, so the free range goes.
        delete_range(&mut conn, "t-del2-free")
            .await
            .expect("a range that holds no address is removed — `.99` is outside it");

        // CONTROL TWO: the refusal left its own range, and the addresses, exactly where they were.
        let left = ranges_in(&pool, "t-del2").await.expect("the plan");
        assert_eq!(
            left,
            vec![(
                v4("100.66.13.10"),
                v4("100.66.13.20"),
                IpPolicy::Static,
                "held".to_string()
            )],
            "the refused range must still be there, and the deleted one gone"
        );
        let (addresses,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM ip_address WHERE subnet_id = 't-del2'")
                .fetch_one(&pool)
                .await
                .expect("the addresses");
        assert_eq!(
            addresses, 2,
            "a range delete may never touch an address — neither the one it refused to leave \
             homeless nor the one outside it"
        );

        drop(conn);
        forget_subnet(&pool, "t-del2").await;
    }

    /// 🔴 **AN EDIT MAY NOT ABANDON WHAT A DELETE MAY NOT ABANDON** (Guy's arbitration, 2026-09-16,
    /// on a defect all three review layers reached and the Edge layer measured with a probe).
    /// `delete_range` refused while the range held a defined address; `update_range` moved the range
    /// off it and answered `Ok`, leaving the address under no range at all — after which the delete
    /// the product had just refused succeeded. *The door was shut and the window beside it was open.*
    ///
    /// 🔑 **The two controls are the whole point of the chosen rule.** Widening abandons nothing and
    /// stays legal; correcting a policy or a label does not move the ground and stays legal. Refusing
    /// every edit of an occupied range would have been the coarser reading — and would have made
    /// correcting a label impossible on any range holding an address.
    #[tokio::test]
    async fn a_range_edit_that_would_abandon_an_address_is_refused_and_the_address_stands_still() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        forget_subnet(&pool, "t-edit3").await;
        let subnet = Subnet::new(v4("100.66.14.0"), 24).expect("a subnet of its own");
        let mut conn = pool.acquire().await.expect("a connection");
        insert_subnet(&mut *conn, "t-edit3", subnet, "the abandoning edit")
            .await
            .expect("the parent row");
        insert_range(
            &mut conn,
            "t-edit3-a",
            "t-edit3",
            v4("100.66.14.10"),
            v4("100.66.14.20"),
            IpPolicy::Static,
            "servers",
        )
        .await
        .expect("the range");
        insert_address(
            &mut conn,
            "t-edit3-x",
            "t-edit3",
            v4("100.66.14.15"),
            "nas-01",
        )
        .await
        .expect("the address it holds");

        for (first, last, what) in [
            ("100.66.14.30", "100.66.14.40", "a move off the address"),
            ("100.66.14.10", "100.66.14.12", "a shrink past the address"),
        ] {
            let refusal = update_range(
                &mut conn,
                "t-edit3-a",
                v4(first),
                v4(last),
                IpPolicy::Static,
                "servers",
            )
            .await
            .expect_err(what);
            assert!(
                matches!(
                    &refusal,
                    RepositoryError::Ipam(IpamError::RangeStillHoldsAddresses)
                ),
                "{what} abandons `.15`, which is what the DELETE refuses by name — so it earns the \
                 same sentence. Got: {refusal:?}"
            );
        }

        // CONTROL ONE: widening abandons nothing.
        update_range(
            &mut conn,
            "t-edit3-a",
            v4("100.66.14.10"),
            v4("100.66.14.25"),
            IpPolicy::Static,
            "servers",
        )
        .await
        .expect("a widening keeps every address it held, so it is legal");

        // CONTROL TWO: the ground does not move, so a policy and a label correction stay legal —
        // this is what the coarser reading would have made impossible.
        update_range(
            &mut conn,
            "t-edit3-a",
            v4("100.66.14.10"),
            v4("100.66.14.25"),
            IpPolicy::Reserved,
            "servers and gear",
        )
        .await
        .expect("correcting a policy or a label on an occupied range is not an abandonment");

        // And the address the refusals protected is still exactly where it was.
        let (addresses,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM ip_address WHERE subnet_id = 't-edit3' AND addr = '100.066.014.015'",
        )
        .fetch_one(&pool)
        .await
        .expect("the protected address");
        assert_eq!(
            addresses, 1,
            "a refused edit may not touch the address it refused to abandon"
        );
        let left = ranges_in(&pool, "t-edit3").await.expect("the plan");
        assert_eq!(
            left,
            vec![(
                v4("100.66.14.10"),
                v4("100.66.14.25"),
                IpPolicy::Reserved,
                "servers and gear".to_string()
            )],
            "only the two legal corrections landed"
        );

        drop(conn);
        forget_subnet(&pool, "t-edit3").await;
    }

    /// 🔴 **`update_address`'s CONTAINMENT RE-VALIDATION, which was carried by no test at all.**
    /// The review's Edge layer replaced `subnet.contains(addr)` with a predicate that is always true
    /// and measured **987 tests, clippy and ten gates GREEN** while an edit moved an address clean
    /// out of its subnet. The function's own doc calls this its reason for existing — *a rule held on
    /// one path and not on its twin* — and nothing reddened when the rule went.
    #[tokio::test]
    async fn an_address_edit_is_re_validated_against_its_subnet() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        forget_subnet(&pool, "t-edit4").await;
        let subnet = Subnet::new(v4("100.66.15.0"), 24).expect("a subnet of its own");
        let mut conn = pool.acquire().await.expect("a connection");
        insert_subnet(&mut *conn, "t-edit4", subnet, "the address edit")
            .await
            .expect("the parent row");
        insert_address(
            &mut conn,
            "t-edit4-x",
            "t-edit4",
            v4("100.66.15.9"),
            "printer",
        )
        .await
        .expect("the address");

        let refusal = update_address(&mut conn, "t-edit4-x", v4("100.66.99.9"), "printer")
            .await
            .expect_err("an edit may not move an address out of its subnet");
        assert!(
            matches!(
                &refusal,
                RepositoryError::Ipam(IpamError::AddressOutsideSubnet)
            ),
            "the insert refuses this at birth, so the edit must refuse it by the same name. \
             Got: {refusal:?}"
        );

        // CONTROL: a move WITHIN the subnet is the ordinary correction, and it lands.
        update_address(&mut conn, "t-edit4-x", v4("100.66.15.10"), "printer-hp")
            .await
            .expect("moving an address inside its own subnet is legal");
        let (moved,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM ip_address WHERE id = 't-edit4-x' AND addr = '100.066.015.010' \
             AND label = 'printer-hp'",
        )
        .fetch_one(&pool)
        .await
        .expect("the corrected row");
        assert_eq!(moved, 1, "the edit carries the address AND the label");

        assert!(
            matches!(
                update_address(&mut conn, "t-edit4-nope", v4("100.66.15.11"), "ghost").await,
                Err(RepositoryError::NotFound)
            ),
            "an id the plan does not carry is NotFound — never a silent success over zero rows"
        );

        drop(conn);
        forget_subnet(&pool, "t-edit4").await;
    }

    /// 🔴 **THE TWO DELETES THAT NO TEST TOUCHED.** The review measured both `NotFound` guards
    /// carried by NOTHING — neutering `rows_affected() == 0` left the whole suite green — and
    /// `delete_subnet`'s headline claim, that the DATABASE refuses a populated subnet by foreign key,
    /// was asserted by no test at all. ⚠️ That claim rests on `0007`'s two foreign keys carrying no
    /// `ON DELETE` clause, so MariaDB's default (`RESTRICT`) applies: if either were `CASCADE`, this
    /// function would silently destroy the operator's whole plan. The layers refuted the cascade by
    /// reading the migration; this measures the refusal itself.
    #[tokio::test]
    async fn the_deletes_refuse_a_populated_subnet_and_answer_a_stale_id() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        forget_subnet(&pool, "t-del3").await;
        let subnet = Subnet::new(v4("100.66.17.0"), 24).expect("a subnet of its own");
        let mut conn = pool.acquire().await.expect("a connection");
        insert_subnet(&mut *conn, "t-del3", subnet, "the populated subnet")
            .await
            .expect("the parent row");
        insert_range(
            &mut conn,
            "t-del3-r",
            "t-del3",
            v4("100.66.17.10"),
            v4("100.66.17.20"),
            IpPolicy::Static,
            "a range",
        )
        .await
        .expect("the range");
        insert_address(
            &mut conn,
            "t-del3-x",
            "t-del3",
            v4("100.66.17.90"),
            "an address",
        )
        .await
        .expect("the address");

        let refusal = delete_subnet(&mut conn, "t-del3")
            .await
            .expect_err("a subnet still holding children may not be removed");
        assert!(
            matches!(&refusal, RepositoryError::Constraint(name) if *name == "foreign_key"),
            "the DATABASE refuses this one — `ERROR 1451` on a foreign key, not arithmetic — and the \
             route needs that name to say *it still holds ranges* rather than *no such subnet*. \
             Got: {refusal:?}"
        );

        for id in ["t-del3-nope", ""] {
            assert!(
                matches!(
                    delete_address(&mut conn, id).await,
                    Err(RepositoryError::NotFound)
                ),
                "an address id naming nothing is NotFound, never a silent Ok: {id:?}"
            );
            assert!(
                matches!(
                    delete_subnet(&mut conn, id).await,
                    Err(RepositoryError::NotFound)
                ),
                "and so is a subnet id naming nothing: {id:?}"
            );
        }

        // Emptied in the order the foreign keys allow, and only THEN does the subnet go.
        delete_address(&mut conn, "t-del3-x")
            .await
            .expect("a defined address is a leaf — nothing refuses its removal");
        delete_range(&mut conn, "t-del3-r")
            .await
            .expect("the range holds no address now");
        delete_subnet(&mut conn, "t-del3").await.expect(
            "an empty subnet is removed — the refusal was about its children, not about it",
        );

        let (left,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ip_subnet WHERE id = 't-del3'")
            .fetch_one(&pool)
            .await
            .expect("the subnet");
        assert_eq!(left, 0, "and it is really gone");

        drop(conn);
        forget_subnet(&pool, "t-del3").await;
    }

    /// 🔴 **FOUR ranges at once in four different subnets all land** — the second review's edge layer
    /// measured ONE replay insufficient here: 11 writes of 40 answered `Contention`, each conflicting
    /// with nothing, the replay deadlocking again in lock-step. Bounded, jittered replays are the
    /// answer (Guy, 2026-09-15), and this is their measurement.
    #[tokio::test]
    async fn four_ranges_at_once_in_four_subnets_all_land() {
        const SUBNETS: [(&str, &str, &str, &str); 4] = [
            ("t-dl4-a", "100.65.10.0", "100.65.10.10", "100.65.10.20"),
            ("t-dl4-b", "100.65.11.0", "100.65.11.10", "100.65.11.20"),
            ("t-dl4-c", "100.65.12.0", "100.65.12.10", "100.65.12.20"),
            ("t-dl4-d", "100.65.13.0", "100.65.13.10", "100.65.13.20"),
        ];
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        for (id, _, _, _) in SUBNETS {
            forget_subnet(&pool, id).await;
        }
        let mut setup = pool.acquire().await.expect("a connection");
        for (id, base, _, _) in SUBNETS {
            let subnet = Subnet::new(v4(base), 24).expect("a subnet of its own");
            insert_subnet(&mut *setup, id, subnet, "four-writer probe")
                .await
                .expect("the parent row");
        }
        drop(setup);

        for round in 0..5 {
            for (id, _, _, _) in SUBNETS {
                sqlx::query("DELETE FROM ip_range WHERE subnet_id = ?")
                    .bind(id)
                    .execute(&pool)
                    .await
                    .expect("the probe's own ranges");
            }
            let mut connections = Vec::new();
            for _ in SUBNETS {
                connections.push(pool.acquire().await.expect("a connection"));
            }
            let ids: Vec<String> = SUBNETS
                .iter()
                .map(|(id, _, _, _)| format!("{id}-{round}"))
                .collect();
            let writes = connections.iter_mut().zip(SUBNETS).zip(&ids).map(
                |((conn, (subnet_id, _, first, last)), range_id)| {
                    insert_range_pausing(
                        conn,
                        range_id,
                        subnet_id,
                        v4(first),
                        v4(last),
                        IpPolicy::Static,
                        "writer",
                        tokio::time::sleep(std::time::Duration::from_millis(400)),
                    )
                },
            );
            let outcomes = futures_util::future::join_all(writes).await;
            assert!(
                outcomes.iter().all(Result::is_ok),
                "round {round}: four ranges in four DIFFERENT subnets conflict with nothing, so all \
                 must land — a deadlock victim is replayed with a random pause. Got {outcomes:?}"
            );
        }
        for (id, _, _, _) in SUBNETS {
            forget_subnet(&pool, id).await;
        }
    }

    /// The lock-wait cap the plan's write statements carry, in seconds — asserted below the handler's
    /// budget by `ipam_write`'s tests.
    pub(crate) const PLAN_LOCK_WAIT_SECONDS: u64 = 4;

    /// 🔴 **A plan write stops waiting on a lock BEFORE the handler's budget does** — the second
    /// review measured a write dropped by its 5 s budget staying in `LOCK WAIT` on the server for as
    /// long as the lock was held, up to the default 50 s, pinning its pool connection. Here the parent
    /// row is held by another transaction and a range write is made to wait on it: it must come back
    /// as `Contention` after the cap, not after fifty seconds.
    #[tokio::test]
    async fn a_plan_write_stops_waiting_on_a_lock_before_the_budget() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = ipam_fixture().await else {
            return;
        };
        forget_subnet(&pool, "t-cap").await;
        let subnet = Subnet::new(v4("100.64.40.0"), 24).expect("a subnet of its own");
        insert_subnet(&pool, "t-cap", subnet, "the capped wait")
            .await
            .expect("the parent row");

        let mut holder = pool.acquire().await.expect("a holder");
        sqlx::query("START TRANSACTION")
            .execute(&mut *holder)
            .await
            .expect("begin");
        sqlx::query("SELECT id FROM ip_subnet WHERE id = 't-cap' FOR UPDATE")
            .execute(&mut *holder)
            .await
            .expect("hold the parent row");

        let mut writer = pool.acquire().await.expect("a writer");
        let started = std::time::Instant::now();
        let refused = insert_range(
            &mut writer,
            "t-cap-r",
            "t-cap",
            v4("100.64.40.10"),
            v4("100.64.40.20"),
            IpPolicy::Static,
            "capped",
        )
        .await;
        let waited = started.elapsed();

        sqlx::query("ROLLBACK")
            .execute(&mut *holder)
            .await
            .expect("release the parent row");
        drop(holder);
        drop(writer);
        forget_subnet(&pool, "t-cap").await;

        assert!(
            matches!(refused, Err(RepositoryError::Contention)),
            "a write that waited out its cap is a lock-wait timeout: {refused:?}"
        );
        assert!(
            waited >= std::time::Duration::from_secs(PLAN_LOCK_WAIT_SECONDS),
            "premise: the write must really have waited on the held row, or this measures nothing \
             ({waited:?})"
        );
        assert!(
            waited < std::time::Duration::from_secs(PLAN_LOCK_WAIT_SECONDS + 2),
            "the write waited {waited:?}: past the cap, the server's own 50 s default is what \
             answered, and a write the budget dropped keeps its connection that long"
        );
    }

    /// Every statement of the plan that can wait on a lock carries the cap.
    ///
    /// 🔑 A source guard beside the behavioural one above, because that one measures the RANGE write
    /// and the subnet and address inserts wait on locks too (a unique key, a foreign key). It names
    /// the statement that lost its cap where the behaviour would only show a slow write.
    ///
    /// 🔴 **IT WAS BLIND TO EVERY VERB STORY 14.4 ADDS, and the validation measured it before a line
    /// of that story existed.** The needles were `"INSERT INTO ip_` and `FOR UPDATE"` alone, so a
    /// `DELETE FROM ip_range …` or an `UPDATE ip_range SET …` matched neither: the guard neither
    /// required the cap nor reddened, and `checked == 5` went on holding. *A guard placed where the
    /// defect cannot occur reads as coverage and is none* — this epic's dominant class, and it would
    /// have met the story that writes the product's FIRST production `DELETE` and `UPDATE`.
    ///
    /// ⚠️ **A destructive statement waits on locks the inserts do not.** `DELETE FROM ip_range`
    /// takes an exclusive lock on every row it matches and on the foreign-key parent it references,
    /// and an `UPDATE` of a range's bounds does the same while a concurrent address write holds the
    /// subnet through `ip_address_subnet_fk` — so uncapped they wait the server's 50 s default with
    /// the handler's connection in hand, which is exactly what the cap exists to prevent.
    ///
    /// 🔑 **The count is the load-bearing half.** It is what says the list moved; move it only after
    /// READING the new statement and deciding it is capped, never to make a suite pass.
    ///
    /// ⚠️ **AND IT IS STILL AN ENUMERATION, WHICH THE FIRST VERSION OF THIS PARAGRAPH DID NOT SAY.**
    /// Four needles are four spellings, not a property: this guard sees a statement REMOVED or
    /// renamed out of their reach — the count falls and it reds — and is blind to one ADDED in a
    /// spelling they cannot match. `"REPLACE INTO ip_`, `INSERT … ON DUPLICATE KEY UPDATE` (which
    /// story 6.3's review had to add to the mirror gate for exactly this reason), `LOCK IN SHARE
    /// MODE`, a lowercase verb, a literal split so the table name leaves the first line, or a table
    /// reached through an alias all pass unseen, and `checked` stays where it was.
    /// *An enumeration cannot claim the completeness of a property* — story 5.12's sentence, and the
    /// honest reading of this guard is a TRIPWIRE against the ordinary gesture, never a barrier.
    #[test]
    fn every_plan_statement_that_can_wait_on_a_lock_is_capped() {
        let source = include_str!("ipam_repo.rs");
        let cut = source
            .find("\n#[cfg(test)]")
            .expect("this file has a trailing test module");
        let production = crate::source_scan::code_only(&source[..cut]);
        let mut checked = 0;
        for needle in [
            "\"INSERT INTO ip_",
            "\"DELETE FROM ip_",
            "\"UPDATE ip_",
            "FOR UPDATE\"",
        ] {
            for (at, _) in production.match_indices(needle) {
                let start = production[..at]
                    .rfind('"')
                    .map_or(at, |q| if needle.starts_with('"') { at } else { q });
                let before = production[..start].trim_end();
                assert!(
                    before.ends_with("capped!("),
                    "a plan statement that can wait on a lock is not capped — `{}` — so a write the \
                     budget drops keeps its connection for the server's 50 s default",
                    &production[start..(at + needle.len()).min(production.len())]
                );
                checked += 1;
            }
        }
        assert_eq!(
            checked, 16,
            "the plan's lockable statements changed — a new one must be capped, and this count \
             updated only after READING what it counts. Today: three inserts (subnet, range, \
             address), three deletes (address, subnet, range), two updates (address, range), and \
             EIGHT locking reads — the insert's sibling scan, `load_subnet_locked`, the address \
             edit's own, `delete_range`'s two (the range row, then the subnet's addresses) and \
             `update_range`'s THREE (the range row carrying its current bounds, the sibling scan \
             with `id <> ?`, and the address scan the code review's abandonment rule added)"
        );
    }

    /// 🔴 **NO GATE IS OWED FOR THIS STORY'S `DELETE`s AND `UPDATE`s, AND THE REASON IS WRITTEN
    /// RATHER THAN LEFT TO THE GATES' SILENCE** (Guy's decision 4, 2026-09-16).
    ///
    /// Story 14.4 writes the product's FIRST production `UPDATE` and `DELETE` outside test cleanup,
    /// and none of the ten `cargo xtask ci` gates names a plan table. Story 5.12's own instruction is
    /// to reopen a gate's perimeter when a new writer appears — so the absence of a gate here is a
    /// DECISION and must read as one.
    ///
    /// 🔑 **What the three write-guarding gates protect is not what this story writes.**
    /// `observed-immutable` guards `observation_record`, because an observation is testimony the
    /// product may not rewrite. `entity-id-immutable` and `authorship` guard `declared_attribute` —
    /// D15's *"never updated. Ever."* and FR13's *who a write claims to be*. **The plan is the
    /// operator's OWN register** (Guy's arbitration 4 of 2026-09-10: IPAM and `declared_attribute`
    /// are two registers), where deleting a range **is the gesture**, not a violation of one. A gate
    /// refusing it would be a barrier against this epic's own subject.
    ///
    /// ⚠️ **Refused, with what each costs**: a sanctioned-site gate on `authorship`'s model would
    /// close the class for good, at the price of an eleventh gate and its located probe corpus in a
    /// story already carrying five write routes; and deferring it with a register row was refused
    /// because this project has measured that *a row with no named owner is not an action*.
    ///
    /// 🔑 **What IS owed is the widening above**, and that is where the gates' silence really did
    /// hide something: the cap's guard could not see a destructive statement at all.
    ///
    /// This test asserts the decision's OBSERVABLE half — that the plan's tables are named by no
    /// gate — so the day someone adds one, this reads as the contradiction it is rather than as a
    /// forgotten sentence.
    ///
    /// 🔴 **AND ITS FIRST VERSION READ TWO SOURCES OF SIX, WHICH ALL THREE REVIEW LAYERS REACHED.**
    /// It named `observed_immutable.rs` and `entity_id_immutable.rs` through `include_str!` while its
    /// own doc reasons about a THIRD gate, `authorship` — the one it calls most likely to acquire a
    /// sanctioned site for a new writer — which lives in `xtask/src/main.rs` and was never read. The
    /// Edge layer measured the hole rather than arguing it: a `PLAN_TABLES` const naming `ip_subnet`
    /// planted in `copy_vocabulary.rs` left this test **green**. *An enumeration cannot claim the
    /// completeness of a property*, in a guard written to record a decision about the whole gate set.
    ///
    /// 🔑 **So it reads the DIRECTORY, not a list.** `include_str!` cannot glob, so the walk happens
    /// at test time over every `.rs` under `xtask/src` — which covers the gate nobody has written
    /// yet, the case a list can never cover. ⚠️ The floor below is the premise this project insists
    /// on: a walk that found nothing would make every assertion vacuously true, which is the shape
    /// that hides here more often than a wrong assertion does.
    #[test]
    fn no_gate_claims_the_plans_tables_and_the_reason_is_recorded() {
        let gate_sources = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../xtask/src");
        let mut walked = 0;
        for entry in std::fs::read_dir(&gate_sources).expect("the xtask sources are readable") {
            let path = entry.expect("a directory entry").path();
            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("a gate source");
            let code = crate::source_scan::code_only(&source);
            for table in ["ip_subnet", "ip_range", "ip_address"] {
                assert!(
                    !code.contains(table),
                    "`{}` now names `{table}`, which contradicts story 14.4's decision 4 — the plan \
                     is the operator's own register and deleting there IS the gesture. If that \
                     decision changed, this test is the place that says so",
                    path.display()
                );
            }
            walked += 1;
        }
        assert!(
            walked >= 6,
            "the premise: {} `.rs` files were walked under {}, and a walk that reads nothing asserts \
             nothing — the six that exist today are the floor, not the target",
            walked,
            gate_sources.display()
        );
    }

    /// Both reads a range write performs take their lock — named individually, because behaviour
    /// cannot name them.
    ///
    /// 🔴 **The mutation pass measured that neither lock is observable on its own.** Drop the
    /// deciding read's `FOR UPDATE` and AC5's harness stays GREEN (the parent row alone serialises
    /// entry); add the DRY line with both locks present and it stays green too (the range lock
    /// catches it). Only the composite reds. *Two guards that each mask the other's mutation are
    /// two guards nothing measures* — so this one names each half, and the composite measures that
    /// the pair works.
    ///
    /// ⚠️ **A source guard, with the limit that word carries** (story 6b.11's AC5 as amended): it
    /// measures what was WRITTEN and not what the server executes, so it cannot see a lock the
    /// engine declines to take. It is cheaper than the composite and it NAMES THE CAUSE where the
    /// composite names only the symptom; the two cumulate rather than substitute.
    ///
    /// ⚠️ It reads the production half of this file only — the text before the first line that
    /// starts a test module — so the mutation strings inside these very doc comments are outside
    /// its perimeter. *A guard that greps a file greps its prose* (story 14.2), met three times in
    /// this story alone.
    ///
    /// 🔴 **COMMENTS ARE STRIPPED AND THE DRY LINE IS KEYED ON THE CALL, because the review defeated
    /// both halves of the first version.** G1: drop the deciding read's `FOR UPDATE` and keep the old
    /// SQL quoted in a comment above it — all 905 tests GREEN, this guard included, one lock gone and
    /// nothing saying so. G2: G1 plus the DRY line spelled `load_subnet(tx.as_mut(), …)` — this guard
    /// still green, because its needle was one spelling of the argument. So it reads CODE, and it
    /// reads the ATTEMPT's body, where a read of the subnet without the lock is refused whatever its
    /// argument looks like.
    ///
    /// 🔴 **And the second review defeated that too, with the other comment form**: the lock dropped
    /// and the old SQL kept in a `/* … */` block — 618 tests green. Its stripper also reset its
    /// string state per line and flipped on a `'"'` char literal. It reads through
    /// [`crate::source_scan::code_only`] now, which carries each of those traps as a test, and the
    /// parent-row SQL is looked for INSIDE `load_subnet_locked` rather than anywhere in the file.
    #[test]
    fn both_reads_of_a_subnet_under_write_take_their_lock() {
        let source = include_str!("ipam_repo.rs");
        let cut = source
            .find("\n#[cfg(test)]")
            .expect("this file has a trailing test module");
        let production = crate::source_scan::code_only(&source[..cut]);
        let body = body_of(&production, "async fn range_attempt(");
        let parent = body_of(&production, "async fn load_subnet_locked(");

        assert!(
            parent.contains("SELECT base, prefix_len FROM ip_subnet WHERE id = ? FOR UPDATE"),
            "the PARENT ROW's lock is gone, which serialises entry to one subnet. AC5's harness will \
             not tell you: each lock is masked by the other, and only the composite mutation reds."
        );
        assert!(
            body.contains("load_subnet_locked("),
            "the range attempt no longer enters through the parent row's lock"
        );
        assert!(
            body.contains(
                "SELECT first_addr, last_addr FROM ip_range WHERE subnet_id = ? FOR UPDATE"
            ),
            "the DECIDING read's lock is gone — the one an ordinary DRY line cannot walk past. AC5's \
             harness will not tell you: the parent row masks it."
        );
        for unlocked in ["load_subnet(", "FROM ip_subnet"] {
            assert!(
                !body.contains(unlocked),
                "a non-locking read of the subnet (`{unlocked}`) inside the range write's transaction \
                 — the DRY line. Under REPEATABLE READ it fixes the snapshot, and the locking read \
                 that follows does not refresh it, so the sibling scan reads a world without the \
                 other range."
            );
        }
    }

    /// The body of the first item whose code starts with `head`, up to its closing brace at column 0.
    fn body_of<'a>(code: &'a str, head: &str) -> &'a str {
        let start = code
            .find(head)
            .unwrap_or_else(|| panic!("`{head}` is gone"));
        let tail = &code[start..];
        &tail[..tail
            .find("\n}\n")
            .unwrap_or_else(|| panic!("`{head}` has no closing brace at column 0"))]
    }
}
