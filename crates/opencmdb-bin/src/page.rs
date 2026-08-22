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
    ConnectorId, Fact, FactKind, L2DomainId, ObsId, Observation, Scope, VantageId,
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
        // 🔑 ONE source for the version, shared with the diagnostic screen (story 6b.9): two
        // `env!` sites would be two representations of one fact, and this file already pins
        // the rendered form.
        version: crate::VERSION,
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
#[derive(Clone)]
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
#[derive(Clone)]
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
    /// The reconciliation card's accessible name (story 6b.10).
    ///
    /// 🔴 It was an English LITERAL in `_gap_card.html` until story 6b.10's sweep, and it is the
    /// clearest case that story exists for: copy a **sighted browser look cannot see**, on the
    /// card that predates this epic. Nine French looks walked past it; one `grep` over the
    /// templates' human-text attributes found it.
    gap_card_label: String,
    /// The example-data marker's badge (story 6b.3).
    example_badge: String,
    /// The example-data marker's sentence (story 6b.3).
    example_sentence: String,
    /// The dashboard's real-half heading (story 6b.5).
    dash_real_heading: String,
    /// The dashboard's example-half heading (story 6b.5).
    dash_example_heading: String,
    /// What the dashboard says when a scan has landed and the identity pass has not run (6b.5).
    dash_pending_resolution: String,
    /// The label before the last-observation instant (story 6b.5).
    dash_last_observed: String,
    /// What the dashboard says when nothing has ever been observed (story 6b.5).
    dash_never_observed: String,
    /// The heading of the *what grew since your last visit* section (story 6b.5).
    dash_since_heading: String,
    /// Its body — an example, because the product has no notion of a visit (story 6b.5).
    dash_since_body: String,
    /// The triage screen's own heading (story 6b.4).
    triage_title: String,
    /// The queue's count line — a fact in a sentence, never a badge (story 6b.4).
    triage_pending: String,
    /// The age-sort toggle's label (story 6b.4, AC3).
    triage_sort_age: String,
    /// The sentence that says what a triage row IS (story 6b.4).
    triage_lede: String,
    /// What the queue says when there is nothing to triage (story 6b.4).
    triage_empty: String,
    /// The badge a not-yet-built GESTURE carries (story 6b.4b).
    ///
    /// ⚠️ It was a pair of its own, distinct from the `pending_*` pair an `Empty` screen carried —
    /// that one said *this SCREEN is not built*, a different population saying a different thing.
    /// Story 6b.9 removed the last `Empty` screen and the pair with it; a control is now the only
    /// thing in this product that says *not built*.
    gesture_badge: String,
    /// The one sentence the action bar shows, pointed at by every planned control (story 6b.4b).
    gesture_not_built: String,
    /// The perimeter label in the navigation footer, as the mock shows it (story 6b.2).
    nav_perimeter: String,
    entity: String,
    refresh: String,
    declared: String,
    observed: String,
    no_observation: String,
    the_gap: String,
    no_gap: String,
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
        gap_card_label: t!("page.gap_card_label").to_string(),
        example_badge: t!("example.badge").to_string(),
        example_sentence: t!("example.sentence").to_string(),
        dash_real_heading: t!("dash.real_heading").to_string(),
        dash_example_heading: t!("dash.example_heading").to_string(),
        dash_pending_resolution: t!("dash.pending_resolution").to_string(),
        dash_last_observed: t!("dash.last_observed").to_string(),
        dash_never_observed: t!("dash.never_observed").to_string(),
        dash_since_heading: t!("dash.since_heading").to_string(),
        dash_since_body: t!("dash.since_body").to_string(),
        triage_title: t!("triage.title").to_string(),
        triage_pending: t!("triage.pending").to_string(),
        triage_sort_age: t!("triage.sort_age").to_string(),
        triage_lede: t!("triage.lede").to_string(),
        triage_empty: t!("triage.empty").to_string(),
        gesture_badge: t!("gesture.badge").to_string(),
        gesture_not_built: t!("gesture.not_built").to_string(),
        nav_perimeter: t!("nav.perimeter").to_string(),
        entity: t!("page.entity").to_string(),
        refresh: t!("page.refresh").to_string(),
        declared: t!("page.declared").to_string(),
        observed: t!("page.observed").to_string(),
        no_observation: t!("page.no_observation").to_string(),
        the_gap: t!("page.the_gap").to_string(),
        no_gap: t!("page.no_gap").to_string(),
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

// ── Story 6b.4: the triage screen, on the real gap ───────────────────

/// Where a value came from and how fresh it is — one discreet line under each photo.
///
/// 🔑 **Both sides carry one**, which is AC1's *"neither side is the truth"* made structural: an
/// observation can be stale or from a blind source, and a declaration can be outdated. A pane
/// without its meta-line invites the reader to treat that side as fact.
struct MetaLine {
    /// The source — a connector for the observed side, an origin for the declared one.
    source: String,
    /// How long ago, already rendered in the operator's language.
    freshness: String,
}

/// One row of the triage queue.
struct QueueRow {
    /// The stable selector this row is addressed by (`?sel=`).
    id: String,
    /// The row's own link, with the selector ESCAPED and the sort preserved.
    ///
    /// 🔴 Built here rather than in the template. `url_escape` existed from the first draft and was
    /// applied only to the sort toggle, while the queue link wrote `?sel={{ row.id }}` raw — Askama
    /// escapes HTML, never a query string. ⚠️ **Measured NOT reachable in production** (`entity_id`
    /// is a `Uuid::now_v7()` at every call site and no operator path chooses it), so this was a
    /// latent inconsistency and not a live defect — fixed because the function's own doc already
    /// warns that an attribute key is operator-supplied, and the day one is, this is where it lands.
    href: String,
    /// The row's kind, in the operator's language.
    kind: String,
    /// The entity the row is about.
    entity: String,
    /// The field, for a row the engine names one for; empty otherwise.
    field: String,
    /// The declared value, when there is one.
    declared: String,
    /// The observed value, when there is one.
    observed: String,
    /// How many fields this row stands for, WITH its unit — FR16b's *"one line and one gesture,
    /// not N failures"*. ⚠️ A bare number beside an address reads as noise; measured by looking.
    count: String,
    /// Whether this is a cause row, and therefore whether `count` is worth showing.
    counted: bool,
    /// The observed side's freshness, right-aligned as the mock puts it.
    seen: String,
    /// Seconds since the newest in-perimeter observation — the sort key, never displayed.
    age_seconds: i64,
    /// Whether this row is the selected one.
    selected: bool,
}

/// What a control on the action bar IS.
///
/// # One variant today, and that is the decision rather than a limitation
///
/// 🔑 **Guy's arbitration (2026-08-19), taken over a struct carrying an `Option` route**, and taken
/// for a reason the validation established by BUILDING both: with `Planned` alone there is no
/// unconstructed variant, so `clippy -D warnings` is clean today — and **the day story 6.4 adds
/// `Live`, `E0004` forces every `match` on this type to be revisited.** That is a compiler-forced
/// moment of attention at exactly the moment it is worth having, and the struct shape has none.
///
/// ⚠️ **What this type does NOT do, stated because the first draft claimed it did.** It does not make
/// *"a button that looks live and calls nothing"* unrepresentable. The validation measured that
/// under a struct a route pointing nowhere reds nothing and renders as a genuine live link — and
/// that the enum is no better, because `clippy`'s dead-code lint asks only whether a variant was
/// instantiated **with any value**, never whether the value means anything. **This is a labelling
/// and typing DISCIPLINE, not a compiler-enforced guarantee** (story 5.12's narrowing, applied
/// again). The closure — a route typed as a member of a closed set — is registered to story 6.4,
/// where that set stops being empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Gesture {
    /// The product does not have this gesture yet.
    ///
    /// ⚠️ `owner` lives HERE and never on the screen: *"arrives in 6.4"* would turn the label into a
    /// calendar, therefore a promise, which story 5.14b refused. Nothing reads it, and `dead_code`
    /// does not red on it — measured, with the `#[derive(Debug)]` hypothesis tested and refuted.
    Planned {
        /// The story that will build it.
        owner: &'static str,
    },
}

/// One control of the action bar, resolved for rendering.
pub(crate) struct GestureView {
    /// Its label, in the operator's language.
    pub(crate) label: String,
    /// The sentence shown when the gesture is not built — `None` once it is.
    ///
    /// 🔑 **ONE field, and the template branches on it**, so the rendered state cannot disagree with
    /// the nature: there is no second *"is it live"* flag to drift out of step. It is produced by a
    /// `match` on [`Gesture`], which is what makes story 6.4's `Live` a compile error here.
    pub(crate) not_built: Option<String>,
}

/// The mock's action bar: five controls, in its order, none of them live today.
///
/// 🔴 **`primary` is chosen from the row's CAUSE, never from its translated label.** The mock shows
/// *Résoudre* on a conflict and *Merger* elsewhere; branching on `DetailPane::kind` — which is
/// already translated — would reproduce story 6b.3's `role_key: "example.badge"` defect, a real,
/// resolving, wrong value that every shape and resolvability check passes.
fn action_bar(primary_key: &'static str) -> Vec<GestureView> {
    // ⚠️ The primary's owner FOLLOWS the primary: *Merger* is story 6.4's (FR13(a) on the
    // abstention line), *Résoudre* needs FR16's ranked candidates and is Epic 6's. This read "6.4"
    // for both until the code review — invisible, because nothing renders `owner`, which is exactly
    // why it would still have been wrong the day something did.
    let primary_owner = match primary_key {
        "gesture.resolve" => "6",
        _ => "6.4",
    };
    planned_gestures(&[
        (primary_key, primary_owner),
        ("gesture.accept_gap", "7"),
        ("gesture.snooze", "7"),
        ("gesture.attach", "7"),
        ("gesture.exclude", "7"),
    ])
}

/// Resolve a list of `(label key, owner)` pairs into controls that are not built yet.
///
/// 🔑 **The `match` on [`Gesture`] is what makes story 6.4's `Live` a compile error**, and it lives
/// here so both callers inherit it — the triage bar's five controls and the diagnostic's two.
/// ⚠️ **The CALLERS stay separate**, and that is a decision: `action_bar` carries the mock's five
/// triage gestures with triage owners, the diagnostic carries two with different ones, and one
/// builder for both would put two premises in one place where a future edit satisfies neither.
pub(crate) fn planned_gestures(entries: &[(&'static str, &'static str)]) -> Vec<GestureView> {
    entries
        .iter()
        .map(|(label_key, owner)| {
            let gesture = Gesture::Planned { owner };
            GestureView {
                label: rust_i18n::t!(*label_key).to_string(),
                not_built: match gesture {
                    Gesture::Planned { .. } => Some(rust_i18n::t!("gesture.not_built").to_string()),
                },
            }
        })
        .collect()
}

/// The detail pane: the two photos, side by side, each with its own meta-line.
struct DetailPane {
    /// The action bar — the mock's five controls, and what each of them is.
    gestures: Vec<GestureView>,
    /// The selected row's kind.
    kind: String,
    /// The entity.
    entity: String,
    /// The field, when the row names one.
    field: String,
    /// The declared value.
    declared: String,
    /// Where the declared value came from and when it was written.
    declared_meta: MetaLine,
    /// The observed value.
    observed: String,
    /// Which connector reported it and when.
    observed_meta: MetaLine,
}

/// Everything `/triage` renders: the queue, the selection, and the sort's state.
struct TriageView {
    /// The queue, already ordered.
    rows: Vec<QueueRow>,
    /// The selected row's two photos, when a row is selected.
    selected: Option<DetailPane>,
    /// How many rows the queue holds.
    total: usize,
    /// Whether the age sort is on. ⚠️ **Off by default** — AC3, and the ban is not that age is
    /// hidden but that it is never brandished.
    sort_by_age: bool,
    /// The href that toggles the sort, preserving the selection.
    sort_href: String,
}

/// Render an interval as the operator reads it — *"il y a 4 min"*.
///
/// 🔴 **`now` is a PARAMETER and that is the whole point.** The builder reads no clock, so one store
/// renders identically twice; the instant is taken once at the impure edge. ⚠️ Story 5.14b's guard
/// `the_view_builder_has_no_clock_so_one_store_renders_identically` does **not** protect this:
/// story 6b.4's validation measured that it calls `build_view` with EMPTY inputs, so a clock in the
/// populated branch is never reached — and `SystemTime::now()` compiles freely where
/// `chrono::Utc::now()` does not. The guard that covers this function is written in this file's test
/// module and named for what it does.
pub(crate) fn relative_time(
    now: chrono::DateTime<chrono::Utc>,
    then: chrono::DateTime<chrono::Utc>,
) -> String {
    use rust_i18n::t;
    let seconds = (now - then).num_seconds();
    if seconds < 0 {
        // A source dated in the future is not an error to hide behind a negative duration.
        return t!("time.ahead").to_string();
    }
    let minutes = seconds / 60;
    if minutes < 1 {
        return t!("time.just_now").to_string();
    }
    if minutes < 60 {
        return t!("time.minutes", n = minutes).to_string();
    }
    let hours = minutes / 60;
    if hours < 24 {
        return t!("time.hours", n = hours).to_string();
    }
    t!("time.days", n = hours / 24).to_string()
}

/// PURE: shape the real gap into the mock's queue and its two photos.
///
/// # What a queue row IS, and why `Ambigu` is not one
///
/// 🔑 **The row vocabulary is MEASURED, not chosen.** Of the mock's five kinds, three are already
/// typed by the engine — a [`Gap`](opencmdb_core::gap::Gap) is *Écart*,
/// `AbstentionCause::NoObservedValue` is *Absence* and `ConflictingObservations` is *Conflit* —
/// *Nouveau* is an observed address no declared entity claims, and **`Ambigu` is omitted because it
/// has no producer**: it needs FR16's ranked candidates, which `link_candidate` stores and nothing
/// reads. Epic 6's.
///
/// 🔑 **A cause row is ONE line carrying its count, never N rows.** That is FR16b's rule verbatim —
/// *"each cause is one line and one gesture, not N failures"* — and not a simplification: the engine
/// returns `abstentions` as a count per cause and names no field, so N rows would mean re-deriving
/// in the adapter a rule the engine owns.
///
/// ⚠️ `OutOfPerimeter` is NOT a row. `reconcile` is written for ONE perimeter, so every pass counts
/// every other entity's observations as out of perimeter — noise of the loop, not a fact about the
/// entity. Surfacing it would put one row per entity per other entity on the operator's screen.
#[allow(clippy::too_many_arguments)]
fn build_triage(
    declared: Vec<(String, String, String)>,
    provenance: Vec<crate::repo::DeclaredProvenance>,
    observations: Vec<crate::repo::ObservedBatch>,
    now: chrono::DateTime<chrono::Utc>,
    selected: Option<&str>,
    sort_by_age: bool,
) -> TriageView {
    use rust_i18n::t;

    // Group declared attributes by entity, preserving first-seen order (as `build_view` does).
    let mut entities: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for (entity_id, key, value) in declared {
        match entities.iter_mut().find(|(id, _)| *id == entity_id) {
            Some((_, attrs)) => attrs.push((key, value)),
            None => entities.push((entity_id, vec![(key, value)])),
        }
    }
    let ipv4_of = |attrs: &[(String, String)]| -> Option<String> {
        attrs
            .iter()
            .find(|(k, _)| k == "ipv4")
            .map(|(_, v)| v.clone())
    };

    // The declared side's provenance, keyed for lookup. Display only — never the comparison.
    let provenance_of = |entity: &str, field: &str| -> Option<&crate::repo::DeclaredProvenance> {
        provenance
            .iter()
            .find(|p| p.entity_id == entity && p.attr_key == field)
    };

    let obs: Vec<Observation> = observations
        .iter()
        .map(|batch| observation_from_facts(batch.facts.clone()))
        .collect();

    let mut rows: Vec<QueueRow> = Vec::new();
    let mut panes: Vec<(String, DetailPane)> = Vec::new();
    let mut claimed: Vec<String> = Vec::new();

    for (entity_id, attrs) in &entities {
        let Some(ipv4) = ipv4_of(attrs) else { continue };
        claimed.push(ipv4.clone());

        // The newest in-perimeter observation is this entity's freshness and its source.
        let newest = observations
            .iter()
            .filter(|b| in_perimeter(&b.facts, &ipv4))
            .max_by_key(|b| b.observed_at);
        let (observed_source, observed_seen, age_seconds) = match newest {
            Some(b) => (
                source_label(&b.connector_id),
                relative_time(now, b.observed_at),
                (now - b.observed_at).num_seconds().max(0),
            ),
            None => (
                t!("meta.no_source").to_string(),
                t!("meta.never_seen").to_string(),
                i64::MAX,
            ),
        };

        // 🔴 A CAUSE row is about the whole entity, so its declared meta-line is the entity's most
        // recent declared write — never `None`. Passing `None` made the Absence pane say *"2 champs
        // déclarés"* over a meta-line reading *"Rien de déclaré"*: two contradictory sentences about
        // the same side, in the same pane. **Found by looking at the screen, by nothing else.**
        let entity_provenance = provenance
            .iter()
            .filter(|p| p.entity_id == *entity_id)
            .max_by_key(|p| p.updated_at);

        let result = reconcile(("ipv4", &ipv4), attrs, &obs);

        for gap in &result.gaps {
            let id = format!("ecart:{entity_id}:{}", gap.field);
            let declared_meta = declared_meta_line(provenance_of(entity_id, &gap.field), now);
            rows.push(QueueRow {
                href: row_href(&id, sort_by_age),
                id: id.clone(),
                kind: t!("triage.kind.ecart").to_string(),
                entity: ipv4.clone(),
                field: gap.field.clone(),
                declared: gap.declared.clone(),
                observed: gap.observed.clone(),
                count: String::new(),
                counted: false,
                seen: observed_seen.clone(),
                age_seconds,
                selected: false,
            });
            panes.push((
                id,
                DetailPane {
                    gestures: action_bar("gesture.document"),
                    kind: t!("triage.kind.ecart").to_string(),
                    entity: ipv4.clone(),
                    field: gap.field.clone(),
                    declared: gap.declared.clone(),
                    declared_meta,
                    observed: gap.observed.clone(),
                    observed_meta: MetaLine {
                        source: observed_source.clone(),
                        freshness: observed_seen.clone(),
                    },
                },
            ));
        }

        for (cause, count) in &result.abstentions {
            let (slug, label) = match cause {
                AbstentionCause::NoObservedValue => ("absence", t!("triage.kind.absence")),
                AbstentionCause::ConflictingObservations => ("conflit", t!("triage.kind.conflit")),
                // Noise of the loop, never a fact about this entity — see this function's doc.
                AbstentionCause::OutOfPerimeter => continue,
            };
            let id = format!("{slug}:{entity_id}");
            rows.push(QueueRow {
                href: row_href(&id, sort_by_age),
                id: id.clone(),
                kind: label.to_string(),
                entity: ipv4.clone(),
                field: String::new(),
                declared: String::new(),
                observed: String::new(),
                count: t!("triage.n_fields", n = *count).to_string(),
                counted: true,
                seen: observed_seen.clone(),
                age_seconds,
                selected: false,
            });
            panes.push((
                id,
                DetailPane {
                    // 🔴 From the CAUSE, never from the translated label: the mock shows *Résoudre*
                    // on a conflict and *Merger* elsewhere, and branching on the rendered string is
                    // story 6b.3's wrong-namespace defect waiting.
                    gestures: action_bar(match cause {
                        AbstentionCause::ConflictingObservations => "gesture.resolve",
                        _ => "gesture.document",
                    }),
                    kind: label.to_string(),
                    entity: ipv4.clone(),
                    field: String::new(),
                    declared: t!("triage.cause.declared_side", n = *count).to_string(),
                    declared_meta: declared_meta_line(entity_provenance, now),
                    observed: match cause {
                        AbstentionCause::NoObservedValue => t!("triage.cause.nothing_observed"),
                        _ => t!("triage.cause.sources_disagree"),
                    }
                    .to_string(),
                    observed_meta: MetaLine {
                        source: observed_source.clone(),
                        freshness: observed_seen.clone(),
                    },
                },
            ));
        }
    }

    // `Nouveau`: an observed address no declared entity claims.
    let mut seen_new: Vec<String> = Vec::new();
    for batch in &observations {
        for (field, value) in batch.facts.iter().filter_map(display_fact) {
            if field != "ipv4" || claimed.contains(&value) || seen_new.contains(&value) {
                continue;
            }
            seen_new.push(value.clone());
            let id = format!("nouveau:{value}");
            let seen = relative_time(now, batch.observed_at);
            rows.push(QueueRow {
                href: row_href(&id, sort_by_age),
                id: id.clone(),
                kind: t!("triage.kind.nouveau").to_string(),
                entity: value.clone(),
                field: "ipv4".to_string(),
                declared: String::new(),
                observed: value.clone(),
                count: String::new(),
                counted: false,
                seen: seen.clone(),
                age_seconds: (now - batch.observed_at).num_seconds().max(0),
                selected: false,
            });
            panes.push((
                id,
                DetailPane {
                    gestures: action_bar("gesture.document"),
                    kind: t!("triage.kind.nouveau").to_string(),
                    entity: value.clone(),
                    field: "ipv4".to_string(),
                    declared: t!("triage.cause.nothing_declared").to_string(),
                    declared_meta: declared_meta_line(None, now),
                    observed: value,
                    observed_meta: MetaLine {
                        source: source_label(&batch.connector_id),
                        freshness: seen,
                    },
                },
            ));
        }
    }

    // AC3: age sorting is available and OFF by default — oldest first when on.
    if sort_by_age {
        rows.sort_by_key(|r| std::cmp::Reverse(r.age_seconds));
    }

    let chosen = selected
        .filter(|id| rows.iter().any(|r| r.id == *id))
        .map(str::to_string)
        .or_else(|| rows.first().map(|r| r.id.clone()));
    for row in &mut rows {
        row.selected = Some(&row.id) == chosen.as_ref();
    }
    // 🔑 The toggle PRESERVES the selection, so sorting never silently moves the operator's row.
    let sort_href = match (&chosen, sort_by_age) {
        (Some(id), true) => format!("/triage?sel={}", url_escape(id)),
        (None, true) => "/triage".to_string(),
        (Some(id), false) => format!("/triage?sort=age&sel={}", url_escape(id)),
        (None, false) => "/triage?sort=age".to_string(),
    };
    let selected = chosen.and_then(|id| {
        panes
            .into_iter()
            .find(|(pane_id, _)| *pane_id == id)
            .map(|(_, pane)| pane)
    });

    TriageView {
        total: rows.len(),
        rows,
        selected,
        sort_by_age,
        sort_href,
    }
}

/// The observed side's source, as an operator can read it.
///
/// 🔴 **The product has NO connector registry** — no table, no name, nothing but the UUID
/// `arp_ping.rs` mints for itself. The mock shows *"UniFi"* because its fixture invented one.
/// Rendering the whole UUID is honest and useless: measured by looking, the meta-line read
/// *"cccccccc-0000-0000-0000-00000000unif · il y a 4 min"*, which tells the operator nothing and
/// pushes the freshness off the line. So the id is SHORTENED and labelled for what it is.
///
/// ⚠️ **This is a stated limit, not a design**: a name per source is what the mock shows and what
/// the operator needs, and it belongs with the screen that owns sources — registered against story
/// 6b.8. Until then, showing a short id is the true sentence.
fn source_label(connector_id: &str) -> String {
    let short: String = connector_id.chars().take(8).collect();
    rust_i18n::t!("meta.source_id", id = short).to_string()
}

/// The declared side's meta-line: its origin and when it was written, or an honest absence.
fn declared_meta_line(
    provenance: Option<&crate::repo::DeclaredProvenance>,
    now: chrono::DateTime<chrono::Utc>,
) -> MetaLine {
    use rust_i18n::t;
    match provenance {
        Some(p) => MetaLine {
            source: t!(origin_key(&p.origin)).to_string(),
            freshness: relative_time(now, p.updated_at),
        },
        None => MetaLine {
            source: t!("meta.nothing_declared").to_string(),
            freshness: String::new(),
        },
    }
}

/// A queue row's link: its selector, escaped, with the sort preserved.
fn row_href(id: &str, sort_by_age: bool) -> String {
    match sort_by_age {
        true => format!("/triage?sel={}&sort=age", url_escape(id)),
        false => format!("/triage?sel={}", url_escape(id)),
    }
}

/// Percent-escape a row selector for a query string.
///
/// ⚠️ Deliberately NARROW: a selector is built by this module from an entity id, a slug and a field
/// name, so the set of characters that can appear is small — but *"the set is small"* is a property
/// of today's inputs, not of the type, and a hostname or an attribute key is operator-supplied. The
/// escape is therefore over a KEEP-list, never a ban-list: anything not plainly safe is escaped.
fn url_escape(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b':' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// The i18n key for a declared value's origin. An unfamiliar token is LABELLED, never a 500 —
/// story 5.14b's arbitration 11: a display may not be the place a write starts failing.
fn origin_key(origin: &str) -> &'static str {
    match origin {
        "manual" => "origin.manual",
        "adopted" => "origin.adopted",
        "imported" => "origin.imported",
        _ => "origin.unknown",
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
    // 🔑 The comparison gets FACTS ONLY. `ObservedBatch` carries the source and the instant for the
    // triage screen's meta-lines; `build_view` is handed neither, and the declared side's
    // provenance is not loaded on this path at all — see `load_declared_provenance_for_display`.
    let facts: Vec<Vec<Fact>> = observations.into_iter().map(|b| b.facts).collect();
    Ok((
        build_view(declared, facts, preferred),
        build_identity_view(reach),
    ))
}

/// The wall clock, read at the impure edge and nowhere else.
///
/// 🔴 **`chrono::Utc::now()` does not exist here and that is deliberate**: `chrono` is declared
/// `default-features = false`, so its `clock` feature is off workspace-wide and the call is a
/// compile error (measured: `E0599`). ⚠️ **But the flag stops `chrono`, not `std`** — this function
/// reads `SystemTime`, which compiles freely, and so would the same call inside a pure builder.
/// *The feature flag is a guard against one spelling, never against reading the clock.* That is why
/// [`build_triage`] takes its instant as a parameter and has a test of its own.
pub(crate) fn now_utc() -> chrono::DateTime<chrono::Utc> {
    let since_epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    chrono::DateTime::from_timestamp(
        i64::try_from(since_epoch.as_secs()).unwrap_or(i64::MAX),
        since_epoch.subsec_nanos(),
    )
    .unwrap_or_else(|| {
        chrono::DateTime::from_timestamp(0, 0).expect("the epoch is a valid instant")
    })
}

/// Load what `/triage` needs and build it. **This is the only impure edge that reads the clock.**
///
/// 🔴 The instant is taken HERE, once, and passed into [`build_triage`] as a parameter — so the
/// builder stays pure and one store renders identically twice.
async fn triage_view(
    pool: &MySqlPool,
    selected: Option<&str>,
    sort_by_age: bool,
) -> Result<(TriageView, IdentityView), Response> {
    let declared = load_declared_attributes(pool).await.map_err(server_error)?;
    let provenance = crate::repo::load_declared_provenance_for_display(pool)
        .await
        .map_err(server_error)?;
    let observations = load_observation_facts(pool).await.map_err(server_error)?;
    let reach = count_engine_reach(pool).await.map_err(server_error)?;
    Ok((
        build_triage(
            declared,
            provenance,
            observations,
            now_utc(),
            selected,
            sort_by_age,
        ),
        build_identity_view(reach),
    ))
}

fn server_error(error: sqlx::Error) -> Response {
    let repo_error = classify(error);
    tracing::error!(?repo_error, "loading the page's state failed");
    (StatusCode::INTERNAL_SERVER_ERROR, store_unreachable_body()).into_response()
}

/// What the operator reads when the store did not answer.
///
/// 🔴 **It was the bare English literal `"internal error"` until story 6b.10**, served at
/// `/triage`, `/dashboard` and `/sources` — the ten screens' own addresses — on a French
/// deployment. Guy's arbitration 2(a′): *an operator whose store is down must not be the one
/// person who reads English on a French deployment.* It is the one path where the interface
/// language silently stopped being the interface language, and it is reached exactly when the
/// operator is least able to guess what happened.
///
/// 🔑 **It says what is NOT lost.** The UX spec's fourth microcopy rule is *error = cause + next
/// step, never blame the user*; the fifth is *empty ≠ failure — calm, never alarming*. A store
/// that did not answer has destroyed nothing: this product never overwrites an observation
/// (NFR5) and never deletes a declared record, so the honest sentence is that the page could not
/// be built, not that something is wrong with the data.
///
/// ⚠️ `t!` is a lookup in an embedded map — it opens no file, touches no store and cannot fail —
/// so it is safe on this path. The **render-error** fallback in [`render_shell`] stays a plain
/// literal by contrast, for the opposite reason: a page whose renderer has already failed is not
/// the place to depend on the renderer.
fn store_unreachable_body() -> String {
    rust_i18n::t!("error.store_unreachable").to_string()
}

/// What the operator reads when a template failed to render but the data was fine.
///
/// A separate sentence from [`store_unreachable_body`] on purpose: the two failures have
/// different causes and different next steps, and collapsing them into one *"internal error"* is
/// what made the old body useless. See that function for the arbitration and the microcopy rules.
pub(crate) fn render_error_body() -> String {
    rust_i18n::t!("error.render_failed").to_string()
}

/// The example-data marker, rendered ONCE for a wholly-example screen.
///
/// # Why the screen and not each section, on an `Example` screen
///
/// 🔴 **Found by looking at the record in a browser.** Its four example sections each included the
/// marker, so the page carried **four identical banners** saying *"Ce contenu est une démonstration
/// — il ne provient pas de votre réseau."* one under the other. That is story 6b.4b's finding
/// exactly — a per-control copy turning a compact row into a stack repeating itself — and no test
/// could see it, because every guard asks whether the marker is PRESENT.
///
/// 🔑 **The rule that resolves it is story 6b.3's own, read precisely**: the marker goes on the
/// smallest unit that is ENTIRELY example. On a [`Mixed`](crate::screens::Nature::Mixed) screen
/// that unit is the section, and the dashboard still marks each of its example sections. On an
/// [`Example`](crate::screens::Nature::Example) screen every section is example, so the smallest
/// such unit **is the screen** — and marking it four times says nothing the first one did not.
///
/// ⚠️ **Emitting it from the dispatch is stronger than a template include, and the promise is
/// NARROWED to what is true** (story 5.12's precedent). For a screen served by
/// [`crate::screens::router`]'s generic loop the marker comes from the same `match` arm as the
/// body, so it cannot be forgotten. 🔴 **It is NOT a universal guarantee, and the counterexample
/// ships in the same commit**: `Screen::Device` is `Example` and is served by `device_record`,
/// outside that `match` — the first version of that handler rendered four example sections with no
/// marker at all, and only `the_marker_partition_follows_every_screens_declared_nature` said so.
/// *A route off `Screen` is off the dispatch too.* The blind review layer found this sentence
/// asserting the unqualified version.
///
/// # Panics
///
/// Never in practice: the template and this struct are compiled together, so a render failure
/// means the binary was built from an inconsistent tree.
pub(crate) fn example_marker() -> String {
    ExampleMarker { s: strings() }
        .render()
        .expect("the marker template and its struct are compiled together")
}

/// The example marker on its own — one partial, one key pair, as story 6b.3 requires.
#[derive(Template)]
#[template(path = "_example_marker.html")]
struct ExampleMarker {
    s: Strings,
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
    /// What `/diagnostic` reports about the product itself (story 6b.9).
    ///
    /// 🔑 **Facts, carried as data.** The log descriptor is what `init_tracing` INSTALLED and the
    /// security posture is derived by probing `auth::is_public` — neither is re-read at the point
    /// of use, which is story 6b.2's M12 applied rather than merely cited.
    pub(crate) diagnostic: crate::diagnostic::DiagnosticFacts,
}

/// `/triage` on its own state, so the perimeter arrives as a parameter rather than being read.
///
/// # Returns
///
/// A router to be merged BEFORE `.layer(auth_deny)`, like every other route (story 6.1 §2): the
/// screen is not public, and merging after the layer would bypass the middleware entirely.
pub(crate) fn triage_router(
    pool: MySqlPool,
    perimeter: Option<String>,
    diagnostic: crate::diagnostic::DiagnosticFacts,
) -> Router {
    Router::new()
        .route("/triage", get(triage))
        .route("/dashboard", get(dashboard))
        // 🔴 Registered BEFORE `Screen::Diagnostic`'s nature changed, for the reason spelled out
        // just below: re-measured on this very screen at story 6b.9, the right order reds nothing
        // in either condition, nature-first reds 0 locally and 1 in CI, and route-first-only reds
        // **19**, all `Overlapping method route. Handler for GET /diagnostic already exists`.
        .route("/diagnostic", get(crate::diagnostic::diagnostic))
        // 🔴 **REGISTER THE ROUTE BEFORE CHANGING THE NATURE, and the order is a MEASUREMENT.**
        // Story 6b.8's validation built both mistakes: a nature changed with the route forgotten
        // reds **nothing** locally — 668/668 green, no warning — and exactly one test in CI, because
        // the route-table loop `continue`s on `Mixed` when no database is reachable; the address
        // simply 404s, and `every_screen_is_refused_without_a_credential` cannot help because it
        // asserts 401, which `auth_deny` returns above routing. The mirror mistake — route added,
        // nature forgotten — reds **18 tests**, all `Overlapping method route`.
        // *The wrong order is silent; the right order fails loudly.*
        .route("/sources", get(sources))
        .with_state(TriageState {
            pool,
            perimeter,
            diagnostic,
        })
}

/// One example figure on the dashboard, with its copy already resolved.
struct StatCardView {
    /// What it counts, in the operator's language.
    label: String,
    /// The figure itself — a STRING, because it is decoration and never arithmetic.
    value: &'static str,
    /// The shape a sparkline would draw, as a bare list of heights.
    spark: Vec<u8>,
}

/// Everything `/dashboard` renders: the product's real reach, and the example surfaces beside it.
///
/// 🔴 **Two populations, never summed** (arbitration 10) — and the guard for that lives at the
/// COMPOSITION and not on either builder, because story 5.14b measured its own guard GREEN when it
/// asserted a property two pure builders cannot violate: neither sees the other's numbers, so
/// neither can add them. This struct is where they meet, so this is where a sum could be written.
struct DashboardView {
    /// The real reach, exactly as story 5.14b shipped it and `/triage` renders it.
    identity: IdentityView,
    /// How long ago the product last observed anything — a `MAX(observed_at)`, in the BODY only.
    last_observed: Option<String>,
    /// The example figures. ⚠️ Example, and each carries the marker on its own section.
    cards: Vec<StatCardView>,
    /// True when something HAS been observed and the identity pass has not placed any of it yet.
    ///
    /// 🔴 **This story is what created the state that needs saying.** It co-located two populations
    /// for the first time — the engine's reach and the last observation — and they can legitimately
    /// disagree: an observation ingested but not yet resolved leaves `count_engine_reach` empty
    /// while `MAX(observed_at)` is recent. The page then read *"Nothing observed yet — run a scan"*
    /// directly above *"Last observed 8 h ago"*, in one div. Found at the code review by seeding the
    /// two independently; **every test fed them from one fixture, so none could see it.**
    pending_resolution: bool,
}

/// The dashboard's body: the real reach section beside labelled example sections (story 6b.5).
#[derive(Template)]
#[template(path = "_dashboard.html")]
struct DashboardBody {
    /// 🔴 **ONE source for the identity counts, and that is not tidying.** This struct carried its
    /// own `identity` field beside `view.identity` until the mutation pass: the handler filled it
    /// from the view and the TEST HELPER filled it from the un-composed original, so **the guard
    /// rendered a shape production does not use** and mutation M1 — a sum planted at the
    /// composition — left it green. *Two fields holding one fact will be filled from two places,
    /// and the test's place is the one nobody ships.* The template now reads `view.identity`.
    view: DashboardView,
    s: Strings,
}

/// PURE: the example half. It reads nothing and depends on nothing.
fn example_cards() -> Vec<StatCardView> {
    use rust_i18n::t;
    vec![
        StatCardView {
            label: t!("dash.card.devices").to_string(),
            value: "37",
            spark: vec![3, 5, 4, 6, 6, 7, 9],
        },
        StatCardView {
            label: t!("dash.card.gaps").to_string(),
            value: "4",
            spark: vec![7, 6, 6, 4, 5, 3, 4],
        },
        StatCardView {
            label: t!("dash.card.sources").to_string(),
            value: "2",
            spark: vec![1, 1, 2, 2, 2, 2, 2],
        },
    ]
}

/// PURE: assemble the real reach and the example surfaces into one view.
///
/// 🔴 **`now` is a PARAMETER.** The builder reads no clock, so one store renders identically twice;
/// the instant is taken once at the impure edge. Story 5.14b's clock guard does **not** cover a
/// populated builder — story 6b.4's review measured that — so this one has its own.
fn build_dashboard(
    identity: IdentityView,
    last_observed_at: Option<chrono::DateTime<chrono::Utc>>,
    now: chrono::DateTime<chrono::Utc>,
) -> DashboardView {
    DashboardView {
        pending_resolution: !identity.has_any && last_observed_at.is_some(),
        identity,
        last_observed: last_observed_at.map(|then| relative_time(now, then)),
        cards: example_cards(),
    }
}

/// `GET /dashboard` — the real reach section beside labelled example sections.
///
/// ⚠️ **On the pool-bearing router, by Guy's arbitration of 2026-08-19.** Its real half reads the
/// store, so it cannot live on `screens::router`'s `Router<()>`; the compile-time refusal of
/// `State<MySqlPool>` therefore does not hold for this one screen, and holds for the eight that
/// remain. See [`crate::screens::Nature::Mixed`] for the cost and the alternative that was refused.
/// How long a screen that CANNOT render without the store waits for it.
///
/// 🔴 **Story 6b.10 gave the store-down page a calm French sentence and nothing measured WHEN it
/// arrives.** Its code review did: with a real MariaDB paused mid-session, `/triage` answered
/// **500 in 30.002731 s** and `/dashboard` in **30.002711 s** — sqlx's default acquire timeout —
/// while `/diagnostic` answered 200 in 2.003674 s on its own budget. *A calm sentence that takes
/// half a minute to arrive is read as a fault of the product, not of the database.*
///
/// ⚠️ **Five seconds, not the diagnostic's two, and the difference is the point.** `/diagnostic`
/// DEGRADES — it renders without the store, so it can afford to be impatient. These two cannot:
/// the budget decides only how fast the honest refusal arrives, so it leaves a genuinely slow
/// first connection room to succeed. Both are far under thirty.
///
/// 🔑 Guy's arbitration 4 of 2026-08-22, option (B), taken over setting `acquire_timeout` on the
/// production pool: that governs the wait for a FREE connection, so under load it turns a
/// legitimate wait into an error, and it would reach the scan pass as well.
const PAGE_STORE_BUDGET: std::time::Duration = std::time::Duration::from_secs(5);

/// A store read bounded by `budget`, or the response the operator gets instead.
///
/// 🔴 **`budget` is a PARAMETER so a test can reach the elapsed arm** — story 6b.9 shipped a
/// guard whose test pool refused faster than the budget, leaving the timeout branch dead code
/// under test while its own comment described why. *A guard placed where the defect cannot occur
/// reads as coverage and is none.*
async fn store_within<T>(
    budget: std::time::Duration,
    read: impl Future<Output = Result<T, Response>>,
) -> Result<T, Response> {
    match tokio::time::timeout(budget, read).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(response)) => Err(response),
        Err(_elapsed) => {
            tracing::error!(
                budget_ms = budget.as_millis(),
                "the store did not answer within this screen's budget — refusing rather than \
                 holding the browser"
            );
            Err(server_error(sqlx::Error::PoolTimedOut))
        }
    }
}

pub async fn dashboard(State(state): State<TriageState>) -> Response {
    let perimeter = state.perimeter.clone();
    let reach = match store_within(PAGE_STORE_BUDGET, async {
        count_engine_reach(&state.pool).await.map_err(server_error)
    })
    .await
    {
        Ok(rows) => rows,
        Err(response) => return response,
    };
    let last = match store_within(PAGE_STORE_BUDGET, async {
        crate::repo::last_observed_at(&state.pool)
            .await
            .map_err(server_error)
    })
    .await
    {
        Ok(instant) => instant,
        Err(response) => return response,
    };
    let view = build_dashboard(build_identity_view(reach), last, now_utc());
    let body = DashboardBody { view, s: strings() };
    match body.render() {
        Ok(body) => Html(render_shell(
            Shell::new(crate::screens::Screen::Dashboard, perimeter),
            body,
        ))
        .into_response(),
        Err(error) => {
            tracing::error!(%error, "rendering the dashboard");
            (StatusCode::INTERNAL_SERVER_ERROR, render_error_body()).into_response()
        }
    }
}

/// One capability line: a fact kind the source does or does not observe, with its sentence.
struct KindLine {
    /// The kind's name, in the operator's language.
    label: String,
    /// What its presence or absence MEANS to the operator — the unlock framing, never the fault one.
    meaning: String,
}

/// Everything `/sources` renders: the product's real capability boundary, and its real freshness.
struct SourceView {
    /// The source's name, resolved — a TYPE name (see [`crate::arp_ping::SOURCE_NAME_KEY`]).
    name: String,
    /// Whether the product REFUSED this perimeter — measured with the connector's own parser.
    ///
    /// 🔴 A refused perimeter shown as an in-force one is the sharpest thing the code review found
    /// on this screen: the product already knows the configuration is bad, and said so in a log
    /// nobody reads. See [`build_sources`].
    refused: bool,
    /// The perimeter it was configured with.
    ///
    /// 🔑 **Not an `Option`, and the code review is why.** `build_sources` returns `None` outright
    /// when no perimeter is configured, so inside a `SourceView` this could never be absent — and
    /// the template carried a `when None` arm that could never execute. *A branch placed where the
    /// case cannot occur reads as handling and is none.*
    perimeter: String,
    /// What it is built to observe.
    observes: Vec<KindLine>,
    /// What it is built NOT to observe — the section AC1 requires to be real.
    cannot_see: Vec<KindLine>,
    /// How long ago anything was observed, or `None`.
    ///
    /// 🔴 **`None` is FOUR different states of the world and the screen says so.** Story 6b.8's
    /// validation booted the real binary four times against four fresh databases: never scanned,
    /// scanned-and-nobody-answered, an INVALID perimeter the product refused, and a blank one — all
    /// four leave `MAX(observed_at)` NULL. FR8's own distinction fails at boot level, so the copy
    /// states the ambiguity instead of picking one reading.
    last_observed: Option<String>,
}

/// The sources screen's body.
#[derive(Template)]
#[template(path = "_sources.html")]
struct SourcesBody {
    /// `None` when no source is configured at all — the case the story's first draft assumed away.
    source: Option<SourceView>,
    s: SourceStrings,
}

/// The copy `/sources` needs, resolved once.
struct SourceStrings {
    title: String,
    lede: String,
    observes_title: String,
    cannot_see_title: String,
    unlock: String,
    freshness_title: String,
    never: String,
    ambiguity: String,
    incident_axis: String,
    perimeter_label: String,
    no_source: String,
    /// What the screen says when the product REFUSED the configured perimeter.
    refused: String,
}

/// Resolve `/sources`' copy.
fn source_strings() -> SourceStrings {
    SourceStrings {
        title: rust_i18n::t!("sources.title").to_string(),
        lede: rust_i18n::t!("sources.lede").to_string(),
        observes_title: rust_i18n::t!("sources.observes").to_string(),
        cannot_see_title: rust_i18n::t!("sources.cannot_see").to_string(),
        unlock: rust_i18n::t!("sources.unlock").to_string(),
        freshness_title: rust_i18n::t!("sources.freshness").to_string(),
        never: rust_i18n::t!("sources.never").to_string(),
        ambiguity: rust_i18n::t!("sources.ambiguity").to_string(),
        incident_axis: rust_i18n::t!("sources.incident_axis").to_string(),
        perimeter_label: rust_i18n::t!("sources.perimeter").to_string(),
        no_source: rust_i18n::t!("sources.no_source").to_string(),
        refused: rust_i18n::t!("sources.refused").to_string(),
    }
}

/// The i18n keys of one [`FactKind`]'s name and of what it means to the operator.
///
/// 🔑 **A `match` on a `FactKind`, and the `_` arm is FORCED by `#[non_exhaustive]`** — the compiler
/// cannot carry exhaustiveness across the crate boundary, so a wildcard is mandatory and is then
/// permanently silent.
///
/// 🔴 **The fallback returns a GENERIC pair, and the doc said *"the kind's `Debug` name"* until the
/// code review caught it two lines above the code that refutes it.** Every unmapped kind would render
/// *identically* — *"Unrecognised kind"* — with nothing on the page telling an operator or a
/// log-reading developer WHICH one appeared.
///
/// ⚠️ **And `FactKind::ALL`'s cross-crate guard does not protect THIS map.** It pins `ALL` against the
/// enum's declaration; it says nothing about whether each member has a key pair here. So an eighth
/// kind correctly added to `ALL` would satisfy that guard and still render *"Genre non reconnu"* on
/// `/sources` — *a guard placed where the defect cannot occur*, one field over. Closed by
/// [`crate::page::tests::every_fact_kind_has_its_own_sentence`], which reds on exactly that.
fn kind_keys(kind: FactKind) -> (&'static str, &'static str) {
    match kind {
        FactKind::Mac => ("kind.mac", "kind.mac.meaning"),
        FactKind::IpV4 => ("kind.ipv4", "kind.ipv4.meaning"),
        FactKind::Hostname => ("kind.hostname", "kind.hostname.meaning"),
        FactKind::DhcpLease => ("kind.dhcp_lease", "kind.dhcp_lease.meaning"),
        FactKind::Uplink => ("kind.uplink", "kind.uplink.meaning"),
        FactKind::OuiVendor => ("kind.oui_vendor", "kind.oui_vendor.meaning"),
        FactKind::Rtt => ("kind.rtt", "kind.rtt.meaning"),
        _ => ("kind.unknown", "kind.unknown.meaning"),
    }
}

/// Build one capability line.
fn kind_line(kind: FactKind) -> KindLine {
    let (label, meaning) = kind_keys(kind);
    KindLine {
        label: rust_i18n::t!(label).to_string(),
        meaning: rust_i18n::t!(meaning).to_string(),
    }
}

/// Build the sources view. Pure: the caller supplies the instant and the perimeter.
///
/// ⚠️ **No clock here** — `now` is a parameter, on the precedent of every view builder in this file.
fn build_sources(
    perimeter: Option<String>,
    last: Option<chrono::DateTime<chrono::Utc>>,
    now: chrono::DateTime<chrono::Utc>,
) -> Option<SourceView> {
    // 🔴 **NO PERIMETER, NO SOURCE.** The story's first draft assumed one configured source
    // throughout; the validation measured that with `OPENCMDB_SCAN_CIDR` unset there are ZERO, and
    // the screen had no copy for it. Listing a source the product was never asked to build would be
    // the same fabrication the liveness arbitration refuses.
    // 🔑 The `?` on the field below is the ONLY place this is decided — it read `perimeter.as_ref()?`
    // here as well, and two refusals of one fact drift.
    let (observes, cannot_see) = crate::arp_ping::observes_and_cannot_see();
    // 🔴 **A PERIMETER THE PRODUCT REFUSED IS NOT A PERIMETER, and the screen said otherwise.**
    // Measured at the code review by booting with `OPENCMDB_SCAN_CIDR=nonsense`: the log carried
    // `ERROR invalid OPENCMDB_SCAN_CIDR — skipping scan`, and this screen rendered a full source
    // card reading *"Périmètre nonsense"* with the generic four-state sentence under it — **the
    // rejected string PRESENTED AS AN IN-FORCE VALUE**. That is worse than the ambiguity the story
    // registered: it is not *we cannot tell which of four*, it is *we are showing you a
    // configuration we already refused, as though it were live*.
    //
    // 🔑 `subnet_hosts` is the SAME parser the connector uses, so the screen and the scan agree by
    // construction rather than by two readings of one rule. ⚠️ `AppConfig::from_env` still does not
    // validate the CIDR — the refusal happens in a detached thread whose error nobody reads — and
    // moving it to boot time is registered rather than done here.
    let refused = crate::arp_ping::subnet_hosts(perimeter.as_deref().unwrap_or_default()).is_err();
    Some(SourceView {
        name: rust_i18n::t!(crate::arp_ping::SOURCE_NAME_KEY).to_string(),
        refused,
        perimeter: perimeter?,
        observes: observes.into_iter().map(kind_line).collect(),
        cannot_see: cannot_see.into_iter().map(kind_line).collect(),
        last_observed: last.map(|instant| relative_time(now, instant)),
    })
}

/// `GET /sources` — what the product's sources can and cannot see.
///
/// 🔴 **The capability section is REAL** (AC1): it is `FactKind::ALL` minus the connector's own
/// declaration, five kinds derived at runtime with no database and no invention. ⚠️ It is **what the
/// source is BUILT to observe**, not what it observed — FR7's static half only, and the copy says so.
///
/// 🔴 **The liveness axis carries NO COLOUR** (AC2, Guy's arbitration of 2026-08-20). The product can
/// establish *an observation arrived at T*; it cannot establish *this source is blind*, which is a
/// verdict nothing computes — deriving it from silence would read a clock and INVENT an incident,
/// which FR8 exists to forbid. ⚠️ **The spec prescribes a colour for BOTH liveness values**, so this
/// is a DIVERGENCE and not conformance: AC2 ships met on its *never two amber pills* half and **not
/// met** on its *blind gets a colour* half. The incident axis arrives with **Epic 13**, which
/// `score.rs`, `connector/mod.rs` and `epics.md` all already name.
pub async fn sources(State(state): State<TriageState>) -> Response {
    let last = match crate::repo::last_observed_at(&state.pool).await {
        Ok(instant) => instant,
        Err(error) => return server_error(error),
    };
    let body = SourcesBody {
        source: build_sources(state.perimeter.clone(), last, now_utc()),
        s: source_strings(),
    };
    match body.render() {
        Ok(body) => Html(render_shell(
            Shell::new(crate::screens::Screen::Sources, state.perimeter),
            body,
        ))
        .into_response(),
        Err(error) => {
            tracing::error!(%error, "rendering the sources screen");
            (StatusCode::INTERNAL_SERVER_ERROR, render_error_body()).into_response()
        }
    }
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
/// pool-free screens sit on a `Router<()>`. Epic constraint 1 is about demonstrations, and
/// it is enforced exactly where it applies.
///
/// Story 6b.4 replaces this body with the mock's two-pane triage; the frame it renders into stays.
pub async fn triage(
    State(state): State<TriageState>,
    axum::extract::Query(query): axum::extract::Query<TriageQuery>,
) -> Response {
    let perimeter = state.perimeter.clone();
    // ⚠️ AC3: age sorting is OFF unless the operator asked for it, by name. Any other value of
    // `sort` is off — a typo must not silently brandish age.
    let sort_by_age = query.sort.as_deref() == Some("age");
    match store_within(
        PAGE_STORE_BUDGET,
        triage_view(&state.pool, query.sel.as_deref(), sort_by_age),
    )
    .await
    {
        Ok((triage, identity)) => {
            let body = TriageBody {
                triage,
                identity,
                s: strings(),
            };
            match body.render() {
                Ok(body) => Html(render_shell(
                    Shell::new(crate::screens::Screen::Triage, perimeter),
                    body,
                ))
                .into_response(),
                Err(error) => {
                    tracing::error!(%error, "rendering the triage screen");
                    (StatusCode::INTERNAL_SERVER_ERROR, render_error_body()).into_response()
                }
            }
        }
        Err(response) => response,
    }
}

/// What the operator's URL says about the triage screen: which row, and whether age sorts it.
///
/// 🔑 Both are OPTIONAL and both default to the quiet answer — no selection means the first row,
/// and no `sort` means the queue's own order. AC3's *"off by default"* is this `Option` being
/// `None`, not a stored preference.
#[derive(serde::Deserialize, Default)]
pub struct TriageQuery {
    /// The selected row's id, as `build_triage` minted it.
    pub sel: Option<String>,
    /// `age` turns the age sort on. Anything else, including a typo, leaves it off.
    pub sort: Option<String>,
}

/// The triage screen's body: the mock's two panes, above story 5.14b's reach section.
#[derive(Template)]
#[template(path = "_triage.html")]
struct TriageBody {
    triage: TriageView,
    identity: IdentityView,
    s: Strings,
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
            (StatusCode::INTERNAL_SERVER_ERROR, render_error_body()).into_response()
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
    /// The ONE anchor every example section carries, on every screen (story 6b.6).
    ///
    /// 🔴 Before this story `_devices_example.html` said `screen-section` and `_dashboard.html`
    /// said `dashboard-example`, so story 6b.5's per-section guard covered the dashboard and
    /// **nothing covered the witness screen** — measured on the committed tree, a marker deleted
    /// from `/devices`'s second section left all 634 tests green. *Two anchors meant two guards,
    /// and only one was written.*
    const EXAMPLE_SECTION_ANCHOR: &str = "example-section\"";

    /// The dashboard's own section anchor, as story 6b.5's guards read it.
    ///
    /// ⚠️ It lost its trailing `class="` when story 6b.6 widened the attribute to carry both
    /// classes. A needle that includes the opening `class="` breaks on the ordinary gesture of
    /// adding a second class — which is a limit worth knowing, not just a fix.
    const DASHBOARD_EXAMPLE_ANCHOR: &str = "dashboard-example";

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
        let html = gap_card_html(view, no_reach());
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
        let html = gap_card_html(view, identity);

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
        let html = gap_card_html(build_view(Vec::new(), Vec::new(), None), no_reach());

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

        let html = gap_card_html(build_view(Vec::new(), Vec::new(), None), view);
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
        let html = gap_card_html(
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
            gap_card_html(
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
        let html = gap_card_html(
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
        let html = gap_card_html(
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

        let html = gap_card_html(view, identity);
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

    /// The reconciliation CARD inside the shell — what `GET /gap` serves.
    ///
    /// 🔴 **This was called `triage_html` and the name went false on 2026-08-19**, when story 6b.4
    /// replaced the triage body with the two panes. It renders `GapFragment` directly and never
    /// touches the route, so **the whole body of `/triage` was swapped and all 387 bin tests stayed
    /// green**. A helper named for a route it does not serve is the dominant defect class wearing a
    /// filename: *reading it could not find that, because it is correct about what it renders.*
    /// Renamed to what it is; `/triage` now has route-level tests of its own below.
    fn gap_card_html(view: ReconciledView, identity: IdentityView) -> String {
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
    /// 🔴 **RECURSIVE since story 6b.4's code review, and it was not before.** `read_dir` does not
    /// descend, so **every guard built on this helper was blind to a subdirectory** — measured:
    /// `style="color: var(--accent-document);"` planted in `templates/sub2/_leak.html` left
    /// `ac4_the_amber_is_reserved_for_the_documenting_gesture` GREEN, which is exactly the
    /// smuggling vector that guard's own comment says it was hardened against, and an undefined
    /// class in `templates/sub/_partial.html` left the new stylesheet guard green too. Askama
    /// accepts `path = "sub/_partial.html"` today, so this is **the ordinary gesture of organising
    /// templates**, not an evasion. 🔑 *Two independently-written guards, one shared helper, one
    /// hole* — and the fix belongs here rather than in either of them.
    ///
    /// Names are returned RELATIVE to `templates/` (`sub/_partial.html`, not `_partial.html`), so a
    /// failure message names a path the reader can open.
    ///
    /// # Panics
    ///
    /// If `templates/` cannot be read — which means the test is running somewhere the source
    /// tree is not, and every guard below would be vacuous rather than merely wrong.
    /// AC1 — **no template hard-codes text a human reads out of an attribute.**
    ///
    /// 🔴 **This is the guard story 6b.10's sweep existed to produce, and the defect it found had
    /// survived every story in this epic**: `_gap_card.html` opened with
    /// `aria-label="Reconciliation result"` — English, on the French UI, since before Epic 6b.
    ///
    /// 🔑 **Why nine browser looks walked past it.** It is copy a **sighted** reader never sees.
    /// The epic's method for catching an untranslated string was *look at the page*, and this is
    /// precisely the class that method cannot reach. *A look at the page reads what the page
    /// shows; a screen reader reads what it does not.*
    ///
    /// ⚠️ **The limit, written.** It is a list of attributes, and *an enumeration cannot claim the
    /// completeness of a property* (story 5.12, fifth application here). A new ARIA attribute
    /// carrying text is invisible until someone adds it below. It is a tripwire against the
    /// ordinary gesture of typing a label into markup, never a barrier.
    #[test]
    fn no_template_hard_codes_text_a_human_reads_from_an_attribute() {
        // Every attribute whose VALUE is prose rather than a token. `class`, `id`, `role`,
        // `type`, `hx-*` and the `aria-*` attributes taking an idref or an enum are excluded on
        // purpose: their values are not read aloud and not translated.
        /// Whether an attribute value carries prose OUTSIDE its Askama interpolations.
        ///
        /// 🔑 The question is never *"is an interpolation present"* — a value may hold one and
        /// still ship English words beside it, which story 6b.10's code review measured. Strip
        /// every `{{ … }}` and `{% … %}`, then ask whether any letter survives. Punctuation and
        /// separators do not: `"{{ a }} — {{ b }}"` is composed, not written.
        fn carries_prose(value: &str) -> bool {
            let mut rest = value;
            let mut bare = String::new();
            while let Some(open) = rest.find("{{").or_else(|| rest.find("{%")) {
                bare.push_str(&rest[..open]);
                let close = if rest[open..].starts_with("{{") {
                    "}}"
                } else {
                    "%}"
                };
                match rest[open..].find(close) {
                    Some(at) => rest = &rest[open + at + close.len()..],
                    // An unterminated interpolation is not something to reason past.
                    None => return true,
                }
            }
            bare.push_str(rest);
            bare.chars().any(char::is_alphabetic)
        }

        const PROSE_ATTRIBUTES: [&str; 8] = [
            "aria-label",
            "aria-description",
            "aria-roledescription",
            "aria-placeholder",
            "aria-valuetext",
            "title",
            "alt",
            "placeholder",
        ];
        let mut checked = 0_usize;
        for (name, body) in templates() {
            for attribute in PROSE_ATTRIBUTES {
                // 🔴 **BOTH quoting styles.** Until story 6b.10's code review this matched
                // `attr="` alone, and a single-quoted English literal was measured served
                // verbatim on the French page with 720/720 tests and nine gates green. HTML
                // permits either quote and a developer reaching for one inside an Askama
                // expression reaches for the other.
                for quote in ['"', '\''] {
                    let needle = format!("{attribute}={quote}");
                    for (at, _) in body.match_indices(&needle) {
                        // A boundary, not a substring: `data-title='…'` is not `title`.
                        //
                        // ⚠️ It does NOT earn its keep on `aria-labelledby`, which the needle
                        // already excludes by carrying the `=` — a claim this comment made until
                        // the same review measured it.
                        let before = body[..at].chars().next_back();
                        if before.is_some_and(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                            continue;
                        }
                        let Some((value, _)) = body[at + needle.len()..].split_once(quote) else {
                            continue;
                        };
                        checked += 1;
                        // 🔴 **What remains once the interpolations are removed**, not merely
                        // whether one is present. `aria-label="Reconciliation result for {{ x }}"`
                        // satisfied `contains("{{")` and shipped four English words to a French
                        // screen reader. An empty value stays legal — `alt=""` is the correct
                        // markup for a decorative image, and it reads nothing aloud.
                        assert!(
                            !carries_prose(value),
                            "{name} carries {attribute}={quote}{value}{quote} with LITERAL prose \
                             outside its interpolations — it is read aloud to an operator whose \
                             interface is in another language, and no browser look can see it. \
                             Route every word through a key."
                        );
                    }
                }
            }
        }
        // 🔑 An EQUALITY, not a floor. *A floor is only a guard while it equals what is there*
        // (this file's own rule, quoted in three places): `>= 4` tolerated the silent loss of
        // every prose attribute but four. Raise this deliberately when you add one.
        assert_eq!(
            checked, 6,
            "the premise: six prose-attribute occurrences exist to inspect — a scan that \
             matched fewer has stopped seeing part of the surface it names"
        );
    }

    /// 🔴 **AC1 on the PRODUCER side — and this guard exists because its sibling reads the
    /// TEMPLATE while the defect lives one file over.**
    ///
    /// Story 6b.10's code review measured the ordinary gesture: replace
    /// `gap_card_label: t!("page.gap_card_label").to_string()` with a bare
    /// `"Reconciliation result".to_string()`, leave the template untouched, and the removed
    /// English string is back on the served page — in English, on a French deployment — with
    /// 485/161/74 tests and all nine gates green. `no_template_hard_codes_text_a_human_reads…`
    /// cannot see it: it is entirely CORRECT about what it tests, and the value it inspects is
    /// `{{ s.gap_card_label }}`, an interpolation, whatever the interpolation resolves to.
    ///
    /// ⚠️ **A TRIPWIRE, not a barrier** (story 5.12's precedent, stated rather than implied). It
    /// bounds ONE function — the constructor every template's `s.*` field comes from. A literal
    /// reaching a template by any other route is outside it, and the day a second such
    /// constructor exists this guard must name it too.
    #[test]
    fn every_field_of_the_shared_strings_comes_from_a_key() {
        let source = include_str!("page.rs");
        let at = source
            .find("fn strings() -> Strings {")
            .expect("the shared strings have one constructor");
        let body = &source[at..];
        let body = &body[..body.find("\n}\n").expect("the constructor is a function")];

        let mut checked = 0_usize;
        for line in body.lines() {
            let line = line.trim();
            // A field initialiser is `name: <expr>,` — never the signature, a `use`, or a brace.
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            if !value.trim_end().ends_with(',') || name.contains(' ') || name.is_empty() {
                continue;
            }
            checked += 1;
            assert!(
                value.contains("t!("),
                "`strings().{name}` is fed by `{}` rather than by a key — every field here is \
                 read by a template and shown to an operator, and a literal is invisible to the \
                 template-side guard, which sees only `{{{{ s.{name} }}}}`",
                value.trim().trim_end_matches(',')
            );
        }
        // 🔑 The premise is DERIVED from the struct, never pinned as a number: a second pass
        // over the declaration, so the two cannot drift apart and neither can rot.
        let at = source
            .find("struct Strings {")
            .expect("the shared strings are one struct");
        let declaration = &source[at..];
        let declaration = &declaration[..declaration.find("\n}\n").expect("the struct closes")];
        let fields = declaration
            .lines()
            .filter(|line| line.trim().ends_with(": String,"))
            .count();
        assert_eq!(
            checked, fields,
            "every field of `Strings` was inspected — a scan that reads fewer initialisers than \
             the struct has fields has stopped seeing part of what it names"
        );
    }

    /// 🔴 **A screen that cannot render without the store REFUSES within its budget, and the
    /// test pool is deliberately SLOWER than the budget so the elapsed arm actually runs.**
    ///
    /// Story 6b.9 shipped the mirror of this guard with its pool set to refuse in 150 ms against
    /// a 2 s budget: the timeout branch was dead code under test, and its own comment described
    /// why while reading as a justification. The blind review layer found it from the diff alone.
    /// Here the pool points at an address nothing answers, with an acquire timeout of thirty
    /// seconds — the sqlx default this budget exists to escape — so the ONLY way the assertion
    /// below can pass is through `store_within`'s elapsed arm.
    ///
    /// ⚠️ It asserts on the CLOCK, which no source-reading guard can do: story 6b.10's code
    /// review measured `/triage` answering 500 in 30.002731 s with a body nothing found fault
    /// with. *The sentence was right and the wait was the defect.*
    #[tokio::test]
    async fn a_screen_that_needs_the_store_refuses_within_its_budget() {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect_lazy("mysql://nobody:nothing@127.0.0.1:1/none")
            .expect("a lazy pool needs no server");
        let budget = std::time::Duration::from_millis(200);

        let started = std::time::Instant::now();
        let outcome = store_within(budget, async {
            crate::repo::last_observed_at(&pool).await.map_err(|e| {
                // Reached only if the pool refuses rather than hanging; either way the
                // assertion below is about the CLOCK, not about which arm produced it.
                server_error(e)
            })
        })
        .await;
        let waited = started.elapsed();

        assert!(outcome.is_err(), "an unreachable store cannot be rendered");
        assert!(
            waited < budget * 4,
            "the screen waited {waited:?} against a {budget:?} budget — the whole point is that \
             the operator is not held for sqlx's thirty-second default while a calm sentence \
             waits behind it"
        );
    }

    /// AC1 / arbitration 2(a′) — **a dead store answers in the operator's language.**
    ///
    /// 🔴 The body was the bare English literal `"internal error"`, served at `/triage`,
    /// `/dashboard` and `/sources` — the ten screens' own addresses. Guy's arbitration of
    /// 2026-08-21 brought it inside the perimeter: *an operator whose store is down must not be
    /// the one person who reads English on a French deployment*, and it is the one path where the
    /// interface language silently stopped being the interface language.
    ///
    /// 🔑 **Through the real handler helper and the real `Response`**, never through the constant:
    /// story 6b.4b's finding is that every guard read the source and every defect lived in what
    /// was served.
    #[tokio::test]
    async fn a_dead_store_answers_in_the_operators_language() {
        let response = server_error(sqlx::Error::PoolTimedOut);
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .expect("the error body is small and in memory");
        let body = String::from_utf8(body.to_vec()).expect("the body is UTF-8");
        assert_ne!(
            body, "internal error",
            "the pre-6b.10 literal — English, at a French deployment's own screen addresses"
        );
        for locale in ["en", "fr"] {
            let expected = rust_i18n::t!("error.store_unreachable", locale = locale).to_string();
            assert_ne!(
                expected, "error.store_unreachable",
                "the key resolves in {locale}"
            );
        }
        assert_eq!(body, rust_i18n::t!("error.store_unreachable").to_string());
        // 🔑 The fourth microcopy rule is *cause + next step, never blame the user*, and the
        // fifth is *empty ≠ failure — calm, never alarming*. A store that did not answer has
        // destroyed nothing, and the body must say so rather than leaving the operator to guess.
        for locale in ["en", "fr"] {
            let sentence = rust_i18n::t!("error.store_unreachable", locale = locale).to_string();
            assert!(
                sentence.len() > 60,
                "{locale}: a cause and a next step do not fit in a noun phrase — {sentence:?}"
            );
        }
    }

    /// AC1 / arbitration 2(a′) — and so does a template failure, with a DIFFERENT sentence.
    ///
    /// ⚠️ Two failures, two causes, two next steps. Collapsing them into one *"internal error"*
    /// is what made the old body useless to the person reading it.
    #[test]
    fn a_render_failure_answers_in_the_operators_language_and_says_something_else() {
        let rendered = render_error_body();
        assert_ne!(rendered, "template error", "the pre-6b.10 literal");
        assert_eq!(rendered, rust_i18n::t!("error.render_failed").to_string());
        assert_ne!(
            rendered,
            store_unreachable_body(),
            "a store that did not answer and a template that did not render are different \
             failures with different next steps"
        );
    }

    /// AC1 — **no response body pairs a status with an untranslated literal.**
    ///
    /// 🔑 **A source scan, and deliberately so**: this asserts the ABSENCE of a code path, and
    /// *you cannot measure the absence of code by running code* (story 5.12's sentence, sixth
    /// application). The two tests above measure what IS served; this one measures that no other
    /// site was left behind — six existed before story 6b.10 and the sweep found them by reading.
    ///
    /// ⚠️ Its limit: a status paired with a literal assembled at run time is invisible here, as
    /// is any file outside the two it reads — and `document.rs` is one of those, on purpose. See
    /// the comment on the file list for what that costs and who owns it.
    ///
    /// 🔴 **It reddened on its own source the first time it ran**, because a file's test module
    /// contains the guard's own needle as a string literal. *A matcher without a boundary finds
    /// the language it is written in* — story 6b.6's sentence, met again — so it reads the CODE
    /// half only, which is D56b's own line and the same cut the `file-size` gate makes.
    #[test]
    fn no_handler_pairs_a_status_with_an_untranslated_literal() {
        /// Everything before the file's trailing `#[cfg(test)]` module (D56b: one per file).
        fn code_half(source: &str) -> &str {
            source
                .find("\n#[cfg(test)]\n")
                .map_or(source, |at| &source[..at])
        }
        // 🔴 **Every status a human reads, not `INTERNAL_SERVER_ERROR` alone.** Until story
        // 6b.10's code review this matched one status while its own name promised *"a status"* —
        // so a hand-written `(StatusCode::NOT_FOUND, "unknown device")` sat outside a guard
        // titled as though it covered it, and that is the ordinary gesture.
        const READ_BY_A_HUMAN: [&str; 6] = [
            "StatusCode::INTERNAL_SERVER_ERROR,",
            "StatusCode::NOT_FOUND,",
            "StatusCode::FORBIDDEN,",
            "StatusCode::BAD_REQUEST,",
            "StatusCode::CONFLICT,",
            "StatusCode::UNAUTHORIZED,",
        ];
        let mut checked = 0_usize;
        // 🔴 `document.rs` is DELIBERATELY absent, and the exclusion is written here rather than
        // implied by the list. Adding it reds on
        // `"documenting failed — the store did not accept the write"` — a real, English,
        // operator-readable 500 body, which this guard found on its first run and which Guy's
        // arbitration 2(a′) of 2026-08-21 places OUTSIDE story 6b.10: no template calls
        // `POST /document-all` today, so translating its bodies now is copy nobody can reach,
        // written against a gesture story 6.4 may reshape. **Owner: story 6.4**, which is the
        // story that gives the route a caller. Widen this list there, not before.
        // ⚠️ `metrics.rs` is DELIBERATELY absent too, and the sentence is here rather than
        // implied by the list — the guard found it on the first run of this widening, answering
        // 500 with the English literal `"metrics encode error"`. Guy's arbitration 3 of
        // 2026-08-22 settles it by its own criterion: the 401 came inside the perimeter because
        // it is *served at the ten screens' own addresses*, and `/metrics` is not one of them.
        // It is a scrape endpoint read by Prometheus, and a French sentence there would be copy
        // written for no reader. **Owner: Epic 16**, which is where alerting gives `/metrics` an
        // operator-facing surface, if it ever does.
        //
        // 🔴 **`example_screens.rs` and `auth.rs` join the list**, and why the first was missing
        // is the sharpest part: it exists BECAUSE story 6b.6 split `page.rs`
        // under file-size pressure, so the next such split would silently move an error body out
        // of this guard's reach. A perimeter keyed to file NAMES rots every time a file is born.
        for (file, source) in [
            ("page.rs", code_half(include_str!("page.rs"))),
            ("diagnostic.rs", code_half(include_str!("diagnostic.rs"))),
            (
                "example_screens.rs",
                code_half(include_str!("example_screens.rs")),
            ),
            ("auth.rs", code_half(include_str!("auth.rs"))),
        ] {
            for status in READ_BY_A_HUMAN {
                for (at, _) in source.match_indices(status) {
                    let tail = source[at..].split_once(',').map_or("", |(_, rest)| rest);
                    let argument = tail.trim_start();
                    checked += 1;
                    assert!(
                        !argument.starts_with('"'),
                        "{file} answers {status} with the literal {}, which reaches an operator \
                         whose interface is in another language — route it through a key",
                        argument.split('"').nth(1).unwrap_or(argument).trim()
                    );
                }
            }
        }
        // 🔑 An EQUALITY. `>= 5` stood over six visible sites, so it tolerated the silent loss of
        // one — and the test's own doc said *"six existed before story 6b.10"* while asserting
        // five. Raise this deliberately when you add an error body.
        assert_eq!(
            checked, 8,
            "the premise: eight operator-readable status sites exist across the four files — a \
             scan that matched fewer has stopped seeing part of the surface it names"
        );
    }

    fn templates() -> Vec<(String, String)> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("templates");
        fn walk(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<(String, String)>) {
            let entries = std::fs::read_dir(dir).unwrap_or_else(|e| {
                panic!("templates/ must be readable at {}: {e}", dir.display())
            });
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, root, out);
                } else if path.extension().is_some_and(|ext| ext == "html") {
                    let name = path
                        .strip_prefix(root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .into_owned();
                    let body = std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("reading {name}: {e}"));
                    out.push((name, body));
                }
            }
        }
        let mut found: Vec<(String, String)> = Vec::new();
        walk(&root, &root, &mut found);
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

    // ── Story 6b.4: the triage screen, on the real gap ────────────────

    /// An instant, `seconds` after the epoch.
    fn at(seconds: i64) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp(seconds, 0).expect("a valid instant")
    }

    /// One observed batch: its source, its instant, its facts.
    fn batch(source: &str, seconds: i64, facts: Vec<Fact>) -> crate::repo::ObservedBatch {
        crate::repo::ObservedBatch {
            connector_id: source.into(),
            observed_at: at(seconds),
            facts,
        }
    }

    /// One declared attribute's provenance, for the display side.
    fn prov(
        entity: &str,
        key: &str,
        origin: &str,
        seconds: i64,
    ) -> crate::repo::DeclaredProvenance {
        crate::repo::DeclaredProvenance {
            entity_id: entity.into(),
            attr_key: key.into(),
            origin: origin.into(),
            updated_at: at(seconds),
        }
    }

    /// 🔴 **The builder reads NO clock, and this is the guard that says so for `build_triage`.**
    ///
    /// # Why story 5.14b's guard does not cover this and a new one was needed
    ///
    /// `the_view_builder_has_no_clock_so_one_store_renders_identically` hands `build_view` EMPTY
    /// declared rows and EMPTY observations, so nothing in its populated branch is ever reached —
    /// measured by story 6b.4's validation. And the carrier everyone assumed, `chrono`'s
    /// `default-features = false`, only stops one SPELLING: `chrono::Utc::now()` is `E0599` here,
    /// while `std::time::SystemTime::now()` compiles freely. *A feature flag is a guard against a
    /// name, never against reading the clock.*
    ///
    /// So this guard feeds a POPULATED store and asserts the output is a function of the instant it
    /// was GIVEN: same store, two different `now`s, two different freshness strings; same store,
    /// the same `now` twice, byte-identical rows.
    /// The `/sources` body as it is served, without the shell.
    fn rendered_sources(
        perimeter: Option<String>,
        last: Option<chrono::DateTime<chrono::Utc>>,
    ) -> String {
        SourcesBody {
            source: build_sources(perimeter, last, at(600)),
            s: source_strings(),
        }
        .render()
        .expect("the sources template and its struct are compiled together")
    }

    /// 🔴 **AC1 — the capability boundary is REAL, and it is derived rather than listed.**
    ///
    /// The complement is `FactKind::ALL` minus what the connector declares. Five kinds today, and
    /// the test names them — but it also asserts the PARTITION, so the day an eighth kind exists the
    /// two halves still cover it. ⚠️ A `#[test]` cannot add an enum variant; the eighth-kind case is
    /// a mutation, and its carrier is the cross-crate row in
    /// `screens::tests::every_variant_of_a_navigated_enum_is_listed_in_all`.
    #[test]
    fn what_the_source_cannot_see_is_derived_from_the_connectors_own_declaration() {
        let (observes, cannot_see) = crate::arp_ping::observes_and_cannot_see();
        assert_eq!(
            observes,
            vec![FactKind::IpV4, FactKind::Rtt],
            "the shipped connector observes an address and a round-trip time, and story 5.14 pinned \
             that it declares no MAC, ever"
        );
        assert_eq!(
            cannot_see,
            vec![
                FactKind::Mac,
                FactKind::Hostname,
                FactKind::DhcpLease,
                FactKind::Uplink,
                FactKind::OuiVendor,
            ],
            "and the five it cannot see are the complement — this is the one section of /sources \
             that AC1 requires to be REAL"
        );
        // The PARTITION, which survives an eighth kind where the two literals above would not.
        let mut union = observes.clone();
        union.extend(cannot_see.iter().copied());
        union.sort();
        let mut all = FactKind::ALL.to_vec();
        all.sort();
        assert_eq!(union, all, "the two halves must cover every kind");
        assert!(
            !observes.iter().any(|kind| cannot_see.contains(kind)),
            "and must not overlap"
        );
    }

    /// 🔴 **EVERY KIND HAS ITS OWN SENTENCE, and nothing checked that until the code review.**
    ///
    /// `FactKind::ALL`'s cross-crate guard pins the CONSTANT against the enum's declaration. It says
    /// nothing about [`kind_keys`], so an eighth kind correctly added to `ALL` would satisfy it and
    /// still fall into the `_` arm — rendering *"Unrecognised kind"* on `/sources`, identically for
    /// every unmapped kind, with the page giving no clue which one appeared. **A guard placed where
    /// the defect cannot occur, one field over**, and the blind review layer found it from the diff
    /// alone by noticing that the two guarantees are not the same guarantee.
    ///
    /// 🔑 It also pins DISTINCTNESS: two kinds sharing a key pair would render the same sentence for
    /// two different facts, which no count and no resolution check can see.
    #[test]
    fn every_fact_kind_has_its_own_sentence() {
        let mut seen: Vec<(&str, &str)> = Vec::new();
        for kind in FactKind::ALL {
            let (label, meaning) = kind_keys(kind);
            assert_ne!(
                label, "kind.unknown",
                "{kind:?} falls into the `_` arm and would render *Unrecognised kind* — the \
                 `FactKind::ALL` guard cannot see this, because it pins the CONSTANT and not this map"
            );
            assert_ne!(meaning, "kind.unknown.meaning", "{kind:?}");
            assert!(
                !seen.contains(&(label, meaning)),
                "{kind:?} shares its key pair with another kind: two different facts would render \
                 one sentence, which no resolution check can notice"
            );
            // Both keys must RESOLVE — `rust-i18n` renders an unknown key verbatim.
            assert_ne!(rust_i18n::t!(label), label, "{label} does not resolve");
            assert_ne!(
                rust_i18n::t!(meaning),
                meaning,
                "{meaning} does not resolve"
            );
            seen.push((label, meaning));
        }
        assert_eq!(seen.len(), FactKind::ALL.len());
    }

    /// 🔴 **AC2 — the liveness axis carries NO COLOUR, and this is what stops one appearing.**
    ///
    /// The product can establish *an observation arrived at T*; it cannot establish *this source is
    /// blind*, which is a verdict nothing computes. ⚠️ **This is a DIVERGENCE from the UX spec**,
    /// which prescribes a colour for BOTH liveness values — registered, not conformance.
    ///
    /// ⚠️ **A TRIPWIRE, NOT A BARRIER, and the narrowing is the code review's** (story 5.12's
    /// precedent, applied again). This asserts the absence of six hand-picked spellings; an inline
    /// `style="color:…"`, a differently-named CSS variable or a `class="dot-live"` would pass
    /// undetected. *An enumeration cannot claim the completeness of a property* — read it as *"a
    /// future story will not paint this axis by accident"*, never as *"this axis cannot be painted"*.
    #[test]
    fn the_liveness_axis_carries_no_colour_and_no_verdict() {
        let html = rendered_sources(Some("192.0.2.0/24".into()), Some(at(0)));
        for banned in [
            "statepill",
            "badge",
            "pill",
            "--color-accent-700",
            "is-blind",
            "is-live",
        ] {
            assert!(
                !html.contains(banned),
                "/sources must carry no {banned:?}: a colour on this axis asserts a verdict the \
                 product cannot compute, and deriving `blind` from silence would invent an incident"
            );
        }
        // And the sentence that says the incident axis is not built must be there.
        assert!(
            html.contains(&rust_i18n::t!("sources.incident_axis").to_string()),
            "the screen must say that the blind/live question is not answered yet"
        );
    }

    /// 🔴 **A REFUSED PERIMETER SAYS SO, and it did not until the code review.**
    ///
    /// Measured by booting the real binary with `OPENCMDB_SCAN_CIDR=nonsense`: the log carried
    /// `ERROR invalid OPENCMDB_SCAN_CIDR — skipping scan` and the screen rendered *"Périmètre
    /// nonsense"* with the generic four-state sentence under it — **the rejected string presented as
    /// an in-force value.** Not the ambiguity the story registered: *we cannot tell which of four* is
    /// one thing, *we are showing a configuration we already refused as though it were live* is
    /// another.
    ///
    /// 🔑 The check uses the connector's OWN parser, so the screen and the scan cannot disagree.
    #[test]
    fn a_refused_perimeter_says_so_rather_than_reading_as_configured() {
        let bad = rendered_sources(Some("nonsense".into()), None);
        assert!(
            bad.contains(&rust_i18n::t!("sources.refused").to_string()),
            "a perimeter the product refused must be named as refused"
        );
        // The control: a VALID perimeter must not carry the sentence, or it would say every
        // configuration was refused and mean nothing.
        let good = rendered_sources(Some("192.0.2.0/24".into()), None);
        assert!(!good.contains(&rust_i18n::t!("sources.refused").to_string()));
        // ⚠️ And a refused perimeter is still SHOWN — the operator must see what was rejected, or
        // they cannot fix it. What changed is that it is no longer shown as if it were live.
        assert!(bad.contains("nonsense"));
    }

    /// 🔴 **`None` is FOUR states of the world and the screen says so.**
    ///
    /// Measured on four live boots at the story's validation: never scanned, scanned and nobody
    /// answered, an INVALID perimeter the product refused, and a blank one — `MAX(observed_at)` is
    /// NULL in all four, so FR8's own distinction fails at boot level.
    #[test]
    fn a_silent_source_states_the_ambiguity_rather_than_picking_a_reading() {
        let html = rendered_sources(Some("192.0.2.0/24".into()), None);
        assert!(html.contains(&rust_i18n::t!("sources.never").to_string()));
        assert!(
            html.contains(&rust_i18n::t!("sources.ambiguity").to_string()),
            "saying only *nothing observed* would let the operator read it as *the source is fine \
             and the network is empty*, which is one of four readings and not the measured one"
        );
        // The control: with an observation, the ambiguity sentence is gone and a duration is shown.
        let seen = rendered_sources(Some("192.0.2.0/24".into()), Some(at(0)));
        assert!(!seen.contains(&rust_i18n::t!("sources.ambiguity").to_string()));
    }

    /// 🔴 **No perimeter, no source** — and the screen says that instead of inventing one.
    ///
    /// The story's first draft assumed one configured source throughout; a real boot with
    /// `OPENCMDB_SCAN_CIDR` unset has none.
    #[test]
    fn no_configured_perimeter_means_no_source_rather_than_an_invented_one() {
        let html = rendered_sources(None, None);
        assert!(html.contains(&rust_i18n::t!("sources.no_source").to_string()));
        let name = rust_i18n::t!(crate::arp_ping::SOURCE_NAME_KEY).to_string();
        assert!(
            !html.contains(&name),
            "listing a source the product was never asked to build is the fabrication the liveness \
             arbitration refuses, in the other direction"
        );
        // The control: with a perimeter, the source IS named.
        let configured = rendered_sources(Some("192.0.2.0/24".into()), None);
        assert!(configured.contains(&name));
    }

    /// The screen carries BOTH halves of the spec's card — *Observes* as well as *Cannot see*.
    #[test]
    fn the_screen_shows_what_the_source_observes_and_not_only_what_it_cannot() {
        let html = rendered_sources(Some("192.0.2.0/24".into()), Some(at(0)));
        for key in ["sources.observes", "sources.cannot_see", "sources.unlock"] {
            assert!(
                html.contains(&rust_i18n::t!(key).to_string()),
                "{key} belongs on the screen: AC1 quotes the negative half only, and delivering \
                 only the negative half is the spec's illustration cut in two"
            );
        }
        // Every kind's own sentence is rendered, not just its name.
        let (observes, cannot_see) = crate::arp_ping::observes_and_cannot_see();
        for kind in observes.into_iter().chain(cannot_see) {
            let (label, meaning) = kind_keys(kind);
            assert!(html.contains(&rust_i18n::t!(label).to_string()), "{label}");
            assert!(
                html.contains(&rust_i18n::t!(meaning).to_string()),
                "{meaning}"
            );
        }
    }

    /// The view builder reads no clock of its own — the house rule for every builder in this file.
    #[test]
    fn build_sources_reads_no_clock_of_its_own() {
        let a = build_sources(Some("192.0.2.0/24".into()), Some(at(0)), at(600));
        let b = build_sources(Some("192.0.2.0/24".into()), Some(at(0)), at(600));
        assert_eq!(
            a.expect("a source").last_observed,
            b.expect("a source").last_observed
        );
    }

    #[test]
    fn build_triage_reads_no_clock_of_its_own() {
        let declared = vec![
            declared_row("e1", "ipv4", "192.0.2.10"),
            declared_row("e1", "hostname", "nas"),
        ];
        let observations = vec![batch(
            "unifi",
            0,
            vec![ipv4("192.0.2.10"), hostname("other")],
        )];
        let build = |now| {
            build_triage(
                declared.clone(),
                vec![prov("e1", "hostname", "manual", 0)],
                observations.clone(),
                now,
                None,
                false,
            )
        };

        // The same instant twice: identical, down to the rendered freshness.
        let a = build(at(3_600));
        let b = build(at(3_600));
        assert_eq!(
            a.rows.iter().map(|r| r.seen.clone()).collect::<Vec<_>>(),
            b.rows.iter().map(|r| r.seen.clone()).collect::<Vec<_>>(),
            "one store, one instant, twice — the builder must be a pure function of its inputs"
        );

        // A different instant: the freshness MOVES, which proves the instant is the one passed in
        // and not one the builder went and read.
        let later = build(at(7_200));
        assert_ne!(
            a.rows[0].seen, later.rows[0].seen,
            "the freshness must follow the `now` the caller supplied — if it does not, this guard \
             is measuring nothing and a clock inside the builder would be invisible to it"
        );
    }

    /// The relative time is rendered from the interval, in the operator's language.
    ///
    /// ⚠️ **Interpolated, not concatenated**: *"4 min ago"* against *"il y a 4 min"* put the number
    /// on opposite sides, so a `format!` of a translated word and a number is correct in exactly one
    /// language. That is an NFR26 defect no locale guard can see, because both halves ARE keys.
    /// 🔴 **`set_locale` was called here, and it is process-wide.** Story 6b.6's validation
    /// measured the consequence with its control: this test run beside
    /// `build_view_empty_when_no_declared_entity` at `--test-threads=2` reddened **18 times out of
    /// 60**, and **0 out of 30** single-threaded — a reproducible race, on a comment eleven hundred
    /// lines above that says in so many words that `set_locale` *"is NOT used, and must not be"*.
    /// It reddened a run of story 6b.6 itself. The French half now resolves the KEY with an explicit
    /// locale instead of moving the whole process, so the two halves can no longer collide.
    ///
    /// ⚠️ The French half is therefore one step weaker: it checks the TRANSLATION and its
    /// interpolation, not `relative_time`'s branch selection, which the English half covers. That is
    /// the trade, stated rather than hidden — and it costs nothing here, because the branch
    /// selection is language-independent by construction.
    #[test]
    fn relative_time_reads_in_both_languages_and_at_the_boundaries() {
        use rust_i18n::t;
        assert_eq!(relative_time(at(100), at(60)), "just now");
        assert_eq!(relative_time(at(3_540), at(0)), "59 min ago");
        // The boundary: 60 minutes is one hour, not "60 min".
        assert_eq!(relative_time(at(3_600), at(0)), "1 h ago");
        assert_eq!(relative_time(at(90_000), at(0)), "1 d ago");
        // A source dated in the future is STATED, never rendered as a negative duration.
        assert_eq!(relative_time(at(0), at(60)), "dated ahead");

        // 🔑 The property this half exists for: *"4 min ago"* against *"il y a 4 min"* put the
        // number on OPPOSITE sides, so a `format!` of a translated word and a number is correct in
        // exactly one language. Resolving with an explicit locale keeps that measurable.
        assert_eq!(t!("time.minutes", n = 59, locale = "fr"), "il y a 59 min");
        assert_eq!(t!("time.hours", n = 1, locale = "fr"), "il y a 1 h");
    }

    /// 🔴 **The queue's row vocabulary is the engine's, and `Ambigu` is absent because nothing
    /// produces it.**
    ///
    /// Of the mock's five kinds, three are already typed by `gap::reconcile` — a `Gap` is *Écart*,
    /// `NoObservedValue` is *Absence*, `ConflictingObservations` is *Conflit* — *Nouveau* is an
    /// observed address no declared entity claims, and `Ambigu` needs FR16's ranked candidates,
    /// which `link_candidate` stores and nothing reads. **Epic 6's, and its absence is asserted so
    /// nobody adds a row the engine cannot fill.**
    ///
    /// ⚠️ `OutOfPerimeter` is never a row: `reconcile` is written for ONE perimeter, so each pass
    /// counts every OTHER entity's observations as out of perimeter. It is noise of the loop, and
    /// surfacing it would put one row per entity per other entity on the operator's screen.
    #[test]
    fn the_queue_carries_the_four_kinds_the_engine_can_produce_and_no_others() {
        let declared = vec![
            // A drift: hostname declared `nas`, observed `intruder`.
            declared_row("drift", "ipv4", "192.0.2.10"),
            declared_row("drift", "hostname", "nas"),
            // An absence: a declared field no observation reports.
            declared_row("absent", "ipv4", "192.0.2.20"),
            declared_row("absent", "hostname", "unseen"),
            // A conflict: two observations disagree on the hostname.
            declared_row("clash", "ipv4", "192.0.2.30"),
            declared_row("clash", "hostname", "either"),
        ];
        let observations = vec![
            batch("unifi", 10, vec![ipv4("192.0.2.10"), hostname("intruder")]),
            batch("unifi", 20, vec![ipv4("192.0.2.20")]),
            batch("unifi", 30, vec![ipv4("192.0.2.30"), hostname("one")]),
            batch("arp", 40, vec![ipv4("192.0.2.30"), hostname("two")]),
            // An address nobody declared.
            batch("arp", 50, vec![ipv4("192.0.2.99")]),
        ];
        let view = build_triage(declared, Vec::new(), observations, at(1_000), None, false);

        let kinds: Vec<&str> = view.rows.iter().map(|r| r.kind.as_str()).collect();
        // ⚠️ « Gap » and not « Drift »: story 6b.6's glossary check found that the English
        // locale rendered a SYNONYM for the product's core term, which the binding table
        // forbids in so many words. This assertion's own message already said *"a Gap"*.
        assert!(kinds.contains(&"Gap"), "a Gap must be a row: {kinds:?}");
        assert!(
            kinds.contains(&"Absence"),
            "NoObservedValue must be a row: {kinds:?}"
        );
        assert!(
            kinds.contains(&"Conflict"),
            "ConflictingObservations must be a row: {kinds:?}"
        );
        assert!(
            kinds.contains(&"New"),
            "an undeclared observed address must be a row: {kinds:?}"
        );
        assert!(
            !view.rows.iter().any(|r| r.id.contains("outofperimeter")),
            "`OutOfPerimeter` is noise of the per-entity loop, never a row the operator sees"
        );
        // FR16b: a cause is ONE line carrying its count, never N failures.
        let absence = view
            .rows
            .iter()
            .find(|r| r.kind == "Absence")
            .expect("the absence row");
        assert!(
            absence.counted && absence.count.contains('1'),
            "a cause row carries its count WITH its unit — *one line and one gesture, not N \
             failures* (FR16b) — and a bare number beside an address reads as noise: {:?}",
            absence.count
        );
        assert!(
            absence.count.chars().any(|c| c.is_alphabetic()),
            "the count carries a unit, not a naked digit: {:?}",
            absence.count
        );
    }

    /// 🔴 **AC3: age sorting is available and OFF by default — and this guard pins the ORDER.**
    ///
    /// ⚠️ **The shape matters more than the assertion.** Story 6b.4's validation built the other
    /// shape — a guard asserting only that *the toggle changes something* — and measured it **GREEN
    /// under the exact mutation it exists to catch** (flip the default to on). This one compares the
    /// default order against the queue's own order and reds.
    #[test]
    fn the_age_sort_is_off_by_default_and_oldest_first_when_on() {
        let declared = vec![
            declared_row("young", "ipv4", "192.0.2.10"),
            declared_row("young", "hostname", "a"),
            declared_row("old", "ipv4", "192.0.2.20"),
            declared_row("old", "hostname", "b"),
        ];
        let observations = vec![
            // `young` was seen at t=900 (recent), `old` at t=10 (ancient).
            batch("unifi", 900, vec![ipv4("192.0.2.10"), hostname("z")]),
            batch("unifi", 10, vec![ipv4("192.0.2.20"), hostname("z")]),
        ];
        let build = |sort| {
            build_triage(
                declared.clone(),
                Vec::new(),
                observations.clone(),
                at(1_000),
                None,
                sort,
            )
        };

        let default_order: Vec<String> = build(false).rows.iter().map(|r| r.id.clone()).collect();
        let sorted_order: Vec<String> = build(true).rows.iter().map(|r| r.id.clone()).collect();

        // OFF by default: the queue keeps the declaration order, `young` first.
        assert!(
            default_order[0].contains("young"),
            "off by default means the queue's own order, not age's: {default_order:?}"
        );
        // ON: oldest first. The ban is not that age is hidden — it is that it is never brandished.
        assert!(
            sorted_order[0].contains("old"),
            "sorting by age puts the oldest first: {sorted_order:?}"
        );
        assert_ne!(
            default_order, sorted_order,
            "the two orders must differ, or this fixture cannot tell the default from the sort"
        );
    }

    /// 🔴 **AC1: BOTH photos carry their own provenance and their own freshness.**
    ///
    /// *Neither side is the truth* — an observation can be stale or from a blind source, and a
    /// declaration can be outdated. A pane whose meta-line is missing invites the reader to treat
    /// that side as fact, which is the one thing this screen exists not to do.
    #[test]
    fn both_photos_carry_a_provenance_and_a_freshness_of_their_own() {
        let declared = vec![
            declared_row("e1", "ipv4", "192.0.2.10"),
            declared_row("e1", "hostname", "nas"),
        ];
        let observations = vec![batch(
            "unifi",
            0,
            vec![ipv4("192.0.2.10"), hostname("other")],
        )];
        let view = build_triage(
            declared,
            vec![prov("e1", "hostname", "adopted", 2_400)],
            observations,
            at(3_000),
            None,
            false,
        );
        let pane = view.selected.expect("the first row is selected");

        assert_eq!(pane.declared_meta.source, "Adopted from an observation");
        assert_eq!(pane.declared_meta.freshness, "10 min ago");
        // The SHORT id, labelled — never the whole UUID (see `source_label`).
        assert_eq!(pane.observed_meta.source, "Source unifi");
        assert_eq!(pane.observed_meta.freshness, "50 min ago");
        assert_ne!(
            (&pane.declared_meta.source, &pane.declared_meta.freshness),
            (&pane.observed_meta.source, &pane.observed_meta.freshness),
            "the two meta-lines must be the two sides' OWN facts — equal ones would mean the pane \
             is showing one side's provenance twice"
        );
    }

    /// An unfamiliar `origin` token is LABELLED and counted, never a 500.
    ///
    /// 🔑 Story 5.14b's arbitration 11, applied to a second column: `declared_attribute.origin` is
    /// a plain `VARCHAR(16)` with no `CHECK`, so an invented token inserts cleanly. Turning it into
    /// an error here would move the failure from the DISPLAY to the WRITE — *a display story may not
    /// be the place a write starts failing.*
    #[test]
    fn an_unfamiliar_origin_is_labelled_rather_than_fatal() {
        let declared = vec![
            declared_row("e1", "ipv4", "192.0.2.10"),
            declared_row("e1", "hostname", "nas"),
        ];
        let view = build_triage(
            declared,
            vec![prov("e1", "hostname", "smuggled", 0)],
            vec![batch(
                "unifi",
                0,
                vec![ipv4("192.0.2.10"), hostname("other")],
            )],
            at(60),
            None,
            false,
        );
        let pane = view.selected.expect("the first row is selected");
        assert_eq!(pane.declared_meta.source, "Unfamiliar origin");
    }

    /// The selection survives the sort toggle, and an unknown `?sel=` falls back to the first row.
    #[test]
    fn the_selection_is_addressable_and_survives_the_sort() {
        let declared = vec![
            declared_row("e1", "ipv4", "192.0.2.10"),
            declared_row("e1", "hostname", "nas"),
        ];
        let observations = vec![batch(
            "unifi",
            0,
            vec![ipv4("192.0.2.10"), hostname("other")],
        )];
        let view = build_triage(
            declared.clone(),
            Vec::new(),
            observations.clone(),
            at(60),
            Some("ecart:e1:hostname"),
            false,
        );
        assert!(
            view.rows
                .iter()
                .any(|r| r.selected && r.id == "ecart:e1:hostname")
        );
        assert!(
            view.sort_href.contains("sort=age") && view.sort_href.contains("sel="),
            "the toggle must carry the selection, or sorting silently moves the operator's row: {}",
            view.sort_href
        );

        // An id nobody minted selects the first row rather than nothing — a URL survives a queue
        // that changed under it.
        let stale = build_triage(
            declared,
            Vec::new(),
            observations,
            at(60),
            Some("ecart:vanished:hostname"),
            false,
        );
        assert!(
            stale.selected.is_some(),
            "a stale selector falls back, never blanks the pane"
        );
    }

    /// The queue's empty state says so, rather than rendering an empty list.
    #[test]
    fn an_empty_queue_says_it_is_empty() {
        let view = build_triage(Vec::new(), Vec::new(), Vec::new(), at(0), None, false);
        assert_eq!(view.total, 0);
        assert!(view.selected.is_none());
        let html = TriageBody {
            triage: view,
            identity: no_reach(),
            s: strings(),
        }
        .render()
        .expect("the triage body renders");
        assert!(
            html.contains("You are up to date"),
            "an empty queue must SAY it is empty, not render an empty list"
        );
    }

    /// The rendered triage body carries both panes and both meta-lines.
    #[test]
    fn the_triage_body_renders_the_queue_and_the_two_photos() {
        let declared = vec![
            declared_row("e1", "ipv4", "192.0.2.10"),
            declared_row("e1", "hostname", "nas"),
        ];
        let view = build_triage(
            declared,
            vec![prov("e1", "hostname", "manual", 0)],
            vec![batch(
                "unifi",
                0,
                vec![ipv4("192.0.2.10"), hostname("intruder")],
            )],
            at(600),
            None,
            false,
        );
        let html = TriageBody {
            triage: view,
            identity: no_reach(),
            s: strings(),
        }
        .render()
        .expect("the triage body renders");

        assert!(html.contains("queue"), "the queue pane renders");
        assert!(html.contains("photos"), "the two-photo pane renders");
        assert!(
            html.contains("nas") && html.contains("intruder"),
            "both values render"
        );
        assert!(
            html.contains("Entered by hand"),
            "the declared meta-line renders"
        );
        assert!(
            html.contains("unifi"),
            "the observed meta-line names its source"
        );
        assert!(html.contains("10 min ago"), "the freshness renders");
        // Story 5.14b's reach section survived the body swap — it was extracted, not deleted.
        assert!(
            html.contains("identity"),
            "the reach section must survive: no acceptance criterion asked to remove it"
        );
    }

    /// 🔴 **No template names a CSS class this stylesheet does not define.**
    ///
    /// # The defect this exists for, and why no earlier layer could see it
    ///
    /// Story 6b.3 shipped `_devices_example.html` carrying `class="screen-section"`, which
    /// `app.css` did not define, and `class="rows"` on a `<table>` — `.rows` being a `<dl>` ruleset
    /// written for `_gap_card.html`, so every descendant rule matched nothing. The sheet had no
    /// `table`, `th` or `td` rule at all, and **the one witness screen that story existed to produce
    /// rendered as browser defaults.** Three review layers measured the served TEXT correct in every
    /// respect; the page was not. It was caught by RECOUNTING the sheet, and only after the fact.
    ///
    /// 🔑 **A PROPERTY over the directory, never a list.** It enumerates `templates/` at run time
    /// and the sheet's selectors at run time, so a class added to a NEW partial is covered the day
    /// the file exists — the failure mode story 6b.2's validation measured on a guard keyed to a
    /// `[&str; 2]` literal.
    ///
    /// ⚠️ **That sentence was FALSE when it was first written, and the review measured it.** The walk
    /// was a flat `read_dir`, so a class planted in `templates/sub/_partial.html` left this guard
    /// green while the same class in a top-level file reddened it — *"covered the day the file
    /// exists"* held for one directory only. It now uses the shared [`templates`] walker, which was
    /// made recursive for the same reason and in the same pass; the inherited `--accent-document`
    /// guard had the identical hole through the identical helper.
    ///
    /// ⚠️ **The exemption list is EMPTY, and that is the point** (story 6.3's idiom): the one form
    /// of allowlist nobody can quietly widen. A class that needs no rule gets a real one — `.col`
    /// is a grid child and `min-width: 0` is what lets it shrink — rather than an entry here.
    ///
    /// ⚠️ **Its limit, written rather than implied**: this is a TRIPWIRE against the ordinary
    /// gesture of naming a class and forgetting the rule. It reads `class="…"` literals only, so a
    /// class assembled in Rust or interpolated by askama is invisible to it, and it says nothing
    /// about whether a rule that EXISTS is the right one — `.rows` on a `<table>` would still pass.
    /// *Only a browser can answer that, and no story in this epic has had one yet.*
    #[test]
    fn every_class_a_template_names_is_defined_in_the_stylesheet() {
        const EXEMPT: [&str; 0] = [];

        // 🔑 The SHARED walker, not a second one of my own. It was a flat `read_dir` here and in
        // `templates()` both, and the review measured the consequence: an undefined class planted
        // in `templates/sub/_partial.html` left this guard GREEN while the same class in a
        // top-level file reddened it. One helper, one fix, and this guard cannot drift from the
        // others again — the DRY rule doing real work rather than tidying.
        let scanned_files = templates();
        let mut used: Vec<(String, String)> = Vec::new();
        let mut scanned = 0_usize;
        for (name, source) in &scanned_files {
            let name = name.clone();
            scanned += 1;
            for (index, _) in source.match_indices("class=\"") {
                let rest = &source[index + "class=\"".len()..];
                let Some(end) = rest.find('"') else { continue };
                let literal = &rest[..end];
                // An askama expression inside the attribute is not a literal class name.
                if literal.contains('{') {
                    continue;
                }
                for class in literal.split_whitespace() {
                    used.push((name.clone(), class.to_string()));
                }
            }
        }

        let sheet = sheet();
        let defined = |class: &str| -> bool {
            sheet.match_indices(&format!(".{class}")).any(|(at, _)| {
                // A selector ends at a character that cannot continue an identifier; otherwise
                // `.count` would be "defined" by a rule for `.counter`.
                sheet[at + 1 + class.len()..]
                    .chars()
                    .next()
                    .is_none_or(|c| !c.is_alphanumeric() && c != '-' && c != '_')
            })
        };

        // The premise FIRST: a walk that found nothing would make every assertion below vacuous.
        assert!(
            scanned >= 5 && used.len() >= 20,
            "the premise: this guard must have read the templates ({scanned} file(s), \
             {} class use(s)) — a walk that went empty asserts nothing",
            used.len()
        );
        for (template, class) in &used {
            assert!(
                EXEMPT.contains(&class.as_str()) || defined(class),
                "{template} names `.{class}` and `app.css` defines no rule for it — it renders as \
                 a browser default, which no test can see and only a look can catch (story 6b.3)"
            );
        }
    }

    // ── Story 6b.4b: the action bar, and the gesture nature ───────────

    /// The triage body as it reaches the wire, on a store that produces one drift row.
    ///
    /// 🔑 Shared by the guards that must read the RENDER rather than the source — see
    /// [`a_planned_control_is_reachable_and_never_natively_disabled`] for what reading the source
    /// cost this story.
    fn rendered_triage_body() -> String {
        let declared = vec![
            declared_row("e1", "ipv4", "192.0.2.10"),
            declared_row("e1", "hostname", "nas"),
        ];
        let view = build_triage(
            declared,
            Vec::new(),
            vec![batch(
                "unifi",
                0,
                vec![ipv4("192.0.2.10"), hostname("other")],
            )],
            at(600),
            None,
            false,
        );
        TriageBody {
            triage: view,
            identity: no_reach(),
            s: strings(),
        }
        .render()
        .expect("the triage body renders")
    }

    /// 🔴 **A planned control is REACHABLE by keyboard, announced as unavailable, and never
    /// natively disabled — asserted on the RENDERED HTML.**
    ///
    /// # Why this reads the render and not the template, which is the whole lesson
    ///
    /// Its first version scanned the template source for the word `disabled`, and the code review
    /// defeated it three ways in one afternoon: **an attribute assembled in Rust** and emitted with
    /// `|safe` — an idiom this codebase already uses — put a real native `disabled` in the served
    /// page while the guard stayed green, because the literal never appears in the `.html`; **a bare
    /// uppercase `DISABLED`** slipped past a case-sensitive `match_indices`, and HTML attribute names
    /// are not case-sensitive; and it could not see `tabindex` at all, which is the DOM property the
    /// spelling was only ever a proxy for. 🔑 *A guard that reads the source measures what was
    /// written, never what was served — and the operator gets what was served.*
    ///
    /// 🔴 **The defect it now pins was real and shipped.** Measured in Chrome via CDP at the code
    /// review: `<span role="button" aria-disabled="true">` **with no `tabindex` has `tabIndex -1`,
    /// refuses `.focus()` outright, and forty dispatched Tab presses never reach it** — with the
    /// control that makes it mean something, a `<button>` and a `<span tabindex="0">` both measuring
    /// `tabIndex 0`. The story's justification for choosing `aria-disabled` over `disabled` was
    /// therefore arguing for a property its own markup did not deliver; the validation had measured
    /// a `<button>` and a `<span>` was built.
    ///
    /// ⚠️ **Its limit, written rather than implied**: it reads the rendered STRING. It cannot see an
    /// attribute a script adds later, and it says nothing about what a screen reader actually
    /// announces — only that the control is in the tab order and marked unavailable. An axe-core
    /// pass over the ten routes is the epic's DoD and is registered as unowned.
    #[test]
    fn a_planned_control_is_reachable_and_never_natively_disabled() {
        let html = rendered_triage_body().to_lowercase();

        // Every `role="button"` must carry a `tabindex`, or it is not in the tab order at all.
        let roles = html.matches("role=\"button\"").count();
        assert!(
            roles >= 5,
            "the premise: the bar renders at least the mock's five controls ({roles} seen) — a \
             render that produced none would assert nothing"
        );
        for (at, _) in html.match_indices("role=\"button\"") {
            let element = &html[html[..at].rfind('<').unwrap_or(0)..];
            let element = &element[..element.find('>').unwrap_or(element.len())];
            assert!(
                element.contains("tabindex="),
                "a control with role=\"button\" and no tabindex has tabIndex -1: it refuses focus \
                 and no amount of Tab reaches it, which is exactly the outcome `aria-disabled` was \
                 chosen to avoid (NFR25, measured in Chrome). Element: {element}"
            );
            assert!(
                element.contains("aria-disabled="),
                "a planned control must be ANNOUNCED unavailable, not merely inert: {element}"
            );
        }

        // The native attribute, in any casing, whether or not it carries a value. A bare boolean
        // attribute has no `=`, which the first version of this check also missed.
        for (at, _) in html.match_indices("disabled") {
            let before = &html[..at];
            assert!(
                before.ends_with("aria-") || before.ends_with("hx-"),
                "the rendered page carries a NATIVE `disabled` attribute. It removes the control \
                 from the tab order entirely — the operator is not even told the gesture exists. \
                 Use `aria-disabled` with a `tabindex`"
            );
        }
    }

    /// 🔴 **No gesture's copy names the story that will build it — in either language.**
    ///
    /// *"Arrives in 6.4"* is not information for the operator: it turns the label into a **calendar,
    /// therefore a promise**, which is what story 5.14b refused. Guy's arbitration: the owner lives
    /// in the type and nothing renders it.
    ///
    /// # Why this resolves the key instead of parsing the file
    ///
    /// 🔴 Its first version read `app.yml` line by line and split on `": "`. The code review defeated
    /// it with **a YAML block scalar** — `fr: |` and the sentence on the next line — which that split
    /// never matches, so the guard walked past it: *"Ce geste arrive avec la story 6.4, prevu au
    /// sprint 12"* rendered **on all five controls, in French, with every guard green**. 🔑 The file
    /// has many legal syntaxes for one value and the resolver has one answer; **reading the KEY names
    /// from the file and the VALUES from `t!()` makes the syntax irrelevant.**
    ///
    /// ⚠️ **A digit is the proxy and the proxy is the limit.** A version, a date or a quarter all
    /// carry one; *"bientôt"* does not, and this guard would not catch it. A tripwire against the
    /// ordinary gesture of being helpful, never a barrier against a determined promise.
    #[test]
    fn no_gesture_copy_names_the_story_that_will_build_it() {
        // The key NAMES come from the file — a robust parse, one identifier per line. The VALUES
        // come from the resolver, which is what the operator actually reads.
        let keys: Vec<String> = include_str!("../locales/app.yml")
            .lines()
            .filter_map(|line| line.strip_suffix(':'))
            .filter(|name| name.starts_with("gesture."))
            .map(str::to_string)
            .collect();
        assert!(
            keys.len() >= 8,
            "the premise: the six gestures plus the badge and the sentence is at least eight keys \
             ({} seen) — a scan that found none would assert nothing",
            keys.len()
        );
        let mut checked = 0_usize;
        for key in &keys {
            for locale in ["en", "fr"] {
                let value = rust_i18n::t!(key.as_str(), locale = locale).to_string();
                assert_ne!(&value, key, "{key} does not resolve in {locale}");
                assert!(
                    !value.chars().any(|c| c.is_ascii_digit()),
                    "{key} renders {value:?} in {locale} — a gesture's copy must not name the story \
                     that will build it: a number turns the label into a calendar, therefore a \
                     promise (story 5.14b)"
                );
                checked += 1;
            }
        }
        assert_eq!(
            checked,
            keys.len() * 2,
            "every key must have been checked in BOTH locales"
        );
    }

    /// The bar carries the mock's five controls, all planned, and `Résoudre` follows the CAUSE.
    ///
    /// 🔴 **The primary is chosen from the cause, never from the translated `kind`.** Branching on
    /// the rendered string would be story 6b.3's `role_key: "example.badge"` defect: a real,
    /// resolving, wrong value that every shape and resolvability check passes.
    #[test]
    fn the_bar_shows_five_planned_controls_and_resolve_follows_the_cause() {
        let declared = vec![
            declared_row("drift", "ipv4", "192.0.2.10"),
            declared_row("drift", "hostname", "nas"),
            declared_row("clash", "ipv4", "192.0.2.30"),
            declared_row("clash", "hostname", "either"),
        ];
        let observations = vec![
            batch("unifi", 10, vec![ipv4("192.0.2.10"), hostname("intruder")]),
            batch("unifi", 30, vec![ipv4("192.0.2.30"), hostname("one")]),
            batch("arp", 40, vec![ipv4("192.0.2.30"), hostname("two")]),
        ];
        let pane_for = |sel: &str| {
            build_triage(
                declared.clone(),
                Vec::new(),
                observations.clone(),
                at(1_000),
                Some(sel),
                false,
            )
            .selected
            .expect("the selected row has a pane")
        };

        let gap_row = pane_for("ecart:drift:hostname");
        assert_eq!(
            gap_row.gestures.len(),
            5,
            "the mock's bar carries five controls"
        );
        assert!(
            gap_row.gestures.iter().all(|g| g.not_built.is_some()),
            "not one of the five exists today — every control is planned"
        );

        // 🔴 **This assertion was the ONE test standing between the product and story 6b.10's
        // arbitration 1, and it carried BOTH retired terms in one line**: it read
        // `assert_eq!(…, "Merge", "a drift offers Merge")` — `Merge`, which the binding glossary
        // retires in English by name, asserted under a message saying `drift`, the synonym story
        // 6b.6 retired for `gap`. *A test that pins the ugly thing is a test that requires it*
        // (story 6b.4), and this one required the defect for two stories running.
        //
        // 🔑 It now compares against the RESOLVED KEY, because what this test is about is **which
        // gesture is primary**, never how that gesture is worded. The premise below is what keeps
        // that from being vacuous.
        let documenting = rust_i18n::t!("gesture.document").to_string();
        let resolving = rust_i18n::t!("gesture.resolve").to_string();
        assert_ne!(
            documenting, resolving,
            "the premise: the two gestures are worded differently, or the pair below would pass \
             whichever one the code selected"
        );
        assert_eq!(
            gap_row.gestures[0].label, documenting,
            "a GAP row's primary control is the DOCUMENTING gesture"
        );

        let conflict = pane_for("conflit:clash");
        assert_eq!(
            conflict.gestures[0].label, resolving,
            "a CONFLICT offers Resolve — and the choice comes from the cause, not from the \
             translated kind string"
        );
    }

    /// The rendered bar says the gesture is not built, and says it in the operator's language.
    #[test]
    fn the_rendered_bar_says_the_gesture_is_not_built() {
        let declared = vec![
            declared_row("e1", "ipv4", "192.0.2.10"),
            declared_row("e1", "hostname", "nas"),
        ];
        let view = build_triage(
            declared,
            Vec::new(),
            vec![batch(
                "unifi",
                0,
                vec![ipv4("192.0.2.10"), hostname("other")],
            )],
            at(600),
            None,
            false,
        );
        let html = TriageBody {
            triage: view,
            identity: no_reach(),
            s: strings(),
        }
        .render()
        .expect("the triage body renders");

        assert!(html.contains("action-bar"), "the bar renders");
        assert_eq!(
            html.matches("aria-disabled=\"true\"").count(),
            5,
            "all five controls are announced as disabled without leaving the tab order"
        );
        // ⚠️ The native attribute in ANY casing and with or without a value — a bare boolean
        // attribute has no `=`, which this check missed until the code review planted `DISABLED`.
        // The dedicated guard above owns this property; the duplication is deliberate, so a reader
        // of THIS test is not left believing the render is unchecked.
        assert!(
            !html.to_lowercase().contains("<button"),
            "never a <button>, which would carry the native attribute by habit"
        );
        assert_eq!(
            html.matches("aria-describedby=\"gesture-not-built\"")
                .count(),
            5,
            "every control POINTS at the one sentence, so a screen reader announces it per control \
             without five visible copies — seen in a browser, repeating it inside each control \
             turned a compact row into a stack that said the same thing five times"
        );
        assert!(
            html.contains(
                "id=\"gesture-not-built\" class=\"gesture-note\">This gesture is not built yet"
            ),
            "the sentence is VISIBLE TEXT, not a `title=` — it sat in a tooltip until the code \
             review, invisible to a keyboard and to a touch screen, while the whole argument for \
             showing a dead control is that it TELLS the operator why"
        );
        assert!(
            html.contains("Accept the gap") && html.contains("Exclude"),
            "the four Epic 7 gestures are VISIBLE and labelled — Guy's premise (2) of 2026-08-13"
        );
    }

    // ── Story 6b.5: the dashboard, real reach beside labelled examples ──

    /// The dashboard's body, rendered on a store that has seen something.
    fn rendered_dashboard(last: Option<chrono::DateTime<chrono::Utc>>) -> String {
        // 🔴 `"match"`, not `"matched"` — the first draft of this fixture used the wrong token, it
        // fell into `build_identity_view`'s unknown-outcome arm, and **`placed` rendered as 0**.
        // ⚠️ And the counts are 11 and 6 on purpose: their sum, 17, collides with none of the
        // example cards' values (37, 4, 2), which is what the first draft's `2` did — it was
        // satisfied by a stat card and asserted nothing about the real half at all.
        let identity = build_identity_view(vec![
            reach("match", None, 11),
            reach("abstained", Some("absence_of_proof"), 6),
        ]);
        let view = build_dashboard(identity, last, at(600));
        DashboardBody { view, s: strings() }
            .render()
            .expect("the dashboard body renders")
    }

    /// 🔴 **The two populations are never summed — and this guard sits at the COMPOSITION.**
    ///
    /// # Why here and not on either builder, which is the whole point
    ///
    /// Story 5.14b wrote this guard over the two pure builders and **its own mutation pass measured
    /// it GREEN**: it asserted that `build_view` and `build_identity_view` do not add each other's
    /// counts, and **neither of them can**, since neither sees the other's numbers. *A guard placed
    /// where the defect cannot occur reads as coverage and is none* — the epic's dominant class, and
    /// its cleanest specimen. Story 6b.5's validation reproduced both halves by building them: the
    /// builder-level guard stays green under a sum planted at the composition; the same guard
    /// written HERE catches it (`left: 2, right: 3`).
    ///
    /// 🔑 This is where the two meet, so this is where a sum could be written — and the assertion is
    /// over the RENDERED body, because the operator reads the render and not the struct.
    ///
    /// ⚠️ Arbitration 13's UNIT is inherited rather than restated: the dashboard **includes**
    /// `_identity_section.html`, so the *"counts sightings, not devices"* note is the same one
    /// `/triage` renders and the same guard covers it. **Hand-roll that markup and this inheritance
    /// silently ends.**
    #[test]
    fn the_dashboard_never_sums_the_two_populations() {
        let html = rendered_dashboard(Some(at(0)));
        // 🔑 Scoped to the REAL half. Asserted over the whole page, a bare `>11<` could be satisfied
        // by an example card — which is exactly how the first version of this guard passed while
        // measuring nothing.
        let real = html
            .split("class=\"dashboard-real\"")
            .nth(1)
            .and_then(|rest| rest.split(DASHBOARD_EXAMPLE_ANCHOR).next())
            .expect("the real section renders before the example ones");

        assert!(real.contains(">11<"), "the placed count renders: {real}");
        assert!(real.contains(">6<"), "the not-placed count renders: {real}");
        // 🔴 Their sum must appear NOWHERE on the page. A `17` could only have been written by
        // adding two populations the product deliberately keeps apart (arbitration 10).
        // ⚠️ Scoped to the REAL half, like its two siblings. Over the whole page this was
        // fixture-coupled — any unrelated `17` would have reddened it — and it is the real half
        // that could carry a sum at all.
        assert!(
            !real.contains("17"),
            "the two populations were SUMMED: 11 + 6 reached the page as 17. They count different \
             things and adding them invents a number the product does not hold (arbitration 10)"
        );
        // And the unit, inherited from the included partial rather than restated here.
        assert!(
            html.contains("sightings, not devices"),
            "the honest unit must reach the page — and it does so because the dashboard INCLUDES \
             `_identity_section.html`; duplicating that markup would end this inheritance silently"
        );
    }

    /// 🔴 **Every example section carries its OWN marker — the sibling the route table cannot be.**
    ///
    /// # Why a second guard, measured rather than argued
    ///
    /// The route-table partition asserts `body.contains("example-marker-badge")`, which is a
    /// property of the whole response. Story 6b.5's validation built a two-example-section body and
    /// dropped the marker from **one**: the screen-level oracle **stayed GREEN**, because the other
    /// section still carries the string; a section-level oracle reds `left: 2, right: 1`. 🔑 *The
    /// partition is a property of the ROUTE TABLE and this is a property inside one BODY — it cannot
    /// be extended, only joined.*
    ///
    /// ⚠️ Its limit: it counts sections and markers on the rendered page. It cannot say the marker is
    /// on the RIGHT section, only that no example section is missing one — and it says nothing about
    /// whether the boundary is legible, which is why the template also carries a heading per section.
    #[test]
    fn every_example_section_carries_its_own_marker() {
        let html = rendered_dashboard(Some(at(0)));
        // 🔴 PER SECTION, and the totals form was measured worthless. This guard compared
        // `markers == sections` until the code review planted **two** markers in the first example
        // section and **none** in the second — net two and two — and **the entire suite stayed
        // green, this guard included**. *It could not tell "each section has exactly one" from
        // "they happen to add up."* A FOURTH occurrence of the class it was written to close.
        let sections: Vec<&str> = html.split(DASHBOARD_EXAMPLE_ANCHOR).skip(1).collect();
        assert!(
            sections.len() >= 2,
            "the premise: the dashboard carries at least two example sections ({}) — with one, \
             this guard cannot tell a per-section rule from a per-screen one",
            sections.len()
        );
        for (index, section) in sections.iter().enumerate() {
            // Each slice runs to the next example section, so a marker cannot be counted twice.
            let body = section
                .split(DASHBOARD_EXAMPLE_ANCHOR)
                .next()
                .unwrap_or(section);
            assert_eq!(
                body.matches("example-marker-badge").count(),
                1,
                "example section {index} must carry EXACTLY ONE marker. The route-table partition \
                 cannot catch a missing one (measured: it stays green), and a totals check cannot \
                 catch a misplaced one (measured: two here and none there stays green too)"
            );
        }
        // And the real half must NOT be marked: that is the other direction of the same rule.
        let real = html
            .split("class=\"dashboard-real\"")
            .nth(1)
            .and_then(|rest| rest.split(DASHBOARD_EXAMPLE_ANCHOR).next())
            .expect("the real section renders first");
        assert!(
            !real.contains("example-marker-badge"),
            "the REAL section carries the marker: the product would be calling the operator's own \
             counts a demonstration"
        );
    }

    /// 🔴 **`build_dashboard` reads no clock of its own**, and this guard is written rather than
    /// inherited.
    ///
    /// Story 5.14b's `the_view_builder_has_no_clock_so_one_store_renders_identically` hands
    /// `build_view` EMPTY inputs, so its populated branch is never reached — story 6b.4's review
    /// measured that. And the carrier everyone assumed, `chrono`'s `default-features = false`, only
    /// stops one SPELLING: `std::time::SystemTime::now()` compiles freely. *A feature flag guards a
    /// name, never the act of reading a clock.*
    #[test]
    fn build_dashboard_reads_no_clock_of_its_own() {
        // `"match"` — the engine's token. This read `"matched"` until the code review, sixty lines
        // below the comment explaining that exact mistake: the fix had been patched at the one site
        // a mutation happened to hit, not closed.
        let identity = build_identity_view(vec![reach("match", None, 1)]);
        let build = |now| build_dashboard(identity.clone(), Some(at(0)), now);

        let a = build(at(600));
        let b = build(at(600));
        assert_eq!(
            a.last_observed, b.last_observed,
            "one store, one instant, twice — the builder must be a pure function of its inputs"
        );
        let later = build(at(7_200));
        assert_ne!(
            a.last_observed, later.last_observed,
            "the freshness must follow the `now` the caller supplied — if it does not, this guard \
             measures nothing and a clock inside the builder would be invisible to it"
        );
    }

    /// The last observation renders in the BODY, and the shell never learns of it.
    ///
    /// 🔑 The converse was measured at validation: the same text placed in the body **cannot** trip
    /// `the_shell_shows_no_last_observation`, because the body arrives as `{{ body|safe }}` and that
    /// guard's self-widening loop follows only `{% include %}` directives inside the two frame
    /// files. The two facts are independent, which is why both are asserted.
    #[test]
    fn the_last_observation_renders_in_the_body_and_says_so_when_there_is_none() {
        let seen = rendered_dashboard(Some(at(0)));
        assert!(
            seen.contains("Last observed") && seen.contains("10 min ago"),
            "the last observation renders from the stored instant"
        );
        let never = rendered_dashboard(None);
        assert!(
            never.contains("Nothing has been observed yet"),
            "a store that has seen nothing SAYS so rather than rendering an empty label"
        );
        assert!(
            !never.contains("Last observed"),
            "and it does not render the label with nothing after it"
        );
    }

    /// 🔴 **A scan that has landed but not been resolved is SAID, not left as a contradiction.**
    ///
    /// # The state this story created, and why no test could see it
    ///
    /// `count_engine_reach` and `last_observed_at` are two populations, and story 6b.5 is what put
    /// them in one div for the first time. They can legitimately disagree: an observation ingested
    /// and not yet resolved leaves the reach empty while `MAX(observed_at)` is recent, and the page
    /// then read *"Nothing observed yet — run a scan"* directly above *"Last observed 8 h ago"*.
    /// ⚠️ **Every other test in this file feeds the two from ONE fixture** — the same synthetic
    /// identity beside the same instant — so the divergence was unreachable by the whole suite until
    /// the code review seeded them independently against a live database.
    #[test]
    fn a_scan_that_has_landed_unresolved_is_said_rather_than_contradicted() {
        // Reach EMPTY, an observation RECENT — the ordinary state between an ingest and the pass.
        let unresolved = build_dashboard(build_identity_view(Vec::new()), Some(at(0)), at(600));
        assert!(
            unresolved.pending_resolution,
            "reach empty beside a real instant is the state that needs saying"
        );
        let html = DashboardBody {
            view: unresolved,
            s: strings(),
        }
        .render()
        .expect("the dashboard renders");
        assert!(
            html.contains("A scan has landed"),
            "the page must reconcile the two, or it says nothing was observed above the instant \
             proving something was: {html}"
        );

        // And it must NOT say it when the two agree, in either direction.
        let resolved = build_dashboard(
            build_identity_view(vec![reach("match", None, 1)]),
            Some(at(0)),
            at(600),
        );
        assert!(
            !resolved.pending_resolution,
            "a store whose reach is populated is not pending resolution"
        );
        let empty = build_dashboard(build_identity_view(Vec::new()), None, at(600));
        assert!(
            !empty.pending_resolution,
            "a store that has observed NOTHING is not pending resolution either — it is empty, and \
             the included section already says so"
        );
    }

    /// 🔴 **The marker's scope follows the screen's NATURE, and every example section is covered
    /// exactly once — counted ONE AT A TIME.**
    ///
    /// # Two rules, because there are two natures
    ///
    /// - an [`Example`](crate::screens::Nature::Example) screen is example all the way through, so
    ///   the smallest unit that is ENTIRELY example is the SCREEN: **exactly one marker**. The
    ///   record shipped **four identical banners down one page** before a browser showed it —
    ///   story 6b.4b's finding reproduced, and no guard could see it because every one of them
    ///   asks whether the marker is PRESENT;
    /// - a [`Mixed`](crate::screens::Nature::Mixed) screen carries real content beside example
    ///   content, so the unit is the SECTION: **one marker per example section**, which is story
    ///   6b.5's rule and which a screen-level marker would break in both directions.
    ///
    /// 🔑 **Per section, never in total.** Story 6b.5 measured that a totals comparison cannot tell
    /// *"each section has one"* from *"they happen to add up"* — two markers in one section and
    /// none in the other left its whole suite green, that guard included.
    ///
    /// ⚠️ It reads the COMPOSED body: the marker is emitted by the dispatch in
    /// [`crate::screens`], so a guard over `inventory_body` alone would find none at all.
    #[test]
    fn every_example_section_is_covered_by_exactly_one_marker() {
        // 🔴 **The badge's CLASS, not its text.** Counting the word was a plain substring count
        // with no delimiters: the edge-case review measured an unresolved key `Example.Kind.bogus`
        // inflating the count to 9 across eight device rows, because the untranslated badge reads
        // *"Example"*. A false positive there is a false NEGATIVE waiting on any defect whose text
        // does not collide. *An oracle that counts a word counts every word that contains it.*
        let badge = r#"class="example-marker-badge""#;

        // 🔴 **WHAT THIS LOOP OWNS SINCE STORY 6b.7, and what it no longer claims.** AC6 of that
        // story named this guard as one of three enumerations to turn into a property, and the
        // review found that it was **not converted**: the page-level *exactly one marker* rule is
        // now asserted over `Screen::ALL` on the real HTTP body, in `main.rs`'s route-table guard,
        // which is where a screen this module cannot build is finally covered.
        //
        // 🔑 **This loop is kept anyway, and NOT as a duplicate**: the assertion below it —
        // *the body carries no marker of its own* — is a property the route-table guard cannot
        // express, because it sees only the SERVED page, where a template-side marker and the
        // dispatch's are indistinguishable. Two markers stacking is what the route-table count
        // catches; a marker that has MOVED from the dispatch into a template is what this catches,
        // and that move is silent until a second screen forgets to make it.
        //
        // ⚠️ The enumeration below is therefore deliberate and narrow: two bodies this module can
        // build directly. It is **not** a claim of coverage over the ten screens, and reading it as
        // one is what the review corrected.
        for (screen, body) in [
            (
                "/devices",
                crate::example_screens::inventory_body(&Default::default()),
            ),
            (
                "/devices/{id}",
                crate::example_screens::record_body("nas-01").expect("nas-01 is an example device"),
            ),
        ] {
            let served = format!("{}{}", example_marker(), body);
            let markers = served.matches(badge).count();
            assert_eq!(
                markers, 1,
                "{screen} is example all the way through and must carry exactly ONE marker; it \
                 carries {markers}. Four identical banners down one page is what a per-section \
                 marker produced on the record, and only a browser showed it"
            );
            assert!(
                !body.contains(badge),
                "{screen}'s own body carries a marker as well as the dispatch's — the two stack, \
                 and the dispatch is where it belongs so a new screen cannot omit it"
            );
        }

        let dashboard = rendered_dashboard(Some(at(0)));
        let mut sections = 0_usize;
        for fragment in dashboard.split(EXAMPLE_SECTION_ANCHOR).skip(1) {
            let section = fragment
                .split_once(EXAMPLE_SECTION_ANCHOR)
                .map_or(fragment, |(head, _)| head);
            let markers = section.matches(badge).count();
            assert_eq!(
                markers, 1,
                "an example section of /dashboard carries {markers} marker(s) and must carry \
                 exactly one — counted per SECTION, because a total cannot tell \"each has one\" \
                 from \"they add up\""
            );
            sections += 1;
        }
        assert_eq!(
            sections, 2,
            "the premise: the dashboard has two example sections ({sections} seen)"
        );
    }
}
