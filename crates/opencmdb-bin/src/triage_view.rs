//! The triage screen's view model, built from the real gap — split out of `page.rs` by story 6.14.
//!
//! `page.rs` stood at 1954 of the 2000 code lines the `file-size` gate allows, and story 6.14 adds a
//! fifth row kind and a candidates pane. The split is behaviour-neutral and was taken in its own
//! commit: every item here moved unchanged but for its visibility, and `page.rs` re-exports the
//! module so its existing paths still resolve.

use opencmdb_core::observation::Observation;
use opencmdb_core::{AbstentionCause, reconcile};

use crate::page::{
    counted_fields, display_fact, hostname_of, in_perimeter, observation_from_batch,
};

// ── Story 6b.4: the triage screen, on the real gap ───────────────────

/// Where a value came from and how fresh it is — one discreet line under each photo.
///
/// 🔑 **Both sides carry one**, which is AC1's *"neither side is the truth"* made structural: an
/// observation can be stale or from a blind source, and a declaration can be outdated. A pane
/// without its meta-line invites the reader to treat that side as fact.
pub(crate) struct MetaLine {
    /// The source — a connector for the observed side, an origin for the declared one.
    pub(crate) source: String,
    /// How long ago, already rendered in the operator's language.
    pub(crate) freshness: String,
}

/// One row of the triage queue.
pub(crate) struct QueueRow {
    /// The stable selector this row is addressed by (`?sel=`).
    pub(crate) id: String,
    /// The row's own link, with the selector ESCAPED and the sort preserved.
    ///
    /// 🔴 Built here rather than in the template. `url_escape` existed from the first draft and was
    /// applied only to the sort toggle, while the queue link wrote `?sel={{ row.id }}` raw — Askama
    /// escapes HTML, never a query string. ⚠️ **Measured NOT reachable in production** (`entity_id`
    /// is a `Uuid::now_v7()` at every call site and no operator path chooses it), so this was a
    /// latent inconsistency and not a live defect — fixed because the function's own doc already
    /// warns that an attribute key is operator-supplied, and the day one is, this is where it lands.
    pub(crate) href: String,
    /// The row's kind, in the operator's language.
    pub(crate) kind: String,
    /// The entity the row is about.
    pub(crate) entity: String,
    /// The instant of the sighting that supplied this row — the ordering key the overwrite above
    /// compares, so the queue does not depend on the order the store hands its rows in.
    pub(crate) observed_at: chrono::DateTime<chrono::Utc>,
    /// The name the entity answers to, when the sighting carries one; empty otherwise.
    ///
    /// 🔑 **This is what a queue of addresses is missing, and it is display ONLY.** The row is
    /// still addressed by `?sel=nouveau:{ipv4}` and the entity is still the address: a name is
    /// what the network calls a host today, and this product does not let a DHCP-assigned name
    /// become an identity. Empty for a host with no PTR record, which on the reference network is
    /// 32 addresses out of 69 — so the column has to read well half-empty.
    pub(crate) name: String,
    /// The field, for a row the engine names one for; empty otherwise.
    pub(crate) field: String,
    /// The declared value, when there is one.
    pub(crate) declared: String,
    /// The observed value, when there is one.
    pub(crate) observed: String,
    /// How many fields this row stands for, WITH its unit — FR16b's *"one line and one gesture,
    /// not N failures"*. ⚠️ A bare number beside an address reads as noise; measured by looking.
    pub(crate) count: String,
    /// Whether this is a cause row, and therefore whether `count` is worth showing.
    pub(crate) counted: bool,
    /// The observed side's freshness, right-aligned as the mock puts it.
    pub(crate) seen: String,
    /// Seconds since the newest in-perimeter observation — the sort key, never displayed.
    pub(crate) age_seconds: i64,
    /// Whether this row is the selected one.
    pub(crate) selected: bool,
}

/// What a control on the action bar IS.
///
/// # Three variants, and the second and third arrived by the mechanism the first bought
///
/// 🔑 **Guy's arbitration (2026-08-19), taken over a struct carrying an `Option` route**, and taken
/// for a reason the validation established by BUILDING both: with `Planned` alone there is no
/// unconstructed variant, so `clippy -D warnings` was clean — and **the day story 6.4 added
/// `Live`, `E0004` forced every `match` on this type to be revisited.** That is a compiler-forced
/// moment of attention at exactly the moment it is worth having, and the struct shape has none.
/// It paid a second time the same day: `Disabled` reopened every arm again.
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
    /// It acts. The variant CARRIES its route, so a live gesture that posts nowhere is
    /// unrepresentable.
    ///
    /// 🔴 **Adding this is what story 6b.4b bought with its single-variant enum** — `E0004` fires
    /// the moment a second arm exists. ⚠️ The validation measured that it fires at exactly **one**
    /// site, not on *"every match"* as that story's record reads, and — more usefully — that
    /// satisfying it naively ships an inert `<span class="btn-gesture live">` with no route and no
    /// method. *A compile error a span can silence is not the guard that story sold*, which is why
    /// the route lives in the type rather than in whoever writes the template.
    ///
    /// _(This doc block was inserted INSIDE `Planned`'s at story 6.4's implementation, so `Live`
    /// opened on a sentence about a field it does not have and `Planned` was left undocumented.
    /// Found by two review layers independently — the mechanical tell of an insertion landing
    /// before the wrong item, which silently disabled a test in story 6b.2.)_
    Live {
        /// Where the control posts. A constant of the product, never operator input.
        route: &'static str,
    },
    /// The product does not have this gesture yet.
    ///
    /// ⚠️ `owner` lives HERE and never on the screen: *"arrives in 6.4"* would turn the label into a
    /// calendar, therefore a promise, which story 5.14b refused. Nothing reads it, and `dead_code`
    /// does not red on it — measured, with the `#[derive(Debug)]` hypothesis tested and refuted.
    Planned {
        /// The story that will build it.
        owner: &'static str,
    },
    /// 🔴 **BUILT, and switched off on this instance** — Guy's arbitration of 2026-08-26, taken at
    /// the code review over hiding the control and over leaving it as it was.
    ///
    /// The state exists because story 6.4 created a falsehood: with the route unmounted the
    /// documenting control rendered `Planned`, so the product said *"not built yet"* about a
    /// gesture it had just built — **on the default configuration, which is what nearly every
    /// deployment runs.** Hiding it was refused as contradicting Guy's premise (2) of 2026-08-13
    /// (*show and label rather than hide*): a fresh install would never learn the product can
    /// document at all.
    ///
    /// 🔑 It carries the SWITCH, so the sentence on screen and the variable the binary reads are
    /// one constant — the [`crate::DOCUMENT_ENABLED_ENV`] idiom that already keeps the route's
    /// path from drifting from the router's.
    Disabled {
        /// The environment variable that turns it on. A constant of the product.
        switch: &'static str,
    },
    /// 🔑 **One of the two answers of `resolve`/*résoudre*** — story 6.14b, and the FOURTH variant this
    /// type's `E0004` forced every `match` to answer.
    ///
    /// 🔴 **A variant of its own and NOT `Live`**, measured by the validation (§0.9(C)): `Live` carries
    /// ONE subject through `hx-vals` and paints the reserved amber `btn-document`, so reusing it raised
    /// no compile error, posted an empty subject, painted both answers amber and was read by both
    /// browser gates as a documenting gesture. The amber stays the documenting gesture's alone.
    Answer {
        /// Where the answer posts. A constant of the product, never operator input.
        route: &'static str,
        /// Which of the two answers this control gives.
        answer: crate::l2_answer::Answer,
    },
}

/// One control of the action bar, resolved for rendering.
#[derive(Debug)]
pub(crate) struct GestureView {
    /// Its label, in the operator's language.
    pub(crate) label: String,
    /// The sentence shown when the gesture is not built — `None` once it is.
    ///
    /// 🔑 **ONE field, and the template branches on it**, so the rendered state cannot disagree with
    /// the nature: there is no second *"is it live"* flag to drift out of step. It is produced by a
    /// `match` on [`Gesture`], so every variant added there reopens this one — which is what
    /// happened twice on 2026-08-25/26, for `Live` and then for `Disabled`.
    pub(crate) nature: GestureRender,
}

/// The mock's action bar: five controls, in its order — ONE of which acts, since story 6.4.
///
/// 🔴 **`primary` is chosen from the row's CAUSE, never from its translated label.** The mock shows
/// *Résoudre* on a conflict and *Merger* elsewhere; branching on `DetailPane::kind` — which is
/// already translated — would reproduce story 6b.3's `role_key: "example.badge"` defect, a real,
/// resolving, wrong value that every shape and resolvability check passes.
/// What the bar's PRIMARY control is, at the one place that knows.
///
/// 🔴 **It replaces a `bool`, and the bool was conflating two different facts.** `primary_is_live`
/// meant *"the route is mounted"* at one call site and *"this kind of row is eligible"* at two
/// others — harmless while every `false` rendered the same *not built* control, and false the
/// moment story 6.4's code review gave the switched-off state its own words: an `Écart` pane would
/// have told the operator to set `OPENCMDB_DOCUMENT_ENABLED` for a gesture that is enabled and
/// simply does not apply to that row.
///
/// 🔑 The decision still lives INSIDE [`action_bar`], never at the call sites: a caller that could
/// hand in a `Gesture` could hand `Live` to `gesture.resolve`, and the amber's reservation would go
/// back to holding by a sentence about which primary happens to be live.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrimaryState {
    /// The documenting gesture, on a row it applies to, with the route mounted.
    Acts,
    /// The documenting gesture, on a row it applies to, with the route NOT mounted.
    SwitchedOff,
    /// A gesture the product has not built — whatever the configuration says.
    NotBuilt,
}

pub(crate) fn action_bar(primary_key: &'static str, primary: PrimaryState) -> Vec<GestureView> {
    // ⚠️ The primary's owner is Epic 7's wherever it does not act: the documenting gesture at FIELD
    // level — FR13(b) — on a gap, an absence or a conflict. 🔴 This read `"gesture.resolve" => "6"`
    // until story 6.14, because a CONFLICT carried *Résoudre* after the mock. Guy's decision B moved
    // *Résoudre* to the AMBIGUITY, whose pane builds its own bar (`ambiguity_view::resolve_bar`), and
    // gave the conflict the documenting gesture — so no caller passes `gesture.resolve` here now.
    let primary_owner = "7";
    let mut bar = planned_gestures(&[
        ("gesture.accept_gap", "7"),
        ("gesture.snooze", "7"),
        ("gesture.attach", "7"),
        ("gesture.exclude", "7"),
    ]);
    // 🔑 The primary is built SEPARATELY because it is the only one that can act, and it is
    // INSERTED at the front rather than pushed — the mock's order is primary first, and story
    // 6b.11's roving-tabindex contract makes that order the keyboard's order too.
    // 🔴 **Only the DOCUMENTING gesture can be live, and that is closed here rather than
    // asserted.** The UX spec reserves the amber `--accent-document` *solely for the documenting
    // gesture* (`:587`) and prescribes *one primary action (amber Document)* (`:1174`). If a
    // `Live` control could be either gesture, the reservation would hold by a SENTENCE about
    // which primary happens to be live — and `Résoudre` is the one Epic 6 will make live. Here
    // `GestureRender::Live` IMPLIES *documenting*, so the template can carry `btn-document` as a
    // static class literal and the amber cannot leak. ⚠️ The day `Résoudre` acts, this `match`
    // is what makes someone decide what colour it wears.
    bar.insert(
        0,
        GestureView::of(
            match (primary, primary_key) {
                (PrimaryState::Acts, "gesture.document") => Gesture::Live {
                    route: crate::document::DOCUMENT_ALL_PATH,
                },
                // 🔴 BUILT AND SWITCHED OFF, never *"not built yet"*. Story 6.4 shipped this arm
                // as `Planned`, so on the default configuration the product said the documenting
                // gesture did not exist — about the gesture it had just built. Guy's arbitration,
                // 2026-08-26, at the code review.
                (PrimaryState::SwitchedOff, "gesture.document") => Gesture::Disabled {
                    switch: crate::DOCUMENT_ENABLED_ENV,
                },
                _ => Gesture::Planned {
                    owner: primary_owner,
                },
            },
            rust_i18n::t!(primary_key).to_string(),
        ),
    );
    bar
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
            GestureView::of(
                Gesture::Planned { owner },
                rust_i18n::t!(*label_key).to_string(),
            )
        })
        .collect()
}

/// What the template must render for one control, with the impossible pairs unrepresentable.
///
/// 🔑 **One field rather than two `Option`s** (story 5.6's idiom, closed in the TYPE). The pair
/// `not_built: Option<String>` + `post_to: Option<&str>` admits four states of which two are
/// nonsense — *planned yet posting somewhere*, and *live with a note and no route* — and the
/// second forces an `unwrap()` in the template, i.e. a **500 carried by a sentence**. Here the
/// template matches ONCE and every arm has exactly what it needs.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum GestureRender {
    /// Not built: the sentence saying so, shown once for the group and announced on each control.
    Planned(String),
    /// Live: where it posts. A constant of the product, never operator input.
    Live(&'static str),
    /// Built and switched off: the sentence saying so, which NAMES the switch that turns it on.
    ///
    /// ⚠️ A separate arm rather than a second `Planned` with different words, because the two
    /// states are answered by different acts: one waits for a story, the other for one line of
    /// configuration the operator already controls.
    Disabled(String),
    /// One answer of `resolve` (story 6.14b): where it posts, the answer's form token, and its
    /// accessible name — the visible label first, then the question it answers, so two answers on two
    /// panes never share one name.
    Answer {
        /// Where it posts.
        route: &'static str,
        /// `same` or `distinct`.
        token: &'static str,
        /// The accessible name.
        name: String,
    },
}

impl GestureView {
    /// One control, its nature, and everything the template needs to render it.
    pub(crate) fn of(gesture: Gesture, label: String) -> Self {
        let nature = match gesture {
            Gesture::Planned { .. } => GestureRender::Planned(
                rust_i18n::t!("gesture.not_built", badge = rust_i18n::t!("gesture.badge"))
                    .to_string(),
            ),
            Gesture::Live { route } => GestureRender::Live(route),
            Gesture::Disabled { switch } => GestureRender::Disabled(
                rust_i18n::t!("gesture.not_enabled", switch = switch).to_string(),
            ),
            Gesture::Answer { route, answer } => GestureRender::Answer {
                route,
                token: answer.token(),
                name: label.clone(),
            },
        };
        Self { label, nature }
    }

    /// One answer of `resolve` on the question `group` — its label from the answer's key, its
    /// accessible name naming the question too.
    pub(crate) fn answer(answer: crate::l2_answer::Answer, group: &str) -> Self {
        let label = rust_i18n::t!(answer.label_key()).to_string();
        let mut view = Self::of(
            Gesture::Answer {
                route: crate::l2_answer::ANSWER_PATH,
                answer,
            },
            label.clone(),
        );
        if let GestureRender::Answer { name, .. } = &mut view.nature {
            *name = rust_i18n::t!("triage.answer.name", answer = label, group = group).to_string();
        }
        view
    }

    /// Is this control one the product has not built? The generic *not built* sentence is rendered
    /// only when one is on the bar — story 6.14b made a bar whose every control acts, and the note
    /// beside live answers would be a shipped sentence made false (story 6.4's defect).
    pub(crate) fn is_planned(&self) -> bool {
        matches!(self.nature, GestureRender::Planned(_))
    }

    /// Is this control built but switched off? The template needs it to decide whether the
    /// group's *"not enabled"* sentence is rendered at all — a note about a state no control is
    /// in would be a false line, and this project counts a false line as a defect.
    pub(crate) fn is_disabled(&self) -> bool {
        matches!(self.nature, GestureRender::Disabled(_))
    }
}

impl DetailPane {
    /// Is any control on this pane's bar BUILT and switched off?
    ///
    /// 🔑 It lives here rather than as a closure in the template because **askama's expression
    /// grammar has no closures** — measured: `gestures.iter().any(|g| …)` fails to parse with
    /// *"matching closing `)` is missing"*. The template asks a question; Rust answers it.
    ///
    /// ⚠️ The group's *"not enabled"* sentence is rendered only when this is true. A note about a
    /// state nothing on screen is in is a false line — which is exactly how the neighbouring
    /// *"not built yet"* sentence came to be wrong the day one control went live.
    pub(crate) fn has_a_switched_off_gesture(&self) -> bool {
        self.gestures.iter().any(GestureView::is_disabled)
    }

    /// Is any control on this pane's bar not built yet? The *not built* note is rendered only then.
    pub(crate) fn has_a_planned_gesture(&self) -> bool {
        self.gestures.iter().any(GestureView::is_planned)
    }
}

/// What an Ambigu pane's answers post besides the answer itself (story 6.14b, AC2): the question's
/// row id, its member interfaces, and the instant the page showed as its freshness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct QuestionForm {
    /// `ambigu:{smallest member}` — the queue row's id.
    pub(crate) group: String,
    /// The member interface ids, comma-separated.
    pub(crate) members: String,
    /// The newest candidate freshness, RFC 3339 with microseconds — the instant the answer is dated
    /// at and the stale-page guard (§0.8 C1).
    pub(crate) shown: String,
}

/// The detail pane: the two photos, side by side, each with its own meta-line.
pub(crate) struct DetailPane {
    /// The action bar — the mock's five controls, and what each of them is.
    pub(crate) gestures: Vec<GestureView>,
    /// The selected row's kind.
    pub(crate) kind: String,
    /// The entity.
    pub(crate) entity: String,
    /// The field, when the row names one.
    pub(crate) field: String,
    /// The declared value.
    pub(crate) declared: String,
    /// Where the declared value came from and when it was written.
    pub(crate) declared_meta: MetaLine,
    /// The observed value.
    pub(crate) observed: String,
    /// Which connector reported it and when.
    pub(crate) observed_meta: MetaLine,
    /// The observation a documenting gesture would act on — empty when the pane carries none.
    ///
    /// 🔴 **Story 6.4 added it, and its absence was the story's one open question.** The queue
    /// named an ADDRESS and the route takes an `ObsId`; nothing on the page carried one, which
    /// the validation confirmed on the wire. ⚠️ It is the MOST RECENT sighting of that address —
    /// Guy's arbitration — so the gesture writes what the network shows now, not the first thing
    /// it ever showed.
    pub(crate) subject: String,
    /// The candidates of an Ambigu row — two observed interfaces or more, and NO declared side
    /// (story 6.14). Empty on every other kind, where the two photos stand instead. 🔴 An ambiguity
    /// rendered in the two-photo shape put one candidate under a *Declared* heading — a false
    /// heading no guard could see (the validation's prototype).
    pub(crate) candidates: Vec<crate::ambiguity_view::Candidate>,
    /// One sentence per verdict that argued, for an Ambigu row; empty otherwise.
    pub(crate) evidence: Vec<String>,
    /// For a `Nouveau` row whose address a candidate of an open question carries: the link to that
    /// question. Since story 6.14b's E1 it stands INSTEAD of *Ajouter* while the question is open (story
    /// 6.14's decision C kept *Ajouter* beside it; E1 superseded it).
    pub(crate) open_question: Option<String>,
    /// The sentence under this pane's bar when it is not the generic one. Empty everywhere since story
    /// 6.14b, whose Ambigu pane carries live answers and no planned control; kept as the seam a future
    /// pane-specific *not built* sentence uses rather than re-deriving it.
    pub(crate) not_built: String,
    /// An open question's answer form (story 6.14b) — `None` on every other kind, and on a question
    /// none of whose candidates is placed now, which offers no answer (AC5).
    pub(crate) question: Option<QuestionForm>,
    /// For an Ambigu pane whose group none of whose candidates is placed now: say why no answer is
    /// offered (story 6.14b, AC5, Guy 2026-09-26).
    pub(crate) no_placement: bool,
    /// The operator's earlier answers touching this group, one sentence each (story 6.14b, Guy
    /// 2026-09-26): a group that re-formed around an answered pair names that answer.
    pub(crate) earlier: Vec<String>,
    /// Whether *the same machine* is withheld because the group holds a pair the operator already
    /// answered distinct (Guy, 2026-09-26, PR #220's review); the pane says why.
    pub(crate) only_distinct: bool,
}

/// Everything `/triage` renders: the queue, the selection, and the sort's state.
pub(crate) struct TriageView {
    /// The queue, already ordered.
    pub(crate) rows: Vec<QueueRow>,
    /// The selected row's two photos, when a row is selected.
    pub(crate) selected: Option<DetailPane>,
    /// How many rows the queue holds.
    pub(crate) total: usize,
    /// Whether the age sort is on. ⚠️ **Off by default** — AC3, and the ban is not that age is
    /// hidden but that it is never brandished.
    pub(crate) sort_by_age: bool,
    /// The href that toggles the sort, preserving the selection.
    pub(crate) sort_href: String,
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
/// The view, told whether `POST /document-all` is mounted.
///
/// 🔴 **The page could not know this before story 6.4**, and the validation measured what that
/// cost: a wiring that rendered the gesture regardless put a control the operator can see, focus
/// and press in front of a **404**, on the DEFAULT configuration. `false` keeps every control
/// planned, which is what an un-mounted route and every non-`Nouveau` kind both mean.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_triage_offering(
    declared: Vec<(String, String, String)>,
    provenance: Vec<crate::repo::DeclaredProvenance>,
    observations: Vec<crate::repo::ObservedBatch>,
    now: chrono::DateTime<chrono::Utc>,
    selected: Option<&str>,
    sort_by_age: bool,
    document_enabled: bool,
    ambiguity: &crate::ambiguity_view::AmbiguityInput,
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

    let obs: Vec<Observation> = observations.iter().map(observation_from_batch).collect();

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
        // The name comes from the SAME sighting as the freshness and the source above. An entity
        // nothing has been seen for shows none — never the name it answered to last month.
        let observed_name = newest
            .and_then(|b| hostname_of(&b.facts))
            .unwrap_or_default();

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
                observed_at: newest.map_or(chrono::DateTime::UNIX_EPOCH, |b| b.observed_at),
                // ⚠️ Suppressed on a `hostname` row, where the diff two columns along already
                // shows the observed name: the same value twice in one line reads as two facts.
                name: if gap.field == "hostname" {
                    String::new()
                } else {
                    observed_name.clone()
                },
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
                    candidates: Vec::new(),
                    evidence: Vec::new(),
                    open_question: None,
                    not_built: String::new(),
                    question: None,
                    no_placement: false,
                    earlier: Vec::new(),
                    only_distinct: false,
                    // No documenting gesture on this kind, so no subject to act on. ⚠️ NOT
                    // `SwitchedOff`: adopting one field of an existing record is FR13(b), Epic
                    // 7's, and telling the operator to set a switch would name a remedy that
                    // changes nothing here.
                    subject: String::new(),
                    gestures: action_bar("gesture.document", PrimaryState::NotBuilt),
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
                observed_at: newest.map_or(chrono::DateTime::UNIX_EPOCH, |b| b.observed_at),
                name: observed_name.clone(),
                field: String::new(),
                declared: String::new(),
                observed: String::new(),
                count: counted_fields("triage.n_fields", *count),
                counted: true,
                seen: observed_seen.clone(),
                age_seconds,
                selected: false,
            });
            panes.push((
                id,
                DetailPane {
                    candidates: Vec::new(),
                    evidence: Vec::new(),
                    open_question: None,
                    not_built: String::new(),
                    question: None,
                    no_placement: false,
                    earlier: Vec::new(),
                    only_distinct: false,
                    // No documenting gesture on this kind, so no subject to act on.
                    subject: String::new(),
                    // 🔴 From the CAUSE, never from the translated label: the mock shows *Résoudre*
                    // on a conflict and *Merger* elsewhere, and branching on the rendered string is
                    // story 6b.3's wrong-namespace defect waiting.
                    // Guy's decision B (2026-09-25, story 6.14): *Résoudre* names the AMBIGUITY gesture,
                    // and two sources disagreeing about a field are answered by declaring its value —
                    // the documenting gesture at field level, Epic 7's FR13(b). A conflict used to
                    // carry *Résoudre* because the mock put it there, which made one word name two acts.
                    gestures: action_bar("gesture.document", PrimaryState::NotBuilt),
                    kind: label.to_string(),
                    entity: ipv4.clone(),
                    field: String::new(),
                    declared: counted_fields("triage.cause.declared_side", *count),
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
    //
    // 🔴 **The MOST RECENT sighting of an address supplies the row, and it used to be the
    // oldest** — Guy's arbitration 1 at story 6.4 (2026-08-25). `load_observation_facts` orders
    // `observed_at` ASCENDING and this loop skipped an address it had already seen, so the FIRST
    // batch won. The validation measured it on two observations ten minutes apart: the older one
    // supplied the row.
    //
    // ⚠️ It was invisible while the row was a label, and it stops being invisible the moment a
    // GESTURE writes from it: story 6.4 documents *the whole record at once* from this batch, so
    // an older sighting would document the record the network has already moved past — one that
    // may lack a `hostname` a later scan saw. *A story that lays a gesture on a choice it knows
    // to be wrong inherits the choice.*
    //
    // 🔑 Kept as a LAST-WINS overwrite rather than a re-sort: the surrounding order is the queue's
    // and is not this loop's to change, and `rows` is already built by the time an address
    // repeats. `newest` remembers where the row went.
    let mut newest: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for batch in &observations {
        for (field, value) in batch.facts.iter().filter_map(display_fact) {
            if field != "ipv4" || claimed.contains(&value) {
                continue;
            }
            if let Some(&at) = newest.get(&value) {
                // A later sighting of an address already queued: replace what the row shows and
                // what the pane would act on, in place, keeping the queue's order.
                //
                // 🔴 **The comparison is the INSTANT, and it was the POSITION until 2026-09-09.**
                // The overwrite was unconditional and correct only because
                // `load_observation_facts` sorts `observed_at` ascending — a pure builder whose
                // answer depended on an ordering stated nowhere in its signature. The blind review
                // layer found the guard unable to tell *latest wins* from *last row wins*, and
                // rebuilding the same two sightings in the other order produced the OLDER one's
                // name under the NEWER one's age. `reconcile` was made order-independent the same
                // day; this is the same fix on the display side, and the guard now builds both
                // orders.
                if batch.observed_at < rows[at].observed_at {
                    continue;
                }
                let seen = relative_time(now, batch.observed_at);
                rows[at].seen = seen.clone();
                rows[at].age_seconds = (now - batch.observed_at).num_seconds().max(0);
                rows[at].observed_at = batch.observed_at;
                // The NAME moves with the freshness, for the reason story 6.4's review gave about
                // the source: a row showing the first sighting's name beside the latest one's age
                // would be two sightings in one line. A host that has since lost its lease shows
                // no name rather than the name it used to answer to.
                rows[at].name = hostname_of(&batch.facts).unwrap_or_default();
                let key = rows[at].id.clone();
                if let Some((_, pane)) = panes.iter_mut().find(|(id, _)| *id == key) {
                    // 🔴 The SOURCE moves with the freshness and the subject. It did not until
                    // story 6.4's code review, and all three layers found the consequence: the
                    // pane showed the FIRST sighting's provenance beside the LATEST one's
                    // freshness while the control posted the latest — measured on the wire
                    // (`Source aaaaaaaa · just now` over `subject: …bbbb`), against a template
                    // comment stating that the two *"cannot name two different observations"*.
                    // ⚠️ Reachable on the shipped product: `connector_id` is minted fresh at every
                    // boot, so scan → restart → scan over one undeclared address produces it.
                    pane.observed_meta = MetaLine {
                        source: source_label(&batch.connector_id),
                        freshness: seen,
                    };
                    pane.subject = batch.id.to_string();
                }
                continue;
            }
            newest.insert(value.clone(), rows.len());
            let id = format!("nouveau:{value}");
            let seen = relative_time(now, batch.observed_at);
            rows.push(QueueRow {
                href: row_href(&id, sort_by_age),
                id: id.clone(),
                kind: t!("triage.kind.nouveau").to_string(),
                entity: value.clone(),
                observed_at: batch.observed_at,
                name: hostname_of(&batch.facts).unwrap_or_default(),
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
                    candidates: Vec::new(),
                    evidence: Vec::new(),
                    open_question: None,
                    not_built: String::new(),
                    question: None,
                    no_placement: false,
                    earlier: Vec::new(),
                    only_distinct: false,
                    // The MOST RECENT sighting of this address, overwritten in place
                    // above when a later one arrives (Guy's arbitration, story 6.4).
                    subject: batch.id.to_string(),
                    gestures: action_bar(
                        "gesture.document",
                        if document_enabled {
                            PrimaryState::Acts
                        } else {
                            PrimaryState::SwitchedOff
                        },
                    ),
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

    // The L2 questions (story 6.14): one row per group, and the link from each `Nouveau` row whose
    // address a candidate carries.
    let (ambiguous_rows, ambiguous_panes, address_to_group) =
        crate::ambiguity_view::ambiguity_rows(ambiguity, &observations, now, sort_by_age);
    for (id, pane) in &mut panes {
        if let Some(address) = id.strip_prefix("nouveau:")
            && let Some(group) = address_to_group.get(address)
        {
            pane.open_question = Some(row_href(group, sort_by_age));
            // 🔑 E1 (story 6.14b, Guy 2026-09-25, UX-DR43): while the question is OPEN, the link stands
            // INSTEAD of *Ajouter* — documenting `obelix` twice before answering is D12's *"the operator
            // announces 300 hosts, the tool shows 340"*. ⚠️ It is keyed on the address being in an OPEN
            // question and on nothing else — the answers live on the question's own pane, not beside this
            // control; story 6.14's condition (a live gesture exists) holds because every open question
            // with a placed candidate offers one. Once answered, either way, the address is in no open
            // question and *Ajouter* returns by itself.
            pane.gestures.retain(|gesture| {
                !matches!(
                    gesture.nature,
                    GestureRender::Live(_) | GestureRender::Disabled(_)
                )
            });
        }
    }
    // 🔑 H1 (story 6.14b): the question sits directly BEFORE the first `Nouveau` row of its addresses,
    // so the eye meets it before its consequences; with no such row it keeps its place at the end.
    // `?sort=age` re-sorts after this, below.
    for row in ambiguous_rows {
        let before = rows.iter().position(|queued| {
            queued
                .id
                .strip_prefix("nouveau:")
                .and_then(|address| address_to_group.get(address))
                == Some(&row.id)
        });
        match before {
            Some(index) => rows.insert(index, row),
            None => rows.push(row),
        }
    }
    panes.extend(ambiguous_panes);

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
pub(crate) fn source_label(connector_id: &str) -> String {
    let short: String = connector_id.chars().take(8).collect();
    rust_i18n::t!("meta.source_id", id = short).to_string()
}

/// The declared side's meta-line: its origin and when it was written, or an honest absence.
pub(crate) fn declared_meta_line(
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
pub(crate) fn row_href(id: &str, sort_by_age: bool) -> String {
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
pub(crate) fn url_escape(raw: &str) -> String {
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
pub(crate) fn origin_key(origin: &str) -> &'static str {
    match origin {
        "manual" => "origin.manual",
        "adopted" => "origin.adopted",
        "imported" => "origin.imported",
        _ => "origin.unknown",
    }
}
