//! The documenting route — it now WRITES a declared value (story 6.2, FR13(a)).
//!
//! `POST /document-all` turns an observed value into a declared one: milestone J3's *corrected*
//! half, which had no code path until this story. The write lands in `declared_attribute` as an
//! ADOPTED row, through `repo::adopt_declared_attribute` and NOWHERE ELSE (story 5.12's
//! authorship gate measures it).
//!
//! The 6.1 no-write carrier that STILL holds: [`DocumentState`] holds NO pool field, so the
//! handler cannot extract `State<MySqlPool>` (mutation M4 is a compile error). The pool lives
//! INSIDE [`StoreDocument`], behind `Arc<dyn DocumentPort>` — the handler reaches the database
//! only through the port's one method, never by an extractor.
//!
//! The vocabulary is `document` / `document-all` (`architecture.md:3818`) — the epic line's
//! older verb is not canonical (D65), and a test over this file carries the ban because the
//! tree-wide gate cannot (story 6.1 §5, measured).

use std::sync::Arc;

use axum::extract::State;
use axum::extract::rejection::FormRejection;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Form, Router};
use opencmdb_core::document::DocumentRefusal;
use opencmdb_core::observation::ObsId;
use opencmdb_core::repo::BoxFuture;
use serde::Deserialize;
use sqlx::MySqlPool;

use crate::repo;

/// The route, under the vocabulary's own token (story 6.1 §5). Tests spell the literal out
/// independently — deliberate redundancy, so renaming either side reds.
pub(crate) const DOCUMENT_ALL_PATH: &str = "/document-all";

/// The realm the CSRF 403 does NOT advertise — kept beside the check so the 403 body and the
/// auth challenge never blur (§5: the 403 is not an auth failure).
const CSRF_REFUSED_BODY: &str = "cross-origin request refused";

/// The request shape: `application/x-www-form-urlencoded`, ⚠️ because that is what the vendored
/// htmx 2.0.4 posts (measured: form-values encoding, zero `fetch(`, no `json-enc` extension).
/// A JSON route here would force story 6.4 to vendor an extension or redo this shape.
#[derive(Debug, Deserialize)]
pub(crate) struct DocumentAllRequest {
    /// The subject: an observation id in UUID text (minted v7 by D48). The route validates the
    /// UUID shape and refuses the NIL sentinel; the id is re-serialised canonical before any SQL
    /// sees it (story 6.2 §2), so a braced/urn:/hyphenless spelling of a real id is harmless.
    pub(crate) subject: String,
}

/// A successful documenting gesture: the entity it minted and how many declared fields it wrote.
pub(crate) struct Documented {
    /// The freshly-minted entity id (a v7 UUID); N adopted rows share it.
    pub(crate) entity_id: String,
    /// Rows WRITTEN — the projected field count after first-occurrence-wins per key, not the
    /// number of facts seen.
    pub(crate) fields: usize,
}

/// Why a documenting gesture did not happen: a DOMAIN refusal (mapped to its status
/// exhaustively, no `_` arm) or a backend failure (500 — `sqlx` stays in bin, D47).
pub(crate) enum DocumentFailure {
    /// A domain refusal with a defined status and pinned body.
    Refused(DocumentRefusal),
    /// A backend failure — logged, never leaked into the response body.
    Backend(sqlx::Error),
}

/// The port: the WHOLE documenting gesture for one subject, atomically. The sub-router's state
/// reaches the world only through this — it can document, and it holds whatever it needs to do
/// so INSIDE the impl, never on the router's state.
pub(crate) trait DocumentPort: Send + Sync {
    /// Perform the documenting gesture for `subject`: verify it names an observation, project
    /// its facts (empty → `NothingToDocument`), mint an entity, and write the adopted rows in
    /// ONE transaction — the unique index turning a re-adoption into `AlreadyDocumented` (no
    /// pre-read, story 6.2 §4/§6.5). A check that commits separately from its write is a TOCTOU
    /// hole, not a check.
    fn document_all(&self, subject: ObsId) -> BoxFuture<'_, Result<Documented, DocumentFailure>>;
}

/// The production wiring: the gesture over a MariaDB pool. The pool lives HERE, inside the impl,
/// not on [`DocumentState`] — which is what keeps the pool unreachable from the handler by type
/// (story 6.1's M4 carrier survives).
pub(crate) struct StoreDocument {
    pool: MySqlPool,
}

impl StoreDocument {
    /// Wire the gesture to a pool.
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

impl DocumentPort for StoreDocument {
    fn document_all(&self, subject: ObsId) -> BoxFuture<'_, Result<Documented, DocumentFailure>> {
        Box::pin(async move {
            let subject_text = subject.as_uuid().to_string(); // canonical, whatever spelling arrived
            let mut tx = self.pool.begin().await.map_err(DocumentFailure::Backend)?;

            // Unknown subject: the id names no observation.
            let Some(observation) = repo::load_observation_by_id(&mut *tx, &subject_text)
                .await
                .map_err(DocumentFailure::Backend)?
            else {
                return Err(DocumentFailure::Refused(DocumentRefusal::UnknownSubject));
            };

            // The projection is core's, shared with the reconcile — or the gap never closes.
            // First occurrence wins per key (a multi-homed device is normal, not a refusal).
            let mut seen: Vec<String> = Vec::new();
            let mut fields: Vec<(String, String)> = Vec::new();
            for (key, value) in opencmdb_core::gap::project(&observation) {
                if !seen.contains(&key) {
                    seen.push(key.clone());
                    fields.push((key, value));
                }
            }
            if fields.is_empty() {
                return Err(DocumentFailure::Refused(DocumentRefusal::NothingToDocument));
            }

            let entity_id = uuid::Uuid::now_v7().to_string();
            for (key, value) in &fields {
                if let Err(error) =
                    repo::adopt_declared_attribute(&mut *tx, &entity_id, key, value, &subject_text)
                        .await
                {
                    // The already-documented refusal rides the unique index, keyed on its NAME
                    // (§6.5): the entity id is freshly minted so the PK cannot collide, and this
                    // is the only unique the adopt INSERT can violate — but keying on the name
                    // keeps the mapping honest against any future constraint.
                    if is_adoption_conflict(&error) {
                        return Err(DocumentFailure::Refused(DocumentRefusal::AlreadyDocumented));
                    }
                    return Err(DocumentFailure::Backend(error));
                }
            }
            tx.commit().await.map_err(DocumentFailure::Backend)?;
            Ok(Documented {
                entity_id,
                fields: fields.len(),
            })
        })
    }
}

/// Whether a write error is the adoption index firing (`declared_one_adoption_per_field`) — the
/// already-documented case. Keyed on the index NAME, not on the bare unique-violation shape, so
/// a future constraint does not silently answer `AlreadyDocumented` (story 6.2 §6, M6).
fn is_adoption_conflict(error: &sqlx::Error) -> bool {
    error
        .as_database_error()
        .is_some_and(|db| db.message().contains("declared_one_adoption_per_field"))
}

/// The sub-router's state — deliberately NO pool field (story 6.1 §7 / 6.2's M4 carrier):
/// `page.rs`'s `State<MySqlPool>` extractors stay byte-for-byte unchanged on the main router,
/// and the document handler can extract only what THIS struct holds — a port, not a pool.
#[derive(Clone)]
pub(crate) struct DocumentState {
    port: Arc<dyn DocumentPort>,
}

/// The production sub-router, wired to the store-backed port.
pub(crate) fn router(pool: MySqlPool) -> Router {
    router_with(Arc::new(StoreDocument::new(pool)))
}

/// The sub-router over an explicit port — the seam tests use to drive the gesture without a
/// database (an in-memory `DocumentPort`).
pub(crate) fn router_with(port: Arc<dyn DocumentPort>) -> Router {
    Router::new()
        .route(DOCUMENT_ALL_PATH, post(document_all))
        .with_state(DocumentState { port })
}

/// The handler. The CSRF check is decided FIRST — no refusal path consults the parsed form
/// (story 6.2 §5). ⚠️ It cannot literally run *before* the parser: `form` is an axum extractor,
/// so the body is parsed before the handler body executes; what holds is that the 403 wins over
/// every other refusal.
async fn document_all(
    State(state): State<DocumentState>,
    headers: HeaderMap,
    form: Result<Form<DocumentAllRequest>, FormRejection>,
) -> Response {
    if !same_origin(&headers) {
        return (StatusCode::FORBIDDEN, CSRF_REFUSED_BODY).into_response();
    }
    let Ok(Form(request)) = form else {
        return malformed();
    };
    let Ok(subject) = request.subject.parse::<uuid::Uuid>() else {
        return malformed();
    };
    // The nil UUID is a load-bearing sentinel (D21) and D48 mints v7, so it can never name an
    // observation — refused as a SHAPE error before the store ever sees it.
    if subject.is_nil() {
        return malformed();
    }
    match state.port.document_all(ObsId::from_uuid(subject)).await {
        Ok(documented) => (
            StatusCode::CREATED,
            format!(
                "documented {} field(s) as entity {}",
                documented.fields, documented.entity_id
            ),
        )
            .into_response(),
        Err(DocumentFailure::Refused(refusal)) => match refusal {
            // 404, ⚠️ colliding with axum's unregistered-route 404 — the BODY is the
            // discriminator (§4): the fallback's body is empty, this is the domain's sentence.
            DocumentRefusal::UnknownSubject => {
                (StatusCode::NOT_FOUND, refusal.to_string()).into_response()
            }
            // 409 — documenting twice counts one box twice.
            DocumentRefusal::AlreadyDocumented => {
                (StatusCode::CONFLICT, refusal.to_string()).into_response()
            }
            // 422 (domain) — distinct body from the shape 422's.
            DocumentRefusal::NothingToDocument => {
                (StatusCode::UNPROCESSABLE_ENTITY, refusal.to_string()).into_response()
            }
        },
        Err(DocumentFailure::Backend(error)) => {
            tracing::error!(%error, "documenting failed at the backend");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "documenting failed — the store did not accept the write",
            )
                .into_response()
        }
    }
}

/// The CSRF Origin check (story 6.2 §5), pure over the request headers. It is a TRIPWIRE against
/// a browser holding the cached Basic credential being made to forge a cross-site write, at the
/// stated strength and no higher:
///
/// - **`Origin` absent → PASS** — a machine caller (`curl -u`) sends none; the threat is a
///   BROWSER, which sends `Origin` on every cross-site POST (measured, Blink);
/// - **`Origin: null` → REFUSE** — sandboxed iframes / some redirect chains; refused because it
///   carries no `://` authority to match (no dedicated branch — measured redundant);
/// - **`Origin` present → compare its authority against `Host`**, ASCII case-insensitively;
///   match → pass, mismatch → refuse. ⚠️ Stated limits: this needs the reverse proxy to FORWARD
///   `Host` (`proxy_set_header Host $host;` — nginx's default rewrite would refuse every POST);
///   `Host` ABSENT (HTTP/2 `:authority`) → refuse; the compare is authority-only, SCHEME-BLIND;
///   default-port elision is compared literally. All registered, none silently absorbed;
/// - **more than one `Origin` header → REFUSE** (6.1's `Authorization` precedent: first-value
///   semantics let right-then-wrong through).
fn same_origin(headers: &HeaderMap) -> bool {
    let mut origins = headers.get_all(header::ORIGIN).into_iter();
    let Some(origin) = origins.next() else {
        return true; // absent → machine caller, pass
    };
    if origins.next().is_some() {
        return false; // more than one Origin
    }
    let Ok(origin) = origin.to_str() else {
        return false;
    };
    // Strip the scheme; compare host[:port] against Host. `Origin: null` (opaque origins —
    // sandboxed iframes, some redirect chains) carries no `://`, so it falls into the refusal
    // below without a dedicated branch (a dedicated `== "null"` check was measured redundant at
    // the mutation pass — it is refused by the missing scheme regardless).
    let origin_authority = origin.split_once("://").map(|(_, rest)| rest);
    let Some(origin_authority) = origin_authority else {
        return false;
    };
    let Some(host) = headers.get(header::HOST).and_then(|h| h.to_str().ok()) else {
        return false; // Host absent (HTTP/2 :authority) → refuse
    };
    origin_authority.eq_ignore_ascii_case(host)
}

/// The request-shape refusal: 422, naming the field (story 6.1 §6). One body for EVERY shape
/// refusal — extractor rejection of any class (the body-size limit included), non-UUID text,
/// and the nil sentinel — naming the field is the only actionable hint a shape-only route can
/// give.
fn malformed() -> Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        "malformed request: expected form field `subject` carrying an observation id",
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, header};
    use tower::ServiceExt;

    use super::*;

    const UNKNOWN_SUBJECT_BODY: &str = "unknown subject: nothing can be documented";
    const ALREADY_BODY: &str = "already documented: this observation's fields are already declared";
    const NOTHING_BODY: &str = "nothing to document: the observation carries no declarable field";

    /// A same-origin urlencoded POST carrying `body`. `Origin`/`Host` agree so the CSRF check
    /// passes; tests that probe the CSRF check set the headers themselves.
    fn form_post(body: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(DOCUMENT_ALL_PATH)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::ORIGIN, "http://nas:8080")
            .header(header::HOST, "nas:8080")
            .body(Body::from(body.to_string()))
            .expect("a valid request")
    }

    async fn body_text(response: Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("a readable body");
        String::from_utf8(bytes.to_vec()).expect("a UTF-8 body")
    }

    // ── In-memory ports: drive the gesture with no database ────────────────────────────────

    /// A port whose every call returns a fixed outcome — for the refusal taxonomy.
    struct FixedPort(fn() -> Result<Documented, DocumentFailure>);
    impl DocumentPort for FixedPort {
        fn document_all(
            &self,
            _subject: ObsId,
        ) -> BoxFuture<'_, Result<Documented, DocumentFailure>> {
            let outcome = (self.0)();
            Box::pin(async move { outcome })
        }
    }

    fn port(outcome: fn() -> Result<Documented, DocumentFailure>) -> Arc<dyn DocumentPort> {
        Arc::new(FixedPort(outcome))
    }

    async fn answer(port: Arc<dyn DocumentPort>, body: &str) -> (StatusCode, String) {
        let response = router_with(port).oneshot(form_post(body)).await.unwrap();
        let status = response.status();
        (status, body_text(response).await)
    }

    fn a_subject() -> String {
        format!("subject={}", uuid::Uuid::now_v7())
    }

    #[tokio::test]
    async fn a_successful_gesture_answers_201_naming_the_entity_and_field_count() {
        let (status, body) = answer(
            port(|| {
                Ok(Documented {
                    entity_id: "00000000-0000-0000-0000-0000000000ee".to_string(),
                    fields: 2,
                })
            }),
            &a_subject(),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert!(body.contains('2'), "names the field count: {body}");
        assert!(body.contains("0000000000ee"), "names the entity: {body}");
    }

    #[tokio::test]
    async fn an_unknown_subject_answers_404_with_the_pinned_body() {
        let (status, body) = answer(
            port(|| Err(DocumentFailure::Refused(DocumentRefusal::UnknownSubject))),
            &a_subject(),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body, UNKNOWN_SUBJECT_BODY);
    }

    #[tokio::test]
    async fn an_already_documented_subject_answers_409() {
        let (status, body) = answer(
            port(|| Err(DocumentFailure::Refused(DocumentRefusal::AlreadyDocumented))),
            &a_subject(),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body, ALREADY_BODY);
    }

    #[tokio::test]
    async fn a_nothing_to_document_subject_answers_422_with_a_distinct_body() {
        let (status, body) = answer(
            port(|| Err(DocumentFailure::Refused(DocumentRefusal::NothingToDocument))),
            &a_subject(),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body, NOTHING_BODY);
        assert_ne!(
            body, "malformed request: expected form field `subject` carrying an observation id",
            "the domain 422 must not collide with the shape 422"
        );
    }

    #[tokio::test]
    async fn a_backend_failure_answers_500_without_leaking_the_error() {
        let (status, body) = answer(
            port(|| Err(DocumentFailure::Backend(sqlx::Error::PoolClosed))),
            &a_subject(),
        )
        .await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(!body.to_lowercase().contains("pool"), "no SQL leak: {body}");
    }

    // ── Shape refusals (no port call needed) ───────────────────────────────────────────────

    #[tokio::test]
    async fn a_form_without_the_subject_field_answers_422_naming_it() {
        let (status, body) = answer(port(|| unreachable!()), "unrelated=x").await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body.contains("subject"), "{body}");
    }

    #[tokio::test]
    async fn a_subject_that_is_not_a_uuid_answers_422() {
        let (status, body) = answer(port(|| unreachable!()), "subject=not-a-uuid").await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body.contains("subject"), "{body}");
    }

    #[tokio::test]
    async fn the_nil_uuid_is_refused_as_malformed() {
        let (status, _) = answer(
            port(|| unreachable!()),
            &format!("subject={}", uuid::Uuid::nil()),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn get_on_the_route_answers_405() {
        let request = Request::builder()
            .method("GET")
            .uri(DOCUMENT_ALL_PATH)
            .body(Body::empty())
            .unwrap();
        let response = router_with(port(|| unreachable!()))
            .oneshot(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    // ── CSRF (§5) end-to-end: the 403 is decided FIRST, before any form refusal ─────────────

    fn document_post(origin: Option<&str>, host: Option<&str>, body: &str) -> Request<Body> {
        let mut b = Request::builder()
            .method("POST")
            .uri(DOCUMENT_ALL_PATH)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
        if let Some(o) = origin {
            b = b.header(header::ORIGIN, o);
        }
        if let Some(h) = host {
            b = b.header(header::HOST, h);
        }
        b.body(Body::from(body.to_string())).unwrap()
    }

    async fn csrf(request: Request<Body>) -> (StatusCode, String) {
        let response = router_with(port(|| unreachable!()))
            .oneshot(request)
            .await
            .unwrap();
        let status = response.status();
        (status, body_text(response).await)
    }

    #[tokio::test]
    async fn a_cross_site_origin_is_refused_403_before_the_form_is_consulted() {
        // Malformed body too: proves the 403 wins over the 422 (M9 / M3).
        let (status, body) = csrf(document_post(
            Some("http://attacker.example"),
            Some("nas:8080"),
            "garbage",
        ))
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body, CSRF_REFUSED_BODY);
    }

    #[tokio::test]
    async fn an_absent_origin_passes_the_csrf_check() {
        // A machine caller: no Origin. It reaches the port (here 404-ish via unreachable guard),
        // so we assert only that it is NOT a 403.
        let (status, _) = answer(
            port(|| Err(DocumentFailure::Refused(DocumentRefusal::UnknownSubject))),
            &a_subject(),
        )
        .await;
        assert_ne!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn a_null_origin_is_refused() {
        let (status, _) = csrf(document_post(Some("null"), Some("nas:8080"), &a_subject())).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn two_origin_headers_are_refused() {
        let request = Request::builder()
            .method("POST")
            .uri(DOCUMENT_ALL_PATH)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::ORIGIN, "http://nas:8080")
            .header(header::ORIGIN, "http://attacker.example")
            .header(header::HOST, "nas:8080")
            .body(Body::from(a_subject()))
            .unwrap();
        assert_eq!(csrf(request).await.0, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn host_absent_is_refused() {
        let (status, _) = csrf(document_post(Some("http://nas:8080"), None, &a_subject())).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn a_matching_origin_and_host_pass_the_check() {
        // Reaches the port; not a 403.
        let request = document_post(Some("http://nas:8080"), Some("nas:8080"), &a_subject());
        let response = router_with(port(|| {
            Err(DocumentFailure::Refused(DocumentRefusal::UnknownSubject))
        }))
        .oneshot(request)
        .await
        .unwrap();
        assert_ne!(response.status(), StatusCode::FORBIDDEN);
    }

    /// The `same_origin` pure fn table (§5), including the stated limits pinned as behaviour.
    #[test]
    fn same_origin_decides_each_case() {
        let with = |pairs: &[(&str, &str)]| {
            let mut h = HeaderMap::new();
            for (k, v) in pairs {
                h.append(
                    if *k == "origin" {
                        header::ORIGIN
                    } else {
                        header::HOST
                    },
                    v.parse().unwrap(),
                );
            }
            h
        };
        assert!(
            same_origin(&with(&[("host", "nas:8080")])),
            "absent origin passes"
        );
        assert!(
            same_origin(&with(&[
                ("origin", "http://nas:8080"),
                ("host", "nas:8080")
            ])),
            "match passes"
        );
        assert!(
            same_origin(&with(&[
                ("origin", "HTTP://NAS:8080"),
                ("host", "nas:8080")
            ])),
            "case-insensitive authority match passes"
        );
        assert!(
            !same_origin(&with(&[
                ("origin", "http://attacker"),
                ("host", "nas:8080")
            ])),
            "mismatch refused"
        );
        assert!(
            !same_origin(&with(&[("origin", "null"), ("host", "nas:8080")])),
            "null refused"
        );
        assert!(
            !same_origin(&with(&[("origin", "http://nas:8080")])),
            "host absent refused"
        );
        // SCHEME-BLIND, stated limit: https origin passes against a bare-authority Host.
        assert!(
            same_origin(&with(&[
                ("origin", "https://nas:8080"),
                ("host", "nas:8080")
            ])),
            "scheme-blind (stated limit): same authority passes across schemes"
        );
    }

    /// AC6's SOURCE tripwire (§9 M12): the PRODUCTION half of this file must NOT carry a local
    /// key table for the projection vocabulary — it must call `gap::project`. A faithful private
    /// copy passes any behavioural test, so the no-copy property is checked textually. The scan
    /// stops at `#[cfg(test)]`: the tests legitimately name the keys as expectations.
    #[test]
    fn the_projection_is_shared_not_copied() {
        let full = include_str!("document.rs");
        let production = full.split("#[cfg(test)]").next().unwrap_or(full);
        assert!(
            production.contains("gap::project(&observation)"),
            "the write must go through the shared gap::project"
        );
        let offenders: Vec<&str> = production
            .lines()
            .filter(|l| {
                l.contains("\"ipv4\"") || l.contains("\"hostname\"") || l.contains("\"mac\"")
            })
            .collect();
        assert!(
            offenders.is_empty(),
            "production code must carry no local projection key table — use gap::project: {offenders:?}"
        );
    }

    /// AC4: the vocabulary is `document`; the banned token is assembled from parts so the file
    /// never carries it. M6 (rename the route) reds on the path pin.
    #[test]
    fn the_vocabulary_is_document_and_the_route_says_so() {
        assert_eq!(DOCUMENT_ALL_PATH, "/document-all");
        let source = include_str!("document.rs").to_ascii_lowercase();
        let banned = ["pro", "mote"].concat();
        assert!(
            !source.contains(&banned),
            "document.rs must not carry the epic line's non-canonical verb"
        );
    }
}
