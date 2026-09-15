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
    /// ⚠️ On a `/31` and a `/32` the two coincide or vanish; the predicate is written as a
    /// comparison against both bounds rather than as arithmetic on the prefix length, so those
    /// cases answer without a special arm.
    pub(crate) fn is_edge(&self, addr: Ipv4Addr) -> bool {
        addr == self.network() || addr == self.last()
    }
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
    sqlx::query(
        "INSERT INTO ip_subnet (id, base, prefix_len, label) \
         VALUES (?, ?, ?, ?)",
    )
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
    // 🔴 **ONE REPLAY ON A DEADLOCK, and the review measured why it is owed** (Guy, 2026-09-15).
    // Two ranges defined at once in two DIFFERENT, empty subnets deadlocked 2 runs of 3 in 0.42 s:
    // both subnet ids fall in one gap of the non-unique `ip_range_subnet` index, the parent locks
    // are different rows and serialise nothing between them, and each transaction then needs an
    // insert-intention lock the other's gap lock blocks. The loser was told *another change was in
    // flight* about a write that conflicted with nothing. InnoDB has already rolled the victim back,
    // so replaying the whole attempt is exactly NFR15's *"the caller replays the whole closure"*.
    // ⚠️ Once, and on 1213 alone: a lock-wait TIMEOUT (1205) is not replayed — the operator's
    // budget is already spent by then — and a second deadlock is answered as `Contention`.
    // The seam is not replayed either: it exists to open ONE window for AC5's harness.
    match range_attempt(
        conn,
        id,
        subnet_id,
        first,
        last,
        policy,
        label,
        after_the_deciding_read,
    )
    .await
    {
        Ok(()) => Ok(()),
        Err(RangeAttempt::Refused(error)) => Err(error),
        Err(RangeAttempt::Deadlocked) => {
            tracing::warn!(
                subnet = subnet_id,
                "a range write was chosen as a deadlock victim — replaying it once"
            );
            match range_attempt(
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
            {
                Ok(()) => Ok(()),
                Err(RangeAttempt::Refused(error)) => Err(error),
                Err(RangeAttempt::Deadlocked) => Err(RepositoryError::Contention),
            }
        }
    }
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
        let siblings: Vec<(String, String)> = sqlx::query_as(
            "SELECT first_addr, last_addr FROM ip_range WHERE subnet_id = ? FOR UPDATE",
        )
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
        sqlx::query(
            "INSERT INTO ip_range (id, subnet_id, first_addr, last_addr, policy, label) \
         VALUES (?, ?, ?, ?, ?, ?)",
        )
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
        // 🔴 ROLLED BACK HERE, EXPLICITLY, and the review's own repair is what exposed it. A refused
        // attempt used to return with `?` and leave `tx` to its `Drop`, which only QUEUES the
        // rollback on the connection: the locks — the parent row and the sibling gap — stay held
        // until that connection is next used. The race test's new end-of-test cleanup met exactly
        // that: its loser still held its connection, and `forget_subnet` waited out
        // `innodb_lock_wait_timeout` (1205, 50 s). The Blind Hunter had suspected it from the diff
        // alone and could not confirm it; this is the confirmation, and the fix is the cause's.
        Err(refused) => {
            if let Err(error) = tx.rollback().await {
                tracing::warn!(%error, "rolling back a refused range write failed");
            }
            Err(refused)
        }
    }
}

/// [`load_subnet`], but taking the subnet ROW's lock — the entry point every write to a subnet's
/// children must pass through first.
///
/// 🔑 It is a second SQL literal rather than `load_subnet` with four characters appended, and the
/// redundancy is DELIBERATE in this codebase's sense: the two differ by the one thing that matters,
/// and a shared helper taking a `lock: bool` would let a caller pass `false` at the exact place
/// where `false` is the defect. What IS shared is the decoding, in [`subnet_from_row`].
async fn load_subnet_locked(
    tx: &mut sqlx::MySqlConnection,
    id: &str,
) -> Result<Subnet, RepositoryError> {
    let row: Option<(String, u8)> =
        sqlx::query_as("SELECT base, prefix_len FROM ip_subnet WHERE id = ? FOR UPDATE")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(classify)?;
    let (base, prefix_len) = row.ok_or(RepositoryError::NotFound)?;
    subnet_from_row(&base, prefix_len)
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
    sqlx::query("INSERT INTO ip_address (id, subnet_id, addr, label) VALUES (?, ?, ?, ?)")
        .bind(id)
        .bind(subnet_id)
        .bind(canonical(addr))
        .bind(label)
        .execute(&mut *conn)
        .await
        .map_err(classify)?;
    Ok(())
}

/// Every defined address in one subnet, in address order.
///
/// 🔑 The `ORDER BY` is the padding's whole point: the store's own order is the numeric order, so
/// no caller has to sort and no caller can forget to.
///
/// # Errors
///
/// The classified `sqlx::Error`, or [`IpamError::MalformedAddress`] when a stored value is not this
/// store's canonical spelling — reachable only by a write that went around this module.
pub(crate) async fn addresses_in<'e, E>(
    executor: E,
    subnet_id: &str,
) -> Result<Vec<Ipv4Addr>, RepositoryError>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT addr FROM ip_address WHERE subnet_id = ? ORDER BY addr")
            .bind(subnet_id)
            .fetch_all(executor)
            .await
            .map_err(classify)?;
    rows.into_iter()
        .map(|(addr,)| from_canonical(&addr).map_err(ipam))
        .collect()
}

/// Every subnet in the plan, in numeric order of its base address.
///
/// 🔑 The order is the store's, for the reason `addresses_in` gives: the padded spelling makes
/// lexicographic order numeric, so no caller sorts and no caller can forget to.
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
    ///   [`addresses_in`] return `Err` for the whole subnet, losing the good rows with the bad.
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
        let read = addresses_in(&mut *conn, "t-order")
            .await
            .expect("read back");
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
    /// ⚠️ **FIVE ROUNDS, because one round is a coin toss**: a single run deadlocked two times in
    /// three, so a guard of one round would pass a third of the time with the replay deleted. Five
    /// rounds of a 2/3 event leave that at under half a percent — a measurement of a race states its
    /// odds rather than implying a certainty it does not have.
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
    /// ⚠️ Its stripper cuts `//` outside a double-quoted string on the same line; an escaped `\"`
    /// inside a literal would confuse it. No such literal exists in this file's production half.
    #[test]
    fn both_reads_of_a_subnet_under_write_take_their_lock() {
        let source = include_str!("ipam_repo.rs");
        let cut = source
            .find("\n#[cfg(test)]")
            .expect("this file has a trailing test module");
        let production = code_only(&source[..cut]);
        let start = production
            .find("async fn range_attempt(")
            .expect("the range write's attempt");
        let body = &production[start..];
        let body = &body[..body.find("\n}\n").expect("the attempt's closing brace")];

        assert!(
            production.contains("SELECT base, prefix_len FROM ip_subnet WHERE id = ? FOR UPDATE"),
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

    /// The code of a Rust source with its `//` comments removed, a `//` inside a double-quoted
    /// string on the same line being kept.
    fn code_only(source: &str) -> String {
        source
            .lines()
            .map(|line| {
                let bytes = line.as_bytes();
                let mut in_string = false;
                let mut cut = line.len();
                for at in 0..bytes.len().saturating_sub(1) {
                    match bytes[at] {
                        b'"' => in_string = !in_string,
                        b'/' if !in_string && bytes[at + 1] == b'/' => {
                            cut = at;
                            break;
                        }
                        _ => {}
                    }
                }
                &line[..cut]
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
