//! The single page that shows the gap (Story 3.7).
//!
//! It reconciles the persisted `declared_attribute` rows against the persisted `observation_record`
//! facts through the SAME pure `reconcile` engine (Story 3.6) and renders the result with Askama.
//! The view-building is a PURE function (`build_view`) so it is unit-tested without a database; the
//! DB read and the HTTP wrapping are the only impure edges.

use askama::Template;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use opencmdb_core::observation::{
    ConnectorId, Fact, L2DomainId, ObsId, Observation, Scope, VantageId,
};
use opencmdb_core::{AbstentionCause, reconcile};
use sqlx::MySqlPool;
use uuid::Uuid;

use opencmdb_core::identity::cascade::IdentityAbstentionCause;

use crate::repo::{
    ReachRow, cause_from_token, classify, load_current_engine_reach, load_declared_attributes,
    load_observation_facts,
};

/// Committed front-end assets, embedded into the binary (no CDN, self-hosted single binary).
#[derive(rust_embed::Embed)]
#[folder = "assets/"]
struct Assets;

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

/// One line of the identity-reach breakdown: a cause, and how many interfaces-worth of engine
/// links carry it. ONE line per cause — never one row per interface (the UX spec's *"N failures"*).
struct IdentityCauseRow {
    cause: String,
    count: usize,
}

/// The identity engine's reach: how many current engine links placed something, how many did not,
/// and the second number broken down by cause.
struct IdentityReach {
    evaluated: usize,
    not_evaluated: usize,
    by_cause: Vec<IdentityCauseRow>,
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
    reach: IdentityReach,
}

/// The user-facing strings, resolved through the i18n `t!()` seam (Story 3.8). The templates read
/// these instead of literals, so every rendered string flows through `rust-i18n`.
struct Strings {
    tagline: String,
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
    identity_reach: String,
    identity_evaluated: String,
    identity_not_evaluated: String,
    identity_because: String,
    identity_floor: String,
}

fn strings() -> Strings {
    use rust_i18n::t;
    Strings {
        tagline: t!("page.tagline").to_string(),
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
        identity_reach: t!("page.identity_reach").to_string(),
        identity_evaluated: t!("page.identity_evaluated").to_string(),
        identity_not_evaluated: t!("page.identity_not_evaluated").to_string(),
        identity_because: t!("page.identity_because").to_string(),
        identity_floor: t!("page.identity_floor").to_string(),
    }
}

#[derive(Template)]
#[template(path = "gap.html")]
struct GapPage {
    view: ReconciledView,
    s: Strings,
}

#[derive(Template)]
#[template(path = "_gap_card.html")]
struct GapFragment {
    view: ReconciledView,
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

/// A human label for an IDENTITY abstention cause — a SECOND vocabulary over a SECOND population.
///
/// ⚠️ Deliberately NOT a widening of [`cause_label`]: that one renders `gap::AbstentionCause` over
/// declared attributes, this one renders `IdentityAbstentionCause` over interfaces. A shared
/// function taking either enum is the silent bridge `deferred-work.md` forbids.
fn identity_cause_label(cause: IdentityAbstentionCause) -> String {
    use rust_i18n::t;
    match cause {
        IdentityAbstentionCause::Ambiguous => t!("identity_cause.ambiguous"),
        IdentityAbstentionCause::AbsenceOfProof => t!("identity_cause.absence_of_proof"),
    }
    .to_string()
}

/// PURE: count the engine's reach out of the current engine links.
///
/// A row carrying an `interface_id` is EVALUATED; a row carrying none is NOT evaluated, and its
/// persisted token is parsed back into [`IdentityAbstentionCause`]. An unknown token is REFUSED —
/// a cause the domain does not have must not reach a screen.
///
/// # Errors
///
/// The offending token, when a row carries one no variant parses.
fn build_reach(rows: &[ReachRow]) -> Result<IdentityReach, String> {
    let mut evaluated = 0usize;
    // A `BTreeMap` keyed by the DOMAIN variant: the order is the enum's own, deterministic, and
    // owed to nothing the database happens to return.
    let mut counts: std::collections::BTreeMap<usize, (IdentityAbstentionCause, usize)> =
        std::collections::BTreeMap::new();
    for row in rows {
        if row.interface_id.is_some() {
            evaluated += 1;
            continue;
        }
        let token = row.abstention_cause.as_deref().unwrap_or("");
        let cause = cause_from_token(token).ok_or_else(|| token.to_string())?;
        let ordinal = IdentityAbstentionCause::all()
            .iter()
            .position(|c| *c == cause)
            .expect("all() carries every variant");
        counts.entry(ordinal).or_insert((cause, 0)).1 += 1;
    }
    let not_evaluated = counts.values().map(|(_, n)| *n).sum();
    Ok(IdentityReach {
        evaluated,
        not_evaluated,
        by_cause: counts
            .into_values()
            .map(|(cause, count)| IdentityCauseRow {
                cause: identity_cause_label(cause),
                count,
            })
            .collect(),
    })
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
    reach: IdentityReach,
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
            reach,
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
        reach,
    }
}

// ── The impure edges: DB read + HTTP handlers ────────────────────────

/// Load the declared + observed state and build the view. `OPENCMDB_ENTITY_IPV4` selects the
/// perimeter entity when set.
async fn reconcile_view(pool: &MySqlPool) -> Result<ReconciledView, Response> {
    let declared = load_declared_attributes(pool).await.map_err(server_error)?;
    let observations = load_observation_facts(pool).await.map_err(server_error)?;
    let rows = load_current_engine_reach(pool).await.map_err(server_error)?;
    // An unknown persisted cause is REFUSED rather than rendered (§6). It cannot be reached through
    // the writer — `cause_token` is the only producer — so the only way here is a hand-written row.
    let reach = build_reach(&rows).map_err(|token| {
        tracing::error!(token, "a persisted abstention cause no variant parses");
        (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
    })?;
    let preferred = std::env::var("OPENCMDB_ENTITY_IPV4").ok();
    Ok(build_view(declared, observations, preferred, reach))
}

fn server_error(error: sqlx::Error) -> Response {
    let repo_error = classify(error);
    tracing::error!(?repo_error, "loading the page's state failed");
    (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response()
}

/// `GET /` — the full page.
pub async fn index(State(pool): State<MySqlPool>) -> Response {
    match reconcile_view(&pool).await {
        Ok(view) => render(GapPage { view, s: strings() }),
        Err(response) => response,
    }
}

/// `GET /gap` — just the card, for HTMX refresh swaps.
pub async fn gap_fragment(State(pool): State<MySqlPool>) -> Response {
    match reconcile_view(&pool).await {
        Ok(view) => render(GapFragment { view, s: strings() }),
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

    /// An empty reach, for the tests whose subject is the DECLARED side.
    fn no_reach() -> IdentityReach {
        build_reach(&[]).expect("an empty slice parses")
    }

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

    #[test]
    fn build_view_surfaces_a_drift_gap() {
        let declared = vec![
            declared_row("e1", "ipv4", "192.0.2.10"),
            declared_row("e1", "hostname", "nas"),
        ];
        let observations = vec![vec![ipv4("192.0.2.10"), hostname("intruder")]];
        let view = build_view(declared, observations, None, no_reach());

        assert!(view.has_entity);
        assert_eq!(view.entity_ipv4, "192.0.2.10");
        assert_eq!(view.gaps.len(), 1);
        assert_eq!(view.gaps[0].field, "hostname");
        assert_eq!(view.gaps[0].declared, "nas");
        assert_eq!(view.gaps[0].observed, "intruder");
        assert_eq!(view.abstention_count, 0);
        // The card renders without error (through the i18n string seam).
        let html = GapFragment { view, s: strings() }.render().unwrap();
        assert!(html.contains("intruder"));
    }

    #[test]
    fn build_view_counts_out_of_perimeter_as_reach() {
        let declared = vec![declared_row("e1", "ipv4", "192.0.2.10")];
        let observations = vec![vec![ipv4("192.0.2.99")]]; // an undocumented device
        let view = build_view(declared, observations, None, no_reach());

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

    fn placed(n: u8) -> ReachRow {
        ReachRow {
            interface_id: Some(format!("00000000-0000-0000-0000-0000000000{n:02x}")),
            abstention_cause: None,
        }
    }

    fn abstained(token: &str) -> ReachRow {
        ReachRow {
            interface_id: None,
            abstention_cause: Some(token.into()),
        }
    }

    /// AC2/AC3 — a NON-EMPTY population, evaluated beside not-evaluated, one line per cause.
    #[test]
    fn reach_counts_evaluated_beside_not_evaluated_and_groups_by_cause() {
        let rows = vec![
            placed(1),
            placed(2),
            placed(3),
            abstained("absence_of_proof"),
            abstained("absence_of_proof"),
        ];
        let reach = build_reach(&rows).expect("every token parses");
        assert_eq!(reach.evaluated, 3);
        assert_eq!(reach.not_evaluated, 2);
        // ONE line per cause — never one row per interface.
        assert_eq!(reach.by_cause.len(), 1, "two abstentions, ONE line");
        assert_eq!(reach.by_cause[0].count, 2);
    }

    /// AC3 — an unknown persisted token is REFUSED, not rendered.
    #[test]
    fn an_unknown_persisted_cause_is_refused() {
        let rows = vec![abstained("confidence_too_low")];
        assert_eq!(
            build_reach(&rows).err().as_deref(),
            Some("confidence_too_low"),
            "a cause the domain does not have must not reach a screen"
        );
    }

    /// The round trip: every token `cause_token` writes, `cause_from_token` reads back.
    #[test]
    fn every_persisted_cause_token_parses_back() {
        for cause in IdentityAbstentionCause::all() {
            assert_eq!(
                cause_from_token(crate::repo::cause_token(&cause)),
                Some(cause)
            );
        }
    }

    /// The rendered section, and every ban asserted over the HTML (AC4).
    fn rendered(rows: &[ReachRow]) -> String {
        let reach = build_reach(rows).expect("parses");
        let view = build_view(
            vec![declared_row("e1", "ipv4", "192.0.2.10")],
            vec![vec![ipv4("192.0.2.10")]],
            None,
            reach,
        );
        GapFragment { view, s: strings() }.render().unwrap()
    }

    /// AC4 — the bans, asserted rather than styled.
    #[test]
    fn the_reach_section_carries_no_alarm() {
        let rows = vec![placed(1), abstained("absence_of_proof"), abstained("absence_of_proof")];
        let html = rendered(&rows);

        // It renders at all, and it renders the two numbers.
        assert!(html.contains("Identity reach"), "the section renders");
        assert!(html.contains("not evaluated"));

        for banned in [
            "alert", "error", "danger", "warning", "critical", "<progress", "gauge", "badge",
            "meter", "overdue", "stale", "ago", "days", "since",
        ] {
            assert!(
                !html.contains(banned),
                "the reach section must carry no {banned:?} — reach, never a reproach"
            );
        }

        // ONE line per cause, never one row per interface: two abstentions, one <li>.
        let lines = html.matches(r#"<li><span class="count">"#).count();
        assert_eq!(
            lines, 1,
            "two abstentions render as ONE line — 'I don't know' is a MOTIF, never N failures"
        );
    }

    /// AC5 — the floor renders beside the number, EN and FR.
    #[test]
    fn the_floor_is_stated_where_the_number_is() {
        let html = rendered(&[abstained("absence_of_proof")]);
        assert!(html.contains("bounded by what the network tells us"));
        assert!(
            rust_i18n::t!("page.identity_floor", locale = "fr").contains("borné par ce que le réseau")
        );
    }

    /// AC6 — the two vocabularies keep separate labels and separate keys.
    #[test]
    fn the_two_abstention_vocabularies_share_no_label_and_no_key() {
        assert_ne!(
            cause_label(AbstentionCause::NoObservedValue),
            identity_cause_label(IdentityAbstentionCause::AbsenceOfProof)
        );
        let locales = include_str!("../locales/app.yml");
        assert!(locales.contains("identity_cause.absence_of_proof"));
        assert!(locales.contains("cause.no_observed_value"));
    }

    #[test]
    fn build_view_empty_when_no_declared_entity() {
        let view = build_view(Vec::new(), Vec::new(), None, no_reach());
        assert!(!view.has_entity);
        // The empty state renders honestly (default locale `en`).
        let html = GapPage { view, s: strings() }.render().unwrap();
        assert!(html.contains("No declared record yet"));
    }
}
