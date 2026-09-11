//! `/ipam` — the addressing plan, drawn from the store.
//!
//! # What this screen shows, and what it deliberately does not
//!
//! It draws the PLAN: the subnets, ranges and addresses the operator declared, and nothing else.
//! **It reads no observation.** The audit — *which addresses are in use but not in the plan* — is
//! story 14.3's, and a join written here would be that story's deliverable arriving early and
//! unmeasured. `AC2`'s guard is a mutation: give this module an observation read and it reds.
//!
//! # The cell vocabulary, and why `free` is not one of the four binding words
//!
//! The PLAN axis (`static` · `dhcp-pool` · `reserved` · `infrastructure`, ratified 2026-09-11 by
//! PR #166) says what a stretch of address space is MEANT FOR. It does not say whether one
//! particular address inside it has been spoken for, and that is a second, orthogonal question this
//! screen must answer per cell. So a cell carries a POLICY (from its range) and a STATE:
//!
//! - **defined** — an `ip_address` row names it;
//! - **free** — it sits inside a range and nothing names it;
//! - **infrastructure** — the network or broadcast address of the subnet, covered by no range.
//!   🔑 This is DERIVED, and it is not a second concept: the binding table defines `infrastructure`
//!   as *"Gateway, equipment management, and the network and broadcast addresses"*. `structural`
//!   was the word story 6b.7 minted for exactly these two cells and Guy retired it on 2026-09-11 —
//!   *the arithmetic is how those rows get there, not a second concept*;
//! - **not covered** — outside every range, and the plan says nothing. ⚠️ It carries **no noun and
//!   no legend entry**, because a story may not extend the binding vocabulary and *not covered* is
//!   not one of its four words.
//!
//! 🔴 **`free` and *not covered* had to be told apart, and on the shipped stylesheet they could
//! not be.** Measured in Chrome against `app.css` as story 14.1 left it, a `.ipam-cell-free` and a
//! bare `.ipam-cell` are byte-identical in every computed property — same transparent background,
//! same border, same box. Not *hard to tell apart by colour*, which WCAG 1.4.1 would already
//! forbid: **indistinguishable in every property there is.** So `free` gained a treatment of its
//! own and the bare cell keeps the blank (Guy, 2026-09-11, re-arbitrated on that measurement).

use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use opencmdb_core::ipam::IpPolicy;
use sqlx::MySqlPool;
use std::net::Ipv4Addr;

use crate::ipam_repo::{self, Subnet};
use crate::page::{render_shell, Shell};
use crate::screens::Screen;

/// What one address in the grid is, as the PLAN says it.
///
/// The variants are ordered as the renderer resolves them, and **that order is a rendering
/// decision that must not double as a repair** — story 6b.7's own recorded finding, where
/// `state_of` tested `used` before `reserved` and an octet in both lists rendered silently as
/// *used*, a corruption no test could see. Here the cases are disjoint by construction except one:
/// an address may be individually defined AND inside a range, which is legal (measured: the
/// adapter accepts it in both orders) and is drawn as **defined, carrying its range's policy**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CellState {
    /// An `ip_address` row names this address. It carries its range's policy when it has one.
    Defined(Option<IpPolicy>),
    /// Inside a range, and no `ip_address` row names it.
    Free(IpPolicy),
    /// The subnet's network or broadcast address, covered by no range.
    Infrastructure,
    /// Outside every range. The plan says nothing about it, and neither does this cell.
    NotCovered,
}

impl CellState {
    /// The CSS modifier this state renders with.
    ///
    /// ⚠️ These strings are invisible to `every_class_a_template_names_is_defined_in_the_stylesheet`,
    /// which skips any `class="…"` containing `{` — the cell is `class="ipam-cell {{ … }}"`. What
    /// makes them visible is the LEGEND's literals, pinned by
    /// `the_legend_names_every_cell_modifier_as_a_literal`. *`NotCovered` has no legend entry by
    /// decision, so it is the one modifier no literal covers, and its test says so.*
    pub(crate) fn modifier(self) -> &'static str {
        match self {
            Self::Defined(_) => "ipam-cell-defined",
            Self::Free(_) => "ipam-cell-free",
            Self::Infrastructure => "ipam-cell-infrastructure",
            Self::NotCovered => "ipam-cell-not-covered",
        }
    }

    /// The i18n key naming this state to the operator.
    pub(crate) fn label_key(self) -> &'static str {
        match self {
            Self::Defined(_) => "ipam.state.defined",
            Self::Free(_) => "ipam.state.free",
            Self::Infrastructure => "ipam.state.infrastructure",
            Self::NotCovered => "ipam.state.not_covered",
        }
    }

    /// The policy this cell inherits from its range, when it has one.
    pub(crate) fn policy(self) -> Option<IpPolicy> {
        match self {
            Self::Defined(policy) => policy,
            Self::Free(policy) => Some(policy),
            Self::Infrastructure | Self::NotCovered => None,
        }
    }

    /// Whether this state may be offered as the next free address.
    ///
    /// 🔑 Only [`Self::Free`] may, and `infrastructure` may not — the binding table says
    /// *"never offered as free"* of it in so many words. ⚠️ **This is the PLAN's answer and not the
    /// network's**: story 14.3 must additionally exclude every OBSERVED address, which is the
    /// criterion the whole epic exists for and the only place the product prevents a duplicate
    /// rather than reporting it.
    pub(crate) fn offerable(self) -> bool {
        matches!(self, Self::Free(_))
    }
}

/// One subnet's plan, derived from the store's rows and from nothing else.
///
/// It is a pure function of its arguments: no clock, no database, no locale. That is what lets the
/// whole grid be tested without a store, and it is why the handler is three lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlanView {
    /// One entry per address in the subnet, in numeric order.
    pub(crate) cells: Vec<(Ipv4Addr, CellState)>,
}

impl PlanView {
    /// Derive the plan for one subnet.
    ///
    /// `ranges` are `(first, last, policy)` and `defined` are the individually-defined addresses.
    /// Overlapping ranges cannot occur — the adapter refuses them — so the first range containing
    /// an address is the only one that does.
    pub(crate) fn derive(
        subnet: Subnet,
        ranges: &[(Ipv4Addr, Ipv4Addr, IpPolicy)],
        defined: &[Ipv4Addr],
    ) -> Self {
        let cells = subnet
            .addresses()
            .map(|addr| {
                let policy = ranges
                    .iter()
                    .find(|(first, last, _)| addr >= *first && addr <= *last)
                    .map(|(_, _, policy)| *policy);
                let state = if defined.contains(&addr) {
                    CellState::Defined(policy)
                } else if let Some(policy) = policy {
                    CellState::Free(policy)
                } else if subnet.is_edge(addr) {
                    CellState::Infrastructure
                } else {
                    CellState::NotCovered
                };
                (addr, state)
            })
            .collect();
        Self { cells }
    }

    /// The lowest address the PLAN can offer, or `None` when it can offer none.
    pub(crate) fn next_offerable(&self) -> Option<Ipv4Addr> {
        self.cells
            .iter()
            .find(|(_, state)| state.offerable())
            .map(|(addr, _)| *addr)
    }

    /// How many cells carry each state, as a LIST and never a ratio.
    ///
    /// 🔑 `example_data.rs:829` recorded the reason before this screen was real: a percentage
    /// invites a reader to compare two subnets whose sizes differ, and the occupancy of a plan is
    /// not a rate. It is also why `ipam/` stays outside the `float-free` gate — there is no
    /// arithmetic here beyond a count.
    pub(crate) fn counts(&self) -> (usize, usize, usize, usize) {
        let mut defined = 0;
        let mut free = 0;
        let mut infrastructure = 0;
        let mut not_covered = 0;
        for (_, state) in &self.cells {
            match state {
                CellState::Defined(_) => defined += 1,
                CellState::Free(_) => free += 1,
                CellState::Infrastructure => infrastructure += 1,
                CellState::NotCovered => not_covered += 1,
            }
        }
        (defined, free, infrastructure, not_covered)
    }
}

/// The `/ipam` sub-router's state.
#[derive(Clone)]
pub(crate) struct IpamState {
    /// The store the plan is read from.
    pool: MySqlPool,
    /// The configured scan perimeter, for the shell's header.
    perimeter: Option<String>,
}

/// The query string `/ipam` accepts.
#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct IpamQuery {
    /// The subnet to draw, by its id. Absent or unknown selects the first.
    ///
    /// ⚠️ **It was a SLUG until story 14.2** — `ExampleSubnet::slug`, from a dataset that no longer
    /// exists. `ip_subnet` has no slug column, so the selector's key had to change meaning, and the
    /// id is what the store actually holds.
    pub(crate) subnet: Option<String>,
}

/// Mount `/ipam` on its own router, with the pool.
///
/// 🔑 It is NOT registered on `page::triage_router`, and the reason is a measured ceiling rather
/// than taste: `page.rs` is at 1954 code lines of the 2000 the `file-size` gate allows — 46 of
/// headroom — so `CLAUDE.md`'s *"split, not grown"* applies before the growth, not after it.
pub(crate) fn router(pool: MySqlPool, perimeter: Option<String>) -> Router {
    Router::new()
        .route("/ipam", get(ipam))
        .with_state(IpamState { pool, perimeter })
}

/// Serve the plan.
async fn ipam(State(state): State<IpamState>, Query(query): Query<IpamQuery>) -> Response {
    match plan_body(&state.pool, query.subnet.as_deref()).await {
        Ok(body) => Html(render_shell(
            Shell::new(Screen::Ipam, state.perimeter.clone()),
            body,
        ))
        .into_response(),
        Err(body) => Html(render_shell(
            Shell::new(Screen::Ipam, state.perimeter.clone()),
            body,
        ))
        .into_response(),
    }
}

/// Read the plan and render it, or render why it could not be read.
async fn plan_body(pool: &MySqlPool, selected: Option<&str>) -> Result<String, String> {
    let subnets = ipam_repo::list_subnets(pool)
        .await
        .map_err(|_| crate::page::render_error_body())?;
    if subnets.is_empty() {
        return Ok(empty_plan_body());
    }
    let (id, subnet, _label) = subnets
        .iter()
        .find(|(id, _, _)| Some(id.as_str()) == selected)
        .or_else(|| subnets.first())
        .cloned()
        .expect("a non-empty list has a first element");
    let ranges = ipam_repo::ranges_in(pool, &id)
        .await
        .map_err(|_| crate::page::render_error_body())?;
    let defined = ipam_repo::addresses_in(pool, &id)
        .await
        .map_err(|_| crate::page::render_error_body())?;
    let bounds: Vec<(Ipv4Addr, Ipv4Addr, IpPolicy)> = ranges
        .iter()
        .map(|(first, last, policy, _)| (*first, *last, *policy))
        .collect();
    let plan = PlanView::derive(subnet, &bounds, &defined);
    Ok(render_plan(&subnets, &id, &plan))
}

/// The sentence an empty plan shows, and the door it opens.
fn empty_plan_body() -> String {
    String::new()
}

/// Render one subnet's grid.
fn render_plan(
    _subnets: &[(String, Subnet, String)],
    _selected: &str,
    _plan: &PlanView,
) -> String {
    String::new()
}
