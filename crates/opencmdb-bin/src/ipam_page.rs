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

use askama::Template;
use axum::Router;
use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use opencmdb_core::ipam::IpPolicy;
use sqlx::MySqlPool;
use std::net::Ipv4Addr;

use crate::ipam_repo::{self, Subnet};
use crate::page::{Shell, render_shell};
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
            // 🔑 The POLICY key, not a state key of its own: `.0` and `.255` ARE infrastructure by
            // the binding table's own definition, and giving them a second word is the synonym
            // problem that table exists to prevent. `structural` was that second word until
            // 2026-09-11.
            Self::Infrastructure => "ipam.policy.infrastructure",
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
    /// 🔴 **A CELL CAN BE `Free` AND STILL NOT OFFERABLE, and the first draft of this function got
    /// it wrong.** It read `matches!(self, Self::Free(_))`, which offered `192.0.2.0` the moment an
    /// operator declared a range with the `infrastructure` policy over it — measured by the test
    /// below, which reddened on its first run with `left: Some(192.0.2.0)`. The binding table says
    /// of `infrastructure`: *"Not host space; **never offered as free**"*, in those words.
    ///
    /// 🔑 *The state and the policy are two axes, and `free` is a statement about the STATE alone.*
    /// An address inside a declared infrastructure range is free of any individual claim and is
    /// still not the operator's to assign — which is exactly why the two axes exist rather than one
    /// flattened enum.
    ///
    /// ⚠️ **This is the PLAN's answer and not the network's**: story 14.3 must additionally exclude
    /// every OBSERVED address, which is the criterion the whole epic exists for and the only place
    /// the product prevents a duplicate rather than reporting it.
    pub(crate) fn offerable(self) -> bool {
        matches!(self, Self::Free(policy) if policy != IpPolicy::Infrastructure)
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
///
/// ⚠️ **Budgeted, and the guard that demands it is DERIVED from `Screen::ALL`** — which is why this
/// story could not forget it: the moment `Screen::Ipam` became `Nature::Fed`,
/// `every_store_backed_screen_refuses_within_the_page_budget` reddened with *"must refuse rather
/// than hang"*, before any human noticed the route had gained a database. Story 6b.8 shipped
/// `/sources` WITHOUT a budget and no review layer found it; the derived guard measured **30.00 s**
/// the first time it ran. *A guard built from an enum covers the screen nobody thought to check.*
async fn ipam(State(state): State<IpamState>, Query(query): Query<IpamQuery>) -> Response {
    let read = crate::page::store_within(crate::page::PAGE_STORE_BUDGET, async {
        plan_data(&state.pool, query.subnet.as_deref())
            .await
            .map_err(|error| {
                tracing::error!(%error, "reading the addressing plan");
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    axum::response::Html(crate::page::render_error_body()),
                )
                    .into_response()
            })
    })
    .await;
    let body = match read {
        Ok(data) => data,
        Err(response) => return response,
    };
    Html(render_shell(
        Shell::new(Screen::Ipam, state.perimeter.clone()),
        body,
    ))
    .into_response()
}

/// Read the plan and render it.
///
/// # Errors
///
/// The store's own failure, classified. A plan with no subnet at all is NOT an error — it is the
/// ordinary state of a fresh install, and it renders the sentence that says so.
async fn plan_data(
    pool: &MySqlPool,
    selected: Option<&str>,
) -> Result<String, opencmdb_core::repo::RepositoryError> {
    let subnets = ipam_repo::list_subnets(pool).await?;
    if subnets.is_empty() {
        return Ok(empty_plan_body());
    }
    // 🔑 An unknown id narrows to NOTHING rather than falling back to the first subnet, on
    // `inventory_body`'s precedent: silently serving another subnet would tell the operator their
    // selection took when it did not — story 6b.4's `?sort=` finding.
    let chosen = match selected {
        Some(wanted) => subnets.iter().find(|(id, _, _)| id == wanted).cloned(),
        None => subnets.first().cloned(),
    };
    let Some((id, subnet, _label)) = chosen else {
        return Ok(unknown_subnet_body(&subnets));
    };
    let ranges = ipam_repo::ranges_in(pool, &id).await?;
    let defined = ipam_repo::addresses_in(pool, &id).await?;
    let bounds: Vec<(Ipv4Addr, Ipv4Addr, IpPolicy)> = ranges
        .iter()
        .map(|(first, last, policy, _)| (*first, *last, *policy))
        .collect();
    let plan = PlanView::derive(subnet, &bounds, &defined);
    Ok(render_plan(&subnets, &id, &plan))
}

/// Every string the template renders, resolved once.
///
/// 🔑 Fields, never a map: a missing field is a compile error where a missing map entry renders an
/// unresolved key name to the operator — story 6b.6 shipped two of those and no guard could see
/// them, `rust-i18n` rendering an unknown key verbatim.
#[derive(Debug, Clone)]
pub(crate) struct IpamStrings {
    title: String,
    lede: String,
    selector_label: String,
    grid_label: String,
    state_defined: String,
    state_free: String,
    policy_static: String,
    policy_dhcp_pool: String,
    policy_reserved: String,
    policy_infrastructure: String,
    occupancy: String,
    next_free_label: String,
    next_free: String,
    next_free_caveat: String,
    empty_plan: String,
    empty_plan_gesture: String,
    unknown_subnet: String,
    gesture_badge: String,
    gesture_not_built: String,
}

/// One cell, ready to render.
#[derive(Debug, Clone)]
pub(crate) struct CellView {
    /// The state's CSS modifier.
    modifier: &'static str,
    /// The policy's CSS modifier, or the empty string when no range covers this address.
    policy_modifier: &'static str,
    /// The accessible name: the address, its state, and its policy when it has one.
    label: String,
}

/// One entry in the subnet selector.
#[derive(Debug, Clone)]
pub(crate) struct SubnetTab {
    /// The subnet's id — the selector's key since story 14.2, a slug before it.
    id: String,
    /// What the operator reads: the CIDR, then the label when there is one.
    label: String,
    /// Whether this is the subnet in force.
    active: bool,
}

/// The grid, ready to render.
#[derive(Debug, Clone)]
pub(crate) struct PlanRender {
    /// One entry per address, in numeric order.
    cells: Vec<CellView>,
}

/// The page.
#[derive(askama::Template)]
#[template(path = "_ipam.html")]
pub(crate) struct IpamBody {
    /// The strings.
    s: IpamStrings,
    /// The selector.
    tabs: Vec<SubnetTab>,
    /// The grid, or `None` when the plan holds no subnet at all.
    plan: Option<PlanRender>,
}

/// The CSS modifier for a policy.
fn policy_modifier(policy: IpPolicy) -> &'static str {
    match policy {
        IpPolicy::Static => "ipam-policy-static",
        IpPolicy::DhcpPool => "ipam-policy-dhcp-pool",
        IpPolicy::Reserved => "ipam-policy-reserved",
        IpPolicy::Infrastructure => "ipam-policy-infrastructure",
    }
}

/// The i18n key naming a policy to the operator.
///
/// 🔴 These four keys ARE the binding table's UI column (PR #166) and
/// [`IpPolicy::as_str`] is its code column. `the_policy_words_are_the_binding_tables_own` compares
/// the two as a SET, because *a count is not a set* (story 6.5's M8).
fn policy_key(policy: IpPolicy) -> &'static str {
    match policy {
        IpPolicy::Static => "ipam.policy.static",
        IpPolicy::DhcpPool => "ipam.policy.dhcp_pool",
        IpPolicy::Reserved => "ipam.policy.reserved",
        IpPolicy::Infrastructure => "ipam.policy.infrastructure",
    }
}

/// Build the body for a plan that holds no subnet at all.
fn empty_plan_body() -> String {
    let body = IpamBody {
        s: strings(None, None),
        tabs: Vec::new(),
        plan: None,
    };
    body.render()
        .unwrap_or_else(|_| crate::page::render_error_body())
}

/// Build the body for an identifier no subnet carries.
fn unknown_subnet_body(subnets: &[(String, Subnet, String)]) -> String {
    let body = IpamBody {
        s: strings(None, None),
        tabs: subnets
            .iter()
            .map(|(id, subnet, label)| SubnetTab {
                id: id.clone(),
                label: tab_label(subnet, label),
                active: false,
            })
            .collect(),
        plan: None,
    };
    body.render()
        .unwrap_or_else(|_| crate::page::render_error_body())
}

/// What the operator reads on a selector tab.
fn tab_label(subnet: &Subnet, label: &str) -> String {
    if label.is_empty() {
        subnet.cidr()
    } else {
        format!("{} · {}", subnet.cidr(), label)
    }
}

/// Resolve every string, with the occupancy and next-free lines when there is a plan.
fn strings(counts: Option<(usize, usize, usize, usize)>, next: Option<Ipv4Addr>) -> IpamStrings {
    let occupancy = match counts {
        Some((defined, free, infrastructure, not_covered)) => rust_i18n::t!(
            "ipam.occupancy",
            defined = defined,
            free = free,
            infrastructure = infrastructure,
            not_covered = not_covered
        )
        .to_string(),
        None => String::new(),
    };
    let next_free = match next {
        Some(addr) => addr.to_string(),
        None => rust_i18n::t!("ipam.next_free_none").to_string(),
    };
    IpamStrings {
        title: rust_i18n::t!("ipam.title").to_string(),
        lede: rust_i18n::t!("ipam.lede").to_string(),
        selector_label: rust_i18n::t!("ipam.selector_label").to_string(),
        grid_label: rust_i18n::t!("ipam.grid_label").to_string(),
        state_defined: rust_i18n::t!("ipam.state.defined").to_string(),
        state_free: rust_i18n::t!("ipam.state.free").to_string(),
        policy_static: rust_i18n::t!("ipam.policy.static").to_string(),
        policy_dhcp_pool: rust_i18n::t!("ipam.policy.dhcp_pool").to_string(),
        policy_reserved: rust_i18n::t!("ipam.policy.reserved").to_string(),
        policy_infrastructure: rust_i18n::t!("ipam.policy.infrastructure").to_string(),
        occupancy,
        next_free_label: rust_i18n::t!("ipam.next_free").to_string(),
        next_free,
        next_free_caveat: rust_i18n::t!("ipam.next_free_caveat").to_string(),
        empty_plan: rust_i18n::t!("ipam.empty_plan").to_string(),
        empty_plan_gesture: rust_i18n::t!("ipam.empty_plan_gesture").to_string(),
        unknown_subnet: rust_i18n::t!("ipam.unknown_subnet").to_string(),
        gesture_badge: rust_i18n::t!("gesture.badge").to_string(),
        gesture_not_built: rust_i18n::t!(
            "gesture.not_built",
            badge = rust_i18n::t!("gesture.badge")
        )
        .to_string(),
    }
}

/// Render one subnet's grid.
pub(crate) fn render_plan(
    subnets: &[(String, Subnet, String)],
    selected: &str,
    plan: &PlanView,
) -> String {
    let tabs = subnets
        .iter()
        .map(|(id, subnet, label)| SubnetTab {
            id: id.clone(),
            label: tab_label(subnet, label),
            active: id == selected,
        })
        .collect();
    let cells = plan
        .cells
        .iter()
        .map(|(addr, state)| {
            let state_word = rust_i18n::t!(state.label_key()).to_string();
            let label = match state.policy() {
                Some(policy) => rust_i18n::t!(
                    "ipam.cell_label_in_policy",
                    address = addr.to_string(),
                    state = state_word,
                    policy = rust_i18n::t!(policy_key(policy))
                )
                .to_string(),
                None => rust_i18n::t!(
                    "ipam.cell_label",
                    address = addr.to_string(),
                    state = state_word
                )
                .to_string(),
            };
            CellView {
                modifier: state.modifier(),
                policy_modifier: state.policy().map(policy_modifier).unwrap_or(""),
                label,
            }
        })
        .collect();
    let body = IpamBody {
        s: strings(Some(plan.counts()), plan.next_offerable()),
        tabs,
        plan: Some(PlanRender { cells }),
    };
    body.render()
        .unwrap_or_else(|_| crate::page::render_error_body())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `/24` for the tests, with its two edges and 254 hosts.
    fn office() -> Subnet {
        Subnet::new("192.0.2.0".parse().unwrap(), 24).expect("a /24")
    }

    fn v4(text: &str) -> Ipv4Addr {
        text.parse().expect("a v4 address")
    }

    /// 🔴 **256 ADDRESSES, 254 HOSTS, and the reference mock conflated them** — it looped `0..256`,
    /// drew `.0` and `.255` as ordinary free cells, and its *next free address* panel then named
    /// the NETWORK address, reproduced on a real build at story 6b.7's validation. ⚠️ That story's
    /// own first draft prescribed a test asserting the counts *"sum to 256"*, which would have
    /// pinned the defect as the expected behaviour.
    #[test]
    fn the_grid_draws_every_address_and_offers_only_hosts() {
        let plan = PlanView::derive(
            office(),
            &[(v4("192.0.2.0"), v4("192.0.2.255"), IpPolicy::Static)],
            &[],
        );
        assert_eq!(plan.cells.len(), 256, "a /24 holds 256 addresses");
        let offerable = plan.cells.iter().filter(|(_, s)| s.offerable()).count();
        assert_eq!(
            offerable, 256,
            "a range that covers the whole subnet makes every cell free — the EDGES are only \
             infrastructure when no range covers them, which is what the next assertion measures"
        );

        // The ordinary shape: a range over the hosts, the edges covered by nothing.
        let plan = PlanView::derive(
            office(),
            &[(v4("192.0.2.1"), v4("192.0.2.254"), IpPolicy::Static)],
            &[],
        );
        assert_eq!(plan.cells.len(), 256);
        assert_eq!(
            plan.cells.iter().filter(|(_, s)| s.offerable()).count(),
            254,
            "the network and broadcast addresses are never offerable"
        );
        assert_eq!(plan.cells[0].1, CellState::Infrastructure, "the .0");
        assert_eq!(plan.cells[255].1, CellState::Infrastructure, "the .255");
        assert_eq!(
            plan.next_offerable(),
            Some(v4("192.0.2.1")),
            "the lowest free host, and never the network address"
        );
    }

    /// 🔑 **`infrastructure` is NEVER OFFERED AS FREE** — the binding table says so in those words,
    /// and it is the one policy whose whole point is that it is not host space.
    #[test]
    fn infrastructure_is_never_offered_as_free() {
        let plan = PlanView::derive(
            office(),
            &[
                (v4("192.0.2.0"), v4("192.0.2.9"), IpPolicy::Infrastructure),
                (v4("192.0.2.10"), v4("192.0.2.254"), IpPolicy::Static),
            ],
            &[],
        );
        assert_eq!(
            plan.next_offerable(),
            Some(v4("192.0.2.10")),
            "the offer must skip the infrastructure range entirely"
        );
        // ⚠️ A range whose policy is `infrastructure` renders as FREE-with-that-policy, not as the
        // derived `Infrastructure` state — the derived state is for edges no range covers. The two
        // are the same WORD by decision (the binding table covers both) and different STATES,
        // because one is declared and the other is arithmetic. This assertion is what stops a
        // future refactor fusing them.
        assert!(
            matches!(plan.cells[0].1, CellState::Free(IpPolicy::Infrastructure)),
            "a declared infrastructure range is a covered cell, not a derived edge"
        );
        assert!(
            !plan.cells[0].1.offerable(),
            "and it is still not offerable"
        );
    }

    /// 🔴 **A PRIORITY ORDER IS A RENDERING DECISION AND MUST NOT DOUBLE AS A REPAIR** — story
    /// 6b.7's own finding, where `state_of` tested `used` before `reserved` and an octet in BOTH
    /// lists rendered silently as *used*: a deliberate corruption of the dataset changed no cell,
    /// no count and no test. Here the one overlapping case is legal — the adapter accepts an
    /// address inside a range, measured in both orders — so the decision is asserted rather than
    /// left to the order of an `if`.
    #[test]
    fn an_address_both_defined_and_in_a_range_is_defined_and_keeps_its_policy() {
        let plan = PlanView::derive(
            office(),
            &[(v4("192.0.2.1"), v4("192.0.2.100"), IpPolicy::DhcpPool)],
            &[v4("192.0.2.50")],
        );
        let (_, state) = plan.cells[50];
        assert_eq!(
            state,
            CellState::Defined(Some(IpPolicy::DhcpPool)),
            "defined wins over free, and the policy is carried rather than lost"
        );
        assert!(!state.offerable(), "a defined address is not on offer");
    }

    /// A cell outside every range says the plan is silent, and says it WITHOUT A NOUN.
    #[test]
    fn a_cell_outside_every_range_is_not_covered() {
        let plan = PlanView::derive(
            office(),
            &[(v4("192.0.2.1"), v4("192.0.2.9"), IpPolicy::Static)],
            &[],
        );
        assert_eq!(plan.cells[10].1, CellState::NotCovered);
        assert_eq!(plan.cells[10].1.policy(), None);
        assert!(!plan.cells[10].1.offerable());
        let (defined, free, infrastructure, not_covered) = plan.counts();
        assert_eq!((defined, free, infrastructure), (0, 9, 2));
        assert_eq!(not_covered, 245);
        assert_eq!(
            defined + free + infrastructure + not_covered,
            256,
            "every cell is accounted for exactly once"
        );
    }

    /// 🔴 **THE FOUR POLICY WORDS ARE THE BINDING TABLE'S OWN, COMPARED AS A SET.**
    ///
    /// *A count is not a set* — story 6.5's M8, where pointing two enum variants at one token left
    /// a count-based guard green with one variant unreachable. Here the two representations are
    /// `IpPolicy::as_str` (the code column) and `ipam.policy.*` (the UI column), and the test
    /// asserts they are in bijection rather than merely equinumerous.
    #[test]
    fn every_policy_carries_its_own_key_and_its_own_modifier() {
        use std::collections::BTreeSet;
        let keys: BTreeSet<&str> = IpPolicy::ALL.into_iter().map(policy_key).collect();
        let modifiers: BTreeSet<&str> = IpPolicy::ALL.into_iter().map(policy_modifier).collect();
        assert_eq!(keys.len(), IpPolicy::ALL.len(), "two policies share a key");
        assert_eq!(
            modifiers.len(),
            IpPolicy::ALL.len(),
            "two policies share a CSS modifier, so the grid cannot tell them apart"
        );
        // ⚠️ And the key must be the POLICY namespace, not a state one: `structural` was retired
        // precisely because two adjacent words for one concept sat on one screen.
        for policy in IpPolicy::ALL {
            assert!(
                policy_key(policy).starts_with("ipam.policy."),
                "{policy} must be named by the binding table's own namespace"
            );
        }
    }

    /// Every state has its own modifier and its own key — the grid's other axis.
    #[test]
    fn every_state_carries_its_own_key_and_its_own_modifier() {
        use std::collections::BTreeSet;
        let states = [
            CellState::Defined(None),
            CellState::Free(IpPolicy::Static),
            CellState::Infrastructure,
            CellState::NotCovered,
        ];
        let modifiers: BTreeSet<&str> = states.iter().map(|s| s.modifier()).collect();
        assert_eq!(modifiers.len(), states.len(), "two states share a modifier");
        let keys: BTreeSet<&str> = states.iter().map(|s| s.label_key()).collect();
        assert_eq!(keys.len(), states.len(), "two states share a key");
    }

    /// 🔴 **THE STYLESHEET MUST DEFINE EVERY MODIFIER THE GRID CAN EMIT, and the project's own
    /// class guard CANNOT SEE THEM**: `every_class_a_template_names_is_defined_in_the_stylesheet`
    /// skips any `class="…"` containing `{`, and the cell is `class="ipam-cell {{ … }}"`. What
    /// covers the four legend entries is their LITERALS in the template; what covers the rest is
    /// this test, which reads the sheet and the code rather than the markup.
    #[test]
    fn the_stylesheet_defines_every_modifier_the_grid_can_emit() {
        let css = include_str!("../assets/app.css");
        let states = [
            CellState::Defined(None),
            CellState::Free(IpPolicy::Static),
            CellState::Infrastructure,
            CellState::NotCovered,
        ];
        for state in states {
            let rule = format!(".{}", state.modifier());
            assert!(
                css.contains(&rule),
                "{rule} is emitted by the grid and defined by nothing — the cell would ship with \
                 no treatment and every existing guard would stay green"
            );
        }
        for policy in IpPolicy::ALL {
            let rule = format!(".{}", policy_modifier(policy));
            assert!(css.contains(&rule), "{rule} is emitted and undefined");
        }
    }

    /// 🔴 **`not-covered` IS THE ONE MODIFIER NO LEGEND ENTRY CARRIES, BY ARBITRATION**, and that
    /// hole is asserted rather than left unremarked. A story may not extend the binding vocabulary,
    /// and *not covered* is not one of its four words — so the legend cannot name it, and the
    /// deliberate redundancy that protects the other modifiers does not reach it. What protects it
    /// instead is `the_stylesheet_defines_every_modifier_the_grid_can_emit`, and this test names
    /// that dependency so nobody deletes the other one thinking the legend has it covered.
    #[test]
    fn the_blank_cell_is_the_one_modifier_no_legend_entry_carries() {
        let template = include_str!("../templates/_ipam.html");
        let legend = template
            .split("ipam-legend")
            .nth(1)
            .expect("the legend block");
        assert!(
            !legend.contains("ipam-cell-not-covered"),
            "the legend must NOT name the blank cell — that is the arbitration, and a legend entry \
             for it would be a fifth word beside the binding table's four"
        );
        for modifier in [
            "ipam-cell-defined",
            "ipam-cell-free",
            "ipam-cell-infrastructure",
        ] {
            assert!(
                legend.contains(modifier),
                "{modifier} must be a LITERAL in the legend: the class guard skips the cells \
                 themselves, so these literals are what make the rules visible to it"
            );
        }
    }

    /// 🔴 **AC2: THIS SCREEN READS NO OBSERVATION.** The audit is story 14.3's, and a join written
    /// here would be that story's deliverable arriving early and unmeasured. The guard is a source
    /// scan because the defect is an ADDED read, which no runtime test can provoke — story 5.12's
    /// *you cannot measure the absence of code by running code*.
    #[test]
    fn the_plan_reads_no_observation() {
        let source = include_str!("ipam_page.rs");
        let code = source
            .split("#[cfg(test)]")
            .next()
            .expect("the non-test half");
        for needle in ["observation_record", "identity_link", "declared_attribute"] {
            assert!(
                !code.contains(needle),
                "`{needle}` appears in the plan screen: the audit is story 14.3's, and the \
                 criterion is that this screen draws the PLAN and nothing else"
            );
        }
    }

    /// An empty plan says the gesture is not yet built, and names it.
    #[test]
    fn an_empty_plan_names_the_gesture_as_not_yet_built() {
        let body = empty_plan_body();
        assert!(
            body.contains(&rust_i18n::t!("ipam.empty_plan").to_string()),
            "the sentence saying there is no plan"
        );
        assert!(
            body.contains(&rust_i18n::t!("ipam.empty_plan_gesture").to_string()),
            "the gesture that would fill it, named"
        );
        assert!(
            body.contains(&rust_i18n::t!("gesture.badge").to_string()),
            "and marked NOT YET BUILT — story 14.2b owns the route, and promising a door that does \
             not exist is story 6b.4's finding pointed the other way"
        );
        assert!(!body.contains("ipam-grid"), "an empty plan draws no grid");
    }

    /// Every cell of the rendered grid carries its own accessible name.
    ///
    /// ⚠️ Inherited from story 6b.7's `every_cell_of_the_grid_carries_its_own_aria_label`, which
    /// died with the example dataset. The property did not die with it.
    #[test]
    fn every_cell_of_the_rendered_grid_carries_its_own_aria_label() {
        let subnets = vec![("t-1".to_string(), office(), "Office".to_string())];
        let plan = PlanView::derive(
            office(),
            &[(v4("192.0.2.1"), v4("192.0.2.254"), IpPolicy::Static)],
            &[v4("192.0.2.9")],
        );
        let body = render_plan(&subnets, "t-1", &plan);
        assert_eq!(
            body.matches("<li class=\"ipam-cell").count(),
            256,
            "256 cells"
        );
        assert_eq!(
            body.matches("aria-label=").count(),
            258,
            "one name per cell, plus the grid's own and the selector's"
        );
        assert!(body.contains("192.0.2.9 ·"), "the defined address is named");
        assert!(
            body.contains("192.0.2.0 ·"),
            "and so is the network address"
        );
        // 🔑 The grid is a LIST and not a presentational image — story 6b.7's measured ARIA reason:
        // `role="img"` makes the subtree presentational, so 256 names would be announced as one
        // sentence, and `aria-label` on a bare `<div>` maps to `generic`, where ARIA 1.2 prohibits
        // it outright.
        assert!(body.contains("class=\"ipam-grid\" role=\"list\""));
        assert!(!body.contains("role=\"img\""));
    }

    /// The selector marks exactly one tab, and never claims to be the page.
    #[test]
    fn the_selector_marks_one_tab_and_never_claims_to_be_the_page() {
        let subnets = vec![
            ("t-1".to_string(), office(), String::new()),
            (
                "t-2".to_string(),
                Subnet::new("198.51.100.0".parse().unwrap(), 24).unwrap(),
                "Workshop".to_string(),
            ),
        ];
        let plan = PlanView::derive(office(), &[], &[]);
        let body = render_plan(&subnets, "t-2", &plan);
        assert_eq!(
            body.matches("aria-current=\"true\"").count(),
            1,
            "exactly one tab is in force"
        );
        assert!(
            !body.contains("aria-current=\"page\""),
            "`page` belongs to the shell's navigation; two of them in one document is an ARIA error"
        );
        assert!(body.contains("/ipam?subnet=t-1"), "every subnet is offered");
        assert!(
            body.contains("198.51.100.0/24 · Workshop"),
            "the label follows the CIDR"
        );
        assert!(
            body.contains("192.0.2.0/24<"),
            "a subnet with no label shows its CIDR alone, with no dangling separator"
        );
    }
}
