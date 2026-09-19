//! `/ipam` — the addressing plan, drawn from the store.
//!
//! # What this screen shows, and what it deliberately does not
//!
//! It draws the PLAN — the subnets, ranges and addresses the operator declared — and, since story
//! 14.3b, **the audit of that plan against the network**: which observed addresses are a `gap` or
//! `undeclared`, which carry « Conflit d'adresse », and which address may be offered. 🔑 It reads the
//! network ONLY through [`crate::ipam_audit`], the one module of the plan that the guard
//! `the_plan_reads_the_network_only_through_the_audit` allows to name the sighting reader and the
//! documented read (Guy's decision 6, 2026-09-15).
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

use crate::ipam_audit::{self, FindingKind, Network, Plan, Seen};
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
    /// The subnet's network or broadcast address — carrying the policy of the range that covers
    /// it, when one does.
    ///
    /// 🔴 **THE EDGE IS DECIDED BEFORE THE RANGES, and the first draft decided it after.** That
    /// order is the whole of blind-review finding #1: with the ranges first, an operator who lays
    /// a `static` range across the whole subnet made `.0` a `Free(Static)` cell, which
    /// `offerable` then offered — *the reference mock's own defect, reproduced by the story that
    /// quotes it three times*. Worse, the story's own test asserted 256 offerable cells and so
    /// **required** the defect: a test that pins the ugly thing is a test that demands it.
    ///
    /// 🔑 An edge is infrastructure BY ARITHMETIC, and no declaration makes it assignable. The
    /// policy is carried so the cell still shows what the operator declared over it — the plan is
    /// drawn as written, and only the OFFER is refused.
    Infrastructure(Option<IpPolicy>),
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
            Self::Infrastructure(_) => "ipam-cell-infrastructure",
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
            Self::Infrastructure(_) => "ipam.policy.infrastructure",
            Self::NotCovered => "ipam.state.not_covered",
        }
    }

    /// The policy this cell inherits from its range, when it has one.
    pub(crate) fn policy(self) -> Option<IpPolicy> {
        match self {
            Self::Defined(policy) => policy,
            Self::Free(policy) => Some(policy),
            Self::Infrastructure(policy) => policy,
            Self::NotCovered => None,
        }
    }

    // ⚠️ **`offerable` LEFT this type at story 14.3b.** Whether an address may be offered is now a
    // PLAN-WIDE question (decision 2: a nested subnet's range protects it) that also reads the
    // network and the declared register (decisions 3 and 4), so one cell cannot answer it — see
    // `ipam_audit::Plan::offerable`. The two defects it carried stay recorded there: the edge decided
    // before any range, and `infrastructure` never offered as free.
}

/// The largest subnet this screen will DRAW, in addresses — a `/22`.
///
/// 🔴 **WITHOUT THIS CEILING THE SCREEN WAS A DENIAL OF SERVICE, measured rather than feared.**
/// With a `10.0.0.0/8` in the plan, `/ipam` served **2.08 GB in 44 s with status 200**; a `/16` —
/// an ordinary corporate subnet — already shipped 8.1 MB and 65 536 `<li>` elements. ⚠️ **And the
/// page budget could not help**: `store_within` wraps the READS, while `PlanView::derive` and
/// `render_plan` are synchronous, so `tokio::time::timeout` has nothing to preempt. Forty
/// concurrent requests took `/healthz` from 1 ms to **10.1 s** and the process to 2.6 GB.
///
/// 🔑 Worse than a hostile URL: the default view is the numerically-lowest subnet, so ONE such row
/// made the plain navigation link do it. The ceiling is therefore checked before the cells are
/// MATERIALISED, not after — checking afterwards is paying the cost to learn you should not have.
///
/// ⚠️ Beyond it the plan is shown as the list of RANGES the operator declared, which is what the
/// plan actually holds; a grid of a million cells was never readable anyway (Guy, 2026-09-11).
pub(crate) const MAX_DRAWN_ADDRESSES: u64 = 1024;

/// One subnet's plan, derived from the store's rows and from nothing else.
///
/// It is a pure function of its arguments: no clock, no database, no locale. That is what lets the
/// whole grid be tested without a store, and it is why the handler is three lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PlanView {
    /// One entry per address in the subnet, in numeric order.
    pub(crate) cells: Vec<(Ipv4Addr, CellState)>,
}

/// How protective a policy is, for the ONE question the grid, the offer and the verdict must answer
/// the same way: *may this address be handed out?*
///
/// 🔑 `static` is the least protective because it is the only policy the offer draws from; every
/// other policy takes an address OUT of the offer, so where several ranges cover one address the
/// most protective of them is the one that decides — decision 2, made visible.
fn protection_rank(policy: IpPolicy) -> u8 {
    match policy {
        IpPolicy::Static => 0,
        IpPolicy::DhcpPool => 1,
        IpPolicy::Reserved => 2,
        IpPolicy::Infrastructure => 3,
    }
}

impl PlanView {
    /// Derive the plan for one subnet.
    ///
    /// `ranges` are `(first, last, policy)` and `defined` are the individually-defined addresses.
    ///
    /// 🔴 **BOTH ARGUMENTS ARE PLAN-WIDE SINCE THE CODE REVIEW, and passing the SUBNET's own was a
    /// defect the blind layer found from the diff alone.** The offer and the verdict read every
    /// range of the plan (decision 2), the grid read only the selected subnet's — so on a NESTED
    /// subnet's page an address a parent subnet's `reserved` range protects was drawn as *not
    /// covered*, counted as not covered, and still named a `gap` by the findings list underneath it;
    /// the occupancy line counted it twice over two pages. *One address, one page, three answers.*
    ///
    /// 🔑 The grid is now derived from exactly what the audit reads, so the three agree by
    /// CONSTRUCTION rather than by a comment asking them to.
    ///
    /// ⚠️ **OVERLAPPING RANGES CAN OCCUR, and the first draft of this doc said they could not.**
    /// The adapter refuses them, but §1(C) records that its rule does NOT hold under concurrency —
    /// two overlapping ranges committed under an injected pause — and `a11y/seed.sql` inserts
    /// ranges by RAW SQL, which bypasses the adapter entirely. The overlap resolves to the MOST
    /// PROTECTIVE covering range ([`protection_rank`]), which is what [`Plan::offerable`] already
    /// did; it resolved to the lowest `first_addr` until the review, so a `dhcp-pool` laid over a
    /// `static` range drew a `static` border over a cell the offer refused. 🔑 *A priority order
    /// must not double as a repair* (story 6b.7) — this one is not a repair: it is the audit's own
    /// rule, applied where the operator looks.
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
                    .filter(|(first, last, _)| addr >= *first && addr <= *last)
                    .map(|(_, _, policy)| *policy)
                    .max_by_key(|policy| protection_rank(*policy));
                // 🔴 THE ORDER IS THE DECISION, and it is asserted rather than left to an `if`.
                // An EDGE outranks a range: `.0` and `.255` are infrastructure by arithmetic and a
                // declared range over them changes what they LOOK like, never whether they may be
                // assigned. `Defined` outranks the edge only because an `ip_address` row naming
                // `.0` is the operator's own statement, and a defined cell is never offered either.
                let state = if defined.contains(&addr) {
                    CellState::Defined(policy)
                } else if subnet.is_edge(addr) {
                    CellState::Infrastructure(policy)
                } else if let Some(policy) = policy {
                    CellState::Free(policy)
                } else {
                    CellState::NotCovered
                };
                (addr, state)
            })
            .collect();
        Self { cells }
    }

    /// How many cells carry each state, as a LIST and never a ratio.
    ///
    /// ⚠️ **Its `free` is the STATE count and NOT the occupancy line's since story 14.3b** (decision
    /// 12): the line counts as free only what the offer would propose, and shows no count of seen
    /// addresses. The renderer composes that line from this and from the audit.
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
                CellState::Infrastructure(_) => infrastructure += 1,
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
    /// The subnet to draw, by its id. **Absent selects the first; UNKNOWN selects nothing.**
    ///
    /// 🔴 This line read *"Absent or unknown selects the first"* until the blind review layer
    /// caught it — the fallback the handler ten lines down exists to REFUSE, asserted on the type a
    /// future caller reads first. Silently serving another subnet would tell the operator their
    /// selection took when it did not (story 6b.4's `?sort=` finding). *A false doc is a defect,
    /// and this one described the defect as the design.*
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
        .route(ADDRESS_CHECK_PATH, get(address_check))
        .route(RANGE_CHECK_PATH, get(range_check))
        .route(DELETE_CHECK_PATH, get(delete_check))
        .with_state(IpamState { pool, perimeter })
}

/// Where the address form asks, BEFORE the write, what the network and the registers know about
/// the address being typed (Guy's decision 7, 2026-09-15).
///
/// 🔴 **A GET route that is neither a write route nor a `Screen`**, so neither perimeter guard walks
/// it by default — story 6b.2's defect, measured on a screen route. `main.rs` names it in its own
/// perimeter test and in the page-budget guard.
pub(crate) const ADDRESS_CHECK_PATH: &str = "/ipam/address-check";

/// The query the address check accepts.
#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct AddressCheckQuery {
    /// The address as typed so far. Anything that is not yet an IPv4 address is answered with an
    /// empty warning, before the store is touched.
    pub(crate) addr: Option<String>,
}

/// Warn about an address before it is defined — and never refuse it.
///
/// 🔑 **The form warns and still writes** (Guy's arbitration of 2026-09-10): the most frequent
/// legitimate case is entering into the plan the machine that is already there. So this answers a
/// fragment the form shows beside the field, and the POST is untouched.
///
/// ⚠️ **Budgeted like the screen**, and focus is NOT moved: the region is `aria-live`, so the warning
/// is announced while the operator keeps typing.
async fn address_check(
    State(state): State<IpamState>,
    Query(query): Query<AddressCheckQuery>,
) -> Response {
    let Some(addr) = query
        .addr
        .as_deref()
        .and_then(|typed| typed.trim().parse::<Ipv4Addr>().ok())
    else {
        return Html(String::new()).into_response();
    };
    answer_a_check(
        crate::page::store_within(crate::page::PAGE_STORE_BUDGET, async {
            address_check_data(&state.pool, addr)
                .await
                .map_err(|error| {
                    tracing::error!(%error, "checking an address against the plan and the network");
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
                })
        })
        .await,
    )
}

/// Answer one of the two checks: the fragment it produced, or a keyed *could not check* sentence.
///
/// 🔴 **A REFUSAL USED TO LEAVE THE PREVIOUS ADDRESS'S WARNING STANDING UNDER A NEW VALUE, and a
/// render error put a WHOLE ERROR PAGE inside the live region.** Measured by the review: the handler
/// answered 500, htmx does not swap a 5xx by default, so the region kept the last answer — the
/// operator read *"192.0.2.20 has been seen on the network"* while typing `192.0.2.30`, which is a
/// warning about an address they are not writing. 🔑 **So a check always answers 200 with a body**:
/// what it knows, or that it could not find out. *A live region that keeps a stale sentence is worse
/// than one that says nothing, because the operator cannot tell which address it is about.*
///
/// ⚠️ The status is NOT how this refusal is signalled, and `main.rs`'s budget guard asserts the new
/// contract (200 within the budget, carrying the keyed sentence) rather than the old one.
fn answer_a_check(read: Result<String, Response>) -> Response {
    Html(read.unwrap_or_else(|_| render_check_unavailable())).into_response()
}

/// The *could not check* fragment — one keyed sentence, and the write is untouched.
fn render_check_unavailable() -> String {
    AddressCheck {
        lines: vec![rust_i18n::t!("ipam.check.unavailable").to_string()],
        sightings: String::new(),
        triage_href: None,
        claimed_note: String::new(),
        triage_link: rust_i18n::t!("ipam.finding.triage_link").to_string(),
    }
    .render()
    .unwrap_or_default()
}

/// Read what the address check needs and render it.
///
/// # Errors
///
/// The store's own failure, classified.
async fn address_check_data(
    pool: &MySqlPool,
    addr: Ipv4Addr,
) -> Result<String, opencmdb_core::repo::RepositoryError> {
    let plan = Plan {
        subnets: ipam_repo::list_subnets(pool)
            .await?
            .into_iter()
            .map(|(_, subnet, _)| subnet)
            .collect(),
        ranges: ipam_repo::plan_ranges(pool).await?,
        defined: ipam_repo::plan_addresses(pool).await?.into_iter().collect(),
    };
    let network = ipam_audit::read_the_network(pool).await?;
    Ok(render_address_check(addr, &plan, &network))
}

/// Where the RANGE form asks, before the write, how much of the network its range would cover
/// (Guy's decision of 2026-09-15, at the code review).
///
/// 🔴 **It was deferred by the implementer and not by Guy**, which the acceptance layer caught: §1(e)
/// said decision 7 covered the range form, and it did not. The address form warns about one address;
/// this warns about a stretch of them — *n addresses already seen would fall inside this static
/// range* — and, like the other, it **refuses nothing**.
pub(crate) const RANGE_CHECK_PATH: &str = "/ipam/range-check";

/// The query the range check accepts — the range form's own three fields.
#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct RangeCheckQuery {
    /// The first address, as typed so far.
    pub(crate) first: Option<String>,
    /// The last address, as typed so far.
    pub(crate) last: Option<String>,
    /// The policy token the form's `<select>` carries.
    pub(crate) policy: Option<String>,
}

/// Warn about a range before it is defined — and never refuse it.
async fn range_check(
    State(state): State<IpamState>,
    Query(query): Query<RangeCheckQuery>,
) -> Response {
    let parse = |typed: &Option<String>| {
        typed
            .as_deref()
            .and_then(|text| text.trim().parse::<Ipv4Addr>().ok())
    };
    let (Some(first), Some(last)) = (parse(&query.first), parse(&query.last)) else {
        return Html(String::new()).into_response();
    };
    // 🔑 **`static` ALONE, which is Guy's wording and not a shortcut**: the warning is that the
    // operator is about to promise addresses by hand that something already answers on. Inside a
    // `dhcp-pool` the occupant changes by design (decision 1), and a `reserved` or `infrastructure`
    // range takes the addresses out of the offer anyway, so the sentence would be noise there.
    let is_static = query.policy.as_deref() == Some(IpPolicy::Static.as_str());
    if first > last || !is_static {
        return Html(String::new()).into_response();
    }
    answer_a_check(
        crate::page::store_within(crate::page::PAGE_STORE_BUDGET, async {
            range_check_data(&state.pool, first, last)
                .await
                .map_err(|error| {
                    tracing::error!(%error, "checking a range against the network");
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
                })
        })
        .await,
    )
}

/// Read what the range check needs and render it.
///
/// # Errors
///
/// The store's own failure, classified.
async fn range_check_data(
    pool: &MySqlPool,
    first: Ipv4Addr,
    last: Ipv4Addr,
) -> Result<String, opencmdb_core::repo::RepositoryError> {
    let network = ipam_audit::read_the_network(pool).await?;
    Ok(render_range_check(first, last, &network))
}

/// Render what is worth knowing about a `static` range before it is defined — the empty string when
/// the network has been seen on none of its addresses.
pub(crate) fn render_range_check(first: Ipv4Addr, last: Ipv4Addr, network: &Network) -> String {
    let count = Plan::seen_inside(&network.seen, first, last);
    if count == 0 {
        return String::new();
    }
    AddressCheck {
        lines: vec![
            counted(
                "ipam.check.range_seen_one",
                "ipam.check.range_seen_many",
                count,
            ),
            rust_i18n::t!("ipam.check.still_writes").to_string(),
        ],
        sightings: String::new(),
        triage_href: None,
        claimed_note: String::new(),
        triage_link: rust_i18n::t!("ipam.finding.triage_link").to_string(),
    }
    .render()
    .unwrap_or_else(|_| crate::page::render_error_body())
}

/// Where a removal control asks, BEFORE it fires, what the deletion would change (decision 3,
/// 2026-09-16: a delete warns before it fires, for BOTH deletes).
///
/// 🔴 **A GET route that is neither a write route nor a `Screen`**, so neither perimeter guard walks
/// it by default — story 6b.2's defect. `main.rs` names it in its own perimeter test and in the
/// page-budget guard, beside the other two checks.
pub(crate) const DELETE_CHECK_PATH: &str = "/ipam/delete-check";

/// The query the delete check accepts.
///
/// 🔑 **The bounds travel with the request instead of being looked up by id**, which is what lets
/// ONE route serve both deletions: with `subnet` alone it is the subnet's own removal, and with the
/// range's bounds beside it, that range's. The rail already renders both, so nothing is read back to
/// learn what the operator is looking at.
// ⚠️ No `Default`: the review found it derived and constructed nowhere. Every field is an
// `Option<String>`, which `serde` fills without it — including for an empty query string — so the
// derive was carrying nothing. If a caller ever needs one, it can come back with that caller.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct DeleteCheckQuery {
    /// The subnet in force, always.
    pub(crate) subnet: Option<String>,
    /// The first address of the range being removed, when it is a range.
    pub(crate) first: Option<String>,
    /// Its last address.
    pub(crate) last: Option<String>,
    /// The address being removed, when it is one defined address.
    ///
    /// 🔴 **Without this the three removals could not be told apart**, and the defect was caught
    /// while wiring the control rather than after: `subnet` alone means the SUBNET's own removal,
    /// so an address row asking with `subnet` alone would have been answered *"this subnet still
    /// holds 5 records"* — a true sentence about the wrong gesture, which is worse than none.
    pub(crate) addr: Option<String>,
}

/// Warn about a removal before it fires — and never refuse it.
///
/// ⚠️ **Budgeted like the screen, and focus is NOT moved**: the region is `aria-live`, so the
/// warning is announced where the operator already is. A store failure renders the keyed *could not
/// check* sentence at 200, because htmx does not swap a 5xx and the region would otherwise keep
/// showing the previous answer under a new question (story 14.3b's measured defect).
async fn delete_check(
    State(state): State<IpamState>,
    // 🔴 **A REJECTED QUERY ANSWERS AN EMPTY REGION, not axum's English sentence.** With
    // `Query<…>` extracted directly, `?subnet=a&subnet=b` served `Failed to deserialize query
    // string: duplicate field 'subnet'` with status 400 — and htmx DOES swap a 4xx, so a framework
    // sentence in English landed in the `aria-live` region of a French page. Story 6b.10's
    // arbitration 2(a′) puts the bodies served at these addresses inside the copy perimeter, and
    // this handler's own doc already says a failure must not leave the region under a new question.
    query: Result<Query<DeleteCheckQuery>, axum::extract::rejection::QueryRejection>,
) -> Response {
    let Ok(Query(query)) = query else {
        // 🔑 **THE KEYED SENTENCE, not silence** (my decision, delegated 2026-09-18, recorded as mine
        // so it can be reversed at the right cost). `ipam.check.unavailable` says *the plan could not
        // be checked just now; the write does not depend on this check* — worded about the CHECK, so
        // it is as true of a request this build cannot read as of a store it cannot reach, and
        // reusing it states nothing false. ⚠️ An empty region made *I could not ask* indistinguishable
        // from *nothing here is worth saying*, immediately before an irreversible gesture. Refused:
        // minting a second sentence for the malformed case, which buys a distinction the operator
        // cannot act on differently.
        return Html(render_check_unavailable()).into_response();
    };
    let Some(subnet_id) = query
        .subnet
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
    else {
        return Html(String::new()).into_response();
    };
    // 🔴 **AN UNPARSEABLE QUALIFIER ANSWERS NOTHING — never the enclosing gesture's sentence.**
    // Both reads used `.ok()` and dropped the failure, so `?subnet=…&addr=nonsense` fell through to
    // the arm for a subnet's own removal and answered *"this subnet still holds 2 records"* over a
    // control asking about one address. That is exactly what [`DeleteCheckQuery::addr`]'s own doc
    // says this route was shaped to avoid — *a true sentence about the wrong gesture, which is worse
    // than none* — reached through the one channel the shape does not close. ⚠️ Measured live by the
    // review, including on the store's OWN canonical spelling (`192.000.002.015`, which
    // `Ipv4Addr::from_str` refuses for its leading zeros): today the rail renders dotted, so this
    // was latent rather than live, and any future producer passing the stored form would have
    // degraded every range and address control into the subnet's warning in silence.
    // 🔑 A qualifier this build cannot read earns the *could not be checked* sentence, for the
    // reason given at the extractor above — never the enclosing gesture's sentence, and never
    // silence. ⚠️ The ABSENT case is different and stays silent: a control that sends no bounds and
    // no address is asking about the subnet, which the arms below answer.
    let bounds = match (query.first.as_deref(), query.last.as_deref()) {
        (None, None) => None,
        (Some(first), Some(last)) => {
            let (Ok(first), Ok(last)) = (
                first.trim().parse::<Ipv4Addr>(),
                last.trim().parse::<Ipv4Addr>(),
            ) else {
                return Html(render_check_unavailable()).into_response();
            };
            if last < first {
                return Html(render_check_unavailable()).into_response();
            }
            Some((first, last))
        }
        // One bound without the other names no interval, and a range control always sends both.
        _ => return Html(render_check_unavailable()).into_response(),
    };
    let addr = match query.addr.as_deref() {
        None => None,
        Some(typed) => {
            let Ok(addr) = typed.trim().parse::<Ipv4Addr>() else {
                return Html(render_check_unavailable()).into_response();
            };
            Some(addr)
        }
    };
    answer_a_check(
        crate::page::store_within(crate::page::PAGE_STORE_BUDGET, async {
            delete_check_data(&state.pool, &subnet_id, bounds, addr)
                .await
                .map_err(|error| {
                    tracing::error!(%error, "checking what a removal would change");
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
                })
        })
        .await,
    )
}

/// Read what the delete check needs and render it.
///
/// # Errors
///
/// The store's own failure, classified.
async fn delete_check_data(
    pool: &MySqlPool,
    subnet_id: &str,
    bounds: Option<(Ipv4Addr, Ipv4Addr)>,
    addr: Option<Ipv4Addr>,
) -> Result<String, opencmdb_core::repo::RepositoryError> {
    let subnets = ipam_repo::list_subnets(pool).await?;
    let Some((_, subnet, _)) = subnets.iter().find(|(id, _, _)| id == subnet_id).cloned() else {
        // An id the plan does not carry warns about nothing: the control that sent it is stale, and
        // a sentence invented for it would describe a record that is not there.
        return Ok(String::new());
    };
    let plan = Plan {
        subnets: subnets.iter().map(|(_, subnet, _)| *subnet).collect(),
        ranges: ipam_repo::plan_ranges(pool).await?,
        defined: ipam_repo::plan_addresses(pool).await?.into_iter().collect(),
    };
    let network = ipam_audit::read_the_network(pool).await?;
    // 🔑 **THIS SUBNET'S OWN RECORDS, which this function already read for its `held` count.** The
    // check now asks the same questions the WRITE path asks, and asking them needs the same scope:
    // `delete_range` refuses by scanning `ip_address WHERE subnet_id = ?` and comparing in Rust, so
    // the warning that predicts that refusal compares the same rows the same way. Nothing plan-wide
    // can answer it — the review measured a warning computed over every range in the plan.
    let ranges_here: Vec<(Ipv4Addr, Ipv4Addr, IpPolicy)> =
        ipam_repo::correctable_ranges_in(pool, subnet_id)
            .await?
            .into_iter()
            .map(|(_, first, last, policy, _)| (first, last, policy))
            .collect();
    let defined_here: Vec<Ipv4Addr> = ipam_repo::correctable_addresses_in(pool, subnet_id)
        .await?
        .into_iter()
        .map(|(_, addr, _)| addr)
        .collect();
    Ok(render_delete_check(
        subnet,
        bounds,
        addr,
        &plan,
        &network,
        &ranges_here,
        &defined_here,
    ))
}

/// Render what a removal would change — the empty string when it would change nothing worth saying.
///
/// 🔑 **It refuses nothing — and it no longer promises that the gesture will succeed when the
/// product is about to refuse it** (my decision, delegated 2026-09-18, recorded as mine so it can be
/// reversed at the right cost).
///
/// 🔴 **Until the slice-C review this function appended *"the removal is still possible — not a
/// refusal"* to EVERY non-empty answer**, including the subnet's own, whose sentence one line above
/// reads *"removing it will be refused until they are gone"*. Two sentences contradicting each other
/// in one `aria-live` region, in both languages, measured on a booted binary — and the 409 the press
/// then earns settles which one was false. ⚠️ The range arm was worse: it consulted the network and
/// the offer and never asked the one question that decides whether the gesture happens at all, so the
/// gesture this story's own arbitration REFUSES was the one told it would go through.
///
/// 🔑 **So the check now asks the same rules the write path asks**, in the same scope — a range is
/// refused while it holds an address defined inside it (`delete_range`'s computed refusal), a subnet
/// while anything points at it (a foreign key) — and `still_removes` is appended only when no
/// refusal is coming. The alternatives were refused and are named rather than implied: deleting the
/// closing sentence outright (it is true and useful for every gesture that really will go through),
/// and patching only the contradiction (which leaves the range arm silent about its own refusal).
///
/// ⚠️ **Bounds that name no range of THIS subnet answer nothing.** They used to fabricate a warning
/// about a range that does not exist — measured, `first=0.0.0.0&last=255.255.255.255` reported ten
/// addresses — and a stale control whose range was removed in another tab reaches exactly that.
/// `delete_check_data` already refuses an unknown SUBNET id for this reason, in a comment saying so.
pub(crate) fn render_delete_check(
    subnet: Subnet,
    bounds: Option<(Ipv4Addr, Ipv4Addr)>,
    addr: Option<Ipv4Addr>,
    plan: &Plan,
    network: &Network,
    ranges_here: &[(Ipv4Addr, Ipv4Addr, IpPolicy)],
    defined_here: &[Ipv4Addr],
) -> String {
    let mut lines = Vec::new();
    // Whether the product is about to REFUSE the gesture this warning is about.
    let mut refused = false;
    // 🔑 One address's removal is asked about FIRST, because a row carries its subnet too: without
    // this arm an address control would be answered with the subnet's sentence — true, and about
    // the wrong gesture.
    if let Some(addr) = addr {
        if network.seen.contains_key(&addr) {
            lines.push(rust_i18n::t!("ipam.check.delete_address_seen").to_string());
        }
        if lines.is_empty() {
            return String::new();
        }
        lines.push(rust_i18n::t!("ipam.check.still_removes").to_string());
        return AddressCheck {
            lines,
            sightings: String::new(),
            triage_href: None,
            claimed_note: String::new(),
            triage_link: rust_i18n::t!("ipam.finding.triage_link").to_string(),
        }
        .render()
        .unwrap_or_else(|_| crate::page::render_error_body());
    }
    let held = ranges_here.len() + defined_here.len();
    match bounds {
        Some((first, last)) => {
            // ⚠️ **BOUNDS THAT NAME NO RANGE OF THIS SUBNET WARN ABOUT NOTHING**, and this is the
            // same rule `delete_check_data` applies to an unknown subnet id one function up. It
            // closes two measured holes at once: a fabricated warning for an interval no range
            // carries, and a warning computed for THIS subnet out of ANOTHER subnet's bounds.
            if !ranges_here
                .iter()
                .any(|(range_first, range_last, _)| *range_first == first && *range_last == last)
            {
                return String::new();
            }
            // 🔴 **THE REFUSAL FIRST, because it decides whether the gesture happens at all.** Same
            // rule and same scope as `ipam_repo::delete_range`: an address defined inside the
            // interval, compared in Rust (D10), over THIS subnet's rows.
            let holds = defined_here
                .iter()
                .filter(|addr| **addr >= first && **addr <= last)
                .count();
            if holds > 0 {
                refused = true;
                lines.push(counted(
                    "ipam.check.delete_range_holds_one",
                    "ipam.check.delete_range_holds_many",
                    holds,
                ));
            }
            // What the range explains today and would stop explaining.
            let seen = Plan::seen_inside(&network.seen, first, last);
            if seen > 0 {
                lines.push(counted(
                    "ipam.check.delete_range_seen_one",
                    "ipam.check.delete_range_seen_many",
                    seen,
                ));
            }
            // 🔑 Whether the offer survives: this SUBNET's ranges minus this one, asked the same
            // question the screen asks. ⚠️ It filtered the PLAN-WIDE set by VALUE until the review —
            // and the overlap rule is scoped per subnet (`WHERE subnet_id = ?`), so two subnets may
            // legally carry ranges with identical bounds and both were struck, computing the answer
            // about a plan the deletion would never produce.
            let without: Vec<_> = ranges_here
                .iter()
                .copied()
                .filter(|(range_first, range_last, _)| *range_first != first || *range_last != last)
                .collect();
            let here = Plan {
                subnets: plan.subnets.clone(),
                ranges: ranges_here.to_vec(),
                defined: plan.defined.clone(),
            };
            let remaining = Plan {
                subnets: plan.subnets.clone(),
                ranges: without,
                defined: plan.defined.clone(),
            };
            if here.has_static_range(subnet) && !remaining.has_static_range(subnet) {
                lines.push(rust_i18n::t!("ipam.check.delete_empties_offer").to_string());
            }
        }
        // The subnet's own removal: the database refuses it while anything points at it, so the
        // warning says the refusal is coming rather than letting the operator meet it blind.
        None if held > 0 => {
            refused = true;
            lines.push(counted(
                "ipam.check.delete_subnet_holds_one",
                "ipam.check.delete_subnet_holds_many",
                held,
            ));
        }
        None => {}
    }
    if lines.is_empty() {
        return String::new();
    }
    // 🔴 **ONLY WHEN NO REFUSAL IS COMING.** Appended unconditionally, it told an operator the
    // removal was still possible directly under a sentence saying it would be refused.
    if !refused {
        lines.push(rust_i18n::t!("ipam.check.still_removes").to_string());
    }
    AddressCheck {
        lines,
        sightings: String::new(),
        triage_href: None,
        claimed_note: String::new(),
        triage_link: rust_i18n::t!("ipam.finding.triage_link").to_string(),
    }
    .render()
    .unwrap_or_else(|_| crate::page::render_error_body())
}

/// One sentence per count, and NEVER a parenthetical plural.
///
/// 🔴 Story 6b.10's review found `1 field(s)` on this product's primary screen and fixed it **as a
/// class**, with a property guarding the whole locale file — *a parenthetical plural is a perfectly
/// resolvable key*. Every count this story renders goes through here, so the class stays closed.
fn counted(one: &str, many: &str, count: usize) -> String {
    if count == 1 {
        rust_i18n::t!(one, count = count).to_string()
    } else {
        rust_i18n::t!(many, count = count).to_string()
    }
}

/// The address check's fragment.
#[derive(askama::Template)]
#[template(path = "_ipam_address_check.html")]
pub(crate) struct AddressCheck {
    /// One sentence per thing worth knowing, or none.
    lines: Vec<String>,
    /// Each hardware address seen on it with its absolute last sighting, joined.
    sightings: String,
    /// The triage question for it, when it was seen and no declared record claims it.
    triage_href: Option<String>,
    /// Why there is no triage question, when the address was seen and a declared record claims it —
    /// the empty string when there is nothing to say.
    ///
    /// 🔴 **The findings list said this and the check did not**, which the acceptance layer caught:
    /// the same address, warned about in two places, linked to its question in one and said nothing
    /// about it in the other. A missing link is indistinguishable from a forgotten one.
    claimed_note: String,
    /// The link's words.
    triage_link: String,
}

/// Render what is worth knowing about `addr` before it is defined — the empty string when nothing is.
pub(crate) fn render_address_check(addr: Ipv4Addr, plan: &Plan, network: &Network) -> String {
    let address = addr.to_string();
    let mut lines = Vec::new();
    let mut sightings = String::new();
    let mut triage_href = None;
    let mut claimed_note = String::new();
    let is_documented = network.documented.contains(&addr);
    // Triage's own comparison, verbatim — see `ipam_audit::AddressFinding::claimed`.
    let is_claimed = network.claimed.contains(&address);
    if let Some(seen) = network.seen.get(&addr) {
        lines.push(rust_i18n::t!("ipam.check.seen", address = address.as_str()).to_string());
        sightings = capped_sighting_lines(seen).join("; ");
        if is_claimed {
            claimed_note = rust_i18n::t!("ipam.finding.documented").to_string();
        } else {
            triage_href = Some(format!("/triage?sel=nouveau:{addr}"));
        }
    }
    if is_documented {
        lines.push(rust_i18n::t!("ipam.check.documented", address = address.as_str()).to_string());
    }
    if plan.defines(addr) {
        lines.push(rust_i18n::t!("ipam.check.defined", address = address.as_str()).to_string());
    }
    if plan.covered_by_a_pool(addr) {
        lines.push(rust_i18n::t!("ipam.check.in_pool", address = address.as_str()).to_string());
    }
    if lines.is_empty() {
        return String::new();
    }
    lines.push(rust_i18n::t!("ipam.check.still_writes").to_string());
    AddressCheck {
        lines,
        sightings,
        triage_href,
        claimed_note,
        triage_link: rust_i18n::t!("ipam.finding.triage_link").to_string(),
    }
    .render()
    .unwrap_or_else(|_| crate::page::render_error_body())
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
    // The audit's inputs, all read inside the page budget and none of them grid-sized: the plan
    // WHOLE (decision 2 is plan-wide), and the network through the one module allowed to read it.
    //
    // 🔑 **Read BEFORE the empty-plan branch returns, which is what the code review's acceptance
    // layer found missing**: with no subnet declared, every observed address is outside the plan,
    // and decision 11's list is then the only true thing this screen has to say. It costs one read
    // on a fresh install and it is what makes the empty plan an answer rather than a shrug.
    let plan = Plan {
        subnets: subnets.iter().map(|(_, subnet, _)| *subnet).collect(),
        ranges: ipam_repo::plan_ranges(pool).await?,
        defined: ipam_repo::plan_addresses(pool).await?.into_iter().collect(),
    };
    let network = ipam_audit::read_the_network(pool).await?;
    let outside_only = Audit {
        plan: &plan,
        network: &network,
        subnet: None,
    };
    if subnets.is_empty() {
        return Ok(empty_plan_body(&outside_only));
    }
    // 🔑 An unknown id narrows to NOTHING rather than falling back to the first subnet, on
    // `inventory_body`'s precedent: silently serving another subnet would tell the operator their
    // selection took when it did not — story 6b.4's `?sort=` finding.
    let chosen = match selected {
        Some(wanted) => subnets.iter().find(|(id, _, _)| id == wanted).cloned(),
        None => subnets.first().cloned(),
    };
    let Some((id, subnet, _label)) = chosen else {
        return Ok(unknown_subnet_body(&subnets, &outside_only));
    };
    let audit = Audit {
        plan: &plan,
        network: &network,
        subnet: Some(subnet),
    };
    // 🔴 THE CEILING IS CHECKED BEFORE THE CELLS ARE MATERIALISED. Checking after would mean
    // paying 2 GB to learn the page should not have been drawn — see `MAX_DRAWN_ADDRESSES`.
    if subnet.size() > MAX_DRAWN_ADDRESSES {
        let ranges = ipam_repo::ranges_in(pool, &id).await?;
        // 🔑 **AC6: the rail renders HERE TOO.** The grid is skipped in this branch because
        // materialising it costs gigabytes; the rail's lists are bounded by what the operator
        // DECLARED, so that reason does not carry — and a subnet too large to draw is exactly where
        // an operator most needs the controls that correct it.
        let rail = RailLists::new(
            id.clone(),
            &ipam_repo::correctable_ranges_in(pool, &id).await?,
            &ipam_repo::correctable_addresses_in(pool, &id).await?,
        );
        return Ok(render_too_large(&subnets, &id, &ranges, &audit, rail));
    }
    // 🔴 **THE GRID IS DRAWN FROM THE PLAN-WIDE RANGES AND ADDRESSES** (`PlanView::derive`'s own
    // doc): the subnet's own were what it read until the code review, and a nested subnet's page
    // then drew as *not covered* an address a parent's `reserved` range protects — while the offer
    // and the findings list, both plan-wide, said otherwise on the same screen.
    let defined: Vec<Ipv4Addr> = plan.defined.iter().copied().collect();
    let view = PlanView::derive(subnet, &plan.ranges, &defined);
    // 🔑 The rail's lists are the SUBNET's own records, where the grid is drawn from the plan-wide
    // ranges and addresses — and the difference is deliberate. The grid must show a nested subnet
    // the protection a parent's range gives it; the controls may only offer to correct what this
    // subnet actually owns, because a control that edits another subnet's record from this page is
    // a gesture whose effect the operator cannot see.
    let rail = RailLists::new(
        id.clone(),
        &ipam_repo::correctable_ranges_in(pool, &id).await?,
        &ipam_repo::correctable_addresses_in(pool, &id).await?,
    );
    Ok(render_plan(&subnets, &id, &view, &audit, rail))
}

/// What the audit of the subnet in force needs, borrowed from the handler's reads.
pub(crate) struct Audit<'a> {
    /// The whole plan.
    pub(crate) plan: &'a Plan,
    /// What the network and the declared register say — the sightings, the documented addresses and
    /// the claimed values, read once per request.
    pub(crate) network: &'a Network,
    /// The subnet in force, or `None` on the pages where none is: an empty plan and an identifier no
    /// subnet carries. 🔑 **Those two pages still show what the network shows OUTSIDE the plan**
    /// (decision 11), which the code review found them dropping — and on an EMPTY plan every
    /// observed address is outside it, so that list is the whole of what this screen can honestly
    /// say on a fresh install.
    pub(crate) subnet: Option<Subnet>,
}

/// One address the audit names, ready to render.
#[derive(Debug, Clone)]
pub(crate) struct FindingRow {
    /// The address.
    address: String,
    /// Whether its verdict is `gap`.
    gap: bool,
    /// Whether its verdict is `undeclared`.
    undeclared: bool,
    /// Whether it is « Conflit d'adresse ».
    conflict: bool,
    /// One line per hardware address with its absolute last sighting, and one for a sighting
    /// without one (decision 9).
    sightings: Vec<String>,
    /// The triage question for it, when triage has one — only for an address no declared record
    /// claims.
    triage_href: Option<String>,
}

/// The audit's part of the page.
#[derive(Debug, Clone)]
pub(crate) struct AuditRender {
    /// Whether a subnet is in force — the findings and the pool warnings are about ONE subnet, and
    /// the two subnet-less pages render the outside list alone.
    subnet_in_force: bool,
    /// The subnet's findings, in numeric order. ⚠️ **Not bounded** (Guy, 2026-09-15): it is bounded
    /// by the plan — one entry per observed address of ONE subnet — where the two lists below are
    /// bounded by the network.
    findings: Vec<FindingRow>,
    /// The observed addresses outside every subnet of the plan (decision 11), the twenty most
    /// recently seen of them.
    outside: Vec<FindingRow>,
    /// *and N others*, when the outside list is bounded — the empty string otherwise.
    ///
    /// 🔴 **A BOUND THAT DOES NOT SAY IT IS ONE IS A LIE BY OMISSION.** The edge layer measured 5 007
    /// findings and a 2.26 MB page over 5 000 outside addresses; decision 11's own word is *short*,
    /// and Guy's bound (2026-09-15) is twenty — with this sentence, so the page never claims the
    /// network showed only what it had room for.
    outside_more: String,
    /// The subnet's addresses defined inside a `dhcp-pool` (decision 13).
    pool_warnings: Vec<String>,
}

/// How many hardware addresses a CELL's accessible name carries before it says *and N others*.
///
/// 🔴 **A cell name reached 13 688 BYTES, measured, and every cell of a churning pool carried one.**
/// A name that long is not read by anyone, with sight or without: a screen reader announces it in
/// full on focus. Three is what Guy bounded it to (2026-09-15) — the findings list under the grid
/// keeps every one of them, so nothing is lost, it is moved to where it can be read.
const MAX_CELL_MACS: usize = 3;

/// How many observed addresses outside the plan the list shows before it says *and N others*.
const MAX_OUTSIDE: usize = 20;

/// An instant as the audit shows it: ABSOLUTE and in UTC, never a relative age (decision 9) — a
/// relative age would change with the clock of the render, and the derivation reads no clock.
fn absolute(at: opencmdb_core::observation::Timestamp) -> String {
    at.format("%Y-%m-%d %H:%M UTC").to_string()
}

/// One line per hardware address seen on an address, with its last sighting, and one for a sighting
/// without a hardware address.
fn sighting_lines(seen: &Seen) -> Vec<String> {
    let mut lines: Vec<String> = seen
        .macs
        .iter()
        .map(|(mac, at)| {
            rust_i18n::t!(
                "ipam.finding.seen_mac",
                mac = mac.to_string(),
                at = absolute(*at)
            )
            .to_string()
        })
        .collect();
    if let Some(at) = seen.without_mac {
        lines.push(rust_i18n::t!("ipam.finding.seen_no_mac", at = absolute(at)).to_string());
    }
    lines
}

/// The hardware addresses a CELL's name carries: the [`MAX_CELL_MACS`] most recently seen, then
/// *and N others*.
///
/// 🔑 Ordered by RECENCY here and by hardware address in the findings list, deliberately: a bound has
/// to choose what it keeps, and the most recent sighting is the one an operator acts on. The list
/// under the grid keeps every one of them in a stable order.
fn capped_sighting_lines(seen: &Seen) -> Vec<String> {
    let mut by_recency: Vec<(opencmdb_core::observation::Timestamp, String)> = seen
        .macs
        .iter()
        .map(|(mac, at)| {
            (
                *at,
                rust_i18n::t!(
                    "ipam.finding.seen_mac",
                    mac = mac.to_string(),
                    at = absolute(*at)
                )
                .to_string(),
            )
        })
        .collect();
    if let Some(at) = seen.without_mac {
        by_recency.push((
            at,
            rust_i18n::t!("ipam.finding.seen_no_mac", at = absolute(at)).to_string(),
        ));
    }
    // Newest first, and the LINE breaks the tie so the order is total — two sightings share an
    // instant whenever one sweep saw them, which is the ordinary case.
    by_recency.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let total = by_recency.len();
    let mut lines: Vec<String> = by_recency
        .into_iter()
        .take(MAX_CELL_MACS)
        .map(|(_, line)| line)
        .collect();
    if total > MAX_CELL_MACS {
        lines.push(counted(
            "ipam.finding.more_one",
            "ipam.finding.more_many",
            total - MAX_CELL_MACS,
        ));
    }
    lines
}

/// The row for one observed address.
fn finding_row(
    seen: &Seen,
    kind: Option<FindingKind>,
    conflict: bool,
    claimed: bool,
) -> FindingRow {
    FindingRow {
        address: seen.addr.to_string(),
        gap: kind == Some(FindingKind::Gap),
        undeclared: kind == Some(FindingKind::Undeclared),
        conflict,
        sightings: sighting_lines(seen),
        // 🔑 Triage raises a `nouveau:` question only for an observed address no declared record
        // claims (`page.rs`), so a claimed address has no question to link to — and the row says so
        // rather than linking to nothing. ⚠️ `claimed` and not `documented`: triage compares the
        // declared VALUES verbatim, so a value stored with whitespace documents the address for the
        // offer and still leaves triage asking (`ipam_audit::AddressFinding::claimed`).
        triage_href: (!claimed).then(|| format!("/triage?sel=nouveau:{}", seen.addr)),
    }
}

/// Everything the audit shows for the subnet in force — and, on the two pages where no subnet is,
/// what the network shows outside the plan.
fn audit_render(audit: &Audit) -> AuditRender {
    let findings = match audit.subnet {
        Some(subnet) => audit
            .plan
            .audit(subnet, &audit.network.seen, &audit.network.claimed)
            .iter()
            .map(|f| finding_row(&f.seen, f.kind, f.conflict, f.claimed))
            .collect(),
        None => Vec::new(),
    };
    // Decision 11, BOUNDED (Guy, 2026-09-15): the twenty most recently seen, then a sentence saying
    // how many are not shown. Selected by recency, rendered in numeric order — the selection is the
    // useful half, the order is the readable one.
    let mut outside: Vec<&Seen> = audit.plan.outside(&audit.network.seen);
    let total_outside = outside.len();
    if total_outside > MAX_OUTSIDE {
        outside.sort_by(|a, b| b.last_seen().cmp(&a.last_seen()).then(a.addr.cmp(&b.addr)));
        outside.truncate(MAX_OUTSIDE);
        outside.sort_by_key(|seen| seen.addr);
    }
    AuditRender {
        subnet_in_force: audit.subnet.is_some(),
        findings,
        outside: outside
            .into_iter()
            .map(|seen| {
                finding_row(
                    seen,
                    None,
                    false,
                    audit.network.claimed.contains(&seen.addr.to_string()),
                )
            })
            .collect(),
        outside_more: if total_outside > MAX_OUTSIDE {
            counted(
                "ipam.outside.more_one",
                "ipam.outside.more_many",
                total_outside - MAX_OUTSIDE,
            )
        } else {
            String::new()
        },
        pool_warnings: match audit.subnet {
            Some(subnet) => audit
                .plan
                .defined_inside_a_pool(subnet)
                .into_iter()
                .map(|addr| {
                    rust_i18n::t!("ipam.pool_warning.item", address = addr.to_string()).to_string()
                })
                .collect(),
            None => Vec::new(),
        },
    }
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
    /// The heading over the rail's list of declared ranges.
    rail_ranges_heading: String,
    /// The heading over the rail's list of defined addresses.
    rail_addresses_heading: String,
    /// What the ranges list says when the subnet has none.
    rail_no_ranges: String,
    /// What the addresses list says when the subnet has none.
    rail_no_addresses: String,
    /// The label on every correction control. ⚠️ A word, never an icon alone: an icon-only control
    /// has no accessible name unless one is supplied, and this product has already shipped controls
    /// a keyboard could not reach at all (story 6b.4b).
    rail_edit: String,
    /// The label on every removal control.
    rail_delete: String,
    /// The label on every release control in the findings list — the binding gesture `release` /
    /// « libérer » (story 14.4b), minted by Guy on 2026-09-19 and never invented here.
    finding_release: String,
    next_free_label: String,
    next_free: String,
    next_free_caveat: String,
    empty_plan: String,
    empty_plan_gesture: String,
    unknown_subnet: String,
    too_large: String,
    range_heading: String,
    form_heading: String,
    form_range_summary: String,
    form_address_summary: String,
    form_cidr: String,
    form_label: String,
    form_first: String,
    form_last: String,
    form_policy: String,
    form_addr: String,
    form_submit: String,
    form_needs_subnet: String,
    form_needs_first_subnet: String,
    findings_heading: String,
    no_findings: String,
    word_gap: String,
    word_undeclared: String,
    word_conflict: String,
    conflict_note: String,
    triage_link: String,
    documented_no_question: String,
    outside_heading: String,
    pool_warning_heading: String,
    too_large_no_offer: String,
}

/// The three forms, ready to render.
///
/// 🔑 **Each route is carried by a [`crate::page::Gesture::Live`] and taken back out of it**, which
/// is §3's *"adopt the type or say why not"* answered by adopting. The variant exists so that *a
/// live gesture posting nowhere is unrepresentable* (story 6b.4b), and until now `/ipam` named the
/// type nowhere — so AC8 would have made this the product's second live gesture with none of the
/// guarantee the first one bought.
///
/// ⚠️ **What the type does and does not buy is `page.rs`'s own narrowing, repeated rather than
/// re-derived**: it is a labelling and typing DISCIPLINE, not a compiler-enforced guarantee.
/// 🔴 **And this sentence overclaimed until story 14.2b's review**: it credited the variant with *"a
/// form and its handler cannot drift apart silently"*. What ties them is `WriteRoute::path()`, the
/// one function the router and the template both read; the variant carries that path in and hands
/// it straight back. It is kept as the product's vocabulary for a live gesture, and it adds no
/// guarantee here that the path does not.
#[derive(Debug, Clone)]
pub(crate) struct IpamForms {
    /// Where the subnet form posts.
    subnet_route: &'static str,
    /// Where the range form posts.
    range_route: &'static str,
    /// Where the address form posts.
    address_route: &'static str,
    /// Where the address field asks for a warning before the write (decision 7).
    address_check_route: &'static str,
    /// Where the range form asks the same question about a stretch of addresses.
    range_check_route: &'static str,
    /// Where the rail's subnet removal posts.
    delete_subnet_route: &'static str,
    /// Where a range row's removal posts.
    delete_range_route: &'static str,
    /// Where an address row's removal posts.
    delete_address_route: &'static str,
    /// Where a range row's correction posts.
    edit_range_route: &'static str,
    /// Where an address row's correction posts.
    edit_address_route: &'static str,
    /// Where a removal control asks, before it fires, what the deletion would change (decision 3).
    delete_check_route: &'static str,
    /// Where a finding's release posts (story 14.4b).
    release_route: &'static str,
    /// The subnet in force, which the range and address forms carry as a hidden field. `None`
    /// when no subnet is selected — the two forms are then replaced by a sentence saying so.
    subnet_id: Option<String>,
    /// Whether the plan holds no subnet at all — the one case where *"choose one above"* points at
    /// nothing (story 14.2b's second review), so the sentence in place of the two forms differs.
    plan_is_empty: bool,
    /// The four binding policy words, as `(token, label)` — the token is what the route accepts,
    /// the label is what the operator reads.
    ///
    /// ⚠️ The TOKEN is never translated: it is the binding word `IpPolicy::as_str` produces and
    /// the route compares WITHOUT trimming or folding, so a translated value would be refused.
    policies: Vec<(&'static str, String)>,
}

impl IpamForms {
    /// The three forms for a screen whose subnet in force is `subnet_id`, over a plan that holds no
    /// subnet at all when `plan_is_empty`.
    fn new(subnet_id: Option<String>, plan_is_empty: bool) -> Self {
        use crate::ipam_write::WriteRoute;
        use crate::page::Gesture;
        // Through the type, never around it: a `Gesture::Live` carries its route, and the template
        // reads what the variant carries.
        let route_of = |route: WriteRoute| match (Gesture::Live {
            route: route.path(),
        }) {
            Gesture::Live { route } => route,
            Gesture::Planned { .. } | Gesture::Disabled { .. } => {
                unreachable!("built as Live one line above")
            }
        };
        Self {
            subnet_route: route_of(WriteRoute::Subnet),
            range_route: route_of(WriteRoute::Range),
            address_route: route_of(WriteRoute::Address),
            address_check_route: ADDRESS_CHECK_PATH,
            range_check_route: RANGE_CHECK_PATH,
            delete_subnet_route: route_of(WriteRoute::DeleteSubnet),
            delete_range_route: route_of(WriteRoute::DeleteRange),
            delete_address_route: route_of(WriteRoute::DeleteAddress),
            edit_range_route: route_of(WriteRoute::EditRange),
            edit_address_route: route_of(WriteRoute::EditAddress),
            delete_check_route: DELETE_CHECK_PATH,
            release_route: route_of(WriteRoute::Release),
            subnet_id,
            plan_is_empty,
            policies: IpPolicy::ALL
                .into_iter()
                .map(|policy| {
                    (
                        policy.as_str(),
                        rust_i18n::t!(policy_key(policy)).to_string(),
                    )
                })
                .collect(),
        }
    }
}

/// One cell, ready to render.
#[derive(Debug, Clone)]
pub(crate) struct CellView {
    /// The state's CSS modifier.
    modifier: &'static str,
    /// The policy's CSS modifier, or the empty string when no range covers this address.
    policy_modifier: &'static str,
    /// Whether the network has been seen on this address — decision 8's marker.
    seen: bool,
    /// The accessible name: the address, its state, and its policy when it has one.
    label: String,
}

/// One declared range, for a subnet too large to draw.
#[derive(Debug, Clone)]
pub(crate) struct RangeRow {
    /// The bounds, as the operator writes them.
    bounds: String,
    /// The policy's own word.
    policy: String,
    /// The operator's label, or the empty string.
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

/// One declared range in the rail, with the id its controls name.
///
/// 🔑 **The current values travel with the row**, so the correction form is PRE-FILLED: an edit form
/// an operator must retype from scratch is a delete-and-redefine wearing another word, and it loses
/// the row's identity — which is exactly what `update_range` exists to keep.
#[derive(Debug, Clone)]
pub(crate) struct RailRangeRow {
    /// The record's id, which its controls name.
    id: String,
    /// The bounds as the operator reads them.
    bounds: String,
    /// The policy's own word, translated.
    policy: String,
    /// The policy's BINDING TOKEN — never translated, because the route compares it without
    /// trimming or folding.
    policy_token: &'static str,
    /// The first address, as the correction form pre-fills it.
    first: String,
    /// The last address, likewise.
    last: String,
    /// The operator's label.
    label: String,
}

/// One defined address in the rail, with the id its controls name.
#[derive(Debug, Clone)]
pub(crate) struct RailAddressRow {
    /// The record's id.
    id: String,
    /// The address, as the operator reads and corrects it.
    addr: String,
    /// The operator's label.
    label: String,
}

/// The rail's two lists — what the operator can correct on the subnet in force.
///
/// ⚠️ **Lists in the rail, not controls on the cells** (Guy's decision 3, 2026-09-16). A cell is
/// 14 px and already carries a state, a policy and an accessible name; two controls hung on it would
/// put the gesture where there is no room to say what it does.
#[derive(Debug, Clone)]
pub(crate) struct RailLists {
    /// The subnet in force, whose own removal the rail offers.
    subnet_id: String,
    /// Its declared ranges.
    ranges: Vec<RailRangeRow>,
    /// Its defined addresses.
    addresses: Vec<RailAddressRow>,
}

impl RailLists {
    /// Build the rail's lists from what the adapter read.
    pub(crate) fn new(
        subnet_id: String,
        ranges: &[(String, Ipv4Addr, Ipv4Addr, IpPolicy, String)],
        addresses: &[(String, Ipv4Addr, String)],
    ) -> Self {
        Self {
            subnet_id,
            ranges: ranges
                .iter()
                .map(|(id, first, last, policy, label)| RailRangeRow {
                    id: id.clone(),
                    bounds: format!("{first} – {last}"),
                    policy: rust_i18n::t!(policy_key(*policy)).to_string(),
                    policy_token: policy.as_str(),
                    first: first.to_string(),
                    last: last.to_string(),
                    label: label.clone(),
                })
                .collect(),
            addresses: addresses
                .iter()
                .map(|(id, addr, label)| RailAddressRow {
                    id: id.clone(),
                    addr: addr.to_string(),
                    label: label.clone(),
                })
                .collect(),
        }
    }
}

/// The page.
#[derive(askama::Template)]
#[template(path = "_ipam.html")]
pub(crate) struct IpamBody {
    /// The rail's two lists, when a subnet is in force and its grid is drawn.
    rail: Option<RailLists>,
    /// The strings.
    s: IpamStrings,
    /// The selector.
    tabs: Vec<SubnetTab>,
    /// The grid, or `None` when the plan holds no subnet at all, when the identifier names none,
    /// or when the subnet is too large to draw.
    plan: Option<PlanRender>,
    /// The declared ranges, rendered INSTEAD of a grid when the subnet is too large.
    too_large: Option<Vec<RangeRow>>,
    /// The three write forms.
    forms: IpamForms,
    /// The audit, when a subnet is in force.
    audit: Option<AuditRender>,
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
///
/// 🔑 It still lists what the network shows OUTSIDE the plan, which with no subnet declared is
/// EVERYTHING the scanner has seen — the code review found this page and the next one dropping that
/// list, and on a fresh install it is the only true thing the screen has to say.
fn empty_plan_body(audit: &Audit) -> String {
    let body = IpamBody {
        s: strings(None, None, false),
        tabs: Vec::new(),
        plan: None,
        too_large: None,
        rail: None,
        forms: IpamForms::new(None, true),
        audit: Some(audit_render(audit)),
    };
    body.render()
        .unwrap_or_else(|_| crate::page::render_error_body())
}

/// Build the body for an identifier no subnet carries.
fn unknown_subnet_body(subnets: &[(String, Subnet, String)], audit: &Audit) -> String {
    let body = IpamBody {
        s: strings(None, None, false),
        tabs: subnets
            .iter()
            .map(|(id, subnet, label)| SubnetTab {
                id: id.clone(),
                label: tab_label(subnet, label),
                active: false,
            })
            .collect(),
        plan: None,
        too_large: None,
        rail: None,
        forms: IpamForms::new(None, false),
        audit: Some(audit_render(audit)),
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
///
/// `no_static` says the subnet in force is reached by NO `static` range at all, which is a different
/// state from an exhausted offer and now has its own sentence.
fn strings(
    counts: Option<(usize, usize, usize, usize)>,
    next: Option<Ipv4Addr>,
    no_static: bool,
) -> IpamStrings {
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
    // 🔴 **TWO SENTENCES, because one of them was VACUOUS in one state and FALSE in the other.**
    // *"Every address a static range holds here is defined, documented, already seen, or an edge"*
    // says nothing over a subnet no static range reaches — nothing was ever on offer — and it is
    // wrong when an overlapping `reserved` range is what emptied a static one. The screen now says
    // which of the two it is.
    let next_free = match next {
        Some(addr) => addr.to_string(),
        None if no_static => rust_i18n::t!("ipam.next_free_no_static").to_string(),
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
        rail_ranges_heading: rust_i18n::t!("ipam.rail.ranges_heading").to_string(),
        rail_addresses_heading: rust_i18n::t!("ipam.rail.addresses_heading").to_string(),
        rail_no_ranges: rust_i18n::t!("ipam.rail.no_ranges").to_string(),
        rail_no_addresses: rust_i18n::t!("ipam.rail.no_addresses").to_string(),
        rail_edit: rust_i18n::t!("ipam.rail.edit").to_string(),
        rail_delete: rust_i18n::t!("ipam.rail.delete").to_string(),
        finding_release: rust_i18n::t!("ipam.finding.release").to_string(),
        next_free_caveat: rust_i18n::t!("ipam.next_free_caveat").to_string(),
        empty_plan: rust_i18n::t!("ipam.empty_plan").to_string(),
        empty_plan_gesture: rust_i18n::t!("ipam.empty_plan_gesture").to_string(),
        unknown_subnet: rust_i18n::t!("ipam.unknown_subnet").to_string(),
        too_large: rust_i18n::t!("ipam.too_large", max = MAX_DRAWN_ADDRESSES).to_string(),
        range_heading: rust_i18n::t!("ipam.ranges_heading").to_string(),
        form_heading: rust_i18n::t!("ipam.form.heading").to_string(),
        form_range_summary: rust_i18n::t!("ipam.form.range_summary").to_string(),
        form_address_summary: rust_i18n::t!("ipam.form.address_summary").to_string(),
        form_cidr: rust_i18n::t!("ipam.form.cidr").to_string(),
        form_label: rust_i18n::t!("ipam.form.label").to_string(),
        form_first: rust_i18n::t!("ipam.form.first").to_string(),
        form_last: rust_i18n::t!("ipam.form.last").to_string(),
        form_policy: rust_i18n::t!("ipam.form.policy").to_string(),
        form_addr: rust_i18n::t!("ipam.form.addr").to_string(),
        form_submit: rust_i18n::t!("ipam.form.submit").to_string(),
        form_needs_subnet: rust_i18n::t!("ipam.form.needs_subnet").to_string(),
        form_needs_first_subnet: rust_i18n::t!("ipam.form.needs_first_subnet").to_string(),
        findings_heading: rust_i18n::t!("ipam.findings.heading").to_string(),
        no_findings: rust_i18n::t!("ipam.findings.none").to_string(),
        // 🔑 The audit's two words are the binding vocabulary's own keys, never new ones (constraint
        // 5): `gap` is « écart », the queue's key; `undeclared` is the STATE axis's.
        word_gap: rust_i18n::t!("triage.kind.ecart").to_string(),
        word_undeclared: rust_i18n::t!("state.undeclared").to_string(),
        word_conflict: rust_i18n::t!("ipam.finding.conflict").to_string(),
        conflict_note: rust_i18n::t!("ipam.finding.conflict_note").to_string(),
        triage_link: rust_i18n::t!("ipam.finding.triage_link").to_string(),
        documented_no_question: rust_i18n::t!("ipam.finding.documented").to_string(),
        outside_heading: rust_i18n::t!("ipam.outside.heading").to_string(),
        pool_warning_heading: rust_i18n::t!("ipam.pool_warning.heading").to_string(),
        too_large_no_offer: rust_i18n::t!("ipam.too_large_no_offer").to_string(),
    }
}

/// Render a subnet the screen refuses to draw, as the list of ranges the operator declared.
///
/// 🔑 It shows what the PLAN holds rather than an apology: a `/16` has at most a handful of ranges,
/// and those ranges are the thing the operator wrote. The grid is what does not scale; the plan
/// does.
///
/// ⚠️ **No offer, and the findings list IS shown** (decision 14): a subnet too large to draw is
/// still a subnet the network can contradict, and its findings are bounded by what was SEEN, not by
/// its size.
fn render_too_large(
    subnets: &[(String, Subnet, String)],
    selected: &str,
    ranges: &[(Ipv4Addr, Ipv4Addr, IpPolicy, String)],
    audit: &Audit,
    rail: RailLists,
) -> String {
    let rows = ranges
        .iter()
        .map(|(first, last, policy, label)| RangeRow {
            bounds: format!("{first} – {last}"),
            policy: rust_i18n::t!(policy_key(*policy)).to_string(),
            label: label.clone(),
        })
        .collect();
    let body = IpamBody {
        s: strings(None, None, false),
        tabs: tabs_for(subnets, selected),
        plan: None,
        too_large: Some(rows),
        // 🔑 **AC6: the lists render in BOTH branches**, and the first version of this arm shipped
        // `None` with a comment explaining why — which was a criterion explained away rather than
        // met. The reason does not even carry: the grid is skipped here because materialising it
        // costs gigabytes, while the rail's lists are bounded by what the operator DECLARED and
        // cost the same at any subnet size. A subnet too large to draw is precisely where an
        // operator most needs the controls that correct it.
        rail: Some(rail),
        forms: IpamForms::new(Some(selected.to_string()), false),
        audit: Some(audit_render(audit)),
    };
    body.render()
        .unwrap_or_else(|_| crate::page::render_error_body())
}

/// The selector, built once for every caller that renders it.
fn tabs_for(subnets: &[(String, Subnet, String)], selected: &str) -> Vec<SubnetTab> {
    subnets
        .iter()
        .map(|(id, subnet, label)| SubnetTab {
            id: id.clone(),
            label: tab_label(subnet, label),
            active: id == selected,
        })
        .collect()
}

/// Render one subnet's grid.
pub(crate) fn render_plan(
    subnets: &[(String, Subnet, String)],
    selected: &str,
    plan: &PlanView,
    audit: &Audit,
    rail: RailLists,
) -> String {
    // A grid needs a subnet in force; the two subnet-less pages have their own bodies. `None` is
    // answered with the error body rather than with a panic on a request.
    let Some(subnet) = audit.subnet else {
        return crate::page::render_error_body();
    };
    let tabs = tabs_for(subnets, selected);
    let cells = plan
        .cells
        .iter()
        .map(|(addr, state)| {
            let state_word = rust_i18n::t!(state.label_key()).to_string();
            let base = match state.policy() {
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
            // 🔑 A held cell carries its last sightings in its accessible name (decision 9): the
            // grid's density is not readable without sight, the name is. ⚠️ BOUNDED to the three
            // most recent (`capped_sighting_lines`): the review measured a 13 688-byte name under
            // MAC churn, which is not a name anybody reads.
            let seen = audit.network.seen.get(addr);
            let label = match seen {
                Some(seen) => rust_i18n::t!(
                    "ipam.cell_label_seen",
                    label = base,
                    sightings = capped_sighting_lines(seen).join("; ")
                )
                .to_string(),
                None => base,
            };
            CellView {
                modifier: state.modifier(),
                policy_modifier: state.policy().map(policy_modifier).unwrap_or(""),
                // Decision 8's marker, built after the code review measured the argument for
                // dropping it to be false: the cells the network has been seen on carry one.
                seen: seen.is_some(),
                label,
            }
        })
        .collect();
    // Decision 12: the occupancy line counts as free only what the offer would propose.
    let (defined, _free_state, infrastructure, not_covered) = plan.counts();
    let offerable = plan
        .cells
        .iter()
        .filter(|(addr, _)| {
            audit
                .plan
                .offerable(*addr, &audit.network.seen, &audit.network.documented)
        })
        .count();
    let next = audit
        .plan
        .next_offerable(subnet, &audit.network.seen, &audit.network.documented);
    let body = IpamBody {
        s: strings(
            Some((defined, offerable, infrastructure, not_covered)),
            next,
            !audit.plan.has_static_range(subnet),
        ),
        tabs,
        plan: Some(PlanRender { cells }),
        too_large: None,
        rail: Some(rail),
        forms: IpamForms::new(Some(selected.to_string()), false),
        audit: Some(audit_render(audit)),
    };
    body.render()
        .unwrap_or_else(|_| crate::page::render_error_body())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    /// What the network says, for a test that needs it to say nothing.
    fn quiet() -> Network {
        Network::default()
    }

    /// What the network says, from sightings and the declared values TRIAGE compares (verbatim).
    fn network_of(seen: BTreeMap<Ipv4Addr, Seen>, declared: &[&str]) -> Network {
        Network {
            seen,
            documented: crate::ipam_audit::documented_addresses(
                &declared
                    .iter()
                    .map(|v| (*v).to_string())
                    .collect::<Vec<_>>(),
            ),
            claimed: declared.iter().map(|v| (*v).to_string()).collect(),
        }
    }

    /// A `/24` for the tests, with its two edges and 254 hosts.
    fn office() -> Subnet {
        Subnet::new("192.0.2.0".parse().unwrap(), 24).expect("a /24")
    }

    /// A rail with nothing in it — what a subnet with no declared range and no defined address
    /// renders. ⚠️ Most tests in this module are about the GRID, and a rail they never read would
    /// only add noise to their fixtures.
    /// A rail whose two lists are EMPTY — ⚠️ which is not the `None` case, and the names are close
    /// enough to be worth separating. This is a `Some` carrying nothing, so the panel, both
    /// headings, both *nothing here yet* sentences and the subnet's removal control all render. The
    /// template's `None` arm renders no markup at all, and a comment nearby describes that one.
    fn no_rail() -> RailLists {
        RailLists::new("s1".to_string(), &[], &[])
    }

    /// A rail carrying one of each, which is what makes the FOUR per-row controls RENDER at all.
    ///
    /// ⚠️ It read *"five"* until the review. Four controls hang off a ROW — correct and remove, on a
    /// range row and on an address row — and the fifth, the subnet's own removal, sits outside both
    /// `is_empty` branches in the partial and renders perfectly well over an empty rail. The count
    /// was wrong and so was the justification attached to it.
    ///
    /// 🔑 **The forms-match-routes guard cannot pass over an empty rail, and that is not a flaw in
    /// the guard**: a correction control must NAME its record, so a page holding no records has no
    /// such control to find. A fixture has to hold the thing the gesture acts on — the alternative
    /// was to render five controls with empty ids, which is a control that cannot work shipped to
    /// satisfy a test.
    fn rail_with_rows() -> RailLists {
        RailLists::new(
            "s1".to_string(),
            &[(
                "r-1".to_string(),
                v4("192.0.2.10"),
                v4("192.0.2.20"),
                IpPolicy::Static,
                "Office".to_string(),
            )],
            &[("a-1".to_string(), v4("192.0.2.9"), "Printer".to_string())],
        )
    }

    fn v4(text: &str) -> Ipv4Addr {
        text.parse().expect("a v4 address")
    }

    /// The office subnet's plan, whole, for the offer — since story 14.3b the offer is the AUDIT's
    /// answer (`ipam_audit::Plan::offerable`), not a cell's.
    fn offer_of(ranges: &[(Ipv4Addr, Ipv4Addr, IpPolicy)], defined: &[Ipv4Addr]) -> Plan {
        Plan {
            subnets: vec![office()],
            ranges: ranges.to_vec(),
            defined: defined.iter().copied().collect(),
        }
    }

    /// Every address of the office subnet the plan would offer, with no sighting and nothing
    /// documented.
    fn offered(plan: &Plan) -> Vec<Ipv4Addr> {
        office()
            .addresses()
            .filter(|addr| plan.offerable(*addr, &BTreeMap::new(), &BTreeSet::new()))
            .collect()
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
        let whole = offer_of(
            &[(v4("192.0.2.0"), v4("192.0.2.255"), IpPolicy::Static)],
            &[],
        );
        let offerable = offered(&whole).len();
        assert_eq!(
            offerable, 254,
            "🔴 A RANGE OVER THE WHOLE SUBNET MUST STILL NOT PUT THE EDGES ON OFFER. This \
             assertion read 256 until the blind review layer showed, FROM THE DIFF ALONE, that it \
             pinned the very defect three comments in the same commit denounce — the mock's own \
             *next free address* panel naming the network address. A test that pins the ugly thing \
             is a test that demands it, and this one demanded it for a day"
        );
        assert_eq!(
            offered(&whole).first().copied(),
            Some(v4("192.0.2.1")),
            "and the offer starts at the first host, not at the network address"
        );

        // The ordinary shape: a range over the hosts, the edges covered by nothing.
        let plan = PlanView::derive(
            office(),
            &[(v4("192.0.2.1"), v4("192.0.2.254"), IpPolicy::Static)],
            &[],
        );
        assert_eq!(plan.cells.len(), 256);
        let hosts = offer_of(
            &[(v4("192.0.2.1"), v4("192.0.2.254"), IpPolicy::Static)],
            &[],
        );
        assert_eq!(
            offered(&hosts).len(),
            254,
            "the network and broadcast addresses are never offerable"
        );
        assert_eq!(plan.cells[0].1, CellState::Infrastructure(None), "the .0");
        assert_eq!(
            plan.cells[255].1,
            CellState::Infrastructure(None),
            "the .255"
        );
        assert_eq!(
            offered(&hosts).first().copied(),
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
        let infra = offer_of(
            &[
                (v4("192.0.2.0"), v4("192.0.2.9"), IpPolicy::Infrastructure),
                (v4("192.0.2.10"), v4("192.0.2.254"), IpPolicy::Static),
            ],
            &[],
        );
        assert_eq!(
            offered(&infra).first().copied(),
            Some(v4("192.0.2.10")),
            "the offer must skip the infrastructure range entirely"
        );
        // 🔑 `.0` is an EDGE, so it is `Infrastructure` whatever covers it — and it CARRIES the
        // policy of the range the operator declared over it, because the plan is drawn as written
        // and only the OFFER is refused.
        assert_eq!(
            plan.cells[0].1,
            CellState::Infrastructure(Some(IpPolicy::Infrastructure)),
            "an edge keeps its declared policy and is still an edge"
        );
        assert!(
            !offered(&infra).contains(&v4("192.0.2.0")),
            "and it is not offerable"
        );
        // The other half of the rule, on a cell that is NOT an edge: a declared infrastructure
        // range is free of any individual claim and still never offered — the binding table's
        // *"never offered as free"*, which no ordering can satisfy on its own.
        assert_eq!(
            plan.cells[5].1,
            CellState::Free(IpPolicy::Infrastructure),
            "a mid-range address under an infrastructure policy is free-with-that-policy"
        );
        assert!(
            !offered(&infra).contains(&v4("192.0.2.5")),
            "and it is not offerable either"
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
        assert!(
            !offered(&offer_of(
                &[(v4("192.0.2.1"), v4("192.0.2.100"), IpPolicy::DhcpPool)],
                &[v4("192.0.2.50")],
            ))
            .contains(&v4("192.0.2.50")),
            "a defined address is not on offer"
        );
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
        assert!(
            !offered(&offer_of(
                &[(v4("192.0.2.1"), v4("192.0.2.9"), IpPolicy::Static)],
                &[],
            ))
            .contains(&v4("192.0.2.10"))
        );
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
            CellState::Infrastructure(None),
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
        // 🔴 **COMMENTS ARE STRIPPED FIRST, and without that this guard was satisfied by its own
        // explanation.** `app.css`'s comment narrating the `free`-versus-blank defect contains the
        // string `.ipam-cell-free`, so `css.contains(…)` was true with the RULE deleted: measured
        // by the edge layer with `cargo xtask mutate`, 875 tests, clippy and ten gates green over a
        // stylesheet missing the one rule this story was re-arbitrated to add.
        // 🔑 *A guard that greps a file greps its prose too* — and the better the prose explains
        // the defect, the more reliably it hides it.
        let raw = include_str!("../assets/app.css");
        let mut css = String::with_capacity(raw.len());
        let mut rest = raw;
        while let Some(open) = rest.find("/*") {
            css.push_str(&rest[..open]);
            match rest[open..].find("*/") {
                Some(close) => rest = &rest[open + close + 2..],
                None => {
                    rest = "";
                    break;
                }
            }
        }
        css.push_str(rest);
        let css = css.as_str();
        assert!(
            !css.contains("/*"),
            "comment stripping left a comment opener behind"
        );
        let states = [
            CellState::Defined(None),
            CellState::Free(IpPolicy::Static),
            CellState::Infrastructure(None),
            CellState::NotCovered,
        ];
        // ⚠️ **A SELECTOR ENDS WHERE AN IDENTIFIER ENDS**, and `contains` alone does not know that:
        // renaming `.ipam-cell-free` to `.ipam-cell-free-GONE` left this guard GREEN, because the
        // old name is a PREFIX of the new one. Found by a mutation of mine that was badly chosen —
        // it meant to delete the rule and only renamed it — and the bad mutation is what exposed
        // the weak oracle. *A mutation named for one thing and applied to another sometimes
        // measures a third.*
        let defines = |selector: &str| {
            css.match_indices(selector).any(|(at, _)| {
                let after = css[at + selector.len()..].chars().next();
                !matches!(after, Some(c) if c.is_alphanumeric() || c == '-' || c == '_')
            })
        };
        for state in states {
            let rule = format!(".{}", state.modifier());
            assert!(
                defines(&rule),
                "{rule} is emitted by the grid and defined by nothing — the cell would ship with \
                 no treatment and every existing guard would stay green"
            );
        }
        for policy in IpPolicy::ALL {
            let rule = format!(".{}", policy_modifier(policy));
            assert!(defines(&rule), "{rule} is emitted and undefined");
        }
    }

    /// 🔴 **`not-covered` IS THE ONE MODIFIER NO LEGEND ENTRY CARRIES, BY ARBITRATION**, and that
    /// hole is asserted rather than left unremarked. A story may not extend the binding vocabulary,
    /// and *not covered* is not one of its four words — so the legend cannot name it, and the
    /// deliberate redundancy that protects the other modifiers does not reach it. What protects it
    /// instead is `the_stylesheet_defines_every_modifier_the_grid_can_emit`, and this test names
    /// that dependency so nobody deletes the other one thinking the legend has it covered.
    ///
    /// ⚠️ **The universal was FALSE when written — there were TWO**, and two review layers found it
    /// separately: `ipam-policy-infrastructure` is emitted by the grid and was in no legend entry,
    /// which was not a decision but an omission. It is in the legend now, so the claim is true
    /// again; the test below enumerates EVERY modifier the grid can emit rather than a hand-written
    /// list, so the next omission reds instead of being described away.
    ///
    /// ⚠️ And the block it reads was `split("ipam-legend").nth(1)` — everything after the first
    /// occurrence to END OF FILE, which made the positive assertions satisfiable by a literal
    /// anywhere below. Bounded now.
    #[test]
    fn the_blank_cell_is_the_one_modifier_no_legend_entry_carries() {
        let template = include_str!("../templates/_ipam.html");
        let after = template
            .split("ipam-legend")
            .nth(1)
            .expect("the legend block");
        let legend = after.split("</p>").next().expect("the legend's end");
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

    /// 🔴 **A SUBNET TOO LARGE TO DRAW IS NEVER MATERIALISED, and the ceiling is measured on the
    /// SIZE rather than on the outcome.** Before it existed, `/ipam` served **2.08 GB in 44 s with
    /// status 200** for a `10.0.0.0/8` — on the plain navigation link, the default view being the
    /// numerically-lowest subnet — and the page budget could not help, because it wraps the reads
    /// while the render is synchronous.
    #[test]
    fn a_subnet_beyond_the_ceiling_is_shown_as_its_ranges_and_never_drawn() {
        let big = Subnet::new("10.0.0.0".parse().unwrap(), 8).expect("a /8");
        assert_eq!(big.size(), 16_777_216, "a /8 holds 2^24 addresses");
        assert!(big.size() > MAX_DRAWN_ADDRESSES);

        let subnets = vec![("t-big".to_string(), big, "Everything".to_string())];
        let ranges = vec![(
            v4("10.0.0.1"),
            v4("10.0.0.50"),
            IpPolicy::Static,
            "Servers".to_string(),
        )];
        let whole = Plan {
            subnets: vec![big],
            ranges: vec![(v4("10.0.0.1"), v4("10.0.0.50"), IpPolicy::Static)],
            defined: BTreeSet::new(),
        };
        let quiet = quiet();
        let audit = Audit {
            plan: &whole,
            network: &quiet,
            subnet: Some(big),
        };
        // 🔑 The POPULATED rail: AC6 requires the two lists in THIS branch too, and an empty rail
        // here would satisfy the compiler while leaving the criterion measured by nothing.
        let body = render_too_large(&subnets, "t-big", &ranges, &audit, rail_with_rows());
        assert!(
            !body.contains("ipam-grid"),
            "no grid is drawn for a subnet beyond the ceiling"
        );
        assert!(
            body.contains("10.0.0.1 – 10.0.0.50"),
            "the plan is shown as what it HOLDS — the ranges the operator declared"
        );
        assert!(
            body.contains(&rust_i18n::t!("ipam.ranges_heading").to_string()),
            "and the list says what it is"
        );

        // 🔑 The BOUNDARY, both sides: a /22 is drawn, a /21 is not. Asserting only the /8 would
        // leave the ceiling's VALUE untested — any ceiling at all would pass.
        let at = Subnet::new("10.0.0.0".parse().unwrap(), 22).expect("a /22");
        assert_eq!(
            at.size(),
            MAX_DRAWN_ADDRESSES,
            "a /22 is exactly the ceiling"
        );
        let over = Subnet::new("10.0.0.0".parse().unwrap(), 21).expect("a /21");
        assert!(over.size() > MAX_DRAWN_ADDRESSES, "a /21 is over it");
    }

    /// 🔴 **THE FOUR STATES ARE TOLD APART WITHOUT COLOUR; THE FOUR POLICIES ARE NOT, AND THAT IS
    /// A STATED LIMIT RATHER THAN AN OVERSIGHT** (Guy, 2026-09-11).
    ///
    /// A 14 px cell offers exactly three border styles that RENDER at 1 px — `solid`, `dashed`,
    /// `dotted` — and the fill is already spoken for by the four states. `border-style: double`
    /// collapses to a solid line below 3 px, measured in Chrome 151 by screenshotting one cell per
    /// policy: `ipam-policy-static` and `ipam-policy-infrastructure` came back **IDENTICAL PIXELS**.
    /// So the channel is exhausted, and the honest answer is to say which axis the GRID separates.
    ///
    /// 🔑 **The policy is carried by each cell's accessible name and by the legend**, both of which
    /// name it in words — which is what constraint 6 asks for in its own terms (*a pattern and a
    /// word, never a hue alone*) and what the UX spec means by *"find a free IP without sight"*.
    /// This test pins the half that IS visual, so a future change cannot quietly lose it too.
    #[test]
    fn the_four_states_carry_four_distinct_fills() {
        let raw = include_str!("../assets/app.css");
        for state in [
            CellState::Defined(None),
            CellState::Free(IpPolicy::Static),
            CellState::Infrastructure(None),
            CellState::NotCovered,
        ] {
            let selector = format!(".{} {{", state.modifier());
            let at = raw
                .find(&selector)
                .unwrap_or_else(|| panic!("{selector} must be a rule of its own"));
            let rule = &raw[at..at + raw[at..].find('}').expect("the rule ends")];
            // Each state's fill is declared explicitly — a state that inherits the base cell's
            // transparent background is indistinguishable from the blank, which is the defect
            // arbitration (D) was re-aimed on.
            assert!(
                rule.contains("background"),
                "{selector} declares no background: it would render as the blank cell, and \
                 `free` did exactly that until this story measured it in a browser"
            );
        }
        // And the policy limit is written where someone would look for it.
        // ⚠️ **Whitespace-normalised before matching**, because a comment is wrapped and a needle
        // that spans the wrap matches nothing: story 6b.6's review found a check reporting 7/9 for
        // exactly that reason and recorded that *a check that fails for the wrong reason is worth
        // nothing*. This one did too, on its first run.
        let flat = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            flat.contains("three border styles that render at 1 px"),
            "the stylesheet must SAY that the policy axis is not separable at this size — a limit \
             nobody wrote down is a limit the next story rediscovers"
        );
    }

    /// 🔴 **THE PLAN READS THE NETWORK ONLY THROUGH THE AUDIT** — story 14.3b's AC8, the guard that
    /// read *"the plan reads no observation"* until the audit it forbade by design arrived. Guy's
    /// decision 6 (2026-09-15) NARROWED it rather than retiring it, in three parts:
    ///
    /// 1. no module of the plan — `ipam_audit.rs` included — names `identity_link`,
    ///    `observation_record`, `declared_attribute` or `address_sighting` in its code: the readers
    ///    live outside the plan's modules;
    /// 2. only `ipam_audit.rs` names the sighting reader (`sighting_repo`) or the documented read
    ///    (`crate::repo::load_documented_ipv4s`);
    /// 3. `ipam_write.rs` cannot read the network at all, because `IpamWriteState` holds a PORT and
    ///    never a pool — carried by the type, and proven by a compile-fail mutation, not by this scan.
    ///
    /// ⚠️ **THREE limits, MEASURED and written rather than implied** — the third was added by this
    /// story's code review, which planted it and watched the whole suite stay green:
    /// `ipam_write.rs` calling `crate::ipam_audit::…` is GREEN here (what forbids it is part 3's type);
    /// a module OUTSIDE the perimeter — a new `src/audit.rs` joining everything with an
    /// `identity_link` query — is GREEN (the perimeter is the plan's files, by name or by table);
    /// and **a read reached through a THIRD module's reader is GREEN**, measured with
    /// `crate::scan_pass::counted_current_engine_links(pool)` planted in `ipam_page.rs`: the call
    /// names neither a plan-foreign table nor the `repo` token, because the table is named inside
    /// `scan_pass.rs`. 🔑 *The guard follows ONE module — `repo` — and any module that re-exports a
    /// read is a door beside it.* Following the call graph is what would close it, which is a
    /// different instrument from a source scan; a TRIPWIRE against the read someone writes here,
    /// never a barrier — story 5.12's own framing, stated for the third time because the third
    /// hole was found by planting rather than by reading.
    ///
    /// The guard is a source scan because the defect is an ADDED read, which no runtime test can
    /// provoke — story 5.12's *you cannot measure the absence of code by running code*.
    ///
    /// # The perimeter is DERIVED, and it had to be
    ///
    /// 🔴 **It walked a HARDCODED PAIR until story 14.2b, and the gap-hunt measured the hole before
    /// the module existed**: planting both forms it exists to catch — a raw
    /// `SELECT COUNT(*) FROM observation_record` and a `crate::repo::count_observations` call — in a
    /// new `ipam_write.rs` left **the test green and all ten gates green**, while the story's own
    /// Dev Notes said the opposite. *A file list written by hand covers the files someone
    /// remembered.*
    ///
    /// 🔑 **Membership is a PROPERTY of what a file does, not of what it is called**: a module is in
    /// the plan's perimeter if its name says so OR if its code names one of the plan's three
    /// tables. Measured over `src/` today that is exactly `ipam_page.rs`, `ipam_repo.rs` and
    /// `ipam_write.rs` — `repo.rs` and `main.rs` mention a plan table only inside their test
    /// modules, so the code-half rule leaves them out without an exception list anyone has to audit.
    ///
    /// # Two traps this guard has already paid for
    ///
    /// 🔴 **ANCHORED AT THE START OF A LINE, and the unanchored form read 382 BYTES OF 47 324.**
    /// `ipam_repo.rs`'s module doc quotes `#[cfg(test)]` on line 7 — in the very sentence explaining
    /// that the `file-size` gate stops at the first one. So the guard cut at a MENTION of the
    /// attribute and inspected six lines of header. It was found only because a mutation that
    /// should have reddened came back GREEN and was disbelieved. The witness is now DERIVED too:
    /// D56b gives one trailing test module per file, so the guard asserts exactly ONE line-start
    /// occurrence and cuts there — a hand-written witness per file could not have survived a
    /// derived list. ⚠️ And the first draft asserted that convention for the WHOLE crate and was
    /// refuted on its first run — `example_screens.rs` has four — so it is asserted for the
    /// perimeter alone, and the measurement is registered.
    ///
    /// 🔴 **COMMENTS ARE STRIPPED, and the first derived run is what forced it.** `ipam_write.rs`'s
    /// module doc says the plan *"is a SECOND declared register beside `declared_attribute`"* — a
    /// true sentence about a table it never reads — and the guard would have reddened on the prose
    /// explaining why the read must not exist. *A guard that greps a file greps its prose*, story
    /// 14.2's finding, and the better the prose the more reliably it fires.
    ///
    /// 🔴 **The second review defeated its first stripper and its `repo` needles with ordinary
    /// code**, measured green each time: `const P: &str = "/ipam/*";` — the stripper took the `/*`
    /// inside the string for a comment and dropped the rest of the file — and the imports
    /// `use crate::{repo as store};` and `use super::repo as store;`, which spell neither
    /// `crate::repo` nor `repo::`. It reads through [`crate::source_scan`] now (string-aware, each
    /// trap a test), and it looks for the `repo` TOKEN outside literals rather than for two
    /// spellings of a path.
    ///
    /// ⚠️ **Its standing limit**: it matches table names as literals and follows the `repo` module
    /// only, so a read reached through another adapter — `identity_view`, say — or through a macro
    /// is invisible to it. A TRIPWIRE against the read someone writes here, never a barrier — story
    /// 5.12's own framing.
    #[test]
    fn the_plan_reads_the_network_only_through_the_audit() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut perimeter: Vec<(String, String, String)> = Vec::new();
        // 🔴 RECURSIVE: the review found the flat `read_dir` blind to a `src/ipam/` submodule, which
        // is the ordinary way a module that grows is split.
        let mut pending = vec![src.clone()];
        let mut files = Vec::new();
        while let Some(dir) = pending.pop() {
            for entry in std::fs::read_dir(&dir).expect("a readable source directory") {
                let path = entry.expect("a readable entry").path();
                if path.is_dir() {
                    pending.push(path);
                } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                    files.push(path);
                }
            }
        }
        for path in files {
            let name = path
                .strip_prefix(&src)
                .expect("a file found under `src`")
                .to_str()
                .expect("a UTF-8 path")
                .to_string();
            let source = std::fs::read_to_string(&path).expect("a readable file");

            let cuts = source.matches("\n#[cfg(test)]").count();
            let half = source
                .split("\n#[cfg(test)]")
                .next()
                .expect("the non-test half");
            let code = crate::source_scan::code_only(half);
            let words = crate::source_scan::code_without_literals(half);

            let names_a_plan_table = ["ip_subnet", "ip_range", "ip_address"]
                .iter()
                .any(|table| code.contains(table));
            if name.starts_with("ipam") || names_a_plan_table {
                // 🔴 **ASSERTED FOR THE PERIMETER AND NOT FOR THE CRATE, because the first draft
                // asserted it for the crate and was REFUTED on its first run**: `example_screens.rs`
                // carries **four** line-start `#[cfg(test)]`, so D56b's *one trailing test module
                // per file* is a convention this codebase does not hold everywhere. Registered
                // rather than fixed here — a guard about the addressing plan may not quietly become
                // a guard about the crate's test layout. What it needs is that the cut is
                // unambiguous for the files it reads, and that is what this says.
                assert_eq!(
                    cuts, 1,
                    "{name} has {cuts} line-start `#[cfg(test)]` and this guard cuts at the first: \
                     with none it would read the whole file including its tests, with two it would \
                     cut somewhere arbitrary"
                );
                perimeter.push((name, code, words));
            }
        }
        perimeter.sort();

        let found: Vec<&str> = perimeter.iter().map(|(name, _, _)| name.as_str()).collect();
        // A floor equal to what is there, never under it (story 6b.7). It catches the direction a
        // derived list cannot: a module that LEAVES the perimeter by being renamed or emptied.
        assert_eq!(
            found,
            [
                "ipam_audit.rs",
                "ipam_page.rs",
                "ipam_repo.rs",
                "ipam_write.rs"
            ],
            "the plan's perimeter changed. A module that joined it is covered from here on; one \
             that left it needs saying why"
        );

        // 🔑 **AC5 of story 14.4b — THE RELEASE'S WRITE IS INSIDE THIS PERIMETER, asserted rather than
        // inherited.** A release is a row in a PLAN table (`address_release`), written by the plan's
        // adapter and reached from `ipam_write.rs`; the shapes Guy refused wrote `address_sighting`,
        // which this guard forbids below — and the one of them it would have admitted made the audit
        // EXPORT A WRITE. Without this positive half, moving the write out of the plan's modules
        // would leave the guard green over a release it no longer walks.
        let writer = perimeter
            .iter()
            .find(|(name, _, _)| name.as_str() == "ipam_repo.rs")
            .expect("the plan's adapter is in the perimeter");
        assert!(
            writer.1.contains("INSERT INTO address_release"),
            "the release's write is not in `ipam_repo.rs`: it must stay inside the plan's perimeter, \
             where this guard can see that it touches no sighting (story 14.4b, AC5)"
        );
        let route = perimeter
            .iter()
            .find(|(name, _, _)| name.as_str() == "ipam_write.rs")
            .expect("the plan's routes are in the perimeter");
        assert!(
            route.1.contains("ipam_repo::release_address"),
            "the release route does not reach the plan's adapter from `ipam_write.rs` (AC5)"
        );

        for (name, code, words) in &perimeter {
            for needle in [
                "observation_record",
                "identity_link",
                "declared_attribute",
                "address_sighting",
            ] {
                assert!(
                    !code.contains(needle),
                    "`{needle}` appears in {name}: no module of the plan writes SQL against a table \
                     outside the plan — the audit reads the network through `sighting_repo` and \
                     `repo::load_documented_ipv4s`, named from `ipam_audit.rs` alone, and \
                     `identity_link` from nowhere (story 14.3b, decision 6)"
                );
            }
            // 🔴 **AND THE READ ARRIVES AS A CALL, not as SQL.** The edge layer inserted
            // `crate::repo::count_observations(pool)` into the handler — a function whose own body
            // carries none of the three literals — and measured 875 tests, clippy and TEN GATES
            // GREEN over an `/ipam` that hit `observation_record` on every request. 🔑 *The natural
            // way to add a read is the way the existing reads are written: a call.*
            // ⚠️ `classify` is the one allowed item — this crate's single translation of a backend
            // error (`repo.rs:1607`) — stated as an allowlist of one rather than as an exception
            // nobody can audit.
            //
            // 🔴 **THE `repo` TOKEN, wherever it stands, outside literals.** The first review planted
            // `use crate::repo;` + `repo::count_observations`; the second, `use crate::{repo as
            // store};` and `use super::repo as store;` — each GREEN against a needle that knew two
            // spellings. So every whole-word `repo` in the code is refused unless it is one of the
            // two allowed shapes: the domain crate's ports (`opencmdb_core::repo`) and the two
            // translations of a backend error that read nothing (`crate::repo::classify`,
            // `crate::repo::is_deadlock`). `ipam_repo` is not a whole word and never matches.
            for (at, _) in words.match_indices("repo") {
                let before = &words[..at];
                let after = &words[at + "repo".len()..];
                let is_ident = |c: char| c.is_alphanumeric() || c == '_';
                if before.chars().next_back().is_some_and(is_ident)
                    || after.chars().next().is_some_and(is_ident)
                {
                    continue;
                }
                let names_one = |item: &str| {
                    after
                        .strip_prefix("::")
                        .and_then(|rest| rest.strip_prefix(item))
                        .is_some_and(|rest| !rest.chars().next().is_some_and(is_ident))
                };
                // 🔑 `datetime_literal` joined the allowlist with story 14.4b, on the list's own
                // criterion: it READS NOTHING — it formats an instant the caller already holds, and
                // it is this crate's single formatting site for one (`sighting_repo.rs` says why a
                // second format string is a defect). The release's write needs it for `released_at`.
                let allowed = before.ends_with("opencmdb_core::")
                    || (before.ends_with("crate::")
                        && (names_one("classify")
                            || names_one("is_deadlock")
                            || names_one("datetime_literal")))
                    || (name.as_str() == "ipam_audit.rs"
                        && before.ends_with("crate::")
                        && names_one("load_documented_ipv4s"));
                let line = words[..at].matches('\n').count() + 1;
                assert!(
                    allowed,
                    "{name}:{line} names the `repo` module outside the items this guard allows \
                     (`crate::repo::classify`, `crate::repo::is_deadlock`, \
                     `crate::repo::datetime_literal` anywhere, and \
                     `crate::repo::load_documented_ipv4s` from `ipam_audit.rs` alone) — imported \
                     whole, renamed or reached another way, the network's reads live there, and the \
                     plan reaches them only through the audit (story 14.3b, decision 6)"
                );
            }

            // Part 2: the sighting reader is named from `ipam_audit.rs` alone.
            if name.as_str() != "ipam_audit.rs" {
                for (at, _) in words.match_indices("sighting_repo") {
                    let is_ident = |c: char| c.is_alphanumeric() || c == '_';
                    let before = &words[..at];
                    let after = &words[at + "sighting_repo".len()..];
                    if before.chars().next_back().is_some_and(is_ident)
                        || after.chars().next().is_some_and(is_ident)
                    {
                        continue;
                    }
                    let line = words[..at].matches('\n').count() + 1;
                    panic!(
                        "{name}:{line} names `sighting_repo`: the sighting summary is read by \
                         `ipam_audit.rs` and nowhere else in the plan (story 14.3b, decision 6)"
                    );
                }
            }
        }
    }

    /// When no subnet is in force, the forms say what to do — and the empty plan does not point
    /// "above" at a subnet it does not have.
    ///
    /// 🔴 The second review found *"choose one above, or define it first"* on the EMPTY plan, where
    /// nothing is above, and no test rendering that branch at all.
    #[test]
    fn the_forms_say_the_right_thing_when_no_subnet_is_in_force() {
        let choose = rust_i18n::t!("ipam.form.needs_subnet").to_string();
        let define = rust_i18n::t!("ipam.form.needs_first_subnet").to_string();
        assert_ne!(choose, define, "two cases, two sentences");

        let nothing = Plan::default();
        let quiet = quiet();
        let no_subnet = Audit {
            plan: &nothing,
            network: &quiet,
            subnet: None,
        };
        let empty = empty_plan_body(&no_subnet);
        assert!(
            empty.contains(&define),
            "the empty plan must say to define a subnet first"
        );
        assert!(
            !empty.contains(&choose),
            "the empty plan must not point above at a subnet it does not have"
        );

        let known = [(
            "01900000-0000-7000-8000-0000000000cc".to_string(),
            Subnet::new("192.0.2.0".parse().expect("an address"), 24).expect("a subnet"),
            "Office".to_string(),
        )];
        let unknown = unknown_subnet_body(&known, &no_subnet);
        assert!(
            unknown.contains(&choose),
            "an identifier no subnet carries, over a plan that has one, says to choose it above"
        );
        assert!(!unknown.contains(&define));
    }

    /// **AC8 — an empty plan LINKS to the gesture that fills it, and the door is a door.**
    ///
    /// 🔴 It said NOT YET BUILT until this story, and correctly: `epics.md`'s criterion 5 asks the
    /// empty plan to *name the gesture that fills it AND LINK TO IT*, which story 14.2 could not
    /// deliver because no route existed. It registered the gap rather than faking the link —
    /// *promising a door that does not exist is story 6b.4's finding pointed the other way*.
    ///
    /// 🔑 **The badge's ABSENCE is asserted, not just the link's presence.** A control that both
    /// links and says NOT YET BUILT is worse than either, and nothing else would have caught it:
    /// the two live in different elements.
    ///
    /// ⚠️ **And the anchor's target must be OPEN.** §2.2 accepted an in-page anchor rather than an
    /// href to another address; what makes that honest is that on an empty plan the form is
    /// rendered `open`, so the link never has to expand a `<details>` — *a door that opens onto a
    /// door is not a door*. This asserts the `open` attribute on the very id the anchor names.
    #[test]
    fn an_empty_plan_links_to_the_gesture_that_fills_it() {
        let nothing = Plan::default();
        let quiet = quiet();
        let body = empty_plan_body(&Audit {
            plan: &nothing,
            network: &quiet,
            subnet: None,
        });
        assert!(
            body.contains(&rust_i18n::t!("ipam.empty_plan").to_string()),
            "the sentence saying there is no plan"
        );
        assert!(
            body.contains(&rust_i18n::t!("ipam.empty_plan_gesture").to_string()),
            "the gesture that fills it, named"
        );
        assert!(
            body.contains("href=\"#ipam-form-subnet\""),
            "the criterion is a LINK, and story 14.2 registered it as undeliverable rather than \
             faking one: {body}"
        );
        assert!(
            !body.contains(&rust_i18n::t!("gesture.badge").to_string()),
            "the NOT YET BUILT badge must be GONE: a control that links AND says it is not built \
             is worse than either, and the two live in different elements"
        );
        assert!(
            body.contains("id=\"ipam-form-subnet\" open"),
            "the anchor's target must be rendered OPEN on an empty plan — an in-page anchor onto a \
             collapsed `<details>` is a door that opens onto a door, which is the cost §2.2 \
             accepted and this is what discharges it"
        );
        assert!(!body.contains("ipam-grid"), "an empty plan draws no grid");
    }

    /// Each form posts where its route is mounted, and the route comes through the TYPE.
    ///
    /// 🔑 §3 asked this story to *adopt `Gesture` or say why not*, because `/ipam` named the type
    /// nowhere and AC8 makes it the product's second live gesture. `IpamForms` builds each route
    /// through `Gesture::Live`, which exists so *a live gesture posting nowhere is
    /// unrepresentable*. ⚠️ `page.rs`'s own narrowing applies unchanged and is not re-derived: a
    /// labelling and typing DISCIPLINE, never a compiler-enforced guarantee. What it buys here is
    /// that the form's action and the router's mount are one constant.
    #[test]
    fn every_form_posts_where_its_route_is_mounted() {
        use crate::ipam_write::WriteRoute;
        let whole = offer_of(&[], &[]);
        // 🔑 **ONE ADDRESS SEEN, because the ninth route is a per-FINDING control** (story 14.4b):
        // the release hangs off a `gap` or `undeclared` row, so over a silent network there is no
        // row to hang it from — the populated rail's reason, one list over.
        let seen = network_of(
            crate::ipam_audit::merge_sightings(&[sighted("192.0.2.20", 1, "2026-09-02T10:00:00Z")]),
            &[],
        );
        let body = render_plan(
            &[("s1".to_string(), office(), "Office".to_string())],
            "s1",
            &PlanView::derive(office(), &[], &[]),
            &Audit {
                plan: &whole,
                network: &seen,
                subnet: Some(office()),
            },
            // 🔑 The POPULATED rail: FOUR of the eight routes are per-row controls, and a control
            // that must name its record cannot render over a page holding none. ⚠️ It read *"five"*
            // until the review, counting the subnet's removal — which names no row, sits outside
            // both `is_empty` branches, and renders over an empty rail, so the reason given here was
            // false precisely for the route it added.
            rail_with_rows(),
        );
        for route in WriteRoute::ALL {
            assert!(
                body.contains(&format!("hx-post=\"{}\"", route.path())),
                "no form posts to `{}`, which the router mounts: a screen and a route that drift \
                 apart leave a control that does nothing and a route nothing calls",
                route.path()
            );
        }
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
        let whole = offer_of(
            &[(v4("192.0.2.1"), v4("192.0.2.254"), IpPolicy::Static)],
            &[v4("192.0.2.9")],
        );
        let quiet = quiet();
        let audit = Audit {
            plan: &whole,
            network: &quiet,
            subnet: Some(office()),
        };
        let body = render_plan(&subnets, "t-1", &plan, &audit, no_rail());
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
        let whole = offer_of(&[], &[]);
        let quiet = quiet();
        let audit = Audit {
            plan: &whole,
            network: &quiet,
            subnet: Some(office()),
        };
        let body = render_plan(&subnets, "t-2", &plan, &audit, no_rail());
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

    // ── The audit on screen (story 14.3b) ────────────────────────────────────────────────────

    /// The process's peak resident memory, in kB. ⚠️ A process-wide HIGH-WATER mark, so a before/after
    /// pair is a LOWER bound of what happened in between — story 14.3a's review. Duplicated from
    /// `sighting_repo`'s private test module rather than widened into a shared helper for two
    /// opt-in measurements.
    fn peak_kib() -> u64 {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|status| {
                status
                    .lines()
                    .find(|line| line.starts_with("VmHWM:"))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|kib| kib.parse().ok())
            })
            .unwrap_or(0)
    }

    /// AC9 — `/ipam`'s whole read-and-render path, timed over a long history. Opt-in:
    /// `OPENCMDB_MEASURE_IPAM=<observation rows>` with `--release -- --exact --nocapture`, against a
    /// store that holds nothing else of value (it clears the plan and the observations).
    ///
    /// 🔑 **Why this is the measurement that matters**: story 14.3's validation timed the reader this
    /// story would have used at 3.0–3.3 s over 1 000 000 rows, against NFR2's 1.5 s. `/ipam` reads story
    /// 14.3a's summary instead, so the observation rows are generated here only to prove that their
    /// number no longer reaches the screen. It asserts the worst of twenty renders stays under NFR2.
    ///
    /// ⚠️ **AND IT MEASURES ROWS, NOT PAIRS — the generator holds the distinct MAC count at 46**
    /// (`(seq - 1) MOD 46` on both the address and the hardware address), so *"their number no
    /// longer reaches the screen"* is true of OBSERVATION ROWS and says nothing about a network
    /// whose distinct pairs grow. That is the axis story 14.3a's register row names (a host that
    /// rotates its hardware address adds a pair per rotation), and it is the one the summary does
    /// not bound. Stated because the sentence read as a general claim; the blind and edge layers
    /// found it separately.
    ///
    /// ⚠️ It also measures `plan_data` — the read and the render — and not an HTTP p95: no server is
    /// bound here, so the shell, the response and the socket are outside it.
    #[tokio::test]
    async fn measure_the_plan_screen_over_a_long_history() {
        let Ok(rows) = std::env::var("OPENCMDB_MEASURE_IPAM") else {
            return;
        };
        let rows: u64 = rows.parse().expect("a row count");
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Ok(url) = std::env::var("DATABASE_URL") else {
            return;
        };
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        let clean = [
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM interface",
            "DELETE FROM observation_record",
            "DELETE FROM address_sighting",
            "DELETE FROM sighting_backfill",
            "DELETE FROM ip_address",
            "DELETE FROM ip_range",
            "DELETE FROM ip_subnet",
        ];
        for statement in clean {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }
        const NIL: &str = "00000000-0000-0000-0000-000000000000";
        let started = std::time::Instant::now();
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "INSERT INTO observation_record (id, connector_id, observed_at, l2_domain, vantage, facts, raw) \
             SELECT CONCAT('eeeeeeee-0000-0000-0000-', LPAD(LOWER(HEX(seq)), 12, '0')), '{NIL}', \
             TIMESTAMP('2026-01-01 00:00:00') + INTERVAL ((seq - 1) DIV 46) * 300 SECOND, '{NIL}', '{NIL}', \
             CONCAT('[{{\"IpV4\":{{\"addr\":\"192.0.2.', ((seq - 1) MOD 46) + 1, \
             '\"}}}},{{\"Mac\":{{\"addr\":[2,0,0,0,0,', ((seq - 1) MOD 46) + 1, \
             '],\"locally_administered\":true}}}}]'), NULL FROM seq_1_to_{rows}"
        )))
        .execute(&pool)
        .await
        .expect("generate the observations");
        crate::sighting_repo::backfill_at_boot(&pool)
            .await
            .expect("backfill the summary");
        println!(
            "MEASURE generated and backfilled {rows} observation rows in {:?}",
            started.elapsed()
        );
        sqlx::query(
            "INSERT INTO ip_subnet (id, base, prefix_len, label) VALUES \
             ('55555555-0000-0000-0000-00000000e001', '192.000.002.000', 24, 'Measured')",
        )
        .execute(&pool)
        .await
        .expect("a subnet");
        sqlx::query(
            "INSERT INTO ip_range (id, subnet_id, first_addr, last_addr, policy, label) VALUES \
             ('55555555-0000-0000-0000-00000000e002', '55555555-0000-0000-0000-00000000e001', \
             '192.000.002.001', '192.000.002.254', 'static', 'Hosts')",
        )
        .execute(&pool)
        .await
        .expect("a static range over the hosts");

        let hwm_before = peak_kib();
        let mut worst = std::time::Duration::ZERO;
        let mut bytes = 0;
        for _ in 0..20 {
            let started = std::time::Instant::now();
            let body = plan_data(&pool, None).await.expect("the plan renders");
            worst = worst.max(started.elapsed());
            bytes = body.len();
        }
        println!(
            "MEASURE /ipam over {rows} observation rows: worst of 20 read-and-renders {worst:?}, \
             {bytes} bytes; VmHWM {hwm_before} kB -> {} kB",
            peak_kib()
        );
        assert!(
            worst < std::time::Duration::from_millis(1500),
            "NFR2's 1.5 s p95: the plan screen took {worst:?} over {rows} observation rows"
        );
        for statement in clean {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }
    }

    fn at(text: &str) -> opencmdb_core::observation::Timestamp {
        chrono::DateTime::parse_from_rfc3339(text)
            .expect("an instant")
            .with_timezone(&chrono::Utc)
    }

    /// One sighting with a hardware address ending in `mac_octet`.
    fn sighted(addr: &str, mac_octet: u8, last: &str) -> crate::sighting_repo::Sighting {
        crate::sighting_repo::Sighting {
            addr: v4(addr),
            l2_domain: opencmdb_core::observation::L2DomainId::from_uuid(uuid::Uuid::nil()),
            mac: Some(opencmdb_core::observation::MacAddr([
                2, 0, 0, 0, 0, mac_octet,
            ])),
            first_seen_at: at("2026-09-01T09:00:00Z"),
            last_seen_at: at(last),
        }
    }

    /// Office: `static` .1–.40 with .9 defined, `dhcp-pool` .80–.126 with .90 defined, `reserved`
    /// .130–.150.
    fn audited_office() -> Plan {
        offer_of(
            &[
                (v4("192.0.2.1"), v4("192.0.2.40"), IpPolicy::Static),
                (v4("192.0.2.80"), v4("192.0.2.126"), IpPolicy::DhcpPool),
                (v4("192.0.2.130"), v4("192.0.2.150"), IpPolicy::Reserved),
            ],
            &[v4("192.0.2.9"), v4("192.0.2.90")],
        )
    }

    /// Story 14.4b, decisions 2 and 4 — the release hangs off a `gap` or an `undeclared` finding, and
    /// off nothing else: not a DEFINED address that is only a conflict, and not the outside list.
    ///
    /// 🔑 Anchored on the hidden field's LITERAL and on the route, never on a translated word — a
    /// negative `contains` over a sentence can pass for an escaping reason (story 14.3b's MP6).
    #[test]
    fn the_release_is_offered_on_a_gap_or_an_undeclared_finding_and_nowhere_else() {
        let plan = audited_office();
        let seen = crate::ipam_audit::merge_sightings(&[
            sighted("192.0.2.20", 1, "2026-09-02T10:00:00Z"),
            sighted("192.0.2.140", 2, "2026-09-02T10:00:00Z"),
            sighted("192.0.2.9", 3, "2026-09-02T10:00:00Z"),
            sighted("192.0.2.9", 4, "2026-09-02T10:00:00Z"),
            sighted("10.0.0.7", 5, "2026-09-02T10:00:00Z"),
        ]);
        let network = network_of(seen, &[]);
        let body = render_plan(
            &[("s1".to_string(), office(), "Office".to_string())],
            "s1",
            &PlanView::derive(office(), &[], &[]),
            &Audit {
                plan: &plan,
                network: &network,
                subnet: Some(office()),
            },
            no_rail(),
        );
        let control =
            |addr: &str| format!("<input type=\"hidden\" name=\"addr\" value=\"{addr}\">");
        assert!(
            body.contains(&control("192.0.2.20")),
            "a `gap` finding carries its release"
        );
        assert!(
            body.contains(&control("192.0.2.140")),
            "an `undeclared` finding carries its release"
        );
        assert!(
            body.contains("192.0.2.9</span>"),
            "the premise: the defined address IS listed, as a conflict"
        );
        assert!(
            !body.contains(&control("192.0.2.9")),
            "a defined address cannot be released (decision 4), so its row offers no release"
        );
        assert!(
            body.contains("10.0.0.7</span>"),
            "the premise: the outside address IS listed"
        );
        assert!(
            !body.contains(&control("10.0.0.7")),
            "the outside list offers no release (decision 2 puts it on the subnet's findings)"
        );
        assert_eq!(
            body.matches("hx-post=\"/ipam/release\"").count(),
            2,
            "exactly one release per releasable finding"
        );
        let name = format!(
            "{} — 192.0.2.20</button>",
            rust_i18n::t!("ipam.finding.release")
        );
        assert!(
            body.contains(&name),
            "the control names its address in its accessible name: {name}"
        );
    }

    /// AC4 — the address check warns about what is known and says the write stays possible, links a
    /// triage question only where one exists, and is empty when nothing is worth knowing.
    #[test]
    fn the_address_check_warns_and_still_writes() {
        let plan = audited_office();
        let seen = crate::ipam_audit::merge_sightings(&[
            sighted("192.0.2.20", 1, "2026-09-02T10:00:00Z"),
            sighted("192.0.2.140", 2, "2026-09-02T10:00:00Z"),
        ]);
        let network = network_of(seen, &["192.0.2.140"]);
        let still = rust_i18n::t!("ipam.check.still_writes").to_string();

        let fresh = render_address_check(v4("192.0.2.20"), &plan, &network);
        assert!(
            fresh.contains(&rust_i18n::t!("ipam.check.seen", address = "192.0.2.20").to_string()),
            "a seen address is named as seen: {fresh}"
        );
        assert!(
            fresh.contains("2026-09-02 10:00 UTC"),
            "with its absolute last sighting"
        );
        assert!(
            fresh.contains("href=\"/triage?sel=nouveau:192.0.2.20\""),
            "and its triage question, since no declared record claims it"
        );
        assert!(fresh.contains(&still), "the form warns and STILL writes");

        let known = render_address_check(v4("192.0.2.140"), &plan, &network);
        assert!(
            known.contains(
                &rust_i18n::t!("ipam.check.documented", address = "192.0.2.140").to_string()
            ),
            "a documented address says so"
        );
        assert!(
            !known.contains("nouveau:"),
            "and links to no triage question, because triage asks none"
        );

        let defined = render_address_check(v4("192.0.2.9"), &plan, &quiet());
        assert!(
            defined
                .contains(&rust_i18n::t!("ipam.check.defined", address = "192.0.2.9").to_string())
        );
        let pooled = render_address_check(v4("192.0.2.85"), &plan, &quiet());
        assert!(
            pooled
                .contains(&rust_i18n::t!("ipam.check.in_pool", address = "192.0.2.85").to_string()),
            "decision 13's warning, before the write too"
        );
        assert_eq!(
            render_address_check(v4("192.0.2.30"), &plan, &quiet()),
            "",
            "nothing worth knowing is an empty region, which announces nothing"
        );
    }

    /// The address field asks its warning from the route the router mounts, and announces it where
    /// it is typed.
    #[test]
    fn the_address_field_asks_the_mounted_check_and_describes_itself_with_the_answer() {
        let whole = offer_of(&[], &[]);
        let quiet = quiet();
        let body = render_plan(
            &[("s1".to_string(), office(), "Office".to_string())],
            "s1",
            &PlanView::derive(office(), &[], &[]),
            &Audit {
                plan: &whole,
                network: &quiet,
                subnet: Some(office()),
            },
            no_rail(),
        );
        assert!(body.contains(&format!("hx-get=\"{ADDRESS_CHECK_PATH}\"")));
        assert!(body.contains("aria-describedby=\"ipam-addr-warning\""));
        assert!(body.contains("id=\"ipam-addr-warning\" class=\"ipam-check-region\" role=\"status\" aria-live=\"polite\""));
    }

    /// The part of a rendered body that is the audit, and nothing else.
    fn audit_part(body: &str) -> &str {
        let from = body
            .find("ipam-audit")
            .expect("the audit section is rendered");
        &body[from..]
    }

    /// AC1, AC2, AC5, AC12 — the findings list words `gap`, `undeclared` and « Conflit d'adresse »
    /// with a treatment each, links a triage question only where one exists, keeps the pool silent,
    /// and lists the address outside every subnet.
    #[test]
    fn the_findings_list_words_each_verdict_and_links_only_where_triage_asks() {
        let plan = audited_office();
        let seen = crate::ipam_audit::merge_sightings(&[
            sighted("192.0.2.20", 1, "2026-09-02T10:00:00Z"),
            sighted("192.0.2.20", 2, "2026-09-03T11:30:00Z"),
            sighted("192.0.2.140", 3, "2026-09-02T10:00:00Z"),
            sighted("192.0.2.99", 4, "2026-09-02T10:00:00Z"),
            sighted("10.9.9.9", 5, "2026-09-02T10:00:00Z"),
        ]);
        let network = network_of(seen, &["192.0.2.140"]);
        let audit = Audit {
            plan: &plan,
            network: &network,
            subnet: Some(office()),
        };
        let body = render_plan(
            &[("s1".to_string(), office(), "Office".to_string())],
            "s1",
            &PlanView::derive(office(), &[], &[]),
            &audit,
            no_rail(),
        );
        let part = audit_part(&body);
        let gap_word = rust_i18n::t!("triage.kind.ecart").to_string();
        let undeclared_word = rust_i18n::t!("state.undeclared").to_string();
        let conflict_word = rust_i18n::t!("ipam.finding.conflict").to_string();
        assert!(
            part.contains(&format!(
                "<span class=\"ipam-word ipam-word-gap\">{gap_word}</span>"
            )),
            "a gap carries its word AND its treatment: {part}"
        );
        assert!(part.contains(&format!(
            "<span class=\"ipam-word ipam-word-undeclared\">{undeclared_word}</span>"
        )));
        assert!(part.contains(&format!(
            "<span class=\"ipam-word ipam-word-conflict\">{conflict_word}</span>"
        )));
        assert!(
            part.contains("href=\"/triage?sel=nouveau:192.0.2.20\""),
            "an undocumented address links to its triage question"
        );
        assert!(
            !part.contains("nouveau:192.0.2.140"),
            "a documented address has no triage question to link to"
        );
        assert!(
            part.contains(&rust_i18n::t!("ipam.finding.documented").to_string()),
            "and it says so rather than linking to nothing"
        );
        let findings = part
            .split(&rust_i18n::t!("ipam.outside.heading").to_string())
            .next()
            .expect("the findings before the outside list");
        assert!(
            !findings.contains("192.0.2.99"),
            "nothing inside a dhcp-pool is a finding"
        );
        assert!(
            part.contains(&rust_i18n::t!("ipam.outside.heading").to_string())
                && part.contains("10.9.9.9"),
            "the address outside every subnet is listed"
        );
        assert!(
            part.contains(
                &rust_i18n::t!("ipam.pool_warning.item", address = "192.0.2.90").to_string()
            ),
            "AC7 — an address defined inside a dhcp-pool is warned about"
        );
    }

    /// AC6 — a held cell and a finding carry each hardware address with an ABSOLUTE last sighting,
    /// and the render is a pure function of its data: rendered twice, identical.
    #[test]
    fn a_seen_address_names_each_mac_with_an_absolute_date() {
        let plan = audited_office();
        let seen = crate::ipam_audit::merge_sightings(&[
            sighted("192.0.2.9", 1, "2026-09-02T10:00:00Z"),
            sighted("192.0.2.9", 2, "2026-09-03T11:30:00Z"),
        ]);
        let network = network_of(seen, &[]);
        let audit = Audit {
            plan: &plan,
            network: &network,
            subnet: Some(office()),
        };
        let view = PlanView::derive(office(), &[], &[v4("192.0.2.9")]);
        let subnets = [("s1".to_string(), office(), "Office".to_string())];
        let body = render_plan(&subnets, "s1", &view, &audit, no_rail());
        let cell = body
            .split("aria-label=\"192.0.2.9 ·")
            .nth(1)
            .expect("the held cell")
            .split('"')
            .next()
            .expect("its name");
        assert!(
            cell.contains("02:00:00:00:00:01") && cell.contains("2026-09-02 10:00 UTC"),
            "the first MAC with its absolute date: {cell}"
        );
        assert!(
            cell.contains("02:00:00:00:00:02") && cell.contains("2026-09-03 11:30 UTC"),
            "the second MAC with its own: {cell}"
        );
        assert_eq!(
            render_plan(&subnets, "s1", &view, &audit, no_rail()),
            body,
            "the same data renders the same page — no clock is read"
        );
    }

    /// AC3, AC13 — the occupancy line counts as free only what the offer proposes, and when the offer
    /// is empty the panel says so with the new exclusions named.
    #[test]
    fn the_occupancy_and_the_empty_offer_follow_the_audit() {
        let plan = offer_of(
            &[(v4("192.0.2.1"), v4("192.0.2.3"), IpPolicy::Static)],
            &[v4("192.0.2.1")],
        );
        let seen =
            crate::ipam_audit::merge_sightings(&[sighted("192.0.2.2", 1, "2026-09-02T10:00:00Z")]);
        let network = network_of(seen, &["192.0.2.3"]);
        let audit = Audit {
            plan: &plan,
            network: &network,
            subnet: Some(office()),
        };
        let view = PlanView::derive(
            office(),
            &[(v4("192.0.2.1"), v4("192.0.2.3"), IpPolicy::Static)],
            &[v4("192.0.2.1")],
        );
        let body = render_plan(
            &[("s1".to_string(), office(), "Office".to_string())],
            "s1",
            &view,
            &audit,
            no_rail(),
        );
        assert!(
            body.contains(&rust_i18n::t!("ipam.next_free_none").to_string()),
            "a defined, a seen and a documented address leave nothing to offer, and the panel says so"
        );
        assert!(
            body.contains(
                &rust_i18n::t!(
                    "ipam.occupancy",
                    defined = 1,
                    free = 0,
                    infrastructure = 2,
                    not_covered = 251
                )
                .to_string()
            ),
            "the seen and the documented address are not counted as free to assign: {}",
            body.split("ipam-occupancy").nth(1).unwrap_or("")
        );
    }

    /// Decision 14 — a subnet too large to draw offers nothing, says so, and shows its findings.
    #[test]
    fn a_subnet_too_large_to_draw_offers_nothing_and_shows_its_findings() {
        let big = Subnet::new("10.0.0.0".parse().unwrap(), 8).expect("a /8");
        let plan = Plan {
            subnets: vec![big],
            ranges: vec![(v4("10.0.0.1"), v4("10.0.0.50"), IpPolicy::Reserved)],
            defined: BTreeSet::new(),
        };
        let seen =
            crate::ipam_audit::merge_sightings(&[sighted("10.0.0.7", 1, "2026-09-02T10:00:00Z")]);
        let network = network_of(seen, &[]);
        let audit = Audit {
            plan: &plan,
            network: &network,
            subnet: Some(big),
        };
        let body = render_too_large(
            &[("t-big".to_string(), big, "Everything".to_string())],
            "t-big",
            &[(
                v4("10.0.0.1"),
                v4("10.0.0.50"),
                IpPolicy::Reserved,
                "Held".to_string(),
            )],
            &audit,
            rail_with_rows(),
        );
        assert!(body.contains(&rust_i18n::t!("ipam.too_large_no_offer").to_string()));
        // 🔑 **AC6 — the rail's lists render in THIS branch too**, and this assertion is what makes
        // that a measurement: the first version of the branch shipped `rail: None` with a comment
        // explaining why, which is a criterion explained away. The grid is skipped here because
        // materialising it costs gigabytes; the lists are bounded by what the operator DECLARED and
        // cost the same at any size — and this is the page where the controls matter most, since
        // there is no grid to click on.
        // ⚠️ **BOTH lists and the subnet's own control — the review found only the RANGES list
        // asserted here.** The addresses list could have vanished from this branch with the test
        // green, on the ONE criterion this story has already corrected twice: explained away, then
        // met in Rust while the template still omitted the partial, then found half met. A guard
        // for AC6 that reads half the rail is the third recurrence, not a check.
        for route in [
            "/ipam/range/delete",
            "/ipam/range/edit",
            "/ipam/address/delete",
            "/ipam/address/edit",
            "/ipam/subnet/delete",
        ] {
            assert!(
                body.contains(route),
                "a subnet too large to draw still offers the controls that correct it, and \
                 `{route}` is missing from the rail in this branch: {body}"
            );
        }
        assert!(
            audit_part(&body).contains("10.0.0.7")
                && audit_part(&body).contains("ipam-word-undeclared"),
            "the findings list is shown for a subnet too large to draw"
        );
    }

    /// 🔴 **THE GRID, THE OFFER AND THE VERDICT MUST SAY THE SAME THING ON A NESTED SUBNET'S PAGE**
    /// — Guy's decision at the code review, on a defect the blind layer found from the diff alone.
    /// The offer and the verdict read every range of the plan (decision 2); the grid read the
    /// SELECTED subnet's, so an address a parent subnet's `reserved` range protects was drawn as
    /// *not covered*, counted as not covered, and named a `gap` by the list underneath it.
    #[test]
    fn the_grid_the_offer_and_the_verdict_agree_on_a_nested_subnets_page() {
        let nested = Subnet::new("192.0.2.0".parse().unwrap(), 26).expect("a /26");
        let ranges = vec![
            (v4("192.0.2.1"), v4("192.0.2.254"), IpPolicy::Static),
            // Declared on the OUTER subnet, and covering part of the nested one.
            (v4("192.0.2.10"), v4("192.0.2.19"), IpPolicy::Reserved),
        ];
        let plan = Plan {
            subnets: vec![office(), nested],
            ranges: ranges.clone(),
            defined: BTreeSet::new(),
        };
        let network = network_of(
            crate::ipam_audit::merge_sightings(&[sighted("192.0.2.12", 1, "2026-09-02T10:00:00Z")]),
            &[],
        );
        let audit = Audit {
            plan: &plan,
            network: &network,
            subnet: Some(nested),
        };
        // The cell carries the RESERVED policy, which is the range that decides — not `static`, the
        // one that happens to start lower.
        let view = PlanView::derive(nested, &ranges, &[]);
        assert_eq!(
            view.cells[12].1,
            CellState::Free(IpPolicy::Reserved),
            "the most protective covering range decides what the cell IS, on every page"
        );
        let body = render_plan(
            &[("nested".to_string(), nested, "Nested".to_string())],
            "nested",
            &view,
            &audit,
            no_rail(),
        );
        assert!(
            body.contains("ipam-policy-reserved"),
            "and the cell is drawn with it"
        );
        assert!(
            audit_part(&body).contains(&rust_i18n::t!("state.undeclared").to_string())
                && !audit_part(&body).contains(&rust_i18n::t!("triage.kind.ecart").to_string()),
            "the list says `undeclared`, which is what the grid and the offer say too: {}",
            audit_part(&body)
        );
        assert!(
            !plan.offerable(v4("192.0.2.12"), &network.seen, &network.documented),
            "and the offer refuses it"
        );
    }

    /// 🔴 **THE PAGE GREW WITHOUT BOUND UNDER MAC CHURN**: the review measured 2.26 MB, 5 007
    /// findings and a **13 688-byte cell name** over 47 pool addresses × 200 hardware addresses plus
    /// 5 000 outside addresses, with axe taking 75 s over it. Guy's bound (2026-09-15): three
    /// hardware addresses in a cell's NAME, twenty addresses in the outside list, the subnet's own
    /// findings left whole — *and each bound SAYS how much it is not showing*.
    #[test]
    fn the_cell_name_and_the_outside_list_are_bounded_and_say_how_much_is_hidden() {
        let plan = audited_office();
        let churn: Vec<crate::sighting_repo::Sighting> = (1..=5)
            .map(|n| sighted("192.0.2.20", n, &format!("2026-09-0{n}T10:00:00Z")))
            .collect();
        let mut outside = churn.clone();
        for n in 0..25u8 {
            outside.push(sighted(&format!("10.9.9.{n}"), n, "2026-09-01T10:00:00Z"));
        }
        let network = network_of(crate::ipam_audit::merge_sightings(&outside), &[]);
        let audit = Audit {
            plan: &plan,
            network: &network,
            subnet: Some(office()),
        };
        let view = PlanView::derive(office(), &[], &[]);
        let body = render_plan(
            &[("s1".to_string(), office(), "Office".to_string())],
            "s1",
            &view,
            &audit,
            no_rail(),
        );
        let cell = body
            .split("aria-label=\"192.0.2.20 ·")
            .nth(1)
            .expect("the seen cell")
            .split('"')
            .next()
            .expect("its name");
        assert_eq!(
            cell.matches("02:00:00:00:00:").count(),
            3,
            "a cell's name carries THREE hardware addresses and not five: {cell}"
        );
        assert!(
            cell.contains(&rust_i18n::t!("ipam.finding.more_many", count = 2).to_string()),
            "and it says how many it is not showing: {cell}"
        );
        assert!(
            cell.contains("2026-09-05 10:00 UTC") && !cell.contains("2026-09-01 10:00 UTC"),
            "the three it keeps are the most recently seen: {cell}"
        );
        // The findings list keeps every one of them — the bound moves them, it does not lose them.
        let part = audit_part(&body);
        let listed = part
            .split(&rust_i18n::t!("ipam.outside.heading").to_string())
            .next()
            .expect("the findings");
        assert_eq!(
            listed.matches("02:00:00:00:00:").count(),
            5,
            "the list under the grid carries all five"
        );
        assert_eq!(
            part.split(&rust_i18n::t!("ipam.outside.heading").to_string())
                .nth(1)
                .expect("the outside list")
                .matches("<li class=\"ipam-finding\">")
                .count(),
            20,
            "the outside list shows twenty of the twenty-five"
        );
        assert!(
            part.contains(&rust_i18n::t!("ipam.outside.more_many", count = 5).to_string()),
            "and says that five are not shown — a bound that hides its own existence would let the \
             page claim the network showed only what it had room for"
        );
    }

    /// 🔴 **THE EMPTY-OFFER SENTENCE WAS VACUOUS IN ONE STATE AND FALSE IN THE OTHER** (acceptance
    /// and blind layers): *"every address a static range holds here is defined, documented, already
    /// seen, or an edge"* says nothing over a subnet no static range reaches, and names none of the
    /// real reasons when an overlapping `reserved` range is what emptied one.
    #[test]
    fn the_empty_offer_says_which_of_the_two_states_it_is() {
        let exhausted = rust_i18n::t!("ipam.next_free_none").to_string();
        let never_offered = rust_i18n::t!("ipam.next_free_no_static").to_string();
        assert_ne!(exhausted, never_offered, "two states, two sentences");

        let subnets = [("s1".to_string(), office(), "Office".to_string())];
        let quiet = quiet();
        // (a) No static range reaches this subnet at all.
        let reserved_only = offer_of(
            &[(v4("192.0.2.1"), v4("192.0.2.254"), IpPolicy::Reserved)],
            &[],
        );
        let body = render_plan(
            &subnets,
            "s1",
            &PlanView::derive(
                office(),
                &[(v4("192.0.2.1"), v4("192.0.2.254"), IpPolicy::Reserved)],
                &[],
            ),
            &Audit {
                plan: &reserved_only,
                network: &quiet,
                subnet: Some(office()),
            },
            no_rail(),
        );
        assert!(
            body.contains(&never_offered) && !body.contains(&exhausted),
            "a subnet with no static range was never offering anything, and says so"
        );

        // (b) A static range IS there, and an overlapping reserved one empties it.
        let overlaid = Plan {
            subnets: vec![office()],
            ranges: vec![
                (v4("192.0.2.1"), v4("192.0.2.3"), IpPolicy::Static),
                (v4("192.0.2.1"), v4("192.0.2.3"), IpPolicy::Reserved),
            ],
            defined: BTreeSet::new(),
        };
        let body = render_plan(
            &subnets,
            "s1",
            &PlanView::derive(office(), &overlaid.ranges, &[]),
            &Audit {
                plan: &overlaid,
                network: &quiet,
                subnet: Some(office()),
            },
            no_rail(),
        );
        assert!(
            body.contains(&exhausted) && !body.contains(&never_offered),
            "a static range emptied by a more protective one is the OTHER sentence"
        );
    }

    /// AC4's other half, built at the code review — **the RANGE form warns before the write too**,
    /// and refuses nothing.
    #[test]
    fn the_range_check_counts_what_the_network_already_shows_and_still_writes() {
        let network = network_of(
            crate::ipam_audit::merge_sightings(&[
                sighted("192.0.2.20", 1, "2026-09-02T10:00:00Z"),
                sighted("192.0.2.30", 2, "2026-09-02T10:00:00Z"),
                sighted("192.0.2.90", 3, "2026-09-02T10:00:00Z"),
            ]),
            &[],
        );
        let two = render_range_check(v4("192.0.2.1"), v4("192.0.2.40"), &network);
        assert!(
            two.contains(&rust_i18n::t!("ipam.check.range_seen_many", count = 2).to_string()),
            "two of the three fall inside: {two}"
        );
        assert!(
            two.contains(&rust_i18n::t!("ipam.check.still_writes").to_string()),
            "and it warns without refusing — the write is untouched"
        );
        let one = render_range_check(v4("192.0.2.85"), v4("192.0.2.95"), &network);
        assert!(
            one.contains(&rust_i18n::t!("ipam.check.range_seen_one", count = 1).to_string()),
            "one address is ONE sentence and never `1 address(es)` — story 6b.10 closed that as a \
             class and a new parenthetical plural here would reopen it: {one}"
        );
        assert_eq!(
            render_range_check(v4("192.0.2.200"), v4("192.0.2.210"), &network),
            "",
            "a range over nothing the network shows says nothing at all"
        );
    }

    /// 🔴 **A CHECK THAT CANNOT READ THE STORE SAYS SO, and it used to leave the PREVIOUS address's
    /// warning standing under a new value** — measured by the review's edge layer: the handler
    /// answered 500, htmx does not swap a 5xx, so the region kept a sentence about an address the
    /// operator was no longer writing. A render failure put a whole error page in the live region.
    #[test]
    fn a_check_that_cannot_read_the_store_says_so_rather_than_keeping_the_last_answer() {
        let body = render_check_unavailable();
        assert!(
            body.contains(&rust_i18n::t!("ipam.check.unavailable").to_string()),
            "the keyed sentence: {body}"
        );
        assert!(
            !body.contains("<main") && !body.contains("<nav"),
            "and it is a FRAGMENT: the error body is a whole page, and a whole page inside a live \
             region is announced as one: {body}"
        );
        assert!(
            !body.contains("nouveau:"),
            "it links to no triage question, because it knows nothing about the address"
        );
    }

    /// Drive one GET at the delete check, THROUGH THE ROUTE.
    ///
    /// 🔴 **Every other test of this check calls `render_delete_check`, the pure function — and
    /// both repairs below live in the HANDLER, which that function cannot reach.** A guard written
    /// on the neighbouring model would have been correct about what it tested and blind to what was
    /// broken: this epic's dominant class, and the review found the product defect precisely because
    /// it drove the route instead.
    ///
    /// 🔑 **The pool is unreachable ON PURPOSE, and that is what supplies the control.** Both
    /// repairs return BEFORE the store is touched, so an empty body means *the handler declined to
    /// answer*, while a request that does reach the store comes back carrying the keyed *could not
    /// check* sentence at 200. Two outcomes a test can tell apart with no live MariaDB.
    /// ⚠️ Port 1 refuses at once — deliberately **not** `main.rs`'s lazy-pool idiom, which names
    /// 3306: on this machine that port belongs to an unrelated project's container, and a lazy pool
    /// that never connects is one edit away from one that does.
    async fn ask_the_delete_check(query: &str) -> (axum::http::StatusCode, String) {
        use tower::ServiceExt;
        let pool = MySqlPool::connect_lazy("mysql://root:x@127.0.0.1:1/none").expect("a lazy pool");
        let response = router(pool, None)
            .oneshot(
                axum::http::Request::builder()
                    .uri(format!("{DELETE_CHECK_PATH}?{query}"))
                    .body(axum::body::Body::empty())
                    .expect("a well-formed request"),
            )
            .await
            .expect("the router answers");
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("a readable body");
        (status, String::from_utf8_lossy(&body).into_owned())
    }

    /// **A qualifier this build cannot read never answers the enclosing gesture's sentence** — it
    /// says the check could not be made.
    ///
    /// 🔴 Both reads used `.ok()` and dropped the failure, so an unreadable `addr` or an unreadable
    /// pair of bounds fell through to the arm for the SUBNET's own removal: a control asking about
    /// one address was answered *"this subnet still holds 2 records"*. [`DeleteCheckQuery::addr`]'s
    /// own doc says the route was shaped to prevent exactly that — *a true sentence about the wrong
    /// gesture, which is worse than none* — and the shape did not close this channel.
    ///
    /// ⚠️ **The oracle changed with decision 2, and the old control is GONE rather than kept as
    /// decoration.** It read *empty body = declined before the store*; now a malformed qualifier and
    /// an unreachable store both render `ipam.check.unavailable`, deliberately, so empty-vs-non-empty
    /// no longer separates them and a control built on it would be measuring nothing. What is
    /// asserted instead is the defect itself: **whatever this route says, it is never the subnet's
    /// sentence** — which is what the four queries below are for, and what the shape did not close.
    ///
    /// ⚠️ **`192.000.002.015` is in the list because the STORE holds addresses in that spelling**
    /// (story 14.1's zero-padded canonical form), and `Ipv4Addr::from_str` refuses leading zeros.
    /// The rail renders dotted today, so this was latent rather than live; any future producer
    /// passing the stored form would have degraded every range and address control into the
    /// subnet's warning, in silence.
    #[tokio::test]
    async fn an_unreadable_qualifier_never_answers_the_subnets_sentence() {
        let unavailable = rust_i18n::t!("ipam.check.unavailable").to_string();
        for query in [
            "subnet=s1&addr=nonsense",
            "subnet=s1&addr=192.000.002.015",
            "subnet=s1&first=nonsense&last=nonsense",
            // One bound without the other names no interval, and a range control sends both.
            "subnet=s1&first=192.0.2.10",
        ] {
            let (status, body) = ask_the_delete_check(query).await;
            assert_eq!(status, axum::http::StatusCode::OK, "`{query}`");
            assert!(
                body.contains(&unavailable),
                "`{query}` must say the check could not be made, rather than leaving the region \
                 silent before an irreversible gesture: {body}"
            );
            for gesture_it_is_not_about in [
                rust_i18n::t!("ipam.check.delete_subnet_holds_one", count = 1).to_string(),
                rust_i18n::t!("ipam.check.delete_subnet_holds_many", count = 2).to_string(),
                rust_i18n::t!("ipam.check.still_removes").to_string(),
            ] {
                assert!(
                    !body.contains(&gesture_it_is_not_about),
                    "`{query}` was answered with a sentence about a gesture it does not name: \
                     {body}"
                );
            }
        }
    }

    /// **A query the extractor refuses answers the product's own sentence, never the framework's.**
    ///
    /// 🔴 With `Query<…>` extracted directly, `?subnet=a&subnet=b` served
    /// `Failed to deserialize query string: duplicate field 'subnet'` at 400 — and **htmx swaps a
    /// 4xx**, so English framework text landed in the `aria-live` region of a French page, at an
    /// `/ipam` address. Story 6b.10's arbitration 2(a′) puts the bodies served at these addresses
    /// inside the copy perimeter.
    #[tokio::test]
    async fn a_query_the_extractor_refuses_says_nothing_in_the_frameworks_words() {
        let (status, body) = ask_the_delete_check("subnet=a&subnet=b").await;
        assert_eq!(
            status,
            axum::http::StatusCode::OK,
            "a 4xx here is swapped into the live region by htmx"
        );
        // 🔑 Since decision 2 the region carries the keyed *could not be checked* sentence rather
        // than nothing, so both halves are asserted: the product's own words are THERE, and the
        // framework's are not. The second half is what the defect was — English this product did
        // not write, swapped by htmx into the live region of a French page.
        assert!(
            body.contains(&rust_i18n::t!("ipam.check.unavailable").to_string()),
            "a refused query must say the check could not be made: {body}"
        );
        for framework_english in ["deserialize", "Failed to", "duplicate field"] {
            assert!(
                !body.contains(framework_english),
                "the extractor's own sentence reached the operator: {body}"
            );
        }
    }

    /// Decision 11 on the two pages that have no subnet in force — **an empty plan, and an
    /// identifier no subnet carries.** Both dropped the list entirely until the code review, and on
    /// an empty plan every observed address is outside the plan.
    #[test]
    fn the_subnet_less_pages_still_show_what_the_network_shows_outside_the_plan() {
        let nothing = Plan::default();
        let network = network_of(
            crate::ipam_audit::merge_sightings(&[sighted("10.9.9.9", 1, "2026-09-02T10:00:00Z")]),
            &[],
        );
        let audit = Audit {
            plan: &nothing,
            network: &network,
            subnet: None,
        };
        let empty = empty_plan_body(&audit);
        assert!(
            empty.contains(&rust_i18n::t!("ipam.outside.heading").to_string())
                && empty.contains("10.9.9.9"),
            "with no subnet declared, every observed address is outside the plan — and that list is \
             the only true thing this screen has to say: {empty}"
        );
        // 🔴 **THE NEEDLE IS THE STRUCTURE, NOT THE SENTENCE, AND THE MUTATION IS WHAT SAID SO.**
        // This read `!empty.contains(&t!("ipam.findings.none"))` and was **VACUOUS**: that key's value
        // is *"Nothing the network has shown contradicts this subnet's plan."*, Askama escapes the
        // apostrophe to `&#x27;`, and a resolved string never matches an escaped render — so the
        // negative assertion passed over a page that carried the sentence. Measured: under
        // `subnet_in_force: true` the suite stayed GREEN (MP6 contradicted its `red` prediction),
        // and printing the body showed the heading rendered twice and the sentence present.
        // 🔑 *A `contains` oracle over a translated sentence can only fail in the direction that
        // escaping does not touch, and the negative direction is exactly the one it cannot measure.*
        // The id is emitted by the template as a literal and carries no escapable character.
        assert!(
            !empty.contains("id=\"ipam-findings-heading\""),
            "the findings section belongs to ONE subnet and no subnet is in force: {empty}"
        );

        let known = [("s1".to_string(), office(), "Office".to_string())];
        let unknown = unknown_subnet_body(&known, &audit);
        assert!(
            unknown.contains("10.9.9.9"),
            "and the same holds for an identifier no subnet carries"
        );
    }

    /// 🔴 **DECISION 8'S MARKER EXISTS, and the story dropped it on arithmetic that was wrong.** Two
    /// review layers re-did it: BLACK reaches 3.25:1 against the defined fill (`--color-accent-700`)
    /// and 18.77:1 against the page ground — both figures COMPUTED by the axe gate in a browser, not
    /// by hand — so the condition Guy set is MET. ⚠️ `--color-text`
    /// (#1d1f20) does not: 2.56:1. What no Rust test can see is the RENDERED contrast, which is why
    /// `a11y/axe-gate.mjs` computes it in the browser on all four fills — this asserts the marker is
    /// emitted and defined, never that it is visible.
    #[test]
    fn a_seen_cell_carries_the_marker_and_the_stylesheet_paints_it() {
        let plan = audited_office();
        let network = network_of(
            crate::ipam_audit::merge_sightings(&[sighted("192.0.2.20", 1, "2026-09-02T10:00:00Z")]),
            &[],
        );
        let body = render_plan(
            &[("s1".to_string(), office(), "Office".to_string())],
            "s1",
            &PlanView::derive(office(), &[], &[]),
            &Audit {
                plan: &plan,
                network: &network,
                subnet: Some(office()),
            },
            no_rail(),
        );
        assert_eq!(
            body.matches("ipam-cell-seen").count(),
            1,
            "exactly the cell the network has been seen on carries the marker"
        );
        // ⚠️ The class is built in Rust, so `every_class_a_template_names_is_defined_in_the_stylesheet`
        // cannot see it (it skips any `class="…"` carrying an expression) and the legend does not
        // name it. This is what covers it, on `the_stylesheet_defines_every_modifier_the_grid_can_emit`'s
        // own precedent.
        let css = include_str!("../assets/app.css");
        assert!(
            css.contains(".ipam-cell-seen::after"),
            "the marker is a pseudo-element and the sheet must define it"
        );
        assert!(
            css.contains("--color-seen-marker: #000000"),
            "and it is BLACK on purpose: #1d1f20 reaches 2.56:1 on the defined fill, under the 3:1 \
             decision 8 conditions the marker on"
        );
    }

    /// **T3b — a removal WARNS before it fires and refuses nothing**, and each sentence is paired
    /// with the case where it must NOT appear.
    ///
    /// 🔑 The controls are the point. *This is the only static range* is meaningless without a plan
    /// where a second static range survives the deletion; *the network still shows this address*
    /// is meaningless without an address the network has not shown. A guard that only ever sees the
    /// interesting case cannot tell a rule from a constant.
    #[test]
    fn a_removal_warns_about_what_it_would_change_and_refuses_nothing() {
        let seen =
            crate::ipam_audit::merge_sightings(&[sighted("192.0.2.15", 1, "2026-09-02T10:00:00Z")]);
        let network = network_of(seen, &[]);
        let still_removes = rust_i18n::t!("ipam.check.still_removes").to_string();
        let empties = rust_i18n::t!("ipam.check.delete_empties_offer").to_string();

        // A range whose removal both strands an observed address and empties the offer.
        // 🔑 The subnet's OWN ranges and addresses are passed since decision 1(a): the check asks the
        // same rules the write path asks, and `delete_range` asks them over `subnet_id = ?`.
        let only_static_ranges = [(v4("192.0.2.10"), v4("192.0.2.20"), IpPolicy::Static)];
        let only_static = offer_of(&only_static_ranges, &[]);
        let warning = render_delete_check(
            office(),
            Some((v4("192.0.2.10"), v4("192.0.2.20"))),
            None,
            &only_static,
            &network,
            &only_static_ranges,
            &[],
        );
        assert!(
            warning.contains(
                &rust_i18n::t!("ipam.check.delete_range_seen_one", count = 1).to_string()
            ),
            "the address the range explains today must be named: {warning}"
        );
        assert!(
            warning.contains(&empties),
            "and the emptied offer too: {warning}"
        );
        assert!(
            warning.contains(&still_removes),
            "and it must say the removal is still possible — a warning read as a refusal is a \
             gesture the operator stops using: {warning}"
        );

        // THE CONTROL: a second static range survives, so the offer is not emptied.
        let two_static = offer_of(
            &[
                (v4("192.0.2.10"), v4("192.0.2.20"), IpPolicy::Static),
                (v4("192.0.2.30"), v4("192.0.2.40"), IpPolicy::Static),
            ],
            &[],
        );
        let two_static_ranges = [
            (v4("192.0.2.10"), v4("192.0.2.20"), IpPolicy::Static),
            (v4("192.0.2.30"), v4("192.0.2.40"), IpPolicy::Static),
        ];
        let survives = render_delete_check(
            office(),
            Some((v4("192.0.2.10"), v4("192.0.2.20"))),
            None,
            &two_static,
            &network,
            &two_static_ranges,
            &[],
        );
        assert!(
            !survives.contains(&empties),
            "another static range remains, so nothing about the offer is true here: {survives}"
        );

        // The subnet's own removal, which the database will refuse while it holds anything.
        let holds = render_delete_check(
            office(),
            None,
            None,
            &two_static,
            &network,
            &two_static_ranges,
            &[v4("192.0.2.9")],
        );
        assert!(
            holds.contains(
                &rust_i18n::t!("ipam.check.delete_subnet_holds_many", count = 3).to_string()
            ),
            "the operator is told the refusal is coming rather than meeting it blind: {holds}"
        );
        // 🔴 **AND IT MUST NOT THEN DENY THE REFUSAL IT JUST ANNOUNCED.** Until decision 1(a) the
        // closing sentence was appended unconditionally, so this region carried *"removing it will
        // be refused"* and *"the removal is still possible — not a refusal"* one after the other, in
        // both languages, and the 409 the press earns settles which was false.
        assert!(
            !holds.contains(&still_removes),
            "the subnet's warning announced a refusal and then denied there was one: {holds}"
        );
        assert!(
            render_delete_check(office(), None, None, &two_static, &network, &[], &[]).is_empty(),
            "and an empty subnet's removal changes nothing worth a sentence"
        );

        // One address: warned when the network still shows it, silent when it does not.
        let stranded = render_delete_check(
            office(),
            None,
            Some(v4("192.0.2.15")),
            &two_static,
            &network,
            &two_static_ranges,
            &[v4("192.0.2.9")],
        );
        assert!(
            stranded.contains(&rust_i18n::t!("ipam.check.delete_address_seen").to_string()),
            "removing the record does not free an address the network still shows: {stranded}"
        );
        assert!(
            !stranded.contains(
                &rust_i18n::t!("ipam.check.delete_subnet_holds_many", count = 3).to_string()
            ),
            "🔴 and it must NOT answer with the SUBNET's sentence — true, and about the wrong \
             gesture, which is the defect this branch exists for: {stranded}"
        );
        assert!(
            render_delete_check(
                office(),
                None,
                Some(v4("192.0.2.16")),
                &two_static,
                &network,
                &two_static_ranges,
                &[v4("192.0.2.9")],
            )
            .is_empty(),
            "THE CONTROL: an address the network has never shown strands nothing, so the region \
             announces nothing at all"
        );

        // 🔴 **DECISION 1(a)'s OWN CARRIER: the range delete's refusal, which the check never
        // mentioned.** `delete_range` refuses while the range holds an address defined inside it —
        // this story's own arbitration — and the warning consulted the network and the offer while
        // never asking that question, then promised the removal was still possible over a press the
        // adapter answers 409. Measured live by the review before the repair.
        let abandons = render_delete_check(
            office(),
            Some((v4("192.0.2.10"), v4("192.0.2.20"))),
            None,
            &two_static,
            &network,
            &two_static_ranges,
            &[v4("192.0.2.15")],
        );
        assert!(
            abandons.contains(
                &rust_i18n::t!("ipam.check.delete_range_holds_one", count = 1).to_string()
            ),
            "the refusal this story minted must be announced before it is met: {abandons}"
        );
        assert!(
            !abandons.contains(&still_removes),
            "and the removal must not be promised over a gesture the adapter refuses: {abandons}"
        );

        // 🔑 THE CONTROL: the same range holding no defined address IS removable, so the closing
        // sentence comes back. Without this half, a check that simply never said it would pass.
        let removable = render_delete_check(
            office(),
            Some((v4("192.0.2.10"), v4("192.0.2.20"))),
            None,
            &two_static,
            &network,
            &two_static_ranges,
            &[v4("192.0.2.99")],
        );
        assert!(
            removable.contains(&still_removes),
            "a range that abandons nothing is removable and must still say so: {removable}"
        );

        // 🔴 **BOUNDS THAT NAME NO RANGE OF THIS SUBNET WARN ABOUT NOTHING.** Measured on the
        // binary before the repair: `first=0.0.0.0&last=255.255.255.255` reported ten addresses
        // "explained by" a range that does not exist, and a stale control whose range was removed in
        // another tab reaches exactly that.
        // ⚠️ **THE INTERVAL MUST BE ONE THAT WOULD OTHERWISE SPEAK, and the first version of this
        // assertion was not.** It used `.50–.60`: with the bounds check neutered that interval
        // contains no observed address and strikes no range from the offer, so the body is empty
        // either way and the assertion passed under the mutation written to red it — measured, M-C4
        // GREEN with no test named. *A guard placed where the defect cannot occur reads as coverage
        // and is none* — this epic's dominant class, committed inside the repair for a review that
        // found it, and caught only because the driver contradicted a prediction. `.14–.16` holds
        // the seen `.15`, so without the check it emits the *addresses seen inside this range* line.
        assert!(
            render_delete_check(
                office(),
                Some((v4("192.0.2.14"), v4("192.0.2.16"))),
                None,
                &two_static,
                &network,
                &two_static_ranges,
                &[],
            )
            .is_empty(),
            "a warning was invented for an interval no range of this subnet carries"
        );
    }

    /// Every `ipam.` key this FILE NAMES ADJACENTLY resolves in BOTH locales, and is not blank in
    /// either.
    ///
    /// ⚠️ **That is narrower than *"every key this module can render"*, which is what this doc and
    /// this test's name both claimed until the slice-C review — and the gap is measurable rather
    /// than theoretical.** The screen renders `triage.kind.ecart` and `state.undeclared` in
    /// production code, and a scan keyed on the `ipam.` namespace is blind to both; a key reached
    /// through a function is covered only where that function's own literals happen to live in this
    /// file. *An enumeration cannot claim the completeness of a property* — so the two known
    /// strangers are asserted below BY NAME, which closes them without pretending the list is
    /// closed. The honest reading is a tripwire over one namespace, not a proof over the module.
    ///
    /// 🔴 **THIS MODULE HAD NO SUCH GUARD AT ALL until story 14.4, and the absence was invisible
    /// because absence always is.** `ipam_write.rs` has guarded its own keys since 14.2b; this file
    /// renders more of them than that one does and was covered by nothing — so thirteen keys added
    /// by this story went in under a green suite, ten gates and a clean clippy. ⚠️ What it costs to
    /// be missing is measured, not supposed: story 6b.10 set one `fr` value to its own key name and
    /// watched 702 tests and nine gates stay green while the French page rendered English, and a
    /// BLANK `fr` served a button with no label at all — *rendering nothing is worse than rendering
    /// the other language*.
    ///
    /// ⚠️ The needle is assembled at RUNTIME. The first version of its sibling was not: a scan of
    /// its own source matches the literal spelling the scan is written with, so it found itself,
    /// and the repair that explained the trap in prose containing the sequence reddened on the
    /// explanation. *A guard that greps a file greps its prose*, so neither the quote nor the
    /// namespace is written adjacently here.
    #[test]
    fn every_ipam_key_this_file_names_resolves_in_both_locales() {
        let source = include_str!("ipam_page.rs");
        let needle = format!("{}ipam.", '"');
        let mut keys: Vec<&str> = Vec::new();
        let mut rest = source;
        while let Some(at) = rest.find(&needle) {
            let after = &rest[at + 1..];
            let end = after.find('"').expect("a closed string literal");
            let name = &after[..end];
            // ⚠️ **A NAMESPACE PREFIX IS NOT A KEY.** The first run of this guard collected
            // `ipam.policy.` — the literal in `the_policy_words_are_the_binding_tables_own`'s
            // `starts_with`, which is a namespace and resolves to nothing. Skipped as a PROPERTY
            // (a key never ends in its separator) rather than by naming that one string, which
            // would be an enumeration standing in for a rule — and this file has already paid for
            // that distinction more than once.
            if !name.ends_with('.') && !keys.contains(&name) {
                keys.push(name);
            }
            rest = &after[end..];
        }
        // 🔑 A count EQUAL to what is there, never a floor under it — and this number is moved only
        // after READING the list the failure prints, which is why it ships first as a placeholder
        // that cannot be right.
        // 🔑 73 → 75 at the slice-C review, and the delta was READ off the list this assertion
        // prints rather than inferred from the arithmetic: the two additions are
        // `ipam.check.delete_range_holds_one` and `…_many`, the sentences decision 1(a) mints so the
        // range delete's own refusal is announced before it is met.
        // 🔑 75 → 76 at story 14.4b, read off the printed list: `ipam.finding.release`, the binding
        // gesture's word on a finding's control.
        assert_eq!(
            keys.len(),
            76,
            "the keys this file can render changed — update the count only after reading the list: \
             {keys:?}"
        );
        for locale in ["en", "fr"] {
            for name in &keys {
                let sentence = rust_i18n::t!(*name, locale = locale).to_string();
                assert_ne!(
                    &sentence, name,
                    "`{name}` has no `{locale}` translation, so the operator reads its key name"
                );
                assert!(
                    !sentence.trim().is_empty(),
                    "`{name}` is BLANK in `{locale}` — rendering nothing is worse than rendering \
                     the other language"
                );
            }
        }
        // ⚠️ **THE TWO KEYS THIS FILE RENDERS THAT THE SCAN ABOVE CANNOT SEE**, named rather than
        // discovered: `word_gap` and `word_undeclared` are built from `triage.kind.ecart` and
        // `state.undeclared` in PRODUCTION code, and a needle keyed on the `ipam.` namespace is
        // blind to both. They are the audit's own vocabulary, borrowed from the triage screen, which
        // is exactly why they carry someone else's prefix. *An enumeration cannot claim the
        // completeness of a property* — this closes the two that are known and says so, where the
        // test's old name went on promising the whole module.
        for locale in ["en", "fr"] {
            for name in ["triage.kind.ecart", "state.undeclared"] {
                let sentence = rust_i18n::t!(name, locale = locale).to_string();
                assert_ne!(
                    sentence, name,
                    "`{name}` has no `{locale}` translation, so the operator reads its key name"
                );
                assert!(
                    !sentence.trim().is_empty(),
                    "`{name}` is BLANK in `{locale}` — rendering nothing is worse than rendering \
                     the other language"
                );
            }
        }
    }
}
