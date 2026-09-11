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
//! # ⚠️ `#![allow(dead_code)]`, and it is an ARBITRATION rather than a formality
//!
//! Story 14.1 ships **no producer** by its own criterion, so every function below is dead code and
//! CI — which builds under `RUSTFLAGS="-D warnings"` — fails without this attribute. Story 6.5
//! escaped only because `repo.rs` carries the same one.
//!
//! 🔴 **And on 2026-09-10 the opposite conclusion was taken one file over, correctly.**
//! `arp_ping.rs` had a blanket `allow(dead_code)` deleted, because with it standing, severing the
//! product's MAC read left `neighbours()` and `neighbour::table()` completely unreferenced **with
//! clippy still green** — the compiler was the only thing watching that wiring, and the attribute
//! blinded it.
//!
//! *An `allow` is a trade between "the compiler cannot see a producer that does not exist yet" and
//! "the compiler is the only thing watching this wiring". Say which one you are in.* Here it is the
//! first, and the attribute is **removed by story 14.2**, which gives these functions a producer.
// 🔑 **NARROWED BY STORY 14.2, from module-wide to item-by-item.**
//
// Story 14.1 shipped this module with a blanket `#![allow(dead_code)]` because it had NO producer
// by its own criterion, and `deferred-work.md` registered the debt with story 14.2 as its owner.
// This story gives the READ path a producer — `/ipam` is fed from here — so the blanket attribute
// would now hide something real: with it standing, severing the plan's read from the screen would
// leave clippy green, which is exactly the trade `arp_ping.rs` was measured on in 2026-09-10 when
// the ABSENCE of the same attribute was found load-bearing.
//
// ⚠️ What remains dead is the WRITE path, and **story 14.2b removes the last of these**. Each
// attribute below names that story, so the day a route calls one, the attribute above it is the
// thing that fails to be needed — and an unnecessary `allow` is visible where a blanket one is not.
//
// 🔴 The register said *eleven items*; a correction of it said *ten warnings covering fourteen*;
// both were wrong. Measured on `38da035`: **eleven warnings covering fifteen items**, identically
// under `cargo build` and `clippy --all-targets`. After this story's wiring: **seven**.

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
#[allow(dead_code, reason = "the write path has no producer until story 14.2b")]
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
    #[allow(dead_code, reason = "used by the write path, which story 14.2b wires")]
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
#[allow(dead_code, reason = "the write path has no producer until story 14.2b")]
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
#[allow(dead_code, reason = "the write path has no producer until story 14.2b")]
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
    let base = from_canonical(&base).map_err(ipam)?;
    Subnet::new(base, prefix_len).map_err(ipam)
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
#[allow(dead_code, reason = "the write path has no producer until story 14.2b")]
pub(crate) async fn insert_range(
    conn: &mut sqlx::MySqlConnection,
    id: &str,
    subnet_id: &str,
    first: Ipv4Addr,
    last: Ipv4Addr,
    policy: IpPolicy,
    label: &str,
) -> Result<(), RepositoryError> {
    // 🔴 BEFORE the overlap scan, and the order is the finding: an inverted range inside a populated
    // subnet was refused as `RangeOverlapsAnother`. An empty interval overlaps nothing, so the
    // reason was wrong wherever a sibling happened to be in the way — and story 14.2 renders these
    // sentences to the operator. *A refusal that names the wrong rule is one nobody can act on.*
    if last < first {
        return Err(ipam(IpamError::RangeBoundsInverted));
    }
    let subnet = load_subnet(&mut *conn, subnet_id).await?;
    if !subnet.contains(first) || !subnet.contains(last) {
        return Err(ipam(IpamError::RangeOutsideSubnet));
    }
    let siblings: Vec<(String, String)> =
        sqlx::query_as("SELECT first_addr, last_addr FROM ip_range WHERE subnet_id = ?")
            .bind(subnet_id)
            .fetch_all(&mut *conn)
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
    .execute(&mut *conn)
    .await
    .map_err(classify)?;
    Ok(())
}

/// Insert one individually defined address.
///
/// # Errors
///
/// [`IpamError::AddressOutsideSubnet`] through [`RepositoryError::Backend`], or the classified
/// `sqlx::Error`. ⚠️ The same instrument note as [`insert_range`] applies: raw SQL bypasses this.
#[allow(dead_code, reason = "the write path has no producer until story 14.2b")]
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
    rows.into_iter()
        .map(|(id, base, prefix_len, label)| {
            let base = from_canonical(&base).map_err(ipam)?;
            let subnet = Subnet::new(base, prefix_len).map_err(ipam)?;
            Ok((id, subnet, label))
        })
        .collect()
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
}
