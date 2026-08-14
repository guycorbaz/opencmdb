//! The auth-deny anchor — a deny-by-default middleware seam (Story 3.8, AC #1; story 6.1).
//!
//! The layer refuses any path it does not explicitly recognize (`401`). Since story 6.1, HTTP
//! Basic stands in the default arm where sessions will stand (Epic 19 is the closure): the
//! walking-skeleton public allowlist shrank to the liveness probe and the assets, and the UI
//! pages answer to the shared Basic pair — the visibility change arbitration 2′ priced in the
//! open. `/metrics` sits behind its scrape Bearer token (FR43-44), unchanged: one caller class,
//! one mechanism.
//!
//! ⚠️ Two mechanisms live near each other here and must never be conflated (story 6.1 AC5): the
//! `OPENCMDB_DOCUMENT_ENABLED` switch decides whether the write route EXISTS (it is in
//! `main::app`, not here, and it is NOT authentication); the Basic pair decides WHO MAY CALL.
//! Neither is a safety net for the other.

use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine as _;

use crate::metrics;
use crate::{AppConfig, BasicCredentials};

/// The challenge the configured-pair 401 carries. One realm for the whole origin — the
/// browser generalises the credential over the origin (story 6.1 §4, probe P4), so a second
/// protection space on the same origin is not a boundary the browser keeps; do not design one.
const BASIC_CHALLENGE: &str = "Basic realm=\"opencmdb\"";

/// Deny-by-default: allow the public surfaces (`/healthz`, `/assets/*`), gate `/metrics` on the
/// scrape token, and answer everything else to the HTTP Basic pair carried in [`AppConfig`]
/// (story 6.1). With the pair unconfigured the default arm refuses WITHOUT the challenge header
/// (arbitration 6): a challenge nothing can satisfy is an infinite browser dialog on every
/// unupgraded deployment.
pub async fn auth_deny(State(config): State<AppConfig>, request: Request, next: Next) -> Response {
    metrics::HTTP_REQUESTS.inc();
    let path = request.uri().path();

    if is_public(path) {
        return next.run(request).await;
    }
    // Exact match on purpose: `/metrics/` (trailing slash) is NOT this branch — it falls to
    // the default arm like any unknown path and is answered with the Basic challenge, so a
    // scraper must use the canonical path (measured at review).
    if path == "/metrics" {
        if scrape_authorized(&request) {
            return next.run(request).await;
        }
        // Never the Basic challenge on this branch (arbitration 6 / F14): a Prometheus does not
        // answer a Basic dialog, and the response bytes must not advertise a scheme this branch
        // does not accept — M5b pins that Basic is NOT accepted here.
        return deny();
    }
    // The default arm — HTTP Basic stands where sessions will stand (arbitration 2′).
    match &config.basic {
        Some(pair) if basic_authorized(&request, pair) => next.run(request).await,
        Some(_) => challenge(),
        None => deny(),
    }
}

/// The public allowlist, shrunk by story 6.1 to the login-free surface: the liveness probe
/// (a probe cannot authenticate) and the assets (CSS/JS/the vendored htmx — style, not data).
/// `/` and `/gap` left this list under arbitration 2′; adding a path back here IS the exposure
/// decision, and AC3's pinned shape exists so it cannot be taken by accident.
///
/// ⚠️ The match is on the RAW request path, by prefix: `/assets/../gap` is public-classified.
/// Containment was MEASURED at review, and rests on two facts — the only handler under the
/// prefix serves rust-embed's bundle (which cannot escape it), and the router does not
/// collapse `..` — so no gated content is reachable through the prefix today. The day a
/// normalizing hop or a second `/assets/`-prefixed route lands, this classification and the
/// router disagree: re-measure then.
fn is_public(path: &str) -> bool {
    path == "/healthz" || path.starts_with("/assets/")
}

/// A 401 WITHOUT the Basic challenge — the pair-unconfigured refusal (arbitration 6) and the
/// `/metrics` refusal (which must never advertise Basic).
///
/// ⚠️ RFC 9110 §15.5.2 says a 401 MUST carry `WWW-Authenticate`; this response violates that
/// MUST deliberately, and the trade is recorded at that strength — not as a UX preference
/// (code review): a challenge nothing can satisfy is an infinite browser dialog on every
/// unupgraded deployment, and a scheme `/metrics` does not accept must not be advertised.
fn deny() -> Response {
    (StatusCode::UNAUTHORIZED, "authentication required").into_response()
}

/// A 401 carrying `WWW-Authenticate: Basic realm="opencmdb"` — emitted only when the pair is
/// configured, and only from the default arm (arbitration 6).
fn challenge() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, BASIC_CHALLENGE)],
        "authentication required",
    )
        .into_response()
}

/// Whether the request carries the configured Basic pair. Every failure answers `false` — the
/// caller turns it into a 401 — and each branch is a measured decode-robustness case
/// (story 6.1 §3):
///
/// - **exactly one `Authorization` header**: right-then-wrong was measured reaching the handler
///   through `HeaderMap::get`'s first-value semantics, so two headers are refused outright;
/// - **scheme case-insensitive** on `Basic` (RFC 7235 §2.1, kept by RFC 9110 §11.1) — the
///   superseded draft caught `scrape_authorized` refusing lowercase `bearer`; that defect is
///   not copied here, and not "fixed" there either (registered, Epic 19);
/// - **garbage base64**, a decoded pair that is not UTF-8, and a pair with no colon are refused;
/// - the pair splits on the **FIRST colon only** (RFC 7617 §2: the user-id must not contain
///   one — enforced at boot — the password may);
/// - the comparison covers the WHOLE decoded pair, both halves. ⚠️ `==` on `String` is not
///   constant-time, and the `&&` SHORT-CIRCUITS: a user mismatch skips the password compare
///   entirely, so timing distinguishes *right user, wrong password* from *wrong user* — a
///   username-confirmation oracle, not just a byte-position leak. Both halves of the leak are
///   a stated limit (single-operator LAN product, TLS at the proxy), registered with Epic 19,
///   not silently "fixed" with a new dependency.
fn basic_authorized(request: &Request, pair: &BasicCredentials) -> bool {
    let mut values = request.headers().get_all(header::AUTHORIZATION).into_iter();
    let Some(value) = values.next() else {
        return false;
    };
    if values.next().is_some() {
        return false;
    }
    let Ok(header_value) = value.to_str() else {
        return false;
    };
    let Some((scheme, payload)) = header_value.split_once(' ') else {
        return false;
    };
    if !scheme.eq_ignore_ascii_case("basic") {
        return false;
    }
    let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(payload.trim()) else {
        return false;
    };
    let Ok(text) = String::from_utf8(decoded) else {
        return false;
    };
    let Some((user, password)) = text.split_once(':') else {
        return false;
    };
    user == pair.user && password == pair.password
}

/// The scrape is authorized only if `OPENCMDB_METRICS_TOKEN` is set (non-empty) and the request
/// carries it as `Authorization: Bearer <token>`. Unset token → no scrape (secure default).
///
/// ⚠️ Deliberately still an env read (story 6.1 arbitration 5 leaves `/metrics` untouched), and
/// deliberately case-SENSITIVE on `Bearer` — a recorded defect owned by Epic 19, not fixed here.
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

#[cfg(test)]
mod tests {
    use axum::body::Body;

    use super::*;

    /// AC3's pinned allowlist shape, path by path: exactly `/healthz` and `/assets/` remain.
    /// M5 (add `/` back) reds here — this pin is the mutation the story is for.
    #[test]
    fn is_public_is_exactly_healthz_and_assets() {
        assert!(is_public("/healthz"), "the liveness probe stays public");
        assert!(is_public("/assets/app.css"), "assets stay public");
        assert!(is_public("/assets/htmx.min.js"), "assets stay public");
        for gated in [
            "/",
            "/gap",
            "/metrics",
            "/document-all",
            "/healthzz",
            "/assets",
        ] {
            assert!(!is_public(gated), "{gated} must NOT be public");
        }
    }

    fn pair() -> BasicCredentials {
        BasicCredentials {
            user: "op".to_string(),
            password: "s3cret".to_string(),
        }
    }

    fn encoded(user_colon_password: &str) -> String {
        base64::engine::general_purpose::STANDARD.encode(user_colon_password)
    }

    fn request_with_authorization(values: &[&str]) -> Request {
        let mut builder = Request::builder().uri("/gap");
        for value in values {
            builder = builder.header(header::AUTHORIZATION, *value);
        }
        builder.body(Body::empty()).expect("a valid request")
    }

    #[test]
    fn the_configured_pair_authorizes() {
        let request = request_with_authorization(&[&format!("Basic {}", encoded("op:s3cret"))]);
        assert!(basic_authorized(&request, &pair()));
    }

    /// The scheme match is case-insensitive (RFC 7235 §2.1) — M12 reds here.
    #[test]
    fn a_mixed_case_scheme_with_the_correct_pair_is_accepted() {
        let request = request_with_authorization(&[&format!("bAsIc {}", encoded("op:s3cret"))]);
        assert!(basic_authorized(&request, &pair()));
    }

    /// The comparison covers BOTH halves — M10 (user-half-only comparison) reds these two.
    /// The natural both-halves-wrong test would leave a user-only comparison green, which is
    /// why the half-right pairs are the prescribed shape (gap-hunt F8).
    #[test]
    fn a_right_user_with_a_wrong_password_is_refused() {
        let request = request_with_authorization(&[&format!("Basic {}", encoded("op:wrong"))]);
        assert!(!basic_authorized(&request, &pair()));
    }

    #[test]
    fn a_wrong_user_with_the_right_password_is_refused() {
        let request = request_with_authorization(&[&format!("Basic {}", encoded("who:s3cret"))]);
        assert!(!basic_authorized(&request, &pair()));
    }

    #[test]
    fn garbage_base64_after_the_scheme_is_refused() {
        let request = request_with_authorization(&["Basic not/base64!!"]);
        assert!(!basic_authorized(&request, &pair()));
    }

    #[test]
    fn a_decoded_pair_with_no_colon_is_refused() {
        let request = request_with_authorization(&[&format!("Basic {}", encoded("no-colon"))]);
        assert!(!basic_authorized(&request, &pair()));
    }

    /// The split is on the FIRST colon only (RFC 7617 §2): a password containing a colon works.
    #[test]
    fn a_password_containing_a_colon_is_accepted() {
        let with_colon = BasicCredentials {
            user: "op".to_string(),
            password: "a:b".to_string(),
        };
        let request = request_with_authorization(&[&format!("Basic {}", encoded("op:a:b"))]);
        assert!(basic_authorized(&request, &with_colon));
    }

    /// Two `Authorization` headers are refused outright — even when the FIRST is the right one.
    /// `HeaderMap::get`'s first-value semantics were measured letting right-then-wrong through.
    #[test]
    fn two_authorization_headers_are_refused_even_right_then_wrong() {
        let right = format!("Basic {}", encoded("op:s3cret"));
        let wrong = format!("Basic {}", encoded("op:wrong"));
        let request = request_with_authorization(&[&right, &wrong]);
        assert!(!basic_authorized(&request, &pair()));
    }

    #[test]
    fn a_missing_header_is_refused() {
        let request = request_with_authorization(&[]);
        assert!(!basic_authorized(&request, &pair()));
    }

    /// Decoded bytes that are not UTF-8 are refused, not panicked on.
    #[test]
    fn a_non_utf8_decoded_pair_is_refused() {
        let payload = base64::engine::general_purpose::STANDARD.encode([0xFFu8, b':', 0xFE]);
        let request = request_with_authorization(&[&format!("Basic {payload}")]);
        assert!(!basic_authorized(&request, &pair()));
    }

    /// A Bearer credential is not a Basic one: the default arm must not accept `/metrics`'
    /// mechanism, symmetric to M5b's ban in the other direction.
    #[test]
    fn a_bearer_header_is_not_a_basic_credential() {
        let request = request_with_authorization(&["Bearer s3cret"]);
        assert!(!basic_authorized(&request, &pair()));
    }
}
