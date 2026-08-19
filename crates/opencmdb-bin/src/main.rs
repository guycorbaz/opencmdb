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
mod document;
mod example_data;
mod example_screens;
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
mod scan_pass;
mod screens;
mod state_vocabulary;
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

/// The shared HTTP Basic credential (story 6.1, arbitration 2′): ONE pair, read from
/// `OPENCMDB_BASIC_USER` / `OPENCMDB_BASIC_PASSWORD` at boot.
///
/// It authenticates a CALLER, not a person — no users, no sessions, no revocation short of
/// changing the variable; everyone holding it is the same principal. Real, and crude: Basic
/// sends the pair base64-encoded on EVERY request, so its confidentiality is TLS's business,
/// which this product does not terminate (a reverse proxy does — `architecture.md:168`). And it
/// does NOT close CSRF: the browser attaches the credential by ambient authority, to a
/// cross-site form POST included — story 6.2 owns CSRF protection, the story where the route
/// first has an effect to forge. Epic 19 (real sessions) is the closure.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct BasicCredentials {
    /// The user half. ASCII, and colon-free (RFC 7617 §2: the user-id must not contain a
    /// colon) — both refused by [`AppConfig::from_env`] at boot, never silently at request time.
    pub(crate) user: String,
    /// The password half. ASCII (refused at boot otherwise); it MAY contain a colon — the
    /// decoder splits on the FIRST colon only (RFC 7617 §2).
    pub(crate) password: String,
}

// Hand-written so a debug-printed config can never leak the password into a log line.
impl std::fmt::Debug for BasicCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BasicCredentials")
            .field("user", &self.user)
            .field("password", &"<redacted>")
            .finish()
    }
}

/// The HTTP surface's configuration — a PARAMETER of [`app`], never an env read inside it
/// (story 6.1, arbitration 5). Tests construct it directly and mutate no env var; only
/// [`run`] calls [`AppConfig::from_env`].
///
/// Two mechanisms, two questions, never conflated (story 6.1 §3 / AC5):
/// - `document_enabled` answers *is the feature configured?* — without it the write route is
///   not in the `Router`. It says NOTHING about who calls: it is defence in depth and an
///   off-by-default posture, and it is NOT authentication.
/// - `basic` answers *who may call — and who may read?* — HTTP Basic, enforced in
///   `auth_deny`'s default arm over every non-public path. This is the security mechanism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppConfig {
    /// `OPENCMDB_DOCUMENT_ENABLED`: whether `POST /document-all` is registered at all.
    /// *Configured vs. unconfigured* — nothing about callers.
    pub(crate) document_enabled: bool,
    /// The Basic pair, or `None` when unconfigured — in which case every non-public path
    /// refuses (401) WITHOUT the challenge header (arbitration 6): a challenge nothing can
    /// satisfy is an infinite browser dialog on every unupgraded deployment.
    pub(crate) basic: Option<BasicCredentials>,
    /// `OPENCMDB_SCAN_CIDR`: the subnet the startup scan sweeps, and — since story 6b.2 — the
    /// **perimeter the shell displays** in its navigation footer, as the reference mock does.
    ///
    /// `None` when unset: the product then scans nothing and the footer says so rather than
    /// showing an empty value. ⚠️ It enters here as a PARAMETER rather than being read at the
    /// point of use (story 6.1's rule — not one test in this crate mutates an env var), and it
    /// is **not validated here**: an invalid CIDR is refused by the scan with a named error, and
    /// displaying what the operator configured is more honest than displaying nothing.
    pub(crate) scan_cidr: Option<String>,
}

/// Whether a configured value carries at least one character an operator could SEE.
///
/// 🔴 `trim().is_empty()` is not enough, and the code review measured it rather than argued it:
/// `"\u{200B}".trim().is_empty()` is **`false`** in Rust, so a zero-width space survived the
/// emptiness filter. The navigation footer then rendered *"Perimeter "* followed by nothing
/// visible — indistinguishable from a blank value — instead of the honest *"not configured"*
/// the surface promises. The defect the filter exists to prevent, reached through a channel it
/// did not consider.
///
/// ⚠️ **Its limit, stated rather than implied**: [`is_invisible`] is an ENUMERATION of Unicode's
/// default-ignorable ranges, and *an enumeration cannot claim the completeness of a property*
/// (story 5.12, where the same class was widened once and deliberately not to exhaustion). This
/// is a tripwire against the character a copy-paste carries in, never a barrier against one
/// someone chose.
fn carries_a_visible_glyph(value: &str) -> bool {
    value
        .chars()
        .any(|glyph| !glyph.is_whitespace() && !glyph.is_control() && !is_invisible(glyph))
}

/// Unicode code points that occupy no visible space, beyond whitespace and control characters.
///
/// The ranges are the default-ignorable ones a text editor or a web form will silently carry:
/// the zero-width family, the bidirectional controls, the word joiners, the variation selectors
/// and the byte-order mark. See [`carries_a_visible_glyph`] for why this is a tripwire.
fn is_invisible(glyph: char) -> bool {
    matches!(
        glyph,
        '\u{00AD}'
            | '\u{180E}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206F}'
            | '\u{FE00}'..='\u{FE0F}'
            | '\u{FEFF}'
            | '\u{E0000}'..='\u{E007F}'
    )
}

/// Why [`AppConfig::from_env`] refuses a configuration — at boot, with the variable named,
/// never at request time silently (story 6.1 §3). The superseded draft measured the trap this
/// exists for: a credential containing a non-ASCII byte refuses everyone, permanently, with no
/// diagnostic — which matters for a French operator setting `sécret`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AppConfigError {
    /// The switch carries a value the product does not recognise (accepted: unset, empty,
    /// `0`/`false`, `1`/`true` — case-insensitive). Refused by name rather than silently
    /// disabled, so a typo is a boot error and not a mysteriously missing route.
    UnrecognisedSwitch {
        /// The value found in `OPENCMDB_DOCUMENT_ENABLED`.
        value: String,
    },
    /// Exactly one half of the Basic pair is set (empty counts as unset, on
    /// `scrape_authorized`'s unset-OR-empty precedent). A half-configured pair is a
    /// misconfiguration, not a choice.
    HalfConfiguredPair {
        /// The variable that is missing or empty.
        missing: &'static str,
    },
    /// A credential half carries a byte outside ASCII. Basic inherits the trap through the
    /// base64 of `user:password` — RFC 7617 records that the original scheme *failed to
    /// specify* the charset, so a non-ASCII pair authenticates nobody, silently.
    NonAsciiCredential {
        /// The variable carrying the non-ASCII byte.
        var: &'static str,
    },
    /// A credential half carries an ASCII CONTROL character — the trailing newline of an
    /// env-file `echo`, a tab, a carriage return. It passes `is_ascii()`, no browser Basic
    /// dialog can type it, so such a credential authenticates nobody, silently — the same trap
    /// class as [`Self::NonAsciiCredential`], found by this story's code review.
    ControlCharacterInCredential {
        /// The variable carrying the control character.
        var: &'static str,
    },
    /// The user half contains a colon, which RFC 7617 §2 forbids: the decoder splits the pair
    /// on the FIRST colon, so such a user could never match and the deployment would refuse
    /// everyone with no diagnostic.
    ColonInUser,
}

impl std::fmt::Display for AppConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnrecognisedSwitch { value } => write!(
                f,
                "OPENCMDB_DOCUMENT_ENABLED={value:?} is not recognised — use 1/true or 0/false"
            ),
            Self::HalfConfiguredPair { missing } => write!(
                f,
                "the Basic pair is half-configured: {missing} is unset or empty — set both \
                 OPENCMDB_BASIC_USER and OPENCMDB_BASIC_PASSWORD, or neither"
            ),
            Self::NonAsciiCredential { var } => write!(
                f,
                "{var} contains a non-ASCII byte — Basic authentication has no reliable \
                 charset (RFC 7617), so such a credential would authenticate nobody"
            ),
            Self::ControlCharacterInCredential { var } => write!(
                f,
                "{var} contains a control character (a trailing newline from an env file?) — \
                 no browser dialog can type it, so such a credential would authenticate nobody"
            ),
            Self::ColonInUser => write!(
                f,
                "OPENCMDB_BASIC_USER contains a colon, which RFC 7617 forbids in the user-id — \
                 such a user could never authenticate"
            ),
        }
    }
}

impl std::error::Error for AppConfigError {}

impl AppConfig {
    /// Parse and VALIDATE the configuration from an environment lookup. Pure — the lookup is a
    /// parameter (on `dburl::from_env`'s precedent), so tests drive it without mutating any env
    /// var. ⚠️ The production call-site is [`run`], which no test drives: the uncovered region
    /// is one call and one `?` (stated on story 5.14's precedent — recording an unavoidable
    /// green is honest only with its extent named).
    ///
    /// # Errors
    ///
    /// Every refusal is a boot error with the variable named — see [`AppConfigError`].
    pub(crate) fn from_env(
        lookup: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, AppConfigError> {
        let document_enabled = match lookup("OPENCMDB_DOCUMENT_ENABLED") {
            None => false,
            Some(value) => match value.trim().to_ascii_lowercase().as_str() {
                "" | "0" | "false" => false,
                "1" | "true" => true,
                _ => return Err(AppConfigError::UnrecognisedSwitch { value }),
            },
        };
        // Empty counts as unset — `scrape_authorized`'s precedent (unset OR empty, both closed).
        // Empty counts as unset here too: an operator who blanks the variable to disable the
        // scan must not then see an empty perimeter rendered as if it were a value.
        let scan_cidr = lookup("OPENCMDB_SCAN_CIDR").filter(|value| carries_a_visible_glyph(value));
        let user = lookup("OPENCMDB_BASIC_USER").filter(|value| !value.is_empty());
        let password = lookup("OPENCMDB_BASIC_PASSWORD").filter(|value| !value.is_empty());
        let basic = match (user, password) {
            (None, None) => None,
            (Some(_), None) => {
                return Err(AppConfigError::HalfConfiguredPair {
                    missing: "OPENCMDB_BASIC_PASSWORD",
                });
            }
            (None, Some(_)) => {
                return Err(AppConfigError::HalfConfiguredPair {
                    missing: "OPENCMDB_BASIC_USER",
                });
            }
            (Some(user), Some(password)) => {
                if !user.is_ascii() {
                    return Err(AppConfigError::NonAsciiCredential {
                        var: "OPENCMDB_BASIC_USER",
                    });
                }
                if !password.is_ascii() {
                    return Err(AppConfigError::NonAsciiCredential {
                        var: "OPENCMDB_BASIC_PASSWORD",
                    });
                }
                if user.chars().any(|c| c.is_ascii_control()) {
                    return Err(AppConfigError::ControlCharacterInCredential {
                        var: "OPENCMDB_BASIC_USER",
                    });
                }
                if password.chars().any(|c| c.is_ascii_control()) {
                    return Err(AppConfigError::ControlCharacterInCredential {
                        var: "OPENCMDB_BASIC_PASSWORD",
                    });
                }
                if user.contains(':') {
                    return Err(AppConfigError::ColonInUser);
                }
                Some(BasicCredentials { user, password })
            }
        };
        Ok(Self {
            document_enabled,
            basic,
            scan_cidr,
        })
    }
}

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
    // The write-route switch and the Basic pair, validated at boot (story 6.1, arbitration 5).
    // A bad value refuses to start WITH the variable named, never at request time silently.
    let config = AppConfig::from_env(|key| std::env::var(key).ok())
        .context("loading the auth configuration")?;
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
    if let Some(cidr) = config.scan_cidr.clone() {
        spawn_startup_scan(database_url.clone(), clock.now(), cidr);
    }

    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("binding {bind}"))?;
    tracing::info!(%bind, "opencmdb listening");
    axum::serve(listener, app(pool, config))
        .await
        .context("serving the HTTP app")?;
    Ok(())
}

/// The HTTP surface, factored out of `main` so it is testable without binding a socket. The
/// database pool is carried in axum state on the main router; the document sub-router carries
/// its own pool-free state (story 6.1 §7), so its handler cannot reach the database BY TYPE.
///
/// 🔴 **Every route is registered BEFORE `.layer(auth_deny)` — the conditional included**
/// (story 6.1 §2, measured on axum 0.8.9: a route added after `.layer()` bypasses the
/// middleware entirely — axum's own doc says so — and a POST with no credential reached its
/// handler). The switch decides what the `Router` CARRIES; it must never decide on which side
/// of the layer a route lands.
fn app(pool: MySqlPool, config: AppConfig) -> Router {
    let mut router = Router::new()
        .route("/", get(redirect_to_triage))
        .route("/gap", get(page::gap_fragment))
        .route("/assets/{*path}", get(page::asset))
        .route("/metrics", get(metrics::handler))
        .route("/healthz", get(healthz))
        .with_state(pool.clone())
        // 🔴 The pool-free screens are merged AFTER `.with_state`, so their router's
        // state is `()` and not the pool. That is what makes epic constraint 1 enforceable:
        // give one of those handlers `State<MySqlPool>` and it fails to COMPILE. Merged before,
        // or built on the main router, the same handler compiles and the guard is a sentence.
        .merge(screens::router(config.scan_cidr.clone()))
        // `/triage` carries the pool AND the perimeter, so the perimeter is a parameter rather
        // than a second reading of the environment (story 6b.2's M12, shipped and now closed).
        .merge(page::triage_router(pool.clone(), config.scan_cidr.clone()));
    if config.document_enabled {
        // The switch governs EXISTENCE only (arbitration 4): merged above the layer, the route
        // is auth-gated exactly like every other non-public path. The pool lives INSIDE the
        // store-backed port, not on the sub-router's state, so the handler still cannot extract
        // it (story 6.1's M4 carrier survives, story 6.2).
        router = router.merge(document::router(pool));
    }
    // Deny-by-default seam over every route AND the fallback (Story 3.8, story 6.1): the
    // public allowlist is `/healthz` + `/assets/*`, `/metrics` sits behind the scrape token,
    // everything else answers to the Basic pair.
    router.layer(axum::middleware::from_fn_with_state(
        config,
        auth::auth_deny,
    ))
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
    use opencmdb_core::observation::{ConnectorId, L2DomainId, Scope, VantageId};
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

            // 🔴 The poll lives in `scan_pass::poll_ingest_resolve` and NOWHERE ELSE. Story 5.14's
            // code review found — by three layers independently, and measured by one of them at
            // 4.009 s / 2.0075 s / 1.0025 s as the probe timeout moved — that extracting the seam
            // had removed the ingest loop here and LEFT the poll, so every startup swept the CIDR
            // TWICE and threw the first sweep away. A host answering the first and missing the
            // second was silently lost. `sink` stayed syntactically used, so the compiler said
            // nothing, and deleting the dead block left all 502 tests green: **nothing pins this**,
            // and that is why the "three uncarried lines" figure below was wrong.
            let pool = match MySqlPool::connect(&database_url).await {
                Ok(pool) => pool,
                Err(error) => {
                    tracing::warn!(%error, "startup scan: could not connect to ingest");
                    return;
                }
            };
            // 🔴 WHAT IS UNCARRIED HERE, stated after the code review corrected it TWICE.
            //
            // Everything below `poll_ingest_resolve` is driven end-to-end by a test with a
            // `FixtureConnector`. Everything in THIS function is not: it is a `thread::spawn` whose
            // handle is dropped, and no test can reach it. Deleting the call below leaves the whole
            // suite green; deleting the `resolve` call inside the seam reds **six** tests, every
            // one on a named assertion.
            //
            // ⚠️ An earlier version of this comment said "the three lines that remain uncarried"
            // and "reds one test", and called itself measured. Both were wrong, and the first was
            // wrong in the way that mattered: the uncarried region is this whole function — the
            // runtime build, the CIDR parse, two environment knobs that decide what the scan
            // MISSES, the pool connect and four early-return branches — and a live defect was
            // sitting in it (a duplicated sweep, above) while the sentence claimed three lines.
            // **A region you have not counted is not a region you have measured.**
            let outcome = crate::scan_pass::poll_ingest_resolve(&mut connector, now, &pool).await;
            tracing::info!(
                ingested = outcome.ingested,
                failed = outcome.failed,
                resolved = outcome.resolution.is_some(),
                "startup scan complete"
            );
        });
    });
}

/// `/` sends the operator to the triage screen.
///
/// # Why a redirect rather than making `/` the screen
///
/// Guy's arbitration (2026-08-18): separating the redirect from the screen keeps **one address
/// per screen**, so `aria-current` has one case rather than two, and every bookmark that ever
/// pointed at `/` still resolves — the README, both manuals, the landing site and the Docker Hub
/// page all point here (their prose is story 6b.12's sweep).
///
/// The target is `/triage` rather than `/dashboard` on a measurement rather than a taste:
/// `epics.md` makes the triage screen the one fed by the REAL gap, while the dashboard is mixed
/// by construction — a real reach section beside example cards. The person who installs the
/// product arrives here, and that is exactly whom the example-data marker defends.
/// ⚠️ Registered for re-examination by story 6b.5; it costs one line, because the target is one.
async fn redirect_to_triage() -> axum::response::Redirect {
    axum::response::Redirect::to("/triage")
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
    use base64::Engine as _;
    use tower::ServiceExt; // for `oneshot`

    /// A pool that never connects — these tests drive routes that issue no query, or assert
    /// statuses a failed connection cannot fake (401 is the layer's, produced before any pool
    /// use).
    fn lazy_pool() -> MySqlPool {
        MySqlPool::connect_lazy("mysql://root:x@127.0.0.1:3306/none").expect("lazy pool")
    }

    /// The test pair. Tests construct [`AppConfig`] directly (arbitration 5) — no env mutation.
    fn pair() -> BasicCredentials {
        BasicCredentials {
            user: "op".to_string(),
            password: "s3cret".to_string(),
        }
    }

    fn config(document_enabled: bool, basic: Option<BasicCredentials>) -> AppConfig {
        AppConfig {
            document_enabled,
            basic,
            // Story 6b.2: the shell's perimeter. `None` here so every pre-existing test keeps
            // exercising the shape it was written for; the perimeter has its own tests.
            scan_cidr: None,
        }
    }

    /// `Authorization: Basic base64(user:password)`.
    fn basic_header(user: &str, password: &str) -> String {
        format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(format!("{user}:{password}"))
        )
    }

    const CHALLENGE: &str = "Basic realm=\"opencmdb\"";

    fn www_authenticate(response: &axum::response::Response) -> Option<String> {
        response
            .headers()
            .get(axum::http::header::WWW_AUTHENTICATE)
            .map(|value| value.to_str().expect("an ASCII header").to_string())
    }

    async fn body_text(response: axum::response::Response) -> String {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("a readable body");
        String::from_utf8(bytes.to_vec()).expect("a UTF-8 body")
    }

    /// An authenticated `POST /document-all` carrying `body` as a urlencoded form.
    fn document_post(authorization: Option<&str>, body: &str) -> Request<Body> {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/document-all")
            .header(
                axum::http::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            );
        if let Some(value) = authorization {
            builder = builder.header(axum::http::header::AUTHORIZATION, value);
        }
        builder.body(Body::from(body.to_string())).unwrap()
    }

    // ── AC1: the route exists only when the switch is set ──────────────────────────────────

    /// Switch unset, valid credential: the layer passes and the FALLBACK answers — 404 with an
    /// EMPTY body, distinguishable from every response the route gives THIS POST (422, the
    /// pinned 404, 501 — all non-empty). ⚠️ Scoped to the POST pair on purpose: the route's own
    /// 405 to a GET is empty-bodied, so the universal sentence would be false (code review).
    /// The discriminator is the AUTHENTICATED pair: the unauthenticated status is 401 in both
    /// shapes and proves nothing (M1's carrier).
    #[tokio::test]
    async fn without_the_switch_an_authenticated_post_reaches_the_empty_fallback() {
        let app = app(lazy_pool(), config(false, Some(pair())));
        let response = app
            .oneshot(document_post(
                Some(&basic_header("op", "s3cret")),
                "subject=x",
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_text(response).await,
            "",
            "the fallback's body is empty — the route answers this POST non-empty in every branch"
        );
    }

    /// Switch set, same credential, same request: the ROUTE answers — one of AC2's refusals,
    /// non-empty body (here 422: `x` is not an observation id).
    #[tokio::test]
    async fn with_the_switch_the_same_authenticated_post_reaches_the_route() {
        let app = app(lazy_pool(), config(true, Some(pair())));
        let response = app
            .oneshot(document_post(
                Some(&basic_header("op", "s3cret")),
                "subject=x",
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = body_text(response).await;
        assert!(!body.is_empty(), "a registered-route response is non-empty");
        assert!(body.contains("subject"), "the 422 names the field: {body}");
    }

    /// The store-backed production route (story 6.2): through the whole app, a well-formed
    /// unknown subject answers the domain's pinned 404 body — non-empty, so it discriminates
    /// against the fallback. 🔴 **DB-gated**: under 6.2 the unknown-subject answer is a real
    /// store read, so a lazy pool would answer 500, not 404 (validation §8(d)). A green LOCAL
    /// suite says nothing here — the CI run is the oracle.
    #[tokio::test]
    async fn the_production_route_answers_the_pinned_unknown_subject_body() {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping document-route DB test: DATABASE_URL unset");
            return;
        };
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        let subject = uuid::Uuid::now_v7(); // never inserted → unknown
        let response = app(pool, config(true, Some(pair())))
            .oneshot(document_post(
                Some(&basic_header("op", "s3cret")),
                &format!("subject={subject}"),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            body_text(response).await,
            "unknown subject: nothing can be documented"
        );
    }

    // ── AC3: Basic stands where the allowlist stood ────────────────────────────────────────

    /// A formerly-public page without a credential: 401 with the exact challenge (M11 reds the
    /// header assertion).
    #[tokio::test]
    async fn formerly_public_pages_challenge_without_a_credential() {
        for path in ["/", "/gap"] {
            let app = app(lazy_pool(), config(false, Some(pair())));
            let response = app
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{path}");
            assert_eq!(
                www_authenticate(&response).as_deref(),
                Some(CHALLENGE),
                "{path} must carry the challenge"
            );
        }
    }

    /// 🔴 **AC4 — the partition, asserted over the ROUTE TABLE and on the real HTTP body.**
    ///
    /// # Why it is shaped like this
    ///
    /// The criterion says it in so many words: *"a test that checks the marker inside the templates
    /// that already have it proves nothing about the eleventh screen."* So this iterates
    /// [`screens::Screen::ALL`], drives the REAL `app()` — auth layer included — and reads the
    /// bytes the browser would receive. It never opens a template.
    ///
    /// ⚠️ **The `Fed` half needs a REAL pool, and that is not laziness.** With a dead pool,
    /// `/triage` answers 500 and its body carries no marker — so *"a fed screen carries no marker"*
    /// would pass **because nothing rendered at all**. That is the guard placed where the defect
    /// cannot occur, which this epic has now shipped five times.
    ///
    /// 🔴 **The first version of this test gated that half on `DATABASE_URL` being set, and the
    /// condition was measuring the wrong thing** — found by running the suite WITH a database, at
    /// which point it reddened at once (`left: 500, right: 200`). `lazy_pool()` is a hardcoded
    /// dead URL (`…:3306/none`) and ignores `DATABASE_URL` entirely, so the environment variable
    /// says nothing about the pool the test uses: the `Fed` half could never have passed, and with
    /// the variable unset it was never even attempted. *A gate keyed on a fact that does not govern
    /// the code under test is not a gate.* It now connects and migrates, like every other
    /// database-backed test in this crate.
    ///
    /// 🔑 **The count assertion at the end is load-bearing and must not be tidied away.** Story
    /// 6b.3's validation measured it: with content dispatched from `nature()`, promoting a second
    /// screen to `Example` gives it real example content AND the marker, so the per-screen loop
    /// stays GREEN — *"this screen is declared Example and carries the marker"* is a true sentence.
    /// **Delete the count and mutation M7 goes green.** It is the only thing standing between the
    /// product and a second witness screen nobody decided on.
    #[tokio::test]
    async fn the_marker_partition_follows_every_screens_declared_nature() {
        let marker = "example-marker-badge";
        // 🔑 The *not built yet* line has its OWN class, and the partition asserts both directions
        // for both of them: an `Empty` screen must say it is not built and must NOT be called a
        // demonstration. Sharing one class would turn this test green for the wrong reason.
        let not_yet = "not-yet-badge";
        let mut example_contents: Vec<(&str, screens::ExampleContent)> = Vec::new();
        let mut probed = 0_usize;

        // A real pool when one is reachable, so the `Fed` half asserts over a page that RENDERED.
        let (pool, fed_reachable) = match std::env::var("DATABASE_URL") {
            Ok(url) => {
                let pool = MySqlPool::connect(&url).await.expect("connect");
                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .expect("migrate");
                (pool, true)
            }
            Err(_) => {
                eprintln!("the fed half of the partition needs DATABASE_URL — demo screens only");
                (lazy_pool(), false)
            }
        };

        for screen in screens::Screen::ALL {
            let nature = screen.nature();
            if let screens::Nature::Example(content) = nature {
                example_contents.push((screen.href(), content));
            }
            if matches!(nature, screens::Nature::Fed | screens::Nature::Mixed) && !fed_reachable {
                // Said on stderr above, never skipped in silence — see this test's doc.
                continue;
            }
            let response = app(pool.clone(), config(false, Some(pair())))
                .oneshot(
                    Request::builder()
                        .uri(screen.href())
                        .header(
                            axum::http::header::AUTHORIZATION,
                            basic_header("op", "s3cret"),
                        )
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "{} must render before its marker can mean anything",
                screen.href()
            );
            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("a rendered body");
            let body = String::from_utf8(body.to_vec()).expect("the page is UTF-8");
            let carries = body.contains(marker);
            let says_pending = body.contains(not_yet);
            match nature {
                screens::Nature::Example(_) => assert!(
                    carries,
                    "{} shows the example dataset and must say so",
                    screen.href()
                ),
                // 🔑 A MIXED screen carries the marker and MUST — on its example sections. This
                // arm says only that the marker is present somewhere; **which section carries it is
                // a property inside one body and the route table cannot see it**. The sibling guard
                // `every_example_section_carries_its_own_marker` owns that, because story 6b.5's
                // validation measured this oracle staying GREEN when one section of two lost its
                // marker: the other section's string is enough to satisfy a `contains`.
                screens::Nature::Mixed => assert!(
                    carries,
                    "{} is mixed and its example sections must say so",
                    screen.href()
                ),
                screens::Nature::Fed | screens::Nature::Empty => assert!(
                    !carries,
                    "{} carries the marker and must not: a fed screen would be calling the \
                     operator's own network a demonstration, and an empty one would be calling \
                     a blank page one",
                    screen.href()
                ),
            }
            // 🔴 The other half of Guy's arbitration of 2026-08-19: a screen whose story has not
            // landed SAYS SO. Eight screens rendered a blank `<main>` with nothing on it until this
            // story's code review, against `epics.md:2092` — *"those whose code is not implemented
            // show an example dataset with a text saying so"*, a premise no layer had surfaced.
            match nature {
                screens::Nature::Empty => assert!(
                    says_pending,
                    "{} is not built yet and must say so, or it reads as a broken screen rather \
                     than a deliberate one",
                    screen.href()
                ),
                screens::Nature::Fed | screens::Nature::Example(_) | screens::Nature::Mixed => {
                    assert!(
                        !says_pending,
                        "{} carries the *not built yet* line over content it actually has",
                        screen.href()
                    )
                }
            }
            probed += 1;
        }

        // 🔴 DERIVED from the route table, never a literal — and the literal is why this test was
        // RED on the committed tree. It read `probed >= 9`, a hardcoded floor written when exactly
        // one screen (`Triage`, `Fed`) was skipped without a database. Story 6b.5 made the dashboard
        // `Mixed`, which the same branch also skips, so the count fell to 8 and the suite failed —
        // **and the edit that broke it was three lines above this assertion, in the same diff**.
        // ⚠️ CI supplies a database, so CI stayed green and only the local path reddened: a floor
        // that CI cannot check is a floor nobody re-reads. Counting the screens the loop was
        // *supposed* to reach makes it impossible to go stale again.
        let expected = screens::Screen::ALL
            .iter()
            .filter(|screen| {
                fed_reachable
                    || !matches!(
                        screen.nature(),
                        screens::Nature::Fed | screens::Nature::Mixed
                    )
            })
            .count();
        assert_eq!(
            probed, expected,
            "the premise: every screen the loop should reach was probed — a loop that went empty, \
             or one whose skip rule drifted from the route table, would assert nothing"
        );
        // 🔴 **TWO witness screens now, and the second is a DECISION rather than a drift.** This
        // read `1` with a message calling a second one *"a witness screen nobody decided on"* —
        // story 6b.6 is the one somebody decided on, and bumping the number without rewriting the
        // sentence would have left a false explanation standing over a true count.
        assert_eq!(
            example_contents.len(),
            2,
            "the witness screens are the inventory (story 6b.3) and the device record (6b.6), and \
             a third is a screen that grew example content without a story deciding it should: \
             {example_contents:?}"
        );
        // 🔑 **The property that actually grows, and it is not the count.** Two screens declaring
        // the SAME `ExampleContent` is one screen silently rendering another's body under its own
        // heading — the defect story 6b.3's code review closed in the TYPE by giving
        // `Nature::Example` a payload. The count notices an undecided screen; this notices a screen
        // showing the wrong thing, and it stays true however many witness screens ship.
        for (index, (href, content)) in example_contents.iter().enumerate() {
            if let Some((other, _)) = example_contents
                .iter()
                .take(index)
                .find(|(_, seen)| seen == content)
            {
                panic!(
                    "{href} and {other} both declare {content:?}: one of them is rendering the \
                     other's body under its own heading"
                );
            }
        }
    }

    /// 🔴 **Every screen of the shell answers 401 without a credential — named, one by one.**
    ///
    /// # Why this exists, and what it replaces
    ///
    /// Story 6b.2 added ten addresses and no test named ONE of them. The code was correct —
    /// `is_public` is a property (`/healthz` or `/assets/*`), so the ten are refused by
    /// construction — but nothing held it there, and the code review measured both ways it
    /// could fall:
    ///
    /// - widen `is_public` with `/devices` and the whole suite stayed GREEN (the story's own
    ///   M7, predicted before implementation and never executed);
    /// - merge the two sub-routers BELOW `auth_deny` and `GET /dashboard` answered **200 OK**
    ///   with no credential at all — the suite reddened only through two GENERIC probes
    ///   (`/anything`, `/admin`) inherited from story 6.1, which name nothing of this story.
    ///
    /// 🔑 It iterates [`screens::Screen::ALL`] rather than a copied list, so a screen added
    /// tomorrow is covered the day it exists — an enumeration would go stale exactly when the
    /// eleventh screen lands.
    ///
    /// ⚠️ A lazy pool is enough BECAUSE the refusal happens in the layer, above every handler:
    /// no request here ever reaches the database. A 200 would mean the layer was bypassed.
    #[tokio::test]
    async fn every_screen_is_refused_without_a_credential() {
        let mut checked = 0_usize;
        for path in screens::Screen::ALL
            .iter()
            .map(|screen| screen.href())
            // 🔴 The record route is OFF `Screen`, so this loop could not see it — the first
            // address in this product `Screen::ALL` structurally cannot represent. Story 6b.2's
            // review found that a screen named by no test answered 200 when merged below the auth
            // layer; an off-`Screen` route is protected by the same `is_public` property and was
            // likewise carried by nothing. Both a KNOWN slug and an unknown one, because they take
            // different branches inside the handler.
            .chain(["/devices/printer-hall", "/devices/does-not-exist", "/"])
        {
            let app = app(lazy_pool(), config(false, Some(pair())));
            let response = app
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "{path} must answer 401 without a credential — a 200 here means the route sits \
                 below `auth_deny`, or that `is_public` has been widened to admit it"
            );
            assert_eq!(
                www_authenticate(&response).as_deref(),
                Some(CHALLENGE),
                "{path} must carry the challenge"
            );
            checked += 1;
        }
        assert_eq!(
            checked, 13,
            "the premise: ten screens, two record addresses and `/` ({checked} probed) — a loop \
             that went empty would assert nothing"
        );
    }

    /// 🔴 **The filter narrows THROUGH THE ROUTE — the guard that could see what the pure one
    /// could not.**
    ///
    /// `the_filter_narrows_and_says_so_when_it_matches_nothing` calls `inventory_body` directly and
    /// was **green over a route that ignored the query entirely**: the router's closure took no
    /// arguments and the content was rendered with `ScreenQuery::default()`, so
    /// `/devices?kind=printer` served all eight devices. Found by looking at the running server.
    ///
    /// 🔑 Epic 5's dominant class — *a guard placed where the defect cannot occur reads as coverage
    /// and is none* — and story 6b.4's `triage_html` was the same shape: a helper that renders what
    /// production does not.
    #[tokio::test]
    async fn the_filter_narrows_through_the_real_route() {
        let app = app(lazy_pool(), config(false, Some(pair())));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/devices?kind=printer")
                    .header(
                        axum::http::header::AUTHORIZATION,
                        basic_header("op", "s3cret"),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = String::from_utf8(
            axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        let rows = body.matches("data-device-id=").count();
        assert_eq!(
            rows, 1,
            "?kind=printer must narrow to the one printer; {rows} rows came back, which means the \
             route is not reading its query"
        );
        assert!(body.contains(r#"data-device-id="printer-hall""#));
    }

    /// 🔴 **The record route answers a NON-CANONICAL slug, and this is the only thing that holds
    /// it.**
    ///
    /// # Why it needs a test of its own
    ///
    /// `Screen::href` returns a `&'static str` used as a route PATTERN by `screens::router` and
    /// FETCHED as a URL by the partition and auth guards. `/devices/{id}` cannot be both, so the
    /// record's address sits **off `Screen`** (Guy's arbitration, 2026-08-19) and **no inherited
    /// guard covers it**. Story 6b.6's validation built the alternative and measured what it costs:
    /// with a static route registered beside the parameterised one, the static wins for exactly the
    /// URL the guards probe — after which the handler could **ignore its slug entirely, serve
    /// device #1 for `/devices/does-not-exist`, and leave 636 tests plus `clippy -D warnings`
    /// green**.
    ///
    /// 🔑 **A non-canonical slug is the whole point.** `Screen::Device.href()` is
    /// `/devices/nas-01`, so a test fetching only that address is satisfied by a handler that
    /// ignores its input. This one asks for a device the navigation does not point at.
    #[tokio::test]
    async fn the_record_route_answers_a_non_canonical_slug() {
        // ⚠️ **The oracle is the ADDRESS, not the name**, and the reason is a measurement: the
        // shell's navigation carries `href="/devices/nas-01"` on every page, because
        // `Screen::Device` is the record's entry and an entry must point somewhere. A name-based
        // oracle therefore matched the NAVIGATION and reported the wrong device as served. *A guard
        // that reads the whole page must key on something the frame cannot contain.*
        for (slug, expected, refused) in [
            // The address the navigation points at.
            ("nas-01", "192.0.2.10", "192.0.2.31"),
            // 🔑 NOT the navigation's address: a handler ignoring its slug reds here and nowhere.
            ("printer-hall", "192.0.2.31", "192.0.2.10"),
        ] {
            let app = app(lazy_pool(), config(false, Some(pair())));
            let response = app
                .oneshot(
                    Request::builder()
                        .uri(format!("/devices/{slug}"))
                        .header(
                            axum::http::header::AUTHORIZATION,
                            basic_header("op", "s3cret"),
                        )
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                response.status(),
                StatusCode::OK,
                "/devices/{slug} must render"
            );
            let body = String::from_utf8(
                axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap()
                    .to_vec(),
            )
            .unwrap();
            assert!(
                body.contains(expected),
                "/devices/{slug} must show {expected}"
            );
            assert!(
                !body.contains(refused),
                "/devices/{slug} shows {refused}: the handler is ignoring its slug"
            );
        }
    }

    /// 🔴 **A slug no device carries says so, and the slug is never echoed back.**
    ///
    /// The obvious implementation — `format!` of the path segment into `Html` — is reflected XSS,
    /// measured at this story's validation serving `<script>alert(1)</script>` back with a 200.
    /// Story 6b.4's review verified *"no XSS"* over TEMPLATES; the record's unknown case is the
    /// first place in this product where a raw path segment could be echoed at all.
    ///
    /// ⚠️ It answers **200 and a page**, not 404: the address is a real screen reached from the
    /// product's own navigation, and what is unknown is the slug.
    #[tokio::test]
    async fn an_unknown_slug_is_answered_without_echoing_it() {
        for slug in ["does-not-exist", "%3Cscript%3Ealert(1)%3C%2Fscript%3E"] {
            let app = app(lazy_pool(), config(false, Some(pair())));
            let response = app
                .oneshot(
                    Request::builder()
                        .uri(format!("/devices/{slug}"))
                        .header(
                            axum::http::header::AUTHORIZATION,
                            basic_header("op", "s3cret"),
                        )
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let body = String::from_utf8(
                axum::body::to_bytes(response.into_body(), usize::MAX)
                    .await
                    .unwrap()
                    .to_vec(),
            )
            .unwrap();
            assert!(
                body.contains(&rust_i18n::t!("record.unknown").to_string()),
                "an unknown slug must be SAID"
            );
            assert!(
                !body.contains("<script>"),
                "the slug reached the page unescaped: reflected XSS"
            );
            assert!(
                !body.contains("alert(1)"),
                "the slug is echoed back at all, escaped or not — it teaches the operator nothing \
                 and it is the shape that regresses"
            );
        }
    }

    /// A valid credential REACHES a formerly-public page: anything but 401, and never the
    /// challenge. ⚠️ With a lazy pool and no database the page handlers answer 500 through
    /// `server_error`; asserting 200 needs a database and belongs to the one CI-gated test.
    #[tokio::test]
    async fn a_valid_credential_reaches_a_formerly_public_page() {
        let app = app(lazy_pool(), config(false, Some(pair())));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/gap")
                    .header(
                        axum::http::header::AUTHORIZATION,
                        basic_header("op", "s3cret"),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_ne!(response.status(), StatusCode::UNAUTHORIZED, "reached");
        assert_eq!(www_authenticate(&response), None, "never the challenge");
    }

    /// The half-right pairs are refused with the challenge (M10's end-to-end carrier — the
    /// both-halves-wrong shape would leave a user-only comparison green).
    #[tokio::test]
    async fn half_right_pairs_are_refused_with_the_challenge() {
        for header in [basic_header("op", "wrong"), basic_header("who", "s3cret")] {
            let app = app(lazy_pool(), config(false, Some(pair())));
            let response = app
                .oneshot(
                    Request::builder()
                        .uri("/gap")
                        .header(axum::http::header::AUTHORIZATION, header.clone())
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{header}");
            assert_eq!(www_authenticate(&response).as_deref(), Some(CHALLENGE));
        }
    }

    /// Decode robustness end-to-end: garbage base64 and a colon-free pair answer 401 (with the
    /// challenge — the pair IS configured); the mixed-case scheme and the colon-carrying
    /// password are ACCEPTED (M12; split on the first colon).
    #[tokio::test]
    async fn decode_robustness_end_to_end() {
        let refused = ["Basic not/base64!!".to_string(), {
            format!(
                "Basic {}",
                base64::engine::general_purpose::STANDARD.encode("no-colon")
            )
        }];
        for header in refused {
            let app = app(lazy_pool(), config(false, Some(pair())));
            let response = app
                .oneshot(
                    Request::builder()
                        .uri("/gap")
                        .header(axum::http::header::AUTHORIZATION, header.clone())
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{header}");
            assert_eq!(www_authenticate(&response).as_deref(), Some(CHALLENGE));
        }

        // Mixed-case scheme, correct pair: accepted (RFC 7235 §2.1 — M12 reds this).
        let mixed_case_app = app(lazy_pool(), config(false, Some(pair())));
        let mixed = format!(
            "bAsIc {}",
            base64::engine::general_purpose::STANDARD.encode("op:s3cret")
        );
        let response = mixed_case_app
            .oneshot(
                Request::builder()
                    .uri("/gap")
                    .header(axum::http::header::AUTHORIZATION, mixed)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_ne!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "mixed-case scheme accepted"
        );

        // A password containing a colon authenticates (split on the FIRST colon, RFC 7617 §2).
        let colon_pair = BasicCredentials {
            user: "op".to_string(),
            password: "a:b".to_string(),
        };
        let colon_app = app(lazy_pool(), config(false, Some(colon_pair)));
        let response = colon_app
            .oneshot(
                Request::builder()
                    .uri("/gap")
                    .header(axum::http::header::AUTHORIZATION, basic_header("op", "a:b"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_ne!(
            response.status(),
            StatusCode::UNAUTHORIZED,
            "colon password accepted"
        );
    }

    /// Two `Authorization` headers are refused outright — even right-then-wrong, the measured
    /// trap of `HeaderMap::get`'s first-value semantics.
    #[tokio::test]
    async fn two_authorization_headers_are_refused_end_to_end() {
        let app = app(lazy_pool(), config(false, Some(pair())));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/gap")
                    .header(
                        axum::http::header::AUTHORIZATION,
                        basic_header("op", "s3cret"),
                    )
                    .header(
                        axum::http::header::AUTHORIZATION,
                        basic_header("op", "wrong"),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    /// Pair UNSET: 401 WITHOUT the challenge header (arbitration 6 — no infinite dialog on
    /// unupgraded deployments). The assertion is on the header's ABSENCE (M7 reds the status).
    #[tokio::test]
    async fn an_unset_pair_refuses_without_the_challenge() {
        for path in ["/", "/gap", "/anything"] {
            let app = app(lazy_pool(), config(false, None));
            let response = app
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{path}");
            assert_eq!(
                www_authenticate(&response),
                None,
                "{path}: a challenge nothing can satisfy must not be emitted"
            );
        }
    }

    /// ⚠️ M9's ONLY carrier (story 6.1 §2): with the switch SET and the pair SET, the write
    /// route without a credential answers 401 with the challenge — i.e. the conditional route
    /// sits ABOVE the layer. With the switch unset the mutation is invisible (both shapes 401),
    /// and the page-path challenge tests never touch the conditional registration.
    #[tokio::test]
    async fn the_write_route_challenges_without_a_credential() {
        let app = app(lazy_pool(), config(true, Some(pair())));
        let response = app.oneshot(document_post(None, "subject=x")).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(www_authenticate(&response).as_deref(), Some(CHALLENGE));
    }

    /// `/metrics` does not accept Basic (M5b) and its 401 never advertises it (arbitration 6 /
    /// F14) — one caller class, one mechanism. Race-safe: whatever `OPENCMDB_METRICS_TOKEN`
    /// holds, a Basic header is never `Bearer <token>`.
    #[tokio::test]
    async fn metrics_does_not_accept_basic_and_never_advertises_it() {
        let app = app(lazy_pool(), config(false, Some(pair())));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .header(
                        axum::http::header::AUTHORIZATION,
                        basic_header("op", "s3cret"),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            www_authenticate(&response),
            None,
            "the metrics 401 must not advertise a scheme its branch does not accept"
        );
    }

    /// The still-public surface is reachable with no credential even when a pair is configured:
    /// assets answer 200; `/healthz` is REACHED (503 here — the lazy pool has no database, and
    /// 503 is the handler's own answer, not the layer's 401).
    #[tokio::test]
    async fn the_public_surface_stays_reachable_without_a_credential() {
        let assets_app = app(lazy_pool(), config(false, Some(pair())));
        let response = assets_app
            .oneshot(
                Request::builder()
                    .uri("/assets/app.css")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let healthz_app = app(lazy_pool(), config(false, Some(pair())));
        let response = healthz_app
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::SERVICE_UNAVAILABLE,
            "healthz is reached — its own DB-less answer, never the layer's 401"
        );
    }

    // ── AppConfig::from_env (arbitration 5; M7b, M8) ───────────────────────────────────────

    /// The lookup over a literal table — no env mutation anywhere in these tests.
    fn lookup_of<'a>(
        table: &'a [(&'static str, &'static str)],
    ) -> impl Fn(&str) -> Option<String> + 'a {
        move |key| {
            table
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| value.to_string())
        }
    }

    #[test]
    fn an_empty_environment_is_a_valid_minimal_config() {
        let config = AppConfig::from_env(lookup_of(&[])).expect("valid");
        assert!(!config.document_enabled);
        assert_eq!(config.basic, None);
    }

    #[test]
    fn the_switch_accepts_its_documented_values_and_refuses_the_rest() {
        for on in ["1", "true", "TRUE", " 1 "] {
            let config =
                AppConfig::from_env(lookup_of(&[("OPENCMDB_DOCUMENT_ENABLED", on)])).expect(on);
            assert!(config.document_enabled, "{on:?} enables");
        }
        for off in ["0", "false", ""] {
            let config =
                AppConfig::from_env(lookup_of(&[("OPENCMDB_DOCUMENT_ENABLED", off)])).expect(off);
            assert!(!config.document_enabled, "{off:?} disables");
        }
        // "The rest" measured on several specimens, not one (code review): the near-misses a
        // hand reaching for a boolean actually types.
        for wrong in ["yes", "on", "enabled", "2", "vrai"] {
            let refused = AppConfig::from_env(lookup_of(&[("OPENCMDB_DOCUMENT_ENABLED", wrong)]));
            assert_eq!(
                refused,
                Err(AppConfigError::UnrecognisedSwitch {
                    value: wrong.to_string()
                }),
                "{wrong:?}: a typo is a boot error, not a mysteriously missing route"
            );
        }
    }

    #[test]
    fn a_full_pair_is_accepted() {
        let config = AppConfig::from_env(lookup_of(&[
            ("OPENCMDB_BASIC_USER", "op"),
            ("OPENCMDB_BASIC_PASSWORD", "s3cret"),
        ]))
        .expect("valid");
        assert_eq!(
            config.basic,
            Some(BasicCredentials {
                user: "op".to_string(),
                password: "s3cret".to_string()
            })
        );
    }

    /// Empty halves count as unset (M7b: an empty pair must yield `basic: None`, never a
    /// credential of two empty strings that `base64(":")` would satisfy).
    #[test]
    fn an_empty_pair_is_unset_not_a_credential() {
        let config = AppConfig::from_env(lookup_of(&[
            ("OPENCMDB_BASIC_USER", ""),
            ("OPENCMDB_BASIC_PASSWORD", ""),
        ]))
        .expect("valid");
        assert_eq!(config.basic, None);
    }

    /// A perimeter with nothing VISIBLE in it counts as unset, invisible characters included.
    ///
    /// 🔴 Written after the code review measured the hole: `"\u{200B}".trim().is_empty()` is
    /// `false`, so a zero-width space passed the emptiness filter and the navigation footer
    /// rendered a label followed by nothing — the very *"blank rather than named"* outcome AC1
    /// exists to forbid, reached through a channel neither the story nor its validation saw.
    ///
    /// 🔑 The two halves are asserted together on purpose: a filter that rejects everything
    /// would satisfy the first half alone, and a real CIDR must still survive.
    #[test]
    fn a_perimeter_with_no_visible_glyph_counts_as_unset() {
        for blank in ["", " ", "\t", "\u{200B}", " \u{FEFF}\u{2060} ", "\u{00AD}"] {
            let config = AppConfig::from_env(lookup_of(&[("OPENCMDB_SCAN_CIDR", blank)]))
                .expect("a blank perimeter is not a boot error, only an unset one");
            assert_eq!(
                config.scan_cidr, None,
                "{blank:?} carries no visible glyph, so the footer must say `not configured` \
                 rather than render a label with nothing after it"
            );
        }
        let configured = AppConfig::from_env(lookup_of(&[("OPENCMDB_SCAN_CIDR", "192.0.2.0/24")]))
            .expect("valid");
        assert_eq!(
            configured.scan_cidr.as_deref(),
            Some("192.0.2.0/24"),
            "and a real perimeter still reaches the surface — a filter that rejects everything \
             would pass the half above and destroy the feature"
        );
    }

    #[test]
    fn a_half_configured_pair_is_a_boot_error_in_both_directions() {
        let missing_password = AppConfig::from_env(lookup_of(&[("OPENCMDB_BASIC_USER", "op")]));
        assert_eq!(
            missing_password,
            Err(AppConfigError::HalfConfiguredPair {
                missing: "OPENCMDB_BASIC_PASSWORD"
            })
        );
        let missing_user = AppConfig::from_env(lookup_of(&[("OPENCMDB_BASIC_PASSWORD", "s3cret")]));
        assert_eq!(
            missing_user,
            Err(AppConfigError::HalfConfiguredPair {
                missing: "OPENCMDB_BASIC_USER"
            })
        );
    }

    /// M8: a non-ASCII credential is refused at boot with the variable named — the measured
    /// trap (`sécret`) would otherwise refuse everyone, permanently, with no diagnostic.
    #[test]
    fn a_non_ascii_credential_is_refused_at_boot() {
        let bad_password = AppConfig::from_env(lookup_of(&[
            ("OPENCMDB_BASIC_USER", "op"),
            ("OPENCMDB_BASIC_PASSWORD", "sécret"),
        ]));
        assert_eq!(
            bad_password,
            Err(AppConfigError::NonAsciiCredential {
                var: "OPENCMDB_BASIC_PASSWORD"
            })
        );
        let bad_user = AppConfig::from_env(lookup_of(&[
            ("OPENCMDB_BASIC_USER", "opérateur"),
            ("OPENCMDB_BASIC_PASSWORD", "s3cret"),
        ]));
        assert_eq!(
            bad_user,
            Err(AppConfigError::NonAsciiCredential {
                var: "OPENCMDB_BASIC_USER"
            })
        );
    }

    /// A control character in either half is refused at boot (code review): `"s3cret\n"` — the
    /// classic env-file `echo` accident — passes `is_ascii()`, and no browser dialog can type
    /// it, so it would refuse everyone permanently with no diagnostic.
    #[test]
    fn a_control_character_in_a_credential_is_refused_at_boot() {
        let newline_password = AppConfig::from_env(lookup_of(&[
            ("OPENCMDB_BASIC_USER", "op"),
            ("OPENCMDB_BASIC_PASSWORD", "s3cret\n"),
        ]));
        assert_eq!(
            newline_password,
            Err(AppConfigError::ControlCharacterInCredential {
                var: "OPENCMDB_BASIC_PASSWORD"
            })
        );
        let tab_user = AppConfig::from_env(lookup_of(&[
            ("OPENCMDB_BASIC_USER", "op\t"),
            ("OPENCMDB_BASIC_PASSWORD", "s3cret"),
        ]));
        assert_eq!(
            tab_user,
            Err(AppConfigError::ControlCharacterInCredential {
                var: "OPENCMDB_BASIC_USER"
            })
        );
    }

    /// A colon in the USER half can never authenticate (the decoder splits on the first
    /// colon), so it is refused at boot rather than silently refusing everyone forever.
    #[test]
    fn a_colon_in_the_user_half_is_refused_at_boot() {
        let refused = AppConfig::from_env(lookup_of(&[
            ("OPENCMDB_BASIC_USER", "op:eration"),
            ("OPENCMDB_BASIC_PASSWORD", "s3cret"),
        ]));
        assert_eq!(refused, Err(AppConfigError::ColonInUser));
    }

    /// The `Debug` impl never prints the password — a debug-printed config must not leak it.
    #[test]
    fn debug_output_redacts_the_password() {
        let printed = format!("{:?}", pair());
        assert!(printed.contains("op"));
        assert!(!printed.contains("s3cret"), "redacted: {printed}");
    }

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
        let response = app(pool, config(false, None))
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

        // `/` left the public allowlist under story 6.1 (arbitration 2′): this test now
        // authenticates through the same `AppConfig` seam production uses — constructed
        // directly, no env mutation, so no interaction with `DB_TEST_LOCK`'s env-free rule.
        // ⚠️ This is the ONE pre-6.1 test the visibility change breaks, and it breaks only
        // where `DATABASE_URL` exists (CI) — a green local suite says nothing here.
        // 🔴 Story 6b.2 made `/` a redirect, so this test now covers BOTH halves — and it is the
        // SECOND time it is the sole casualty of a decision about `/`, which the comment above
        // recorded for the first. The redirect first:
        let redirect = app(pool.clone(), config(false, Some(pair())))
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(
                        axum::http::header::AUTHORIZATION,
                        basic_header("op", "s3cret"),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            redirect.status(),
            StatusCode::SEE_OTHER,
            "`/` sends the operator to the triage screen rather than being one"
        );
        assert_eq!(
            redirect
                .headers()
                .get(axum::http::header::LOCATION)
                .and_then(|value| value.to_str().ok()),
            Some("/triage"),
            "and the target is the FED screen, not the mixed dashboard"
        );

        // Then the screen it points at, which must still show the real gap.
        let response = app(pool, config(false, Some(pair())))
            .oneshot(
                Request::builder()
                    .uri("/triage")
                    .header(
                        axum::http::header::AUTHORIZATION,
                        basic_header("op", "s3cret"),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
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

    /// 🔴 Story 6.2, AC5 — J3's CORRECTED half, measured end-to-end for the first time: an
    /// observation the product found → documented → the gap it would have shown is CLOSED.
    /// Through the whole stack against a live MariaDB; the assertion is `gaps.is_empty()` AND
    /// `abstention_count == 0` (validation H2: a wrong key yields an abstention, not a gap, so
    /// both halves are load-bearing). Provenance is verified through the ONE sanctioned reader
    /// (`repo::read_declared_provenance_for_test`, §6.5).
    #[tokio::test]
    async fn document_all_closes_the_gap_end_to_end() {
        use opencmdb_core::observation::{
            ConnectorId, Fact, HostnameSource, L2DomainId, ObsId, Observation, Scope, VantageId,
        };
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping J3 DB test: DATABASE_URL unset");
            return;
        };
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        // Children before parents (the FK from identity_link.observation_id, 0003). Static SQL.
        for statement in [
            "DELETE FROM declared_attribute",
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM interface",
            "DELETE FROM observation_record",
        ] {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }

        // The day-one case (FR13(a)): a sighting with ipv4 + hostname, NO declared record.
        let subject = ObsId::from_uuid(uuid::Uuid::now_v7());
        let observation = Observation {
            obs_id: subject,
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
                    name: "nas".into(),
                    source: HostnameSource::Dns,
                },
            ],
            raw: None,
        };
        repo::insert_observation(&pool, &observation)
            .await
            .expect("ingest");

        // Document it — a braced spelling of the real id (the canonical-form closure, §2).
        let response = app(pool.clone(), config(true, Some(pair())))
            .oneshot(document_post(
                Some(&basic_header("op", "s3cret")),
                &format!("subject={{{}}}", subject.as_uuid()),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let created = body_text(response).await;
        // Parse the minted entity id out of the 201 body ("… as entity <uuid>").
        let entity_id = created
            .rsplit(' ')
            .next()
            .expect("entity id in body")
            .to_string();

        // Provenance: two adopted rows, origin_obs_id = the subject, human author.
        let provenance = repo::read_declared_provenance_for_test(&pool, &entity_id)
            .await
            .expect("read provenance");
        assert_eq!(provenance.len(), 2, "one adopted row per projected field");
        for row in &provenance {
            let key = &row.attr_key;
            assert_eq!(row.origin, "adopted", "{key} is adopted");
            assert_eq!(
                row.origin_obs_id.as_deref(),
                Some(subject.as_uuid().to_string().as_str()),
                "{key} points at the subject"
            );
            assert_eq!(row.actor_id, "operator", "{key} carries a human author");
            assert_eq!(
                row.entity_id, entity_id,
                "{key} belongs to the minted entity"
            );
        }

        // 🔴 AC6's DIRECT oracle (code review): the documented KEYS equal `gap::project`'s keys
        // through the REAL fn — not a copy. First-occurrence-wins dedup, then compare as sorted
        // sets. A drifted key (M11) or a bin-local copy reds this directly, not only transitively.
        let mut expected_keys: Vec<String> = Vec::new();
        for (key, _) in opencmdb_core::gap::project(&observation) {
            if !expected_keys.contains(&key) {
                expected_keys.push(key);
            }
        }
        expected_keys.sort();
        let documented_keys: Vec<String> =
            provenance.iter().map(|row| row.attr_key.clone()).collect();
        assert_eq!(
            documented_keys, expected_keys,
            "the documented keys ARE gap::project's keys, through the real fn"
        );

        // The gap is CLOSED: reconcile the documented entity, no gap AND no abstention.
        let declared: Vec<(String, String)> = repo::load_declared_attributes(&pool)
            .await
            .expect("load declared")
            .into_iter()
            .filter(|(e, _, _)| *e == entity_id)
            .map(|(_, k, v)| (k, v))
            .collect();
        let reconciliation =
            opencmdb_core::gap::reconcile(("ipv4", "192.0.2.10"), &declared, &[observation]);
        assert!(
            reconciliation.gaps.is_empty(),
            "the documented fields carry no divergence: {:?}",
            reconciliation.gaps
        );
        assert_eq!(
            reconciliation.abstention_count(),
            0,
            "no abstention on the documented entity: {:?}",
            reconciliation.abstentions
        );
    }

    // ─── Story 6.3 — NFR5's SECOND assertion ────────────────────────────────────────────
    //
    // *"Documenting a field sets the declared value AND leaves the observation record
    // bit-for-bit unchanged, with the link intact"* (`prd.md:1214-1217`).
    //
    // 🔴 On the committed tree this property holds BY CONSTRUCTION: there is no
    // `UPDATE observation_record` anywhere in `crates/`, and the documenting transaction issues
    // one SELECT on `observation_record` plus N INSERTs on `declared_attribute`. So these tests
    // CANNOT FAIL as the tree stands — which is Epic 5's dominant defect class, and exactly why
    // the story pairs them with a SOURCE gate (`observed-immutable`). Their job is to red on the
    // day a future story adds the write; the gate's job is to red on the day it is merely
    // AUTHORED. Neither subsumes the other, and mutation M1b measures that.

    use opencmdb_core::observation::{
        ConnectorId, Fact, L2DomainId, ObsId, Observation, Scope, VantageId,
    };

    /// Connect, migrate, and empty every table this story touches — children before parents,
    /// because `identity_link.observation_id` is a foreign key (`0003_resolver_guards.sql:43-45`).
    ///
    /// `None` when `DATABASE_URL` is unset, and then the caller returns: ⚠️ a green local suite
    /// says NOTHING about this story, whose entire deliverable is database-backed.
    async fn nfr5_pool() -> Option<MySqlPool> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping story 6.3 DB test: DATABASE_URL unset");
            return None;
        };
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        for statement in [
            "DELETE FROM declared_attribute",
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM interface",
            "DELETE FROM observation_record",
        ] {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }
        Some(pool)
    }

    /// The story's subject: a sighting carrying every declarable kind — `ipv4`, `hostname` AND
    /// `mac` — plus a **non-NULL `raw`** and a microsecond-precision instant.
    ///
    /// ⚠️ Both of those last two are load-bearing rather than decoration. `raw` is nullable, and
    /// a fixture leaving it `NULL` would put a column in the comparison that carries nothing —
    /// mutation M6 is the control that measures it. The `mac` is what gives the observation an
    /// L1 key, so `resolve` places it on a real interface and *"the link is intact"* has a link
    /// to be intact about.
    fn nfr5_observation(ip: &str, hostname: &str, mac: [u8; 6]) -> Observation {
        use opencmdb_core::observation::{HostnameSource, MacAddr};
        Observation {
            obs_id: ObsId::from_uuid(uuid::Uuid::now_v7()),
            connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()),
            observed_at: chrono::DateTime::from_timestamp_micros(1_700_000_000_123_456)
                .expect("in range"),
            scope: Scope {
                l2_domain: L2DomainId::from_uuid(uuid::Uuid::nil()),
                vantage: VantageId::from_uuid(uuid::Uuid::nil()),
            },
            facts: vec![
                Fact::IpV4 {
                    addr: ip.parse().expect("an ipv4"),
                },
                Fact::Hostname {
                    name: hostname.into(),
                    source: HostnameSource::Dns,
                },
                Fact::Mac {
                    addr: MacAddr(mac),
                    locally_administered: false,
                },
            ],
            raw: Some("{\"opaque\":\"provenance blob, é\"}".into()),
        }
    }

    /// Seed one observation and run the identity pass over it, so that a real `identity_link`
    /// exists before the gesture under test.
    async fn seed_and_resolve(pool: &MySqlPool, observation: &Observation) {
        repo::insert_observation(pool, observation)
            .await
            .expect("ingest");
        let mut conn = pool.acquire().await.expect("acquire");
        crate::resolver::resolve(&mut conn, std::slice::from_ref(observation))
            .await
            .expect("resolve");
    }

    /// Story 6.3, AC1 — a SUCCESSFUL documenting gesture leaves the observed side untouched.
    ///
    /// The observation row is compared on **all seven columns** (through
    /// `repo::snapshot_observation_records`, which reads them as the server renders them rather
    /// than round-tripping through Rust types), and the link is compared through
    /// `load_current_links_for_observation`, whose `PersistedLink` **carries the row `id`** —
    /// which is what tells *"the same link"* from *"an equal link written afresh"*.
    ///
    /// ⚠️ Both populations are asserted NON-EMPTY first. Comparing two empty vectors is the
    /// vacuous-guard shape this story exists to avoid; mutation M1 would stay green under it.
    #[tokio::test]
    async fn documenting_leaves_the_observation_and_its_link_untouched() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = nfr5_pool().await else {
            return;
        };
        let observation = nfr5_observation("192.0.2.10", "nas", [0x02, 0, 0, 0, 0, 0x10]);
        let subject = observation.obs_id;
        seed_and_resolve(&pool, &observation).await;

        let observations_before = repo::snapshot_observation_records(&pool)
            .await
            .expect("snapshot observations");
        let links_before = repo::load_current_links_for_observation(&pool, subject)
            .await
            .expect("load links");
        assert_eq!(
            observations_before.len(),
            1,
            "the comparison needs a row to compare"
        );
        assert!(
            observations_before[0].raw.is_some(),
            "a NULL raw would put a column in the comparison that carries nothing"
        );
        assert!(
            !links_before.is_empty(),
            "the pass must have placed the sighting, or 'the link is intact' asserts nothing"
        );

        let response = app(pool.clone(), config(true, Some(pair())))
            .oneshot(document_post(
                Some(&basic_header("op", "s3cret")),
                &format!("subject={}", subject.as_uuid()),
            ))
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::CREATED,
            "the gesture happened"
        );

        let observations_after = repo::snapshot_observation_records(&pool)
            .await
            .expect("snapshot observations");
        let links_after = repo::load_current_links_for_observation(&pool, subject)
            .await
            .expect("load links");
        assert_eq!(
            observations_before, observations_after,
            "documenting moved a byte of the observed record"
        );
        assert_eq!(
            links_before, links_after,
            "documenting disturbed the identity link"
        );
    }

    /// Story 6.3, AC1 — the same two comparisons across a **REFUSED** gesture.
    ///
    /// 🔴 The refusal paths are where a *"mark it attempted"* write would land, and they are the
    /// half a naive after-only check cannot see. ⚠️ And they are harder to guard than they look:
    /// a write placed on the handler's own transaction is ROLLED BACK when the refusal returns
    /// without committing, so it is invisible here by construction — measured at this story's
    /// validation, and the reason mutation M4′ writes on its OWN connection instead.
    #[tokio::test]
    async fn a_refused_documenting_gesture_leaves_the_observation_and_its_link_untouched() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = nfr5_pool().await else {
            return;
        };
        let observation = nfr5_observation("192.0.2.20", "printer", [0x02, 0, 0, 0, 0, 0x20]);
        let subject = observation.obs_id;
        seed_and_resolve(&pool, &observation).await;

        // Document it once, so the SECOND attempt is the 409 the unique index raises.
        let first = app(pool.clone(), config(true, Some(pair())))
            .oneshot(document_post(
                Some(&basic_header("op", "s3cret")),
                &format!("subject={}", subject.as_uuid()),
            ))
            .await
            .unwrap();
        assert_eq!(first.status(), StatusCode::CREATED);

        let observations_before = repo::snapshot_observation_records(&pool)
            .await
            .expect("snapshot observations");
        let links_before = repo::load_current_links_for_observation(&pool, subject)
            .await
            .expect("load links");
        assert!(!observations_before.is_empty() && !links_before.is_empty());

        let refused = app(pool.clone(), config(true, Some(pair())))
            .oneshot(document_post(
                Some(&basic_header("op", "s3cret")),
                &format!("subject={}", subject.as_uuid()),
            ))
            .await
            .unwrap();
        assert_eq!(
            refused.status(),
            StatusCode::CONFLICT,
            "the second adoption is refused, which is the point of the fixture"
        );

        assert_eq!(
            observations_before,
            repo::snapshot_observation_records(&pool)
                .await
                .expect("snapshot observations"),
            "a refused gesture moved a byte of the observed record"
        );
        assert_eq!(
            links_before,
            repo::load_current_links_for_observation(&pool, subject)
                .await
                .expect("load links"),
            "a refused gesture disturbed the identity link"
        );
    }

    /// Story 6.3, AC1 — the second refusal shape: a subject that projects to NOTHING answers
    /// 422-domain, and leaves the observed side untouched too. Distinct from the 409 above
    /// because it refuses BEFORE the write loop rather than inside it.
    #[tokio::test]
    async fn a_nothing_to_document_refusal_leaves_the_observation_untouched() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = nfr5_pool().await else {
            return;
        };
        // Rtt-only: no declarable field, and no L1 key either, so the pass abstains.
        let observation = Observation {
            obs_id: ObsId::from_uuid(uuid::Uuid::now_v7()),
            connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()),
            observed_at: chrono::DateTime::from_timestamp_micros(1_700_000_500_000_001)
                .expect("in range"),
            scope: Scope {
                l2_domain: L2DomainId::from_uuid(uuid::Uuid::nil()),
                vantage: VantageId::from_uuid(uuid::Uuid::nil()),
            },
            facts: vec![Fact::Rtt { millis: 12 }],
            raw: Some("{\"opaque\":\"rtt only\"}".into()),
        };
        let subject = observation.obs_id;
        seed_and_resolve(&pool, &observation).await;

        let before = repo::snapshot_observation_records(&pool)
            .await
            .expect("snapshot observations");
        let links_before = repo::load_current_links_for_observation(&pool, subject)
            .await
            .expect("load links");
        assert_eq!(before.len(), 1, "the comparison needs a row to compare");
        // ⚠️ AC1 promises BOTH comparisons across BOTH refusals, and this one was missing until
        // story 6.3's code review. The reviewing layer excused it as probably vacuous — an
        // Rtt-only sighting having no link — and that excuse is REFUTED by measurement: the pass
        // abstains and writes a CURRENT abstention link (`absence_of_proof`), so the comparison
        // has a row to compare and its absence was a real gap.
        assert_eq!(
            links_before.len(),
            1,
            "an Rtt-only sighting still carries a current abstention link, or this compares nothing"
        );

        let refused = app(pool.clone(), config(true, Some(pair())))
            .oneshot(document_post(
                Some(&basic_header("op", "s3cret")),
                &format!("subject={}", subject.as_uuid()),
            ))
            .await
            .unwrap();
        assert_eq!(refused.status(), StatusCode::UNPROCESSABLE_ENTITY);

        assert_eq!(
            before,
            repo::snapshot_observation_records(&pool)
                .await
                .expect("snapshot observations"),
            "a nothing-to-document refusal moved a byte of the observed record"
        );
        assert_eq!(
            links_before,
            repo::load_current_links_for_observation(&pool, subject)
                .await
                .expect("load links"),
            "a nothing-to-document refusal disturbed the identity link"
        );
    }

    // ─── Story 6.3 — NFR5's FIRST assertion, re-asserted through the NEW write path ──────
    //
    // *"Ingesting an observation that contradicts a declared field leaves that field unchanged
    // and opens a divergence"* (`prd.md:1212-1213`).
    //
    // 🔴 The two halves are NOT equally reachable, and this story measures the boundary rather
    // than hiding it. *Unchanged* holds unconditionally and is the invariant NFR5 is actually
    // about. *A divergence opens* does NOT hold while the documented sighting is still in the
    // store: `reconcile` treats two disagreeing in-perimeter sightings as a CONFLICT, drops the
    // field, and abstains twice — FR16 working, *never picked, never merged*. Producing a gap
    // therefore requires removing the documented sighting, which is Guy's arbitration of
    // 2026-08-15 and which the third test below performs explicitly.

    /// Ingest one batch through the REAL pass — `poll_ingest_resolve`, driven by the committed
    /// `FixtureConnector` — rather than through `repo::insert_observation`.
    ///
    /// ⚠️ This is what makes the test a re-assertion *"through the new write path"*: the shipped
    /// ARP/ping connector emits only `IpV4` + `Rtt` and can never produce a contradicting
    /// `hostname`, so a fixture connector is the only way to drive a real ingestion of one.
    async fn ingest_through_the_pass(pool: &MySqlPool, observations: Vec<Observation>) {
        use opencmdb_core::observation::{Capabilities, FactKind};
        use std::collections::BTreeSet;
        let capabilities = Capabilities {
            as_of: chrono::DateTime::from_timestamp(1_700_000_000, 0).expect("in range"),
            kinds: BTreeSet::from([FactKind::IpV4, FactKind::Hostname, FactKind::Mac]),
        };
        let mut connector = crate::fixture_connector::FixtureConnector::from_observations(
            ConnectorId::from_uuid(uuid::Uuid::nil()),
            capabilities,
            vec![Scope {
                l2_domain: L2DomainId::from_uuid(uuid::Uuid::nil()),
                vantage: VantageId::from_uuid(uuid::Uuid::nil()),
            }],
            "story 6.3 contradicting ingestion",
            observations,
        )
        .expect("the in-memory stream must load");
        let outcome = crate::scan_pass::poll_ingest_resolve(
            &mut connector,
            chrono::DateTime::from_timestamp(1_700_000_900, 0).expect("in range"),
            pool,
        )
        .await;
        assert!(
            outcome.ingested > 0,
            "the pass must actually have ingested, or the test measures nothing"
        );
    }

    /// Document one sighting through the route and return `(subject, entity_id, observation)`.
    async fn document_one(pool: &MySqlPool, observation: Observation) -> (ObsId, String) {
        let subject = observation.obs_id;
        seed_and_resolve(pool, &observation).await;
        let response = app(pool.clone(), config(true, Some(pair())))
            .oneshot(document_post(
                Some(&basic_header("op", "s3cret")),
                &format!("subject={}", subject.as_uuid()),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let entity_id = body_text(response)
            .await
            .rsplit(' ')
            .next()
            .expect("entity id in body")
            .to_string();
        (subject, entity_id)
    }

    /// Story 6.3, AC2 — an ingestion that CONTRADICTS a declared field changes **nothing** on the
    /// declared side, on all seven columns including `updated_at`.
    ///
    /// 🔑 `updated_at` is the column that matters most here: a silent re-write which happened to
    /// preserve the value would still move it, and that is the drift this assertion exists to
    /// catch. The comparison goes through the ONE sanctioned provenance reader, widened to seven
    /// columns at this story (`repo::read_declared_provenance_for_test`).
    #[tokio::test]
    async fn a_contradicting_ingestion_leaves_the_declared_side_byte_identical() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = nfr5_pool().await else {
            return;
        };
        let documented = nfr5_observation("192.0.2.30", "nas", [0x02, 0, 0, 0, 0, 0x30]);
        let (_subject, entity_id) = document_one(&pool, documented).await;

        let declared_before = repo::read_declared_provenance_for_test(&pool, &entity_id)
            .await
            .expect("read declared");
        assert_eq!(
            declared_before.len(),
            3,
            "ipv4 + hostname + mac were documented, or the comparison is thinner than it looks"
        );

        // The contradiction: same address, a DIFFERENT hostname, through the real pass.
        let contradicting = nfr5_observation("192.0.2.30", "intruder", [0x02, 0, 0, 0, 0, 0x30]);
        ingest_through_the_pass(&pool, vec![contradicting]).await;

        assert_eq!(
            declared_before,
            repo::read_declared_provenance_for_test(&pool, &entity_id)
                .await
                .expect("read declared"),
            "the scanner altered a declared field — NFR5's first assertion is broken"
        );
    }

    /// Story 6.3, AC2 — the boundary, pinned with BOTH numbers: while the documented sighting is
    /// still in the store, a contradicting ingestion opens **no gap** and abstains **twice**.
    ///
    /// ⚠️ Do NOT relax this to `gaps.is_empty()`: the pair `(0, 2)` and its cause breakdown are
    /// what distinguish *"the product correctly refused to pick"* from *"the product saw
    /// nothing"*. A third disagreeing sighting is asserted too, because the conflict is counted
    /// once per FIELD and not once per sighting — counter-intuitive, and pinned for that reason.
    #[tokio::test]
    async fn two_disagreeing_sightings_abstain_rather_than_open_a_divergence() {
        use opencmdb_core::gap::AbstentionCause;
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = nfr5_pool().await else {
            return;
        };
        let documented = nfr5_observation("192.0.2.40", "nas", [0x02, 0, 0, 0, 0, 0x40]);
        let documented_copy = documented.clone();
        let (_subject, entity_id) = document_one(&pool, documented).await;

        let contradicting = nfr5_observation("192.0.2.40", "intruder", [0x02, 0, 0, 0, 0, 0x40]);
        let contradicting_copy = contradicting.clone();
        ingest_through_the_pass(&pool, vec![contradicting]).await;

        let declared = declared_pairs(&pool, &entity_id).await;
        let two = opencmdb_core::gap::reconcile(
            ("ipv4", "192.0.2.40"),
            &declared,
            &[documented_copy.clone(), contradicting_copy.clone()],
        );
        assert!(
            two.gaps.is_empty(),
            "two disagreeing sightings must not pick one: {:?}",
            two.gaps
        );
        assert_eq!(
            two.abstention_count(),
            2,
            "the conflict and the now-unobserved field: {:?}",
            two.abstentions
        );
        assert_eq!(
            two.abstentions
                .get(&AbstentionCause::ConflictingObservations),
            Some(&1),
            "one conflicting field: {:?}",
            two.abstentions
        );
        assert_eq!(
            two.abstentions.get(&AbstentionCause::NoObservedValue),
            Some(&1),
            "and the declared hostname then has no observed value: {:?}",
            two.abstentions
        );

        // A THIRD disagreeing sighting does not add an abstention: the conflict is per FIELD.
        let third = nfr5_observation("192.0.2.40", "impostor", [0x02, 0, 0, 0, 0, 0x40]);
        let three = opencmdb_core::gap::reconcile(
            ("ipv4", "192.0.2.40"),
            &declared,
            &[documented_copy, contradicting_copy, third],
        );
        assert!(three.gaps.is_empty());
        assert_eq!(
            three.abstention_count(),
            2,
            "the conflict is counted once per FIELD, not once per sighting: {:?}",
            three.abstentions
        );
    }

    /// Story 6.3, AC2 — and the divergence DOES open once the documented sighting is gone.
    ///
    /// 🔴 **The DELETE is this test's own gesture and no production code path performs it**
    /// (Guy's arbitration, 2026-08-15). It models *the old sighting aged out*: while the sighting
    /// that supplied the declared value is still in the store, every contradicting ingestion
    /// conflicts with it, so a gap is unreachable through the documenting gesture. The
    /// alternatives were refused with their reasons in the story file — seeding the declared row
    /// manually would lose *"through the new write path"*, and dropping this half would stop
    /// measuring drift detection, which D22 makes the property that keeps NFR5 alive.
    ///
    /// 🔴 **The abstention count depends on WHAT THE NEW SIGHTING CARRIES, not on the shape —
    /// measured here, against a prediction that said otherwise.** The story file predicted
    /// `(1, 1)` for this shape; that figure was taken with a contradicting sighting carrying **no
    /// MAC**, and it does not survive a realistic re-scan. Both cases are pinned below:
    ///
    /// - the same NIC seen again under a new hostname (same MAC) → **`(1, 0)`**: `mac` is still
    ///   observed, so nothing abstains;
    /// - a contradicting sighting that carries no MAC → **`(1, 1)`**, the spare abstention being
    ///   the declared `mac` with no observed value.
    ///
    /// ⚠️ So story 6.2's oracle (`abstention_count == 0`) is CORRECT for the first and WRONG for
    /// the second. *A figure quoted without the fixture that produced it is not a measurement*,
    /// and pinning only one of these two would have hidden that.
    #[tokio::test]
    async fn the_divergence_opens_once_the_documented_sighting_is_gone() {
        use opencmdb_core::gap::AbstentionCause;
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = nfr5_pool().await else {
            return;
        };
        let documented = nfr5_observation("192.0.2.50", "nas", [0x02, 0, 0, 0, 0, 0x50]);
        let (subject, entity_id) = document_one(&pool, documented).await;

        let contradicting = nfr5_observation("192.0.2.50", "intruder", [0x02, 0, 0, 0, 0, 0x50]);
        let contradicting_copy = contradicting.clone();
        ingest_through_the_pass(&pool, vec![contradicting]).await;

        let declared_before = repo::read_declared_provenance_for_test(&pool, &entity_id)
            .await
            .expect("read declared");

        // Remove the documented sighting — children before parents (the FK from identity_link).
        let subject_text = subject.as_uuid().to_string();
        for statement in [
            "DELETE FROM link_candidate WHERE link_id IN \
             (SELECT id FROM identity_link WHERE observation_id = ?)",
            "DELETE FROM identity_link WHERE observation_id = ?",
            "DELETE FROM observation_record WHERE id = ?",
        ] {
            sqlx::query(statement)
                .bind(&subject_text)
                .execute(&pool)
                .await
                .expect("age the old sighting out");
        }

        // The declared side survives the removal untouched — `origin_obs_id` carries no FK.
        assert_eq!(
            declared_before,
            repo::read_declared_provenance_for_test(&pool, &entity_id)
                .await
                .expect("read declared"),
            "removing the source sighting must not touch the declared record"
        );

        let declared = declared_pairs(&pool, &entity_id).await;
        let one =
            opencmdb_core::gap::reconcile(("ipv4", "192.0.2.50"), &declared, &[contradicting_copy]);
        assert_eq!(one.gaps.len(), 1, "the divergence opens: {:?}", one.gaps);
        assert_eq!(one.gaps[0].field, "hostname");
        assert_eq!(one.gaps[0].declared, "nas");
        assert_eq!(one.gaps[0].observed, "intruder");
        assert_eq!(
            one.abstention_count(),
            0,
            "the re-scan still sees the same MAC, so nothing abstains: {:?}",
            one.abstentions
        );

        // The CONTRAST, and it is why the figure above needs its fixture stated: strip the MAC
        // from the contradicting sighting and the declared `mac` loses its observed value, so the
        // same shape measures (1, 1). Neither number is a property of the shape alone.
        let mut mac_less = nfr5_observation("192.0.2.50", "intruder", [0x02, 0, 0, 0, 0, 0x50]);
        mac_less
            .facts
            .retain(|fact| !matches!(fact, Fact::Mac { .. }));
        let without_mac =
            opencmdb_core::gap::reconcile(("ipv4", "192.0.2.50"), &declared, &[mac_less]);
        assert_eq!(
            without_mac.gaps.len(),
            1,
            "the divergence still opens: {:?}",
            without_mac.gaps
        );
        assert_eq!(
            without_mac.abstention_count(),
            1,
            "and now the declared mac has no observed value: {:?}",
            without_mac.abstentions
        );
        assert_eq!(
            without_mac
                .abstentions
                .get(&AbstentionCause::NoObservedValue),
            Some(&1),
            "named, not merely counted: {:?}",
            without_mac.abstentions
        );
    }

    /// The declared `(key, value)` pairs of one entity, as the reconcile consumes them.
    async fn declared_pairs(pool: &MySqlPool, entity_id: &str) -> Vec<(String, String)> {
        repo::load_declared_attributes(pool)
            .await
            .expect("load declared")
            .into_iter()
            .filter(|(e, _, _)| e == entity_id)
            .map(|(_, k, v)| (k, v))
            .collect()
    }

    /// 🔴 Story 6.2, AC2 (M6) — a subject whose facts project to NOTHING (an Rtt-only sighting)
    /// answers 422 `NothingToDocument` through the STORE-BACKED port, and writes no row. This is
    /// the store-level carrier the handler-arm test cannot be (it injects the refusal directly);
    /// without it, removing the port's empty-projection guard is invisible (validation: a guard
    /// placed where the defect cannot occur reads as coverage and is none).
    #[tokio::test]
    async fn an_empty_projection_subject_answers_nothing_to_document_and_writes_nothing() {
        use opencmdb_core::observation::{
            ConnectorId, Fact, L2DomainId, ObsId, Observation, Scope, VantageId,
        };
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping empty-projection DB test: DATABASE_URL unset");
            return;
        };
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        for statement in [
            "DELETE FROM declared_attribute",
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM interface",
            "DELETE FROM observation_record",
        ] {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }
        let subject = ObsId::from_uuid(uuid::Uuid::now_v7());
        let observation = Observation {
            obs_id: subject,
            connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()),
            observed_at: chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
            scope: Scope {
                l2_domain: L2DomainId::from_uuid(uuid::Uuid::nil()),
                vantage: VantageId::from_uuid(uuid::Uuid::nil()),
            },
            facts: vec![Fact::Rtt { millis: 3 }], // ignored by `gap::project` → empty projection
            raw: None,
        };
        repo::insert_observation(&pool, &observation)
            .await
            .expect("ingest");

        let response = app(pool.clone(), config(true, Some(pair())))
            .oneshot(document_post(
                Some(&basic_header("op", "s3cret")),
                &format!("subject={}", subject.as_uuid()),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body_text(response).await,
            "nothing to document: the observation carries no declarable field"
        );
        assert_eq!(
            repo::count_declared_attributes(&pool).await.expect("count"),
            0,
            "nothing was written"
        );
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
            // No Basic pair here: this test is about the Bearer branch and deny-by-default.
            app(pool.clone(), config(false, None)).oneshot(request)
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

    #[test]
    fn default_bind_is_all_interfaces_port_8080() {
        let config = config::Config::builder()
            .set_default("bind", "0.0.0.0:8080")
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(config.get_string("bind").unwrap(), "0.0.0.0:8080");
    }

    /// 🔴 **`/triage` through the REAL router, on rows a real database holds** (story 6b.4).
    ///
    /// # Why this test exists, and what its absence let through
    ///
    /// The story replaced the whole body of `/triage`, and **all 387 bin tests stayed green** — the
    /// helper that claimed to cover it, `page::tests::triage_html`, rendered `GapFragment` directly
    /// and never touched the route. *A helper named for a route it does not serve is the dominant
    /// defect class wearing a filename, and reading it could not find that, because it is correct
    /// about what it renders.* This one drives the router.
    ///
    /// ⚠️ **DB-gated, and that is not a formality here**: `/triage` reads three tables. A local run
    /// with `DATABASE_URL` unset returns early and proves nothing about any of this — the tell is
    /// the clock, ~0.06 s against ~5 s.
    #[tokio::test]
    async fn the_triage_route_renders_the_queue_and_both_photos() {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping the triage route test: DATABASE_URL unset");
            return;
        };
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        // Children first: `identity_link` has a foreign key onto `observation_record`. Literal
        // statements, one per table: sqlx 0.9 refuses a `format!`-built query (`SqlSafeStr`), and
        // that refusal is a guard worth respecting rather than routing around.
        for statement in [
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM interface",
            "DELETE FROM declared_attribute",
            "DELETE FROM observation_record",
        ] {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }

        // 🔴 TWO entities, and the second is not decoration. The first version seeded ONE, so the
        // response held ONE queue row — and **with n = 1 a sort order is structurally
        // unobservable**, which the code review measured: no age-sort mutation could red this test
        // whatever its shape, while the mutation table recorded it as a carrier.
        // *A fixture of one cannot witness an ordering.*
        let entity = uuid::Uuid::now_v7().to_string();
        let older = uuid::Uuid::now_v7().to_string();
        for (key, value) in [("ipv4", "192.0.2.10"), ("hostname", "nas")] {
            repo::insert_declared_attribute(&pool, &entity, key, value)
                .await
                .expect("declare");
        }
        for (key, value) in [("ipv4", "192.0.2.20"), ("hostname", "old-box")] {
            repo::insert_declared_attribute(&pool, &older, key, value)
                .await
                .expect("declare");
        }
        // One in-perimeter observation that DISAGREES on the hostname — a drift the queue must show.
        let observation = opencmdb_core::observation::Observation {
            obs_id: opencmdb_core::observation::ObsId::from_uuid(uuid::Uuid::now_v7()),
            connector_id: opencmdb_core::observation::ConnectorId::from_uuid(uuid::Uuid::now_v7()),
            observed_at: chrono::DateTime::from_timestamp(1_700_000_000, 0).expect("an instant"),
            scope: opencmdb_core::observation::Scope {
                l2_domain: opencmdb_core::observation::L2DomainId::from_uuid(uuid::Uuid::nil()),
                vantage: opencmdb_core::observation::VantageId::from_uuid(uuid::Uuid::nil()),
            },
            facts: vec![
                opencmdb_core::observation::Fact::IpV4 {
                    addr: "192.0.2.10".parse().unwrap(),
                },
                opencmdb_core::observation::Fact::Hostname {
                    name: "intruder".into(),
                    source: opencmdb_core::observation::HostnameSource::Dns,
                },
            ],
            raw: None,
        };
        repo::insert_observation(&pool, &observation)
            .await
            .expect("observe");
        let ancient = opencmdb_core::observation::Observation {
            obs_id: opencmdb_core::observation::ObsId::from_uuid(uuid::Uuid::now_v7()),
            observed_at: chrono::DateTime::from_timestamp(1_600_000_000, 0).expect("an instant"),
            facts: vec![
                opencmdb_core::observation::Fact::IpV4 {
                    addr: "192.0.2.20".parse().unwrap(),
                },
                opencmdb_core::observation::Fact::Hostname {
                    name: "stranger".into(),
                    source: opencmdb_core::observation::HostnameSource::Dns,
                },
            ],
            ..observation.clone()
        };
        repo::insert_observation(&pool, &ancient)
            .await
            .expect("observe the older one");

        let body = |response: axum::response::Response| async move {
            let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("a rendered body");
            String::from_utf8(bytes.to_vec()).expect("the page is UTF-8")
        };
        let get = |uri: &str| {
            Request::builder()
                .uri(uri.to_string())
                .header(
                    axum::http::header::AUTHORIZATION,
                    basic_header("op", "s3cret"),
                )
                .body(Body::empty())
                .unwrap()
        };

        let response = app(pool.clone(), config(false, Some(pair())))
            .oneshot(get("/triage"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK, "/triage must render");
        let html = body(response).await;

        assert!(
            html.contains("class=\"queue\""),
            "the queue pane must reach the wire"
        );
        assert!(
            html.contains("class=\"photos\""),
            "the two-photo pane must reach the wire"
        );
        assert!(
            html.contains("nas") && html.contains("intruder"),
            "both values must render"
        );
        // AC1: each side carries its own provenance — the declared one's origin, the observed
        // one's connector. Neither pane may render bare.
        assert!(
            html.contains("Entered by hand"),
            "the declared side names its origin"
        );
        // 🔴 The SHORT id, not the whole UUID — and this assertion was written the wrong way round
        // first: it required `contains(<full uuid>)`, which PASSED on the defect. A meta-line
        // reading *"cccccccc-0000-0000-0000-00000000unif · il y a 4 min"* tells the operator
        // nothing, and a test that pins the ugly thing is a test that requires it.
        let short: String = observation
            .connector_id
            .as_uuid()
            .to_string()
            .chars()
            .take(8)
            .collect();
        assert!(
            html.contains(&format!("Source {short}")),
            "the observed side names its source, short enough to read beside the freshness"
        );
        assert!(
            !html.contains(&observation.connector_id.as_uuid().to_string()),
            "the whole UUID must NOT reach the operator's screen — it is an internal id, and \
             there is no connector registry to turn it into a name (registered, story 6b.8)"
        );
        // 🔴 The observed FRESHNESS, computed from the column the story widened the SELECT for.
        // ⚠️ This assertion is here because its absence was MEASURED: mutation M1 froze
        // `observed_at` to the epoch inside `load_observation_facts` and **the whole suite stayed
        // green** — every unit test builds `ObservedBatch` by hand and never reads the column, so
        // the round-trip T1 exists to add was carried by nothing. *A guard that never touches the
        // read cannot notice the read breaking.*
        let observed_days = (chrono::DateTime::from_timestamp(
            i64::try_from(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("after the epoch")
                    .as_secs(),
            )
            .expect("an instant that fits"),
            0,
        )
        .expect("an instant")
            - observation.observed_at)
            .num_days();
        // ⚠️ ±1 day, and the tolerance is the point rather than a shrug. The page read the clock
        // once at render; this assertion reads it again afterwards. Two independent reads that
        // straddle midnight UTC differ by one, and the test would fail with nothing broken —
        // issue #38's family, reintroduced by the first version of this line. What is under test
        // is that the freshness comes from the STORED instant at all, which a thousand-day-old
        // fixture makes unmistakable either way.
        let rendered_days: Vec<i64> = html
            .split(" d ago")
            .filter_map(|before| {
                before
                    .rsplit(|c: char| !c.is_ascii_digit())
                    .next()
                    .filter(|digits| !digits.is_empty())
                    .and_then(|digits| digits.parse().ok())
            })
            .collect();
        assert!(
            rendered_days.iter().any(|d| (d - observed_days).abs() <= 1),
            "the observed side's freshness must come from the stored `observed_at` — expected \
             about {observed_days} d ago, the page rendered {rendered_days:?}"
        );

        // AC3: the sort is OFF unless asked, and the toggle is offered.
        assert!(
            html.contains("aria-pressed=\"false\""),
            "the age sort is offered and OFF by default — the ban is not that age is hidden, it \
             is that it is never brandished"
        );
        let sorted = body(
            app(pool.clone(), config(false, Some(pair())))
                .oneshot(get("/triage?sort=age"))
                .await
                .unwrap(),
        )
        .await;
        assert!(
            sorted.contains("aria-pressed=\"true\""),
            "asking for the sort turns it on"
        );

        // 🔑 With two rows the ORDER is observable, which a one-row fixture could never witness:
        // oldest first when the sort is on, the queue's own order when it is off.
        let first_row = |page: &str| -> String {
            page.split("class=\"entity mono\">")
                .nth(1)
                .and_then(|rest| rest.split('<').next())
                .unwrap_or_default()
                .to_string()
        };
        assert_eq!(
            first_row(&html),
            "192.0.2.10",
            "off by default: the queue keeps its own order, the recently-seen entity first"
        );
        assert_eq!(
            first_row(&sorted),
            "192.0.2.20",
            "sorting by age puts the OLDEST first — and this assertion is unreachable with a \
             one-row fixture, which is why the second entity is seeded"
        );

        for statement in [
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM interface",
            "DELETE FROM declared_attribute",
            "DELETE FROM observation_record",
        ] {
            sqlx::query(statement).execute(&pool).await.ok();
        }
    }
}
