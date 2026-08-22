//! The self-diagnostic screen: what the product knows about ITSELF, measured at runtime.
//!
//! Story 6b.9. This is the second screen of Epic 6b whose centre is real rather than example, and
//! the first that reads the database **about itself**. Its whole value is that the operator can
//! believe it: a single decorative row destroys the other sixteen.
//!
//! ⚠️ **SEVENTEEN rows, not the reference mock's sixteen** — the *Journal* group carries five where
//! the mock carries four, because file retention and rotation are two separate facts here and only
//! one of them applies when nothing is written to a file. This read *"the other fifteen"* until the
//! code review: **two layers counted seventeen independently, one from the diff alone and one from
//! the live render**, and no test pinned the total. `tests::ROWS` and the TWO guards on it — one
//! over the built view, one over the rendered HTML — are what stop that
//! recurring.
//!
//! # 🔴 Three shapes here are arbitrations, not preferences
//!
//! **(1) — the story calls it arbitration `(c′)`, and the code cites it by that name — [`ScanReport`]
//! exists because `MAX(observed_at)` is NOT the last scan.** ⚠️ The two labels are one decision: the
//! letter is the story file's (options (a) to (d) were weighed there and (c′) chosen), the number is
//! this list's. The blind review layer found `(c′)` cited FIVE times in code and defined nowhere in
//! it — *a citation a reader cannot resolve from the material in front of them*. The story's first
//! draft recommended feeding a *last scan* row from `repo::last_observed_at`, and the validation
//! refuted it on a running binary: a scan RAN and SUCCEEDED over an empty subnet and
//! `MAX(observed_at)` did not move — the row would have read *"2 years 9 months ago"* thirty
//! seconds after the scan. And when a scan does find something, `observed_at` is the instant handed
//! into `spawn_startup_scan` at boot, one value for the whole sweep, so the row would have shown the
//! BOOT instant. 🔑 *The draft's own recommendation was the defect*, and what replaces it is scoped
//! to the running process and says so.
//!
//! **(2) [`SecurityPosture`] carries no free text, and that is AC2's real carrier.** A
//! forbidden-sentence guard was measured green on three of the reference mock's claims planted as a
//! French literal — *a literal is not a key*, and an enumeration of forbidden sentences, in two
//! languages, against paraphrase, cannot claim the completeness of a property (story 5.12's
//! sentence, third application in this epic). Every field below is a `bool` or a DERIVED list, so a
//! false security claim **cannot be typed into this screen**. The word guard is a second line.
//!
//! **(3) [`LogDescriptor`] is what `init_tracing` INSTALLED, never what the environment holds.**
//! Built the other way — an `AppConfig` field per variable, which is what the story first
//! prescribed — the group was measured shipping two false rows: file logging OFF while the screen
//! named a directory, and a level the variable named but the subscriber did not install. That is story 6b.8's own finding (`OPENCMDB_SCAN_CIDR=nonsense` rendered as
//! an in-force perimeter), one story later, in a design that cited it twice.
//! 🔑 *The environment is the request; the descriptor is the answer, and a diagnostic screen must
//! show the answer.*

use std::sync::{Arc, RwLock};
use std::time::Duration;

use askama::Template;
use axum::response::{Html, IntoResponse, Response};
use opencmdb_core::observation::Timestamp;

/// What one completed scan-and-resolve pass did, as the pass itself measured it.
///
/// ⚠️ **Scoped to the running process, and the screen says so.** A figure that silently resets at
/// every restart while reading as an all-time fact is the growing-counter family in mirror image —
/// the UX specification's first hard ban, and story 5.14b's arbitration 13, which settled that a
/// number must carry an honest unit. *"Since this boot"* is that unit here.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ScanReport {
    /// The instant the SOURCE dated its own descriptor — `PollSummary::capabilities.as_of`.
    ///
    /// 🔑 **This value already existed and was thrown away.** Story 6b.8 bound and traced it after
    /// finding it discarded by an `if let Err(...)`; it is FR7's dated descriptor, and it is the
    /// scan's own instant rather than an observation's. The first draft of this story never
    /// mentioned it and reached for `MAX(observed_at)` instead.
    pub(crate) as_of: Timestamp,
    /// How long poll + ingest + resolve took, measured around the whole pass.
    pub(crate) duration: Duration,
    /// Observations the database accepted during that pass.
    pub(crate) ingested: usize,
    /// Whether the identity pass ran to completion over what landed.
    pub(crate) resolved: bool,
}

/// Where the last pass's report is published for the request path to read.
///
/// ⚠️ **The lock is the only uncarried part of arbitration (c′), and its size is stated rather than
/// implied.** [`crate::scan_pass::poll_ingest_resolve`] MEASURES the report and is driven end to end
/// by the committed `FixtureConnector`, so the measurement itself is tested; what no test reaches is
/// the `Arc` clone that `spawn_startup_scan` carries into its detached thread — the same region
/// `scan_pass`'s module doc already names as unassertable.
pub(crate) type ScanReportSlot = Arc<RwLock<Option<ScanReport>>>;

/// The product's security posture, as facts rather than as sentences.
///
/// 🔴 **No field here is free text, and that is the point** (AC5). See this module's doc for the
/// measurement that made a word list insufficient.
#[derive(Debug, Clone)]
pub(crate) struct SecurityPosture {
    /// The paths that reach a handler WITHOUT a credential, derived by probing
    /// [`crate::auth::is_public`] rather than copied from it.
    ///
    /// 🔴 **A copied literal was measured drifting.** `is_public` is a PREDICATE
    /// (`path == "/healthz" || path.starts_with("/assets/")`), so there is nothing to enumerate;
    /// widening it with `|| path == "/ipam"` reds exactly one test and leaves
    /// `is_public_is_exactly_healthz_and_assets` GREEN, because that guard is six negative examples
    /// and `/ipam` is not among them. *A screen stating the security perimeter from a hand-copied
    /// literal is the false security claim AC2 forbids, created by the fix for AC2.*
    ///
    /// ⚠️ **Its limit, stated rather than implied**: the probe universe is finite (every
    /// [`crate::screens::Screen`] address plus the product's fixed routes), so this is a tripwire
    /// against a widening that touches a KNOWN address, never a barrier against one that invents a
    /// new prefix. An enumeration cannot claim the completeness of a property.
    pub(crate) public_paths: Vec<&'static str>,
    /// Whether an HTTP Basic pair is configured. **A `bool`, never the credential** — measured: with
    /// both secrets configured, neither appears anywhere in the rendered page, and the leak the
    /// story prescribed a guard for is a compile error instead, the value not being in scope.
    pub(crate) basic_configured: bool,
    /// Whether the `/metrics` scrape token is configured.
    ///
    /// 🔑 **Read from the ONE reader.** `scrape_authorized` used to read the environment inside the
    /// request path; a second reader for this screen would have re-created story 6b.2's shipped M12
    /// — and here with teeth, because a second reader would naturally have reused
    /// `carries_a_visible_glyph`, which is what `scan_cidr` is filtered with, while
    /// `scrape_authorized` refuses only on `is_empty()`: a token of `" "` would then have protected
    /// `/metrics` while this screen reported it unconfigured. `auth_deny` already took
    /// `State<AppConfig>`; the token moved there and the second reader was never created.
    ///
    /// ⚠️ **The ACTUAL filter is `!is_empty()`** (`AppConfig::from_env`) — the rule moved with the
    /// read rather than being rewritten, so `/metrics` behaves exactly as before. This paragraph
    /// read as a present-tense description of that filter until the blind review layer caught the
    /// tense: *a counterfactual written in the indicative is a false sentence about the code.*
    pub(crate) metrics_token_configured: bool,
}

/// What the logging subsystem actually INSTALLED at boot.
///
/// See this module's doc for why this is a descriptor rather than three configuration fields.
#[derive(Debug, Clone)]
pub(crate) struct LogDescriptor {
    /// The filter directives as `EnvFilter` itself renders them AFTER its lossy parse — so the row
    /// shows what is IN FORCE, never what was typed.
    ///
    /// ⚠️ **The two differ in more than one way, and the first draft of this doc named only one.**
    /// `opencmdb=nope`, `!!!` and an empty value collapse to **`error`**; a bare `notalevel` is read
    /// as a TARGET and becomes `notalevel=trace`, after which the product logs nothing of its own.
    /// Both were measured — see `the_filter_in_force_is_not_the_variable_that_was_typed` — and the
    /// second was found by BOOTING the binary, after the first had been written down as the whole
    /// story.
    pub(crate) directives: String,
    /// The file sink, or `None` when only stdout is active — including when a directory WAS
    /// configured and could not be built.
    pub(crate) file: Option<LogFile>,
}

/// A file log sink that was really opened.
#[derive(Debug, Clone)]
pub(crate) struct LogFile {
    /// The directory the appender writes into.
    pub(crate) directory: String,
    /// How many rotated files are kept.
    pub(crate) retention: usize,
}

/// The facts the diagnostic screen needs that do not come from the store.
///
/// They enter the router as a PARAMETER (story 6.1's rule — not one test in this crate mutates an
/// environment variable), which is also what lets the screen render with the database down.
#[derive(Clone)]
pub(crate) struct DiagnosticFacts {
    /// What `init_tracing` installed.
    pub(crate) log: Arc<LogDescriptor>,
    /// The security posture, derived once at boot.
    pub(crate) security: Arc<SecurityPosture>,
    /// Where the scan pass publishes its report.
    pub(crate) scan: ScanReportSlot,
}

impl DiagnosticFacts {
    /// Assemble the facts from what the composition root holds.
    pub(crate) fn new(log: LogDescriptor, security: SecurityPosture, scan: ScanReportSlot) -> Self {
        Self {
            log: Arc::new(log),
            security: Arc::new(security),
            scan,
        }
    }
}

/// Every path this product answers on, as the probe universe for [`SecurityPosture::public_paths`].
///
/// 🔑 Built from [`crate::screens::Screen::ALL`] rather than typed out, so a screen added tomorrow
/// is probed the day it exists. The fixed routes are the ones `Screen` does not carry.
fn probe_universe() -> Vec<&'static str> {
    let mut paths: Vec<&'static str> = crate::screens::Screen::ALL
        .iter()
        .map(|screen| screen.href())
        .collect();
    paths.extend([
        "/",
        "/gap",
        "/healthz",
        "/metrics",
        "/document-all",
        "/assets/app.css",
    ]);
    paths
}

/// Derive the security posture by ASKING the code rather than describing it.
pub(crate) fn security_posture(
    basic_configured: bool,
    metrics_token_configured: bool,
) -> SecurityPosture {
    SecurityPosture {
        public_paths: probe_universe()
            .into_iter()
            .filter(|path| crate::auth::is_public(path))
            .collect(),
        basic_configured,
        metrics_token_configured,
    }
}

/// The four groups of the reference mock's diagnostic, in its order.
///
/// 🔑 **An enum and not a `vec![…]` of headings.** The story's mutation M12 — delete one group —
/// predicted a red *"on a count derived from the group enumeration, not on a literal"*, and the
/// validation measured that written the natural way the groups are a vector literal, so the count
/// IS a literal and the prediction's own condition fails. This is the shape that satisfies it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DiagGroup {
    /// *Moteur* — the build, the store, the schema, the last pass.
    Engine,
    /// *Observation* — what has been recorded and how much of it the engine placed.
    Observation,
    /// *Sécurité* — the perimeter, derived; and what the product does not have.
    Security,
    /// *Journal* — what the logging subsystem installed.
    Journal,
}

impl DiagGroup {
    /// The groups in the mock's order.
    pub(crate) const ALL: [DiagGroup; 4] = [
        DiagGroup::Engine,
        DiagGroup::Observation,
        DiagGroup::Security,
        DiagGroup::Journal,
    ];

    /// The i18n key of the group's heading.
    fn heading_key(self) -> &'static str {
        match self {
            DiagGroup::Engine => "diagnostic.group.engine",
            DiagGroup::Observation => "diagnostic.group.observation",
            DiagGroup::Security => "diagnostic.group.security",
            DiagGroup::Journal => "diagnostic.group.journal",
        }
    }
}

/// One rendered row: a label and the measured value beside it.
pub(crate) struct DiagRow {
    /// What the row is, in the operator's language.
    pub(crate) key: String,
    /// The measured value. Rendered in the mono face, as the mock does.
    pub(crate) value: String,
}

/// One rendered group.
pub(crate) struct DiagGroupView {
    /// The group's heading.
    pub(crate) title: String,
    /// Its rows.
    pub(crate) rows: Vec<DiagRow>,
}

/// What the store contributes — `None` when the database cannot be read.
///
/// 🔴 **`None` is a rendered state, not an error page** (AC7). Measured on the shape this screen was
/// told to copy: `/sources`, `/triage` and `/dashboard` each answer a bare `500 internal error` —
/// no shell, no navigation — when the pool is dead. *The one state in which an operator opens a
/// self-diagnostic screen is the one where the store is down*, and half of this screen needs no pool
/// at all. Story 5.14b's arbitration 11 settled *the reader never fails* for an unfamiliar token;
/// nothing had carried it to an unreachable store.
pub(crate) struct StoreFacts {
    /// What `SELECT VERSION()` returned, verbatim.
    ///
    /// ⚠️ **Verbatim by decision.** A live MariaDB 10.11 answers `10.11.16-MariaDB-ubu2204`, not the
    /// mock's tidy `MariaDB 10.11.11`; prettifying it is a transformation of a fact, on the screen
    /// whose subject is facts.
    pub(crate) engine_version: String,
    /// The highest migration version applied, with its description.
    pub(crate) schema: String,
    /// How many migrations the store has applied.
    pub(crate) migrations_applied: usize,
    /// Observations recorded, all time.
    pub(crate) observations: i64,
    /// Sightings the engine placed on an interface.
    pub(crate) placed: i64,
    /// Sightings the engine could not place.
    pub(crate) not_placed: i64,
}

/// Everything `/diagnostic` renders.
pub(crate) struct DiagnosticView {
    /// The four groups, in the mock's order.
    pub(crate) groups: Vec<DiagGroupView>,
    /// True when the store could not be read, so the page can say so once rather than per row.
    pub(crate) store_unreachable: bool,
}

/// How many migrations this binary EMBEDS — the other half of the schema row.
///
/// ⚠️ **The applied count cannot differ from this on a reachable instance, and that is measured**:
/// `run()` migrates before it binds the listener, and both divergent states refuse to boot (*"is
/// partially applied"* / *"was previously applied but is missing"*). So the ratio is a tautology in
/// production and the screen leads with the schema VERSION instead. The count is still rendered,
/// still measured at runtime, and its guard drives the applied side away from this value inside a
/// rolled-back transaction — the only place the two can ever differ.
pub(crate) fn embedded_migrations() -> usize {
    sqlx::migrate!("./migrations").migrations.len()
}

/// Build the diagnostic view. PURE: the caller supplies the store facts and the instant.
///
/// ⚠️ **No clock, no environment read, no query here** — every view builder in this crate is pure
/// (story 6b.4's M6 measured that a feature flag guards a spelling, never the act of reading a
/// clock), and this one additionally takes the security posture and the log descriptor as data so
/// that neither can be re-derived at the point of use.
pub(crate) fn build_diagnostic(
    facts: &DiagnosticFacts,
    store: Option<&StoreFacts>,
    scan: Option<ScanReport>,
    now: chrono::DateTime<chrono::Utc>,
) -> DiagnosticView {
    use rust_i18n::t;
    let unknown = t!("diagnostic.store_unreachable_value").to_string();
    let groups: Vec<DiagGroupView> = DiagGroup::ALL
        .iter()
        .map(|group| DiagGroupView {
            title: t!(group.heading_key()).to_string(),
            rows: match group {
                DiagGroup::Engine => vec![
                    row("diagnostic.version", format!("v{}", crate::VERSION)),
                    row(
                        "diagnostic.engine",
                        store.map_or_else(|| unknown.clone(), |store| store.engine_version.clone()),
                    ),
                    row(
                        "diagnostic.schema",
                        store.map_or_else(
                            || {
                                t!("diagnostic.schema_embedded_only", n = embedded_migrations())
                                    .to_string()
                            },
                            |store| {
                                t!(
                                    "diagnostic.schema_value",
                                    schema = store.schema.clone(),
                                    applied = store.migrations_applied,
                                    embedded = embedded_migrations()
                                )
                                .to_string()
                            },
                        ),
                    ),
                    row("diagnostic.last_pass", last_pass_value(scan, now)),
                ],
                DiagGroup::Observation => vec![
                    row(
                        "diagnostic.observations",
                        store.map_or_else(
                            || unknown.clone(),
                            |store| {
                                t!("diagnostic.observations_value", n = store.observations)
                                    .to_string()
                            },
                        ),
                    ),
                    row(
                        "diagnostic.placed",
                        store.map_or_else(
                            || unknown.clone(),
                            |store| t!("diagnostic.sightings_value", n = store.placed).to_string(),
                        ),
                    ),
                    row(
                        "diagnostic.not_placed",
                        store.map_or_else(
                            || unknown.clone(),
                            |store| {
                                t!("diagnostic.sightings_value", n = store.not_placed).to_string()
                            },
                        ),
                    ),
                    // 🔴 The mock says *"90 days · last state kept"* and NOTHING purges an
                    // observation or a link in this product — story 5.14 measured five runs over one
                    // host leaving five current links. The honest row says so.
                    // ⚠️ And it is QUALIFIED against the Journal group's file retention: the mock's
                    // word means DATA retention, `OPENCMDB_LOG_RETENTION` means log files, and one
                    // word carrying two meanings is what `prd.md:988` forbids.
                    row(
                        "diagnostic.data_retention",
                        t!("diagnostic.data_retention_value").to_string(),
                    ),
                ],
                DiagGroup::Security => security_rows(&facts.security),
                DiagGroup::Journal => journal_rows(&facts.log),
            },
        })
        .collect();
    DiagnosticView {
        groups,
        store_unreachable: store.is_none(),
    }
}

/// One row, with its label resolved.
fn row(label_key: &'static str, value: String) -> DiagRow {
    DiagRow {
        key: rust_i18n::t!(label_key).to_string(),
        value,
    }
}

/// The last pass, or the sentence that says there has not been one since this boot.
///
/// ⚠️ **The scoping is IN THE VALUE, not in a footnote**: an operator reading *"4 min ago · 1823
/// ms"* must not be able to read it as an all-time fact.
fn last_pass_value(scan: Option<ScanReport>, now: chrono::DateTime<chrono::Utc>) -> String {
    use rust_i18n::t;
    match scan {
        None => t!("diagnostic.no_pass_since_boot").to_string(),
        // 🔑 `resolved` is RENDERED and not merely recorded. A pass that ingested observations and
        // whose identity pass was then refused is a real state of this product — `resolve` raises
        // `InstantRegressed` and `ContradictoryObservation`, both of which abandon the pass while
        // the ingest stands — and it is exactly the state an operator opens this screen to find.
        // A field held and never shown would be the *"guard placed where the defect cannot occur"*
        // family in its data form.
        Some(report) => t!(
            if report.resolved {
                "diagnostic.last_pass_value"
            } else {
                "diagnostic.last_pass_unresolved"
            },
            ago = crate::page::relative_time(now, report.as_of),
            ms = report.duration.as_millis(),
            n = report.ingested
        )
        .to_string(),
    }
}

/// The security rows.
///
/// 🔴 **This function takes a [`SecurityPosture`] and NOTHING ELSE, which is the carrier for AC5.**
/// There is no `&str` parameter and no free-text field on the input, so a claim about the product's
/// security cannot be typed into this screen — it can only be DERIVED from a `bool` or from a probed
/// list. See this module's doc for the measurement that made a forbidden-word list insufficient.
fn security_rows(posture: &SecurityPosture) -> Vec<DiagRow> {
    use rust_i18n::t;
    vec![
        row(
            "diagnostic.public_paths",
            if posture.public_paths.is_empty() {
                t!("diagnostic.public_paths_none").to_string()
            } else {
                posture.public_paths.join(" · ")
            },
        ),
        row(
            "diagnostic.basic",
            t!(if posture.basic_configured {
                "diagnostic.configured"
            } else {
                // 🔑 NOT a weakness, and the copy must not read as one: with no pair configured
                // every non-public path answers 401 WITHOUT a challenge (story 6.1's arbitration 6).
                // The product is closed by default, which no mock row says and which an operator
                // cannot learn any other way.
                "diagnostic.basic_unset"
            })
            .to_string(),
        ),
        row(
            "diagnostic.metrics",
            t!(if posture.metrics_token_configured {
                "diagnostic.metrics_token_set"
            } else {
                "diagnostic.metrics_token_unset"
            })
            .to_string(),
        ),
        // The mock asserts a credential store and an encryption key. This product has neither —
        // no table, no crypto call site — and the row says which epics build them rather than
        // describing something that does not exist.
        row(
            "diagnostic.secrets",
            t!("diagnostic.secrets_value").to_string(),
        ),
    ]
}

/// The journal rows, read from what was INSTALLED.
fn journal_rows(log: &LogDescriptor) -> Vec<DiagRow> {
    use rust_i18n::t;
    let (directory, rotation, retention) = match &log.file {
        // 🔴 `None` covers *no directory configured* AND *a directory that could not be opened*,
        // and the second is what the environment-reading design got wrong: it named a directory
        // while `build_file_writer` had already returned `None` and logged
        // *"file logging disabled — cannot use …"*.
        None => (
            t!("diagnostic.log_stdout_only").to_string(),
            t!("diagnostic.log_no_rotation").to_string(),
            t!("diagnostic.log_no_retention").to_string(),
        ),
        Some(file) => (
            file.directory.clone(),
            t!("diagnostic.log_rotation_daily").to_string(),
            t!("diagnostic.log_retention_value", n = file.retention).to_string(),
        ),
    };
    vec![
        row("diagnostic.log_directory", directory),
        row("diagnostic.log_rotation", rotation),
        // ⚠️ `EnvFilter`'s own rendering, AFTER its lossy parse — so this row is the filter IN
        // FORCE. The variable and the filter diverge in two measured ways, and the story's first
        // draft knew only one: `opencmdb=nope` collapses to `error`, and a bare `notalevel` becomes
        // the TARGET `notalevel=trace`. The second was found by booting the binary.
        row("diagnostic.log_level", log.directives.clone()),
        row("diagnostic.log_retention", retention),
        // Nothing stores an error in this product, so *"last error"* is named as absent rather
        // than computed from silence — which would read a clock and INVENT an incident (FR8).
        row(
            "diagnostic.log_last_error",
            t!("diagnostic.log_last_error_value").to_string(),
        ),
    ]
}

/// The two controls the mock puts under the diagnostic, neither of them live.
///
/// 🔴 **A SECOND builder, and not a parameter on `action_bar`.** That one hardcodes the mock's five
/// TRIAGE gestures with triage owners; the diagnostic has two with different owners. Reshaping it
/// into one builder for both would put the triage bar's five-control premise and this one's
/// two-control premise in one place, where a future edit satisfies neither.
pub(crate) fn diagnostic_gestures() -> Vec<crate::page::GestureView> {
    crate::page::planned_gestures(&[
        // *Vérifier maintenant* — an on-demand poll. The scheduler is FR6's.
        ("gesture.check_now", "6.5"),
        // *Exporter le journal* — Epic 13 owns the incident axis this would serve.
        ("gesture.export_log", "13"),
    ])
}

/// The diagnostic screen's body.
#[derive(Template)]
#[template(path = "_diagnostic.html")]
struct DiagnosticBody {
    /// The four groups and the store's reachability.
    view: DiagnosticView,
    /// The two planned controls.
    gestures: Vec<crate::page::GestureView>,
    /// The copy.
    s: DiagnosticStrings,
}

/// The screen's own copy, resolved once.
struct DiagnosticStrings {
    /// The screen's heading.
    title: String,
    /// The line under it.
    lede: String,
    /// Shown once when the store could not be read.
    store_unreachable: String,
    /// The badge on a control that is not built.
    gesture_badge: String,
    /// The one sentence every control points at.
    gesture_not_built: String,
}

/// Resolve the copy.
fn diagnostic_strings() -> DiagnosticStrings {
    use rust_i18n::t;
    DiagnosticStrings {
        title: t!("diagnostic.title").to_string(),
        lede: t!("diagnostic.lede").to_string(),
        store_unreachable: t!("diagnostic.store_unreachable").to_string(),
        gesture_badge: t!("gesture.badge").to_string(),
        gesture_not_built: t!("gesture.not_built").to_string(),
    }
}

/// Read what the store contributes.
///
/// 🔴 **It takes a CONNECTION and not the pool, and that is a guard rather than a style.** The first
/// version took `&MySqlPool`, and its test manufactured the only divergence that can ever exist
/// between the applied and the embedded migration counts inside a rolled-back transaction — which
/// a pool read cannot see. So the test asserted over its own raw SQL while `read_store` was
/// exercised only in the healthy case, where the two counts are equal: **hardcoding the applied
/// count would have left it green.** Epic 5's dominant defect class — *a guard placed where the
/// defect cannot occur reads as coverage and is none* — inside the guard written for this story's
/// own AC1.
///
/// # Errors
///
/// Propagates the database error, which the handler turns into `None` rather than into a 500 —
/// see [`StoreFacts`].
async fn read_store(conn: &mut sqlx::MySqlConnection) -> Result<StoreFacts, sqlx::Error> {
    let (engine_version,): (String,) = sqlx::query_as("SELECT VERSION()")
        .fetch_one(&mut *conn)
        .await?;
    // ⚠️ A RAW read of sqlx's own bookkeeping table, and it trips no gate — measured: eight green
    // with both of these present. sqlx exposes the EMBEDDED set through `Migrator::iter()` but the
    // APPLIED set only behind a `&mut` connection trait method, so a query is the route.
    let (migrations_applied,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = TRUE")
            .fetch_one(&mut *conn)
            .await?;
    let schema: Option<(i64, String)> = sqlx::query_as(
        "SELECT version, description FROM _sqlx_migrations WHERE success = TRUE \
         ORDER BY version DESC LIMIT 1",
    )
    .fetch_optional(&mut *conn)
    .await?;
    let observations = crate::repo::count_observations(&mut *conn).await?;
    let reach = crate::repo::count_engine_reach(&mut *conn).await?;
    // 🔑 SIGHTINGS on both sides, never links — story 5.14b's code review found `placed` and
    // `not_placed` in DIFFERENT units side by side, `join` writing one row per L1 key while an
    // observation abstains once. `count_engine_reach` already counts `DISTINCT observation_id`.
    let placed = reach
        .iter()
        .filter(|row| row.outcome == "match")
        .map(|row| row.count)
        .sum();
    let not_placed = reach
        .iter()
        .filter(|row| row.outcome != "match")
        .map(|row| row.count)
        .sum();
    Ok(StoreFacts {
        engine_version,
        schema: schema.map_or_else(
            || rust_i18n::t!("diagnostic.schema_none").to_string(),
            |(version, description)| format!("{version:04} — {description}"),
        ),
        migrations_applied: migrations_applied.max(0) as usize,
        observations,
        placed,
        not_placed,
    })
}

/// Acquire a connection and read the store through it.
///
/// # Errors
///
/// Propagates the acquire failure or the read's own error.
async fn read_store_from(pool: &sqlx::MySqlPool) -> Result<StoreFacts, sqlx::Error> {
    let mut conn = pool.acquire().await?;
    read_store(&mut conn).await
}

/// The store's contribution, or `None` if it did not answer within `budget`.
///
/// 🔴 **The budget is a PARAMETER so a test can reach the branch that enforces it.** The end-to-end
/// guard drives a pool that REFUSES fast, so `read_store_from` always resolved with an error long
/// before the budget could elapse — the timeout arm was never executed, and its own test comment
/// said so without noticing: *"an acquire timeout well under the handler's budget"* is a
/// description of why that branch is unreachable, not of why the test is sound. Found by the blind
/// review layer, from the diff alone, with no repository access.
///
/// 🔑 *A guard placed where the defect cannot occur reads as coverage and is none* — this epic's
/// dominant class, a FOURTH time inside this story, and the first one no mutation caught.
async fn store_within(pool: &sqlx::MySqlPool, budget: std::time::Duration) -> Option<StoreFacts> {
    match tokio::time::timeout(budget, read_store_from(pool)).await {
        Ok(Ok(facts)) => Some(facts),
        Ok(Err(error)) => {
            tracing::warn!(%error, "the diagnostic could not read the store — rendering without it");
            None
        }
        Err(_elapsed) => {
            tracing::warn!(
                budget_ms = budget.as_millis(),
                "the diagnostic's store read did not answer in time — rendering without it"
            );
            None
        }
    }
}

/// How long the diagnostic waits for the store before rendering without it.
///
/// 🔴 **AC7 was defeated by a timeout nobody had thought about, and the test that proved it was
/// measuring something else.** Rendering *without* the store is what AC7 asks for, and the first
/// implementation did exactly that — after **thirty seconds**, sqlx's default acquire timeout,
/// which the end-to-end test surfaced by taking thirty seconds itself. *A page that eventually
/// says the database is unreachable is not a page an operator can use when it is.*
///
/// ⚠️ Two seconds is a BUDGET, not a health check: a store that is merely slow still answers, and a
/// store that is down costs the operator two seconds rather than half a minute.
const STORE_READ_BUDGET: std::time::Duration = std::time::Duration::from_secs(2);

/// `GET /diagnostic` — the product's report about itself.
///
/// 🔴 **A store failure is a rendered STATE, never a 500** (AC7). See [`StoreFacts`].
pub(crate) async fn diagnostic(
    axum::extract::State(state): axum::extract::State<crate::page::TriageState>,
) -> Response {
    let store = store_within(&state.pool, STORE_READ_BUDGET).await;
    // A poisoned lock must not take the screen down either: the report is a convenience, and the
    // rest of the page is what the operator came for.
    let scan = state.diagnostic.scan.read().ok().and_then(|slot| *slot);
    let body = DiagnosticBody {
        view: build_diagnostic(
            &state.diagnostic,
            store.as_ref(),
            scan,
            crate::page::now_utc(),
        ),
        gestures: diagnostic_gestures(),
        s: diagnostic_strings(),
    };
    match body.render() {
        Ok(body) => Html(crate::page::render_shell(
            crate::page::Shell::new(crate::screens::Screen::Diagnostic, state.perimeter),
            body,
        ))
        .into_response(),
        Err(error) => {
            tracing::error!(%error, "rendering the diagnostic screen");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                crate::page::render_error_body(),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use sqlx::MySqlPool;
    use tower::ServiceExt;

    /// How many rows the screen renders, across all four groups.
    ///
    /// 🔴 **A pinned total, because there was none and the prose was wrong.** The module doc said
    /// *"the other fifteen"* — i.e. sixteen, the reference mock's count — while the screen builds
    /// **seventeen**; the *Journal* group carries five where the mock carries four. Two review
    /// layers counted it independently, one from the diff alone and one from the live render, and
    /// `every_group_of_the_enum_is_rendered_once` could not: it asserts the GROUP count and that no
    /// group is empty, and is satisfied whatever the rows do. *A number nothing counts is a number
    /// that drifts.*
    ///
    /// ⚠️ It lives in the test module rather than beside the builder: a production constant only
    /// the suite reads is dead code, and `clippy -D warnings` says so. The count is a property of
    /// the tests' expectation, not a parameter of the screen.
    const ROWS: usize = 17;

    /// Every row the view holds, across every group.
    fn row_count(view: &DiagnosticView) -> usize {
        view.groups.iter().map(|group| group.rows.len()).sum()
    }

    /// A descriptor a real boot could produce, with values the ENVIRONMENT could not hold.
    ///
    /// 🔑 **That is the whole design of the M3 guard.** `opencmdb=trace,warn` and `/var/log/x` are
    /// handed in as data; if a future edit re-reads `OPENCMDB_LOG` or `OPENCMDB_LOG_DIR` inside the
    /// handler, the process has no such value and the assertions below red. **No environment is
    /// mutated to measure that** — story 6.1's rule.
    fn descriptor() -> LogDescriptor {
        LogDescriptor {
            directives: "opencmdb=trace,warn".to_string(),
            file: Some(LogFile {
                directory: "/var/log/opencmdb-under-test".to_string(),
                retention: 21,
            }),
        }
    }

    fn facts(security: SecurityPosture) -> DiagnosticFacts {
        DiagnosticFacts::new(descriptor(), security, ScanReportSlot::default())
    }

    fn store() -> StoreFacts {
        StoreFacts {
            engine_version: "10.11.16-MariaDB-under-test".to_string(),
            schema: "0005 — document guards".to_string(),
            migrations_applied: 5,
            observations: 137,
            placed: 11,
            not_placed: 26,
        }
    }

    /// Every rendered value, flattened, so an assertion can ask *does the page carry this figure*.
    fn values(view: &DiagnosticView) -> String {
        view.groups
            .iter()
            .flat_map(|group| group.rows.iter())
            .map(|row| format!("{}={}", row.key, row.value))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The value rendered beside ONE label, found by its key.
    ///
    /// 🔑 Needed because a `contains` over the whole page cannot say WHICH row carries a figure —
    /// see the assertion in `every_store_row_carries_the_value_the_store_returned`.
    fn row_value(view: &DiagnosticView, label_key: &str) -> String {
        let label = rust_i18n::t!(label_key).to_string();
        view.groups
            .iter()
            .flat_map(|group| group.rows.iter())
            .find(|row| row.key == label)
            .unwrap_or_else(|| panic!("no row is labelled {label:?}"))
            .value
            .clone()
    }

    fn at(seconds: i64) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp(seconds, 0).expect("representable")
    }

    // ── AC1: every row is MEASURED, not written into a template ──────────────────────────

    /// 🔴 **AC1's carrier, and it is deliberately built on values no template could contain.**
    /// `137`, `10.11.16-MariaDB-under-test` and `0005 — document guards` are supplied by this test;
    /// a row that hardcoded anything would not carry them.
    #[test]
    fn every_store_row_carries_the_value_the_store_returned() {
        let facts = facts(security_posture(true, true));
        let view = build_diagnostic(&facts, Some(&store()), None, at(0));
        let rendered = values(&view);
        for needle in [
            "10.11.16-MariaDB-under-test",
            "0005 — document guards",
            "137",
        ] {
            assert!(
                rendered.contains(needle),
                "the diagnostic must render {needle:?} — it is what the store answered, and a row \
                 that does not carry it is written into the template rather than measured\n\
                 {rendered}"
            );
        }
        // 🔴 **ROW BY ROW, and the first version of this assertion could not tell them apart.**
        // It read `contains("11 sightings") && contains("26 sightings")`, which is satisfied
        // whichever row carries which figure — mutation M6 swapped the two outcome filters and left
        // it GREEN. *An oracle that counts a word counts it wherever it is* (story 6b.6's finding,
        // reproduced here by a guard written after it).
        assert_eq!(
            row_value(&view, "diagnostic.placed"),
            "11",
            "the PLACED row carries the placed count"
        );
        assert_eq!(
            row_value(&view, "diagnostic.not_placed"),
            "26",
            "and the NOT-PLACED row carries the other one"
        );
        // 🔑 **The UNIT is in the label, and it is pinned there.** Story 5.14b's arbitration 13
        // requires these counts to read as *sightings* and never as *devices* — a figure that rises
        // because the product looked many times is the radar's range, not the operator's debt. The
        // value is a bare number because the label already says it; this assertion is what stops
        // the label losing it.
        for key in ["diagnostic.placed", "diagnostic.not_placed"] {
            let label = rust_i18n::t!(key).to_string();
            assert!(
                label.contains("sighting"),
                "{key} must name its unit — it reads {label:?}"
            );
        }
    }

    /// The version comes from the crate, and from the one place the crate reads it.
    #[test]
    fn the_version_row_is_this_build() {
        let facts = facts(security_posture(false, false));
        let view = build_diagnostic(&facts, Some(&store()), None, at(0));
        assert!(
            values(&view).contains(&format!("v{}", crate::VERSION)),
            "the engine group names this build"
        );
    }

    /// ⚠️ **A store that cannot be read is a rendered STATE** (AC7): the rows that need no pool are
    /// still there, and the ones that do say so rather than taking the page down.
    #[test]
    fn a_store_that_cannot_be_read_leaves_the_pool_free_rows_standing() {
        let facts = facts(security_posture(true, false));
        let view = build_diagnostic(&facts, None, None, at(0));
        assert!(view.store_unreachable, "the page says so once");
        let rendered = values(&view);
        assert!(
            rendered.contains(&format!("v{}", crate::VERSION)),
            "the version needs no pool\n{rendered}"
        );
        assert!(
            rendered.contains("/healthz"),
            "the security perimeter needs no pool\n{rendered}"
        );
        assert!(
            rendered.contains("opencmdb=trace,warn"),
            "the journal needs no pool\n{rendered}"
        );
    }

    // ── AC4: the scan report says what the pass DID ───────────────────────────────────────

    /// 🔴 **The report shows what a pass DID, never what the configuration asks for.** This is the
    /// guard story 6b.8's finding demands: a perimeter is configured, no pass has completed, and the
    /// row must say *no pass since this start-up* rather than dressing the configuration as an
    /// event. It is also the case the refuted design could not express — `MAX(observed_at)` cannot
    /// tell *never scanned* from *scanned and nobody answered*.
    #[test]
    fn a_configured_but_never_run_scan_reports_no_pass() {
        let facts = facts(security_posture(false, false));
        let view = build_diagnostic(&facts, Some(&store()), None, at(10_000));
        assert!(
            values(&view).contains(&rust_i18n::t!("diagnostic.no_pass_since_boot").to_string()),
            "with no completed pass the row says so"
        );
    }

    /// A completed pass carries its own instant, its duration, and the scoping that stops it from
    /// reading as an all-time fact.
    #[test]
    fn a_completed_pass_carries_its_instant_its_duration_and_its_scope() {
        let facts = facts(security_posture(false, false));
        let report = ScanReport {
            as_of: at(9_400),
            duration: Duration::from_millis(1_823),
            ingested: 7,
            resolved: true,
        };
        let view = build_diagnostic(&facts, Some(&store()), Some(report), at(10_000));
        let rendered = values(&view);
        assert!(rendered.contains("1823"), "the duration, in ms\n{rendered}");
        assert!(
            rendered.contains("10 min ago"),
            "the SOURCE's own instant, not the boot instant\n{rendered}"
        );
        // 🔴 The scoping is IN the value. A figure that resets at every restart must not read as an
        // all-time fact — the UX specification's first hard ban, in mirror image.
        assert!(
            rendered.contains("since this start-up"),
            "the report is scoped to this process and says so\n{rendered}"
        );
    }

    /// A pass whose identity half was refused is a DIFFERENT sentence, not a silent one.
    #[test]
    fn a_pass_whose_identity_half_was_refused_says_nothing_was_placed() {
        let facts = facts(security_posture(false, false));
        let refused = ScanReport {
            as_of: at(9_900),
            duration: Duration::from_millis(12),
            ingested: 3,
            resolved: false,
        };
        let view = build_diagnostic(&facts, Some(&store()), Some(refused), at(10_000));
        let rendered = values(&view);
        // ⚠️ The DISTINCTIVE half of the sentence, not the whole interpolated string: the first
        // version of this assertion rebuilt the expected text by substituting the placeholders and
        // got `relative_time` wrong (100 seconds is *1 min ago*, not *just now*), so it failed for
        // a reason that had nothing to do with what it measures. *An oracle that restates the
        // expectation instead of measuring the code fails on the restatement.*
        assert!(
            rendered.contains("nothing placed"),
            "a refused identity pass is stated rather than rendered as a success\n{rendered}"
        );
        assert!(
            !rendered.contains(&rust_i18n::t!("diagnostic.no_pass_since_boot").to_string()),
            "and it is NOT the same sentence as *no pass at all* — the pass ran\n{rendered}"
        );
    }

    // ── AC5 / AC6: the security group ────────────────────────────────────────────────────

    /// 🔴 **AC6's carrier: the row is what `is_public` ADMITS, not what someone typed.**
    /// A hand-copied literal was measured drifting — widening `is_public` with `|| path == "/ipam"`
    /// reds exactly one test elsewhere and leaves `is_public_is_exactly_healthz_and_assets` green.
    ///
    /// ⚠️ **Its limit, stated**: the probe universe is finite, so this is a tripwire against a
    /// widening that touches a KNOWN address, never a barrier against one that invents a prefix.
    #[test]
    fn the_public_row_agrees_with_the_predicate_it_describes() {
        let posture = security_posture(false, false);
        for path in probe_universe() {
            assert_eq!(
                posture.public_paths.contains(&path),
                crate::auth::is_public(path),
                "{path} is classified differently by the screen and by the middleware — a screen \
                 that states the security perimeter from anything but the predicate itself is the \
                 false security claim AC2 exists to forbid"
            );
        }
        assert!(
            posture.public_paths.contains(&"/healthz"),
            "the probe universe must actually CONTAIN a public path, or the agreement above holds \
             vacuously"
        );
        assert!(
            posture.public_paths.len() < probe_universe().len(),
            "and a gated one"
        );
    }

    /// 🔴 **The second line of AC2, and it is stated as a tripwire rather than as a barrier.** The
    /// FIRST line is the shape: [`security_rows`] takes a [`SecurityPosture`] and nothing else, so
    /// a claim about security cannot be typed into this screen. This guard catches the residue —
    /// a claim smuggled through a translation VALUE — and it is an enumeration, in two languages,
    /// against paraphrase. *An enumeration cannot claim the completeness of a property.*
    ///
    /// 🔑 **Story 6b.10 gave it a stronger sibling and did NOT retire it, and the division of
    /// labour is the point.** `every_sentence_the_security_group_can_render_is_pinned_in_both_
    /// languages` pins each of the six sentences that group can render and proves the set closed
    /// — which is a real closure, not a longer list, and it caught the register row's own
    /// paraphrase on its first run. But it covers **only that group**. This test reads the WHOLE
    /// rendered page, so a security claim planted in any other row's copy is its business and not
    /// the pinning's. *Two carriers, different perimeters, neither subsuming the other* — the same
    /// shape as this story's two vocabulary carriers.
    ///
    /// ⚠️ **It cannot tell an assertion from its NEGATION, and it fired on this story's own copy
    /// to prove it**: the *stored credentials* row read *"…holds no encryption key"* and reddened
    /// here. The copy was rewritten rather than the guard weakened, and the reason is not the
    /// guard — **a screenshot cropped mid-sentence shows the noun phrase without the denial**. So
    /// the discipline this test enforces is wider than it looks: the screen does not reuse the
    /// reference mock's security phrases *even to deny them*.
    #[test]
    fn the_screen_states_no_security_property_the_product_does_not_hold() {
        // 🔴 **THE RENDERED PAGE, and the first version of this guard read the BUILDER.** Mutation
        // M7 planted *"Toutes les surfaces HTTP sont authentifiées."* as a literal in the template
        // and left the whole suite GREEN: `build_diagnostic`'s output cannot contain a sentence the
        // template adds. That is story 6b.4b's headline — *every guard read the source and every
        // defect lived in the render* — inside the guard this story wrote to close it.
        let rendered = rendered_body().to_lowercase();
        for claim in [
            "all http surfaces",
            "toutes authentifiées",
            "toutes les surfaces",
            "encrypted at rest",
            "chiffrés au repos",
            "encryption key",
            "clé de chiffrement",
            // Widened ONCE at story 6b.10, from the paraphrase `deferred-work.md` records as
            // having left 696/696 green and reached the served page. ⚠️ **Widened once and
            // deliberately NOT to exhaustion**: an enumeration cannot claim the completeness of a
            // property (story 5.12), and a longer list is a longer list, not a closure.
            "cannot be breached",
            "ne peut pas être compromis",
            "secure by design",
            "sécurisé par conception",
            "no known vulnerabilit",
            "aucune vulnérabilité connue",
        ] {
            assert!(
                !rendered.contains(claim),
                "the diagnostic states {claim:?}, which this product does not hold: a false claim \
                 about security, made by the product about itself\n{rendered}"
            );
        }
    }

    /// 🔴 **AC2's first line made STRONGER by story 6b.10: every sentence the security group can
    /// render is PINNED, in both languages, and the set is proven CLOSED.**
    ///
    /// # What this replaces, and why widening the word list was not enough
    ///
    /// `deferred-work.md` (story 6b.9's section) records the hole this closes: a translation value
    /// reading *"none stored — this deployment cannot be breached by a remote attacker"* left
    /// **696/696 green** and appeared on the served page. The shape stops a claim being TYPED into
    /// the row builder — [`security_rows`] takes a [`SecurityPosture`] and nothing else — and it
    /// does not stop one entering through the copy. The sibling word guard below is an
    /// ENUMERATION against paraphrase, and *an enumeration cannot claim the completeness of a
    /// property*: the paraphrase above contains none of its forbidden phrases.
    ///
    /// 🔑 **A closed set beats a longer list.** The group renders values from exactly six keys
    /// plus the public-path data, and this pins each of them. A new claim can then only arrive by
    /// EDITING a pinned sentence, which reds — and whoever reads that diff is looking at a
    /// security sentence changing, which is the review this row has never had.
    ///
    /// ⚠️ **Its limit, and it is real**: this pins the sentences, not their truth. *"none stored"*
    /// is true because no credential table exists; the day one does, this test passes and the row
    /// lies. The mechanism that would catch THAT is the derived-from-the-code shape story 6b.9
    /// built for the public-paths row, and extending it to the other three is not this story's.
    #[test]
    fn every_sentence_the_security_group_can_render_is_pinned_in_both_languages() {
        // Each key, and the sentence it must resolve to. A change here is a SECURITY CLAIM
        // changing: say why in the commit, and do not "update the expected value".
        const PINNED: [(&str, &str, &str); 6] = [
            ("diagnostic.public_paths_none", "nothing", "rien"),
            ("diagnostic.configured", "configured", "configuré"),
            (
                "diagnostic.basic_unset",
                "none configured — every other path is refused",
                "aucun configuré — toute autre adresse est refusée",
            ),
            (
                "diagnostic.metrics_token_set",
                "token required",
                "jeton requis",
            ),
            (
                "diagnostic.metrics_token_unset",
                "no token configured — the scrape is refused",
                "aucun jeton configuré — la collecte est refusée",
            ),
            (
                "diagnostic.secrets_value",
                "none stored — Epics 10 and 19 build credential storage",
                "aucun stocké — les épics 10 et 19 construisent le stockage d'identifiants",
            ),
        ];
        for (key, en, fr) in PINNED {
            assert_eq!(
                rust_i18n::t!(key, locale = "en"),
                en,
                "{key} is a SECURITY sentence and it changed — this is not a number to update, it \
                 is a claim the product makes about itself"
            );
            assert_eq!(rust_i18n::t!(key, locale = "fr"), fr, "{key}, in French");
        }

        // 🔴 **The set is CLOSED over the postures SAMPLED, and the difference matters.**
        // Pinning six sentences proves nothing if a seventh can appear beside them, so every
        // value must be a pinned sentence or the public-path DATA — which is `auth::is_public`'s
        // own list, not copy.
        //
        // ⚠️ *"Every posture the type admits"* stood here until story 6b.10's code review and was
        // FALSE: `public_paths` is a `Vec`, so the type admits unboundedly many and this samples
        // TWO — the empty one and the shipped one. The two booleans ARE exhaustive. What is
        // proven is therefore: over eight sampled postures, no unpinned sentence appears. That
        // is worth having and it is not what the sentence claimed.
        //
        // ⚠️ And the CLOSURE half is measured in English only: `security_rows` renders at the
        // ambient locale, which no test sets (`set_locale` is process-wide and would make the
        // suite order-dependent). The six sentences ARE pinned in both languages above; what is
        // English-only is the proof that no SEVENTH can appear. Closing that needs a
        // locale-parameterised renderer, registered rather than implied.
        let mut seen = 0_usize;
        for basic in [false, true] {
            for metrics in [false, true] {
                for paths in [Vec::new(), vec!["/healthz", "/assets/*"]] {
                    let posture = SecurityPosture {
                        public_paths: paths.clone(),
                        basic_configured: basic,
                        metrics_token_configured: metrics,
                    };
                    for row in security_rows(&posture) {
                        seen += 1;
                        // 🔴 The public-path disjunct is loop-invariant, and for the four
                        // postures where `paths` is empty it read `row.value == ""` — so a row
                        // rendering the EMPTY STRING was accepted as "pinned" in half the matrix.
                        // *A guard covered only by a disjunction is a guard nothing covers*, this
                        // story's own sentence, met inside the guard that quotes it. The data
                        // disjunct now only applies where there IS data.
                        let is_public_path_data =
                            !paths.is_empty() && row.value == paths.join(" · ");
                        let pinned =
                            is_public_path_data || PINNED.iter().any(|(_, en, _)| row.value == *en);
                        assert!(
                            pinned,
                            "the security group rendered {:?}, which is neither a pinned sentence \
                             nor the public-path data — a claim about security has entered the \
                             screen through a route this test cannot vouch for",
                            row.value
                        );
                    }
                }
            }
        }
        assert_eq!(
            seen, 32,
            "eight SAMPLED postures × four rows — both booleans exhaustively, `public_paths` in \
             its empty and its shipped shape"
        );
    }

    /// ⚠️ **The unconfigured pair is NOT reported as a weakness**, because it is not one: every
    /// non-public path then answers 401 without a challenge. The product is closed by default, and
    /// this is the one place an operator can learn it.
    #[test]
    fn an_unconfigured_pair_reads_as_closed_rather_than_as_open() {
        let facts = facts(security_posture(false, false));
        let view = build_diagnostic(&facts, Some(&store()), None, at(0));
        let rendered = values(&view);
        assert!(
            rendered.contains(&rust_i18n::t!("diagnostic.basic_unset").to_string()),
            "no pair configured is stated as a refusal, not as an opening\n{rendered}"
        );
        assert!(
            rendered.contains(&rust_i18n::t!("diagnostic.metrics_token_unset").to_string()),
            "and so is an unconfigured scrape token\n{rendered}"
        );
    }

    // ── The journal shows what was INSTALLED ─────────────────────────────────────────────

    /// 🔴 **M3's guard, and it is built so the environment cannot satisfy it.** The descriptor
    /// carries `opencmdb=trace,warn` and a directory no test process has; a handler that re-read
    /// `OPENCMDB_LOG` would render `info` and red here. ⚠️ It reds **without a database**, which the
    /// story's first mutation table did not: the screen's pool-free half is what makes that
    /// possible.
    #[test]
    fn the_journal_shows_the_descriptor_that_was_installed() {
        let facts = facts(security_posture(false, false));
        let view = build_diagnostic(&facts, Some(&store()), None, at(0));
        let rendered = values(&view);
        assert!(
            rendered.contains("opencmdb=trace,warn"),
            "the level in force is the one the subscriber installed\n{rendered}"
        );
        assert!(
            rendered.contains("/var/log/opencmdb-under-test") && rendered.contains("21"),
            "the directory and the retention are the appender's own\n{rendered}"
        );
    }

    /// 🔴 **The case the environment-reading design got wrong.** A directory was configured and the
    /// appender could not open it: the descriptor's `file` is `None`, and the screen must say
    /// *standard output only* rather than name a directory nothing writes to.
    #[test]
    fn a_file_sink_that_was_not_opened_is_never_named() {
        let log = LogDescriptor {
            directives: "info".to_string(),
            file: None,
        };
        let facts = DiagnosticFacts::new(
            log,
            security_posture(false, false),
            ScanReportSlot::default(),
        );
        let view = build_diagnostic(&facts, Some(&store()), None, at(0));
        let rendered = values(&view);
        assert!(
            rendered.contains(&rust_i18n::t!("diagnostic.log_stdout_only").to_string()),
            "with no file sink the screen says so\n{rendered}"
        );
        assert!(
            !rendered.contains("/var/log"),
            "and names no directory: the appender opened none\n{rendered}"
        );
    }

    // ── The group set ────────────────────────────────────────────────────────────────────

    /// M12's carrier: the group count is derived from the enum, never from a vector literal.
    #[test]
    fn every_group_of_the_enum_is_rendered_once() {
        let facts = facts(security_posture(false, false));
        let view = build_diagnostic(&facts, Some(&store()), None, at(0));
        assert_eq!(
            view.groups.len(),
            DiagGroup::ALL.len(),
            "one rendered group per declared group"
        );
        for group in DiagGroup::ALL {
            let title = rust_i18n::t!(group.heading_key()).to_string();
            assert_eq!(
                view.groups
                    .iter()
                    .filter(|rendered| rendered.title == title)
                    .count(),
                1,
                "{group:?} is rendered exactly once"
            );
        }
        assert!(
            view.groups.iter().all(|group| !group.rows.is_empty()),
            "a group with no rows is a heading over nothing"
        );
        // 🔴 **The total, which nothing counted until the code review.** Two layers arrived at
        // seventeen independently — one from the diff alone, one from the live render — against a
        // module doc that said sixteen. ⚠️ It is a bookkeeping assertion whose message reads *update
        // this number*, which a developer follows: it is kept because it is the only thing that
        // notices a row appearing or vanishing, and because the figure is quoted in prose.
        assert_eq!(
            row_count(&view),
            ROWS,
            "the diagnostic renders {ROWS} rows; if that changed on purpose, change the module \
             doc and the story's own figure with it"
        );
    }

    // ── AC9: the two controls, at the RENDER level ───────────────────────────────────────

    /// 🔴 **Story 6b.4b's four HIGH findings were one mistake made four times: every guard read the
    /// SOURCE and every defect lived in the RENDER.** So this asserts on the served HTML.
    ///
    /// ⚠️ **And 6b.4b's own guard cannot help here**: it reads `rendered_triage_body()` and carries
    /// a `roles >= 5` premise. Measured at this story's validation — two controls planted on this
    /// screen with no `tabindex` and a bare uppercase native `DISABLED` left the whole suite green.
    #[test]
    fn the_two_planned_controls_are_reachable_and_never_natively_disabled() {
        let html = rendered_body().to_lowercase();
        let controls = html.matches("role=\"button\"").count();
        assert_eq!(
            controls, 2,
            "the mock puts two controls under the diagnostic; the premise of everything below is \
             that they are BOTH here\n{html}"
        );
        assert_eq!(
            html.matches("tabindex=\"0\"").count(),
            controls,
            "every planned control is in the tab order — a `<span role=\"button\">` without \
             `tabindex` has `tabIndex -1`, refuses `.focus()`, and forty dispatched Tab presses \
             reach none of them\n{html}"
        );
        assert_eq!(
            html.matches("aria-disabled=\"true\"").count(),
            controls,
            "and every one is announced as unavailable rather than removed"
        );
        // 🔴 **A TOKEN scan, and the enumeration it replaces was defeated by a NEWLINE.** The
        // first version tested three literals — `" disabled>"`, `" disabled "`, `" disabled="` —
        // and mutation M8b planted a bare uppercase `DISABLED` followed by a line break, which none
        // of them matches. A boolean attribute has no `=`, `aria-disabled` must not be mistaken for
        // it, and the separator can be ANY whitespace: *an enumeration cannot claim the
        // completeness of a property* (story 5.12, and 6b.4b measured this same class on this same
        // attribute).
        let native_disabled = html
            .split(|c: char| c.is_whitespace() || c == '>')
            .any(|token| token == "disabled" || token.starts_with("disabled="));
        assert!(
            !native_disabled,
            "a NATIVE disabled attribute reached the page: it leaves the tab order, and a blind \
             operator is then not even told the gesture exists\n{html}"
        );
        // 🔑 And the ROWS total again, at the RENDER level: the builder count cannot see a template
        // that drops or duplicates a row, which is the half every guard in this story kept getting
        // wrong.
        assert_eq!(
            html.matches("class=\"diag-row\"").count(),
            ROWS,
            "{ROWS} rows reach the page\n{html}"
        );
        assert_eq!(
            html.matches("aria-describedby=\"diag-gesture-not-built\"")
                .count(),
            controls,
            "one visible sentence for the group, announced on each control"
        );
        assert_eq!(
            html.matches("id=\"diag-gesture-not-built\"").count(),
            1,
            "and it is rendered ONCE — rendered per control it became a stack saying the same \
             thing over and over"
        );
    }

    /// The rendered body, for the render-level assertions above.
    fn rendered_body() -> String {
        let facts = facts(security_posture(false, false));
        DiagnosticBody {
            view: build_diagnostic(&facts, Some(&store()), None, at(0)),
            gestures: diagnostic_gestures(),
            s: diagnostic_strings(),
        }
        .render()
        .expect("the diagnostic template and its struct are compiled together")
    }

    /// 🔑 **No secret reaches the page, and it is a COMPILE property rather than this test.**
    /// [`SecurityPosture`] carries `bool`s, so the credential is not in scope where the rows are
    /// built. This guard exists to pin that the shape has not been widened back.
    #[test]
    fn no_configured_secret_reaches_the_rendered_page() {
        let html = rendered_body();
        for secret in ["s3cret", "supersecrettoken"] {
            assert!(
                !html.contains(secret),
                "a credential reached the page: the security group must carry bools, never values"
            );
        }
    }

    // ── AC7, end to end: the route answers with the store down ───────────────────────────

    /// 🔴 **AC7 through the REAL route, with a pool that cannot connect.** `/sources`, `/triage`
    /// and `/dashboard` each answer a bare `500 internal error` in this state — no shell, no
    /// navigation — and the one moment an operator opens a self-diagnostic screen is exactly that
    /// one.
    #[tokio::test]
    async fn the_route_answers_with_the_store_unreachable() {
        // A pool that never connects, with an acquire timeout well under the handler's budget —
        // so this test measures the HANDLER's refusal to wait, not sqlx's.
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_millis(150))
            .connect_lazy("mysql://root:x@127.0.0.1:1/none")
            .expect("lazy pool");
        let started = std::time::Instant::now();
        let router = crate::page::triage_router(
            pool,
            Some("192.0.2.0/24".to_string()),
            facts(security_posture(true, false)),
        );
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/diagnostic")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("the route answers");
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "a store that cannot be read is a rendered state, never a 500"
        );
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        let html = String::from_utf8_lossy(&body).to_string();
        assert!(
            html.contains(&rust_i18n::t!("diagnostic.store_unreachable").to_string()),
            "and it says so"
        );
        assert!(
            html.contains("nav-entry"),
            "inside the shell, with the navigation — the operator must be able to leave the page"
        );
        assert!(
            html.contains(&format!("v{}", crate::VERSION)),
            "and the rows that need no pool are all there"
        );
        // 🔴 **AC7 is about being USABLE with the store down, and a page that answers eventually is
        // not.** The first version of this test took THIRTY SECONDS — sqlx's default acquire
        // timeout — and passed, which is how the missing budget was found: the assertion said
        // *renders without the store* and the clock said *after half a minute*.
        // 🔴 **A LITERAL bound, and the first version compared against `STORE_READ_BUDGET`
        // itself.** Mutation M13 raised the budget to sixty seconds and the assertion moved with
        // it — GREEN. *An oracle that restates the expectation instead of measuring the code cannot
        // fail*, which is story 5.8's M5 family and one of this project's oldest recurring defects.
        let waited = started.elapsed();
        assert!(
            waited < std::time::Duration::from_secs(3),
            "the diagnostic waited {waited:?} for a store that is down — AC7 is about being USABLE \
             with the store down, and a page that answers eventually is not"
        );
        assert!(
            STORE_READ_BUDGET <= std::time::Duration::from_secs(3),
            "and the budget itself stays within what an operator will wait: {STORE_READ_BUDGET:?}"
        );
    }

    /// 🔴 **The BUDGET branch, and nothing reached it until the blind review layer said so.**
    /// `the_route_answers_with_the_store_unreachable` drives a pool that refuses in 150 ms, so
    /// `read_store_from` always resolved with an error and `tokio::time::timeout` never elapsed —
    /// *the arm that enforces the budget was dead code under test*, while the test's own comment
    /// explained exactly why, in the language of a justification.
    ///
    /// 🔑 A pool pointed at a NON-ROUTABLE address hangs instead of refusing, which is what a slow
    /// store does; the budget is a parameter so this costs 60 ms rather than two seconds.
    /// ⚠️ Proved to red: replacing `budget` with an hour makes it wait **5.002 s** — the pool's own
    /// acquire timeout — and fail on the bound below.
    #[tokio::test]
    async fn a_store_that_is_slow_rather_than_down_is_bounded_by_the_budget() {
        // 10.255.255.1 is RFC 1918 space with no host: the TCP handshake hangs rather than being
        // refused. The pool's own acquire timeout is set FAR above the budget on purpose — if the
        // budget stopped working, this test would wait five seconds and fail on the bound below.
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect_lazy("mysql://root:x@10.255.255.1:3306/none")
            .expect("lazy pool");
        let budget = std::time::Duration::from_millis(60);
        let started = std::time::Instant::now();
        let store = store_within(&pool, budget).await;
        let waited = started.elapsed();
        assert!(
            store.is_none(),
            "a store that does not answer inside its budget contributes nothing"
        );
        assert!(
            waited < std::time::Duration::from_millis(600),
            "the BUDGET bounded the wait, not the pool: {waited:?} against a 60 ms budget and a \
             5 s acquire timeout"
        );
    }

    // ── The applied-migration count, driven away from the embedded one ───────────────────

    /// 🔴 **The ONE place the applied count can differ from the embedded one, and it is a test.**
    /// Measured on a live instance: `run()` migrates before binding the listener and both divergent
    /// states refuse to boot (*"is partially applied"*, *"was previously applied but is missing"*),
    /// so on any instance that answers HTTP the two are equal. A guard that only ever sees them
    /// equal cannot tell a measurement from a literal — which is why the story's M1 and M2 are the
    /// same mutation up to a constant, and why this test deletes a row inside a transaction it
    /// rolls back.
    #[tokio::test]
    async fn the_applied_count_is_read_and_not_assumed() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping the applied-migration test: DATABASE_URL unset");
            return;
        };
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");

        let healthy = read_store_from(&pool).await.expect("read the store");
        assert_eq!(
            healthy.migrations_applied,
            embedded_migrations(),
            "a healthy instance has applied every migration it embeds — and this equality is why \
             the divergence below has to be manufactured"
        );

        // Drive the applied count AWAY from the embedded one, inside a transaction that is rolled
        // back, so the fixture is never left short a migration.
        let mut tx = pool.begin().await.expect("begin");
        let highest: (i64,) =
            sqlx::query_as("SELECT MAX(version) FROM _sqlx_migrations WHERE success = TRUE")
                .fetch_one(&mut *tx)
                .await
                .expect("read the highest version");
        sqlx::query("DELETE FROM _sqlx_migrations WHERE version = ?")
            .bind(highest.0)
            .execute(&mut *tx)
            .await
            .expect("delete one migration row");
        // 🔴 **Read it back THROUGH `read_store`**, not through raw SQL: the first version of this
        // test asserted over its own `SELECT COUNT(*)`, which measured the DATABASE rather than the
        // code — hardcoding the count in `read_store` left it green, and that is the very defect
        // AC1 exists to forbid.
        let short = read_store(&mut tx)
            .await
            .expect("read inside the transaction");
        assert_eq!(
            short.migrations_applied,
            embedded_migrations() - 1,
            "`read_store` reports what the store has APPLIED — a hardcoded count, or the embedded \
             count returned in its place, cannot see this"
        );
        assert!(
            short.schema.contains(&format!("{:04}", highest.0 - 1)),
            "and the schema row follows it down to {}, rather than naming a migration this store \
             no longer records: {}",
            highest.0 - 1,
            short.schema
        );
        tx.rollback().await.expect("rollback");

        // And the fixture is intact: the next reader sees every migration again.
        assert_eq!(
            read_store_from(&pool)
                .await
                .expect("read the store again")
                .migrations_applied,
            embedded_migrations(),
            "the manufactured divergence was rolled back"
        );
    }

    /// 🔴 **`read_store` puts the placed count on the PLACED row, and this is the only guard that
    /// can see it.** Mutation M6 swaps the two outcome filters; the pure-builder assertions cannot
    /// notice, because they are handed a `StoreFacts` built by hand — *a guard on the builder
    /// cannot see a defect in the reader*, which is Epic 5's dominant class and the THIRD time this
    /// story met it. The rows are seeded so the two counts are DIFFERENT and neither is zero: equal
    /// counts make a swap invisible, and a zero makes half the assertion vacuous.
    #[tokio::test]
    async fn the_reader_puts_each_reach_count_on_its_own_row() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping the reach-row test: DATABASE_URL unset");
            return;
        };
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");

        // Everything below is rolled back, so the fixture is untouched by this test.
        let mut tx = pool.begin().await.expect("begin");
        for statement in [
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM interface",
            "DELETE FROM observation_record",
        ] {
            sqlx::query(statement)
                .execute(&mut *tx)
                .await
                .expect("clear");
        }
        let nil = crate::repo::ABSTAINED_SUBJECT;
        let interface = "11111111-1111-4111-8111-111111111111";
        sqlx::query(
            "INSERT INTO interface (id, l2_domain, mac_canon, first_seen_at, last_seen_at) \
             VALUES (?, ?, '02:00:00:00:00:01', '2026-01-01 00:00:00', '2026-01-01 00:00:00')",
        )
        .bind(interface)
        .bind(nil)
        .execute(&mut *tx)
        .await
        .expect("mint an interface");

        // ONE placed sighting and TWO unplaced ones: different, and neither zero.
        for (index, placed) in [(1, true), (2, false), (3, false)] {
            let observation = format!("22222222-2222-4222-8222-00000000000{index}");
            sqlx::query(
                "INSERT INTO observation_record (id, connector_id, observed_at, l2_domain, \
                 vantage, facts) VALUES (?, ?, '2026-01-01 00:00:00', ?, ?, '[]')",
            )
            .bind(&observation)
            .bind(nil)
            .bind(nil)
            .bind(nil)
            .execute(&mut *tx)
            .await
            .expect("record an observation");
            let (outcome, rule, cause, interface_id) = if placed {
                ("match", Some("l1-exact-mac"), None, Some(interface))
            } else {
                ("abstained", None, Some("absence_of_proof"), None)
            };
            sqlx::query(
                "INSERT INTO identity_link (id, observation_id, interface_id, current_subject, \
                 outcome, rule_id, abstention_cause, evidence, ruleset_version, decided_by, \
                 valid_from, valid_to) VALUES (?, ?, ?, ?, ?, ?, ?, '[]', 1, 'ENGINE', \
                 '2026-01-01 00:00:00', ?)",
            )
            .bind(format!("33333333-3333-4333-8333-00000000000{index}"))
            .bind(&observation)
            .bind(interface_id)
            .bind(interface_id.unwrap_or(nil))
            .bind(outcome)
            .bind(rule)
            .bind(cause)
            .bind(crate::repo::OPEN_END)
            .execute(&mut *tx)
            .await
            .expect("write a link");
        }

        let facts = read_store(&mut tx).await.expect("read the store");
        assert_eq!(facts.placed, 1, "one sighting was placed on an interface");
        assert_eq!(
            facts.not_placed, 2,
            "and two were not — a swap of the two outcome filters must not be able to satisfy \
             this, which is why the counts differ and neither is zero"
        );
        tx.rollback().await.expect("rollback");
    }

    /// 🔴 **The measurement the whole descriptor design rests on, pinned rather than asserted in
    /// prose.** Story 6b.9's validation reported that `EnvFilter`'s lossy parse *"DISCARDS"* an
    /// invalid directive, and the browser look refuted it for the input that matters most: a bare
    /// word is read as a TARGET at `trace`, not thrown away.
    ///
    /// 🔑 **And the conclusion survives its wrong reason, in a stronger form.** For `!!!`,
    /// `opencmdb=nope` and the empty string the filter really does collapse — to **`error`**, so a
    /// screen reading the ENVIRONMENT would show `opencmdb=nope` while the process logs almost
    /// nothing. *Whichever way the parse goes, what the variable says and what is in force differ*,
    /// which is exactly why this screen renders the descriptor.
    #[test]
    fn the_filter_in_force_is_not_the_variable_that_was_typed() {
        let installed = |raw: &str| tracing_subscriber::EnvFilter::new(raw).to_string();
        // Unchanged when the directive is valid.
        assert_eq!(installed("info"), "info");
        assert_eq!(installed("opencmdb=debug,warn"), "opencmdb=debug,warn");
        // A bare word is a TARGET, not a level — measured on a real boot with
        // `OPENCMDB_LOG=notalevel`, where the screen showed `notalevel=trace` and the product then
        // logged nothing of its own.
        assert_eq!(installed("notalevel"), "notalevel=trace");
        // And these really do collapse — to `error`, not to `info`.
        for collapsing in ["!!!", "opencmdb=nope", ""] {
            assert_eq!(
                installed(collapsing),
                "error",
                "{collapsing:?} leaves a filter that shows almost nothing, while the variable \
                 still reads as though a level had been chosen"
            );
        }
    }
}
