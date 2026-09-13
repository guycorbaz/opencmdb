//! The addressing plan's write routes — the operator's hands on the plan (story 14.2b).
//!
//! Story 14.2 drew the plan and could not fill it: `/ipam` reads the store, and nothing in the
//! product had ever written an `ip_subnet`, an `ip_range` or an `ip_address` outside a test. This
//! module is the producer, and **`POST /ipam/subnet` is the FIRST of the three** because the other
//! two need a subnet to hang from — with two routes an empty plan stays empty for ever (§1(a)).
//!
//! # What this file carries, and what it deliberately does not
//!
//! Epic 14's constraint (1): *the write cost is front-loaded, and the FIRST new route carries the
//! shared machinery*. Here that is the Origin check, a KEYED refusal body per status in both
//! locales, and the exhaustive mapping of [`RepositoryError`]. The second and third routes reuse
//! them, and a test asserts the reuse rather than a comment claiming it.
//!
//! 🔑 **The Origin check is SHARED and the refusal BODIES are not** (Guy, 2026-09-12). The check is
//! [`crate::write_guard::same_origin`], reached from here and from `document.rs`, with a tripwire
//! asserting the comparison exists in exactly one file: *a CSRF check is the one function that must
//! never be duplicated*. The sentences are local, because `document.malformed` and
//! `ipam.refusal.malformed_subnet` are different sentences about different gestures, and one body serving
//! two surfaces is the shape story 6.4 paid for.
//!
//! ⚠️ **Nothing here writes provenance, so the `authorship` gate owes this file no sanction.** The
//! plan is a SECOND declared register beside `declared_attribute` (Guy's arbitration (4) of
//! 2026-09-10) and the two never touch; `ip_subnet` has no `origin` column and no author. Said
//! rather than left to be inferred from the gate staying green.
//!
//! # The state holds a PORT and no pool, and that is a compile-time property
//!
//! [`IpamWriteState`] has no pool field, so a handler here cannot extract `State<MySqlPool>` —
//! story 6.1's M4 carrier, re-measured by two review layers on `document.rs` and measured again for
//! this module (AC2). The pool lives INSIDE [`StoreIpamWrite`], behind `Arc<dyn IpamWritePort>`.
//!
//! ⚠️ `/ipam` itself is on its own POOL-BEARING router (`ipam_page::router`) since story 14.2, so
//! the two must not be fused: merging these routes onto that one silently retires the guarantee.
//!
//! # No switch
//!
//! `OPENCMDB_DOCUMENT_ENABLED` guards an AUTHORSHIP hazard — a route that turns an observed value
//! into a declared one — which the plan does not have. These routes are always mounted, above
//! `auth_deny` like every other non-public path. ⚠️ A fresh install therefore gains a live write
//! surface with no opt-in, which the release notes owe a sentence.

use std::net::Ipv4Addr;
use std::sync::Arc;

use axum::extract::State;
use axum::extract::rejection::FormRejection;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodRouter, post};
use axum::{Form, Router};
use opencmdb_core::ipam::IpamError;
use opencmdb_core::repo::{BoxFuture, RepositoryError};
use serde::Deserialize;
use sqlx::MySqlPool;

use crate::ipam_repo::{self, Subnet};

/// The longest label the plan's three tables accept.
///
/// ⚠️ **It is refused HERE rather than left to the store, and the reason is not tidiness**: `label`
/// is `VARCHAR(120)` on all three tables, so a longer value gives the operator a raw driver
/// sentence (`1406: Data too long`) — and under a non-strict `sql_mode` it is worse than that, the
/// value being silently TRUNCATED instead. *A refusal the operator can read beats a truncation
/// nobody sees.*
///
/// 🔑 The unit is CHARACTERS, not bytes: MariaDB counts `VARCHAR(120)` in characters under
/// `utf8mb4`, so « Réseau des imprimantes » must be measured the way the column measures it.
const MAX_LABEL_CHARS: usize = 120;

/// A refusal the operator can read: the status the rule earns, and the KEY that names it.
///
/// 🔑 **It is DATA and not a `Response`**, for two reasons that turned out to be the same one. The
/// mappers below are then pure, so a test can assert WHICH rule a refusal names rather than
/// scraping a rendered body — which is what AC4's SET test needs. And clippy's
/// `result_large_err` refuses a `Result<_, Response>` outright (measured: *the `Err`-variant is at
/// least 128 bytes*), so the lint and the design pointed the same way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Refusal {
    /// The status this rule earns.
    status: StatusCode,
    /// The i18n key naming the rule, never a sentence — story 6.4's finding: a body built with
    /// `format!` ships in English under a French UI.
    key: &'static str,
}

impl Refusal {
    /// A refusal at `status`, naming `key`.
    const fn new(status: StatusCode, key: &'static str) -> Self {
        Self { status, key }
    }

    /// The status this refusal answers with.
    pub(crate) const fn status(self) -> StatusCode {
        self.status
    }

    /// The key naming the rule that was broken.
    pub(crate) const fn key(self) -> &'static str {
        self.key
    }
}

impl IntoResponse for Refusal {
    fn into_response(self) -> Response {
        (self.status(), rust_i18n::t!(self.key()).to_string()).into_response()
    }
}

/// How long a write may hold the browser before the product answers instead of waiting.
///
/// 🔴 **This story adds the product's FIRST BLOCKING WRITE, and the loser's wait tracks the
/// holder's**: measured at the validation, a 3 s hold gave a **6.005 s** round trip, bounded only
/// by `innodb_lock_wait_timeout` — **50 s** on a stock container. ⚠️ Story 6b.10 put a per-handler
/// budget on the GET screens after measuring a 30 s hang; every budgeted file in this product is a
/// screen, and `document.rs` carries **zero**. This is where a write can do it.
///
/// 🔑 **The budget wraps the whole TRANSACTION and not its first read** — story 14.2's denial of
/// service was exactly a budget around the wrong half, `store_within` wrapping the reads while the
/// derivation and the render ran unbounded after them.
///
/// ⚠️ Five seconds, the same figure as [`crate::page::PAGE_STORE_BUDGET`], and the reason is that
/// there is no measurement that would justify a different one. *A number invented to look
/// considered is a number with nothing behind it.*
const IPAM_WRITE_BUDGET: std::time::Duration = std::time::Duration::from_secs(5);

/// Run one write within [`IPAM_WRITE_BUDGET`], answering rather than holding the browser.
///
/// ⚠️ **A timeout is reported to the operator as [`RepositoryError::Contention`], deliberately.**
/// The two are different facts — *the store never answered* against *the store said deadlock* — and
/// they are the same ACTION: nothing was written, try again. The distinction that matters to
/// whoever is debugging is kept where it belongs, in the log.
///
/// 🔑 Dropping the future drops the transaction, so *nothing was written* is true and not a hope:
/// an uncommitted MariaDB transaction is rolled back when its connection is returned.
async fn within_budget<T>(
    work: impl std::future::Future<Output = Result<T, RepositoryError>>,
) -> Result<T, RepositoryError> {
    match tokio::time::timeout(IPAM_WRITE_BUDGET, work).await {
        Ok(result) => result,
        Err(_elapsed) => {
            tracing::error!(
                budget_ms = IPAM_WRITE_BUDGET.as_millis(),
                "the addressing plan's write did not finish within its budget — refusing rather \
                 than holding the browser"
            );
            Err(RepositoryError::Contention)
        }
    }
}

/// One write route of the addressing plan.
///
/// 🔑 **The router is BUILT by iterating [`WriteRoute::ALL`]**, so the list of paths and the set of
/// mounts cannot drift — and because each variant must answer [`WriteRoute::path`] and
/// [`WriteRoute::handler`] through an exhaustive `match`, adding a route without mounting it is an
/// `error[E0004]` rather than a route nobody registered. AC3's guard walks this same list, which is
/// what lets it assert both halves: 401 without a credential, and something OTHER than 404 with
/// one. ⚠️ Story 6b.2 measured why the positive half is needed — `auth_deny` layers the fallback,
/// so a misspelt path, an unmounted route and a typo all answer 401 exactly like a real route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WriteRoute {
    /// `POST /ipam/subnet` — define a subnet, the entry the other two hang from.
    Subnet,
    /// `POST /ipam/range` — define a stretch of it and say what it is MEANT for.
    Range,
    /// `POST /ipam/address` — define one address.
    Address,
}

impl WriteRoute {
    /// Every write route this sub-router carries.
    pub(crate) const ALL: &'static [WriteRoute] =
        &[WriteRoute::Subnet, WriteRoute::Range, WriteRoute::Address];

    /// Every path this sub-router carries — AC3's list, DERIVED from the variants.
    ///
    /// 🔴 **The criterion says `PATHS: &[&str]`, and a const of that shape was written first and
    /// then removed: clippy called it `never used`.** Nothing in production could read it — the
    /// router is built from the variants, because only a variant carries its handler — so the const
    /// existed for the guard alone, which is the compiler saying *this is a second spelling with no
    /// reader*. Pinning the two together with a test was the alternative; deriving them removes the
    /// question. 🔑 *The house rule keeps a redundancy a test pins; it does not ask for one where
    /// there need be none.* Same shape as `Screen::ALL`, `IpPolicy::ALL`, `FactKind::ALL`.
    ///
    /// ⚠️ **`#[cfg(test)]`, because it has exactly one reader and that reader is a guard.** Left in
    /// production it is `never used` under `-D warnings` — measured TWICE, first as a const and
    /// then as this function. `document::PATHS` is a const in production because ITS router really
    /// is built from it; here only a variant carries its handler. *Each list lives where its reader
    /// is, and neither pretends to a use it has not got.*
    #[cfg(test)]
    pub(crate) fn paths() -> Vec<&'static str> {
        Self::ALL.iter().map(|route| route.path()).collect()
    }

    /// The path the route is mounted at, and the path the guard probes.
    pub(crate) const fn path(self) -> &'static str {
        match self {
            WriteRoute::Subnet => "/ipam/subnet",
            WriteRoute::Range => "/ipam/range",
            WriteRoute::Address => "/ipam/address",
        }
    }

    /// The shape refusal this form earns — one sentence per form, naming ITS fields.
    const fn malformed(self) -> Refusal {
        Refusal::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            match self {
                WriteRoute::Subnet => "ipam.refusal.malformed_subnet",
                WriteRoute::Range => "ipam.refusal.malformed_range",
                WriteRoute::Address => "ipam.refusal.malformed_address",
            },
        )
    }

    /// The method router mounted at [`WriteRoute::path`].
    fn handler(self) -> MethodRouter<IpamWriteState> {
        match self {
            WriteRoute::Subnet => post(define_subnet),
            WriteRoute::Range => post(define_range),
            WriteRoute::Address => post(define_address),
        }
    }
}

/// The `POST /ipam/subnet` request: `application/x-www-form-urlencoded`, ⚠️ because that is what
/// the vendored htmx 2.0.4 posts (measured at story 6.2: form-values encoding, zero `fetch(`, no
/// `json-enc` extension).
#[derive(Debug, Deserialize)]
pub(crate) struct DefineSubnetRequest {
    /// The subnet in CIDR notation, as an operator writes it — `192.0.2.0/24`.
    ///
    /// ⚠️ **NOT the stored spelling.** The store holds `192.000.002.000` so that lexicographic
    /// order is numeric order (story 14.1's arbitration); that padding is an implementation of
    /// ordering and never something an operator should type or read.
    pub(crate) cidr: String,
    /// What the operator calls this subnet. Required, and bounded by [`MAX_LABEL_CHARS`].
    pub(crate) label: String,
}

/// The `POST /ipam/range` request.
#[derive(Debug, Deserialize)]
pub(crate) struct DefineRangeRequest {
    /// The subnet the range belongs to, by the id `/ipam`'s own selector carries.
    pub(crate) subnet_id: String,
    /// The first address of the range, as an operator writes it — `192.0.2.10`.
    pub(crate) first: String,
    /// The last address, inclusive.
    pub(crate) last: String,
    /// What the stretch is MEANT for, as one of the four binding tokens.
    pub(crate) policy: String,
    /// What the operator calls this range. Required, and bounded by [`MAX_LABEL_CHARS`].
    pub(crate) label: String,
}

/// The `POST /ipam/address` request.
#[derive(Debug, Deserialize)]
pub(crate) struct DefineAddressRequest {
    /// The subnet the address belongs to.
    pub(crate) subnet_id: String,
    /// The address itself, as an operator writes it.
    pub(crate) addr: String,
    /// What the operator calls it. Required, and bounded by [`MAX_LABEL_CHARS`].
    pub(crate) label: String,
}

/// The port: one gesture of the plan, whole. The sub-router's state reaches the world only
/// through this, and whatever it needs to do so lives INSIDE the impl.
pub(crate) trait IpamWritePort: Send + Sync {
    /// Define a subnet and answer with the id it was given.
    ///
    /// # Errors
    ///
    /// Whatever the adapter refuses, as a [`RepositoryError`] — the handler maps it exhaustively.
    fn define_subnet(
        &self,
        subnet: Subnet,
        label: String,
    ) -> BoxFuture<'_, Result<String, RepositoryError>>;

    /// Define a range inside a subnet and answer with the id it was given.
    ///
    /// # Errors
    ///
    /// Whatever the adapter refuses, as a [`RepositoryError`] — an unknown subnet, an overlap, a
    /// range outside its subnet, or the loser of a lock wait.
    fn define_range(
        &self,
        subnet_id: String,
        first: Ipv4Addr,
        last: Ipv4Addr,
        policy: opencmdb_core::ipam::IpPolicy,
        label: String,
    ) -> BoxFuture<'_, Result<String, RepositoryError>>;

    /// Define one address inside a subnet and answer with the id it was given.
    ///
    /// # Errors
    ///
    /// Whatever the adapter refuses — an unknown subnet, an address outside it, or the same
    /// address defined twice.
    fn define_address(
        &self,
        subnet_id: String,
        addr: Ipv4Addr,
        label: String,
    ) -> BoxFuture<'_, Result<String, RepositoryError>>;
}

/// The production wiring: the plan's gestures over a MariaDB pool, which lives HERE and not on
/// [`IpamWriteState`].
pub(crate) struct StoreIpamWrite {
    /// The pool, unreachable from any handler by type.
    pool: MySqlPool,
}

impl StoreIpamWrite {
    /// Wire the plan's gestures to a pool.
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

impl IpamWritePort for StoreIpamWrite {
    fn define_subnet(
        &self,
        subnet: Subnet,
        label: String,
    ) -> BoxFuture<'_, Result<String, RepositoryError>> {
        Box::pin(async move {
            // 🔑 THE SERVER MINTS THE ID, v7 (Guy, 2026-09-12) — `document.rs:120`'s half of the
            // same decision. The other half, refusing an ARRIVING nil, belongs to the two routes
            // that receive a `subnet_id` from the browser; this one receives no id at all.
            let id = uuid::Uuid::now_v7().to_string();
            let mut conn = self.pool.acquire().await.map_err(crate::repo::classify)?;
            // ⚠️ ONE statement, so no transaction: `insert_subnet` reads nothing and decides
            // nothing against another row. The range route is where a transaction and a lock
            // become load-bearing (§1(d)), and wrapping this one would suggest a guarantee it does
            // not need and does not have.
            ipam_repo::insert_subnet(&mut *conn, &id, subnet, &label).await?;
            Ok(id)
        })
    }

    fn define_range(
        &self,
        subnet_id: String,
        first: Ipv4Addr,
        last: Ipv4Addr,
        policy: opencmdb_core::ipam::IpPolicy,
        label: String,
    ) -> BoxFuture<'_, Result<String, RepositoryError>> {
        Box::pin(async move {
            let id = uuid::Uuid::now_v7().to_string();
            let mut conn = self.pool.acquire().await.map_err(crate::repo::classify)?;
            // ⚠️ THE TRANSACTION AND THE TWO LOCKS ARE `insert_range`'s OWN, deliberately: the
            // overlap rule is decided by a read, so the lock has to be taken by whatever performs
            // that read. Opening a transaction here as well would nest one inside another, and
            // MariaDB's `BEGIN` implicitly commits the outer.
            ipam_repo::insert_range(&mut conn, &id, &subnet_id, first, last, policy, &label)
                .await?;
            Ok(id)
        })
    }

    fn define_address(
        &self,
        subnet_id: String,
        addr: Ipv4Addr,
        label: String,
    ) -> BoxFuture<'_, Result<String, RepositoryError>> {
        Box::pin(async move {
            let id = uuid::Uuid::now_v7().to_string();
            let mut conn = self.pool.acquire().await.map_err(crate::repo::classify)?;
            // 🔑 NO LOCK HERE, and the asymmetry with the range is not an oversight: an address is
            // refused twice by `ip_address_in_subnet`, a UNIQUE key, which needs no read to decide.
            // The range's rule compares a row against its SIBLINGS, which a `CHECK` cannot express
            // (`ERROR 1901`) and only a read can answer — *the lock is owed by the rule that reads,
            // not by the act of writing*.
            ipam_repo::insert_address(&mut conn, &id, &subnet_id, addr, &label).await?;
            Ok(id)
        })
    }
}

/// The sub-router's state — deliberately NO pool field. See the module doc.
#[derive(Clone)]
pub(crate) struct IpamWriteState {
    /// The plan's gestures. A port, never a pool.
    port: Arc<dyn IpamWritePort>,
}

/// The production sub-router, wired to the store-backed port.
pub(crate) fn router(pool: MySqlPool) -> Router {
    router_with(Arc::new(StoreIpamWrite::new(pool)))
}

/// The sub-router over an explicit port — the seam tests use to drive a gesture without a
/// database.
pub(crate) fn router_with(port: Arc<dyn IpamWritePort>) -> Router {
    let mut router = Router::new();
    // Built from the variants and not from `PATHS`, because only the variant carries its handler —
    // and the two lists are pinned equal by a test.
    for route in WriteRoute::ALL {
        router = router.route(route.path(), route.handler());
    }
    router.with_state(IpamWriteState { port })
}

/// `POST /ipam/subnet` — define a subnet.
///
/// The order of the refusals is the order of the decisions, and the first one is the CSRF check
/// (story 6.2 §5): the 403 wins over every other refusal. ⚠️ It cannot literally run *before* the
/// parser — `form` is an axum extractor, so the body is parsed before this body executes — and what
/// holds is that no refusal path consults the parsed form before the Origin has been compared.
async fn define_subnet(
    State(state): State<IpamWriteState>,
    headers: HeaderMap,
    form: Result<Form<DefineSubnetRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return (StatusCode::FORBIDDEN, crate::write_guard::CSRF_REFUSED_BODY).into_response();
    }
    let Ok(Form(request)) = form else {
        return WriteRoute::Subnet.malformed().into_response();
    };
    // 🔑 THE CIDR IS READ BEFORE THE LABEL, and the order is a decision rather than an accident.
    // A form wrong in two places can only be told about one of them, and the honest one to name is
    // the FIRST field the operator filled — the form is `cidr` then `label`, and a sentence about
    // the second while the first is unusable reads as though the first had been accepted.
    // ⚠️ T2 shipped the opposite order with nothing asserting either, which is what made this a
    // decision to take rather than a preference to keep.
    let subnet = match parse_cidr(&request.cidr) {
        Ok(subnet) => subnet,
        Err(refusal) => return refusal.into_response(),
    };
    let label = match checked_label(&request.label) {
        Ok(label) => label,
        Err(refusal) => return refusal.into_response(),
    };
    let work = state.port.define_subnet(subnet, label);
    answer(within_budget(work).await, None, "ipam.done.subnet")
}

/// `POST /ipam/range` — define a stretch of a subnet and say what it is MEANT for.
///
/// 🔑 **It reuses, it does not re-implement.** The Origin check, the label rules, the subnet-id
/// refusal, the exhaustive `RepositoryError` mapping and the budget are the same functions the
/// subnet route calls, and `every_route_reuses_the_shared_machinery` drives all three through the
/// same probes rather than trusting this sentence.
async fn define_range(
    State(state): State<IpamWriteState>,
    headers: HeaderMap,
    form: Result<Form<DefineRangeRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return (StatusCode::FORBIDDEN, crate::write_guard::CSRF_REFUSED_BODY).into_response();
    }
    let route = WriteRoute::Range;
    let Ok(Form(request)) = form else {
        return route.malformed().into_response();
    };
    let subnet_id = match checked_subnet_id(&request.subnet_id, route) {
        Ok(id) => id,
        Err(refusal) => return refusal.into_response(),
    };
    let (first, last) = match (
        request.first.trim().parse::<Ipv4Addr>(),
        request.last.trim().parse::<Ipv4Addr>(),
    ) {
        (Ok(first), Ok(last)) => (first, last),
        _ => return route.malformed().into_response(),
    };
    let policy = match parse_policy(&request.policy) {
        Ok(policy) => policy,
        Err(refusal) => return refusal.into_response(),
    };
    let label = match checked_label(&request.label) {
        Ok(label) => label,
        Err(refusal) => return refusal.into_response(),
    };
    let work = state
        .port
        .define_range(subnet_id.clone(), first, last, policy, label);
    answer(
        within_budget(work).await,
        Some(&subnet_id),
        "ipam.done.range",
    )
}

/// `POST /ipam/address` — define one address of a subnet.
async fn define_address(
    State(state): State<IpamWriteState>,
    headers: HeaderMap,
    form: Result<Form<DefineAddressRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return (StatusCode::FORBIDDEN, crate::write_guard::CSRF_REFUSED_BODY).into_response();
    }
    let route = WriteRoute::Address;
    let Ok(Form(request)) = form else {
        return route.malformed().into_response();
    };
    let subnet_id = match checked_subnet_id(&request.subnet_id, route) {
        Ok(id) => id,
        Err(refusal) => return refusal.into_response(),
    };
    let Ok(addr) = request.addr.trim().parse::<Ipv4Addr>() else {
        return route.malformed().into_response();
    };
    let label = match checked_label(&request.label) {
        Ok(label) => label,
        Err(refusal) => return refusal.into_response(),
    };
    let work = state.port.define_address(subnet_id.clone(), addr, label);
    answer(
        within_budget(work).await,
        Some(&subnet_id),
        "ipam.done.address",
    )
}

/// Turn a port's answer into the operator's, sending the browser back to the plan it just changed.
///
/// 🔑 `HX-Redirect` for the reason story 6.4 adopted it: a narrow swap would leave the screen
/// asserting the state it had before the write. The browser goes back to `/ipam` selected on the
/// subnet concerned, which re-renders from the store — *one URL per state*, epic constraint 4.
fn answer(
    outcome: Result<String, RepositoryError>,
    subnet_id: Option<&str>,
    done_key: &'static str,
) -> Response {
    match outcome {
        Ok(id) => {
            // `None` means the record just created IS the subnet, so it is its own redirect target.
            let target = subnet_id.unwrap_or(&id);
            tracing::info!(id = %id, subnet = %target, key = done_key, "the plan was changed");
            (
                StatusCode::CREATED,
                [(
                    axum::http::HeaderName::from_static("hx-redirect"),
                    format!("/ipam?subnet={target}"),
                )],
                rust_i18n::t!(done_key).to_string(),
            )
                .into_response()
        }
        Err(error) => {
            let refusal = repository_refusal(&error);
            if refusal.status() == StatusCode::INTERNAL_SERVER_ERROR {
                tracing::error!(%error, "the addressing plan's write failed at the backend");
            }
            refusal.into_response()
        }
    }
}

/// The subnet id the form carried, or the refusal it earns.
///
/// 🔑 **THE NIL UUID IS REFUSED HERE, AT THE ROUTE** (Guy, 2026-09-12, §2's decision 3) — the other
/// half of `document.rs`'s pair, whose `:120` mints a v7 for the record being created and whose
/// `:195` refuses a nil ARRIVING from the client. Both halves are live on this route: it creates a
/// range (id minted here) and it carries a subnet id chosen in the browser.
///
/// ⚠️ **A TRIPWIRE, not a barrier**: the adapter still accepts the nil, measured at story 14.2 on
/// all three `insert_*`. Story 5.12's precedent, stated rather than implied.
///
/// ⚠️ The nil is folded into the form's shape sentence rather than given a key of its own
/// (`document.rs:195`'s precedent): it is a sentinel no operator types, so a sentence for it would
/// be a sentence spent on a hand-crafted request.
///
/// # Errors
///
/// The form's shape refusal when the id is not a UUID, or is the nil sentinel.
fn checked_subnet_id(raw: &str, route: WriteRoute) -> Result<String, Refusal> {
    let Ok(parsed) = raw.trim().parse::<uuid::Uuid>() else {
        return Err(route.malformed());
    };
    if parsed.is_nil() {
        return Err(route.malformed());
    }
    // Re-serialised canonical, so a braced, urn: or hyphenless spelling of a real id is harmless
    // before any SQL sees it (story 6.2 §2).
    Ok(parsed.to_string())
}

/// The policy the operator chose, or the refusal that says it is not one of the four.
///
/// 🔴 **NO `trim()`, and the first draft had one — its own test caught it.** A label is free text
/// an operator typed, where a stray space is a typo worth absorbing; a policy is a TOKEN from a
/// closed set, chosen by a control, where a stray space means the sender is not the form. Trimming
/// it would make the route's acceptance set larger than the binding table's, which is the one
/// property that table exists to hold — and `ascii_bin` being PAD SPACE, `'static '` is exactly the
/// value the schema needed an INTEGER comparison to refuse one layer down. *The same reflex that is
/// kindness on free text is a widened vocabulary on a token.*
///
/// ⚠️ **Not `ipam_repo::policy_from_token`, and the difference is the POPULATION.** That one reads a
/// token the STORE holds, so an unknown one is a row this build cannot render — a fault, answered
/// as a backend failure. This one reads a token the BROWSER sent, where an unknown one is an
/// ordinary form mistake. *The same string means different things depending on who wrote it.*
///
/// # Errors
///
/// A 422 naming the axis rather than paraphrasing the four binding words.
fn parse_policy(raw: &str) -> Result<opencmdb_core::ipam::IpPolicy, Refusal> {
    opencmdb_core::ipam::IpPolicy::ALL
        .into_iter()
        .find(|policy| policy.as_str() == raw)
        .ok_or_else(|| {
            Refusal::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "ipam.refusal.unknown_policy",
            )
        })
}

/// The label the operator typed, or the refusal that names which rule it broke.
///
/// # Errors
///
/// A 422 keyed on the rule: empty (§2's decision 4 — a plan of numbers with no words is not a
/// plan) or longer than [`MAX_LABEL_CHARS`].
fn checked_label(raw: &str) -> Result<String, Refusal> {
    let label = raw.trim();
    if label.is_empty() {
        return Err(Refusal::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "ipam.refusal.label_empty",
        ));
    }
    if label.chars().count() > MAX_LABEL_CHARS {
        return Err(Refusal::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "ipam.refusal.label_too_long",
        ));
    }
    Ok(label.to_string())
}

/// Parse `192.0.2.0/24` into the subnet the arithmetic can answer for.
///
/// # Errors
///
/// A 422: the subnet form's own shape sentence when the text is not a CIDR at all, or the keyed
/// sentence of whichever [`IpamError`] [`Subnet::new`] raises.
fn parse_cidr(raw: &str) -> Result<Subnet, Refusal> {
    let malformed = || WriteRoute::Subnet.malformed();
    let (base, prefix) = raw.trim().split_once('/').ok_or_else(malformed)?;
    let base: Ipv4Addr = base.parse().map_err(|_| malformed())?;
    // ⚠️ A prefix that does not fit a `u8` — `/999` — is MALFORMED, while one that fits and cannot
    // belong to IPv4 — `/64` — gets `PrefixLengthNotInFamily`, which names the real rule. Two
    // sentences for what looks like one mistake, and the distinction is the honest one: the product
    // can only name a rule it can evaluate.
    let prefix: u8 = prefix.parse().map_err(|_| malformed())?;
    Subnet::new(base, prefix).map_err(|error| ipam_refusal(&error))
}

/// The sentence a named database constraint earns, or `None` for a name nobody has mapped.
///
/// 🔑 **The `Option` is the whole point, and it exists because an explicit 500 and a fallthrough
/// 500 are the same ANSWER and not the same STATEMENT.** [`RepositoryError::Constraint`] carries a
/// `&'static str`, so no `match` over it can be exhaustive and the compiler cannot help; what
/// carries AC4's second half is `every_constraint_name_the_store_can_produce_is_mapped`, which
/// reads the names `repo::classify` can emit and asserts each one is `Some` here. Fold `check` into
/// the `_` arm and the answer to the operator does not change by one byte — and that test reds,
/// which is the difference between a decision and an omission.
fn constraint_refusal(name: &str) -> Option<Refusal> {
    match name {
        // The likeliest refusal on this screen: the operator re-enters a subnet the plan already
        // holds. It rides `ip_subnet_cidr`, and NO pre-read is taken — a check that commits
        // separately from its write is a TOCTOU hole, not a check.
        "unique" => Some(Refusal::new(
            StatusCode::CONFLICT,
            "ipam.refusal.already_defined",
        )),
        // A `subnet_id` that names no row — reachable from the two routes that receive one.
        "foreign_key" => Some(Refusal::new(
            StatusCode::NOT_FOUND,
            "ipam.refusal.unknown_subnet",
        )),
        // 🔑 OURS, not the operator's, and mapped EXPLICITLY for that reason. Every `CHECK` on the
        // plan's three tables restates something the adapter already refuses — the canonical
        // spelling, the policy domain, the ordered bounds — so a `check` reaching here means a
        // value went round the adapter. That is a fault in this product, not a mistake the operator
        // can be told how to correct.
        "check" => Some(backend()),
        _ => None,
    }
}

/// Map a [`RepositoryError`] to the operator's sentence.
///
/// 🔑 **Exhaustive, with no `_` arm, and that is half of AC4's carrier**: a new VARIANT is an
/// `error[E0004]` here. ⚠️ The other half cannot be the compiler — [`RepositoryError::Constraint`]
/// carries a `&'static str`, which forces a `_` inside, so a new constraint NAME is invisible to
/// it. That half is a SET test over the names the adapter can produce.
pub(crate) fn repository_refusal(error: &RepositoryError) -> Refusal {
    match error {
        RepositoryError::Ipam(ipam) => ipam_refusal(ipam),
        // The three names `classify` can produce. ⚠️ The `_` is what the SET test covers.
        // ⚠️ THE COMPILER STOPS HERE. `Constraint` carries a `&'static str`, so the match INSIDE
        // it needs a `_` and a new constraint NAME is invisible to `E0004`. That half is carried by
        // [`constraint_refusal`] answering `None`, which the SET test reads.
        RepositoryError::Constraint(name) => constraint_refusal(name).unwrap_or_else(backend),
        RepositoryError::NotFound => {
            Refusal::new(StatusCode::NOT_FOUND, "ipam.refusal.unknown_subnet")
        }
        // 🔴 Reachable since this story: `classify`'s `Contention` arm was dead until T1 repaired
        // it, and the plan's routes are the first writes in this product that CONTEND. 503 rather
        // than 409, because nothing about the request is wrong — the honest answer is *try again*.
        RepositoryError::Contention => {
            Refusal::new(StatusCode::SERVICE_UNAVAILABLE, "ipam.refusal.contention")
        }
        // Neither can arise here: both belong to the identity engine's write path, which this
        // module never enters. They are mapped rather than swept into a `_`, because the `_` is
        // what would swallow a variant added tomorrow.
        RepositoryError::InstantRegressed | RepositoryError::ContradictoryObservation => backend(),
        RepositoryError::Backend(_) => backend(),
    }
}

/// Map an [`IpamError`] to the operator's sentence — exhaustive, no `_` arm.
pub(crate) fn ipam_refusal(error: &IpamError) -> Refusal {
    let key = match error {
        IpamError::RangeOutsideSubnet => "ipam.refusal.range_outside_subnet",
        IpamError::RangeOverlapsAnother => "ipam.refusal.range_overlaps",
        IpamError::RangeBoundsInverted => "ipam.refusal.range_bounds_inverted",
        IpamError::AddressOutsideSubnet => "ipam.refusal.address_outside_subnet",
        IpamError::BaseIsNotTheNetworkAddress => "ipam.refusal.base_not_network",
        IpamError::PrefixLengthNotInFamily => "ipam.refusal.prefix_not_in_family",
        IpamError::MalformedAddress => "ipam.refusal.not_an_address",
    };
    Refusal::new(StatusCode::UNPROCESSABLE_ENTITY, key)
}

/// A backend failure: answered with a sentence that leaks none of its cause. The cause itself is
/// logged by the caller, which is what keeps this mapper pure.
const fn backend() -> Refusal {
    Refusal::new(StatusCode::INTERNAL_SERVER_ERROR, "ipam.refusal.backend")
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, header};
    use tower::ServiceExt;

    use super::*;

    /// A port that answers whatever it was built with, and records what it was asked.
    struct FakePort {
        answer: std::sync::Mutex<Option<Result<String, RepositoryError>>>,
        asked: std::sync::Mutex<Vec<(String, String)>>,
    }

    impl FakePort {
        /// The one answer this port was built with. Panics on a second call, which is the point:
        /// a test that reaches the port twice is a test whose refusal did not stop the handler.
        fn take_answer(&self) -> Result<String, RepositoryError> {
            self.answer
                .lock()
                .expect("the fake port's answer")
                .take()
                .expect("the fake port answers once per test")
        }

        fn answering(answer: Result<String, RepositoryError>) -> Arc<Self> {
            Arc::new(Self {
                answer: std::sync::Mutex::new(Some(answer)),
                asked: std::sync::Mutex::new(Vec::new()),
            })
        }
    }

    impl IpamWritePort for FakePort {
        fn define_subnet(
            &self,
            subnet: Subnet,
            label: String,
        ) -> BoxFuture<'_, Result<String, RepositoryError>> {
            self.asked
                .lock()
                .expect("the fake port's log")
                .push((subnet.cidr(), label));
            let answer = self.take_answer();
            Box::pin(async move { answer })
        }

        fn define_range(
            &self,
            subnet_id: String,
            first: Ipv4Addr,
            last: Ipv4Addr,
            _policy: opencmdb_core::ipam::IpPolicy,
            label: String,
        ) -> BoxFuture<'_, Result<String, RepositoryError>> {
            self.asked
                .lock()
                .expect("the fake port's log")
                .push((format!("{subnet_id}:{first}-{last}"), label));
            let answer = self.take_answer();
            Box::pin(async move { answer })
        }

        fn define_address(
            &self,
            subnet_id: String,
            addr: Ipv4Addr,
            label: String,
        ) -> BoxFuture<'_, Result<String, RepositoryError>> {
            self.asked
                .lock()
                .expect("the fake port's log")
                .push((format!("{subnet_id}:{addr}"), label));
            let answer = self.take_answer();
            Box::pin(async move { answer })
        }
    }

    /// A same-origin urlencoded POST to the subnet route. `Origin`/`Host` agree, so the CSRF check
    /// passes; the test that probes the check sets the headers itself.
    fn form_post(body: &str) -> Request<Body> {
        form_post_to(WriteRoute::Subnet, body)
    }

    /// The same, to any of the three routes.
    fn form_post_to(route: WriteRoute, body: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(route.path())
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::HOST, "opencmdb.example")
            .header(header::ORIGIN, "https://opencmdb.example")
            .body(Body::from(body.to_string()))
            .expect("a well-formed test request")
    }

    /// Drive one request through the sub-router over a fake port, and answer with the status and
    /// the body an operator would read.
    async fn drive(port: Arc<dyn IpamWritePort>, request: Request<Body>) -> (StatusCode, String) {
        let response = router_with(port)
            .oneshot(request)
            .await
            .expect("the sub-router answers");
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("a readable body");
        (status, String::from_utf8_lossy(&body).into_owned())
    }

    /// The bodies are read from the resolver, never copied: pinning the literal here would recreate
    /// the two-spellings drift the keys exist to remove (story 6.4's finding, `document.rs`'s own
    /// note). What is asserted is WHICH key answered, and that is what makes a wrong mapping red.
    fn key(name: &str) -> String {
        rust_i18n::t!(name).to_string()
    }

    #[tokio::test]
    async fn a_cross_origin_post_is_refused_before_anything_else() {
        // 🔑 The port answers a SUCCESS. If the CSRF check did not win, this test would go green on
        // a 201 — which is exactly the ordering story 6.2 §5 buys and what this asserts.
        let port = FakePort::answering(Ok("id".to_string()));
        let request = Request::builder()
            .method("POST")
            .uri(WriteRoute::Subnet.path())
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::HOST, "opencmdb.example")
            .header(header::ORIGIN, "https://attacker.example")
            .body(Body::from("cidr=192.0.2.0/24&label=Office"))
            .expect("a well-formed test request");
        let (status, body) = drive(port.clone(), request).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "a cross-origin write is refused"
        );
        assert_eq!(body, crate::write_guard::CSRF_REFUSED_BODY);
        assert!(
            port.asked.lock().expect("the log").is_empty(),
            "the port must not have been reached at all"
        );
    }

    #[tokio::test]
    async fn a_well_formed_definition_is_created_and_sends_the_browser_to_the_plan() {
        let port = FakePort::answering(Ok("01900000-0000-7000-8000-000000000001".to_string()));
        let response = router_with(port.clone())
            .oneshot(form_post("cidr=192.0.2.0/24&label=Office"))
            .await
            .expect("the sub-router answers");
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            response
                .headers()
                .get("hx-redirect")
                .expect("the browser is sent back to the plan")
                .to_str()
                .expect("an ASCII header"),
            "/ipam?subnet=01900000-0000-7000-8000-000000000001",
            "the redirect selects the subnet just defined — one URL per state"
        );
        assert_eq!(
            port.asked.lock().expect("the log").as_slice(),
            [("192.0.2.0/24".to_string(), "Office".to_string())],
            "the CIDR is parsed and the label is trimmed before the store sees either"
        );
    }

    #[tokio::test]
    async fn a_body_that_is_not_the_form_is_refused_as_malformed() {
        let port = FakePort::answering(Ok("unreached".to_string()));
        let (status, body) = drive(port, form_post("subject=nothing-like-it")).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body, key("ipam.refusal.malformed_subnet"));
    }

    #[tokio::test]
    async fn an_empty_label_is_refused_by_its_own_rule() {
        // ⚠️ WHITESPACE, not the empty string: `label=` and `label=%20%20` are the same mistake,
        // and only the trimmed form catches the second.
        let port = FakePort::answering(Ok("unreached".to_string()));
        let (status, body) = drive(port, form_post("cidr=192.0.2.0/24&label=%20%20")).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body, key("ipam.refusal.label_empty"));
    }

    #[tokio::test]
    async fn a_label_over_the_column_is_refused_here_rather_than_by_the_driver() {
        let port = FakePort::answering(Ok("unreached".to_string()));
        let long = "e".repeat(MAX_LABEL_CHARS + 1);
        let (status, body) =
            drive(port, form_post(&format!("cidr=192.0.2.0/24&label={long}"))).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body, key("ipam.refusal.label_too_long"));
    }

    #[tokio::test]
    async fn a_label_at_the_column_in_accented_characters_is_accepted() {
        // 🔑 THE UNIT IS CHARACTERS, NOT BYTES. 120 accented characters are 240 bytes, and a
        // byte-counting bound would refuse a label the column accepts — which is the same class as
        // measuring a column in the wrong unit, one layer up.
        let port = FakePort::answering(Ok("id".to_string()));
        let accented = "é".repeat(MAX_LABEL_CHARS);
        let (status, _) = drive(
            port,
            form_post(&format!("cidr=192.0.2.0/24&label={accented}")),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::CREATED,
            "120 characters is 240 bytes and still 120 characters"
        );
    }

    #[tokio::test]
    async fn text_that_is_not_a_cidr_is_refused_as_malformed() {
        for spelling in [
            "192.0.2.0",
            "not-an-address/24",
            "192.0.2.0/",
            "192.0.2.0/999",
        ] {
            let port = FakePort::answering(Ok("unreached".to_string()));
            let (status, body) =
                drive(port, form_post(&format!("cidr={spelling}&label=Office"))).await;
            assert_eq!(
                status,
                StatusCode::UNPROCESSABLE_ENTITY,
                "`{spelling}` is not a subnet the product can evaluate"
            );
            assert_eq!(
                body,
                key("ipam.refusal.malformed_subnet"),
                "for `{spelling}`"
            );
        }
    }

    #[tokio::test]
    async fn a_prefix_outside_the_family_names_that_rule_and_not_malformed() {
        // 🔴 The distinction is deliberate and is the honest one: `/999` does not fit a `u8` and is
        // MALFORMED, `/64` fits and cannot belong to IPv4. The product names only a rule it can
        // evaluate. ⚠️ And this one is a REFUSAL rather than a lint because `Subnet::last` would
        // compute `32 - 64` and panic on subtract-with-overflow (story 14.1's measurement).
        let port = FakePort::answering(Ok("unreached".to_string()));
        let (status, body) = drive(port, form_post("cidr=192.0.2.0/64&label=Office")).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body, key("ipam.refusal.prefix_not_in_family"));
    }

    #[tokio::test]
    async fn a_base_carrying_host_bits_is_refused_and_the_sentence_names_the_subnet_meant() {
        let port = FakePort::answering(Ok("unreached".to_string()));
        let (status, body) = drive(port, form_post("cidr=192.0.2.5/24&label=Office")).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body, key("ipam.refusal.base_not_network"));
    }

    #[tokio::test]
    async fn a_subnet_the_plan_already_holds_is_a_conflict() {
        let port = FakePort::answering(Err(RepositoryError::Constraint("unique")));
        let (status, body) = drive(port, form_post("cidr=192.0.2.0/24&label=Office")).await;
        assert_eq!(
            status,
            StatusCode::CONFLICT,
            "the likeliest refusal on this screen: the operator re-enters what is already there"
        );
        assert_eq!(body, key("ipam.refusal.already_defined"));
    }

    #[tokio::test]
    async fn contention_is_transient_and_says_so() {
        // 🔴 Unreachable before this story: `classify`'s `Contention` arm was dead code until T1
        // repaired it, because nothing in this product contended.
        let port = FakePort::answering(Err(RepositoryError::Contention));
        let (status, body) = drive(port, form_post("cidr=192.0.2.0/24&label=Office")).await;
        assert_eq!(
            status,
            StatusCode::SERVICE_UNAVAILABLE,
            "nothing about the request is wrong, so it is not a 4xx"
        );
        assert_eq!(body, key("ipam.refusal.contention"));
    }

    #[tokio::test]
    async fn a_backend_failure_leaks_none_of_its_cause() {
        let port = FakePort::answering(Err(RepositoryError::Backend(
            "Access denied for user 'root'@'10.0.0.7' (using password: YES)".to_string(),
        )));
        let (status, body) = drive(port, form_post("cidr=192.0.2.0/24&label=Office")).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body, key("ipam.refusal.backend"));
        assert!(
            !body.contains("root") && !body.contains("password"),
            "the driver's sentence stays in the log: {body}"
        );
    }

    /// Every refusal the plan's domain can raise renders a DISTINCT sentence.
    ///
    /// 🔴 Story 14.1's review measured why this is a property over `IpamError::ALL` and not a list
    /// written here: its predecessor hand-wrote a six-element array in its own body, so a seventh
    /// variant returning a VERBATIM duplicate left the whole suite green.
    #[test]
    fn every_domain_refusal_has_its_own_sentence_in_both_locales() {
        for locale in ["en", "fr"] {
            let mut seen: Vec<String> = Vec::new();
            for error in IpamError::ALL {
                let refusal = ipam_refusal(&error);
                assert_eq!(
                    refusal.status(),
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "{error:?} is the operator's mistake, not the server's"
                );
                // 🔑 The key comes from the MAPPER, never from a copy of it written here. A test
                // that restates the mapping measures the copy: story 6.5's M8 — *a count is not a
                // set* — one degree over, where a second table would agree with itself.
                let sentence = rust_i18n::t!(refusal.key(), locale = locale).to_string();
                assert_ne!(
                    sentence,
                    refusal.key(),
                    "{error:?} has no `{locale}` translation, so the page renders its key name"
                );
                assert!(
                    !seen.contains(&sentence),
                    "two refusals read the same in `{locale}`, so the operator cannot tell them \
                     apart: {sentence}"
                );
                seen.push(sentence);
            }
        }
    }

    #[tokio::test]
    async fn the_cidr_is_read_before_the_label() {
        // A form wrong in BOTH places can only be told about one of them. Nothing pinned the order
        // when T2 shipped, so either answer would have passed; this is the decision, asserted.
        let port = FakePort::answering(Ok("unreached".to_string()));
        let (status, body) = drive(port, form_post("cidr=nonsense&label=")).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            key("ipam.refusal.malformed_subnet"),
            "the first field the operator filled is the one the refusal names"
        );
    }

    /// Every constraint NAME `repo::classify` can produce is mapped here by name.
    ///
    /// 🔑 **This is AC4's second carrier, and it exists because the compiler cannot be the first
    /// one twice.** An exhaustive `match` over `RepositoryError` makes a new VARIANT an `E0004`;
    /// [`RepositoryError::Constraint`] carries a `&'static str`, so a new NAME is invisible to it.
    /// Add `Constraint("range_overlap")` to `classify` and this test reds while every other test,
    /// clippy and all ten gates stay green.
    ///
    /// ⚠️ **It reads the WHOLE of `repo.rs`, test module included, deliberately.** Story 14.2's
    /// guard split on the first `#[cfg(test)]` and thereby read 382 bytes of 47 324; here the test
    /// module's own literals are the same three names, so including them costs nothing and the trap
    /// is not reachable at all. *A perimeter you do not have to compute is a perimeter that cannot
    /// be computed wrongly.*
    #[test]
    fn every_constraint_name_the_store_can_produce_is_mapped() {
        const NEEDLE: &str = "RepositoryError::Constraint(\"";
        let source = include_str!("repo.rs");
        let mut names: Vec<&str> = Vec::new();
        let mut rest = source;
        while let Some(at) = rest.find(NEEDLE) {
            let after = &rest[at + NEEDLE.len()..];
            let end = after.find('"').expect("a closed string literal");
            let name = &after[..end];
            if !names.contains(&name) {
                names.push(name);
            }
            rest = &after[end..];
        }
        // 🔴 THE MAPPING IS ASSERTED FIRST, AND THE ORDER IS THE FINDING. The floor came first in
        // the draft, so a new constraint name reddened on `left: 4, right: 3` — a count — and the
        // assertion written for the defect was never reached. Story 5.13 met the same shape and
        // this project has now met it five times: *an assertion that fires first decides what the
        // failure says*, and a count says nothing a reader can act on.
        for name in &names {
            assert!(
                constraint_refusal(name).is_some(),
                "`Constraint(\"{name}\")` reaches the handler and nobody has said what it means \
                 to the operator — map it by name, even when the answer is the backend sentence"
            );
        }
        // A floor equal to what is there, never under it (story 6b.7). It catches the other
        // direction: a name that DISAPPEARS, which leaves a mapping nothing can reach.
        assert_eq!(
            names.len(),
            3,
            "the names `classify` can produce changed: {names:?}"
        );
    }

    /// Every key this module can render resolves in BOTH locales.
    ///
    /// ⚠️ **A source scan reads the file's prose as well as its code** (story 14.2's finding, where
    /// a guard was satisfied by the comment narrating the defect). Here that is harmless and even
    /// useful: the needle is a double-quoted literal, so a key named in a doc comment inside
    /// backticks is not matched, and a key named in a literal that does NOT exist is a finding
    /// whichever line it sits on.
    #[test]
    fn every_key_this_module_can_render_resolves_in_both_locales() {
        let source = include_str!("ipam_write.rs");
        // 🔴 THE NEEDLE IS BUILT AT RUNTIME, and the first version was not — it found ITSELF, then
        // found the comment saying so. A scan of its own source matches the literal spelling the
        // scan is written with, so the needle came back as a key named after the prefix alone and
        // the test reddened on a key nobody wrote; the first repair explained the trap IN PROSE
        // CONTAINING THE SEQUENCE, and reddened again on the explanation. 🔑 *A guard that greps a
        // file greps its prose, and the better the prose explains the defect, the more reliably it
        // reproduces it* — story 14.2's finding, met twice in five minutes. Hence: the prefix is
        // assembled from a quote character and the namespace, and this comment names neither
        // adjacently.
        let needle = format!("{}ipam.", '"');
        let mut keys: Vec<&str> = Vec::new();
        let mut rest = source;
        while let Some(at) = rest.find(&needle) {
            let after = &rest[at + 1..];
            let end = after.find('"').expect("a closed string literal");
            let name = &after[..end];
            if !keys.contains(&name) {
                keys.push(name);
            }
            rest = &after[end..];
        }
        assert!(
            keys.len() >= 15,
            "the scan found only {} key(s), so it is no longer reading this file: {keys:?}",
            keys.len()
        );
        for locale in ["en", "fr"] {
            for name in &keys {
                let sentence = rust_i18n::t!(*name, locale = locale).to_string();
                assert_ne!(
                    &sentence, name,
                    "`{name}` has no `{locale}` translation, so the operator reads its key name"
                );
                assert!(
                    !sentence.trim().is_empty(),
                    "`{name}` is BLANK in `{locale}` — rendering nothing is worse than rendering \
                     the other language"
                );
            }
        }
    }

    /// Every refusal the HANDLER CAN RECEIVE names a rule the operator can read.
    ///
    /// 🔴 **The set is what the adapter hands back, not `IpamError::ALL`** — §1(c) measured that
    /// four of the refusals an operator meets first are not `IpamError`s at all: an empty plan is
    /// `NotFound`, a re-entered address is `Constraint("unique")`, the loser of a lock wait is
    /// `Contention` since T1 repaired `classify`, and anything else is `Backend`. *A set over
    /// `IpamError` is correct about `IpamError` and silent about the four.*
    #[test]
    fn every_refusal_the_handler_can_receive_names_a_rule() {
        let receivable = [
            RepositoryError::NotFound,
            RepositoryError::Constraint("unique"),
            RepositoryError::Constraint("foreign_key"),
            RepositoryError::Constraint("check"),
            RepositoryError::Contention,
            RepositoryError::Backend("1406: Data too long for column 'label'".to_string()),
            RepositoryError::Ipam(IpamError::RangeOverlapsAnother),
        ];
        for error in &receivable {
            let refusal = repository_refusal(error);
            assert!(
                refusal.status().is_client_error() || refusal.status().is_server_error(),
                "{error} answered {}, which is not a refusal at all",
                refusal.status()
            );
            for locale in ["en", "fr"] {
                let sentence = rust_i18n::t!(refusal.key(), locale = locale).to_string();
                assert_ne!(
                    sentence,
                    refusal.key(),
                    "{error} renders its key name in `{locale}`"
                );
            }
            assert!(
                !rust_i18n::t!(refusal.key()).contains("1406"),
                "the driver's own sentence reached the operator: {error}"
            );
        }
    }

    /// **AC9b — a write that never finishes answers, and does not hold the browser.**
    ///
    /// ⚠️ **The clock is PAUSED, and what that does and does not prove is said rather than left to
    /// be assumed.** `start_paused` makes tokio auto-advance to the next timer, so this measures
    /// the BRANCH — that the budget is armed, that it wraps the work, and that the operator gets a
    /// sentence — and it does NOT measure five real seconds. The alternative was a five-second
    /// test, which buys one number and costs it on every run for ever.
    ///
    /// 🔑 The port's future never completes, so without the budget this test would HANG rather
    /// than fail — which is the defect exactly: a handler with no budget does not answer wrongly,
    /// it does not answer.
    #[tokio::test(start_paused = true)]
    async fn a_write_that_never_finishes_answers_within_its_budget() {
        struct NeverAnswers;
        impl IpamWritePort for NeverAnswers {
            fn define_subnet(
                &self,
                _subnet: Subnet,
                _label: String,
            ) -> BoxFuture<'_, Result<String, RepositoryError>> {
                Box::pin(std::future::pending())
            }
            fn define_range(
                &self,
                _subnet_id: String,
                _first: Ipv4Addr,
                _last: Ipv4Addr,
                _policy: opencmdb_core::ipam::IpPolicy,
                _label: String,
            ) -> BoxFuture<'_, Result<String, RepositoryError>> {
                Box::pin(std::future::pending())
            }
            fn define_address(
                &self,
                _subnet_id: String,
                _addr: Ipv4Addr,
                _label: String,
            ) -> BoxFuture<'_, Result<String, RepositoryError>> {
                Box::pin(std::future::pending())
            }
        }
        let (status, body) = drive(
            Arc::new(NeverAnswers),
            form_post("cidr=192.0.2.0/24&label=Office"),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::SERVICE_UNAVAILABLE,
            "a write that outlives its budget is answered, not waited out"
        );
        assert_eq!(
            body,
            key("ipam.refusal.contention"),
            "the operator is told the same thing as for a real lock wait, because the action is \
             the same: nothing was written, try again"
        );
    }

    /// A body every field of which is valid, for each route.
    ///
    /// 🔑 The `match` is exhaustive, so a fourth route cannot be added without saying what a good
    /// request to it looks like — which is what makes the reuse test cover it automatically rather
    /// than by someone remembering.
    fn a_valid_body(route: WriteRoute, label: &str) -> String {
        let subnet = "01900000-0000-7000-8000-0000000000aa";
        match route {
            WriteRoute::Subnet => format!("cidr=192.0.2.0/24&label={label}"),
            WriteRoute::Range => format!(
                "subnet_id={subnet}&first=192.0.2.10&last=192.0.2.20&policy=static&label={label}"
            ),
            WriteRoute::Address => format!("subnet_id={subnet}&addr=192.0.2.9&label={label}"),
        }
    }

    /// **AC1 — the second and third routes REUSE the machinery, and this drives all three through
    /// the same probes rather than trusting a comment that says so.**
    ///
    /// 🔴 The criterion's words are *"the others reuse it, and a TEST asserts the reuse rather than
    /// a comment claiming it"*. A comment can be true when written and false two commits later; a
    /// loop over `WriteRoute::ALL` covers a route the day it is added.
    ///
    /// ⚠️ What it asserts is that each route answers the SAME WAY to the same shared mistake — not
    /// that they call the same function, which no test can see. A second implementation that
    /// happened to behave identically would pass, and that is the honest limit of a behavioural
    /// guard. What narrows it is that the shape refusal is per-route and asserted DISTINCT, so a
    /// copy would have to reproduce the difference too.
    #[tokio::test]
    async fn every_route_reuses_the_shared_machinery() {
        let mut shape_sentences: Vec<String> = Vec::new();
        for route in WriteRoute::ALL {
            let route = *route;
            let at = route.path();

            // The Origin check, decided first: the port answers a SUCCESS, so a route that skipped
            // the check would go green on a 201.
            let cross = Request::builder()
                .method("POST")
                .uri(at)
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::HOST, "opencmdb.example")
                .header(header::ORIGIN, "https://attacker.example")
                .body(Body::from(a_valid_body(route, "Office")))
                .expect("a well-formed test request");
            let port = FakePort::answering(Ok("unreached".to_string()));
            let (status, body) = drive(port.clone(), cross).await;
            assert_eq!(
                status,
                StatusCode::FORBIDDEN,
                "`{at}` skipped the Origin check"
            );
            assert_eq!(body, crate::write_guard::CSRF_REFUSED_BODY, "at `{at}`");
            assert!(
                port.asked.lock().expect("the log").is_empty(),
                "`{at}` reached the port despite a cross-origin request"
            );

            // The label rules, both halves.
            for (label, key_name, what) in [
                ("%20%20", "ipam.refusal.label_empty", "a blank label"),
                (
                    "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                    "ipam.refusal.label_too_long",
                    "a label past the column",
                ),
            ] {
                let port = FakePort::answering(Ok("unreached".to_string()));
                let (status, body) =
                    drive(port, form_post_to(route, &a_valid_body(route, label))).await;
                assert_eq!(
                    status,
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "`{at}` accepted {what}"
                );
                assert_eq!(
                    body,
                    key(key_name),
                    "`{at}` named the wrong rule for {what}"
                );
            }

            // The shape refusal, which is per-route and must stay so.
            let port = FakePort::answering(Ok("unreached".to_string()));
            let (status, body) = drive(port, form_post_to(route, "nothing=useful")).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "at `{at}`");
            assert!(
                !shape_sentences.contains(&body),
                "`{at}` gives the same shape sentence as another form, so it names none of its own \
                 fields: {body}"
            );
            shape_sentences.push(body);

            // And the success path, including the redirect every route owes.
            let port = FakePort::answering(Ok("01900000-0000-7000-8000-0000000000bb".to_string()));
            let response = router_with(port)
                .oneshot(form_post_to(route, &a_valid_body(route, "Office")))
                .await
                .expect("the sub-router answers");
            assert_eq!(response.status(), StatusCode::CREATED, "at `{at}`");
            assert!(
                response
                    .headers()
                    .get("hx-redirect")
                    .is_some_and(|value| value
                        .to_str()
                        .is_ok_and(|value| value.starts_with("/ipam?subnet="))),
                "`{at}` does not send the browser back to the plan it changed"
            );
        }
        assert_eq!(
            shape_sentences.len(),
            WriteRoute::ALL.len(),
            "every route was probed"
        );
    }

    /// The two routes that RECEIVE a subnet id refuse the nil sentinel, at the route.
    #[tokio::test]
    async fn a_nil_subnet_id_is_refused_before_the_store() {
        for (route, body) in [
            (
                WriteRoute::Range,
                "subnet_id=00000000-0000-0000-0000-000000000000&first=192.0.2.10&last=192.0.2.20&policy=static&label=Office",
            ),
            (
                WriteRoute::Address,
                "subnet_id=00000000-0000-0000-0000-000000000000&addr=192.0.2.9&label=Office",
            ),
        ] {
            let port = FakePort::answering(Ok("unreached".to_string()));
            let (status, _) = drive(port.clone(), form_post_to(route, body)).await;
            assert_eq!(
                status,
                StatusCode::UNPROCESSABLE_ENTITY,
                "`{}` accepted the nil UUID",
                route.path()
            );
            assert!(
                port.asked.lock().expect("the log").is_empty(),
                "`{}` handed the nil sentinel to the store",
                route.path()
            );
        }
    }

    /// A policy outside the four binding words is refused, and the four are accepted.
    #[tokio::test]
    async fn only_the_four_binding_policies_are_accepted() {
        for policy in opencmdb_core::ipam::IpPolicy::ALL {
            let port = FakePort::answering(Ok("id".to_string()));
            let body = format!(
                "subnet_id=01900000-0000-7000-8000-0000000000aa&first=192.0.2.10&last=192.0.2.20&policy={}&label=Office",
                policy.as_str()
            );
            let (status, _) = drive(port, form_post_to(WriteRoute::Range, &body)).await;
            assert_eq!(status, StatusCode::CREATED, "`{policy}` is a binding word");
        }
        // ⚠️ `Static ` with a trailing space is the PAD SPACE trap one layer up: the schema's own
        // `IN (...)` accepts it, so the route must not.
        for rejected in ["structural", "Static", "static ", ""] {
            let port = FakePort::answering(Ok("unreached".to_string()));
            let body = format!(
                "subnet_id=01900000-0000-7000-8000-0000000000aa&first=192.0.2.10&last=192.0.2.20&policy={rejected}&label=Office"
            );
            let (status, sentence) = drive(port, form_post_to(WriteRoute::Range, &body)).await;
            assert_eq!(
                status,
                StatusCode::UNPROCESSABLE_ENTITY,
                "`{rejected}` was accepted as a policy"
            );
            assert_eq!(sentence, key("ipam.refusal.unknown_policy"));
        }
    }

    /// The list the router is built from is the list the guard walks — asserted, because the whole
    /// point of `WriteRoute` is that a route cannot be declared and left unmounted.
    #[tokio::test]
    async fn every_declared_route_is_mounted() {
        for route in WriteRoute::ALL {
            let port = FakePort::answering(Ok("id".to_string()));
            let response = router_with(port)
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(route.path())
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .header(header::HOST, "opencmdb.example")
                        .header(header::ORIGIN, "https://opencmdb.example")
                        .body(Body::from("cidr=192.0.2.0/24&label=Office"))
                        .expect("a well-formed test request"),
                )
                .await
                .expect("the sub-router answers");
            assert_ne!(
                response.status(),
                StatusCode::NOT_FOUND,
                "`{}` is declared and not mounted",
                route.path()
            );
        }
    }
}
