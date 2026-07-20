//! The auth-deny anchor — a deny-by-default middleware seam (Story 3.8, AC #1).
//!
//! The layer refuses any path it does not explicitly recognize (`401`) — the seam the real auth
//! epic attaches to. Because v0.1.0 must be viewable before login exists, the walking-skeleton UI
//! surfaces and the liveness probe are an EXPLICIT, temporary allowlist; `/metrics` sits behind a
//! scrape Bearer token (FR43-44). When real user auth lands, the public UI moves behind sessions
//! and the seam keeps its shape.

use axum::extract::Request;
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::metrics;

/// Deny-by-default: allow the public walking-skeleton surfaces, gate `/metrics` on the scrape
/// token, refuse everything else.
pub async fn auth_deny(request: Request, next: Next) -> Response {
    metrics::HTTP_REQUESTS.inc();
    let path = request.uri().path();

    if is_public(path) {
        return next.run(request).await;
    }
    if path == "/metrics" {
        if scrape_authorized(&request) {
            return next.run(request).await;
        }
        return deny();
    }
    // The default arm — the seam real auth attaches to.
    deny()
}

/// The walking-skeleton public allowlist (temporary — real user auth is a later epic).
fn is_public(path: &str) -> bool {
    path == "/" || path == "/gap" || path == "/healthz" || path.starts_with("/assets/")
}

fn deny() -> Response {
    (StatusCode::UNAUTHORIZED, "authentication required").into_response()
}

/// The scrape is authorized only if `OPENCMDB_METRICS_TOKEN` is set (non-empty) and the request
/// carries it as `Authorization: Bearer <token>`. Unset token → no scrape (secure default).
fn scrape_authorized(request: &Request) -> bool {
    let Ok(expected) = std::env::var("OPENCMDB_METRICS_TOKEN") else {
        return false;
    };
    if expected.is_empty() {
        return false;
    }
    request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|header| header == format!("Bearer {expected}"))
}
