//! opencmdb — the single binary.
//!
//! The composition root (D55): SQL, HTTP, HTML, files, the clock, secrets. `anyhow` is
//! legitimate here (D47) — nobody matches on the variant, and a `.context()` chain the
//! operator reads on stderr is worth money. This is the walking-skeleton entry point; the
//! `Repository` skeleton, the askama surface and the reconciliation engine attach to the
//! `app()` seam in the stories that follow.

// Documentation is a project rule (CLAUDE.md): every public item carries a doc comment.
// `warn` for now, graduating to `-D missing_docs` once the tree is clean.
#![deny(missing_docs)]

mod arp_ping;
mod auth;
mod dburl;
mod fault_injection;
mod fixture_connector;
mod fixtures;
mod l1_runner;
mod metrics;
mod page;
/// Deterministic permutation sources for the arrival-order measurements (story 5.11b).
///
/// Test-only: it supports no production path, so it is gated rather than shipped. This `mod` line
/// is the ONE change story 5.11b makes to this file, and it carries no behaviour — the story's AC7
/// names the exception explicitly, because *"`main.rs` untouched"* was measured unsatisfiable.
#[cfg(test)]
mod permute;
mod repo;
mod resolver;
mod trap_gate;

// The i18n seam (D39/D66): user-facing strings resolve through `t!()` against `locales/`. EN is
// the fallback; the source YAML is greppable so the D65 vocabulary gate can later lint it.
rust_i18n::i18n!("locales", fallback = "en");

/// Serializes the DB-touching tests: they share one MariaDB (CI's service) and would otherwise
/// race on `migrate!` — two concurrent migrations both insert version 1 into `_sqlx_migrations`,
/// a duplicate-PRIMARY-KEY error. Held for each DB test's duration.
#[cfg(test)]
pub(crate) static DB_TEST_LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use opencmdb_core::Clock;
use opencmdb_core::observation::Timestamp;
use sqlx::MySqlPool;

/// The real clock. It reads the wall clock through `std::time` (a composition-root privilege)
/// and converts with `chrono::DateTime::from_timestamp` — NOT chrono's `clock` feature, which
/// must stay off so `opencmdb-core` cannot read the clock (D19).
struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after the Unix epoch");
        chrono::DateTime::from_timestamp(since_epoch.as_secs() as i64, since_epoch.subsec_nanos())
            .expect("a current instant is representable")
    }
}

#[tokio::main]
async fn main() -> std::process::ExitCode {
    // Hold the file-log writer guard for the whole process (dropping it flushes + stops the
    // writer). It must outlive `run`, and be dropped only after the fatal error below is logged.
    let _log_guard = init_tracing();
    match run().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            // Returning `Err` from `main` prints through `Termination`, on stderr, bypassing the
            // tracing subscriber entirely — so a crash-looping container left nothing in the
            // daily log files but a column of `opencmdb starting` lines, and the actual cause
            // was visible only to whoever happened to run in the foreground (issue #7).
            // `{:#}` keeps the whole `.context()` chain on one line.
            tracing::error!(error = format!("{error:#}"), "opencmdb failed to start");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Everything that can fail on the way up. Split out of `main` so a failure is logged through
/// `tracing` — reaching the log files — instead of being printed past the subscriber.
async fn run() -> anyhow::Result<()> {
    // Select the UI locale (default `en`); user-facing strings resolve through `t!()`.
    let locale = std::env::var("OPENCMDB_LOCALE").unwrap_or_else(|_| "en".to_string());
    rust_i18n::set_locale(&locale);
    // Register the metrics so `/metrics` is non-empty on the first scrape.
    metrics::init();
    // The one place the wall clock is read; the domain receives Timestamps, never a clock.
    let clock = SystemClock;
    tracing::info!(started_at = %clock.now(), "opencmdb starting");
    let bind = load_bind_address().context("loading configuration")?;
    // Discrete DATABASE_* variables are the documented path; DATABASE_URL is the deprecated
    // fallback that keeps CI and existing deployments working (issue #6).
    let (database_url, source) = dburl::from_env(|key| std::env::var(key).ok())
        .map_err(anyhow::Error::msg)
        .context("loading the database configuration")?;
    if source == dburl::Source::Url {
        tracing::warn!(
            "DATABASE_URL is deprecated — prefer DATABASE_HOST, DATABASE_PORT, DATABASE_NAME, \
             DATABASE_USERNAME and DATABASE_PASSWORD, which need no manual percent-encoding"
        );
    }

    let pool = match MySqlPool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(error) => {
            // `1045 Access denied` is the costliest error in this product's deployment story:
            // it points at the password, which is usually the one thing that is right (issue #5).
            if let Some(hint) = dburl::explain_connect_error(&error) {
                tracing::error!("{hint}");
            }
            return Err(anyhow::Error::new(error).context("connecting to MariaDB"));
        }
    };
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("applying database migrations")?;
    tracing::info!("database connected and migrations applied");

    // Optional one-shot startup scan: the real ARP/ping connector (Story 3.5) pings a declared
    // subnet and ingests observations, so the page shows genuinely observed state. Unset → the
    // page renders the declared side only. The periodic scheduler (FR6) is a later story.
    if let Ok(cidr) = std::env::var("OPENCMDB_SCAN_CIDR") {
        spawn_startup_scan(database_url.clone(), clock.now(), cidr);
    }

    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("binding {bind}"))?;
    tracing::info!(%bind, "opencmdb listening");
    axum::serve(listener, app(pool))
        .await
        .context("serving the HTTP app")?;
    Ok(())
}

/// The HTTP surface, factored out of `main` so it is testable without binding a socket. The
/// database pool is carried in axum state.
fn app(pool: MySqlPool) -> Router {
    Router::new()
        .route("/", get(page::index))
        .route("/gap", get(page::gap_fragment))
        .route("/assets/{*path}", get(page::asset))
        .route("/metrics", get(metrics::handler))
        .route("/healthz", get(healthz))
        // Deny-by-default seam over every route (Story 3.8): the public UI is allowlisted,
        // `/metrics` sits behind the scrape token, everything else is refused.
        .layer(axum::middleware::from_fn(auth::auth_deny))
        .with_state(pool)
}

/// Run a one-shot scan off the request path: build the ARP/ping connector for `cidr`, poll it, and
/// ingest each answered host as an immutable observation (FR11). Best-effort — a bad CIDR or a scan
/// error is logged, never fatal; the page still serves whatever is already persisted.
///
/// It runs on a DEDICATED thread with its own current-thread runtime and its own pool. That is
/// deliberate: `Connector::poll` holds a `&mut dyn ObservationSink` across an await, so its future
/// is not `Send` and cannot be `tokio::spawn`ed onto the multi-thread runtime (Story 2.3 left the
/// scheduler's Send story for later). `block_on` on a single-thread runtime imposes no `Send`
/// bound, and a fresh pool avoids sharing connections across runtimes. The periodic scheduler
/// (FR6) will supersede this.
fn spawn_startup_scan(database_url: String, now: Timestamp, cidr: String) {
    use opencmdb_core::connector::{Connector, VecSink};
    use opencmdb_core::observation::{ConnectorId, L2DomainId, Scope, VantageId};
    use tokio_util::sync::CancellationToken;
    use uuid::Uuid;

    use crate::arp_ping::ArpPingConnector;

    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                tracing::error!(%error, "could not build the scan runtime");
                return;
            }
        };

        runtime.block_on(async move {
            let scope = Scope {
                l2_domain: L2DomainId::from_uuid(Uuid::nil()),
                vantage: VantageId::from_uuid(Uuid::nil()),
            };
            let connector_id = ConnectorId::from_uuid(Uuid::now_v7());
            let connector = match ArpPingConnector::from_cidr(connector_id, scope, &cidr) {
                Ok(connector) => connector,
                Err(error) => {
                    tracing::error!(%error, %cidr, "invalid OPENCMDB_SCAN_CIDR — skipping scan");
                    return;
                }
            };
            // How many probes may be in flight at once. A politeness bound, not a throughput
            // one: the scan is I/O-bound and runs on a single thread either way.
            let concurrency = std::env::var("OPENCMDB_SCAN_CONCURRENCY")
                .ok()
                .and_then(|raw| raw.parse::<usize>().ok())
                .unwrap_or(crate::arp_ping::DEFAULT_CONCURRENCY);
            // How long one probe waits for its reply. The knob that decides what the scan MISSES:
            // one probe is sent per host and there is no retry yet, so a device slower than this
            // is simply recorded as absent. Raise it on a congested or wireless network.
            let timeout_ms = std::env::var("OPENCMDB_SCAN_TIMEOUT_MS")
                .ok()
                .and_then(|raw| raw.parse::<u64>().ok())
                .filter(|ms| *ms > 0)
                .unwrap_or(crate::arp_ping::DEFAULT_TIMEOUT_MS);
            let mut connector = connector
                .with_concurrency(concurrency)
                .with_timeout(std::time::Duration::from_millis(timeout_ms));

            tracing::info!(%cidr, concurrency, timeout_ms, "startup scan: pinging subnet");
            let mut sink = VecSink::default();
            if let Err(error) = connector
                .poll(now, &mut sink, CancellationToken::new())
                .await
            {
                tracing::warn!(?error, "startup scan failed");
                return;
            }

            let pool = match MySqlPool::connect(&database_url).await {
                Ok(pool) => pool,
                Err(error) => {
                    tracing::warn!(%error, "startup scan: could not connect to ingest");
                    return;
                }
            };
            ingest_and_resolve(&pool, sink.observations).await;
        });
    });
}

/// Write the scan's observations, then run the identity pass over that same slice (story 5.14).
///
/// **Two transaction units, not one, and the reason is D34 §2** — *"everything emitted before it is
/// still true"*. An observation is immutable and independently true (FR11), so a resolution failure
/// must not take the observations down with it. It is also not a choice the current shape offers:
/// the observations are already written one `transact` EACH (below), so there is no single unit for
/// the pass to join.
///
/// The pass itself is ONE unit, which D21 requires: *"an identity decision is NEVER split across two
/// transactions"*.
///
/// Best-effort like the scan itself, but the refusal is logged at `error!` **by name** —
/// `InstantRegressed` and `ContradictoryObservation` are the two a real network can produce, and a
/// silent skip would make the reach counter lie by omission.
async fn ingest_and_resolve(
    pool: &MySqlPool,
    observations: Vec<opencmdb_core::observation::Observation>,
) -> usize {
    use opencmdb_core::repo::WriteRepository;

    use crate::repo::{MariaRepository, classify, insert_observation};

    let repo = MariaRepository::new(pool.clone());
    let mut ingested = 0usize;
    for observation in observations.iter().cloned() {
        let result = repo
            .transact(move |unit| {
                let observation = observation.clone();
                Box::pin(async move {
                    insert_observation(unit.executor(), &observation)
                        .await
                        .map_err(classify)
                })
            })
            .await;
        match result {
            Ok(()) => ingested += 1,
            Err(error) => tracing::warn!(?error, "ingesting a scanned observation failed"),
        }
    }
    tracing::info!(ingested, "startup scan complete");

    let slice = observations.clone();
    match repo
        .transact(move |unit| {
            let slice = slice.clone();
            Box::pin(async move { crate::resolver::resolve(unit.executor(), &slice).await })
        })
        .await
    {
        Ok(resolution) => tracing::info!(
            links_written = resolution.links_written,
            abstentions = resolution.abstentions,
            interfaces_minted = resolution.interfaces_minted,
            "identity pass complete"
        ),
        // 🔴 `error!` with the refusal NAMED — never a silent skip.
        Err(error) => tracing::error!(refusal = ?error, "the identity pass refused this slice"),
    }
    ingested
}

/// Readiness: `200 OK` when the database answers a trivial query, `503` when it does not.
async fn healthz(State(pool): State<MySqlPool>) -> StatusCode {
    // Static SQL — no `AssertSqlSafe` needed (that is for dynamic queries).
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => StatusCode::OK,
        Err(error) => {
            tracing::warn!(%error, "healthz: database unreachable");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

/// The address to bind, from `OPENCMDB_BIND` (default `0.0.0.0:8080` — a container binds all
/// interfaces). Read as a string so this bootstrap needs no `serde` in `bin`.
fn load_bind_address() -> anyhow::Result<String> {
    let config = config::Config::builder()
        .set_default("bind", "0.0.0.0:8080")?
        .add_source(config::Environment::with_prefix("OPENCMDB"))
        .build()?;
    Ok(config.get_string("bind")?)
}

/// Configure tracing: always to stdout (so `docker logs` works), and — when `OPENCMDB_LOG_DIR`
/// is set — additionally to a DAILY-rotating file (`opencmdb.YYYY-MM-DD.log`) for on-NAS
/// debugging. Level filtering comes from `OPENCMDB_LOG` (e.g. `info`, `opencmdb=debug,warn`),
/// defaulting to `info`. Returns the non-blocking writer's guard, which must be held for the
/// process's lifetime, or `None` when only stdout is active.
///
/// File logging degrades gracefully: if the directory cannot be written, it logs a warning to
/// stderr and continues with stdout only — a missing/unwritable log mount never crashes startup.
#[must_use]
fn init_tracing() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{EnvFilter, Layer, Registry, fmt};

    let directives = std::env::var("OPENCMDB_LOG").unwrap_or_else(|_| "info".to_string());
    let mut layers: Vec<Box<dyn Layer<Registry> + Send + Sync>> = Vec::new();

    // Always to stdout, so `docker logs` works.
    layers.push(
        fmt::layer()
            .with_filter(EnvFilter::new(&directives))
            .boxed(),
    );

    // Additionally to a daily-rotating file when `OPENCMDB_LOG_DIR` is set and writable.
    let guard = match build_file_writer() {
        Some((writer, guard)) => {
            layers.push(
                // No ANSI colour codes in files — they are read as plain text.
                fmt::layer()
                    .with_ansi(false)
                    .with_writer(writer)
                    .with_filter(EnvFilter::new(&directives))
                    .boxed(),
            );
            Some(guard)
        }
        None => None,
    };

    Registry::default().with(layers).init();
    guard
}

/// A non-blocking, DAILY-rotating file writer from `OPENCMDB_LOG_DIR` (retention
/// `OPENCMDB_LOG_RETENTION` days, default 14). `None` — stdout only — when the dir is unset or
/// unwritable; an unwritable log mount logs to stderr and never crashes startup.
fn build_file_writer() -> Option<(
    tracing_appender::non_blocking::NonBlocking,
    tracing_appender::non_blocking::WorkerGuard,
)> {
    let dir = std::env::var("OPENCMDB_LOG_DIR").ok()?;
    if dir.is_empty() {
        return None;
    }
    let retention: usize = std::env::var("OPENCMDB_LOG_RETENTION")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(14);

    match tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("opencmdb")
        .filename_suffix("log")
        .max_log_files(retention)
        .build(&dir)
    {
        Ok(appender) => Some(tracing_appender::non_blocking(appender)),
        Err(error) => {
            eprintln!("opencmdb: file logging disabled — cannot use {dir:?}: {error}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt; // for `oneshot`

    /// Readiness against a real MariaDB. Gated on `DATABASE_URL`: runs in CI (the MariaDB
    /// service, Story 1.5) and locally against a `mariadb:10.11.11` container; no-ops otherwise.
    #[tokio::test]
    async fn healthz_reports_200_when_database_answers() {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping healthz DB test: DATABASE_URL unset");
            return;
        };
        let _guard = crate::DB_TEST_LOCK.lock().await; // serialize DB tests (see the static)
        let pool = MySqlPool::connect(&url).await.expect("connect to MariaDB");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("run migrations");
        let response = app(pool)
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

    /// End-to-end: seed a declared entity and a linked-but-drifting observation, then `GET /`
    /// and assert the rendered page carries the drift gap. Gated on `DATABASE_URL`, serialized.
    #[tokio::test]
    async fn index_renders_the_real_gap() {
        use opencmdb_core::observation::{
            ConnectorId, Fact, HostnameSource, L2DomainId, ObsId, Observation, Scope, VantageId,
        };
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping index DB test: DATABASE_URL unset");
            return;
        };
        let _guard = crate::DB_TEST_LOCK.lock().await;
        // Do not let a stray env var steer the perimeter choice.
        unsafe { std::env::remove_var("OPENCMDB_ENTITY_IPV4") };
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        sqlx::query("DELETE FROM declared_attribute")
            .execute(&pool)
            .await
            .expect("clean declared");
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
            .expect("clean observations");

        // Declared: entity 192.0.2.10 named `nas`.
        let entity = "00000000-0000-0000-0000-0000000000aa";
        repo::insert_declared_attribute(&pool, entity, "ipv4", "192.0.2.10")
            .await
            .expect("declare ipv4");
        repo::insert_declared_attribute(&pool, entity, "hostname", "nas")
            .await
            .expect("declare hostname");
        // Observed: same IP, a DIFFERENT hostname → a drift on `hostname`.
        let observation = Observation {
            obs_id: ObsId::from_uuid(uuid::Uuid::now_v7()),
            connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()),
            observed_at: chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
            scope: Scope {
                l2_domain: L2DomainId::from_uuid(uuid::Uuid::nil()),
                vantage: VantageId::from_uuid(uuid::Uuid::nil()),
            },
            facts: vec![
                Fact::IpV4 {
                    addr: "192.0.2.10".parse().unwrap(),
                },
                Fact::Hostname {
                    name: "intruder".into(),
                    source: HostnameSource::Dns,
                },
            ],
            raw: None,
        };
        repo::insert_observation(&pool, &observation)
            .await
            .expect("ingest observation");

        let response = app(pool)
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("192.0.2.10"), "renders the entity");
        assert!(html.contains("nas"), "renders the declared hostname");
        assert!(html.contains("intruder"), "renders the observed hostname");
    }

    /// The auth-deny seam, exercised without a database (a lazy pool never connects because these
    /// routes issue no query). Deny-by-default holds; `/metrics` sits behind the scrape token; the
    /// public allowlist stays reachable.
    #[tokio::test]
    async fn auth_denies_by_default_and_gates_metrics() {
        metrics::init();
        let pool =
            MySqlPool::connect_lazy("mysql://root:x@127.0.0.1:3306/none").expect("lazy pool");

        let get = |uri: &str, bearer: Option<&str>| {
            let mut builder = Request::builder().uri(uri.to_string());
            if let Some(token) = bearer {
                builder =
                    builder.header(axum::http::header::AUTHORIZATION, format!("Bearer {token}"));
            }
            let request = builder.body(Body::empty()).unwrap();
            app(pool.clone()).oneshot(request)
        };

        // No scrape token configured → `/metrics` is closed; an un-allowlisted path is denied.
        unsafe { std::env::remove_var("OPENCMDB_METRICS_TOKEN") };
        assert_eq!(
            get("/metrics", None).await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            get("/admin", None).await.unwrap().status(),
            StatusCode::UNAUTHORIZED,
            "deny by default"
        );
        // A public walking-skeleton surface stays reachable (no DB query).
        assert_eq!(
            get("/assets/app.css", None).await.unwrap().status(),
            StatusCode::OK
        );

        // With a token, the correct Bearer scrapes; a wrong one is refused.
        unsafe { std::env::set_var("OPENCMDB_METRICS_TOKEN", "s3cret") };
        let ok = get("/metrics", Some("s3cret")).await.unwrap();
        assert_eq!(ok.status(), StatusCode::OK);
        let body = axum::body::to_bytes(ok.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(
            String::from_utf8_lossy(&body).contains("opencmdb_build_info"),
            "the registry is non-empty"
        );
        assert_eq!(
            get("/metrics", Some("wrong")).await.unwrap().status(),
            StatusCode::UNAUTHORIZED
        );
        unsafe { std::env::remove_var("OPENCMDB_METRICS_TOKEN") };
    }

    /// The i18n `t!()` seam resolves EN and FR. Uses an explicit `locale =` so it never mutates the
    /// global locale (no race with rendering tests).
    #[test]
    fn i18n_resolves_en_and_fr() {
        assert_eq!(rust_i18n::t!("page.the_gap", locale = "en"), "The gap");
        assert_eq!(rust_i18n::t!("page.the_gap", locale = "fr"), "L'écart");
        assert_eq!(
            rust_i18n::t!("cause.out_of_perimeter", locale = "fr"),
            "Hors du périmètre"
        );
    }

    /// Wipe every table the identity pass touches, children before parents.
    async fn clean(pool: &MySqlPool) {
        for statement in [
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM interface",
            "DELETE FROM observation_record",
        ] {
            sqlx::query(sqlx::AssertSqlSafe(statement))
                .execute(pool)
                .await
                .expect("clean");
        }
    }

    /// The facts `ArpPingConnector` emits, verbatim: `{IpV4, Rtt}` — its `poll` builds exactly this
    /// vector (`arp_ping.rs:177`) and its `Capabilities` declares exactly these two `FactKind`s.
    fn scanned(n: u8) -> opencmdb_core::observation::Observation {
        use opencmdb_core::observation::{
            ConnectorId, Fact, L2DomainId, ObsId, Observation, Scope, VantageId,
        };
        Observation {
            obs_id: ObsId::from_uuid(uuid::Uuid::now_v7()),
            connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()),
            observed_at: chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
            scope: Scope {
                l2_domain: L2DomainId::from_uuid(uuid::Uuid::nil()),
                vantage: VantageId::from_uuid(uuid::Uuid::nil()),
            },
            facts: vec![
                Fact::IpV4 {
                    addr: format!("192.0.2.{n}").parse().unwrap(),
                },
                Fact::Rtt { millis: 3 },
            ],
            raw: None,
        }
    }

    /// 🔴 EXPERIMENT A — what the SHIPPED startup path actually produces.
    #[tokio::test]
    async fn the_startup_path_produces_a_reach_counter() {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            return;
        };
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        clean(&pool).await;

        let slice: Vec<_> = (10..=14).map(scanned).collect();
        let ingested = ingest_and_resolve(&pool, slice).await;
        assert_eq!(ingested, 5);

        let rows = repo::load_current_engine_reach(&pool)
            .await
            .expect("read reach");
        let evaluated = rows.iter().filter(|r| r.interface_id.is_some()).count();
        let not_evaluated = rows.len() - evaluated;
        let causes: std::collections::BTreeSet<_> = rows
            .iter()
            .filter_map(|r| r.abstention_cause.clone())
            .collect();

        // 🔴 MEASURED, and it is the finding: the SHIPPED connector emits `{IpV4, Rtt}` and no MAC,
        // so `join` produces no L1 key and EVERY scanned observation falls to the tail abstention
        // loop. Wiring the pass makes the not-evaluated half real and leaves the evaluated half a
        // structural zero.
        assert_eq!(evaluated, 0, "the shipped scan places nothing — it carries no MAC");
        assert_eq!(not_evaluated, 5);
        assert_eq!(
            causes,
            std::collections::BTreeSet::from(["absence_of_proof".to_string()])
        );

        // The counter GROWS with every scan: a second pass over a fresh slice of the same five
        // hosts mints five new `obs_id`s and therefore five more CURRENT abstention links.
        let again: Vec<_> = (10..=14).map(scanned).collect();
        ingest_and_resolve(&pool, again).await;
        let after = repo::load_current_engine_reach(&pool)
            .await
            .expect("read reach again");
        assert_eq!(
            after.len(),
            10,
            "5 → 10 for the SAME five hosts: the counter measures uptime, not reach"
        );
    }

    /// The CURRENT-only filter (§6's ⚠️), given a guard of its own.
    #[tokio::test]
    async fn a_superseded_link_is_not_counted() {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            return;
        };
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        clean(&pool).await;

        let one = scanned(20);
        ingest_and_resolve(&pool, vec![one.clone()]).await;
        // Supersede it by hand: stamp `valid_to` and drop it out of the current key.
        sqlx::query(sqlx::AssertSqlSafe(
            "UPDATE identity_link SET valid_to = '2024-01-01 00:00:00.000000', \
             current_subject = NULL WHERE decided_by = 'ENGINE'",
        ))
        .execute(&pool)
        .await
        .expect("supersede");

        let rows = repo::load_current_engine_reach(&pool)
            .await
            .expect("read reach");
        assert!(
            rows.is_empty(),
            "a superseded row must not be counted, or the number ages with every re-scan; got {rows:?}"
        );
    }

    /// §5 — the TWO-unit boundary: a refused pass leaves the observations standing.
    #[tokio::test]
    async fn a_refused_pass_does_not_take_the_observations_down() {
        use opencmdb_core::observation::Fact;
        let Ok(url) = std::env::var("DATABASE_URL") else {
            return;
        };
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        clean(&pool).await;

        // One `obs_id`, two different contents — `ContradictoryObservation`.
        let a = scanned(40);
        let mut b = a.clone();
        b.facts = vec![Fact::Rtt { millis: 99 }];
        ingest_and_resolve(&pool, vec![a, b]).await;

        let (observations,): (i64,) = sqlx::query_as(sqlx::AssertSqlSafe(
            "SELECT COUNT(*) FROM observation_record",
        ))
        .fetch_one(&pool)
        .await
        .expect("count");
        assert_eq!(
            observations, 1,
            "TWO units: the observation the scan emitted survives a refused pass (FR11, D34 §2)"
        );
        let rows = repo::load_current_engine_reach(&pool).await.expect("reach");
        assert!(rows.is_empty(), "and the pass wrote nothing at all");
    }

    /// AC7 — `Ambiguous` and `link_candidate` are UNREACHABLE. **Epic 6 makes this fall.**
    #[tokio::test]
    async fn ambiguous_and_link_candidate_are_unreachable_until_epic_6() {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            return;
        };
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        clean(&pool).await;
        ingest_and_resolve(&pool, (30..=34).map(scanned).collect()).await;

        let rows = repo::load_current_engine_reach(&pool)
            .await
            .expect("read reach");
        assert!(
            !rows
                .iter()
                .any(|r| r.abstention_cause.as_deref() == Some("ambiguous")),
            "L1 emits no Supports and no Opposes, so it cannot conclude Ambiguous — \
             EPIC 6 owns the producer, and this assertion is what will fall when it arrives"
        );
        let (candidates,): (i64,) =
            sqlx::query_as(sqlx::AssertSqlSafe("SELECT COUNT(*) FROM link_candidate"))
                .fetch_one(&pool)
                .await
                .expect("count candidates");
        assert_eq!(
            candidates, 0,
            "nothing fills candidates_for_link — EPIC 6 owns filling it"
        );
    }

    #[test]
    fn default_bind_is_all_interfaces_port_8080() {
        let config = config::Config::builder()
            .set_default("bind", "0.0.0.0:8080")
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(config.get_string("bind").unwrap(), "0.0.0.0:8080");
    }
}
