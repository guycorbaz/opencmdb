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

/// The `current_subject` of a current link that names no interface — an abstention.
///
/// This is D21's `NIL_INTERFACE`, which the register names in the same breath as [`OPEN_END`] and
/// for the same reason: *"Same reasoning for `NIL_INTERFACE`/`NIL_DEVICE`"* [architecture.md:1468].
/// `interface_id` is NULL for an abstention, the uniqueness key contains the subject, and MariaDB
/// holds NULLs distinct — so without the sentinel two current abstentions for one observation
/// would both insert and the constraint would be decorative for exactly the half FR16 exists to
/// display. `identity_link_current_subject` is what stops it drifting from what it stands for.
///
/// It is **never** an `interface.id` — `interface_id` keeps its foreign key and stays NULL — and
/// `interface_id_not_nil` refuses an interface that would collide with it.
pub const ABSTAINED_SUBJECT: &str = "00000000-0000-0000-0000-000000000000";

/// [`OPEN_END`] as a [`Timestamp`], for the callers that pass a link's `valid_to` as a parameter.
///
/// The single derivation site, and it is checked against the literal by
/// `the_sentinel_instant_renders_as_the_sentinel_literal`: two spellings of one instant that drift
/// apart would make `identity_link_current_subject` refuse every current link the resolver writes,
/// with a `check` violation naming nothing a reader could act on.
///
/// _(This lived in `repo.rs`'s test module until story 5.9b, which needs it in production: the
/// resolver writes every current link at the sentinel.)_
///
/// # Panics
///
/// Never in practice — the date is a literal in range, and the `expect` documents that.
pub(crate) fn open_end() -> Timestamp {
    use chrono::TimeZone;
    chrono::Utc
        .with_ymd_and_hms(9999, 12, 31, 23, 59, 59)
        .single()
        .expect("the sentinel instant")
        + chrono::Duration::microseconds(999_999)
}

/// The persisted token for a [`Conclusion`], by an exhaustive `match`.
///
/// No `#[derive(Serialize)]`, deliberately: a derived variant name is a wire format nobody chose,
/// and renaming a variant would silently rewrite stored bytes — the *"silent data migration, the
/// worst kind"* D14 names about `ruleset_version`. [`Conclusion`] is also deliberately not
/// `#[non_exhaustive]`, so a new variant produces `error[E0004]` here. **No `_` arm** — the `_` is
/// what turns that compile error into a silent mis-classification.
pub(crate) fn outcome_token(conclusion: &Conclusion) -> &'static str {
    match conclusion {
        Conclusion::Match { .. } => "match",
        Conclusion::NoMatch { .. } => "no_match",
        Conclusion::Abstained { .. } => "abstained",
    }
}

/// The persisted token for an [`IdentityAbstentionCause`], by an exhaustive `match`.
///
/// Same refusal and same reason as [`outcome_token`].
pub(crate) fn cause_token(cause: &IdentityAbstentionCause) -> &'static str {
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

/// Find the interface an L1 key names, or `None` when the key has never been seen.
///
/// This is what makes a re-run reproducible: `0002`'s header states that *"the re-run finds an
/// interface by its key"*, and if the id were re-minted on every pass, every reproduced link would
/// carry a different `interface_id` and story 5.10's bit-for-bit purge test could never pass. It is
/// also what makes read-your-own-writes real rather than a convention — called with a unit of
/// work's connection, it sees an interface the same transaction just inserted.
///
/// # Why an ordered first match is correct here
///
/// `interface_l1_key` is deliberately NOT unique (D21): a cloned MAC is two real interfaces sharing
/// one address, and a UNIQUE there would turn the case the engine must ABSTAIN on into a 500. So
/// this returns **an** interface for the key, `ORDER BY id` for determinism rather than for meaning
/// — [`InterfaceId`] is a UUID and its order is a construction device, exactly as
/// `CandidatePair`'s is. Telling two interfaces on one key apart is the cloned-MAC problem and
/// belongs to Epic 6; until then a second row on the same key is unreachable through the resolver,
/// which mints at most one interface per key per pass.
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`]. A stored id that is
/// not a UUID decodes as [`sqlx::Error::Decode`] — the id newtypes have no `FromStr`, so the parse
/// happens here.
pub async fn find_interface_by_l1_key<'e, E>(
    executor: E,
    l2_domain: L2DomainId,
    mac_canon: &MacAddr,
) -> Result<Option<InterfaceId>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT id FROM interface WHERE l2_domain = ? AND mac_canon = ? ORDER BY id LIMIT 1",
    )
    .bind(l2_domain.to_string())
    .bind(mac_canon.to_string())
    .fetch_all(executor)
    .await?;
    match rows.into_iter().next() {
        None => Ok(None),
        Some((id,)) => {
            let parsed =
                uuid::Uuid::parse_str(&id).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
            Ok(Some(InterfaceId::from_uuid(parsed)))
        }
    }
}

/// Widen an interface's seen-window to cover two more instants, never narrowing it.
///
/// `LEAST`/`GREATEST` rather than a read-modify-write in Rust, for one reason: `sqlx` is built here
/// without its `chrono` feature, so a `DATETIME(6)` has no Rust type to decode into and the window
/// could only be read back as the string MariaDB renders — comparing those in Rust would be an
/// instant comparison wearing a string costume.
///
/// **This is not the comparison D10 forbids.** D10 keeps SQL out of *domain value* comparison
/// because identity is the product; a seen-window is bookkeeping, no value is under judgement, and
/// MariaDB is the only engine (D64). _(Enabling `sqlx`'s `chrono` feature would let this be written
/// in Rust and would collapse `load_link_valid_to`'s second rendering site with it; that is
/// registered, with the first story that needs to read an instant back as a VALUE as its owner.)_
///
/// Widening rather than assigning is what makes an out-of-order arrival safe: a batch older than
/// the stored window must extend `first_seen_at` backwards, and a narrowed `first_seen_at` is
/// unrecoverable.
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`].
pub async fn widen_interface_seen_window<'e, E>(
    executor: E,
    id: InterfaceId,
    first_seen_at: Timestamp,
    last_seen_at: Timestamp,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    sqlx::query(
        "UPDATE interface \
         SET first_seen_at = LEAST(first_seen_at, ?), last_seen_at = GREATEST(last_seen_at, ?) \
         WHERE id = ?",
    )
    .bind(datetime_literal(first_seen_at))
    .bind(datetime_literal(last_seen_at))
    .bind(id.to_string())
    .execute(executor)
    .await?;
    Ok(())
}

/// Format an instant the way MariaDB writes one. The single formatting site; do not invent a
/// second format string.
///
/// `pub(crate)` rather than private since story 5.9b: `resolver.rs` reads instants back with
/// `CAST(… AS CHAR)` and compares them against this rendering, and a private function here is what
/// makes a second format string the natural reflex over there.
pub(crate) fn datetime_literal(at: Timestamp) -> String {
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
    let valid_to_literal = datetime_literal(valid_to);
    sqlx::query(
        "INSERT INTO identity_link \
         (id, observation_id, interface_id, current_subject, outcome, rule_id, abstention_cause, \
          evidence, ruleset_version, decided_by, valid_from, valid_to) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(observation_id.to_string())
    .bind(interface.map(|i| i.to_string()))
    .bind(current_subject_of(interface, &valid_to_literal))
    .bind(outcome_token(&decision.conclusion))
    .bind(rule_id)
    .bind(abstention_cause)
    .bind(evidence_json)
    .bind(decision.ruleset_version.0)
    .bind(decided_by.token())
    .bind(datetime_literal(valid_from))
    .bind(&valid_to_literal)
    .execute(executor)
    .await?;
    Ok(())
}

/// The subject a CURRENT link occupies — the interface, or [`ABSTAINED_SUBJECT`] when it names none.
///
/// The single derivation site, and the one callers outside this module want: it answers *"which
/// slot in `identity_link_one_current` does this placement hold?"* without making the caller render
/// an instant it does not have a use for. [`current_subject_of`] delegates here, so the sentinel is
/// still spelled once.
///
/// _(Split out of `current_subject_of` by story 5.11: the resolver needs the subject to LOOK UP the
/// current version before it writes, and reaching the old signature from there meant passing
/// `&datetime_literal(open_end())` and unwrapping an `Option` on a branch that cannot be taken —
/// a panic path bought for nothing.)_
pub(crate) fn subject_of(interface: Option<InterfaceId>) -> String {
    interface.map_or_else(|| ABSTAINED_SUBJECT.to_string(), |i| i.to_string())
}

/// The `current_subject` COLUMN for a link pointing at `interface` and expiring at
/// `valid_to_literal` — [`subject_of`] while the row is current, `None` once it is not.
fn current_subject_of(interface: Option<InterfaceId>, valid_to_literal: &str) -> Option<String> {
    if valid_to_literal != OPEN_END {
        return None;
    }
    Some(subject_of(interface))
}

/// Close a CURRENT link by stamping its `valid_to` and dropping it out of the uniqueness key — an
/// SCD2 supersede is this plus an append.
///
/// The old row stays readable with its old `valid_to`: *"a bad link is UNLINKED, never erased"*
/// [architecture.md:1016-1017]. `closed_at` is a parameter, never the clock.
///
/// # Three refusals, each of which was measured happening before it existed
///
/// - **only a current row closes.** The `WHERE` names [`OPEN_END`]; without it, re-closing an
///   already-closed row rewrote its historical stamp and returned `Ok(())`, and closing one back
///   AT the sentinel resurrected a superseded link as current.
/// - **closing nothing is an error.** `rows_affected() == 0` is [`RepositoryError::NotFound`];
///   without it, closing an unknown id returned `Ok(())` and the caller's supersede then failed
///   on the append with a confusing uniqueness error.
/// - **`closed_at` may not be [`OPEN_END`].** The sentinel is a reserved value the type cannot
///   exclude, so the function must: closing at it left the link current while reporting success.
///
/// # Errors
///
/// [`RepositoryError::NotFound`] when no current link with that id exists, and
/// [`RepositoryError::Constraint`] when `closed_at` is the sentinel or would invert the interval.
pub async fn close_identity_link<'e, E>(
    executor: E,
    id: LinkId,
    closed_at: Timestamp,
) -> Result<(), RepositoryError>
where
    E: Executor<'e, Database = MySql>,
{
    let closed_at_literal = datetime_literal(closed_at);
    if closed_at_literal == OPEN_END {
        return Err(RepositoryError::Constraint("check"));
    }
    let result = sqlx::query(
        "UPDATE identity_link SET valid_to = ?, current_subject = NULL \
         WHERE id = ? AND valid_to = ?",
    )
    .bind(&closed_at_literal)
    .bind(id.to_string())
    .bind(OPEN_END)
    .execute(executor)
    .await
    .map_err(classify)?;
    if result.rows_affected() == 0 {
        return Err(RepositoryError::NotFound);
    }
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

/// One `identity_link` row as sqlx decodes it, before it becomes a [`PersistedLink`]:
/// `(id, interface_id, outcome, rule_id, abstention_cause, evidence, ruleset_version, decided_by)`.
/// The three `Option`s are the three nullable columns — a non-`Option` binding fails to decode.
type LinkRow = (
    String,
    Option<String>,
    String,
    Option<String>,
    Option<String>,
    String,
    u32,
    String,
);

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
    let rows: Vec<LinkRow> = sqlx::query_as(
        "SELECT id, interface_id, outcome, rule_id, abstention_cause, evidence, \
                ruleset_version, decided_by \
         FROM identity_link WHERE observation_id = ? AND current_subject IS NOT NULL \
         ORDER BY current_subject",
    )
    .bind(observation_id.to_string())
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

/// Load the CURRENT link the ENGINE holds on one `(observation_id, subject)` slot, if any.
///
/// This is what a compare-then-supersede write path reads before it decides whether to write at all
/// (story 5.11). It is a SIBLING of [`load_current_links_for_observation`] rather than a widening of
/// it: that function is plural by design — the L1 join puts one observation on every key it carries
/// — and two dozen call sites depend on its shape.
///
/// # 🔴 `decided_by = 'ENGINE'` is the load-bearing clause
///
/// Without it, a compare-then-supersede path finds an OPERATOR's row in the slot and treats it as
/// its own. **Two things follow, and the second is the one an earlier version of this doc claimed
/// while measuring the first:**
///
/// - when the human's row happens to carry the decision the engine would have written, the pass
///   **ADOPTS it** — `same_decision` returns `true`, the slot is reported `Unchanged`, and the
///   engine silently takes credit for a human's assertion. This is what mutation M1 actually
///   reddens, because the operator rows in both its tests carry `nothing_was_evaluated()`;
/// - when it carries a DIFFERENT decision, the pass **supersedes it** — a human's assertion closed
///   and replaced by a derivation. `the_engine_never_adopts_or_supersedes_a_differing_operator_row`
///   is the test that measures this one, and it exists because nothing did.
///
/// With the clause, the operator's row is invisible here, the caller falls through to its insert,
/// and `identity_link_one_current` refuses it exactly as it did before story 5.11 existed:
/// `Constraint("unique")` and a rolled-back pass. *"May an operator override the engine?"* stays
/// story 5.14's question, unanswered rather than answered by accident.
///
/// `subject` is [`subject_of`]'s value — an interface id, or [`ABSTAINED_SUBJECT`] for an
/// abstention. It never carries `valid_to`: `identity_link_current_subject` makes
/// `current_subject IS NOT NULL` equivalent to *"current"*, so naming the subject IS naming the
/// current row.
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`].
pub async fn load_current_engine_link<'e, E>(
    executor: E,
    observation_id: ObsId,
    subject: &str,
) -> Result<Option<CurrentEngineLink>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    type Row = (
        String,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
        String,
        u32,
        String,
        String,
    );
    // `fetch_optional`, not `fetch_all().next()`: at most one row is not a hope, it is
    // `identity_link_one_current (observation_id, current_subject)` with a non-NULL bind, and
    // asking the driver for one row makes the impossibility of a second one structural rather than
    // a silent discard.
    let row: Option<Row> = sqlx::query_as(
        "SELECT id, interface_id, outcome, rule_id, abstention_cause, evidence, \
                ruleset_version, decided_by, CAST(valid_from AS CHAR) \
         FROM identity_link \
         WHERE observation_id = ? AND current_subject = ? AND decided_by = 'ENGINE'",
    )
    .bind(observation_id.to_string())
    .bind(subject)
    .fetch_optional(executor)
    .await?;
    let Some(r) = row else {
        return Ok(None);
    };
    Ok(Some(CurrentEngineLink {
        link: PersistedLink {
            id: r.0,
            interface_id: r.1,
            outcome: r.2,
            rule_id: r.3,
            abstention_cause: r.4,
            evidence: serde_json::from_str(&r.5).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
            ruleset_version: r.6,
            decided_by: r.7,
        },
        valid_from: r.8,
    }))
}

/// The current ENGINE version of one slot, plus the instant it opened at.
///
/// `valid_from` is here and NOT on [`PersistedLink`] because only the supersede path needs it:
/// before closing a version, the writer must refuse an instant that would run the interval
/// backwards, and that refusal is the one place in this crate that compares an instant the caller
/// HOLDS against one the database STORED.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrentEngineLink {
    /// The link as it was read back.
    pub link: PersistedLink,
    /// When this version opened, as MariaDB renders it — the same shape [`datetime_literal`]
    /// produces, which is what makes the comparison a string comparison rather than a parse.
    pub valid_from: String,
}

/// Every CURRENT engine slot one observation occupies: `(link id, subject)`, ordered by subject.
///
/// The resolver needs this to close what it did NOT visit. `write_link` only ever reads the slot it
/// is about to fill, so a key that vanished from the input produces no visit and its link would stay
/// current forever — pointing at an interface no fact in the input supports. Before story 5.11 the
/// blind append made that case fail loudly on `identity_link_one_current`; the compare routes
/// around the key, so the detection has to be explicit.
///
/// `decided_by = 'ENGINE'`, for [`load_current_engine_link`]'s reason: the engine closes its own
/// beliefs and never a human's.
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`].
pub async fn load_current_engine_slots<'e, E>(
    executor: E,
    observation_id: ObsId,
) -> Result<Vec<(String, String)>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT id, current_subject FROM identity_link \
         WHERE observation_id = ? AND current_subject IS NOT NULL AND decided_by = 'ENGINE' \
         ORDER BY current_subject",
    )
    .bind(observation_id.to_string())
    .fetch_all(executor)
    .await?;
    Ok(rows)
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

/// Delete every link the ENGINE derived, and return how many LINKS went.
///
/// ⚠️ **The count excludes cascaded `link_candidate` rows** — measured: two links carrying two
/// candidates report `2`, not `4`, because InnoDB does not report cascaded deletions to the client.
/// A caller logging *"purged N rows"* would understate.
///
/// ⚠️ **It is GLOBAL and unscoped**: every engine link in the database, superseded ones included.
/// The list below says what it leaves; what it takes is everything else. A replay that covers fewer
/// observations than the purge removed loses the difference **silently** — measured: purge six,
/// replay two, `Ok(2)`, and the snapshot goes 6 → 2 with no error. **The caller's precondition is
/// that the replay covers the same observation set.**
///
/// **It is a `DELETE`, not a `TRUNCATE`.** D14 writes the purge as
/// `TRUNCATE ... WHERE decided_by='ENGINE'` [architecture.md:1038-1039] and `epics.md:1627` repeats
/// it, but **MariaDB's `TRUNCATE` takes no `WHERE` clause** — measured: `TRUNCATE TABLE t WHERE 1=0`
/// is `ERROR 1064` at the parser. The register carries the correction; this is the statement that
/// runs.
///
/// `link_candidate` rows follow their link by `ON DELETE CASCADE` — story 5.9's review measured
/// `RESTRICT` failing `ERROR 1451` the moment an engine link carried a candidate, which is exactly
/// the ambiguity case the table exists for.
///
/// ⚠️ **`interface` rows are NOT touched**, and story 5.10 rests on it: the replay finds each
/// interface by its key rather than minting a new one, so every reproduced link points at the same
/// row. Purging interfaces here would make the whole invariant unmeasurable.
///
/// ⚠️ **`decided_by='OPERATOR'` rows are NOT touched either.** They are INPUTS, not derivations, on
/// a par with an observation — D14's *"two natures in one table, and if that frontier is fuzzy in
/// the code, the invariant is dead"*.
///
/// ⚠️ **It deletes the engine's HISTORY, not only its current beliefs.** There is no
/// `current_subject` filter here, so superseded engine versions go too — while [`snapshot_links`]
/// only ever compared current rows. A purge-and-replay therefore leaves the store SMALLER than it
/// found it, and the snapshots still compare equal:
/// `a_purge_after_a_supersede_loses_history_and_still_replays` measures both numbers, because the
/// equal snapshots alone hide the loss.
///
/// **That is deliberate** (Guy's arbitration at story 5.11's contexting). A link is *"a cache of
/// attention, not of truth"* [architecture.md:1036-1039], so what the engine believed yesterday is
/// not a truth to preserve and the replay owes history nothing.
/// `architecture.md:1016-1017`'s *"a bad link is UNLINKED, never erased"* governs an OPERATOR's
/// correction of a live belief, not the engine's own scratch history. Inert until story 5.11,
/// which is the first that supersedes.
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`].
pub async fn purge_engine_links<'e, E>(executor: E) -> Result<u64, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    let result = sqlx::query("DELETE FROM identity_link WHERE decided_by = 'ENGINE'")
        .execute(executor)
        .await?;
    Ok(result.rows_affected())
}

/// One current link, reduced to what CARRIES ITS DECISION — and to nothing else.
///
/// # There is deliberately no `id` field
///
/// `identity_link.id` is a v7 UUID, and a v7 UUID embeds a 48-bit wall-clock millisecond, so a
/// replayed link is minted with a different one every time — measured at story 5.9b's code review,
/// two runs over identical input 57 ms apart. D14's *"reproduce the same decisions bit for bit"*
/// therefore cannot mean the row identifier, and **a row identifier is not a decision**.
///
/// The exclusion is STRUCTURAL rather than a habit: there is no field to forget to skip.
/// `interface_id` IS here, which is what makes the exclusion safe — if the replay re-minted its
/// interfaces, every reproduced link would point elsewhere and the comparison would red.
///
/// `current_subject` is absent for a different reason: it is `interface_id`-or-`NIL_INTERFACE`
/// while current, held to that by `identity_link_current_subject`, so it is a FUNCTION of a field
/// already here and comparing it would measure nothing new. It is the `ORDER BY` key instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkSnapshot {
    /// The observation the link places.
    pub observation_id: String,
    /// The interface it places it on — `None` for an abstention.
    pub interface_id: Option<String>,
    /// The persisted outcome token.
    pub outcome: String,
    /// The rule that settled it — `None` for an abstention.
    pub rule_id: Option<String>,
    /// Why it abstained — `None` unless it did.
    pub abstention_cause: Option<String>,
    /// The observations that justified it.
    pub evidence: Vec<ObsId>,
    /// The ruleset that produced it (D14).
    pub ruleset_version: u32,
    /// `ENGINE` or `OPERATOR` — the frontier D14 calls load-bearing.
    pub decided_by: String,
    /// When the version opened, as MariaDB renders it.
    pub valid_from: String,
    /// When it closes.
    ///
    /// ⚠️ **Under [`snapshot_links`]' own `WHERE` this field is a CONSTANT.**
    /// `identity_link_current_subject` makes `current_subject IS NOT NULL ⟺ valid_to = OPEN_END`,
    /// so every row this snapshot can return carries the sentinel and this column can never carry a
    /// divergence. It is here for shape rather than for evidence — measured at the code review,
    /// which also measured that blanking it leaves the whole suite green. `valid_from` is NOT in
    /// that position: it is genuinely data-derived.
    pub valid_to: String,
}

/// Every CURRENT link, reduced to its decision and ordered so two snapshots compare as sequences.
///
/// # Why only current links
///
/// `current_subject` is NULL on a superseded row by design, so two superseded versions of one
/// placement carry EQUAL sort keys and the order between them is InnoDB's accident — measured at
/// this story's validation. Restricting to current rows makes `(observation_id, current_subject)` a
/// TOTAL order, because `identity_link_one_current` makes that pair unique exactly there. Nothing
/// in the purge-and-replay supersedes anything, so nothing is lost; story 5.11 is the one that will
/// supersede, and it should not inherit an order that is decorative over history.
///
/// # Why the instants come back as strings
///
/// `sqlx` is built here without its `chrono` feature, so a `DATETIME(6)` has no Rust type to decode
/// into — the same reason [`load_link_valid_to`] renders with `CAST(… AS CHAR)`. Two snapshots are
/// compared against EACH OTHER, never against a domain value, so this is transport and not the
/// comparison D10 forbids. ⚠️ No `Timestamp` is ever produced, which is why the registered
/// *"first story that needs to read an instant back as a value"* entry stays unmet.
///
/// # Errors
///
/// Returns the `sqlx::Error` as it came; callers classify it with [`classify`].
pub async fn snapshot_links<'e, E>(executor: E) -> Result<Vec<LinkSnapshot>, sqlx::Error>
where
    E: Executor<'e, Database = MySql>,
{
    type SnapshotRow = (
        String,
        Option<String>,
        String,
        Option<String>,
        Option<String>,
        String,
        u32,
        String,
        String,
        String,
    );
    let rows: Vec<SnapshotRow> = sqlx::query_as(
        "SELECT observation_id, interface_id, outcome, rule_id, abstention_cause, evidence, \
                ruleset_version, decided_by, CAST(valid_from AS CHAR), CAST(valid_to AS CHAR) \
         FROM identity_link WHERE current_subject IS NOT NULL \
         ORDER BY observation_id, current_subject",
    )
    .fetch_all(executor)
    .await?;
    rows.into_iter()
        .map(|r| {
            Ok(LinkSnapshot {
                observation_id: r.0,
                interface_id: r.1,
                outcome: r.2,
                rule_id: r.3,
                abstention_cause: r.4,
                evidence: serde_json::from_str(&r.5)
                    .map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                ruleset_version: r.6,
                decided_by: r.7,
                valid_from: r.8,
                valid_to: r.9,
            })
        })
        .collect()
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

    /// Connect, migrate and empty `declared_attribute`. `None` when `DATABASE_URL` is unset — and
    /// then the caller returns, which is why §7 of story 5.12 insists a green suite says nothing.
    async fn declared_fixture() -> Option<MySqlPool> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping declared-authorship test: DATABASE_URL unset");
            return None;
        };
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        sqlx::query("DELETE FROM declared_attribute")
            .execute(&pool)
            .await
            .expect("clean");
        Some(pool)
    }

    /// 🔴 The ONE raw write to `declared_attribute` outside the adapter, and the second of the
    /// authorship gate's sanctioned sites.
    ///
    /// It exists because the adapter **cannot** produce the input this story must test: `'operator'`
    /// is a LITERAL in its SQL, so a test written through `insert_declared_attribute` measures
    /// nothing about the CHECK — story 5.9's M3, a fourth time in this project.
    ///
    /// ⚠️ Its NAME is load-bearing: `xtask`'s `declared-authorship` gate allowlists exactly this
    /// identifier. Renaming it turns the gate red, which is the intended coupling — the exemption is
    /// one named site, not a blanket `#[cfg(test)]` hole (measured at story 5.12's validation to
    /// hide a planted write).
    async fn raw_declared_write_for_ddl_test(
        pool: &MySqlPool,
        entity_id: &str,
        actor_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO declared_attribute \
             (entity_id, attr_key, attr_value, origin, actor_id, updated_at) \
             VALUES (?, 'hostname', 'nas', 'manual', ?, NOW(6))",
        )
        .bind(entity_id)
        .bind(actor_id)
        .execute(pool)
        .await
        .map(|_| ())
    }

    /// 🔴 AC2 — the DDL CHECK bites, measured through RAW SQL because the adapter cannot reach it.
    ///
    /// Assertion-carried on purpose: `.expect_err()` would make this a PANIC, and story 5.11b's
    /// review caught exactly that mislabelling. The carrier is recorded in the story's T5 table.
    #[tokio::test]
    async fn a_scanner_authored_declared_write_is_refused_by_the_database() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = declared_fixture().await else {
            return;
        };

        let refused = raw_declared_write_for_ddl_test(
            &pool,
            "00000000-0000-0000-0000-0000000000f1",
            "scanner",
        )
        .await;
        assert!(
            refused.is_err(),
            "`declared_actor_not_scanner` must refuse it; got {refused:?}"
        );
        assert!(
            matches!(
                refused.map_err(classify),
                Err(RepositoryError::Constraint(_))
            ),
            "and it must classify as a constraint violation, not an opaque backend error"
        );
        assert_eq!(
            count_declared_attributes(&pool).await.expect("count"),
            0,
            "nothing was written"
        );

        // ⚠️ CHAR(36) pads, so the comparison is on the PADDED value: a trailing space is refused
        // too. Measured at validation — the CHECK bans one padded VALUE, not one byte string.
        let padded = raw_declared_write_for_ddl_test(
            &pool,
            "00000000-0000-0000-0000-0000000000f2",
            "scanner ",
        )
        .await;
        assert!(
            padded.is_err(),
            "CHAR padding makes 'scanner ' the same value"
        );
    }

    /// 🔑 AC2's second half — what the CHECK does NOT hold, pinned as an HONEST LIMIT.
    ///
    /// `CHECK (actor_id <> 'scanner')` bans one value, not a property. `'engine'` is a perfectly
    /// good name for a non-human author and the database accepts it. **This is not a defect and must
    /// not be "fixed" here**: an allowlist in DDL is a migration every time an actor is added, and
    /// there is no `actor` table (Epic 6). The real guard is the `declared-authorship` gate, which
    /// stops the write from ever being authored. This test says out loud what the tripwire is worth.
    #[tokio::test]
    async fn a_non_human_author_other_than_scanner_is_accepted_and_that_is_the_limit() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = declared_fixture().await else {
            return;
        };

        let accepted = raw_declared_write_for_ddl_test(
            &pool,
            "00000000-0000-0000-0000-0000000000f3",
            "engine",
        )
        .await;
        assert!(
            accepted.is_ok(),
            "the CHECK bans ONE value, not a property — got {accepted:?}"
        );
        assert_eq!(
            count_declared_attributes(&pool).await.expect("count"),
            1,
            "🔑 the row IS there: the database's guarantee is one spelling, and the gate is what \
             holds the property"
        );
    }

    /// 🔑 Mechanism 5 — the adapter cannot OVERWRITE a declared value at all.
    ///
    /// The PK is `(entity_id, attr_key)` and there is no `ON DUPLICATE KEY UPDATE` anywhere in this
    /// file, so a second write to one field is `ERROR 1062`. For a story called *never overwrite*
    /// this is the strongest of the five mechanisms, and story 5.12's first draft omitted it.
    #[tokio::test]
    async fn the_adapter_cannot_overwrite_an_existing_declared_value() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = declared_fixture().await else {
            return;
        };
        let entity = "00000000-0000-0000-0000-0000000000f4";

        insert_declared_attribute(&pool, entity, "hostname", "nas")
            .await
            .expect("the first write is legal");
        let second = insert_declared_attribute(&pool, entity, "hostname", "stolen").await;

        assert!(
            second.is_err(),
            "never overwrite: a second write to one field is refused, not silently applied"
        );
        let (value,): (String,) = sqlx::query_as(
            "SELECT attr_value FROM declared_attribute WHERE entity_id = ? AND attr_key = 'hostname'",
        )
        .bind(entity)
        .fetch_one(&pool)
        .await
        .expect("read back");
        assert_eq!(value, "nas", "and the original value is untouched");
    }

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
        // ⚠️ Children before parents. `identity_link.observation_id` gained a foreign key in
        // `0003_resolver_guards.sql`, so deleting observations while a link still points at one
        // fails ERROR 1451. This is ORDER-DEPENDENT: it only bites once some earlier test has
        // committed a link, which is why it stayed invisible until story 5.9b fixed the twelve
        // tests the key reddened outright.
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

    /// Connect, migrate and empty the identity tables **and `observation_record`**.
    ///
    /// ⚠️ This doc said `observation_record` *"is NOT touched, and does not need to be"* until story
    /// 5.9b, because `identity_link.observation_id` carried no foreign key and these tests minted an
    /// `ObsId` without ever inserting an observation. `0003_resolver_guards.sql` adds that key —
    /// measured reddening **twelve** tests here — so every link now needs a real observation behind
    /// it, and the cleanup deletes children before parents.
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
        sqlx::query("DELETE FROM observation_record")
            .execute(&pool)
            .await
            .expect("clean observations");
        Some(pool)
    }

    /// Insert a minimal observation and return its id — the row a link's `observation_id` names.
    ///
    /// Story 5.9's tests minted an `ObsId` out of thin air, which `0003`'s foreign key now refuses.
    /// The facts are irrelevant to every test that calls this: what the FK constrains is that the
    /// row EXISTS, and a link's meaning comes from its rule and its evidence, not from the
    /// observation's contents.
    async fn an_observation(pool: &MySqlPool) -> ObsId {
        use opencmdb_core::observation::{ConnectorId, Observation, Scope, VantageId};

        let id = ObsId::from_uuid(uuid::Uuid::now_v7());
        let observation = Observation {
            obs_id: id,
            connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()),
            observed_at: at(1_700_000_000),
            scope: Scope {
                l2_domain: L2DomainId::from_uuid(uuid::Uuid::nil()),
                vantage: VantageId::from_uuid(uuid::Uuid::nil()),
            },
            facts: vec![],
            raw: None,
        };
        insert_observation(pool, &observation)
            .await
            .map_err(classify)
            .expect("insert observation");
        id
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
        let obs = an_observation(&pool).await;
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
        let obs = an_observation(&pool).await;
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
        let obs = an_observation(&pool).await;
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
        let obs = an_observation(&pool).await;
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
        let obs = an_observation(&pool).await;
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
    /// the `current_subject` nil sentinel both `interface_id`s are NULL, MariaDB holds NULLs distinct,
    /// and the constraint would be decorative for exactly the half FR16 exists to display.
    #[tokio::test]
    async fn a_second_current_abstention_for_one_observation_is_refused() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let obs = an_observation(&pool).await;
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
        let obs = an_observation(&pool).await;
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
        let obs = an_observation(&pool).await;

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
             (id, observation_id, interface_id, current_subject, outcome, rule_id, abstention_cause, \
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
             (id, observation_id, interface_id, current_subject, outcome, rule_id, abstention_cause, \
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
             (id, observation_id, interface_id, current_subject, outcome, rule_id, abstention_cause, \
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

    /// Every guard the DDL declares is REACHED by something — the review found four that were not.
    ///
    /// Each of these can only be violated by going around the adapter, which is what makes them a
    /// second line of defence; and each was measured droppable with the whole suite green before
    /// this test existed. Same family as M3.
    #[tokio::test]
    async fn every_ddl_guard_refuses_what_it_names() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let obs = an_observation(&pool).await;
        let iface = an_interface(&pool, l2, [0, 1, 2, 3, 4, 20]).await;
        let other = an_interface(&pool, l2, [0, 1, 2, 3, 4, 21]).await;

        // `identity_link_current_subject` — the sentinel drifted from what it stands for.
        assert_eq!(
            raw_link(
                &pool,
                obs,
                Some(iface),
                Some(&other.to_string()),
                "match",
                OPEN_END
            )
            .await,
            Some(RepositoryError::Constraint("check")),
            "current_subject must not name an interface the link does not place the observation on"
        );

        // `identity_link_outcome` — a token outside the closed set.
        assert_eq!(
            raw_link(
                &pool,
                obs,
                Some(iface),
                Some(&iface.to_string()),
                "linked",
                OPEN_END
            )
            .await,
            Some(RepositoryError::Constraint("check")),
            "outcome is match | no_match | abstained"
        );

        // `identity_link_current_subject`, currency half — a closed row still in the key.
        assert_eq!(
            raw_link(
                &pool,
                obs,
                Some(iface),
                Some(&iface.to_string()),
                "match",
                "2023-06-01 12:00:00.000000"
            )
            .await,
            Some(RepositoryError::Constraint("check")),
            "a superseded row leaves the uniqueness key"
        );

        // `identity_link_interval` — a version that ends before it begins.
        assert_eq!(
            raw_link(
                &pool,
                obs,
                Some(iface),
                None,
                "match",
                "2000-01-01 00:00:00.000000"
            )
            .await,
            Some(RepositoryError::Constraint("check")),
            "a version covers a half-open interval"
        );

        // `identity_link_interface_fk` — a link naming an interface that does not exist.
        let ghost = InterfaceId::from_uuid(uuid::Uuid::now_v7());
        assert_eq!(
            raw_link(
                &pool,
                obs,
                Some(ghost),
                Some(&ghost.to_string()),
                "match",
                OPEN_END
            )
            .await,
            Some(RepositoryError::Constraint("foreign_key")),
            "a link points at a real interface"
        );

        // `interface_id_not_nil` — an interface that would collide with ABSTAINED_SUBJECT.
        assert_eq!(
            insert_interface(
                &pool,
                InterfaceId::from_uuid(uuid::Uuid::nil()),
                l2,
                &MacAddr([0, 1, 2, 3, 4, 22]),
                at(1_700_000_000),
                at(1_700_000_100),
            )
            .await
            .map_err(classify)
            .err(),
            Some(RepositoryError::Constraint("check")),
            "the nil UUID is D21's NIL_INTERFACE and must not also be a real interface"
        );

        // `interface_seen_window` — a window that closes before it opens.
        assert_eq!(
            insert_interface(
                &pool,
                InterfaceId::from_uuid(uuid::Uuid::now_v7()),
                l2,
                &MacAddr([0, 1, 2, 3, 4, 23]),
                at(1_700_000_100),
                at(1_700_000_000),
            )
            .await
            .map_err(classify)
            .err(),
            Some(RepositoryError::Constraint("check")),
            "first_seen_at precedes last_seen_at"
        );

        // `link_candidate_link_fk` / `link_candidate_interface_fk` — candidates of nothing.
        assert_eq!(
            insert_link_candidate(
                &pool,
                LinkId::from_uuid(uuid::Uuid::now_v7()),
                iface,
                &[obs]
            )
            .await
            .map_err(classify)
            .err(),
            Some(RepositoryError::Constraint("foreign_key")),
            "a candidate hangs off a real link"
        );
    }

    /// AC7 — the three guards `0003_resolver_guards.sql` installs, each measured by a RAW insert
    /// that goes around the adapter.
    ///
    /// ⚠️ Around the adapter deliberately. Story 5.9's M3 is the lesson: dropping the
    /// rule-XOR-cause CHECK left all 378 tests green, because `insert_identity_link` derives the
    /// rule and the cause from one `match` and cannot emit an incoherent pair — the guard is only
    /// reachable by not using it. The same is true of all three here: `MacAddr`'s `Display` cannot
    /// produce an uppercase `mac_canon`, and the engine's two rule ids are non-empty constants.
    ///
    /// ⚠️ The FOREIGN KEY is not a CHECK. It is grouped with them because all three are the same
    /// story's guards, not because they are the same kind of constraint.
    #[tokio::test]
    async fn the_resolver_guards_refuse_what_they_name() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let obs = an_observation(&pool).await;
        let iface = an_interface(&pool, l2, [0, 1, 2, 3, 4, 30]).await;

        // `identity_link_observation_fk` — a link naming an observation that does not exist.
        let ghost_observation = ObsId::from_uuid(uuid::Uuid::now_v7());
        assert_eq!(
            raw_link(
                &pool,
                ghost_observation,
                Some(iface),
                Some(&iface.to_string()),
                "match",
                OPEN_END
            )
            .await,
            Some(RepositoryError::Constraint("foreign_key")),
            "a link points at an observation that exists — the evidence must be there to point at"
        );

        // `interface_mac_canon_lower` — the canonical form is the lowercase colon form.
        let uppercase = sqlx::query(
            "INSERT INTO interface (id, l2_domain, mac_canon, first_seen_at, last_seen_at) \
             VALUES (?, ?, '00:11:22:33:44:AA', ?, ?)",
        )
        .bind(uuid::Uuid::now_v7().to_string())
        .bind(l2.to_string())
        .bind(datetime_literal(at(1_700_000_000)))
        .bind(datetime_literal(at(1_700_000_100)))
        .execute(&pool)
        .await
        .map_err(classify)
        .err();
        assert_eq!(
            uppercase,
            Some(RepositoryError::Constraint("check")),
            "an uppercase mac_canon would create a second interface for one physical NIC, \
             invisibly, because the L1 index is deliberately non-unique"
        );

        // `identity_link_rule_id_not_empty` — '' satisfies rule-XOR-cause, and is not a name.
        let nameless = sqlx::query(
            "INSERT INTO identity_link \
             (id, observation_id, interface_id, current_subject, outcome, rule_id, \
              abstention_cause, evidence, ruleset_version, decided_by, valid_from, valid_to) \
             VALUES (?, ?, ?, ?, 'match', '', NULL, '[]', 1, 'ENGINE', ?, ?)",
        )
        .bind(uuid::Uuid::now_v7().to_string())
        .bind(obs.to_string())
        .bind(iface.to_string())
        .bind(iface.to_string())
        .bind(datetime_literal(at(1_700_000_000)))
        .bind(OPEN_END)
        .execute(&pool)
        .await
        .map_err(classify)
        .err();
        assert_eq!(
            nameless,
            Some(RepositoryError::Constraint("check")),
            "a decision names the rule that settled it, and the empty string is not a name"
        );
    }

    /// Write one link with raw SQL, going AROUND the adapter's derivations, and return the error.
    ///
    /// The adapter cannot produce most of the rows the DDL refuses — that is the point of the
    /// CHECKs — so reaching them needs this.
    async fn raw_link(
        pool: &MySqlPool,
        obs: ObsId,
        interface: Option<InterfaceId>,
        current_subject: Option<&str>,
        outcome: &str,
        valid_to: &str,
    ) -> Option<RepositoryError> {
        sqlx::query(
            "INSERT INTO identity_link \
             (id, observation_id, interface_id, current_subject, outcome, rule_id, \
              abstention_cause, evidence, ruleset_version, decided_by, valid_from, valid_to) \
             VALUES (?, ?, ?, ?, ?, 'l1-exact-mac', NULL, '[]', 1, 'ENGINE', ?, ?)",
        )
        .bind(uuid::Uuid::now_v7().to_string())
        .bind(obs.to_string())
        .bind(interface.map(|i| i.to_string()))
        .bind(current_subject.map(str::to_string))
        .bind(outcome)
        .bind(datetime_literal(at(1_700_000_000)))
        .bind(valid_to)
        .execute(pool)
        .await
        .map_err(classify)
        .err()
    }

    /// The three tokens no other test writes to the database: `OPERATOR`, `no_match` and
    /// `AbsenceOfProof`. Each is a string literal in two independent places; misspell either side
    /// and the suite stayed green while the first production write would have failed.
    #[tokio::test]
    async fn the_tokens_no_other_test_stores_round_trip() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let iface = an_interface(&pool, l2, [0, 1, 2, 3, 4, 30]).await;

        // OPERATOR — a human asserting a link.
        let obs_op = an_observation(&pool).await;
        insert_identity_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::now_v7()),
            obs_op,
            Some(iface),
            &a_match("l1-exact-mac"),
            &[obs_op],
            DecidedBy::Operator,
            at(1_700_000_000),
            open_end(),
        )
        .await
        .map_err(classify)
        .expect("an operator may assert a link");

        // no_match — a rule that FORBADE the pair. It names the interface it excluded, which is
        // what `identity_link_abstained_has_no_interface` requires of any non-abstention.
        let obs_no = an_observation(&pool).await;
        let no_match = Decision {
            conclusion: Conclusion::NoMatch {
                rule: RuleId("l1-distinct-mac".into()),
            },
            verdict_vector: vec![],
            ruleset_version: CURRENT_RULESET_VERSION,
        };
        insert_identity_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::now_v7()),
            obs_no,
            Some(iface),
            &no_match,
            &[obs_no],
            DecidedBy::Engine,
            at(1_700_000_000),
            open_end(),
        )
        .await
        .map_err(classify)
        .expect("a refusal names the rule that forbade the pair");

        // AbsenceOfProof — the abstention cause the well-formed path never exercised.
        let obs_ab = an_observation(&pool).await;
        insert_identity_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::now_v7()),
            obs_ab,
            None,
            &an_abstention(IdentityAbstentionCause::AbsenceOfProof),
            &[obs_ab],
            DecidedBy::Engine,
            at(1_700_000_000),
            open_end(),
        )
        .await
        .map_err(classify)
        .expect("absence of proof is an abstention like any other");

        for (obs, decided_by, outcome, rule, cause) in [
            (obs_op, "OPERATOR", "match", Some("l1-exact-mac"), None),
            (obs_no, "ENGINE", "no_match", Some("l1-distinct-mac"), None),
            (
                obs_ab,
                "ENGINE",
                "abstained",
                None,
                Some("absence_of_proof"),
            ),
        ] {
            let links = load_current_links_for_observation(&pool, obs)
                .await
                .map_err(classify)
                .expect("read back");
            assert_eq!(links.len(), 1);
            assert_eq!(links[0].decided_by, decided_by);
            assert_eq!(links[0].outcome, outcome);
            assert_eq!(links[0].rule_id.as_deref(), rule);
            assert_eq!(links[0].abstention_cause.as_deref(), cause);
        }
    }

    /// `close_identity_link`'s three refusals, each of which was measured HAPPENING before the
    /// guards existed: an unknown id returned `Ok(())`, a re-close rewrote history, and closing at
    /// the sentinel resurrected a superseded link as current.
    #[tokio::test]
    async fn closing_a_link_refuses_what_it_must() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let obs = an_observation(&pool).await;
        let iface = an_interface(&pool, l2, [0, 1, 2, 3, 4, 40]).await;
        let link = LinkId::from_uuid(uuid::Uuid::now_v7());
        write_link(
            &pool,
            link,
            obs,
            Some(iface),
            &a_match("l1-exact-mac"),
            open_end(),
        )
        .await
        .expect("the link");

        assert_eq!(
            close_identity_link(
                &pool,
                LinkId::from_uuid(uuid::Uuid::now_v7()),
                at(1_700_000_500)
            )
            .await,
            Err(RepositoryError::NotFound),
            "closing a link that does not exist is an error, not a silent success"
        );
        assert_eq!(
            close_identity_link(&pool, link, open_end()).await,
            Err(RepositoryError::Constraint("check")),
            "closing AT the sentinel would leave the link current while reporting success"
        );

        close_identity_link(&pool, link, at(1_700_000_500))
            .await
            .expect("the real close");
        assert_eq!(
            close_identity_link(&pool, link, at(1_600_000_000)).await,
            Err(RepositoryError::NotFound),
            "an already-closed row is not current, so its stamp cannot be rewritten"
        );
        assert_eq!(
            load_link_valid_to(&pool, link)
                .await
                .map_err(classify)
                .expect("read"),
            Some(datetime_literal(at(1_700_000_500))),
            "and the historical stamp is intact"
        );
    }

    /// Two versions of ONE placement closed at the SAME derived instant — the collision the review
    /// measured under the old key, where the second close was refused and the link silently stayed
    /// current. Every instant here is data-derived and never the clock, so a replay reproduces
    /// them: this is story 5.10's purge-and-replay, not an exotic path.
    #[tokio::test]
    async fn two_versions_may_be_closed_at_the_same_instant() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let obs = an_observation(&pool).await;
        let iface = an_interface(&pool, l2, [0, 1, 2, 3, 4, 50]).await;
        let closed_at = at(1_700_000_500);

        for _ in 0..2 {
            let link = LinkId::from_uuid(uuid::Uuid::now_v7());
            write_link(
                &pool,
                link,
                obs,
                Some(iface),
                &a_match("l1-exact-mac"),
                open_end(),
            )
            .await
            .expect("a version");
            close_identity_link(&pool, link, closed_at)
                .await
                .expect("closing it must not collide with the previous version's stamp");
        }

        assert_eq!(
            load_current_links_for_observation(&pool, obs)
                .await
                .map_err(classify)
                .expect("read")
                .len(),
            0,
            "both versions are closed, so nothing is current"
        );
    }

    /// Story 5.10's purge deletes engine links wholesale. With `RESTRICT` it failed ERROR 1451 the
    /// moment any engine link carried a candidate — i.e. the ambiguity case `link_candidate` exists
    /// for. `ON DELETE CASCADE` is what makes the purge possible; this measures it.
    #[tokio::test]
    async fn purging_engine_links_takes_their_candidates_with_them() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = identity_fixture().await else {
            return;
        };
        let l2 = L2DomainId::from_uuid(uuid::Uuid::nil());
        let obs = an_observation(&pool).await;
        let iface = an_interface(&pool, l2, [0, 1, 2, 3, 4, 60]).await;
        let link = LinkId::from_uuid(uuid::Uuid::now_v7());
        write_link(
            &pool,
            link,
            obs,
            None,
            &an_abstention(IdentityAbstentionCause::Ambiguous),
            open_end(),
        )
        .await
        .expect("an abstained link");
        insert_link_candidate(&pool, link, iface, &[obs])
            .await
            .map_err(classify)
            .expect("its candidate");

        purge_engine_links(&pool)
            .await
            .map_err(classify)
            .expect("purge the engine's links");

        assert_eq!(
            count_identity_links(&pool)
                .await
                .map_err(classify)
                .expect("count"),
            0
        );
        assert_eq!(
            load_link_candidates(&pool, link)
                .await
                .map_err(classify)
                .expect("candidates")
                .len(),
            0,
            "the candidates went with their link"
        );
    }

    /// The persisted tokens are pinned, every one of them. No database needed.
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

    /// 🔴 `datetime_literal` TRUNCATES below the microsecond, and this is where that is asserted.
    ///
    /// The register carried this as *"truncates below the microsecond in silence… **Nothing asserts
    /// it**"* since story 5.9's code review. Half of that reproach is a property of a pure function
    /// and needed no future story: it is closed here.
    ///
    /// ⚠️ **The other half is not.** Two distinct instants render identically, so a caller that
    /// compares an instant it HOLDS against one it STORED can be wrong. Nothing here does that yet.
    /// ⚠️ And `resolver`'s `the_stored_instants_are_the_derived_ones` cannot catch it either: it
    /// builds its expected value by passing the instant through **this very function**, so both
    /// sides are truncated identically — the same bilateral-oracle shape story 5.10's code review
    /// found in `snapshot_links`. **Owner of that half: story 5.11**, the first that holds two
    /// instants for one placement and must decide whether they are the same.
    #[test]
    fn datetime_literal_truncates_below_the_microsecond() {
        let precise =
            chrono::DateTime::from_timestamp(1_700_000_000, 123_456_789).expect("in range");
        let neighbour =
            chrono::DateTime::from_timestamp(1_700_000_000, 123_456_001).expect("in range");

        assert_eq!(datetime_literal(precise), "2023-11-14 22:13:20.123456");
        assert_ne!(precise, neighbour, "788 ns apart, and distinct in Rust");
        assert_eq!(
            datetime_literal(neighbour),
            datetime_literal(precise),
            "and INDISTINGUISHABLE once rendered — that is the truncation, asserted rather than \
             merely true"
        );
    }

    /// The two sentinels are what they claim to be, and `current_subject` is derived from one place.
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
        assert_eq!(
            current_subject_of(Some(iface), OPEN_END),
            Some(iface.to_string())
        );
        assert_eq!(
            current_subject_of(None, OPEN_END),
            Some(ABSTAINED_SUBJECT.to_string())
        );
        // Once the row is not current it leaves the uniqueness key entirely.
        assert_eq!(
            current_subject_of(Some(iface), "2023-01-01 00:00:00.000000"),
            None
        );
        assert_eq!(current_subject_of(None, "2023-01-01 00:00:00.000000"), None);
    }
}
