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
//! `ipam.refusal.malformed` are different sentences about different gestures, and one body serving
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
}

impl WriteRoute {
    /// Every write route this sub-router carries.
    pub(crate) const ALL: &'static [WriteRoute] = &[WriteRoute::Subnet];

    /// The path the route is mounted at, and the path the guard probes.
    pub(crate) const fn path(self) -> &'static str {
        match self {
            WriteRoute::Subnet => "/ipam/subnet",
        }
    }

    /// The method router mounted at [`WriteRoute::path`].
    fn handler(self) -> MethodRouter<IpamWriteState> {
        match self {
            WriteRoute::Subnet => post(define_subnet),
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
        return malformed().into_response();
    };
    let label = match checked_label(&request.label) {
        Ok(label) => label,
        Err(refusal) => return refusal.into_response(),
    };
    let subnet = match parse_cidr(&request.cidr) {
        Ok(subnet) => subnet,
        Err(refusal) => return refusal.into_response(),
    };
    match state.port.define_subnet(subnet, label).await {
        Ok(id) => {
            tracing::info!(subnet = %subnet.cidr(), id = %id, "defined a subnet of the plan");
            // 🔑 `HX-Redirect` for the same reason story 6.4 adopted it: a narrow swap would leave
            // the empty-plan sentence standing over a plan that now exists — a claim and its
            // refutation in one viewport. The browser goes back to `/ipam`, which re-renders from
            // the store, selected on the subnet just defined. *One URL per state* is epic
            // constraint 4, and post-redirect-get is its idiom rather than an exception to it.
            (
                StatusCode::CREATED,
                [(
                    axum::http::HeaderName::from_static("hx-redirect"),
                    format!("/ipam?subnet={id}"),
                )],
                rust_i18n::t!("ipam.done.subnet").to_string(),
            )
                .into_response()
        }
        Err(error) => {
            let refusal = repository_refusal(&error);
            // The driver's sentence stays in the log and never reaches the body. Logged HERE
            // rather than inside the mapper, so the mapper stays pure and testable as data.
            if refusal.status() == StatusCode::INTERNAL_SERVER_ERROR {
                tracing::error!(%error, "the addressing plan's write failed at the backend");
            }
            refusal.into_response()
        }
    }
}

/// The request-shape refusal, one sentence for every shape mistake: an extractor rejection of any
/// class (the body-size limit included) and a CIDR the product cannot parse.
const fn malformed() -> Refusal {
    Refusal::new(StatusCode::UNPROCESSABLE_ENTITY, "ipam.refusal.malformed")
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
/// A 422: `ipam.refusal.malformed` when the text is not a CIDR at all, or the keyed sentence of
/// whichever [`IpamError`] [`Subnet::new`] raises.
fn parse_cidr(raw: &str) -> Result<Subnet, Refusal> {
    let (base, prefix) = raw.trim().split_once('/').ok_or_else(malformed)?;
    let base: Ipv4Addr = base.parse().map_err(|_| malformed())?;
    // ⚠️ A prefix that does not fit a `u8` — `/999` — is MALFORMED, while one that fits and cannot
    // belong to IPv4 — `/64` — gets `PrefixLengthNotInFamily`, which names the real rule. Two
    // sentences for what looks like one mistake, and the distinction is the honest one: the product
    // can only name a rule it can evaluate.
    let prefix: u8 = prefix.parse().map_err(|_| malformed())?;
    Subnet::new(base, prefix).map_err(|error| ipam_refusal(&error))
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
        RepositoryError::Constraint(name) => match *name {
            // The likeliest refusal on this screen: the operator re-enters a subnet the plan
            // already holds. It rides `ip_subnet_cidr`, and no pre-read is taken — a check that
            // commits separately from its write is a TOCTOU hole, not a check.
            "unique" => Refusal::new(StatusCode::CONFLICT, "ipam.refusal.already_defined"),
            // A `subnet_id` that names no row — reachable from the two routes that receive one.
            "foreign_key" => Refusal::new(StatusCode::NOT_FOUND, "ipam.refusal.unknown_subnet"),
            _ => backend(),
        },
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
        IpamError::MalformedAddress => "ipam.refusal.malformed_address",
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
            let answer = self
                .answer
                .lock()
                .expect("the fake port's answer")
                .take()
                .expect("the fake port answers once per test");
            Box::pin(async move { answer })
        }
    }

    /// A same-origin urlencoded POST to the subnet route. `Origin`/`Host` agree, so the CSRF check
    /// passes; the test that probes the check sets the headers itself.
    fn form_post(body: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(WriteRoute::Subnet.path())
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
        assert_eq!(body, key("ipam.refusal.malformed"));
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
            assert_eq!(body, key("ipam.refusal.malformed"), "for `{spelling}`");
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
