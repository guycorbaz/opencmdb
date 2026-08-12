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

use crate::repo::{
    EngineReachRow, classify, count_engine_reach, load_declared_attributes, load_observation_facts,
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
struct IdentityView {
    /// Sightings the engine placed on an interface.
    placed: i64,
    /// Sightings it could not place.
    not_placed: i64,
    /// Why, one line per cause — never one line per failure (FR16b).
    causes: Vec<IdentityCauseRow>,
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
        identity_title: t!("identity.title").to_string(),
        identity_placed: t!("identity.placed").to_string(),
        identity_not_placed: t!("identity.not_placed").to_string(),
        identity_because: t!("identity.because").to_string(),
        identity_floor: t!("identity.floor").to_string(),
        identity_unit: t!("identity.unit").to_string(),
        identity_nothing_seen: t!("identity.nothing_seen").to_string(),
        identity_all_placed: t!("identity.all_placed").to_string(),
    }
}

// ⚠️ `identity` is a SIBLING of `view`, not a field of it, and that is load-bearing: `build_view`
// returns EARLY with a fully-zeroed `ReconciledView` when no declared entity exists, so an identity
// count carried inside it would be silently zeroed in exactly the deployment the section exists for
// — a fresh install that has scanned and declared nothing.
#[derive(Template)]
#[template(path = "gap.html")]
struct GapPage {
    view: ReconciledView,
    identity: IdentityView,
    s: Strings,
}

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
    let mut placed = 0i64;
    let mut not_placed = 0i64;
    let mut causes: Vec<IdentityCauseRow> = Vec::new();
    for row in rows {
        if row.outcome == "abstained" {
            not_placed += row.count;
            causes.push(IdentityCauseRow {
                cause: identity_cause_label(row.cause.as_deref().unwrap_or_default()),
                count: row.count,
            });
        } else {
            placed += row.count;
        }
    }
    IdentityView {
        placed,
        not_placed,
        causes,
        has_any: placed + not_placed > 0,
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

/// `GET /` — the full page.
pub async fn index(State(pool): State<MySqlPool>) -> Response {
    match reconcile_view(&pool).await {
        Ok((view, identity)) => render(GapPage {
            view,
            identity,
            s: strings(),
        }),
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
        let html = GapPage {
            view,
            identity: no_reach(),
            s: strings(),
        }
        .render()
        .unwrap();
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
        assert_eq!(view.not_placed, 9, "and both abstained groups are not-placed");
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
    /// 🔴 The three numbers are DISTINCT ON PURPOSE: reconciliation 2, identity 9, sum 11. Measured
    /// during this story's validation, a fixture whose reconciliation count is zero — the natural
    /// one for a story about the identity section — leaves the summing mutation GREEN in one
    /// direction. **An anti-sum guard over a zero addend asserts nothing.**
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
        assert!(html.contains("Out of perimeter"), "the reconciliation frame");
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
        let html = GapPage {
            view,
            identity,
            s: strings(),
        }
        .render()
        .unwrap();

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
        let html = GapPage {
            view: build_view(Vec::new(), Vec::new(), None),
            identity: no_reach(),
            s: strings(),
        }
        .render()
        .unwrap();

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

        let html = GapPage {
            view: build_view(Vec::new(), Vec::new(), None),
            identity: view,
            s: strings(),
        }
        .render()
        .unwrap();
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
        const KEYS: [&str; 10] = [
            "identity.title",
            "identity.placed",
            "identity.not_placed",
            "identity.because",
            "identity.all_placed",
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
    #[test]
    fn the_surface_states_both_limits_separately() {
        let html = GapPage {
            view: build_view(Vec::new(), Vec::new(), None),
            identity: build_identity_view(vec![reach("abstained", Some("absence_of_proof"), 4)]),
            s: strings(),
        }
        .render()
        .unwrap();

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
            GapPage {
                view: build_view(Vec::new(), Vec::new(), None),
                identity: build_identity_view(rows.clone()),
                s: strings(),
            }
            .render()
            .unwrap()
        };

        assert_eq!(
            render_once(),
            render_once(),
            "the same store must render byte for byte the same, twice — nothing here may read a \
             clock, an elapsed time or an age"
        );
    }

    /// **AC6** — no gauge, no percentage, no badge markup in the identity section.
    #[test]
    fn the_identity_section_carries_no_gauge_and_no_percentage() {
        let html = GapPage {
            view: build_view(Vec::new(), Vec::new(), None),
            identity: build_identity_view(vec![reach("abstained", Some("absence_of_proof"), 113)]),
            s: strings(),
        }
        .render()
        .unwrap();

        assert!(!html.contains('%'), "no percentage: a rate presented as the state of the operator's work is a grade");
        assert!(!html.contains("<progress"), "no gauge");
        assert!(!html.contains("<meter"), "no gauge");
        assert!(html.contains("113"), "the premise: the number IS rendered");
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

        let html = GapPage {
            view,
            identity,
            s: strings(),
        }
        .render()
        .unwrap();
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
        let css = include_str!("../assets/app.css");
        let block: Vec<&str> = css
            .lines()
            .filter(|line| line.trim_start().starts_with(".identity"))
            .collect();

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
}
