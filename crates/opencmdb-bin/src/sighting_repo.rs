//! The address sighting summary (story 14.3a): one row per distinct sighting — an IPv4 address, the
//! L2 domain it was seen in, and the hardware address seen on it — with the first and the last
//! instant it was seen. Story 14.3b's audit reads it; nothing renders it yet.
//!
//! 🔑 **Why it exists, measured and not argued.** Story 14.3's validation timed the only reader of
//! every observation's facts ([`crate::repo::load_observation_facts`] plus a fold) at **3.0–3.3 s
//! and 492 MB for 1 000 000 rows** — about 75 days of sweeping 46 hosts every five minutes — against
//! NFR2's 1.5 s. At that size there are 46 distinct addresses. This summary is bounded by the
//! network, not by time.
//!
//! Three decisions, Guy's (2026-09-15), each with the alternative refused:
//!
//! 1. **Maintained in the observation's OWN transaction** ([`insert_with_sightings`], which
//!    [`crate::repo::insert_observation`] delegates to), so every writer keeps the summary in step.
//!    ⚠️ The cost is written rather than discovered: a deadlock on the summary now rolls back the
//!    OBSERVATION too, where the plain insert had no such failure mode. Two things answer it — the
//!    pairs of one observation are upserted in KEY ORDER ([`sighting_keys`] sorts), which removes
//!    the measured shape (two 46-pair transactions in opposite orders deadlocked 200 times in 400),
//!    and the production ingest replays ONCE on a deadlock ([`ingest_observation`]). Refused: an
//!    upsert AFTER the observation's transaction, which never loses an observation but leaves the
//!    summary behind for ever once the one-time backfill has run.
//! 2. **Backfilled once at boot, before serving** ([`backfill_at_boot`]): one streaming read folded
//!    into a map bounded by distinct pairs, then ONE flush transaction in key order that writes the
//!    completion marker. Measured by the validation at 1 M rows: 1 176 ms and 9.7 MB, where
//!    `fetch_all` took 398 MB and keyset chunks cost 2.8× for no resumability. Refused: a
//!    background backfill, which would let the audit under-report during an upgraded install's
//!    first minute.
//! 3. **Keyed (address, `l2_domain`, MAC)**, the address first because story 14.3b looks addresses
//!    up, and the MAC `-` when the sighting carried none — MariaDB holds NULLs distinct in a key.
//!
//! 🔴 **Product code never deletes a row here.** Epic 14's constraint (3): a sighting protects an
//! address until the operator releases it, and release is story 14.4's.
//!
//! D10: canonicalising an address or a MAC is Rust's; `LEAST`/`GREATEST` on the instants is
//! bookkeeping, on [`crate::repo::widen_interface_seen_window`]'s precedent.

use std::collections::BTreeMap;
use std::net::Ipv4Addr;

use futures_util::TryStreamExt;
use opencmdb_core::observation::{Fact, L2DomainId, MacAddr, Observation, Timestamp};
use sqlx::{Acquire, Executor, MySql, MySqlConnection, MySqlPool};

use crate::ipam_repo::{canonical, from_canonical};
use crate::repo::{datetime_literal, is_deadlock};

/// The `mac` of a sighting that carried no hardware address.
///
/// A sentinel and not `NULL`, because the column is in the primary key and MariaDB holds NULLs
/// distinct. ⚠️ **Not the zero MAC**, which is a valid [`Fact`]: [`sighting_keys`] reads a zero MAC
/// as absent and writes this instead.
pub(crate) const MAC_ABSENT: &str = "-";

/// The name the one-time backfill records its completion under, in `sighting_backfill`.
pub(crate) const BACKFILL_NAME: &str = "address_sighting";

/// One row of the summary, as the adapter writes it: every field in its stored spelling.
///
/// 🔑 **The field order IS the primary key's column order**, and the derived `Ord` compares fields
/// in declaration order over bytes — which is `ascii_bin`'s order. So sorting a `Vec<SightingKey>`
/// sorts it the way InnoDB walks the key, and that is what makes two transactions touching the same
/// rows take their locks in the same order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SightingKey {
    /// The address in `0007`'s canonical padded form.
    pub(crate) addr: String,
    /// The observation's L2 domain, lowercase hyphenated.
    pub(crate) l2_domain: String,
    /// A lowercase colon MAC, or [`MAC_ABSENT`].
    pub(crate) mac: String,
}

/// Every sighting one observation implies, in key order and without repetition.
///
/// One row per IPv4 × MAC pair: a two-MAC observation on one address is two rows, a no-MAC one is
/// one row carrying [`MAC_ABSENT`], and an observation with no `IpV4` fact is **no row at all** —
/// a MAC seen without an address sights nothing this summary can key.
///
/// ⚠️ `DhcpLease.ip` is **not** counted: no connector produces it, and a lease is a promise by a
/// server rather than a sighting on the wire. Registered rather than decided silently.
///
/// ⚠️ **The zero MAC reads as absent.** The neighbour table drops it before it reaches a fact, but
/// `MacAddr([0; 6])` is a valid [`Fact`], and keying it would give an address two "no hardware
/// address" rows that nothing downstream could tell apart.
///
/// 🔴 **The sort is load-bearing, not tidiness.** Without it the pairs are upserted in FACT order,
/// and two observations listing the same MACs in opposite orders deadlock — measured 200 times in
/// 400 by the validation. `opposite_mac_orders_at_once_do_not_deadlock` is its carrier.
pub(crate) fn sighting_keys(l2_domain: L2DomainId, facts: &[Fact]) -> Vec<SightingKey> {
    let mut addrs: Vec<Ipv4Addr> = Vec::new();
    let mut macs: Vec<String> = Vec::new();
    for fact in facts {
        match fact {
            Fact::IpV4 { addr } => addrs.push(*addr),
            Fact::Mac { addr, .. } if addr.0 != [0; 6] => macs.push(addr.to_string()),
            _ => {}
        }
    }
    if macs.is_empty() {
        macs.push(MAC_ABSENT.to_owned());
    }
    let domain = l2_domain.to_string();
    let mut keys: Vec<SightingKey> = addrs
        .iter()
        .flat_map(|addr| {
            let addr = canonical(*addr);
            let domain = &domain;
            macs.iter().map(move |mac| SightingKey {
                addr: addr.clone(),
                l2_domain: domain.clone(),
                mac: mac.clone(),
            })
        })
        .collect();
    keys.sort();
    keys.dedup();
    keys
}

/// Write one sighting, or widen the window of the one already there — never narrowing it.
///
/// `VALUE()` names the proposed tuple's column. Widening rather than assigning is what makes an
/// out-of-order arrival safe: an observation older than the stored window extends `first_seen_at`
/// backwards. ⚠️ MariaDB evaluates `address_sighting_window` on the PROPOSED tuple, so callers
/// always propose `first <= last`.
async fn upsert_sighting(
    conn: &mut MySqlConnection,
    key: &SightingKey,
    first_seen_at: Timestamp,
    last_seen_at: Timestamp,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO address_sighting (addr, l2_domain, mac, first_seen_at, last_seen_at) \
         VALUES (?, ?, ?, ?, ?) \
         ON DUPLICATE KEY UPDATE \
         first_seen_at = LEAST(first_seen_at, VALUE(first_seen_at)), \
         last_seen_at = GREATEST(last_seen_at, VALUE(last_seen_at))",
    )
    .bind(&key.addr)
    .bind(&key.l2_domain)
    .bind(&key.mac)
    .bind(datetime_literal(first_seen_at))
    .bind(datetime_literal(last_seen_at))
    .execute(conn)
    .await?;
    Ok(())
}

/// Insert one observation row, and nothing else. All values are bound as strings (D48);
/// `observed_at` goes through [`datetime_literal`], the single formatting site.
///
/// _(Until story 14.3a this statement lived in `repo::insert_observation` with a second
/// `format("%Y-%m-%d %H:%M:%S%.6f")` of its own — the format string `datetime_literal`'s doc says
/// must not exist twice. Same output, one site.)_
///
/// 🔴 **Private, and that is decision 1's premise made structural.** An observation written through
/// this alone has no sightings, and after the one-time backfill nothing ever repairs that — so the
/// only crate-visible writer is [`insert_with_sightings`], through `repo::insert_observation`. It was
/// `pub(crate)` until story 14.3a's code review, two layers of which named it the ready-made way
/// around the summary.
async fn insert_observation_row<'e, E>(
    executor: E,
    observation: &Observation,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let facts =
        serde_json::to_string(&observation.facts).map_err(|e| sqlx::Error::Encode(Box::new(e)))?;
    sqlx::query(
        "INSERT INTO observation_record \
         (id, connector_id, observed_at, l2_domain, vantage, facts, raw) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(observation.obs_id.to_string())
    .bind(observation.connector_id.to_string())
    .bind(datetime_literal(observation.observed_at))
    .bind(observation.scope.l2_domain.to_string())
    .bind(observation.scope.vantage.to_string())
    .bind(facts)
    .bind(observation.raw.clone())
    .execute(executor)
    .await?;
    Ok(())
}

/// Insert one observation AND its sightings, atomically (decision 1).
///
/// On a pool this is its own transaction; on a connection that is already inside one — a
/// `MariaUnit`'s — `begin` is a SAVEPOINT, so a unit failing after this call rolls back both the
/// observation and its sightings (`a_unit_failing_after_the_upsert_leaves_neither`).
///
/// 🔑 **Atomic on the pool path too, and a test makes the SECOND write fail to show it**:
/// `a_sighting_write_failing_after_the_row_leaves_no_observation` holds the sighting's row from
/// another transaction, so the upsert times out with the observation row already inserted. The
/// review's edge layer had replaced the rollback below with a commit and every test stayed green.
///
/// ⚠️ **The rollback is explicit, and DEFENSIVE — carried by no test**, on story 14.2b's `settle`
/// precedent: a dropped sqlx `Transaction` queues its rollback, and the pool pings a returned
/// connection, which flushes it — so `drop(tx)` in its place also leaves every test green (measured by
/// the review). It is kept so releasing the locks does not depend on a pool's return path.
///
/// # Errors
///
/// The `sqlx::Error` as it came — a duplicate `obs_id` refused before any sighting is written, a
/// backend failure, or a deadlock the caller may replay ([`ingest_observation`] does, once).
pub(crate) async fn insert_with_sightings<'a, A>(
    conn: A,
    observation: &Observation,
) -> Result<(), sqlx::Error>
where
    A: Acquire<'a, Database = MySql>,
{
    let mut tx = conn.begin().await?;
    match write_observation_and_sightings(&mut tx, observation).await {
        Ok(()) => tx.commit().await,
        Err(error) => {
            // Best-effort: after a deadlock the server has already rolled the transaction back,
            // and a savepoint it discarded cannot be rolled back to. The original error matters.
            let _ = tx.rollback().await;
            Err(error)
        }
    }
}

/// The two writes of [`insert_with_sightings`], on one connection: the observation, then its
/// sightings in key order.
///
/// ⚠️ **What keeps a refused observation from leaving a sighting is the TRANSACTION, not this
/// order** — the review measured the reversed order green. The observation goes first so a duplicate
/// `obs_id` is refused before any sighting row is locked.
async fn write_observation_and_sightings(
    conn: &mut MySqlConnection,
    observation: &Observation,
) -> Result<(), sqlx::Error> {
    insert_observation_row(&mut *conn, observation).await?;
    let at = observation.observed_at;
    for key in sighting_keys(observation.scope.l2_domain, &observation.facts) {
        upsert_sighting(conn, &key, at, at).await?;
    }
    Ok(())
}

/// The production ingest of one scanned observation: [`insert_with_sightings`] on its own
/// transaction, **replayed once** if MariaDB chose it as a deadlock victim.
///
/// The replay lives HERE, at the transaction's owner, and not inside [`insert_with_sightings`]: a
/// deadlock rolls back the WHOLE transaction, so a savepoint caller has nothing left to replay into.
///
/// ⚠️ **Whether the replay fires on a real deadlock is carried by NOTHING, and that is stated.** The
/// count of attempts is tested with a stand-in predicate, and removing the call reds clippy's
/// `dead_code` alone; a predicate that never matches left the whole suite green (the review's
/// acceptance layer). No test manufactures a deadlock on this path, and none can happen here today:
/// the scan loop ingests one observation at a time and nothing else writes the summary at runtime.
///
/// # Errors
///
/// The `sqlx::Error` of the last attempt.
pub(crate) async fn ingest_observation(
    pool: &MySqlPool,
    observation: &Observation,
) -> Result<(), sqlx::Error> {
    replay_once_if(is_deadlock, || insert_with_sightings(pool, observation)).await
}

/// Run `attempt`, and once more if its error satisfies `retryable`.
///
/// Kept apart from [`ingest_observation`] so the count of attempts is testable without
/// manufacturing a real deadlock: the predicate is a parameter.
async fn replay_once_if<T, F, Fut>(
    retryable: fn(&sqlx::Error) -> bool,
    mut attempt: F,
) -> Result<T, sqlx::Error>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, sqlx::Error>>,
{
    match attempt().await {
        Err(error) if retryable(&error) => {
            tracing::warn!(
                ?error,
                "an attempt failed with a retryable error — replaying once"
            );
            attempt().await
        }
        other => other,
    }
}

/// One sighting as the audit reads it, decoded.
///
/// ⚠️ **One row per (address, L2 domain, MAC)** — merging across L2 domains, and treating a `None`
/// row beside a MAC row for the same address, are story 14.3b's. An address seen once with a MAC
/// and once without carries both rows; the `None` one is not a second hardware address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Sighting {
    /// The address.
    pub(crate) addr: Ipv4Addr,
    /// The L2 domain it was seen in.
    pub(crate) l2_domain: L2DomainId,
    /// The hardware address seen on it, or `None` for the [`MAC_ABSENT`] row.
    pub(crate) mac: Option<MacAddr>,
    /// The earliest instant it was seen.
    pub(crate) first_seen_at: Timestamp,
    /// The latest instant it was seen.
    pub(crate) last_seen_at: Timestamp,
}

/// Read the whole summary, decoded, in key order.
///
/// # Errors
///
/// A `sqlx::Error` on a backend failure, or `Decode` on a row the schema admits and this reader
/// cannot read — which the CHECKs make unreachable through the adapter.
pub(crate) async fn load_sightings<'e, E>(executor: E) -> Result<Vec<Sighting>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT addr, l2_domain, mac, \
         DATE_FORMAT(first_seen_at, '%Y-%m-%dT%H:%i:%s.%fZ'), \
         DATE_FORMAT(last_seen_at, '%Y-%m-%dT%H:%i:%s.%fZ') \
         FROM address_sighting ORDER BY addr, l2_domain, mac",
    )
    .fetch_all(executor)
    .await?;
    rows.into_iter()
        .map(|(addr, l2_domain, mac, first, last)| {
            let decode = |e: Box<dyn std::error::Error + Send + Sync>| sqlx::Error::Decode(e);
            Ok(Sighting {
                addr: from_canonical(&addr).map_err(|e| decode(Box::new(e)))?,
                l2_domain: L2DomainId::from_uuid(
                    l2_domain
                        .parse::<uuid::Uuid>()
                        .map_err(|e| decode(Box::new(e)))?,
                ),
                mac: if mac == MAC_ABSENT {
                    None
                } else {
                    Some(mac.parse::<MacAddr>().map_err(|e| decode(Box::new(e)))?)
                },
                first_seen_at: parse_instant(&first).map_err(decode)?,
                last_seen_at: parse_instant(&last).map_err(decode)?,
            })
        })
        .collect()
}

/// Parse the `DATE_FORMAT` rendering every reader in this crate uses.
fn parse_instant(text: &str) -> Result<Timestamp, Box<dyn std::error::Error + Send + Sync>> {
    Ok(chrono::DateTime::parse_from_rfc3339(text)?.with_timezone(&chrono::Utc))
}

/// What the one-time backfill did, as the marker records it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BackfillReport {
    /// Observation rows the streaming read returned.
    pub(crate) observations_read: u64,
    /// Rows among them that could not be decoded, each named in the log and skipped.
    pub(crate) observations_skipped: u64,
    /// Distinct sightings the flush wrote or widened.
    pub(crate) pairs_flushed: u64,
}

/// Whether [`backfill_at_boot`] had anything to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackfillOutcome {
    /// The marker was already there: the history was not read.
    AlreadyDone(BackfillReport),
    /// This boot read the history and wrote the marker.
    Completed(BackfillReport),
}

/// Backfill the summary from every observation already stored — once per store (decision 2).
///
/// Called at boot after the migrations and **before** the scan loop starts or the server binds, so
/// nothing ingests concurrently and nothing reads a partial summary.
///
/// 1. If `sighting_backfill` holds the marker, return without reading anything.
/// 2. Otherwise read `observation_record` in ONE streaming `SELECT` — never `fetch_all` — and fold
///    it into a map keyed by [`SightingKey`], so memory is bounded by distinct pairs.
/// 3. Flush the map in key order in ONE transaction that also writes the marker. A run
///    interrupted before the commit leaves no marker and the next boot redoes it; the flush
///    widens rather than assigns, so a redo over rows ingest already wrote is harmless.
///
/// 🔴 **An undecodable row is skipped and NAMED, and the backfill completes.** The validation
/// planted one fact of an unknown kind and the first design failed after reading everything — at
/// boot that either stops the product or never writes the marker and re-reads the history at every
/// boot. Such a row is realistic: written by a newer binary, or by hand.
///
/// # Errors
///
/// A backend failure while reading the marker, streaming, or flushing. The caller refuses to
/// start rather than serving an audit over a partial summary.
pub(crate) async fn backfill_at_boot(pool: &MySqlPool) -> Result<BackfillOutcome, sqlx::Error> {
    if let Some(report) = load_backfill_marker(pool).await? {
        return Ok(BackfillOutcome::AlreadyDone(report));
    }
    let (pairs, read, skipped) = fold_history(pool).await?;
    let report = flush_backfill(pool, &pairs, read, skipped).await?;
    Ok(BackfillOutcome::Completed(report))
}

/// The marker, if the backfill has completed on this store.
async fn load_backfill_marker(pool: &MySqlPool) -> Result<Option<BackfillReport>, sqlx::Error> {
    let row: Option<(u64, u64, u64)> = sqlx::query_as(
        "SELECT observations_read, observations_skipped, pairs_flushed \
         FROM sighting_backfill WHERE name = ?",
    )
    .bind(BACKFILL_NAME)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(read, skipped, pairs)| BackfillReport {
        observations_read: read,
        observations_skipped: skipped,
        pairs_flushed: pairs,
    }))
}

/// The folded history: every distinct sighting with its window, plus the rows read and skipped.
type FoldedHistory = (BTreeMap<SightingKey, (Timestamp, Timestamp)>, u64, u64);

/// Stream every observation and fold its sightings. Instants are compared in Rust (D10).
async fn fold_history(pool: &MySqlPool) -> Result<FoldedHistory, sqlx::Error> {
    let mut rows = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, l2_domain, DATE_FORMAT(observed_at, '%Y-%m-%dT%H:%i:%s.%fZ'), facts \
         FROM observation_record",
    )
    .fetch(pool);
    let mut pairs: BTreeMap<SightingKey, (Timestamp, Timestamp)> = BTreeMap::new();
    let (mut read, mut skipped) = (0_u64, 0_u64);
    while let Some((id, l2_domain, observed_at, facts)) = rows.try_next().await? {
        read += 1;
        match decode_history_row(&l2_domain, &observed_at, &facts) {
            Ok((domain, at, facts)) => {
                for key in sighting_keys(domain, &facts) {
                    pairs
                        .entry(key)
                        .and_modify(|(first, last)| {
                            *first = (*first).min(at);
                            *last = (*last).max(at);
                        })
                        .or_insert((at, at));
                }
            }
            Err(reason) => {
                skipped += 1;
                tracing::warn!(
                    observation = %id,
                    %reason,
                    "the sighting backfill skipped an observation it cannot decode"
                );
            }
        }
    }
    Ok((pairs, read, skipped))
}

/// Decode the three columns a sighting needs, or say why not.
fn decode_history_row(
    l2_domain: &str,
    observed_at: &str,
    facts: &str,
) -> Result<(L2DomainId, Timestamp, Vec<Fact>), String> {
    let domain = l2_domain
        .parse::<uuid::Uuid>()
        .map_err(|e| format!("l2_domain: {e}"))?;
    let at = parse_instant(observed_at).map_err(|e| format!("observed_at: {e}"))?;
    let facts: Vec<Fact> = serde_json::from_str(facts).map_err(|e| format!("facts: {e}"))?;
    Ok((L2DomainId::from_uuid(domain), at, facts))
}

/// Write the folded pairs in key order and the marker, in one transaction.
async fn flush_backfill(
    pool: &MySqlPool,
    pairs: &BTreeMap<SightingKey, (Timestamp, Timestamp)>,
    read: u64,
    skipped: u64,
) -> Result<BackfillReport, sqlx::Error> {
    let report = BackfillReport {
        observations_read: read,
        observations_skipped: skipped,
        pairs_flushed: pairs.len() as u64,
    };
    let mut tx = pool.begin().await?;
    match write_flush(&mut tx, pairs, report).await {
        Ok(()) => {
            tx.commit().await?;
            Ok(report)
        }
        Err(error) => {
            let _ = tx.rollback().await;
            Err(error)
        }
    }
}

/// The body of [`flush_backfill`], on the transaction's connection.
async fn write_flush(
    conn: &mut MySqlConnection,
    pairs: &BTreeMap<SightingKey, (Timestamp, Timestamp)>,
    report: BackfillReport,
) -> Result<(), sqlx::Error> {
    // `BTreeMap` iterates in key order — the same order ingest locks rows in.
    for (key, (first, last)) in pairs {
        upsert_sighting(&mut *conn, key, *first, *last).await?;
    }
    // 🔑 A marker that is already there is NOT an error. Two instances booting on one store both
    // find no marker, both fold and both flush; the flush widens, so the second one's rows change
    // nothing. The first design inserted plainly and the review's edge layer measured the second
    // instance refuse to START on `1062` — a refusal that protected nothing. The first report stays.
    sqlx::query(
        "INSERT INTO sighting_backfill \
         (name, observations_read, observations_skipped, pairs_flushed) VALUES (?, ?, ?, ?) \
         ON DUPLICATE KEY UPDATE name = VALUE(name)",
    )
    .bind(BACKFILL_NAME)
    .bind(report.observations_read)
    .bind(report.observations_skipped)
    .bind(report.pairs_flushed)
    .execute(conn)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    use opencmdb_core::observation::{ConnectorId, HostnameSource, ObsId, Scope, VantageId};
    use opencmdb_core::repo::{RepositoryError, WriteRepository};

    use crate::repo::{MariaRepository, classify};

    // ── Builders ─────────────────────────────────────────────────────────────────────────────

    fn ts(text: &str) -> Timestamp {
        chrono::DateTime::parse_from_rfc3339(text)
            .expect("a test instant")
            .with_timezone(&chrono::Utc)
    }

    fn domain(n: u128) -> L2DomainId {
        L2DomainId::from_uuid(uuid::Uuid::from_u128(n))
    }

    fn mac(last: u8) -> MacAddr {
        MacAddr([0x02, 0, 0, 0, 0, last])
    }

    fn ip(last: u8) -> Ipv4Addr {
        Ipv4Addr::new(192, 0, 2, last)
    }

    fn ipv4(last: u8) -> Fact {
        Fact::IpV4 { addr: ip(last) }
    }

    fn mac_fact(last: u8) -> Fact {
        Fact::Mac {
            addr: mac(last),
            locally_administered: true,
        }
    }

    fn hostname() -> Fact {
        Fact::Hostname {
            name: "nas-01".to_owned(),
            source: HostnameSource::Dns,
        }
    }

    fn observation(l2_domain: L2DomainId, at: &str, facts: Vec<Fact>) -> Observation {
        Observation {
            obs_id: ObsId::from_uuid(uuid::Uuid::now_v7()),
            connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()),
            observed_at: ts(at),
            scope: Scope {
                l2_domain,
                vantage: VantageId::from_uuid(uuid::Uuid::nil()),
            },
            facts,
            raw: None,
        }
    }

    fn key(addr: &str, l2_domain: L2DomainId, mac: &str) -> SightingKey {
        SightingKey {
            addr: addr.to_owned(),
            l2_domain: l2_domain.to_string(),
            mac: mac.to_owned(),
        }
    }

    fn sighting(
        last_octet: u8,
        l2_domain: L2DomainId,
        mac_octet: Option<u8>,
        first: &str,
        last: &str,
    ) -> Sighting {
        Sighting {
            addr: ip(last_octet),
            l2_domain,
            mac: mac_octet.map(mac),
            first_seen_at: ts(first),
            last_seen_at: ts(last),
        }
    }

    /// A key and its window as the reader would decode them — the bridge that lets a fold computed
    /// in Rust be compared with rows read back from the store.
    fn decoded(key: &SightingKey, first: Timestamp, last: Timestamp) -> Sighting {
        Sighting {
            addr: from_canonical(&key.addr).expect("a canonical address"),
            l2_domain: L2DomainId::from_uuid(key.l2_domain.parse().expect("a uuid")),
            mac: (key.mac != MAC_ABSENT).then(|| key.mac.parse().expect("a mac")),
            first_seen_at: first,
            last_seen_at: last,
        }
    }

    // ── The derivation, pure ─────────────────────────────────────────────────────────────────

    /// AC3/AC4 — the cross product, in the order InnoDB walks the key, whatever order the facts
    /// came in.
    #[test]
    fn every_address_times_every_mac_in_key_order() {
        let keys = sighting_keys(domain(1), &[mac_fact(2), ipv4(9), mac_fact(1), ipv4(3)]);
        assert_eq!(
            keys,
            vec![
                key("192.000.002.003", domain(1), "02:00:00:00:00:01"),
                key("192.000.002.003", domain(1), "02:00:00:00:00:02"),
                key("192.000.002.009", domain(1), "02:00:00:00:00:01"),
                key("192.000.002.009", domain(1), "02:00:00:00:00:02"),
            ],
            "two addresses and two MACs are four sightings, sorted by (addr, l2_domain, mac)"
        );
    }

    /// AC3 — the sentinel, and the zero MAC read as absent rather than keyed.
    #[test]
    fn no_mac_is_the_sentinel_and_the_zero_mac_is_no_mac() {
        let zero = Fact::Mac {
            addr: MacAddr([0; 6]),
            locally_administered: false,
        };
        assert_eq!(
            sighting_keys(domain(1), &[ipv4(5)]),
            vec![key("192.000.002.005", domain(1), MAC_ABSENT)]
        );
        assert_eq!(
            sighting_keys(domain(1), &[ipv4(5), zero.clone()]),
            vec![key("192.000.002.005", domain(1), MAC_ABSENT)],
            "a zero MAC alone must read as no MAC, not as a second 'absent' spelling"
        );
        assert_eq!(
            sighting_keys(domain(1), &[ipv4(5), zero, mac_fact(7)]),
            vec![key("192.000.002.005", domain(1), "02:00:00:00:00:07")],
            "a zero MAC beside a real one adds no row"
        );
    }

    /// AC3 — no `IpV4`, no row; a DHCP lease is not a sighting.
    #[test]
    fn no_address_is_no_sighting_and_a_lease_is_not_one() {
        assert!(sighting_keys(domain(1), &[mac_fact(1), hostname()]).is_empty());
        assert!(
            sighting_keys(
                domain(1),
                &[Fact::DhcpLease {
                    ip: ip(4),
                    expires_at: None
                }]
            )
            .is_empty(),
            "DhcpLease.ip is not counted: no producer, and a lease is a server's promise"
        );
    }

    /// A fact repeated inside one observation is one sighting, not two upserts of one row.
    #[test]
    fn a_repeated_fact_is_one_sighting() {
        assert_eq!(
            sighting_keys(domain(1), &[ipv4(5), ipv4(5), mac_fact(1), mac_fact(1)]),
            vec![key("192.000.002.005", domain(1), "02:00:00:00:00:01")]
        );
    }

    /// The replay's count, measured without manufacturing a deadlock: the predicate is a parameter.
    #[tokio::test]
    async fn a_retryable_error_is_replayed_once_and_nothing_else_is() {
        fn not_found(error: &sqlx::Error) -> bool {
            matches!(error, sqlx::Error::RowNotFound)
        }

        let calls = Cell::new(0);
        let always = replay_once_if(not_found, || {
            calls.set(calls.get() + 1);
            std::future::ready(Err::<(), _>(sqlx::Error::RowNotFound))
        })
        .await;
        assert!(always.is_err());
        assert_eq!(calls.get(), 2, "a retryable error is replayed exactly once");

        let calls = Cell::new(0);
        let terminal = replay_once_if(not_found, || {
            calls.set(calls.get() + 1);
            std::future::ready(Err::<(), _>(sqlx::Error::PoolClosed))
        })
        .await;
        assert!(terminal.is_err());
        assert_eq!(calls.get(), 1, "a non-retryable error is not replayed");

        let calls = Cell::new(0);
        let recovered = replay_once_if(not_found, || {
            calls.set(calls.get() + 1);
            std::future::ready(if calls.get() == 1 {
                Err(sqlx::Error::RowNotFound)
            } else {
                Ok(7)
            })
        })
        .await;
        assert_eq!(recovered.expect("the replay succeeded"), 7);
        assert_eq!(calls.get(), 2);
    }

    // ── The store ────────────────────────────────────────────────────────────────────────────

    /// Connect, migrate, and empty what these tests write. `None` when `DATABASE_URL` is unset.
    /// Every caller holds [`crate::DB_TEST_LOCK`] first.
    async fn store() -> Option<MySqlPool> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping sighting test: DATABASE_URL unset");
            return None;
        };
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        for statement in [
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM interface",
            "DELETE FROM observation_record",
            "DELETE FROM address_sighting",
            "DELETE FROM sighting_backfill",
        ] {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }
        Some(pool)
    }

    /// A raw insert, around the adapter ON PURPOSE: story 5.9's M3 — a guard the adapter cannot
    /// violate is measured by raw SQL or by nothing.
    async fn raw_sighting(
        pool: &MySqlPool,
        addr: &str,
        l2_domain: &str,
        mac: &str,
        first: &str,
        last: &str,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            "INSERT INTO address_sighting (addr, l2_domain, mac, first_seen_at, last_seen_at) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(addr)
        .bind(l2_domain)
        .bind(mac)
        .bind(first)
        .bind(last)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(classify)
    }

    const NIL: &str = "00000000-0000-0000-0000-000000000000";
    const T0: &str = "2026-01-01 00:00:00.000000";
    const T1: &str = "2026-01-01 00:00:01.000000";

    async fn count(pool: &MySqlPool, table: &str) -> i64 {
        let (n,): (i64,) =
            sqlx::query_as(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM {table}")))
                .fetch_one(pool)
                .await
                .expect("count");
        n
    }

    /// AC1 — the MAC column refuses every spelling the adapter can never send, and admits the two
    /// it does. The first two are the validation's measured holes in `interface.mac_canon`'s CHECK.
    #[tokio::test]
    async fn the_mac_column_refuses_every_spelling_the_adapter_cannot_send() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        for (octet, bad) in [
            "",
            "-\n",
            "- ",
            "--",
            "zz",
            "AA:BB:CC:DD:EE:FF",
            "aa:bb:cc:dd:ee:f",
            "aa-bb-cc-dd-ee-ff",
        ]
        .into_iter()
        .enumerate()
        {
            let addr = format!("192.000.002.{:03}", octet + 1);
            assert_eq!(
                raw_sighting(&pool, &addr, NIL, bad, T0, T0).await,
                Err(RepositoryError::Constraint("check")),
                "the MAC {bad:?} must be refused by the shape CHECK"
            );
        }
        for (addr, good) in [
            ("192.000.002.100", MAC_ABSENT),
            ("192.000.002.101", "aa:bb:cc:dd:ee:ff"),
        ] {
            raw_sighting(&pool, addr, NIL, good, T0, T0)
                .await
                .unwrap_or_else(|e| panic!("the MAC {good:?} must be admitted: {e:?}"));
        }
    }

    /// AC1 — the address, the L2 domain and the window refuse what the adapter cannot send.
    #[tokio::test]
    async fn the_address_domain_and_window_refuse_what_the_adapter_cannot_send() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        for bad in [
            "192.000.002.009\n",
            "192.000.002.009 ",
            "192.0.2.9",
            "999.999.999.999",
        ] {
            assert_eq!(
                raw_sighting(&pool, bad, NIL, MAC_ABSENT, T0, T0).await,
                Err(RepositoryError::Constraint("check")),
                "the address {bad:?} must be refused"
            );
        }
        for bad in [
            "0000000A-0000-0000-0000-000000000000",
            "zzzzzzzz-zzzz-zzzz-zzzz-zzzzzzzzzzzz",
            "000000000000000000000000000000000000",
        ] {
            assert_eq!(
                raw_sighting(&pool, "192.000.002.009", bad, MAC_ABSENT, T0, T0).await,
                Err(RepositoryError::Constraint("check")),
                "the L2 domain {bad:?} must be refused"
            );
        }
        assert_eq!(
            raw_sighting(&pool, "192.000.002.009", NIL, MAC_ABSENT, T1, T0).await,
            Err(RepositoryError::Constraint("check")),
            "a window whose first sighting is after its last must be refused"
        );
    }

    /// AC1 — one key, one row.
    #[tokio::test]
    async fn one_key_is_one_row() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        raw_sighting(&pool, "192.000.002.009", NIL, MAC_ABSENT, T0, T0)
            .await
            .expect("the first row");
        assert_eq!(
            raw_sighting(&pool, "192.000.002.009", NIL, MAC_ABSENT, T0, T1).await,
            Err(RepositoryError::Constraint("unique"))
        );
    }

    /// AC2/AC3 — an observation lands with its sightings, carrying its own instant to the
    /// microsecond (which is what routing it through `datetime_literal` must preserve).
    #[tokio::test]
    async fn an_observation_lands_with_its_sightings_and_its_exact_instant() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        let at = "2026-03-01T10:00:00.123456Z";
        crate::repo::insert_observation(
            &pool,
            &observation(domain(1), at, vec![ipv4(9), mac_fact(2), mac_fact(1)]),
        )
        .await
        .expect("insert");
        assert_eq!(
            load_sightings(&pool).await.expect("read"),
            vec![
                sighting(9, domain(1), Some(1), at, at),
                sighting(9, domain(1), Some(2), at, at),
            ]
        );
    }

    /// AC2 — a refused observation writes no sighting.
    #[tokio::test]
    async fn a_refused_observation_writes_no_sighting() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        let at = "2026-03-01T10:00:00Z";
        let first = observation(domain(1), at, vec![ipv4(9)]);
        crate::repo::insert_observation(&pool, &first)
            .await
            .expect("insert");
        let mut duplicate = first.clone();
        duplicate.facts = vec![ipv4(10)];
        let refused = crate::repo::insert_observation(&pool, &duplicate)
            .await
            .map_err(classify);
        assert_eq!(refused, Err(RepositoryError::Constraint("unique")));
        assert_eq!(
            load_sightings(&pool).await.expect("read"),
            vec![sighting(9, domain(1), None, at, at)],
            "the refused observation's address must not have been sighted"
        );
    }

    /// AC2 — inside a unit the write is a savepoint, so a unit failing AFTER the upsert rolls back
    /// both the observation and its sighting.
    #[tokio::test]
    async fn a_unit_failing_after_the_upsert_leaves_neither() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        let obs = observation(domain(1), "2026-03-01T10:00:00Z", vec![ipv4(9)]);
        let result = MariaRepository::new(pool.clone())
            .transact(move |unit| {
                let obs = obs.clone();
                Box::pin(async move {
                    crate::repo::insert_observation(unit.executor(), &obs)
                        .await
                        .map_err(classify)?;
                    let (written,): (i64,) =
                        sqlx::query_as("SELECT COUNT(*) FROM address_sighting")
                            .fetch_one(unit.executor())
                            .await
                            .map_err(classify)?;
                    assert_eq!(
                        written, 1,
                        "the premise: the sighting WAS written in the unit"
                    );
                    Err::<(), _>(RepositoryError::Backend("planted after the upsert".into()))
                })
            })
            .await;
        assert!(result.is_err());
        assert_eq!(count(&pool, "observation_record").await, 0);
        assert_eq!(count(&pool, "address_sighting").await, 0);
    }

    /// AC3 — a window widens whichever order the sightings arrive in, and never narrows.
    #[tokio::test]
    async fn a_window_widens_in_both_orders_and_never_narrows() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        let (a, b, c) = (
            "2026-03-01T10:00:01Z",
            "2026-03-01T10:00:02Z",
            "2026-03-01T10:00:03Z",
        );
        for at in [b, a, c, b] {
            crate::repo::insert_observation(&pool, &observation(domain(1), at, vec![ipv4(9)]))
                .await
                .expect("insert");
        }
        for at in [c, b, a] {
            crate::repo::insert_observation(&pool, &observation(domain(2), at, vec![ipv4(9)]))
                .await
                .expect("insert");
        }
        assert_eq!(
            load_sightings(&pool).await.expect("read"),
            vec![
                sighting(9, domain(1), None, a, c),
                sighting(9, domain(2), None, a, c),
            ]
        );
    }

    /// AC3 — an address seen with and without a MAC keeps both rows; the `None` one is not a
    /// second hardware address, which story 14.3b's conflict must know.
    #[tokio::test]
    async fn an_address_seen_with_and_without_a_mac_keeps_both_rows() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        let at = "2026-03-01T10:00:00Z";
        for facts in [vec![ipv4(9)], vec![ipv4(9), mac_fact(1)]] {
            crate::repo::insert_observation(&pool, &observation(domain(1), at, facts))
                .await
                .expect("insert");
        }
        assert_eq!(
            load_sightings(&pool).await.expect("read"),
            vec![
                sighting(9, domain(1), None, at, at),
                sighting(9, domain(1), Some(1), at, at),
            ]
        );
    }

    /// AC4 — two OS threads released by a `Barrier`, each landing observations that list the same
    /// 46 MACs in OPPOSITE orders on one address. Unsorted, the validation measured 200 deadlock
    /// victims in 400 transactions; the key order is what makes it zero.
    ///
    /// ⚠️ It calls [`insert_with_sightings`] and NOT [`ingest_observation`]: the replay would hide
    /// a deadlock, and this test measures whether one happens.
    #[tokio::test]
    async fn opposite_mac_orders_at_once_do_not_deadlock() {
        const ROUNDS: u32 = 40;
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        let url = std::env::var("DATABASE_URL").expect("store() checked it");
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let threads: Vec<_> = [false, true]
            .into_iter()
            .map(|reversed| {
                let url = url.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("runtime");
                    runtime.block_on(async move {
                        let pool = MySqlPool::connect(&url).await.expect("connect");
                        let mut macs: Vec<Fact> = (1..=46).map(mac_fact).collect();
                        if reversed {
                            macs.reverse();
                        }
                        barrier.wait();
                        let mut deadlocks = 0_u32;
                        for round in 0..ROUNDS {
                            let mut facts = vec![ipv4(77)];
                            facts.extend(macs.iter().cloned());
                            let at = format!("2026-03-01T10:00:{:02}Z", round % 60);
                            match insert_with_sightings(&pool, &observation(domain(9), &at, facts))
                                .await
                            {
                                Ok(()) => {}
                                Err(error) if is_deadlock(&error) => deadlocks += 1,
                                Err(error) => panic!("an unexpected ingest error: {error}"),
                            }
                        }
                        deadlocks
                    })
                })
            })
            .collect();
        let deadlocks: u32 = threads
            .into_iter()
            .map(|thread| thread.join().expect("the thread finished"))
            .sum();
        assert_eq!(
            deadlocks, 0,
            "two observations listing the same MACs in opposite orders deadlocked"
        );
        assert_eq!(count(&pool, "address_sighting").await, 46);
        assert_eq!(
            count(&pool, "observation_record").await,
            i64::from(ROUNDS * 2)
        );
    }

    // ── The backfill ─────────────────────────────────────────────────────────────────────────

    /// Two L2 domains, a two-MAC observation, a no-MAC one with two addresses, a lease-only one,
    /// and instants out of order — AC5's corpus.
    fn corpus() -> Vec<Observation> {
        vec![
            observation(
                domain(1),
                "2026-03-01T10:00:05Z",
                vec![ipv4(1), mac_fact(2), mac_fact(1)],
            ),
            observation(
                domain(1),
                "2026-03-01T10:00:01Z",
                vec![ipv4(1), mac_fact(1)],
            ),
            observation(
                domain(2),
                "2026-03-01T10:00:03Z",
                vec![ipv4(1), mac_fact(1)],
            ),
            observation(
                domain(1),
                "2026-03-01T10:00:04Z",
                vec![ipv4(2), ipv4(3), hostname()],
            ),
            observation(
                domain(2),
                "2026-03-01T10:00:02Z",
                vec![Fact::DhcpLease {
                    ip: ip(4),
                    expires_at: None,
                }],
            ),
            observation(domain(1), "2026-03-01T10:00:06Z", vec![ipv4(2)]),
        ]
    }

    /// The corpus's summary, written by hand — a second oracle beside the comparison of two paths.
    fn corpus_sightings() -> Vec<Sighting> {
        vec![
            sighting(
                1,
                domain(1),
                Some(1),
                "2026-03-01T10:00:01Z",
                "2026-03-01T10:00:05Z",
            ),
            sighting(
                1,
                domain(1),
                Some(2),
                "2026-03-01T10:00:05Z",
                "2026-03-01T10:00:05Z",
            ),
            sighting(
                1,
                domain(2),
                Some(1),
                "2026-03-01T10:00:03Z",
                "2026-03-01T10:00:03Z",
            ),
            sighting(
                2,
                domain(1),
                None,
                "2026-03-01T10:00:04Z",
                "2026-03-01T10:00:06Z",
            ),
            sighting(
                3,
                domain(1),
                None,
                "2026-03-01T10:00:04Z",
                "2026-03-01T10:00:04Z",
            ),
        ]
    }

    async fn ingest(pool: &MySqlPool, observations: &[Observation]) {
        for obs in observations {
            crate::repo::insert_observation(pool, obs)
                .await
                .expect("insert");
        }
    }

    /// AC5 — the backfill produces exactly what ingest maintains, is idempotent, and a later boot
    /// with the marker present does not re-read the history.
    #[tokio::test]
    async fn the_backfill_produces_exactly_what_ingest_maintains() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        let observations = corpus();
        ingest(&pool, &observations).await;
        let maintained = load_sightings(&pool).await.expect("read");
        assert_eq!(
            maintained,
            corpus_sightings(),
            "ingest against the hand-written oracle"
        );

        sqlx::query("DELETE FROM address_sighting")
            .execute(&pool)
            .await
            .expect("forget the summary");
        let report = BackfillReport {
            observations_read: 6,
            observations_skipped: 0,
            pairs_flushed: 5,
        };
        assert_eq!(
            backfill_at_boot(&pool).await.expect("backfill"),
            BackfillOutcome::Completed(report)
        );
        assert_eq!(
            load_sightings(&pool).await.expect("read"),
            maintained,
            "the backfill and ingest maintenance must agree row for row"
        );

        // A later boot: the marker is there, so the history is NOT read — proven by planting an
        // observation the backfill would count and a row it would have to skip.
        crate::repo::insert_observation(
            &pool,
            &observation(domain(1), "2026-03-01T10:00:07Z", vec![ipv4(8)]),
        )
        .await
        .expect("insert");
        plant_unreadable(&pool, "2026-03-01 10:00:08").await;
        assert_eq!(
            backfill_at_boot(&pool).await.expect("backfill"),
            BackfillOutcome::AlreadyDone(report),
            "the marker's report is returned unchanged: eight rows, one unreadable, were not re-read"
        );

        // A redo WIDENS, it never assigns — and a redo over the SAME history cannot tell the two
        // apart (the review's blind layer). So the observation that set (.1, mac 1)'s first sighting
        // is deleted first: constraint (3)'s own case, the summary remembering what the table forgot.
        // A flush that assigned would move that first sighting from 10:00:01 to 10:00:05.
        sqlx::query("DELETE FROM observation_record WHERE id = ?")
            .bind(observations[1].obs_id.to_string())
            .execute(&pool)
            .await
            .expect("forget the earliest observation");
        sqlx::query("DELETE FROM sighting_backfill")
            .execute(&pool)
            .await
            .expect("forget the marker");
        let before = load_sightings(&pool).await.expect("read");
        assert_eq!(
            before[0],
            sighting(
                1,
                domain(1),
                Some(1),
                "2026-03-01T10:00:01Z",
                "2026-03-01T10:00:05Z"
            ),
            "the premise: the deleted observation's instant is the first sighting on record"
        );
        assert!(matches!(
            backfill_at_boot(&pool).await.expect("backfill"),
            BackfillOutcome::Completed(BackfillReport {
                observations_skipped: 1,
                ..
            })
        ));
        assert_eq!(
            load_sightings(&pool).await.expect("read"),
            before,
            "a redo over a history that lost an observation must keep what that observation sighted"
        );
    }

    /// Plant an observation this binary cannot decode — a fact kind it does not know, which is how a
    /// newer binary's row reads to this one — around the adapter.
    async fn plant_unreadable(pool: &MySqlPool, at: &str) {
        sqlx::query(
            "INSERT INTO observation_record \
             (id, connector_id, observed_at, l2_domain, vantage, facts, raw) \
             VALUES (?, ?, ?, ?, ?, ?, NULL)",
        )
        .bind(uuid::Uuid::now_v7().to_string())
        .bind(NIL)
        .bind(at)
        .bind(NIL)
        .bind(NIL)
        .bind(r#"[{"IpV4":{"addr":"192.0.2.50"}},{"Lldp":{"chassis":"x"}}]"#)
        .execute(pool)
        .await
        .expect("plant an unreadable observation");
    }

    /// AC2 on the PRODUCTION path — a pool, not a unit. Nothing had made a sighting write fail AFTER
    /// the observation row, so the review's edge layer replaced the rollback with a commit and every
    /// test stayed green. A row held by another transaction does it: with a one-second lock wait the
    /// upsert times out (1205), and a timed-out statement is rolled back alone, not its transaction.
    #[tokio::test]
    async fn a_sighting_write_failing_after_the_row_leaves_no_observation() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        let url = std::env::var("DATABASE_URL").expect("store() checked it");
        let impatient = sqlx::mysql::MySqlPoolOptions::new()
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    sqlx::query("SET SESSION innodb_lock_wait_timeout = 1")
                        .execute(conn)
                        .await
                        .map(|_| ())
                })
            })
            .connect(&url)
            .await
            .expect("connect");
        let at = "2026-03-01T10:00:00Z";
        let mut holder = pool.begin().await.expect("begin");
        upsert_sighting(
            &mut holder,
            &key("192.000.002.009", domain(1), MAC_ABSENT),
            ts(at),
            ts(at),
        )
        .await
        .expect("hold the row");
        let refused =
            insert_with_sightings(&impatient, &observation(domain(1), at, vec![ipv4(9)])).await;
        holder.rollback().await.expect("release the row");
        assert_eq!(
            refused.map_err(classify),
            Err(RepositoryError::Contention),
            "the premise: the upsert waited out its lock after the observation row was written"
        );
        assert_eq!(
            count(&pool, "observation_record").await,
            0,
            "the observation row must be rolled back with the sighting that could not be written"
        );
        assert_eq!(count(&pool, "address_sighting").await, 0);
    }

    /// Two instances booting on one store at once both complete. The first design refused the second
    /// on the marker's duplicate key after its flush had already widened every row (measured `1062`).
    #[tokio::test]
    async fn two_boots_at_once_both_complete() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        ingest(&pool, &corpus()).await;
        sqlx::query("DELETE FROM address_sighting")
            .execute(&pool)
            .await
            .expect("forget the summary");
        let url = std::env::var("DATABASE_URL").expect("store() checked it");
        let other = MySqlPool::connect(&url).await.expect("connect");
        let (first, second) = tokio::join!(backfill_at_boot(&pool), backfill_at_boot(&other));
        assert!(
            first.is_ok() && second.is_ok(),
            "both boots must complete: {first:?} / {second:?}"
        );
        assert_eq!(
            load_sightings(&pool).await.expect("read"),
            corpus_sightings()
        );
        assert!(load_backfill_marker(&pool).await.expect("marker").is_some());
    }

    /// AC5 — an undecodable row is skipped, named in the log, counted in the marker, and the
    /// backfill completes over it.
    #[tokio::test]
    async fn an_undecodable_row_is_skipped_counted_and_the_backfill_completes() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        let good = observation(
            domain(1),
            "2026-03-01T10:00:01Z",
            vec![ipv4(1), mac_fact(1)],
        );
        ingest(&pool, std::slice::from_ref(&good)).await;
        plant_unreadable(&pool, "2026-03-01 10:00:02").await;
        sqlx::query("DELETE FROM address_sighting")
            .execute(&pool)
            .await
            .expect("forget the summary");

        let report = BackfillReport {
            observations_read: 2,
            observations_skipped: 1,
            pairs_flushed: 1,
        };
        assert_eq!(
            backfill_at_boot(&pool)
                .await
                .expect("the backfill completes over the bad row"),
            BackfillOutcome::Completed(report)
        );
        assert_eq!(
            load_backfill_marker(&pool).await.expect("marker"),
            Some(report)
        );
        assert_eq!(
            load_sightings(&pool).await.expect("read"),
            vec![sighting(
                1,
                domain(1),
                Some(1),
                "2026-03-01T10:00:01Z",
                "2026-03-01T10:00:01Z"
            )]
        );
    }

    /// AC5 — a flush that fails part-way leaves no pair and no marker, and the next boot completes.
    #[tokio::test]
    async fn a_failed_flush_leaves_nothing_and_the_next_boot_completes() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        ingest(&pool, &corpus()).await;
        sqlx::query("DELETE FROM address_sighting")
            .execute(&pool)
            .await
            .expect("forget the summary");

        let (mut pairs, read, skipped) = fold_history(&pool).await.expect("fold");
        // A key the schema refuses, sorting AFTER every real one — so the real upserts have run
        // when it fails, and only the transaction can undo them.
        let at = ts("2026-03-01T10:00:00Z");
        pairs.insert(key("255.255.255.255", domain(1), "ZZ"), (at, at));
        assert!(flush_backfill(&pool, &pairs, read, skipped).await.is_err());
        assert_eq!(count(&pool, "address_sighting").await, 0);
        assert_eq!(load_backfill_marker(&pool).await.expect("marker"), None);

        assert!(matches!(
            backfill_at_boot(&pool).await.expect("backfill"),
            BackfillOutcome::Completed(_)
        ));
        assert_eq!(
            load_sightings(&pool).await.expect("read"),
            corpus_sightings()
        );
    }

    // ── What must not change ─────────────────────────────────────────────────────────────────

    /// AC7 — the identity pass, its purge and its replay never touch the summary.
    #[tokio::test]
    async fn the_identity_pass_and_its_purge_and_replay_never_touch_the_summary() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        let observations = corpus();
        ingest(&pool, &observations).await;
        let before = load_sightings(&pool).await.expect("read");
        let repo = MariaRepository::new(pool.clone());
        for _pass in 0..2 {
            let batch = observations.clone();
            repo.transact(move |unit| {
                let batch = batch.clone();
                Box::pin(async move { crate::resolver::resolve(unit.executor(), &batch).await })
            })
            .await
            .expect("the identity pass");
            assert_eq!(load_sightings(&pool).await.expect("read"), before);
            crate::repo::purge_engine_links(&pool).await.expect("purge");
            assert_eq!(load_sightings(&pool).await.expect("read"), before);
        }
    }

    // ── The seeds (AC8) ──────────────────────────────────────────────────────────────────────

    /// The summary the stored observations imply, folded in Rust — the oracle a seed's hand-written
    /// rows are compared with.
    async fn implied_by_the_observations(pool: &MySqlPool) -> Vec<Sighting> {
        let (pairs, _read, skipped) = fold_history(pool).await.expect("fold");
        assert_eq!(skipped, 0, "every seeded observation must decode");
        pairs
            .iter()
            .map(|(key, (first, last))| decoded(key, *first, *last))
            .collect()
    }

    /// AC8 — the accessibility seed writes exactly the summary its observations imply, from ONE
    /// instant. `sqlx::raw_sql` runs the whole file on one connection, so `@t` holds across it.
    #[tokio::test]
    async fn the_accessibility_seed_writes_the_summary_its_observations_imply() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        sqlx::raw_sql(include_str!("../../../a11y/seed.sql"))
            .execute(&pool)
            .await
            .expect("the seed runs");
        let seeded = load_sightings(&pool).await.expect("read");
        assert_eq!(
            seeded.len(),
            4,
            "the premise: the seed sights four addresses"
        );
        assert_eq!(seeded, implied_by_the_observations(&pool).await);
        assert!(
            seeded
                .iter()
                .all(|s| s.first_seen_at == seeded[0].first_seen_at
                    && s.last_seen_at == seeded[0].first_seen_at),
            "one @t for every observation and every sighting"
        );
        forget_the_seed(&pool).await;
    }

    /// Remove what a seed wrote, children first, so a seed test does not leave four declared
    /// entities, two subnets and their links in the shared test store (the review's blind layer).
    /// ⚠️ A failing assertion skips it; each other test's fixture clears its own tables.
    async fn forget_the_seed(pool: &MySqlPool) {
        for statement in [
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM ip_address",
            "DELETE FROM ip_range",
            "DELETE FROM ip_subnet",
            "DELETE FROM observation_record",
            "DELETE FROM address_sighting",
            "DELETE FROM declared_attribute",
        ] {
            sqlx::query(statement)
                .execute(pool)
                .await
                .expect("forget the seed");
        }
    }

    /// AC8 — the operator's demo seed, run twice: its sighting follows its observation, and a re-run
    /// does not keep the first run's instant.
    #[tokio::test]
    async fn the_demo_seed_writes_its_sighting_and_a_rerun_follows_it() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };
        for _run in 0..2 {
            sqlx::raw_sql(include_str!("../../../docker/seed-example.sql"))
                .execute(&pool)
                .await
                .expect("the seed runs");
            let seeded = load_sightings(&pool).await.expect("read");
            assert_eq!(seeded.len(), 1, "the premise: the seed sights one address");
            assert_eq!(seeded, implied_by_the_observations(&pool).await);
        }
        forget_the_seed(&pool).await;
    }

    // ── The measurement (AC5, AC6, AC9) ──────────────────────────────────────────────────────

    fn peak_resident_kib() -> u64 {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|status| {
                status
                    .lines()
                    .find(|line| line.starts_with("VmHWM:"))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|kib| kib.parse().ok())
            })
            .unwrap_or(0)
    }

    /// Opt-in: `OPENCMDB_MEASURE_SIGHTINGS=<observation rows>` with `--release --nocapture`.
    /// Generates the rows server-side in the shape of the ARP/ping connector's (46 hosts, a sweep
    /// every five minutes), times the backfill and its peak memory, the reader at the reference
    /// summary size and at 10 000 pairs, and the ingest cost per observation with and without the
    /// summary. It returns at once when the variable is unset, so the suite never pays for it.
    #[tokio::test]
    async fn measure_the_backfill_the_reader_and_the_ingest_cost() {
        let Ok(rows) = std::env::var("OPENCMDB_MEASURE_SIGHTINGS") else {
            return;
        };
        let rows: u64 = rows.parse().expect("a row count");
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = store().await else {
            return;
        };

        let started = std::time::Instant::now();
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "INSERT INTO observation_record (id, connector_id, observed_at, l2_domain, vantage, facts, raw) \
             SELECT CONCAT('eeeeeeee-0000-0000-0000-', LPAD(LOWER(HEX(seq)), 12, '0')), '{NIL}', \
             TIMESTAMP('2026-01-01 00:00:00') + INTERVAL ((seq - 1) DIV 46) * 300 SECOND, '{NIL}', '{NIL}', \
             CONCAT('[{{\"IpV4\":{{\"addr\":\"10.0.0.', ((seq - 1) MOD 46) + 1, \
             '\"}}}},{{\"Mac\":{{\"addr\":[2,0,0,0,0,', ((seq - 1) MOD 46) + 1, \
             '],\"locally_administered\":true}}}},{{\"Rtt\":{{\"millis\":3}}}}]'), NULL \
             FROM seq_1_to_{rows}"
        )))
        .execute(&pool)
        .await
        .expect("generate the observations");
        println!(
            "MEASURE generated {rows} observation rows in {:?}",
            started.elapsed()
        );

        let hwm_before = peak_resident_kib();
        let started = std::time::Instant::now();
        let outcome = backfill_at_boot(&pool).await.expect("backfill");
        println!(
            "MEASURE backfill {outcome:?} in {:?}; VmHWM {hwm_before} kB -> {} kB",
            started.elapsed(),
            peak_resident_kib()
        );

        let started = std::time::Instant::now();
        let summary = load_sightings(&pool).await.expect("read");
        println!(
            "MEASURE reader: {} pairs in {:?}",
            summary.len(),
            started.elapsed()
        );
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "INSERT INTO address_sighting (addr, l2_domain, mac, first_seen_at, last_seen_at) \
             SELECT CONCAT('172.', LPAD((seq DIV 65536) MOD 256, 3, '0'), '.', \
             LPAD((seq DIV 256) MOD 256, 3, '0'), '.', LPAD(seq MOD 256, 3, '0')), '{NIL}', '-', \
             '{T0}', '{T0}' FROM seq_1_to_10000"
        )))
        .execute(&pool)
        .await
        .expect("synthesise pairs");
        let started = std::time::Instant::now();
        let summary = load_sightings(&pool).await.expect("read");
        println!(
            "MEASURE reader: {} pairs in {:?}",
            summary.len(),
            started.elapsed()
        );

        // Ingest: 100 sweeps of 46 hosts, one transaction per observation, as the scan pass does.
        let sweeps: Vec<Observation> = (0..100_u32)
            .flat_map(|sweep| {
                (1..=46_u8).map(move |host| {
                    observation(
                        domain(1),
                        &format!("2026-06-01T{:02}:{:02}:00Z", sweep / 60, sweep % 60),
                        vec![
                            Fact::IpV4 {
                                addr: Ipv4Addr::new(10, 1, 0, host),
                            },
                            mac_fact(host),
                            Fact::Rtt { millis: 3 },
                        ],
                    )
                })
            })
            .collect();
        for round in 0..2 {
            for with_summary in [false, true] {
                for statement in [
                    "DELETE FROM observation_record",
                    "DELETE FROM address_sighting",
                ] {
                    sqlx::query(statement).execute(&pool).await.expect("clean");
                }
                let started = std::time::Instant::now();
                for obs in &sweeps {
                    if with_summary {
                        insert_with_sightings(&pool, obs).await.expect("ingest");
                    } else {
                        let mut tx = pool.begin().await.expect("begin");
                        insert_observation_row(&mut *tx, obs).await.expect("insert");
                        tx.commit().await.expect("commit");
                    }
                }
                let per = started.elapsed() / u32::try_from(sweeps.len()).expect("fits");
                println!(
                    "MEASURE ingest round {round} with_summary={with_summary}: {} observations, {per:?} each",
                    sweeps.len()
                );
            }
        }
        for statement in [
            "DELETE FROM observation_record",
            "DELETE FROM address_sighting",
            "DELETE FROM sighting_backfill",
        ] {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }
    }
}
