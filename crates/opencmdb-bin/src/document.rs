//! The documenting route's SHAPE — and nothing else (story 6.1, FR13(a)).
//!
//! `POST /document-all` exists so the route's refusals are settled while nothing is at stake:
//! **nothing at all is written here** — the write is story 6.2's, on story 5.3's precedent (the
//! vocabulary ships before the engine). The no-write claim is carried in three named layers
//! (story 6.1 §8), each honest about its strength:
//!
//! 1. **by type** — this sub-router's state is [`DocumentState`], which holds a read-only
//!    lookup and NO pool field, so the handler cannot extract `State<MySqlPool>`: adding that
//!    parameter fails to compile (mutation M4, re-measured);
//! 2. **by gate** — a write to `declared_attribute` from this file through any smuggled
//!    connection reds story 5.12's `authorship` gate (this file is not a sanctioned site);
//! 3. ⚠️ **by nothing** — a handler that opened its OWN connection from env and wrote any OTHER
//!    table is carried by no guard. On story 5.12's precedent this design is a TRIPWIRE against
//!    the good-faith mistake, never a barrier against a determined one.
//!
//! The vocabulary is `document` / `document-all` (`architecture.md:3818`) — the epic line's
//! older verb is not canonical (D65 names the retirement pattern), and a test over this file
//! carries the ban because the tree-wide gate cannot (story 6.1 §5, measured).

use std::sync::Arc;

use axum::extract::State;
use axum::extract::rejection::FormRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Form, Router};
use opencmdb_core::document::DocumentRefusal;
use opencmdb_core::observation::ObsId;
use opencmdb_core::repo::BoxFuture;
use serde::Deserialize;

/// The route, under the vocabulary's own token (story 6.1 §5). Tests spell the literal out
/// independently — deliberate redundancy, so renaming either side reds.
pub(crate) const DOCUMENT_ALL_PATH: &str = "/document-all";

/// The request shape: `application/x-www-form-urlencoded`, ⚠️ because that is what the vendored
/// htmx 2.0.4 posts (measured: form-values encoding, zero `fetch(`, no `json-enc` extension).
/// A JSON route here would force story 6.4 to vendor an extension or redo this shape.
#[derive(Debug, Deserialize)]
pub(crate) struct DocumentAllRequest {
    /// The subject: an observation id (UUIDv7 text). FR13(a) documents a SIGHTING's whole
    /// record, and the observation is what the reach section's cause line will name. Kept as
    /// text and parsed deliberately, so the 422 body is this module's, not serde's.
    pub(crate) subject: String,
}

/// Read-only: answers whether a subject may be documented. The document sub-router's WHOLE
/// state reaches the world through this trait — it can look, never write.
pub(crate) trait SubjectLookup: Send + Sync {
    /// `Ok(())` when the subject is known; the domain refusal otherwise.
    fn check(&self, subject: ObsId) -> BoxFuture<'_, Result<(), DocumentRefusal>>;
}

/// Story 6.1's production wiring: a shape-only route truthfully answers *unknown* for every
/// subject, since nothing can be documented yet. A test pins this wiring; story 6.2 replaces it
/// with the store-backed impl. The known-subject branch is reached through the in-memory test
/// impl only.
pub(crate) struct AlwaysUnknown;

impl SubjectLookup for AlwaysUnknown {
    fn check(&self, _subject: ObsId) -> BoxFuture<'_, Result<(), DocumentRefusal>> {
        Box::pin(async { Err(DocumentRefusal::UnknownSubject) })
    }
}

/// The sub-router's state — deliberately NO pool field (story 6.1 §7): `page.rs`'s
/// `State<MySqlPool>` extractors stay byte-for-byte unchanged on the main router, and the
/// document handler can extract only what THIS struct holds.
#[derive(Clone)]
pub(crate) struct DocumentState {
    /// The one capability the route has: asking whether a subject exists.
    lookup: Arc<dyn SubjectLookup>,
}

/// The production sub-router, wired to [`AlwaysUnknown`].
pub(crate) fn router() -> Router {
    router_with(Arc::new(AlwaysUnknown))
}

/// The sub-router over an explicit lookup — the seam tests use to reach the known-subject
/// branch without a database.
pub(crate) fn router_with(lookup: Arc<dyn SubjectLookup>) -> Router {
    Router::new()
        .route(DOCUMENT_ALL_PATH, post(document_all))
        .with_state(DocumentState { lookup })
}

/// The handler. Every refusal is enumerated and mapped deliberately (story 6.1 §6); the status
/// half lives here, the domain half in `opencmdb_core::document` (D47 — no `axum` in core).
async fn document_all(
    State(state): State<DocumentState>,
    form: Result<Form<DocumentAllRequest>, FormRejection>,
) -> Response {
    let Ok(Form(request)) = form else {
        return malformed();
    };
    let Ok(subject) = request.subject.parse::<uuid::Uuid>() else {
        return malformed();
    };
    match state.lookup.check(ObsId::from_uuid(subject)).await {
        // Exhaustive, no `_` arm (story 5.3's precedent): a new refusal variant must produce
        // `error[E0004]` here, never fall into a silent catch-all.
        Err(refusal) => match refusal {
            // 404, ⚠️ colliding with axum's unregistered-route 404 — the BODY is the
            // discriminator (§6): the fallback's body is empty, this one is the domain's own
            // sentence, pinned verbatim by a test.
            DocumentRefusal::UnknownSubject => {
                (StatusCode::NOT_FOUND, refusal.to_string()).into_response()
            }
        },
        // The subject is known and NOTHING IS WRITTEN: the write is story 6.2's, and answering
        // success for a write that did not happen would be a lie. 501 says exactly what is true.
        Ok(()) => (
            StatusCode::NOT_IMPLEMENTED,
            "documenting is not implemented yet: nothing was written",
        )
            .into_response(),
    }
}

/// The request-shape refusal: 422, naming the field (story 6.1 §6). One body for both shapes
/// (extractor rejection, non-UUID text) — the field is the same and so is the fix.
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

    /// The body the unknown-subject 404 must carry, spelled out here INDEPENDENTLY of the
    /// domain's `Display` (deliberate redundancy): if either side drifts, this reds.
    const UNKNOWN_SUBJECT_BODY: &str = "unknown subject: the id names no observation";

    fn form_post(body: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(DOCUMENT_ALL_PATH)
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(body.to_string()))
            .expect("a valid request")
    }

    async fn body_text(response: Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("a readable body");
        String::from_utf8(bytes.to_vec()).expect("a UTF-8 body")
    }

    /// A missing `subject` field answers 422 and the body names the field (AC2, M2).
    #[tokio::test]
    async fn a_form_without_the_subject_field_answers_422_naming_it() {
        let response = router().oneshot(form_post("unrelated=x")).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_text(response).await;
        assert!(body.contains("subject"), "the body names the field: {body}");
    }

    /// A `subject` that is not a UUID is a SHAPE refusal (422), not an unknown subject.
    #[tokio::test]
    async fn a_subject_that_is_not_a_uuid_answers_422() {
        let response = router()
            .oneshot(form_post("subject=not-a-uuid"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_text(response).await;
        assert!(body.contains("subject"), "the body names the field: {body}");
    }

    /// The unknown-subject 404 carries the EXACT pinned body (AC2, M3b) — the discriminator
    /// against the fallback's empty-body 404 (§6).
    #[tokio::test]
    async fn an_unknown_subject_answers_404_with_the_pinned_body() {
        let subject = uuid::Uuid::now_v7();
        let response = router()
            .oneshot(form_post(&format!("subject={subject}")))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(body_text(response).await, UNKNOWN_SUBJECT_BODY);
    }

    /// An in-memory lookup that knows exactly one subject — the seam that makes the
    /// unknown-subject refusal CONDITIONAL (§6: without this pair the refusal is unconditional
    /// and its test vacuous).
    struct KnowsOne(ObsId);

    impl SubjectLookup for KnowsOne {
        fn check(&self, subject: ObsId) -> BoxFuture<'_, Result<(), DocumentRefusal>> {
            let known = self.0;
            Box::pin(async move {
                if subject == known {
                    Ok(())
                } else {
                    Err(DocumentRefusal::UnknownSubject)
                }
            })
        }
    }

    /// A KNOWN subject does NOT answer `UnknownSubject` (AC2's discriminating half, M3) — and
    /// what it does answer says truthfully that nothing was written.
    #[tokio::test]
    async fn a_known_subject_is_not_answered_unknown() {
        let known = ObsId::from_uuid(uuid::Uuid::now_v7());
        let response = router_with(Arc::new(KnowsOne(known)))
            .oneshot(form_post(&format!("subject={}", known.as_uuid())))
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::NOT_IMPLEMENTED,
            "known subject: the route's honest answer, never the unknown-subject refusal"
        );
        let body = body_text(response).await;
        assert_ne!(body, UNKNOWN_SUBJECT_BODY);
        assert!(
            body.contains("nothing was written"),
            "the body says what is true: {body}"
        );
    }

    /// The same lookup refuses a DIFFERENT subject — the other half of the discriminating pair.
    #[tokio::test]
    async fn the_knowing_lookup_still_refuses_an_unknown_subject() {
        let known = ObsId::from_uuid(uuid::Uuid::now_v7());
        let other = uuid::Uuid::now_v7();
        let response = router_with(Arc::new(KnowsOne(known)))
            .oneshot(form_post(&format!("subject={other}")))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(body_text(response).await, UNKNOWN_SUBJECT_BODY);
    }

    /// GET on the route answers 405 — the cheap method pin (§3's names table).
    #[tokio::test]
    async fn get_on_the_route_answers_405() {
        let request = Request::builder()
            .method("GET")
            .uri(DOCUMENT_ALL_PATH)
            .body(Body::empty())
            .unwrap();
        let response = router().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    /// The production wiring is `AlwaysUnknown`, pinned directly (AC2): every subject is
    /// truthfully unknown while nothing can be documented.
    #[tokio::test]
    async fn always_unknown_refuses_every_subject() {
        let verdict = AlwaysUnknown
            .check(ObsId::from_uuid(uuid::Uuid::now_v7()))
            .await;
        assert_eq!(verdict, Err(DocumentRefusal::UnknownSubject));
    }

    /// AC4: the vocabulary is `document`, carried HERE because the tree-wide `vocabulary` gate
    /// is inert for this word (story 6.1 §5, measured). The banned token is assembled from
    /// parts so this file never carries it; M6 (rename the route) reds on the path pin.
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
