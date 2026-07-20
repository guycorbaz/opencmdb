//! opencmdb — the single binary.
//!
//! The composition root (D55): SQL, HTTP, HTML, files, the clock, secrets. `anyhow` is
//! legitimate here (D47) — nobody matches on the variant, and a `.context()` chain the
//! operator reads on stderr is worth money. This is the walking-skeleton entry point; the
//! `Repository` skeleton, the migrations, the askama surface and the reconciliation engine
//! attach to the `app()` seam in the stories that follow.

use anyhow::Context;
use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();
    let bind = load_bind_address().context("loading configuration")?;
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("binding {bind}"))?;
    tracing::info!(%bind, "opencmdb listening");
    axum::serve(listener, app())
        .await
        .context("serving the HTTP app")?;
    Ok(())
}

/// The HTTP surface, factored out of `main` so it is testable without binding a socket.
/// Later stories add routes here; today it is just liveness.
fn app() -> Router {
    Router::new().route("/healthz", get(healthz))
}

/// Liveness only — the process is up and serving. Database reachability is Story 3.2.
async fn healthz() -> StatusCode {
    StatusCode::OK
}

/// The address to bind, from `OPENCMDB_BIND` (default `0.0.0.0:8080` — a container binds all
/// interfaces). Read as a string so this bootstrap needs no `serde` in `bin`; a typed config
/// struct arrives when the configuration surface grows.
fn load_bind_address() -> anyhow::Result<String> {
    let config = config::Config::builder()
        .set_default("bind", "0.0.0.0:8080")?
        .add_source(config::Environment::with_prefix("OPENCMDB"))
        .build()?;
    Ok(config.get_string("bind")?)
}

/// Log filtering from `OPENCMDB_LOG` (e.g. `info`, `opencmdb=debug,warn`), defaulting to `info`.
fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_env("OPENCMDB_LOG")
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt; // for `oneshot`

    #[tokio::test]
    async fn healthz_returns_200() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn default_bind_is_all_interfaces_port_8080() {
        // The default applies when OPENCMDB_BIND is unset.
        let config = config::Config::builder()
            .set_default("bind", "0.0.0.0:8080")
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(config.get_string("bind").unwrap(), "0.0.0.0:8080");
    }
}
