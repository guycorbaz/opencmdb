//! The shell: the mock's frame, its ten addresses, and the SIX screens it renders without the store.
//!
//! # What this module is, and the one structural decision inside it
//!
//! Story 6b.2 gives the product more than one address for the first time. Ten screens, each
//! server-rendered at its own URL — no client-side router, no screen chosen by JavaScript.
//!
//! 🔴 **The EIGHT pool-free screens live on a `Router<()>`, and that shape is a GUARD.** ⚠️ Nine
//! until story 6b.5 made the dashboard `Mixed`; the count is written here in ONE place and derived
//! everywhere it is asserted, because the literal that was not derived broke a test.
//! Epic 6b's constraint 1 says no demo screen opens a database connection, and the validation
//! measured that forbidding it by discipline is worth nothing: on the main router — whose state
//! IS the pool — a handler may take `State<MySqlPool>` and it compiles cleanly. Merged *after*
//! `.with_state(pool)`, this sub-router's state is `()`, so the same handler **fails to
//! compile**. The shape is the carrier; the sentence is only its description.
//!
//! 🔑 **Story 6b.3's AC3 — *"no database connection is opened and no row is written, and a test
//! says so"* — is discharged HERE, by this shape, and NO RUNTIME TEST WAS ADDED FOR IT.** That is a
//! decision, not an omission, and the reason is that a run-time assertion would be strictly WEAKER
//! than what already holds: a test can only observe that no query ran on the paths it happens to
//! exercise, while the type refuses the extractor on every path there will ever be. Writing one
//! would also be the epic's own dominant defect — a guard placed where the defect cannot occur.
//! The story's validation re-measured the carrier (`E0308`) before this sentence was written.
//!
//! ⚠️ `/triage` is deliberately NOT here. It is the one screen `epics.md` feeds with the real
//! gap (story 6b.4), and it renders today's reconciliation card, so it needs the pool and stays
//! on the main router. Nine screens without, one with — constraint 1 is about demonstrations,
//! and it is satisfied exactly where it applies.

use axum::Router;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;

use crate::page::{Shell, render_shell};

/// One of the ten screens the navigation offers.
///
/// The order is the mock's, and so are the three groups. A screen is identified by the same key
/// the mock uses (`data-screen`), which keeps the two comparable when someone checks fidelity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    /// *Triage* — the loop's entry point, and the one screen fed by the real gap.
    Triage,
    /// *Tableau de bord* — the daily overview (story 6b.5).
    Dashboard,
    /// *Appareils* — the device list Epic 6's grouping will fill (6b.6).
    Devices,
    /// *Fiche appareil* — one device's record (6b.6).
    Device,
    /// *Applications* — Epic 15's frame (6b.7).
    Apps,
    /// *IPAM* — Epic 14's frame (6b.7).
    Ipam,
    /// *Sources* — where observations come from (6b.8).
    Sources,
    /// *Alertes* — Epic 16's frame (6b.8).
    Alerts,
    /// *Auto-diagnostic* — what the product knows about itself (6b.9).
    Diagnostic,
    /// *Mise en service* — adopting the observed state as a baseline (6b.9).
    ///
    /// 🔴 **`Commissioning`, and it was `Onboarding` until story 6b.9.** The UX specification's F11
    /// correction reads *"bootstrap is a MODE, not an onboarding. Filing it under 'first run' was a
    /// design error: the wall recurs on every large migration"*, and the spec's own screen list
    /// names this screen **Commissioning**. The operator-visible label already read *Mise en
    /// service* in both locales; the identifier and the address were the last two artefacts still
    /// carrying the retired framing. Guy's arbitration of 2026-08-21, taken while `v0.2.0` was
    /// still ahead (story 6b.12) and the rename therefore free: **6 sites and one key**.
    Commissioning,
}

/// What a screen's content IS, and therefore whether it owes the operator a marker.
///
/// 🔴 **THREE variants since story 6b.9, and the count has moved twice — which is why it is written
/// as a history rather than as a number.** It read *"three variants and not two"* while there were
/// four (story 6b.5 added [`Mixed`](Nature::Mixed) six lines below the sentence and left it
/// standing; the code review caught it). Story 6b.9 then removed `Empty`, whose own doc had
/// promised since 6b.3 that *"when story 6b.9 closes there should be no `Empty` left"* — and that
/// promise is now carried by the compiler instead of by a sentence: the variant does not exist, so
/// a screen cannot be declared to hold nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Nature {
    /// The screen shows what the product really observed and really holds. It owes NO marker, and
    /// carrying one would be a lie in the other direction.
    Fed,
    /// The screen is **real in one section and example in the others**, and it owes the marker on
    /// each example section — never on the screen.
    ///
    /// # Why a fourth variant, and why not one of the three
    ///
    /// 🔴 Story 6b.3's AC2 predicted this in words: *"a screen-level-only marker would either lie
    /// about the real half or hide the example half"*. The dashboard is where the sentence stops
    /// being a prediction — declared [`Fed`](Nature::Fed) it carries a marker it must not, declared
    /// [`Example`](Nature::Example) it calls the real reach section a demonstration, and declared
    /// declared to hold nothing (a nature story 6b.9 removed) it would claim to hold nothing while
    /// holding the product's own counts.
    ///
    /// 🔑 **Guy's arbitration (2026-08-19), taken on a measurement**: adding this variant produces
    /// exactly **three `E0004` sites** — the body dispatch below and the partition test's two match
    /// arms — so the compiler forces both the marker decision and the pending-badge decision, and
    /// nothing silently defaults.
    ///
    /// ⚠️ **A `Mixed` screen is NOT on the pool-free router**, and that is the cost the arbitration
    /// accepted: its real half reads the store, so `/dashboard` sits with `/triage` and for that one
    /// screen the compile-time refusal of `State<MySqlPool>` no longer holds. The guard survives for
    /// the eight screens that remain, and is narrowed in writing for this one (story 5.12's
    /// precedent). 🔴 **A fragment-loaded reach section WAS built at validation and refused on a
    /// measured cost**: the route-table partition asserts on ONE synchronous body, and a second
    /// request is one a `oneshot` client cannot drive.
    Mixed,
    /// The screen shows the example dataset. It owes the marker, on the smallest unit that carries
    /// example content — and it **names WHICH example content**, which is the whole point of the
    /// payload.
    ///
    /// 🔴 **The variant carries its content so that "declared `Example` with nothing of its own"
    /// cannot be written.** Until story 6b.3's code review this read `Example` with no payload and
    /// the body was dispatched from the nature alone, which meant any second screen declared
    /// `Example` silently rendered the DEVICE inventory under its own heading — the compiler
    /// enforced that a nature was DECLARED, never that the content MATCHED the screen. The sole
    /// carrier was a bookkeeping count assertion that a future tidy-up would delete as redundant.
    /// Guy's arbitration (2026-08-19): close it in the TYPE, on story 5.6's precedent — that story
    /// closed the self-pair in `CandidatePair::new` rather than in a test, and this story's own AC3
    /// argues the same way, that a compile refusal beats an assertion.
    Example(ExampleContent),
}

/// Which body an [`Nature::Example`] screen shows.
///
/// 🔑 **One variant today, and that is not a reason to collapse it into the nature.** Stories
/// 6b.5–6b.9 each add a screen with its own example content; this enum is where each one declares
/// what it shows, and the payload on [`Nature::Example`] is what makes declaring the nature without
/// declaring the content impossible to express.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExampleContent {
    /// The example inventory: the device list and the sightings the example engine did not place
    /// (story 6b.3's witness screen, filled out to the mock's shape by story 6b.6).
    DevicesInventory,
    /// One device's record — its fields, what it hosts, its composite identity, its observation
    /// history (story 6b.6).
    ///
    /// ⚠️ **This content is NOT served by the generic loop in [`router`].** Its address is a
    /// parameterised route, which [`Screen`] structurally cannot represent: [`Screen::href`] returns
    /// a `&'static str` used both as a route PATTERN and as a URL the guards FETCH, and
    /// `/devices/{id}` cannot be both. See [`router`] for the skip and for what it costs.
    DeviceRecord,
    /// The example application inventory — Epic 15's frame (story 6b.7).
    AppsInventory,
    /// The example subnet occupancy — Epic 14's frame (story 6b.7).
    IpamOccupancy,
    /// The example alert list — Epic 16's frame (story 6b.8).
    AlertList,
    /// The example commissioning walk-through and its baselining block — Epic 9's frame
    /// (story 6b.9).
    Commissioning,
}

impl ExampleContent {
    /// Render this content's body.
    fn render(self, query: &crate::example_screens::ScreenQuery) -> String {
        match self {
            // ⚠️ `query`, not `ScreenQuery::default()`. It read `default()` after the parameter
            // had been threaded through the router, `demonstration_screen` and this signature —
            // and **nothing warned**: Rust does not lint an unused function PARAMETER. The route
            // filtered nothing while every pure test stayed green. Only the route test saw it.
            ExampleContent::DevicesInventory => crate::example_screens::inventory_body(query),
            // Unreachable by construction, and by the SAME mechanism as `Nature::Fed` below:
            // `router` never registers this screen's address, because the parameterised route
            // serves it. It is `unreachable!` rather than a silent fallback so that the day
            // someone deletes the skip, a test says WHICH assumption broke.
            ExampleContent::DeviceRecord => unreachable!(
                "the device record is served by the parameterised route, not by the generic loop"
            ),
            // ⚠️ Both take the query and one of them ignores it — see `apps_body`'s doc for why the
            // underscore there is a statement rather than a habit.
            ExampleContent::AppsInventory => crate::example_screens::apps_body(query),
            ExampleContent::IpamOccupancy => crate::example_screens::ipam_body(query),
            ExampleContent::AlertList => crate::example_screens::alerts_body(query),
            ExampleContent::Commissioning => crate::example_screens::commissioning_body(query),
        }
    }
}

/// The three groups the mock's navigation is divided into, in its order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NavGroup {
    /// *Boucle* — the daily loop.
    Loop,
    /// *Inventaire* — what is out there.
    Inventory,
    /// *Machine* — the product's own machinery.
    Machine,
}

impl Screen {
    /// Every screen, in the mock's navigation order.
    pub(crate) const ALL: [Screen; 10] = [
        Screen::Triage,
        Screen::Dashboard,
        Screen::Devices,
        Screen::Device,
        Screen::Apps,
        Screen::Ipam,
        Screen::Sources,
        Screen::Alerts,
        Screen::Diagnostic,
        Screen::Commissioning,
    ];

    /// The screen's own address — one URL per screen, deep-linkable cold.
    ///
    /// ⚠️ `/device` addresses no particular device, which is what the mock does and what Guy
    /// arbitrated for now. The honest shape is `/devices/{id}` and it needs an id that does not
    /// exist yet; the debt is registered with story 6b.6.
    pub(crate) fn href(self) -> &'static str {
        match self {
            Screen::Triage => "/triage",
            Screen::Dashboard => "/dashboard",
            Screen::Devices => "/devices",
            Screen::Device => "/devices/nas-01",
            Screen::Apps => "/apps",
            Screen::Ipam => "/ipam",
            Screen::Sources => "/sources",
            Screen::Alerts => "/alerts",
            Screen::Diagnostic => "/diagnostic",
            Screen::Commissioning => "/commissioning",
        }
    }

    /// The i18n key of the entry's label.
    pub(crate) fn label_key(self) -> &'static str {
        match self {
            Screen::Triage => "nav.triage",
            Screen::Dashboard => "nav.dashboard",
            Screen::Devices => "nav.devices",
            Screen::Device => "nav.device",
            Screen::Apps => "nav.apps",
            Screen::Ipam => "nav.ipam",
            Screen::Sources => "nav.sources",
            Screen::Alerts => "nav.alerts",
            Screen::Diagnostic => "nav.diagnostic",
            Screen::Commissioning => "nav.commissioning",
        }
    }

    /// Which of the mock's three groups the entry belongs to.
    pub(crate) fn group(self) -> NavGroup {
        match self {
            Screen::Triage | Screen::Dashboard => NavGroup::Loop,
            Screen::Devices | Screen::Device | Screen::Apps | Screen::Ipam => NavGroup::Inventory,
            Screen::Sources | Screen::Alerts | Screen::Diagnostic | Screen::Commissioning => {
                NavGroup::Machine
            }
        }
    }

    /// The i18n key of the screen's own title.
    pub(crate) fn title_key(self) -> &'static str {
        self.label_key()
    }

    /// What this screen's content is, and therefore whether it owes the marker.
    ///
    /// 🔴 **A `match`, deliberately, and never a field with a default.** AC4 requires that *"a route
    /// added without a declared nature must FAIL rather than default"*, and this is the carrier: a
    /// new `Screen` variant does not compile until someone decides what it shows. Story 6b.3's
    /// validation measured it — a variant added without an arm here is `E0004`, in company with
    /// [`Screen::href`], [`Screen::label_key`] and [`Screen::group`].
    ///
    /// 🔑 **The OTHER half of AC4 — that the variant reaches [`Screen::ALL`] — is pinned by
    /// `every_variant_of_a_navigated_enum_is_listed_in_all` in this file's test module**, and that
    /// guard is story 6b.3's code review, not the story. Until then the only carrier was `dead_code`
    /// under `cargo clippy --workspace --locked -- -D warnings`, which is **outside
    /// `cargo xtask ci`** — a developer running only the eight gates never saw the red — and which
    /// the review measured **silenced by one ordinary line of production code** constructing the
    /// variant anywhere. See that test for what it does and does not promise.
    pub(crate) fn nature(self) -> Nature {
        match self {
            // The one screen fed by what the product really observed (story 6b.2, `page::triage`).
            Screen::Triage => Nature::Fed,
            // Real reach beside labelled example sections — the one screen that is mixed by
            // construction (story 6b.5, Guy's arbitration of 2026-08-19).
            Screen::Dashboard => Nature::Mixed,
            // The witness screen, filled from the example dataset (Guy's arbitration, 2026-08-19).
            Screen::Devices => Nature::Example(ExampleContent::DevicesInventory),
            // The device record, served by the parameterised route (story 6b.6).
            Screen::Device => Nature::Example(ExampleContent::DeviceRecord),
            // ⚠️ Each of these became `Example` in ITS OWN story, listed beside it.
            // Epic 15's and Epic 14's frames, filled from the example dataset (story 6b.7).
            Screen::Apps => Nature::Example(ExampleContent::AppsInventory),
            Screen::Ipam => Nature::Example(ExampleContent::IpamOccupancy),
            // 🔴 **`Fed`, NOT `Mixed` — and the story said `Mixed` until the code was written.**
            // Its §0e reasoned that `/sources` would hold a real capability section beside an
            // example alert list. It does not: AC3 puts the alert list on its OWN screen, which
            // `Screen::ALL` has carried since story 6b.2, so **every section of `/sources` is real**
            // and the screen owes no marker at all. Declaring it `Mixed` would have made the
            // route-table partition demand a marker for example content that does not exist.
            // 🔑 *A nature is a statement about what the screen SHOWS, and the screen turned out to
            // show something simpler than the plan for it.*
            //
            // ⚠️ Its route was registered in `page::triage_router` BEFORE this line changed, and the
            // order is a measurement: a nature changed with the route forgotten reds **nothing**
            // locally and exactly one test in CI — the loop skips `Fed` and `Mixed` alike when no
            // database is reachable — while the mirror mistake reds eighteen.
            Screen::Sources => Nature::Fed,
            // Epic 16's frame, example all the way through (story 6b.8).
            Screen::Alerts => Nature::Example(ExampleContent::AlertList),
            // 🔴 **`Fed`, and every section of it is real** (story 6b.9): the build, the store's
            // own version and schema, what the last pass did, what has been recorded and placed,
            // the security perimeter derived by probing `auth::is_public`, and the descriptor
            // `init_tracing` installed. Nothing on it is fabricated, so it owes no marker —
            // declaring it `Mixed` would make the partition demand a marker for example content
            // that does not exist. *A nature is a statement about what the screen SHOWS.*
            Screen::Diagnostic => Nature::Fed,
            // Epic 9's baselining, example all the way through (story 6b.9).
            Screen::Commissioning => Nature::Example(ExampleContent::Commissioning),
        }
    }
}

impl NavGroup {
    /// The groups in the mock's order.
    pub(crate) const ALL: [NavGroup; 3] = [NavGroup::Loop, NavGroup::Inventory, NavGroup::Machine];

    /// The i18n key of the group's heading.
    pub(crate) fn heading_key(self) -> &'static str {
        match self {
            NavGroup::Loop => "nav.group.loop",
            NavGroup::Inventory => "nav.group.inventory",
            NavGroup::Machine => "nav.group.machine",
        }
    }
}

/// The seven pool-free screens (see this module's doc). ⚠️ Seven since story 6b.8 made
/// `Sources` **`Fed`** — its content is real and it reads the store, so it lives on the main router
/// with `/triage` and `/dashboard`. This sentence said *`Mixed`* until the code review: the story
/// planned `Mixed`, the code shipped `Fed`, and **the doc kept the plan's word twenty lines below the
/// arm that refutes it** — the dominant defect of this project, in the commit that corrects the same
/// slip elsewhere. ⚠️ `Fed` and `Mixed` are both excluded:
/// each reads the store.
///
/// # Returns
///
/// A `Router<()>` to be merged into the main router *after* its `.with_state(pool)`. The return
/// type is load-bearing: change it to `Router<MySqlPool>` and the whole guard evaporates.
pub(crate) fn router(perimeter: Option<String>) -> Router {
    let mut router = Router::new();
    for screen in Screen::ALL {
        if screen
            .href()
            .strip_prefix(RECORD_PREFIX)
            .is_some_and(|slug| crate::example_data::device_by_id(slug).is_some())
        {
            // ⚠️ **The skip names a REAL device, and the narrowing is a code-review finding.** It
            // read `starts_with(RECORD_PREFIX)` alone, which swallowed **any** future screen
            // addressed under `/devices/` — measured: a `Screen` with `href() = "/devices/backup"`
            // and its own `ExampleContent` compiled with **zero warnings**, its render arm became
            // silently dead, and the address served the *unknown device* page. The three tests
            // that reddened were all bookkeeping COUNTS whose messages read *"update this
            // number"*, and doing so left 652 tests, clippy and eight gates green over a screen
            // serving the wrong body. Epic 5's dominant class, inside a guard written for this
            // story. Now a screen the record route cannot actually serve falls through and gets
            // its own static route, which axum matches ahead of the parameterised one.
            // 🔴 **Served by the parameterised route registered below, never by this loop**, and
            // the skip is the whole of Guy's arbitration of 2026-08-19 (*"hors de `Screen`"*).
            //
            // `Screen::href` returns a `&'static str` that this loop uses as a route PATTERN while
            // the partition and auth-perimeter tests FETCH it as a URL. A parameterised path cannot
            // be both, so the record's address is the first in this product that `Screen::ALL`
            // structurally cannot represent. The story's validation BUILT both alternatives:
            //
            // - a `Screen` variant with `href() = "/devices/{id}"` **compiles without a warning**
            //   and ships a literal `href="/devices/{id}"` in all eleven navigations, which every
            //   existing guard accepts;
            // - registering this screen's address statically as well **shadows** the parameterised
            //   route for exactly the URL the guards probe — after which the record handler was
            //   measured able to ignore its slug entirely, serve device #1 for
            //   `/devices/does-not-exist`, and leave 636 tests and `clippy -D warnings` green.
            //
            // Skipping makes the shadow unrepresentable rather than merely tested. ⚠️ The cost is
            // that no INHERITED guard covers this route, so it carries two of its own — see
            // `the_record_route_answers_a_non_canonical_slug` and its unknown-slug twin.
            continue;
        }
        if matches!(screen.nature(), Nature::Fed | Nature::Mixed) {
            // 🔑 Keyed on the NATURE, not on the identity: `Screen::Triage` was named here until
            // story 6b.3, and the two would drift the day a second screen becomes fed. Now the
            // exclusion and the body-dispatch below read the same decision.
            // ⚠️ `Mixed` joined `Fed` at story 6b.5: its real half reads the store, so it cannot
            // live on a `Router<()>`. That is the cost Guy's arbitration accepted, and it is
            // narrowed rather than hidden — see [`Nature::Mixed`].
            continue;
        }
        let perimeter = perimeter.clone();
        router = router.route(
            screen.href(),
            get(
                // 🔴 **The query is EXTRACTED, and it was not until a look at the running server.**
                // The closure took no arguments and `ExampleContent::render` called the inventory
                // with `ScreenQuery::default()`, so `/devices?kind=printer` served all eight
                // devices while `the_filter_narrows_…` — which calls the pure builder directly —
                // stayed green. Epic 5's dominant class: *a guard placed where the defect cannot
                // occur reads as coverage and is none*, and story 6b.4's `triage_html` was the same
                // shape. `the_filter_narrows_through_the_real_route` is the guard that can see it.
                move |axum::extract::Query(query): axum::extract::Query<
                    crate::example_screens::ScreenQuery,
                >| async move { demonstration_screen(screen, perimeter, &query) },
            ),
        );
    }
    {
        let perimeter = perimeter.clone();
        router = router.route(
            RECORD_ROUTE,
            get(
                move |axum::extract::Path(id): axum::extract::Path<String>| async move {
                    device_record(&id, perimeter)
                },
            ),
        );
    }
    router
}

/// Every address the parameterised record route covers.
///
/// 🔑 A PREFIX and not a list: `router` skips whatever it covers, so adding a screen under it can
/// never leave two routes racing for one address.
pub(crate) const RECORD_PREFIX: &str = "/devices/";

/// The record route's pattern, in axum's syntax.
pub(crate) const RECORD_ROUTE: &str = "/devices/{id}";

/// One device's record, or the page that says the slug names no device.
///
/// ⚠️ The unknown case answers **200 and a page**, not 404. The address is a real screen of the
/// product reached from its own navigation; what is unknown is the slug, and a 404 would tell the
/// operator the feature does not exist. 🔴 The slug is **never echoed** — see
/// `_device_unknown.html` for the reflected-XSS shape that refuses.
fn device_record(id: &str, perimeter: Option<String>) -> Response {
    // 🔑 The marker, from the same place `demonstration_screen` gets it. This route does not go
    // through that function — it is the one address off `Screen` — and the first version of it
    // therefore served four example sections with no marker at all. `Screen::Device`'s nature is
    // `Example`, so the partition test named it immediately; the point is that the skip in
    // `router` buys structural safety on one axis and costs it on another, which is why this call
    // is here and not implied.
    let body = format!(
        "{}{}",
        crate::page::example_marker(),
        crate::example_screens::record_body(id)
            .unwrap_or_else(crate::example_screens::unknown_device_body)
    );
    Html(render_shell(Shell::new(Screen::Device, perimeter), body)).into_response()
}

/// A demonstration screen, rendered according to what its content IS.
///
/// 🔑 **The body is chosen by [`Screen::nature`], never by the screen's identity.** That is what
/// makes the marker impossible to forget: a screen declared `Example` gets example content AND the
/// marker from the same decision, so there is no arrangement of this function in which content
/// arrives unmarked.
///
/// ⚠️ **There is no *not built yet* arm any more.** Story 6b.9 filled the last two screens, so
/// [`Nature`] lost its `Empty` variant and the sentence with it: this product no longer has a screen
/// that holds nothing. A future screen that does must add the variant back deliberately rather than
/// inherit it — and the compiler will ask, because every `match` on a nature is exhaustive.
fn demonstration_screen(
    screen: Screen,
    perimeter: Option<String>,
    query: &crate::example_screens::ScreenQuery,
) -> Response {
    let body = match screen.nature() {
        // 🔑 **The CONTENT and the MARKER come from the same decision**, so no screen can be
        // `Example` without having said what it shows and without saying it IS an example — see
        // [`Nature::Example`] for the defect the payload closes, and
        // [`crate::page::example_marker`] for why the marker is emitted here rather than included
        // by each template.
        Nature::Example(content) => {
            format!("{}{}", crate::page::example_marker(), content.render(query))
        }
        // Unreachable by construction: `router` never merges a `Fed` screen — those need the pool
        // and live on the main router. It is `unreachable!` rather than a silent fallback so the
        // day someone changes `nature` without changing `router`, the test says WHICH assumption
        // broke instead of quietly serving a blank page.
        // Unreachable for the same reason and by the same mechanism: `router` skips both, because
        // both read the store. Kept as two arms rather than one so the message names which
        // assumption broke.
        Nature::Fed => unreachable!("a Fed screen is not merged onto the pool-free router"),
        Nature::Mixed => unreachable!("a Mixed screen is not merged onto the pool-free router"),
    };
    Html(render_shell(Shell::new(screen, perimeter), body)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::{Shell, render_shell};

    /// The rendered frame for one screen, with an empty body.
    fn shell_html(screen: Screen) -> String {
        render_shell(
            Shell::new(screen, Some("192.0.2.0/24".to_string())),
            String::new(),
        )
    }

    /// AC1 — ten entries, three groups, on every screen.
    #[test]
    fn every_screen_renders_ten_entries_in_three_groups() {
        for screen in Screen::ALL {
            let html = shell_html(screen);
            assert_eq!(
                html.matches("class=\"nav-entry\"").count(),
                10,
                "{screen:?}: the navigation offers ten screens from the first day — hiding one \
                 is story 6b.3's subject, not this story's escape hatch"
            );
            assert_eq!(
                html.matches("class=\"nav-group\"").count(),
                3,
                "{screen:?}: the mock's three groups (Boucle · Inventaire · Machine)"
            );
        }
    }

    /// AC1 — exactly one current entry. *At least one* would pass with all ten marked.
    #[test]
    fn exactly_one_entry_is_current_on_each_screen() {
        for screen in Screen::ALL {
            let html = shell_html(screen);
            assert_eq!(
                html.matches("aria-current=\"page\"").count(),
                1,
                "{screen:?}: exactly one entry is the current one"
            );
        }
    }

    /// AC1 — the version comes from the crate, never from a literal.
    ///
    /// A number typed into a template is a number that lies on a release day; the mock's own
    /// `v0.1.1 · maquette` is exactly such a literal, and the word *maquette* must not ship.
    #[test]
    fn the_version_is_the_crates_own_and_no_literal_is_typed() {
        let html = shell_html(Screen::Triage);
        assert!(
            html.contains(&format!("v{}", env!("CARGO_PKG_VERSION"))),
            "the header shows the crate's version"
        );
        assert!(!html.contains("maquette"), "the mock's word does not ship");
        let shell = include_str!("../templates/_shell.html");
        assert!(
            !shell.contains("v0."),
            "no version literal in the template — it must come from the crate"
        );
    }

    /// AC3 — every entry is OFFERED, not merely displayed, and the guard is a property.
    ///
    /// # What this guard does NOT see, stated rather than discovered
    ///
    /// The validation attacked an implementation of this ten ways and **two survived**:
    /// `display: none` written in `app.css` (which is where anyone would write it in a
    /// hand-authored-CSS story) and an inline `pointer-events: none`. Both leave markup this
    /// guard accepts. 🔑 **A guard covering them is not writable at the template-text level** —
    /// it needs computed styles, i.e. axe-core or a headless browser over the ten routes, which
    /// the epic's Definition of Done asks for and which no story yet owns (registered).
    #[test]
    fn every_entry_is_offered_with_its_own_address() {
        let html = shell_html(Screen::Triage);
        for screen in Screen::ALL {
            let anchor = format!("href=\"{}\"", screen.href());
            assert!(
                html.contains(&anchor),
                "{screen:?} must be reachable at its own address: {anchor} is absent"
            );
        }
        for withheld in [
            "href=\"#\"",
            "href=\"\"",
            "hidden",
            "aria-disabled",
            "display:none",
        ] {
            assert!(
                !html.contains(withheld),
                "an entry may not be displayed and withheld: found {withheld:?} — note the mock \
                 writes href=\"#\" on all ten, so transcribing it faithfully produces this"
            );
        }
    }

    /// AC2 — the screen is chosen by the ROUTE, not by JavaScript.
    #[test]
    fn no_screen_is_chosen_by_javascript() {
        let js = include_str!("../assets/app.js");
        for forbidden in ["data-screen", "pushState", "location.hash", "history."] {
            assert!(
                !js.contains(forbidden),
                "the mock switches screens client-side; this product does not: found {forbidden:?}"
            );
        }
    }

    /// AC2 — each screen renders its OWN identity, so a deep link lands where it says.
    #[test]
    fn each_screen_marks_itself_current() {
        for screen in Screen::ALL {
            let html = shell_html(screen);
            let marked = format!("href=\"{}\" aria-current=\"page\"", screen.href());
            assert!(
                html.contains(&marked),
                "{screen:?} must mark its OWN entry, or a deep link lands on the wrong screen"
            );
        }
    }

    /// AC6 — every key a screen references RESOLVES.
    ///
    /// 🔴 `rust-i18n` renders a missing key as its own NAME — no panic, not empty — so a typo
    /// ships as visible page text. The validation measured it: `nav.apps` → `nav.appz` put the
    /// literal `nav.appz` in the navigation with every other guard green.
    #[test]
    fn no_screen_renders_a_key_name_as_a_label() {
        let html = shell_html(Screen::Triage);
        for screen in Screen::ALL {
            assert!(
                !html.contains(screen.label_key()),
                "{:?} rendered its KEY instead of its label — rust-i18n falls back to the key \
                 name, so a typo is invisible to every other guard",
                screen
            );
        }
        for group in NavGroup::ALL {
            assert!(
                !html.contains(group.heading_key()),
                "{group:?} rendered its key instead of its heading"
            );
        }
    }

    /// AC6 — no key ever regresses to a single locale.
    ///
    /// 🔴 **This is the direction that is silent.** `rust-i18n` falls back to `en`, so deleting a
    /// FRENCH value renders English inside the French interface and no render-time guard can
    /// see it — measured: with `nav.alerts`' `fr` half removed, the whole suite stayed green.
    /// The English direction is loud by comparison. So the assertion is over the FILE, in both
    /// directions, rather than over a rendering.
    ///
    /// Baseline measured on `master`: 32 top-level entries, not one missing a locale. The guard
    /// is therefore *"no key regresses"*, never *"add the missing ones"*.
    #[test]
    fn every_key_carries_both_locales() {
        let yaml = include_str!("../locales/app.yml");
        let mut key = String::new();
        let mut locales: Vec<&str> = Vec::new();
        let mut checked = 0usize;
        let flush = |key: &str, locales: &mut Vec<&str>, checked: &mut usize| {
            if key.is_empty() || key == "_version" {
                locales.clear();
                return;
            }
            *checked += 1;
            assert!(
                locales.contains(&"en") && locales.contains(&"fr"),
                "{key} carries {locales:?} — a key with only `en` renders English inside the \
                 French UI, silently, because rust-i18n falls back"
            );
            locales.clear();
        };
        for line in yaml.lines() {
            let trimmed = line.trim_end();
            if trimmed.starts_with("  en:") {
                locales.push("en");
            } else if trimmed.starts_with("  fr:") {
                locales.push("fr");
            } else if let Some(name) = trimmed.strip_suffix(':')
                && !name.starts_with(' ')
                && !name.starts_with('#')
            {
                flush(&key, &mut locales, &mut checked);
                key = name.to_string();
            }
        }
        flush(&key, &mut locales, &mut checked);
        assert!(
            checked >= 47,
            "the premise: 48 entries minus `_version` ({checked} scanned) — a scan that found \
             nothing would assert nothing"
        );
    }

    /// AC1 — the perimeter comes from configuration, and its absence is said rather than blank.
    #[test]
    fn the_perimeter_is_shown_and_its_absence_is_named() {
        let configured = shell_html(Screen::Triage);
        assert!(
            configured.contains("192.0.2.0/24"),
            "the nav footer shows the perimeter the operator configured"
        );
        let unset = render_shell(Shell::new(Screen::Triage, None), String::new());
        assert!(
            unset.contains("not configured") || unset.contains("non configuré"),
            "an unset perimeter is NAMED, never rendered as an empty value"
        );
        assert!(
            !unset.contains("Perimeter </div>") && !unset.contains("Périmètre </div>"),
            "and never a dangling label"
        );
    }

    /// AC1 / T4b — the perimeter has exactly ONE reader, and it is `AppConfig::from_env`.
    ///
    /// 🔴 This guard exists because the story shipped its own mutation. The table predicts M12
    /// — *read the perimeter in the handler instead of `AppConfig`* — as a red; M12 was never
    /// executed, and the first implementation of `/triage` WAS M12, with all 608 tests green.
    ///
    /// 🔑 The sibling guard cannot see it and is right about what it tests: it hands
    /// `render_shell` a `None` directly, so it never reaches the code that *produces* the value.
    /// The defect lives in the producer — `AppConfig::from_env` filters a blank value and a
    /// second reader does not, so one fact rendered two ways on one shell. What must be
    /// asserted is therefore the absence of a second reader, and that is a property of the
    /// SOURCE: you cannot measure the absence of code by running it (story 5.12's rule).
    ///
    /// ⚠️ Its stated limits, both measured rather than supposed: it matches a line carrying the
    /// variable's name AND a `var(`/`lookup(` call, so a reader that assembles the name at
    /// runtime, or splits the call across two lines, is invisible to it. A tripwire against the
    /// ordinary gesture, never a barrier — story 5.12's narrowing, applied rather than inherited.
    #[test]
    fn the_perimeter_has_a_single_reader() {
        // 🔑 Assembled from two halves so the guard does not find ITSELF — measured: spelled
        // whole, it reported its own matcher as a fourth reader.
        const NEEDLE: &str = concat!("OPENCMDB_", "SCAN_CIDR");
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut scanned = 0_usize;
        let mut readers: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(&dir).expect("src/ must be readable") {
            let path = entry.expect("a readable entry").path();
            if path.extension().is_none_or(|ext| ext != "rs") {
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let body = std::fs::read_to_string(&path).expect("a readable source file");
            scanned += 1;
            for (number, line) in body.lines().enumerate() {
                // 🔑 Comments first, or this guard reds on the prose explaining it — the trap
                // this story already met once today.
                let code = line.split_once("//").map_or(line, |(before, _)| before);
                // 🔑 A READ, not a mention. Named alone, the variable also appears in a
                // `tracing::error!` string in `main.rs` — measured, and it is a log line, not a
                // reader. A guard that reds on a log message is wrong in the second direction.
                let reads = code.contains("var(") || code.contains("lookup(");
                if code.contains(NEEDLE) && reads {
                    readers.push(format!("{name}:{}", number + 1));
                }
            }
        }
        assert!(
            scanned >= 15,
            "the premise: src/ holds the crate's modules ({scanned} scanned) — an empty scan \
             would assert nothing"
        );
        assert_eq!(
            readers.len(),
            1,
            "the perimeter must have ONE reader and it must be `AppConfig::from_env`; found \
             {readers:?}. A second reader does not filter what the first one filters, so a \
             blanked variable renders `not configured` on every demonstration screen and a \
             dangling label on the fed one — nine and one when story 6b.2 measured it, eight and \
             two since 6b.5. Configuration enters as a PARAMETER (story 6.1)."
        );
        assert!(
            readers[0].starts_with("main.rs:"),
            "and that reader lives in `main.rs`, not in a handler: {readers:?}"
        );
    }

    /// AC1 — the header carries no last observation, and no screen reads the clock or the store.
    ///
    /// ⚠️ Written over the WHOLE shell rather than over the `<header>` element: the validation
    /// measured that a guard scoped to the header passes over a shell carrying both facts in the
    /// sidebar — *the guard placed where the defect cannot occur*.
    #[test]
    fn the_shell_shows_no_last_observation() {
        // 🔑 Comments stripped first — the same idiom `float-free` uses so the architecture may
        // be QUOTED in a comment without tripping a gate, and the same trap story 6b.1 met when
        // its radius comment contained the very text its scanner counted. This template's own
        // comment names the deferred fact in order to explain it.
        // 🔴 BOTH files of the frame, not just the navigation. This guard's own comment used to
        // claim it was *"written over the WHOLE shell rather than over the `<header>` element"*
        // — story 5.14b's lesson quoted by name — while it read `_nav.html` alone. Measured by
        // the code review: the mock's own text planted in `_shell.html`'s `<header>` left the
        // whole suite green. The guard was correct about what it tested, and its comment was
        // false; a guard that claims a perimeter it does not walk is worse than a narrow one.
        //
        // ⚠️ `_gap_card.html` is deliberately NOT here: it SHOWS observed values, which is its
        // whole job. What is banned is the last-observation INSTANT (a `MAX(observed_at)`), and
        // it is banned from the frame, which renders on all ten screens.
        let frame = [
            ("_shell.html", include_str!("../templates/_shell.html")),
            ("_nav.html", include_str!("../templates/_nav.html")),
        ];
        let strip = |source: &str| -> String {
            source
                .split("{#")
                .enumerate()
                .map(|(i, part)| {
                    if i == 0 {
                        part.to_string()
                    } else {
                        part.split_once("#}")
                            .map(|(_, rest)| rest)
                            .unwrap_or("")
                            .to_string()
                    }
                })
                .collect()
        };
        for (name, source) in frame {
            // 🔑 Comments stripped first — the same idiom `float-free` uses so the architecture
            // may be QUOTED in a comment without tripping a gate, and the same trap story 6b.1
            // met when its radius comment contained the very text its scanner counted. Both of
            // these templates name the deferred fact in a comment, in order to explain it.
            let code = strip(source);
            assert!(
                !code.to_lowercase().contains("observ"),
                "{name}: the last observation is a MAX(observed_at) — a database read on ten \
                 screens, nine of them demonstrations. Deferred to story 6b.5 (registered), and \
                 the ABSENCE is asserted because it is a decision, not an omission"
            );
        }
        // 🔑 And the frame WIDENS ITSELF: any partial the shell pulls in must be scanned above,
        // or this reds. Without it the guard goes stale the day someone splits the header out —
        // an enumeration that cannot notice what it stopped covering.
        for (name, source) in frame {
            for (index, _) in strip(source).match_indices("{% include") {
                let rest = &strip(source)[index..];
                let quoted = rest
                    .split_once('"')
                    .and_then(|(_, after)| {
                        after.split_once('"').map(|(inner, _)| inner.to_string())
                    })
                    .unwrap_or_else(|| panic!("{name}: an include with no quoted path"));
                assert!(
                    frame.iter().any(|(scanned, _)| *scanned == quoted),
                    "{name} includes {quoted:?}, which this guard does not scan — add it to \
                     `frame`, or the ban stops covering the frame it names"
                );
            }
        }
    }

    /// 🔴 **Every variant of a navigated enum is listed in its `ALL`, in BOTH directions.**
    ///
    /// # What this pins, and why it is a test rather than a ninth gate
    ///
    /// `Screen::ALL` and `NavGroup::ALL` are literal arrays, and **the compiler does not check an
    /// array for exhaustiveness**. A variant wired into every `match` — `href`, `label_key`,
    /// `group`, `nature` — and left out of `ALL` compiles cleanly and disappears from the
    /// navigation, from the routing, and from every test that iterates the table, including the
    /// route-table partition AC4 asks for. That is *the eleventh screen* AC4 names.
    ///
    /// 🔑 **Story 6b.3 reasoned that pinning it would have to be a GATE, and the reasoning was
    /// misapplied.** Story 5.12's rule — *you cannot measure the absence of code by running code* —
    /// governs an UNBOUNDED absence: no file in the tree, including files that do not exist yet.
    /// The property here is bounded and present: do the variants declared in this file appear in
    /// the array declared in this file? Both constructs exist now, in one place, and reading the
    /// source measures it exactly. **The idiom is already in this very module** —
    /// `every_key_carries_both_locales` reads `include_str!("../locales/app.yml")` and
    /// `the_perimeter_has_a_single_reader` walks `src/`.
    ///
    /// ⚠️ **What it replaces was measured DEFEATED.** Until story 6b.3's code review the only
    /// carrier was `dead_code` under `cargo clippy --workspace --locked -- -D warnings`. That lint
    /// lives **outside `cargo xtask ci`**, so a developer running only the eight gates never saw
    /// the red; and the review measured that **one throw-away line of production code constructing
    /// the variant — `let _bypass = Screen::Probe;`, nothing to do with `ALL` — makes clippy exit 0
    /// while the variant is still missing from the array, the navigation, the routing and the
    /// partition test.** This guard is inside `cargo test --workspace`, and no such line silences
    /// it.
    ///
    /// ⚠️ **Its limit, written rather than implied — a TRIPWIRE against the ordinary gesture, never
    /// a barrier** (story 5.12's narrowing, third application in this project). It reads THIS
    /// file's text: move either enum to another module and the guard goes blind without a word.
    /// It is aimed at the developer who adds a screen and forgets one place, which is the gesture
    /// story 6b.2's review measured as the real one — not at anyone working around it.
    ///
    /// 🔑 **`NavGroup` is covered by the same property, and that is the argument for a property
    /// over an enumeration**: its `ALL` is a literal `[NavGroup; 3]` with the identical hole, and a
    /// guard written for `Screen` alone would have covered one of the two.
    /// 🔴 **The navigation's device address names a device that EXISTS.**
    ///
    /// `Screen::Device.href()` is the literal `"/devices/nas-01"` — a slug of
    /// [`crate::example_data`] with nothing tying the two together at compile time, because
    /// [`Screen::href`] returns a `&'static str` and a record needs an id. **Rename or remove that
    /// device and the product's own primary link to the record silently degrades to the
    /// *unknown device* page**: a real, tested and WRONG 200, with no compiler error and no
    /// inherited guard positioned to see it.
    ///
    /// 🔑 Found by the review layer that had **only the diff** — it could not check the slug
    /// against the dataset, so it asked what happens when the two drift, which is the question
    /// having both in front of you does not prompt.
    #[test]
    fn the_navigations_device_address_names_a_device_that_exists() {
        let href = Screen::Device.href();
        let slug = href
            .strip_prefix(RECORD_PREFIX)
            .unwrap_or_else(|| panic!("{href} must live under {RECORD_PREFIX}"));
        assert!(
            crate::example_data::device_by_id(slug).is_some(),
            "the navigation points at {href} and no example device carries the slug {slug:?} — \
             the entry would render the *unknown device* page, which is a correct 200 for a wrong \
             reason"
        );
    }

    #[test]
    fn every_variant_of_a_navigated_enum_is_listed_in_all() {
        // 🔑 The needles are BUILT from the parameter rather than spelled, so the guard cannot
        // find itself — `the_perimeter_has_a_single_reader` twelve screens above was measured
        // reporting its own matcher as a reader before it was written this way.
        fn variants(source: &str, enum_name: &str) -> Vec<String> {
            let head = format!("enum {enum_name} {{");
            let start = source
                .find(&head)
                .unwrap_or_else(|| panic!("no declaration of {enum_name} in this file"))
                + head.len();
            let rest = &source[start..];
            let end = rest
                .find("\n}")
                .unwrap_or_else(|| panic!("{enum_name}'s declaration is not closed"));
            rest[..end]
                .lines()
                .map(str::trim)
                .filter(|line| {
                    !line.is_empty() && !line.starts_with("//") && !line.starts_with('#')
                })
                .filter_map(|line| line.strip_suffix(','))
                .filter(|name| {
                    !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_')
                })
                .map(str::to_string)
                .collect()
        }

        fn listed(source: &str, enum_name: &str) -> Vec<String> {
            let head = format!("const ALL: [{enum_name}; ");
            let start = source
                .find(&head)
                .unwrap_or_else(|| panic!("no ALL for {enum_name}"));
            let rest = &source[start..];
            let open = rest.find("= [").expect("ALL's array must open") + 3;
            let close = rest[open..].find("];").expect("ALL's array must close");
            let qualified = format!("{enum_name}::");
            rest[open..open + close]
                .split(&qualified)
                .skip(1)
                .map(|tail| {
                    tail.chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect()
                })
                .collect()
        }

        // 🔑 **Three files, not one, since story 6b.6.** The guard read `screens.rs` alone while
        // its own doc called the hole it closes *"the variant that never reaches `ALL`"* — a
        // property of every enum with an `ALL`, not of this file. `ObjectState::ALL` is
        // `#[cfg(test)]`, which makes the hole WIDER there rather than narrower: nothing in
        // production would notice the omission at all.
        // (source, enum, the count below which the parse itself is suspect)
        for (source, enum_name, floor) in [
            (include_str!("screens.rs"), "Screen", 10_usize),
            (include_str!("screens.rs"), "NavGroup", 3),
            (include_str!("state_vocabulary.rs"), "ObjectState", 5),
            (include_str!("example_data.rs"), "DeviceKind", 7),
            // 🔴 **A CROSS-CRATE ROW, and it is the ONLY thing that catches an eighth `FactKind`.**
            // `FactKind` is `#[non_exhaustive]`, so `opencmdb-bin` cannot match it exhaustively —
            // the `_` arm the compiler demands is permanently silent, and a new kind would drop
            // out of `/sources`' *what this source cannot see* list without a word. Story 6b.8's
            // validation measured a planted eighth variant passing the suite, clippy and all eight
            // gates; with this row it reds by name.
            // ⚠️ `include_str!` is a COMPILE-TIME file read, not a dependency: the `frontier` gate
            // stays green, verified. And if `observation/mod.rs` MOVES, this row does **not** go
            // blind — the crate fails to compile, loudly. 🔑 *The doc said "goes blind" until the
            // code review; that word belongs to a guard that keeps passing while checking nothing,
            // which is the opposite of what a missing `include_str!` path does.* What would make it
            // blind is the enum being RENAMED, which the row's own name argument would then miss.
            (
                include_str!("../../opencmdb-core/src/observation/mod.rs"),
                "FactKind",
                7,
            ),
        ] {
            let declared = variants(source, enum_name);
            let in_all = listed(source, enum_name);

            // 🔑 The premise FIRST: a parser that silently returned nothing would make every
            // assertion below vacuously true, which is the shape this project keeps catching.
            assert!(
                declared.len() >= floor,
                "the premise: {enum_name} must declare at least {floor} variants and the parse \
                 found {} — a guard reading nothing asserts nothing",
                declared.len()
            );

            for variant in &declared {
                assert!(
                    in_all.contains(variant),
                    // ⚠️ The consequence is named GENERICALLY since story 6b.8 added a
                    // `FactKind` row: this message said *"it vanishes from the navigation, the
                    // routing"*, which is true for `Screen` and meaningless for an enum that has
                    // nothing to do with either — the review layer that planted an eighth
                    // `FactKind` read the red and found the text describing another enum's world.
                    // *A guard that fires with the wrong explanation sends its reader looking in
                    // the wrong place.*
                    "{enum_name}::{variant} is declared but absent from {enum_name}::ALL. It \
                     compiles, and it then vanishes from everything that iterates that list — \
                     the navigation for `Screen`, the *what this source cannot see* complement \
                     for `FactKind` — with no error and no warning anywhere"
                );
            }
            // The other direction: an entry naming a variant that no longer exists cannot compile,
            // but one whose spelling drifts inside a comment or a doc example can — and an ALL
            // longer than the enum means the parse is reading something it should not.
            for entry in &in_all {
                assert!(
                    declared.contains(entry),
                    "{enum_name}::ALL names {entry:?}, which this file does not declare"
                );
            }
        }
    }
}
