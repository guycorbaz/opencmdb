//! The single page that shows the gap (Story 3.7).
//!
//! It reconciles the persisted `declared_attribute` rows against the persisted `observation_record`
//! facts through the SAME pure `reconcile` engine (Story 3.6) and renders the result with Askama.
//! The view-building is a PURE function (`build_view`) so it is unit-tested without a database; the
//! DB read and the HTTP wrapping are the only impure edges.

use askama::Template;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use opencmdb_core::observation::{
    ConnectorId, Fact, L2DomainId, ObsId, Observation, Scope, VantageId,
};
use opencmdb_core::{AbstentionCause, reconcile};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::repo::{
    EngineReachRow, classify, count_engine_reach, load_declared_attributes, load_observation_facts,
};

/// Committed front-end assets, embedded into the binary (no CDN, self-hosted single binary).
#[derive(rust_embed::Embed)]
#[folder = "assets/"]
struct Assets;

// ── The shell (story 6b.2) ───────────────────────────────────────────

/// One navigation entry, shaped for the template.
struct NavEntry {
    /// The entry's own address — never `#`, which is what the mock writes.
    href: &'static str,
    /// The translated label.
    label: String,
    /// Whether this is the screen being rendered; exactly one entry per render is `true`.
    current: bool,
}

/// One of the mock's three navigation groups, with its entries.
struct NavGroupView {
    /// The translated group heading.
    heading: String,
    /// The entries, in the mock's order.
    entries: Vec<NavEntry>,
}

/// Everything the frame needs, and deliberately nothing else.
///
/// 🔑 There is no database handle here and no field that could hold one: the shell renders on
/// ten screens, nine of which are demonstrations, and epic constraint 1 forbids those to open a
/// connection. The perimeter arrives from `AppConfig`, the version from the crate.
pub(crate) struct Shell {
    screen: crate::screens::Screen,
    perimeter: Option<String>,
}

impl Shell {
    /// The frame for one screen.
    pub(crate) fn new(screen: crate::screens::Screen, perimeter: Option<String>) -> Self {
        Self { screen, perimeter }
    }
}

/// The frame, rendered around a screen's body.
///
/// `body` is inserted unescaped: it is template output, not user input. The only caller that
/// passes a non-empty body is `/triage`, which passes the reconciliation card.
pub(crate) fn render_shell(shell: Shell, body: String) -> String {
    let groups: Vec<NavGroupView> = crate::screens::NavGroup::ALL
        .iter()
        .map(|group| NavGroupView {
            heading: rust_i18n::t!(group.heading_key()).to_string(),
            entries: crate::screens::Screen::ALL
                .iter()
                .filter(|screen| screen.group() == *group)
                .map(|screen| NavEntry {
                    href: screen.href(),
                    label: rust_i18n::t!(screen.label_key()).to_string(),
                    current: *screen == shell.screen,
                })
                .collect(),
        })
        .collect();

    ShellPage {
        lang: rust_i18n::locale().to_string(),
        title: rust_i18n::t!(shell.screen.title_key()).to_string(),
        title_separator: "—",
        version: env!("CARGO_PKG_VERSION"),
        perimeter: shell
            .perimeter
            .unwrap_or_else(|| rust_i18n::t!("nav.perimeter_unset").to_string()),
        groups,
        s: strings(),
        body,
    }
    .render()
    .unwrap_or_else(|error| {
        tracing::error!(%error, "the shell failed to render");
        String::from("<!doctype html><title>opencmdb</title><p>render error")
    })
}

#[derive(Template)]
#[template(path = "_shell.html")]
struct ShellPage {
    lang: String,
    title: String,
    title_separator: &'static str,
    version: &'static str,
    perimeter: String,
    groups: Vec<NavGroupView>,
    s: Strings,
    body: String,
}

// ── View models (what the templates render) ──────────────────────────

struct KeyValue {
    key: String,
    value: String,
}

struct GapRow {
    field: String,
    declared: String,
    observed: String,
}

struct AbstentionRow {
    cause: String,
    count: usize,
}

/// One cause line of the identity engine's reach — *"N sightings, because …"*.
struct IdentityCauseRow {
    cause: String,
    count: i64,
}

/// The identity engine's reach, shaped for rendering.
///
/// # The unit is SIGHTINGS, and that is a decision rather than a caption
///
/// Every scan mints fresh `obs_id`s and the identity pass supersedes no engine link across passes,
/// so one machine seen ten times is ten rows. The number therefore counts SIGHTINGS, not devices,
/// and the surface says so on both sides of the pair.
///
/// 🔑 Naming the unit truthfully is what keeps the number from reading as a backlog: a figure that
/// rises because the product looked many times is the radar's range, not the operator's debt. ⚠️ It
/// does not make the UX bans MET — *"no growing counter"* and *"after six months of inaction it
/// reads the same number"* are still open, owned by Epic 6, and registered. A true unit does not
/// stop a number growing.
///
/// ⚠️ **The unit is TEMPORARY.** Epic 6 gives the population an identity, at which point *sighting*
/// stops being the honest word and the locale keys change with it. That rename is a scheduled
/// consequence, not a correction of a mistake.
///
/// # 🔴 Three cases, and they are the OPERATOR's three, not the engine's
///
/// Guy's taxonomy (2026-08-12), which is what decides where each outcome goes:
///
/// | case | what the engine wrote | who acts | the gesture |
/// |---|---|---|---|
/// | **no ambiguity** | `Match`, and also `NoMatch` | the software | none — it decided |
/// | **ambiguity** | `Abstained { Ambiguous }` | the operator lifts the doubt | choose among the candidates and their evidence (FR16) |
/// | **unknown** | `Abstained { AbsenceOfProof }` | the operator creates the entity | **declare** — the documenting gesture |
///
/// 🔑 **`NoMatch` is case ONE**, which is why it is neither placed nor listed among what awaits the
/// operator: *a rule FORBADE the pair* is a decision, not an absence. An earlier draft folded it
/// into `placed` through a bare `else`, so a refused placement was reported as a placement and the
/// page rendered *"every sighting was placed"* over it — found independently by all three review
/// layers.
///
/// ⚠️ **Neither gesture EXISTS in the product yet**, and this view deliberately announces neither:
/// the ambiguity gesture needs candidates nothing produces (Epic 6), and the documenting gesture
/// needs a write surface the product does not have. **Announcing an absent gesture is a promise;
/// this section stays descriptive until the gesture is there** (Guy, 2026-08-12). The taxonomy is
/// registered as the criterion for both.
struct IdentityView {
    /// Sightings the engine placed on an interface — case one, `Match`.
    placed: i64,
    /// Sightings it could not place — case two and case three together.
    not_placed: i64,
    /// Why, one line per cause — never one line per failure (FR16b).
    ///
    /// ⚠️ **The one-line-per-cause property belongs to the CALLER**, not to this type: it holds
    /// because `count_engine_reach` groups by cause in SQL. Feed this view two rows carrying the
    /// same cause and it renders two identical lines. Stated rather than enforced, because the only
    /// producer is the grouped read.
    causes: Vec<IdentityCauseRow>,
    /// Outcomes the engine SETTLED without placing — `NoMatch`, and any token no variant names.
    ///
    /// Rendered only when non-empty, and today it always is empty: `resolve` cannot produce a
    /// `NoMatch` (`placement_decision` only judges pairs inside one `join` group, which share their
    /// key by construction), and `repo::cause_token`'s exhaustive `match` is what writes the rest.
    /// It is counted and labelled rather than folded anywhere, on `identity_cause_label`'s
    /// precedent: the tolerant reader for the CAUSE token had a silent twin on the OUTCOME token,
    /// and this is that twin, made explicit.
    settled: Vec<IdentityCauseRow>,
    /// Has the engine seen anything at all? Distinguishes *"nothing yet"* from *"nothing unplaced"*.
    has_any: bool,
}

/// Everything the card template needs — shaped for rendering, honest about the empty state.
struct ReconciledView {
    has_entity: bool,
    entity_ipv4: String,
    declared: Vec<KeyValue>,
    observed: Vec<KeyValue>,
    gaps: Vec<GapRow>,
    abstentions: Vec<AbstentionRow>,
    abstention_count: usize,
}

/// The user-facing strings, resolved through the i18n `t!()` seam (Story 3.8). The templates read
/// these instead of literals, so every rendered string flows through `rust-i18n`.
struct Strings {
    tagline: String,
    /// The navigation's accessible name (story 6b.2).
    nav_label: String,
    /// The example-data marker's badge (story 6b.3).
    example_badge: String,
    /// The example-data marker's sentence (story 6b.3).
    example_sentence: String,
    /// The witness screen's inventory heading (story 6b.3).
    devices_title: String,
    /// Column: the device's name (story 6b.3).
    devices_name: String,
    /// Column: the address (story 6b.3).
    devices_ipv4: String,
    /// Column: the hardware address (story 6b.3).
    devices_mac: String,
    /// Column: what the device is for (story 6b.3).
    devices_role: String,
    /// The witness screen's second heading — sightings the engine did not place (story 6b.3).
    unplaced_title: String,
    /// Column: why a sighting was not placed (story 6b.3).
    unplaced_reason: String,
    /// The perimeter label in the navigation footer, as the mock shows it (story 6b.2).
    nav_perimeter: String,
    entity: String,
    refresh: String,
    declared: String,
    observed: String,
    no_observation: String,
    the_gap: String,
    no_drift: String,
    arrow_observed: String,
    reach: String,
    reach_hint: String,
    nothing_unplaced: String,
    no_declared_title: String,
    no_declared_hint: String,
    identity_title: String,
    identity_placed: String,
    identity_not_placed: String,
    identity_because: String,
    identity_floor: String,
    identity_unit: String,
    identity_nothing_seen: String,
    identity_all_placed: String,
    identity_settled: String,
}

fn strings() -> Strings {
    use rust_i18n::t;
    Strings {
        tagline: t!("page.tagline").to_string(),
        nav_label: t!("nav.label").to_string(),
        example_badge: t!("example.badge").to_string(),
        example_sentence: t!("example.sentence").to_string(),
        devices_title: t!("devices.title").to_string(),
        devices_name: t!("devices.name").to_string(),
        devices_ipv4: t!("devices.ipv4").to_string(),
        devices_mac: t!("devices.mac").to_string(),
        devices_role: t!("devices.role").to_string(),
        unplaced_title: t!("unplaced.title").to_string(),
        unplaced_reason: t!("unplaced.reason").to_string(),
        nav_perimeter: t!("nav.perimeter").to_string(),
        entity: t!("page.entity").to_string(),
        refresh: t!("page.refresh").to_string(),
        declared: t!("page.declared").to_string(),
        observed: t!("page.observed").to_string(),
        no_observation: t!("page.no_observation").to_string(),
        the_gap: t!("page.the_gap").to_string(),
        no_drift: t!("page.no_drift").to_string(),
        arrow_observed: t!("page.arrow_observed").to_string(),
        reach: t!("page.reach").to_string(),
        reach_hint: t!("page.reach_hint").to_string(),
        nothing_unplaced: t!("page.nothing_unplaced").to_string(),
        no_declared_title: t!("page.no_declared_title").to_string(),
        no_declared_hint: t!("page.no_declared_hint").to_string(),
        identity_title: t!("identity.title").to_string(),
        identity_placed: t!("identity.placed").to_string(),
        identity_not_placed: t!("identity.not_placed").to_string(),
        identity_because: t!("identity.because").to_string(),
        identity_floor: t!("identity.floor").to_string(),
        identity_unit: t!("identity.unit").to_string(),
        identity_nothing_seen: t!("identity.nothing_seen").to_string(),
        identity_all_placed: t!("identity.all_placed").to_string(),
        identity_settled: t!("identity.settled").to_string(),
    }
}

// ⚠️ `identity` is a SIBLING of `view`, not a field of it, and that is load-bearing: `build_view`
// returns EARLY with a fully-zeroed `ReconciledView` when no declared entity exists, so an identity
// count carried inside it would be silently zeroed in exactly the deployment the section exists for
// — a fresh install that has scanned and declared nothing.
#[derive(Template)]
#[template(path = "_gap_card.html")]
struct GapFragment {
    view: ReconciledView,
    identity: IdentityView,
    s: Strings,
}

// ── The pure view builder (unit-tested without a DB) ─────────────────

/// A human label for an abstention cause — reach, never a reproach (FR39). Routed through the
/// i18n `t!()` seam (Story 3.8).
fn cause_label(cause: AbstentionCause) -> String {
    use rust_i18n::t;
    match cause {
        AbstentionCause::OutOfPerimeter => t!("cause.out_of_perimeter"),
        AbstentionCause::NoObservedValue => t!("cause.no_observed_value"),
        AbstentionCause::ConflictingObservations => t!("cause.conflicting_observations"),
    }
    .to_string()
}

/// A human label for an IDENTITY abstention cause, from the token as it is persisted.
///
/// # 🔴 This function is TOTAL, and refusing to fail is the whole point
///
/// `identity_link.abstention_cause` is a plain `VARCHAR(32)` with no `CHECK`, so the database can
/// hold a token no variant of [`opencmdb_core::identity::cascade::IdentityAbstentionCause`] names —
/// measured, an invented token inserts cleanly. And `page.rs`'s handlers turn any error into a `500`
/// for the WHOLE page, so a reader that failed here would take the gap display down with it, for one
/// unfamiliar row.
///
/// So an unrecognised token is **labelled and carried**, never dropped and never fatal. It is still
/// COUNTED by [`build_identity_view`]: a total that silently shrank would be the counter lying by
/// omission, which is worse than an unfamiliar word on the page.
///
/// ⚠️ **A `match` on tokens cannot be exhaustive over the enum**, and that is exactly why this is a
/// tripwire rather than a barrier: adding a variant breaks the WRITER ([`crate::repo::cause_token`],
/// an exhaustive `match` with no `_` arm) and breaks nothing here. A variant added with the minimal
/// repair therefore persists a token this function does not know — and the page renders it as
/// unrecognised instead of dying. That is the designed behaviour, not a gap in it.
///
/// The stronger closure — a DDL `CHECK` on the token domain — was weighed and refused for story
/// 5.14b: it moves the failure from the display to the WRITE, so a future variant would break the
/// identity pass rather than show an unfamiliar label. It is registered as the real closure.
fn identity_cause_label(token: &str) -> String {
    use rust_i18n::t;
    match token {
        "absence_of_proof" => t!("identity.cause.absence_of_proof").to_string(),
        "ambiguous" => t!("identity.cause.ambiguous").to_string(),
        other => t!("identity.cause.unrecognised", token = other).to_string(),
    }
}

/// PURE: shape the database's grouped reach rows into a renderable view.
///
/// Abstained rows are the not-placed population and each contributes one cause line; everything else
/// is placed. **One line per cause, never one line per failure** — FR16b's *"96 multi-interface
/// devices is not 96 failures, it is ONE question"*.
///
/// ⚠️ **An abstained row whose cause is NULL cannot exist**: `identity_link_rule_xor_cause` makes the
/// cause non-NULL exactly when `outcome = 'abstained'`. This function is nonetheless total over the
/// type, and the empty token then falls to the unrecognised label. That is totality, **not a guard** —
/// no test can red it, and it is not claimed as covering anything.
fn build_identity_view(rows: Vec<EngineReachRow>) -> IdentityView {
    use rust_i18n::t;

    let mut placed = 0i64;
    let mut not_placed = 0i64;
    let mut causes: Vec<IdentityCauseRow> = Vec::new();
    let mut settled: Vec<IdentityCauseRow> = Vec::new();
    let mut settled_count = 0i64;
    for row in rows {
        // 🔴 An explicit arm per outcome, and NO bare `else`. The `else` an earlier draft used sent
        // `no_match` — *a rule FORBADE this pair* — into `placed`, i.e. reported a refusal as a
        // success. `identity_link_outcome` admits exactly these three tokens; anything else can only
        // arrive from a store written by something other than `repo::outcome_token`, and it is
        // carried rather than folded, exactly as an unknown CAUSE token is.
        match row.outcome.as_str() {
            "match" => placed += row.count,
            "abstained" => {
                not_placed += row.count;
                causes.push(IdentityCauseRow {
                    cause: identity_cause_label(row.cause.as_deref().unwrap_or_default()),
                    count: row.count,
                });
            }
            "no_match" => {
                settled_count += row.count;
                settled.push(IdentityCauseRow {
                    cause: t!("identity.outcome.no_match").to_string(),
                    count: row.count,
                });
            }
            other => {
                settled_count += row.count;
                settled.push(IdentityCauseRow {
                    cause: t!("identity.outcome.unrecognised", token = other).to_string(),
                    count: row.count,
                });
            }
        }
    }
    IdentityView {
        placed,
        not_placed,
        causes,
        settled,
        has_any: placed + not_placed + settled_count > 0,
    }
}

/// Project a fact into a displayable `(label, value)` pair (a superset of the engine's projection —
/// the page also shows `rtt`, which the engine does not reconcile as a declared field).
fn display_fact(fact: &Fact) -> Option<(String, String)> {
    match fact {
        Fact::IpV4 { addr } => Some(("ipv4".into(), addr.to_string())),
        Fact::Hostname { name, .. } => Some(("hostname".into(), name.clone())),
        Fact::Mac { addr, .. } => Some(("mac".into(), addr.to_string())),
        Fact::Rtt { millis } => Some(("rtt".into(), format!("{millis} ms"))),
        _ => None,
    }
}

/// Does an observation carry the perimeter identity `("ipv4", ipv4)`?
fn in_perimeter(facts: &[Fact], ipv4: &str) -> bool {
    facts
        .iter()
        .any(|f| matches!(f, Fact::IpV4 { addr } if addr.to_string() == ipv4))
}

/// Build the [`Observation`] the engine reconciles from a bag of facts. The engine reads only the
/// facts, so the ids/scope/time are placeholders — this keeps the page independent of them.
fn observation_from_facts(facts: Vec<Fact>) -> Observation {
    Observation {
        obs_id: ObsId::from_uuid(Uuid::nil()),
        connector_id: ConnectorId::from_uuid(Uuid::nil()),
        observed_at: chrono::DateTime::from_timestamp(0, 0).expect("epoch is representable"),
        scope: Scope {
            l2_domain: L2DomainId::from_uuid(Uuid::nil()),
            vantage: VantageId::from_uuid(Uuid::nil()),
        },
        facts,
        raw: None,
    }
}

/// PURE: shape the declared rows + observation facts into a renderable view. Picks the perimeter
/// entity (the caller's preferred `ipv4`, else the first declared entity carrying an `ipv4`),
/// reconciles it, and returns an honest empty view when there is no such entity.
fn build_view(
    declared: Vec<(String, String, String)>,
    observations: Vec<Vec<Fact>>,
    preferred_ipv4: Option<String>,
) -> ReconciledView {
    // Group declared attributes by entity, preserving first-seen order.
    let mut entities: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for (entity_id, key, value) in declared {
        match entities.iter_mut().find(|(id, _)| *id == entity_id) {
            Some((_, attrs)) => attrs.push((key, value)),
            None => entities.push((entity_id, vec![(key, value)])),
        }
    }

    // The perimeter entity's `ipv4` value, and its declared pairs.
    let ipv4_of = |attrs: &[(String, String)]| -> Option<String> {
        attrs
            .iter()
            .find(|(k, _)| k == "ipv4")
            .map(|(_, v)| v.clone())
    };
    let chosen = match &preferred_ipv4 {
        Some(pref) => entities
            .iter()
            .find(|(_, attrs)| ipv4_of(attrs).as_deref() == Some(pref.as_str())),
        None => entities.iter().find(|(_, attrs)| ipv4_of(attrs).is_some()),
    };

    let Some((_, attrs)) = chosen else {
        return ReconciledView {
            has_entity: false,
            entity_ipv4: String::new(),
            declared: Vec::new(),
            observed: Vec::new(),
            gaps: Vec::new(),
            abstentions: Vec::new(),
            abstention_count: 0,
        };
    };
    let ipv4 = ipv4_of(attrs).expect("chosen entity carries an ipv4");
    let declared_pairs: Vec<(String, String)> = attrs.clone();

    // Observed rows: the projected facts of in-perimeter observations, de-duplicated in order.
    let mut observed: Vec<KeyValue> = Vec::new();
    for facts in &observations {
        if !in_perimeter(facts, &ipv4) {
            continue;
        }
        for (key, value) in facts.iter().filter_map(display_fact) {
            if !observed.iter().any(|r| r.key == key && r.value == value) {
                observed.push(KeyValue { key, value });
            }
        }
    }

    // Reconcile through the pure engine.
    let obs: Vec<Observation> = observations
        .into_iter()
        .map(observation_from_facts)
        .collect();
    let result = reconcile(("ipv4", &ipv4), &declared_pairs, &obs);

    let gaps = result
        .gaps
        .iter()
        .map(|g| GapRow {
            field: g.field.clone(),
            declared: g.declared.clone(),
            observed: g.observed.clone(),
        })
        .collect();
    let abstentions = result
        .abstentions
        .iter()
        .map(|(cause, count)| AbstentionRow {
            cause: cause_label(*cause),
            count: *count,
        })
        .collect();

    ReconciledView {
        has_entity: true,
        entity_ipv4: ipv4,
        declared: declared_pairs
            .into_iter()
            .map(|(key, value)| KeyValue { key, value })
            .collect(),
        observed,
        gaps,
        abstentions,
        abstention_count: result.abstention_count(),
    }
}

// ── The impure edges: DB read + HTTP handlers ────────────────────────

/// Load the declared + observed state and build the view. `OPENCMDB_ENTITY_IPV4` selects the
/// perimeter entity when set.
async fn reconcile_view(pool: &MySqlPool) -> Result<(ReconciledView, IdentityView), Response> {
    let declared = load_declared_attributes(pool).await.map_err(server_error)?;
    let observations = load_observation_facts(pool).await.map_err(server_error)?;
    let reach = count_engine_reach(pool).await.map_err(server_error)?;
    let preferred = std::env::var("OPENCMDB_ENTITY_IPV4").ok();
    Ok((
        build_view(declared, observations, preferred),
        build_identity_view(reach),
    ))
}

fn server_error(error: sqlx::Error) -> Response {
    let repo_error = classify(error);
    tracing::error!(?repo_error, "loading the page's state failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
}

/// One example device, with its copy already resolved into the operator's language.
///
/// 🔴 The resolution happens HERE and not in the template, because `example_data` holds i18n KEYS
/// rather than sentences — see [`crate::example_data::ExampleDevice::role_key`] for the defect
/// that taught us the difference, which was found by looking at the screen and by nothing else.
struct ExampleDeviceView {
    id: &'static str,
    name: &'static str,
    ipv4: &'static str,
    mac: &'static str,
    role: String,
}

/// One example sighting the engine did not place, with its reason resolved.
struct ExampleSightingView {
    ipv4: &'static str,
    mac: &'static str,
    reason: String,
}

/// The witness screen's body — the example inventory, in two sections (story 6b.3).
#[derive(Template)]
#[template(path = "_devices_example.html")]
struct DevicesExample {
    devices: Vec<ExampleDeviceView>,
    sightings: Vec<ExampleSightingView>,
    s: Strings,
}

/// Render the example inventory that the witness screen shows.
///
/// # Panics
///
/// Never in practice: the template is compiled into the binary by askama and its inputs are
/// constants, so a failure here would mean the template no longer matches its struct — which is a
/// compile error, not a run-time one. The `expect` states that rather than hiding it behind a
/// fallback string nobody would ever see.
pub(crate) fn devices_example_body() -> String {
    DevicesExample {
        devices: crate::example_data::devices()
            .into_iter()
            .map(|device| ExampleDeviceView {
                id: device.id,
                name: device.name,
                ipv4: device.ipv4,
                mac: device.mac,
                role: rust_i18n::t!(device.role_key).to_string(),
            })
            .collect(),
        sightings: crate::example_data::unplaced_sightings()
            .into_iter()
            .map(|sighting| ExampleSightingView {
                ipv4: sighting.ipv4,
                mac: sighting.mac,
                reason: rust_i18n::t!(sighting.reason_key).to_string(),
            })
            .collect(),
        s: strings(),
    }
    .render()
    .expect("the example inventory template and its struct are compiled together")
}

/// What `/triage` needs: the store it reads, and the perimeter it displays.
///
/// 🔴 **The perimeter is a FIELD, never an `std::env::var` at the point of use.** Story 6b.2's
/// own mutation table calls reading it in the handler **M12** and predicts a red — M12 was never
/// executed, and the first implementation of this handler WAS M12, shipped, with the whole suite
/// green. The cost was not stylistic: [`crate::AppConfig::from_env`] discards a blank value and a
/// second reader does not, so a blanked variable rendered *"not configured"* on the nine
/// demonstration screens and a dangling label on this one — one fact, one shell, two behaviours,
/// on the very screen `/` redirects to.
///
/// Configuration enters as a PARAMETER (story 6.1's rule), and the sub-router is what carries it.
#[derive(Clone)]
pub(crate) struct TriageState {
    /// The store the reconciliation card reads.
    pub(crate) pool: MySqlPool,
    /// The configured perimeter, already normalised by [`crate::AppConfig::from_env`] — `None`
    /// when unset OR blank, which is why this handler must not re-derive it.
    pub(crate) perimeter: Option<String>,
}

/// `/triage` on its own state, so the perimeter arrives as a parameter rather than being read.
///
/// # Returns
///
/// A router to be merged BEFORE `.layer(auth_deny)`, like every other route (story 6.1 §2): the
/// screen is not public, and merging after the layer would bypass the middleware entirely.
pub(crate) fn triage_router(pool: MySqlPool, perimeter: Option<String>) -> Router {
    Router::new()
        .route("/triage", get(triage))
        .with_state(TriageState { pool, perimeter })
}

/// `GET /triage` — the shell, with today's reconciliation card inside it.
///
/// # Why this handler exists, and why it is not on the demonstration sub-router
///
/// 🔴 Story 6b.2 turned `/` into a redirect. Without this handler nothing would route to the
/// reconciliation card at all — `/gap` serves the *fragment*, and the page that hosted it is
/// gone — so the product's ONLY fed screen would have vanished between this story and 6b.4, in
/// an epic whose purpose is to make the product more usable. The validation layer measured the
/// second half of that: `index` became dead code and `clippy -D warnings` failed.
///
/// It therefore keeps `State<MySqlPool>` and lives on the main router, while the nine
/// demonstration screens sit on a pool-free one. Epic constraint 1 is about demonstrations, and
/// it is enforced exactly where it applies.
///
/// Story 6b.4 replaces this body with the mock's two-pane triage; the frame it renders into stays.
pub async fn triage(State(state): State<TriageState>) -> Response {
    let perimeter = state.perimeter.clone();
    match reconcile_view(&state.pool).await {
        Ok((view, identity)) => {
            let card = GapFragment {
                view,
                identity,
                s: strings(),
            };
            match card.render() {
                Ok(body) => Html(render_shell(
                    Shell::new(crate::screens::Screen::Triage, perimeter),
                    body,
                ))
                .into_response(),
                Err(error) => {
                    tracing::error!(%error, "rendering the triage card");
                    (StatusCode::INTERNAL_SERVER_ERROR, "template error").into_response()
                }
            }
        }
        Err(response) => response,
    }
}

/// `GET /gap` — just the card, for HTMX refresh swaps.
pub async fn gap_fragment(State(pool): State<MySqlPool>) -> Response {
    match reconcile_view(&pool).await {
        Ok((view, identity)) => render(GapFragment {
            view,
            identity,
            s: strings(),
        }),
        Err(response) => response,
    }
}

fn render<T: Template>(template: T) -> Response {
    match template.render() {
        Ok(html) => Html(html).into_response(),
        Err(error) => {
            tracing::error!(%error, "rendering a template failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "template error").into_response()
        }
    }
}

/// `GET /assets/{*path}` — embedded, self-hosted static files (no CDN).
pub async fn asset(Path(path): Path<String>) -> Response {
    match Assets::get(&path) {
        Some(file) => (
            [(header::CONTENT_TYPE, content_type(&path))],
            file.data.into_owned(),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("woff2") => "font/woff2",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declared_row(entity: &str, key: &str, value: &str) -> (String, String, String) {
        (entity.into(), key.into(), value.into())
    }

    fn ipv4(addr: &str) -> Fact {
        Fact::IpV4 {
            addr: addr.parse().unwrap(),
        }
    }

    fn hostname(name: &str) -> Fact {
        Fact::Hostname {
            name: name.into(),
            source: opencmdb_core::observation::HostnameSource::Dns,
        }
    }

    /// A grouped-reach row as [`count_engine_reach`] returns it.
    fn reach(outcome: &str, cause: Option<&str>, count: i64) -> EngineReachRow {
        EngineReachRow {
            outcome: outcome.into(),
            cause: cause.map(Into::into),
            count,
        }
    }

    /// The identity view of a store the engine has never touched.
    fn no_reach() -> IdentityView {
        build_identity_view(Vec::new())
    }

    #[test]
    fn build_view_surfaces_a_drift_gap() {
        let declared = vec![
            declared_row("e1", "ipv4", "192.0.2.10"),
            declared_row("e1", "hostname", "nas"),
        ];
        let observations = vec![vec![ipv4("192.0.2.10"), hostname("intruder")]];
        let view = build_view(declared, observations, None);

        assert!(view.has_entity);
        assert_eq!(view.entity_ipv4, "192.0.2.10");
        assert_eq!(view.gaps.len(), 1);
        assert_eq!(view.gaps[0].field, "hostname");
        assert_eq!(view.gaps[0].declared, "nas");
        assert_eq!(view.gaps[0].observed, "intruder");
        assert_eq!(view.abstention_count, 0);
        // The card renders without error (through the i18n string seam).
        let html = GapFragment {
            view,
            identity: no_reach(),
            s: strings(),
        }
        .render()
        .unwrap();
        assert!(html.contains("intruder"));
    }

    #[test]
    fn build_view_counts_out_of_perimeter_as_reach() {
        let declared = vec![declared_row("e1", "ipv4", "192.0.2.10")];
        let observations = vec![vec![ipv4("192.0.2.99")]]; // an undocumented device
        let view = build_view(declared, observations, None);

        assert!(view.has_entity);
        assert!(view.gaps.is_empty());
        // Two abstentions, both honest reach: the undocumented device is Out of perimeter (1), and
        // with no in-perimeter observation the declared `ipv4` field has No observed value (1).
        assert_eq!(view.abstention_count, 2);
        assert!(
            view.abstentions
                .iter()
                .any(|a| a.cause == "Out of perimeter" && a.count == 1)
        );
        assert!(
            view.abstentions
                .iter()
                .any(|a| a.cause == "No observed value" && a.count == 1)
        );
    }

    #[test]
    fn build_view_empty_when_no_declared_entity() {
        let view = build_view(Vec::new(), Vec::new(), None);
        assert!(!view.has_entity);
        // The empty state renders honestly (default locale `en`).
        let html = triage_html(view, no_reach());
        assert!(html.contains("No declared record yet"));
    }

    // ── Story 5.14b: the identity engine's reach ──────────────────────

    /// **AC1** — one line PER CAUSE, and the fixture carries TWO different causes on purpose.
    ///
    /// A single-cause fixture cannot tell "group by cause" from "group by outcome": both produce one
    /// line. The second cause is what makes the assertion about grouping rather than about counting.
    #[test]
    fn the_identity_view_shows_one_line_per_cause() {
        let view = build_identity_view(vec![
            reach("abstained", Some("absence_of_proof"), 7),
            reach("abstained", Some("ambiguous"), 2),
            reach("match", None, 3),
        ]);

        assert_eq!(view.placed, 3, "the `match` rows are the placed sightings");
        assert_eq!(
            view.not_placed, 9,
            "and both abstained groups are not-placed"
        );
        assert_eq!(
            view.causes.len(),
            2,
            "ONE LINE PER CAUSE — collapsing the grouping, or grouping by outcome alone, gives one"
        );
        assert_eq!(view.causes[0].cause, "No proof of identity");
        assert_eq!(view.causes[0].count, 7);
        assert_eq!(view.causes[1].cause, "Several possible identities");
        assert_eq!(view.causes[1].count, 2);
    }

    /// **AC3** — the two engines' counts are never added, and the fixture makes that measurable.
    ///
    /// 🔴 **Read this for what it is: the two `assert_ne!`s below CANNOT FAIL.** Measured at the
    /// code review, under the summing mutation in BOTH directions this test stayed green — because
    /// it composes the two views itself, and neither builder can add a number it never sees. What
    /// it really carries is that both frames render side by side and that no rendered number is
    /// their sum. **The anti-sum property is carried by
    /// [`one_real_page_build_keeps_the_two_counts_apart`]**, which goes through `reconcile_view`,
    /// the impure edge where a sum can actually be written.
    ///
    /// The three numbers are still distinct on purpose (reconciliation 2, identity 9, sum 11), so
    /// the render assertion below is not vacuous even though the `assert_ne!`s are.
    #[test]
    fn the_two_engines_counts_are_never_added() {
        let declared = vec![declared_row("e1", "ipv4", "192.0.2.10")];
        let observations = vec![vec![ipv4("192.0.2.99")]]; // out of perimeter -> 2 abstentions
        let view = build_view(declared, observations, None);
        let identity = build_identity_view(vec![
            reach("abstained", Some("absence_of_proof"), 7),
            reach("abstained", Some("ambiguous"), 2),
        ]);

        assert_eq!(view.abstention_count, 2, "the premise: reconciliation is 2");
        assert_eq!(identity.not_placed, 9, "the premise: identity is 9");
        assert_ne!(
            view.abstention_count, 11,
            "the reconciliation count must not absorb the identity one — 2 + 9 = 11, and the two \
             range over DIFFERENT populations (declared fields vs sightings), so their sum denotes \
             nothing"
        );
        assert_ne!(
            identity.not_placed, 11,
            "and not the other way round either — this is the direction a zero-reconciliation \
             fixture cannot see"
        );

        // Both frames render, side by side, in one card.
        let html = GapFragment {
            view,
            identity,
            s: strings(),
        }
        .render()
        .unwrap();
        assert!(
            html.contains("Out of perimeter"),
            "the reconciliation frame"
        );
        assert!(html.contains("No proof of identity"), "the identity frame");
        assert!(
            !html.contains(">11<"),
            "and no rendered number is their sum"
        );
    }

    /// **AC4** — the section renders with NO declared entity at all.
    ///
    /// 🔴 This is the fresh install: nothing declared, a scan has run. Before the hoist the whole
    /// section lived inside `{% if view.has_entity %}` and was invisible in exactly this case — the
    /// default at first boot.
    ///
    /// ⚠️ `identity` is a sibling of `view` rather than a field of it, because `build_view` returns
    /// EARLY with a zeroed view here: an identity count carried inside it would be zeroed with it.
    #[test]
    fn the_identity_section_is_visible_without_a_declared_entity() {
        let view = build_view(Vec::new(), Vec::new(), None);
        let identity = build_identity_view(vec![reach("abstained", Some("absence_of_proof"), 4)]);

        assert!(!view.has_entity, "the premise: nothing is declared");
        let html = triage_html(view, identity);

        assert!(
            html.contains("No declared record yet"),
            "the declared side still says it is empty"
        );
        assert!(
            html.contains("No proof of identity"),
            "and the identity section is THERE — this is what the hoist buys, and it is the only \
             state a first boot has"
        );
        assert!(html.contains("Identity"), "under its own title");
    }

    /// **AC4** — the empty state, which is the OTHER half of a fresh install.
    ///
    /// Measured during validation: deleting this string from both locales left the whole suite
    /// green, because the branch rendered and nothing asserted on it.
    #[test]
    fn the_identity_section_says_so_when_nothing_has_been_observed() {
        let html = triage_html(build_view(Vec::new(), Vec::new(), None), no_reach());

        assert!(
            html.contains("Nothing observed yet"),
            "a store the engine has never touched says so, rather than showing a bare 0"
        );
    }

    /// **AC7** — an unrecognised token is COUNTED, labelled, and the page still renders.
    ///
    /// 🔴 The total must not shrink: a counter that silently drops a row it does not understand is
    /// lying by omission, which is worse than an unfamiliar word on the page. And `page.rs` turns
    /// any error into a 500 for the WHOLE page, so failing here would take the gap display down too.
    #[test]
    fn an_unrecognised_cause_is_counted_labelled_and_does_not_kill_the_page() {
        let view = build_identity_view(vec![
            reach("abstained", Some("absence_of_proof"), 5),
            reach("abstained", Some("a_cause_no_variant_names"), 3),
        ]);

        assert_eq!(
            view.not_placed, 8,
            "the total INCLUDES the unfamiliar row — dropping it would make the counter lie by \
             omission"
        );
        assert_eq!(view.causes.len(), 2, "and it gets its own line");
        let unknown = &view.causes[1];
        assert!(
            unknown.cause.contains("Unrecognised cause"),
            "labelled as unrecognised, not silently mislabelled as a known cause: {}",
            unknown.cause
        );
        assert!(
            unknown.cause.contains("a_cause_no_variant_names"),
            "and it carries the raw token, so the operator can report what it saw: {}",
            unknown.cause
        );

        let html = triage_html(build_view(Vec::new(), Vec::new(), None), view);
        assert!(
            html.contains("a_cause_no_variant_names"),
            "and THE PAGE RENDERS — this is the assertion the whole design is for"
        );
    }

    /// **AC5a** — both locales carry every identity key, asserted through the PER-CALL override.
    ///
    /// 🔴 `assert_ne!` against the key itself, and the reason is measured: a missing `rust-i18n` key
    /// is a SILENT ECHO — `t!` returns the literal `"identity.floor"`, with no compile error and no
    /// panic.
    ///
    /// ⚠️ **Be exact about what that makes tautological, because an earlier draft of this comment was
    /// not.** A render assertion is vacuous when it checks that *something* appeared, or checks for
    /// the key's own text. It is NOT vacuous when it names a distinctive phrase of the TRANSLATION —
    /// measured: deleting `identity.floor` from both locales reds this test AND
    /// `the_surface_states_both_limits_separately`, which asserts on *"floor is set by the data"*.
    /// The two guards are independent, and saying otherwise credited this one with a reach it does
    /// not have.
    ///
    /// ⚠️ `set_locale` is NOT used, and must not be: it is process-wide, so a test that calls it
    /// makes the suite order-dependent. Measured during validation at 2-3 varying reds out of 290 —
    /// one of them the ageing guard, reddened by a locale with no clock anywhere.
    #[test]
    fn both_locales_carry_every_identity_key() {
        use rust_i18n::t;
        const KEYS: [&str; 12] = [
            "identity.settled",
            "identity.outcome.no_match",
            "identity.all_placed",
            "identity.title",
            "identity.placed",
            "identity.not_placed",
            "identity.because",
            "identity.nothing_seen",
            "identity.unit",
            "identity.floor",
            "identity.cause.absence_of_proof",
            "identity.cause.ambiguous",
        ];
        for key in KEYS {
            for locale in ["en", "fr"] {
                let resolved = t!(key, locale = locale);
                assert_ne!(
                    resolved, key,
                    "`{key}` is missing from `{locale}`. A missing key is a SILENT ECHO — `t!` \
                     returns the key itself, with no compile error and no panic — so any test \
                     asserting merely that SOMETHING was rendered, or asserting on the key's own \
                     text, would pass here"
                );
            }
        }
        // The interpolated one, checked on its substitution rather than on its presence.
        for locale in ["en", "fr"] {
            let outcome = t!(
                "identity.outcome.unrecognised",
                locale = locale,
                token = "zz"
            );
            assert_ne!(outcome, "identity.outcome.unrecognised");
            assert!(
                outcome.contains("zz"),
                "the OUTCOME token must be substituted in `{locale}` too: {outcome}"
            );
            let resolved = t!("identity.cause.unrecognised", locale = locale, token = "zz");
            assert_ne!(resolved, "identity.cause.unrecognised");
            assert!(
                resolved.contains("zz"),
                "`%{{token}}` must be substituted in `{locale}`, else the operator never sees which \
                 token was unfamiliar: {resolved}"
            );
        }
    }

    /// **AC5b** — the two limit facts reach the SURFACE, and they are two sentences.
    ///
    /// They must not be fused: the unit is a property of THIS BUILD that Epic 6 removes; the floor
    /// is a permanent property of the problem. Fused, a reader carries the temporary one as
    /// permanent.
    ///
    /// ⚠️ **Both sentences are CONDITIONAL on `has_any`, and that is a deliberate scope this AC did
    /// not state.** A store the engine has never touched shows the section and neither limit —
    /// defensible, because a limit qualifies a number and there is no number to qualify, but it
    /// means AC5 is met *for a store that has something to show* and not unconditionally. The
    /// assertion below therefore builds a non-empty view, and the empty case is covered by
    /// [`the_identity_section_says_so_when_nothing_has_been_observed`], which asserts the other
    /// branch instead.
    #[test]
    fn the_surface_states_both_limits_separately() {
        let html = triage_html(
            build_view(Vec::new(), Vec::new(), None),
            build_identity_view(vec![reach("abstained", Some("absence_of_proof"), 4)]),
        );

        assert!(
            html.contains("counts sightings, not devices"),
            "this build's own limit: the number counts sightings"
        );
        assert!(
            html.contains("floor is set by the data"),
            "and the permanent one: the floor is the data's, not the engine's"
        );
    }

    /// **AC6** — the view builder reads NO clock, so the same store renders identically twice.
    ///
    /// ⚠️ **Read this for exactly what it is.** It carries the view builder's clock-freedom, and
    /// that is a strictly WEAKER property than `epics.md`'s ban (*"after six months of inaction it
    /// reads the same number"*), which is about the displayed number over calendar time — and this
    /// build FAILS that ban, measurably: the scanner keeps scanning while the operator is inactive,
    /// so the store itself grows. The ban is open, owned by Epic 6, and registered. This is a
    /// tripwire against a clock arriving here, never a proof that the number is stable.
    #[test]
    fn the_view_builder_has_no_clock_so_one_store_renders_identically() {
        let rows = vec![
            reach("abstained", Some("absence_of_proof"), 113),
            reach("match", None, 187),
        ];
        let render_once = || {
            triage_html(
                build_view(Vec::new(), Vec::new(), None),
                build_identity_view(rows.clone()),
            )
        };

        assert_eq!(
            render_once(),
            render_once(),
            "the same store must render byte for byte the same, twice, microseconds apart. \
             ⚠️ That is ALL this proves: a clock coarser than the gap between the two renders — a \
             date, a time of day, an age in days — passes it, measured"
        );
    }

    /// **AC6** — no gauge, no percentage, no badge markup in the identity section.
    #[test]
    fn the_page_carries_no_gauge_and_no_percentage() {
        let html = triage_html(
            build_view(Vec::new(), Vec::new(), None),
            build_identity_view(vec![reach("abstained", Some("absence_of_proof"), 113)]),
        );

        // ⚠️ `html` is the WHOLE page, not the identity section. Strictly stronger, so no hole —
        // but say so, or this reds one day for a reason its name does not predict.
        assert!(
            !html.contains('%'),
            "no percentage anywhere on the page: a rate presented as the state of the operator's \
             work is a grade"
        );
        assert!(!html.contains("<progress"), "no gauge");
        assert!(!html.contains("<meter"), "no gauge");
        assert!(html.contains("113"), "the premise: the number IS rendered");
    }

    /// **Guy's taxonomy** — `no_match` is case ONE and belongs on neither side of the pair.
    ///
    /// 🔴 Found by all three review layers: a bare `else` folded every non-`abstained` outcome into
    /// `placed`, so *a rule FORBADE this pair* was reported as a placement, and with no other row
    /// present the page rendered *"Every sighting was placed."* over it.
    #[test]
    fn a_forbidden_placement_is_neither_placed_nor_awaiting_the_operator() {
        let view = build_identity_view(vec![
            reach("match", None, 2),
            reach("no_match", None, 5),
            reach("abstained", Some("absence_of_proof"), 3),
        ]);

        assert_eq!(
            view.placed, 2,
            "a refused placement is NOT a placement — folding it here reported a refusal as a \
             success, which is the opposite of what the row means"
        );
        assert_eq!(
            view.not_placed, 3,
            "nor does it await the operator: `no_match` is the SOFTWARE deciding, so there is \
             neither a doubt to lift nor an entity to create"
        );
        assert_eq!(
            view.causes.len(),
            1,
            "one cause line, for the abstention only"
        );
        assert_eq!(
            view.settled.len(),
            1,
            "and `no_match` gets its own settled line"
        );
        assert_eq!(view.settled[0].count, 5);
        assert_eq!(view.settled[0].cause, "A rule forbade the placement");
    }

    /// **AC7's twin, on the OUTCOME token** — an unrecognised outcome is counted and labelled.
    ///
    /// The tolerant reader for the CAUSE token had a silent twin on the OUTCOME token: dropping
    /// `no_match` rows entirely left the whole suite green, unmeasured in both directions.
    #[test]
    fn an_unrecognised_outcome_is_counted_and_labelled_rather_than_folded() {
        let view = build_identity_view(vec![reach("wat", None, 4)]);

        assert_eq!(
            view.placed, 0,
            "an outcome nothing names is not a placement"
        );
        assert_eq!(view.not_placed, 0, "and it is not an abstention either");
        assert_eq!(view.settled.len(), 1);
        assert!(
            view.settled[0].cause.contains("wat"),
            "it carries its raw token so the operator can report it: {}",
            view.settled[0].cause
        );
        assert!(
            view.has_any,
            "and the section must NOT claim nothing was observed while holding a row"
        );
    }

    /// **AC4's third branch** — *"every sighting was placed"*, which nothing rendered.
    ///
    /// Measured at the code review: replacing this branch's body with nonsense left all 519 tests
    /// green. ⚠️ AC4's own comment records this exact defect found in validation for the SIBLING key
    /// `nothing_seen`; one half was fixed and the other missed.
    #[test]
    fn the_section_says_so_when_every_sighting_was_placed() {
        let html = triage_html(
            build_view(Vec::new(), Vec::new(), None),
            build_identity_view(vec![reach("match", None, 5)]),
        );

        assert!(
            html.contains("Every sighting was placed"),
            "the third of the section's three mutually exclusive states must render"
        );
        assert!(
            !html.contains("Not placed, because"),
            "and it must not also claim there is something to explain"
        );
    }

    /// **AC3, through the COMPOSITION** — the two counts survive one real page build unadded.
    ///
    /// # 🔴 Why this test exists, and it was found by a mutation coming back GREEN
    ///
    /// `the_two_engines_counts_are_never_added` above builds the two views and composes them
    /// ITSELF, so it can only ever prove that `build_view` and `build_identity_view` do not add —
    /// and **neither of them can**, since neither sees the other's numbers. The only place a sum can
    /// be written is [`reconcile_view`], the impure edge that assembles both, and no unit test
    /// reaches it. Measured: adding the reconciliation count into the identity one THERE left the
    /// whole suite green.
    ///
    /// 🔑 *A guard placed where the defect cannot occur reads as coverage and is none.* This one
    /// goes through `reconcile_view`, which needs a database, and plants both populations so the two
    /// counts and their sum are three distinct numbers.
    #[tokio::test]
    async fn one_real_page_build_keeps_the_two_counts_apart() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping page composition test: DATABASE_URL unset");
            return;
        };
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        for statement in [
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM interface",
            "DELETE FROM observation_record",
            "DELETE FROM declared_attribute",
        ] {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }

        // The DECLARED side: one entity, observed out of perimeter -> 2 reconciliation abstentions.
        crate::repo::insert_declared_attribute(&pool, "e1", "ipv4", "192.0.2.10")
            .await
            .expect("declare");
        // The IDENTITY side: a scan whose sightings the engine cannot place -> 3 abstentions.
        let mut source = crate::fixture_connector::FixtureConnector::from_observations(
            opencmdb_core::observation::ConnectorId::from_uuid(uuid::Uuid::from_u128(0x514b)),
            opencmdb_core::observation::Capabilities {
                as_of: chrono::DateTime::from_timestamp(1_700_000_000, 0).expect("in range"),
                kinds: std::collections::BTreeSet::from([
                    opencmdb_core::observation::FactKind::IpV4,
                ]),
            },
            vec![Scope {
                l2_domain: L2DomainId::from_uuid(Uuid::from_u128(0x514c)),
                vantage: VantageId::from_uuid(Uuid::nil()),
            }],
            "story 5.14b page composition",
            (0..3)
                .map(|i| Observation {
                    obs_id: ObsId::from_uuid(Uuid::from_u128(0x5150 + i)),
                    connector_id: ConnectorId::from_uuid(Uuid::from_u128(0x514b)),
                    observed_at: chrono::DateTime::from_timestamp(1_700_001_000 + i as i64, 0)
                        .expect("in range"),
                    scope: Scope {
                        l2_domain: L2DomainId::from_uuid(Uuid::from_u128(0x514c)),
                        vantage: VantageId::from_uuid(Uuid::nil()),
                    },
                    facts: vec![Fact::IpV4 {
                        addr: "192.0.2.99".parse().expect("a documentation address"),
                    }],
                    raw: None,
                })
                .collect(),
        )
        .expect("the in-memory stream must load");
        crate::scan_pass::poll_ingest_resolve(
            &mut source,
            chrono::DateTime::from_timestamp(1_700_001_000, 0).expect("in range"),
            &pool,
        )
        .await;

        let (view, identity) = reconcile_view(&pool).await.expect("build the page's state");

        // Three out-of-perimeter sightings are three reconciliation abstentions, plus one
        // `NoObservedValue` for the declared `ipv4` nothing in-perimeter reported.
        assert_eq!(
            view.abstention_count, 4,
            "the premise: the reconciliation side counts 4"
        );
        assert_eq!(
            identity.not_placed, 3,
            "the premise: the identity side counts 3 — and 4, 3 and 7 are three DISTINCT numbers, \
             which is what makes the assertions below able to fail"
        );
        assert_ne!(
            identity.not_placed, 7,
            "the identity count must not absorb the reconciliation one. This is the direction a \
             zero-reconciliation fixture cannot see, and it is the one a real page build reaches"
        );
        assert_ne!(view.abstention_count, 7, "nor the other way round");

        let html = triage_html(view, identity);
        assert!(
            !html.contains(">7<"),
            "and no rendered number is their sum: the two populations are declared FIELDS and \
             SIGHTINGS, so their total denotes nothing"
        );
    }

    /// **AC6** — this section's own CSS rules never reach for `--accent`.
    ///
    /// ⚠️ **This checks the TEXT OF THESE RULES, not a resolved colour**, and the difference was
    /// measured: `#gap-card .abstentions .cause { color: var(--accent) }` really does recolour the
    /// section — an id selector wins — and leaves this test green. *An assertion over CSS is an
    /// enumeration.* What carries *"does not redden"* is the palette, which holds no red at all
    /// (`--attention: #f0f4fa`, "severity by luminosity + weight, never hue").
    #[test]
    fn the_identity_sections_own_rules_never_reach_for_the_accent() {
        // ⚠️ Scan from each `.identity` selector to its closing brace, NOT only the selector lines.
        // Measured at the code review: filtering on lines that START with `.identity` misses a
        // multi-line rule entirely — `\n.identity .note {\n  color: var(--accent);\n}` left this
        // test green. That is not an adversary's shape; any formatter or hand edit produces it.
        let css = include_str!("../assets/app.css");
        let mut block: Vec<&str> = Vec::new();
        let mut inside = false;
        for line in css.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with(".identity") {
                inside = true;
            }
            if inside {
                block.push(line);
                if line.contains('}') {
                    inside = false;
                }
            }
        }

        assert!(
            block.len() >= 4,
            "the premise: this test found the section's rules ({} lines) — if the selector is \
             renamed this count is what tells you, rather than a silently empty scan",
            block.len()
        );
        for rule in &block {
            assert!(
                !rule.contains("--accent"),
                "`--accent` is the amber reserved for the document action, never for reach: {rule}"
            );
        }
    }

    /// 🔴 Story 6.3's code review — NFR5's divergence boundary, measured through the PRODUCTION
    /// path instead of a hand-assembled call.
    ///
    /// The story's first implementation asserted this boundary from `main.rs` by handing
    /// `gap::reconcile` in-memory `Observation` clones taken BEFORE ingestion, with a
    /// hand-supplied perimeter tuple. Two review layers found it independently: that proves the
    /// pure function's contract — which `opencmdb-core`'s own tests already cover — and **not** the
    /// path that decides WHICH observations feed a reconcile. AC5's own prose had named the risk
    /// ("a re-derivation of `build_view`'s perimeter selection in `main.rs`, a second oracle free
    /// to drift") and the implementation did it anyway.
    ///
    /// This one reads the observations back from the store and lets `build_view` choose the
    /// perimeter, which is what task T3 prescribed all along.
    #[tokio::test]
    async fn the_divergence_boundary_holds_through_the_real_page_build() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping divergence-boundary page test: DATABASE_URL unset");
            return;
        };
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        for statement in [
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM interface",
            "DELETE FROM observation_record",
            "DELETE FROM declared_attribute",
        ] {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }

        let scope = Scope {
            l2_domain: L2DomainId::from_uuid(Uuid::from_u128(0x6301)),
            vantage: VantageId::from_uuid(Uuid::nil()),
        };
        let connector = ConnectorId::from_uuid(Uuid::from_u128(0x6302));
        let sighting = |id: u128, host: &str| Observation {
            obs_id: ObsId::from_uuid(Uuid::from_u128(id)),
            connector_id: connector,
            observed_at: chrono::DateTime::from_timestamp(1_700_002_000 + id as i64, 0)
                .expect("in range"),
            scope,
            facts: vec![
                Fact::IpV4 {
                    addr: "192.0.2.60".parse().expect("a documentation address"),
                },
                Fact::Hostname {
                    name: host.to_string(),
                    source: opencmdb_core::observation::HostnameSource::Dns,
                },
            ],
            raw: None,
        };
        let ingest = |pool: MySqlPool, observations: Vec<Observation>| async move {
            let mut source = crate::fixture_connector::FixtureConnector::from_observations(
                connector,
                opencmdb_core::observation::Capabilities {
                    as_of: chrono::DateTime::from_timestamp(1_700_000_000, 0).expect("in range"),
                    kinds: std::collections::BTreeSet::from([
                        opencmdb_core::observation::FactKind::IpV4,
                        opencmdb_core::observation::FactKind::Hostname,
                    ]),
                },
                vec![scope],
                "story 6.3 divergence boundary",
                observations,
            )
            .expect("the in-memory stream must load");
            crate::scan_pass::poll_ingest_resolve(
                &mut source,
                chrono::DateTime::from_timestamp(1_700_002_500, 0).expect("in range"),
                &pool,
            )
            .await;
        };

        // The declared side, as the documenting gesture leaves it.
        for (key, value) in [("ipv4", "192.0.2.60"), ("hostname", "nas")] {
            crate::repo::insert_declared_attribute(&pool, "e63", key, value)
                .await
                .expect("declare");
        }

        // ONE contradicting sighting in perimeter -> the divergence OPENS.
        ingest(pool.clone(), vec![sighting(0x6310, "intruder")]).await;
        let (view, _) = reconcile_view(&pool).await.expect("build the page's state");
        assert_eq!(view.gaps.len(), 1, "the divergence opens");
        assert_eq!(view.gaps[0].field, "hostname");
        assert_eq!(view.gaps[0].declared, "nas");
        assert_eq!(view.gaps[0].observed, "intruder");
        assert_eq!(
            view.abstention_count, 0,
            "nothing abstains while one sighting carries the field"
        );

        // A SECOND, DISAGREEING sighting -> the gap CLOSES into two abstentions. FR16 working:
        // never picked, never merged. This is the shape a real network produces, and it is why
        // `epics.md:1790`'s "a divergence opens" is unreachable while the older sighting lives.
        ingest(pool.clone(), vec![sighting(0x6311, "nas")]).await;
        let (view, _) = reconcile_view(&pool).await.expect("build the page's state");
        assert!(
            view.gaps.is_empty(),
            "two disagreeing sightings must not pick one, yet {} gap(s) opened",
            view.gaps.len()
        );
        assert_eq!(
            view.abstention_count, 2,
            "the conflict and the now-unobserved field"
        );
    }

    // ── Story 6b.1: the design system ────────────────────────────────────
    //
    // These read the committed stylesheet and the templates as TEXT. That is a deliberate
    // limit, stated once here rather than in each test: an assertion over CSS is an
    // enumeration, and a more specific selector elsewhere can override any rule they check.
    // What they carry is that the SOURCE says what the story says it says.

    /// What `/triage` actually serves: the reconciliation card inside the shell.
    ///
    /// 🔑 Story 6b.2 removed `GapPage`/`gap.html` — the shell IS the document now — so tests that
    /// rendered the standalone page render this instead. They gain rather than lose: they assert
    /// over the bytes the product really sends, frame included.
    fn triage_html(view: ReconciledView, identity: IdentityView) -> String {
        let card = GapFragment {
            view,
            identity,
            s: strings(),
        }
        .render()
        .expect("the card renders");
        render_shell(Shell::new(crate::screens::Screen::Triage, None), card)
    }

    /// The stylesheet, as bytes, for every test in this section.
    fn sheet() -> &'static str {
        include_str!("../assets/app.css")
    }

    /// **Every** template under `templates/`, read from disk at test time.
    ///
    /// 🔴 This ENUMERATES the directory; it does not list it. Story 6b.1's review rewrote two
    /// guards as properties *"over the sheet AND both templates"*, keyed to a
    /// `[&'static str; 2]` literal — and story 6b.2's validation measured what that costs: plant
    /// `data-theme="dark"` in a NEW partial and `style="color: var(--accent-document)"` on ten
    /// nav entries, and **607 tests stay green**, because a typed literal array does not fail to
    /// compile when a file is added. *A guard repaired yesterday, undone by the ordinary act of
    /// adding a file.*
    ///
    /// Reading the directory costs a filesystem call in a test and makes the omission
    /// impossible: a template that exists is a template that is scanned.
    ///
    /// # Panics
    ///
    /// If `templates/` cannot be read — which means the test is running somewhere the source
    /// tree is not, and every guard below would be vacuous rather than merely wrong.
    fn templates() -> Vec<(String, String)> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
        let mut found: Vec<(String, String)> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("templates/ must be readable at {}: {e}", dir.display()))
            .filter_map(Result::ok)
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "html"))
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().into_owned();
                let body = std::fs::read_to_string(entry.path())
                    .unwrap_or_else(|e| panic!("reading {name}: {e}"));
                (name, body)
            })
            .collect();
        found.sort();
        assert!(
            found.len() >= 3,
            "the premise: templates/ holds at least the card, the nav and the shell ({} found) \
             — an empty scan would make every guard below assert nothing",
            found.len()
        );
        found
    }

    /// The stylesheet with `/* … */` comments removed.
    ///
    /// 🔴 Scanning CSS **with** its comments is a measured defect here, not a theoretical one:
    /// the radius comment in this sheet contains the text `var(--radius-*)` in order to say
    /// that nothing reads those tokens, and the scanner below counted it as a read — the guard
    /// reddened on a sentence. `xtask`'s `float-free` gate solved the same problem the same way
    /// in story 5.4b, precisely so the architecture may be QUOTED without tripping a gate.
    fn without_comments(css: &str) -> String {
        let mut out = String::with_capacity(css.len());
        let mut rest = css;
        while let Some(start) = rest.find("/*") {
            out.push_str(&rest[..start]);
            match rest[start..].find("*/") {
                Some(end) => rest = &rest[start + end + 2..],
                None => return out,
            }
        }
        out.push_str(rest);
        out
    }

    /// Every `--token` defined in a `:root` block that **no condition wraps**.
    ///
    /// 🔴 This walks brace DEPTH rather than searching for the first `:root {`, and the review
    /// measured why: a `@media (prefers-color-scheme: dark) { :root { … } }` placed above the
    /// real block — an ordinary thing to write — made a first-match search return the
    /// CONDITIONAL block's tokens, after which a live rule could read a value that exists only
    /// under that condition while the guard stayed green.
    ///
    /// A `:root` at depth 0 applies always; one at depth 1 sits inside an at-rule and does not.
    fn unconditional_tokens(css: &str) -> Vec<String> {
        let css = without_comments(css);
        let mut tokens = Vec::new();
        let mut depth = 0usize;
        let mut in_root = false;

        for line in css.lines() {
            let trimmed = line.trim();
            if trimmed.ends_with('{') {
                if depth == 0 {
                    let name = trimmed.trim_end_matches('{').trim();
                    in_root = name.split(',').any(|s| s.trim() == ":root");
                }
                depth += 1;
                continue;
            }
            if trimmed.starts_with('}') {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    in_root = false;
                }
                continue;
            }
            if depth == 1
                && in_root
                && let Some(name) = trimmed.strip_prefix("--").and_then(|l| l.split(':').next())
            {
                tokens.push(format!("--{name}"));
            }
        }
        tokens
    }

    /// Every `--token` a rule READS, in the spellings CSS actually permits.
    ///
    /// 🔴 `var( --x )` (spaces inside the parens) and `var(--x, fallback)` are ordinary CSS and
    /// both defeated a plain `matches("var(--x)")` count — measured by the review, on the guard
    /// that exists to keep the amber unused.
    fn tokens_read_by_rules(css: &str) -> Vec<String> {
        let css = without_comments(css).replace("var( ", "var(");
        let mut used = Vec::new();
        for (i, _) in css.match_indices("var(--") {
            let rest = &css[i + 4..];
            let name = rest.split([')', ',', ' ', '\n']).next().unwrap_or("");
            if !name.is_empty() {
                used.push(name.to_string());
            }
        }
        used
    }

    /// Every URL the product tells a browser to fetch — from the sheet AND both templates.
    ///
    /// 🔴 The first no-external-request guard was a list of four probes (`http://`, `https://`,
    /// `//fonts.`, `@import url(`). The review measured what it lets through:
    /// `url(//evil-cdn.example.net/track.gif)` — a real cross-origin request every visiting
    /// browser would make — because the `//fonts.` probe assumed the only protocol-relative
    /// risk was a font CDN. *An enumeration of bad hosts cannot express "nothing leaves this
    /// binary".* So the URLs are collected and the assertion states the property instead.
    fn referenced_urls() -> Vec<String> {
        let mut urls = Vec::new();
        let css = without_comments(sheet());
        for (i, _) in css.match_indices("url(") {
            let rest = &css[i + 4..];
            if let Some(end) = rest.find(')') {
                urls.push(rest[..end].trim().trim_matches(['"', '\'']).to_string());
            }
        }
        for (_name, html) in templates() {
            for attr in ["src=\"", "href=\""] {
                for (i, _) in html.match_indices(attr) {
                    let rest = &html[i + attr.len()..];
                    if let Some(end) = rest.find('"') {
                        urls.push(rest[..end].to_string());
                    }
                }
            }
        }
        urls
    }

    /// AC2 — the palette is the mock's, not the walking skeleton's.
    #[test]
    fn ac2_the_sheet_carries_the_mocks_light_base_and_ramps() {
        let css = sheet();
        for (token, value) in [
            ("--color-bg", "#f2f2f3"),
            ("--color-surface", "#e9e9ea"),
            ("--color-text", "#1d1f20"),
            ("--color-accent", "#5980a6"),
            ("--color-neutral-500", "#98989b"),
            ("--color-accent-700", "#416180"),
        ] {
            assert!(
                css.contains(&format!("{token}: {value}")),
                "the mock's token {token} must carry {value}"
            );
        }
    }

    /// AC2 — the typefaces are embedded, and nothing is fetched from the network.
    ///
    /// The second half is the one that matters for the single-binary promise: a `@font-face`
    /// pointing at a CDN would render identically on the developer's machine and fail on an
    /// air-gapped one.
    #[test]
    fn ac2_five_faces_are_declared_and_no_request_leaves_the_product() {
        let css = sheet();
        for face in [
            "fonts/Barlow-Regular.woff2",
            "fonts/Barlow-Medium.woff2",
            "fonts/Barlow-Bold.woff2",
            "fonts/BarlowCondensed-Regular.woff2",
            "fonts/BarlowCondensed-SemiBold.woff2",
        ] {
            assert!(css.contains(face), "the sheet must declare {face}");
        }
        assert_eq!(
            css.matches("@font-face").count(),
            5,
            "five faces, and the count is what tells you when a sixth arrives unannounced"
        );

        // 🔴 Declaring a face in the sheet is not shipping it. Measured on 2026-08-18: `cargo
        // build` does not see a NEW file under `assets/` — the binary is built, reports
        // `Finished`, and embeds nothing — so a test reading only the CSS would pass over a
        // product that serves 404 for every glyph. These five reads are what M8 reddens.
        for face in [
            "fonts/Barlow-Regular.woff2",
            "fonts/Barlow-Medium.woff2",
            "fonts/Barlow-Bold.woff2",
            "fonts/BarlowCondensed-Regular.woff2",
            "fonts/BarlowCondensed-SemiBold.woff2",
        ] {
            let embedded = Assets::get(face)
                .unwrap_or_else(|| panic!("{face} is declared by the sheet but not embedded"));
            assert!(
                embedded.data.len() > 40_000,
                "{face} is embedded but truncated ({} bytes) — a placeholder would pass a \
                 presence check",
                embedded.data.len()
            );
        }

        // The licence travels with the fonts (SIL OFL 1.1 requires it), and it is served
        // rather than hidden — measured: `/assets/fonts/OFL.txt` answers 200. That is the
        // licence doing its job, not a leak.
        assert!(
            Assets::get("fonts/OFL.txt").is_some(),
            "OFL 1.1 requires the notice to travel with the faces"
        );

        // 🔴 Not a probe list. Every URL the sheet and the templates hand a browser must be
        // served by US — measured as a property, because the four-probe version let
        // `url(//evil-cdn.example.net/track.gif)` through, a real cross-origin request.
        let urls = referenced_urls();
        assert!(
            urls.len() >= 7,
            "the premise: five faces + the sheet + htmx + app.js are referenced ({} found) — a \
             scan that went empty would assert nothing below",
            urls.len()
        );
        // 🔴 The property is *nothing leaves this product*, not *everything is an asset*: a
        // template legitimately links `/gap` or `#`, and a first version of this guard reddened
        // on `_gap_card.html`'s own anchor. An absolute-external reference is what must not
        // exist — `scheme://host`, or the protocol-relative `//host` a four-probe list missed.
        for url in &urls {
            assert!(
                !url.contains("://") && !url.starts_with("//"),
                "nothing may be fetched from outside this binary: {url:?} names a host"
            );
        }
        // The five faces specifically must come from our own asset route.
        let faces: Vec<&String> = urls.iter().filter(|u| u.ends_with(".woff2")).collect();
        assert_eq!(faces.len(), 5, "five faces, referenced from the sheet");
        for face in faces {
            assert!(
                face.starts_with("/assets/fonts/"),
                "a face must be served by us: {face:?}"
            );
        }
        assert!(
            !without_comments(css).contains("@import"),
            "an @import pulls a second sheet, and its URL is not one this guard can see"
        );
    }

    /// AC3 — the dark set is still in the sheet.
    ///
    /// Its twin below asserts that nothing READS it. Two claims, two tests: deleting the block
    /// and depending on the block are different defects, and M7/M7b are their two mutations.
    #[test]
    fn ac3_the_dark_token_set_is_still_present() {
        let css = sheet();
        assert!(
            css.contains(r#"[data-theme="dark"]"#),
            "the dark token block must still be in the sheet — its return is a story, not an \
             excavation"
        );
        assert!(
            css.contains("#0f1420"),
            "the dark background must still be there; deleting the block is what this catches"
        );
    }

    /// AC3, first direction — the light set is what renders, because nothing selects the other.
    ///
    /// This is a claim about the TEMPLATES, not about the sheet: the dark block is inert only
    /// for as long as no `data-theme` attribute reaches a browser. The two halves of AC3 are
    /// two claims and need two tests — checking the sheet says nothing about the markup.
    #[test]
    fn ac3_no_template_selects_a_theme() {
        for (name, html) in templates() {
            assert!(
                !html.contains("data-theme"),
                "{name} must select no theme — the light set renders because it is the \
                 unconditional one, and story 6b.1 removed the hardcoded dark attribute"
            );
        }
    }

    /// AC3, second direction — 🔴 the half a prototype measured GREEN before this test existed.
    ///
    /// *"Referenced by nothing"* is not carried by checking the TEMPLATE for `data-theme`: that
    /// is a different claim. A rule outside the conditional block may read a token the block
    /// alone defines, and then the sheet depends on a theme nothing selects — silently, because
    /// `var()` on an undefined token simply yields nothing.
    ///
    /// So: every token a live rule uses must be defined UNCONDITIONALLY.
    #[test]
    fn ac3_no_live_rule_depends_on_a_conditional_block() {
        let css = sheet();
        let unconditional = unconditional_tokens(css);
        // ⚠️ These floors are the MEASURED counts, not round numbers an order of magnitude
        // below them. A floor of 8 over 56 tokens catches only a TOTAL scan failure, never a
        // partial one — and a partial mis-scope is what a brace inside a future comment or
        // string would cause, since `unconditional_tokens` searches for `:root {` literally
        // and stops at the first `}` with no depth tracking. Raise these with the sheet.
        assert!(
            unconditional.len() >= 56,
            "the premise: the base :root block defines 56 tokens ({} found) — a scan that \
             lost part of the block would still clear a low floor and then compare against an \
             incomplete set",
            unconditional.len()
        );

        let used = tokens_read_by_rules(css);
        assert!(
            used.len() >= 42,
            "the premise: the sheet makes 42 `var()` reads ({} found) — same reasoning as the \
             floor above",
            used.len()
        );

        for token in &used {
            assert!(
                unconditional.contains(token),
                "{token} is read by a live rule but is not defined in the base :root block — \
                 the sheet would then depend on a theme no template selects"
            );
        }
    }

    /// AC4 — the amber is named for the gesture, and no structure reaches for it.
    #[test]
    fn ac4_the_amber_is_reserved_for_the_documenting_gesture() {
        let css = sheet();
        assert!(
            css.contains("--accent-document:"),
            "the amber must be named for what it means"
        );
        assert!(
            !css.contains("--accent:"),
            "the bare `--accent` must be gone — a token that names a colour rather than a \
             gesture is what let structure borrow it"
        );
        // 🔴 Counting the literal `var(--accent-document)` was measured evadable THREE ways,
        // each of them ordinary CSS or ordinary HTML: `var( --accent-document )` with spaces,
        // `var(--accent-document, #b5793a)` with a fallback, and an inline `style=` attribute
        // in a template the guard never read. So: scan the READS, normalised, in the sheet AND
        // in both templates.
        let mut amber_reads: Vec<String> = tokens_read_by_rules(css)
            .into_iter()
            .filter(|token| token == "--accent-document")
            .collect();
        for (_name, html) in templates() {
            amber_reads.extend(
                tokens_read_by_rules(&html)
                    .into_iter()
                    .filter(|token| token == "--accent-document"),
            );
        }
        assert_eq!(
            amber_reads.len(),
            0,
            "story 6.4 adds the first legitimate use; until then the honest count is zero, and \
             this number is what tells you when one arrives — found {amber_reads:?}"
        );

        // 🔴 Absence is half the claim. Without the positive half, migrating `.refresh:hover`
        // to `--color-accent-2` — the OTHER ramp, also defined unconditionally — leaves every
        // check above green while the button renders in the wrong hue. Measured as a gap by
        // the review's diff-only layer, which could not see the rest of the file.
        for rule in [
            ".card:focus { outline: 2px solid var(--color-accent);",
            "  color: var(--color-accent);",
            "  border: 1px solid var(--color-accent);",
            ".refresh:hover { background: color-mix(in srgb, var(--color-accent) 12%",
        ] {
            assert!(
                css.contains(rule),
                "the four structural sites must read the mock's PRIMARY blue: {rule:?} is not \
                 in the sheet"
            );
        }
    }

    /// AC7b — D37: the vendored asset carries its version in its filename.
    #[test]
    fn ac7b_htmx_is_vendored_under_its_versioned_name() {
        // Story 6b.2 replaced `gap.html` with the shell, which is now the document every
        // screen renders inside — so this reads the shell instead.
        let gap = include_str!("../templates/_shell.html");
        assert!(
            gap.contains("/assets/vendor/htmx-2.0.4.min.js"),
            "D37: the version belongs in the filename, so an upgrade is visible in the diff"
        );
        assert!(
            !gap.contains("/assets/htmx.min.js"),
            "the unversioned path must be gone, not merely unused"
        );
        assert!(
            Assets::get("vendor/htmx-2.0.4.min.js").is_some(),
            "and the file must actually be embedded under that name"
        );
    }
}
