//! The `/metrics` anchor — raw `prometheus` and our own handler (D66: no middleware magic).
//!
//! A minimal, always-non-empty registry (a build-info gauge + a request counter) so a Prometheus
//! scrape has something to read from day one. The scrape auth (a Bearer token) is enforced by the
//! auth middleware, not here — this handler only encodes (FR43-44). Richer metrics attach later.

use std::sync::LazyLock;

use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use prometheus::{Encoder, IntCounter, IntGauge, Registry, TextEncoder};

/// The one registry the handler encodes. Our own, not the global default (explicit is better).
static REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

/// `opencmdb_build_info` — always `1`, so a scrape confirms the target is the expected build.
static BUILD_INFO: LazyLock<IntGauge> = LazyLock::new(|| {
    let gauge = IntGauge::new(
        "opencmdb_build_info",
        concat!(
            "opencmdb build info (version ",
            env!("CARGO_PKG_VERSION"),
            ")"
        ),
    )
    .expect("valid metric");
    REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("register build_info");
    gauge.set(1);
    gauge
});

/// `opencmdb_http_requests_total` — incremented by the auth middleware for every request.
pub static HTTP_REQUESTS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "opencmdb_http_requests_total",
        "Total HTTP requests seen by the auth layer",
    )
    .expect("valid metric");
    REGISTRY
        .register(Box::new(counter.clone()))
        .expect("register http_requests");
    counter
});

/// Force registration of the metrics at startup so `/metrics` is never empty on the first scrape.
pub fn init() {
    LazyLock::force(&BUILD_INFO);
    LazyLock::force(&HTTP_REQUESTS);
}

/// `GET /metrics` — gather and encode the registry in the Prometheus text exposition format. The
/// scrape token is enforced upstream by the auth middleware.
pub async fn handler() -> Response {
    let encoder = TextEncoder::new();
    let mut buffer = Vec::new();
    if let Err(error) = encoder.encode(&REGISTRY.gather(), &mut buffer) {
        tracing::error!(%error, "encoding metrics failed");
        return (StatusCode::INTERNAL_SERVER_ERROR, "metrics encode error").into_response();
    }
    ([(header::CONTENT_TYPE, encoder.format_type())], buffer).into_response()
}
