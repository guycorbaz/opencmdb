//! The three `GET` routes that WARN before a write and refuse nothing (stories 14.2b–14.4).
//!
//! 🔴 **SPLIT OUT OF `ipam_page.rs` AT STORY 14.6, and the split was MEASURED AND REGISTERED a story
//! earlier so that this one did not have to re-take the decision.** Story 14.5's code review found
//! that file at 1992 code lines of the 2000 the `file-size` gate allows, extracted the rail (99
//! lines) as the smallest safe cut, and wrote down what the NEXT one should be: *"the three `GET`
//! checks … ~550 lines and a sharp concept of their own — routes that warn before a write and refuse
//! nothing"*, with the reason the bigger cut was not taken then (*it would have shipped an
//! unreviewed restructure to buy headroom a hundred lines already bought*). Story 14.6 crossed the
//! ceiling at **2015**, and `CLAUDE.md`'s rule is *split, not grown*.
//!
//! 🔑 **The concept is one sentence: these routes ANSWER A QUESTION and change nothing.** They are
//! `GET`, they are reachable by no `WriteRoute`, and a refusal is not among their answers — the
//! write path decides that. Story 14.4's third review round is why they say so out loud: the delete
//! check appended *"the removal is still possible — not a refusal"* to every non-empty answer while
//! the write path answered 409, so one `aria-live` region contradicted itself.
//!
//! ⚠️ **They are still mounted by `ipam_page::router`**, which is the one thing a reader must not
//! have to discover: the split moved the bodies, not the addresses.

use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse, Response};
use opencmdb_core::ipam::IpPolicy;
use std::net::IpAddr;

use crate::ipam_audit::{Network, Plan};
use crate::ipam_page::{IpamState, capped_sighting_lines};
use crate::ipam_repo::{self, Subnet};
use askama::Template;
use sqlx::MySqlPool;

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
    /// The address as typed so far. Anything that is not yet an address the plan can hold is
    /// answered with an empty warning, before the store is touched.
    ///
    /// ⚠️ It said **IPv4** until story 14.6's second review round — true of the handler it
    /// documents until that story widened the parse to `IpAddr`, and false in the commit that
    /// widened it. Measured on a booted binary: `?addr=2001:db8:1466::100` is answered normally.
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
pub(crate) async fn address_check(
    State(state): State<IpamState>,
    Query(query): Query<AddressCheckQuery>,
) -> Response {
    let Some(addr) = query
        .addr
        .as_deref()
        .and_then(|typed| typed.trim().parse::<IpAddr>().ok())
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
pub(crate) fn render_check_unavailable() -> String {
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
    addr: IpAddr,
) -> Result<String, opencmdb_core::repo::RepositoryError> {
    let plan = Plan {
        subnets: ipam_repo::list_subnets(pool)
            .await?
            .into_iter()
            .map(|planned| planned.subnet)
            .collect(),
        ranges: ipam_repo::plan_ranges(pool).await?,
        defined: ipam_repo::plan_addresses(pool).await?.into_iter().collect(),
    };
    let network = crate::ipam_audit::read_the_network(pool).await?;
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
pub(crate) async fn range_check(
    State(state): State<IpamState>,
    Query(query): Query<RangeCheckQuery>,
) -> Response {
    let parse = |typed: &Option<String>| {
        typed
            .as_deref()
            .and_then(|text| text.trim().parse::<IpAddr>().ok())
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
    first: IpAddr,
    last: IpAddr,
) -> Result<String, opencmdb_core::repo::RepositoryError> {
    let network = crate::ipam_audit::read_the_network(pool).await?;
    Ok(render_range_check(first, last, &network))
}

/// Render what is worth knowing about a `static` range before it is defined — the empty string when
/// the network has been seen on none of its addresses.
pub(crate) fn render_range_check(first: IpAddr, last: IpAddr, network: &Network) -> String {
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
pub(crate) async fn delete_check(
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
    // `IpAddr::from_str` refuses for its leading zeros): today the rail renders dotted, so this
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
                first.trim().parse::<IpAddr>(),
                last.trim().parse::<IpAddr>(),
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
            let Ok(addr) = typed.trim().parse::<IpAddr>() else {
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
    bounds: Option<(IpAddr, IpAddr)>,
    addr: Option<IpAddr>,
) -> Result<String, opencmdb_core::repo::RepositoryError> {
    let subnets = ipam_repo::list_subnets(pool).await?;
    let Some(here) = subnets
        .iter()
        .find(|planned| planned.id == subnet_id)
        .cloned()
    else {
        // An id the plan does not carry warns about nothing: the control that sent it is stale, and
        // a sentence invented for it would describe a record that is not there.
        return Ok(String::new());
    };
    let plan = Plan {
        subnets: subnets.iter().map(|planned| planned.subnet).collect(),
        ranges: ipam_repo::plan_ranges(pool).await?,
        defined: ipam_repo::plan_addresses(pool).await?.into_iter().collect(),
    };
    let subnet = here.subnet;
    let network = crate::ipam_audit::read_the_network(pool).await?;
    // 🔑 **THIS SUBNET'S OWN RECORDS, which this function already read for its `held` count.** The
    // check now asks the same questions the WRITE path asks, and asking them needs the same scope:
    // `delete_range` refuses by scanning `ip_address WHERE subnet_id = ?` and comparing in Rust, so
    // the warning that predicts that refusal compares the same rows the same way. Nothing plan-wide
    // can answer it — the review measured a warning computed over every range in the plan.
    let ranges_here: Vec<(IpAddr, IpAddr, IpPolicy)> =
        ipam_repo::correctable_ranges_in(pool, subnet_id)
            .await?
            .into_iter()
            .map(|(_, first, last, policy, _)| (first, last, policy))
            .collect();
    let defined_here: Vec<IpAddr> = ipam_repo::correctable_addresses_in(pool, subnet_id)
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
    bounds: Option<(IpAddr, IpAddr)>,
    addr: Option<IpAddr>,
    plan: &Plan,
    network: &Network,
    ranges_here: &[(IpAddr, IpAddr, IpPolicy)],
    defined_here: &[IpAddr],
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
pub(crate) fn counted(one: &str, many: &str, count: usize) -> String {
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
pub(crate) fn render_address_check(addr: IpAddr, plan: &Plan, network: &Network) -> String {
    let address = addr.to_string();
    let mut lines = Vec::new();
    let mut sightings = String::new();
    let mut triage_href = None;
    let mut claimed_note = String::new();
    let is_documented = network.documented.contains(&addr);
    // Triage's own comparison, verbatim — see `crate::ipam_audit::AddressFinding::claimed`.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every `ipam.` key THIS file names resolves in both locales.
    ///
    /// 🔴 **The guard moved with the code, and that is the point of it moving.** Story 14.6's split
    /// took ~600 lines out of `ipam_page.rs`, and its key guard reads `include_str!("ipam_page.rs")`
    /// — so every key this file names would have left that guard's population WITHOUT reddening it,
    /// since a smaller list is not a failure. ⚠️ *A guard keyed on a file measures the file, not the
    /// concept*, and a split is exactly when the two come apart.
    #[test]
    fn every_ipam_key_this_file_names_resolves_in_both_locales() {
        let source = include_str!("ipam_checks.rs");
        let needle = format!("{}ipam.", '"');
        let mut keys: Vec<&str> = Vec::new();
        let mut rest = source;
        while let Some(at) = rest.find(&needle) {
            let after = &rest[at + 1..];
            let end = after.find('"').expect("a closed string literal");
            let name = &after[..end];
            // A namespace prefix is not a key — `ipam_page`'s own guard's measured lesson.
            if !name.ends_with('.') && !keys.contains(&name) {
                keys.push(name);
            }
            rest = &after[end..];
        }
        // A count EQUAL to what is there, moved only after READING the list the failure prints.
        assert_eq!(
            keys.len(),
            19,
            "the keys this file can render changed — update the count only after reading the list: \
             {keys:?}"
        );
        for locale in ["en", "fr"] {
            for name in &keys {
                let sentence = rust_i18n::t!(*name, locale = locale).to_string();
                assert_ne!(
                    sentence, *name,
                    "`{name}` renders its own key name in `{locale}`"
                );
                assert!(
                    !sentence.trim().is_empty(),
                    "`{name}` is BLANK in `{locale}` — rendering nothing is worse than rendering \
                     the other language"
                );
            }
        }
    }

    /// 🔑 **The three addresses did NOT move with the bodies**, which is the one thing a split must
    /// not change. `ipam_page::router` mounts them and `main.rs`'s perimeter guard names them; this
    /// asserts the constants themselves so a rename here reds before either of those does.
    #[test]
    fn the_three_check_paths_are_what_the_page_mounts() {
        assert_eq!(ADDRESS_CHECK_PATH, "/ipam/address-check");
        assert_eq!(RANGE_CHECK_PATH, "/ipam/range-check");
        assert_eq!(DELETE_CHECK_PATH, "/ipam/delete-check");
        for path in [ADDRESS_CHECK_PATH, RANGE_CHECK_PATH, DELETE_CHECK_PATH] {
            assert!(
                path.starts_with("/ipam/") && path.ends_with("-check"),
                "`{path}` left the shape the three share, and a reader who knows one knows all three"
            );
        }
    }
}
