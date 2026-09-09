//! The dashboard's state, shaped for the page — the daily overview (story 6b.5).
//!
//! # Why this is a module of its own
//!
//! 🔴 **`page.rs` reached the `file-size` gate's 2000-line ceiling for the third time** — at 2016,
//! with the gate naming it — when `/devices` gained a handler that reads the store. `CLAUDE.md`'s
//! rule is *split, not grown*, and this is the split, on `identity_view.rs`'s and
//! `sources_view.rs`'s precedent: the dashboard has its own types and its own doctrine, so it
//! comes out whole rather than by line count. ⚠️ **Its TESTS stay in `page.rs`**, where the render
//! helpers they share live; the ceiling counts the lines before the first `#[cfg(test)]`, so a
//! test module here would have bought nothing.
//!
//! # What the screen is FOR
//!
//! It was the product's first MIXED screen (story 6b.5): the identity engine's real reach beside
//! two labelled example sections. ⚠️ Its own review recorded that the invented cards are
//! **visually dominant** over the honest section — 22 px figures with sparklines against grey prose
//! — and that finding is registered rather than closed. `/devices` joined it on 2026-09-09 and
//! inherits the same risk, which is why its real half is rendered FIRST.

use askama::Template;

use crate::identity_view::IdentityView;
use crate::page::{Strings, relative_time};

/// One example figure on the dashboard, with its copy already resolved.
pub(crate) struct StatCardView {
    /// What it counts, in the operator's language.
    pub(crate) label: String,
    /// The figure itself — a STRING, because it is decoration and never arithmetic.
    pub(crate) value: &'static str,
    /// The shape a sparkline would draw, as a bare list of heights.
    pub(crate) spark: Vec<u8>,
}

/// Everything `/dashboard` renders: the product's real reach, and the example surfaces beside it.
///
/// 🔴 **Two populations, never summed** (arbitration 10) — and the guard for that lives at the
/// COMPOSITION and not on either builder, because story 5.14b measured its own guard GREEN when it
/// asserted a property two pure builders cannot violate: neither sees the other's numbers, so
/// neither can add them. This struct is where they meet, so this is where a sum could be written.
pub(crate) struct DashboardView {
    /// The real reach, exactly as story 5.14b shipped it and `/triage` renders it.
    pub(crate) identity: IdentityView,
    /// How long ago the product last observed anything — a `MAX(observed_at)`, in the BODY only.
    pub(crate) last_observed: Option<String>,
    /// The example figures. ⚠️ Example, and each carries the marker on its own section.
    pub(crate) cards: Vec<StatCardView>,
    /// True when something HAS been observed and the identity pass has not placed any of it yet.
    ///
    /// 🔴 **This story is what created the state that needs saying.** It co-located two populations
    /// for the first time — the engine's reach and the last observation — and they can legitimately
    /// disagree: an observation ingested but not yet resolved leaves `count_engine_reach` empty
    /// while `MAX(observed_at)` is recent. The page then read *"Nothing observed yet — run a scan"*
    /// directly above *"Last observed 8 h ago"*, in one div. Found at the code review by seeding the
    /// two independently; **every test fed them from one fixture, so none could see it.**
    pub(crate) pending_resolution: bool,
}

/// The dashboard's body: the real reach section beside labelled example sections (story 6b.5).
#[derive(Template)]
#[template(path = "_dashboard.html")]
pub(crate) struct DashboardBody {
    /// 🔴 **ONE source for the identity counts, and that is not tidying.** This struct carried its
    /// own `identity` field beside `view.identity` until the mutation pass: the handler filled it
    /// from the view and the TEST HELPER filled it from the un-composed original, so **the guard
    /// rendered a shape production does not use** and mutation M1 — a sum planted at the
    /// composition — left it green. *Two fields holding one fact will be filled from two places,
    /// and the test's place is the one nobody ships.* The template now reads `view.identity`.
    pub(crate) view: DashboardView,
    pub(crate) s: Strings,
}

/// PURE: the example half. It reads nothing and depends on nothing.
pub(crate) fn example_cards() -> Vec<StatCardView> {
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
pub(crate) fn build_dashboard(
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
