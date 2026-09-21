//! The addressing plan's write routes — the operator's hands on the plan (story 14.2b).
//!
//! Story 14.2 drew the plan and could not fill it: `/ipam` reads the store, and nothing in the
//! product had ever written an `ip_subnet`, an `ip_range` or an `ip_address` outside a test. This
//! module is the producer, and **`POST /ipam/subnet` is the FIRST of the three** because the other
//! two need a subnet to hang from — with two routes an empty plan stays empty for ever (§1(a)).
//!
//! # What this file carries, and what it deliberately does not
//!
//! Epic 14's constraint (1): *the write cost is front-loaded, and the FIRST new route carries the
//! shared machinery*. Here that is the Origin check, a KEYED refusal body per status in both
//! locales, and the exhaustive mapping of [`RepositoryError`]. The second and third routes reuse
//! them, and a test asserts the reuse rather than a comment claiming it.
//!
//! 🔑 **The Origin check is SHARED and the refusal BODIES are not** (Guy, 2026-09-12). The check is
//! [`crate::write_guard::same_origin`], reached from here and from `document.rs`, with a tripwire
//! asserting the comparison exists in exactly one file: *a CSRF check is the one function that must
//! never be duplicated*. The sentences are local, because `document.malformed` and
//! `ipam.refusal.malformed_subnet` are different sentences about different gestures, and one body serving
//! two surfaces is the shape story 6.4 paid for.
//!
//! ⚠️ **Nothing here writes provenance, so the `authorship` gate owes this file no sanction.** The
//! plan is a SECOND declared register beside `declared_attribute` (Guy's arbitration (4) of
//! 2026-09-10); `ip_subnet` has no `origin` column and no author. Said rather than left to be
//! inferred from the gate staying green.
//!
//! 🔴 **The two registers are COMPARED since story 14.3b, and this doc said they *never touch*.**
//! The audit reads the documented `ipv4` values to decide what may be offered (decision 4) and what
//! triage has a question about — through `ipam_audit.rs` alone, which is the one module the plan's
//! guard allows to reach them. They are still not FUSED, which is what Guy's arbitration protects:
//! *IPAM says what was MEANT to be there, `declared_attribute` says what is DOCUMENTED*, and an
//! address documented outside the plan stays visible as exactly that. ⚠️ Nothing changed for THIS
//! file — `IpamWriteState` holds a port and no pool, so it cannot read either register.
//!
//! # The state holds a PORT and no pool, and that is a compile-time property
//!
//! [`IpamWriteState`] has no pool field, so a handler here cannot extract `State<MySqlPool>` —
//! story 6.1's M4 carrier, re-measured by two review layers on `document.rs` and measured again for
//! this module (AC2). The pool lives INSIDE [`StoreIpamWrite`], behind `Arc<dyn IpamWritePort>`.
//!
//! ⚠️ `/ipam` itself is on its own POOL-BEARING router (`ipam_page::router`) since story 14.2, so
//! the two must not be fused: merging these routes onto that one silently retires the guarantee.
//!
//! # No switch
//!
//! `OPENCMDB_DOCUMENT_ENABLED` guards an AUTHORSHIP hazard — a route that turns an observed value
//! into a declared one — which the plan does not have. These routes are always mounted, above
//! `auth_deny` like every other non-public path. ⚠️ A fresh install therefore gains a live write
//! surface with no opt-in, which the release notes owe a sentence.

use std::net::Ipv4Addr;
use std::sync::Arc;

use axum::extract::State;
use axum::extract::rejection::FormRejection;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodRouter, post};
use axum::{Form, Router};
use opencmdb_core::ipam::IpamError;
use opencmdb_core::repo::{BoxFuture, RepositoryError};
use serde::Deserialize;
use sqlx::MySqlPool;

use crate::ipam_repo::{self, Subnet};

/// The longest label the plan's three tables accept.
///
/// ⚠️ **It is refused HERE rather than left to the store, and the reason is not tidiness**: `label`
/// is `VARCHAR(120)` on all three tables, so a longer value gives the operator a raw driver
/// sentence (`1406: Data too long`) — and under a non-strict `sql_mode` it is worse than that, the
/// value being silently TRUNCATED instead. *A refusal the operator can read beats a truncation
/// nobody sees.*
///
/// 🔑 The unit is CHARACTERS, not bytes: MariaDB counts `VARCHAR(120)` in characters under
/// `utf8mb4`, so « Réseau des imprimantes » must be measured the way the column measures it.
const MAX_LABEL_CHARS: usize = 120;

/// A refusal the operator can read: the status the rule earns, and the KEY that names it.
///
/// 🔑 **It is DATA and not a `Response`**, for two reasons that turned out to be the same one. The
/// mappers below are then pure, so a test can assert WHICH rule a refusal names rather than
/// scraping a rendered body — which is what AC4's SET test needs. And clippy's
/// `result_large_err` refuses a `Result<_, Response>` outright (measured: *the `Err`-variant is at
/// least 128 bytes*), so the lint and the design pointed the same way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Refusal {
    /// The status this rule earns.
    status: StatusCode,
    /// The i18n key naming the rule, never a sentence — story 6.4's finding: a body built with
    /// `format!` ships in English under a French UI.
    key: &'static str,
}

impl Refusal {
    /// A refusal at `status`, naming `key`.
    const fn new(status: StatusCode, key: &'static str) -> Self {
        Self { status, key }
    }

    /// The status this refusal answers with.
    pub(crate) const fn status(self) -> StatusCode {
        self.status
    }

    /// The key naming the rule that was broken.
    pub(crate) const fn key(self) -> &'static str {
        self.key
    }
}

impl IntoResponse for Refusal {
    fn into_response(self) -> Response {
        (self.status(), rust_i18n::t!(self.key()).to_string()).into_response()
    }
}

/// How long a write may hold the browser before the product answers instead of waiting.
///
/// 🔴 **This story adds the product's FIRST BLOCKING WRITE, and the loser's wait tracks the
/// holder's**: measured at the validation, a 3 s hold gave a **6.005 s** round trip, bounded only
/// by `innodb_lock_wait_timeout` — **50 s** on a stock container. ⚠️ Story 6b.10 put a per-handler
/// budget on the GET screens after measuring a 30 s hang; every budgeted file in this product is a
/// screen, and `document.rs` carries **zero**. This is where a write can do it.
///
/// 🔑 **The budget wraps the whole TRANSACTION and not its first read** — story 14.2's denial of
/// service was exactly a budget around the wrong half, `store_within` wrapping the reads while the
/// derivation and the render ran unbounded after them.
///
/// ⚠️ Five seconds, the same figure as [`crate::page::PAGE_STORE_BUDGET`], and the reason is that
/// there is no measurement that would justify a different one. *A number invented to look
/// considered is a number with nothing behind it.*
const IPAM_WRITE_BUDGET: std::time::Duration = std::time::Duration::from_secs(5);

/// Run one write within [`IPAM_WRITE_BUDGET`], answering rather than holding the browser.
///
/// ⚠️ **A timeout is reported to the operator as [`RepositoryError::Contention`], deliberately.**
/// The two are different facts — *the store never answered* against *the store said deadlock* — and
/// they call for the same ACT: reload the plan, and try again if the change is missing. The
/// distinction that matters to whoever is debugging is kept where it belongs, in the log.
///
/// 🔴 **This doc promised *"nothing was written is true and not a hope"*, and the review measured it
/// false.** The subnet and address writes ran in AUTOCOMMIT, so a statement already at the server
/// when the future was dropped committed AFTER the 503: a held parent row, `POST /ipam/address`
/// answered at 5.002 s, the row present once the holder committed, and the retry refused as
/// already defined. All three writes now run inside a transaction, so a write dropped BEFORE its
/// commit is rolled back with its connection — `a_write_dropped_while_waiting_leaves_no_row_behind`.
/// ⚠️ **What no transaction settles is a timeout DURING `COMMIT`**: the server may have committed
/// and the answer never arrived. So the operator's sentence does not promise either way — *it may
/// have been saved; reload the plan* — which is true in every case (Guy, 2026-09-15).
async fn within_budget<T>(
    work: impl std::future::Future<Output = Result<T, RepositoryError>>,
) -> Result<T, RepositoryError> {
    match tokio::time::timeout(IPAM_WRITE_BUDGET, work).await {
        Ok(result) => result,
        Err(_elapsed) => {
            tracing::error!(
                budget_ms = IPAM_WRITE_BUDGET.as_millis(),
                "the addressing plan's write did not finish within its budget — refusing rather \
                 than holding the browser"
            );
            Err(RepositoryError::Contention)
        }
    }
}

/// One write route of the addressing plan.
///
/// 🔑 **The router is BUILT by iterating [`WriteRoute::ALL`]**, and every variant must answer
/// [`WriteRoute::path`] and [`WriteRoute::handler`] through an exhaustive `match`, so a variant
/// cannot exist without a path and a handler.
///
/// 🔴 **BUT `ALL` ITSELF IS A HAND-WRITTEN ARRAY, AND THE COMPILER SAYS NOTHING ABOUT WHAT IS
/// MISSING FROM IT.** This doc claimed that *"adding a route without mounting it is an
/// `error[E0004]`"*; that sentence was false when written and the review measured how far. A variant
/// left out of `ALL` is mounted by nothing, and **everything that looks like a check derives from
/// `ALL`**: `router_with` iterates it, [`WriteRoute::paths`] maps over it, `main.rs`'s nine-path
/// premise is built from it, and even this story's own route-coverage assertion compares against
/// `ALL.len()`. All of them agree, because none of them has a second opinion to disagree with.
/// What carries it instead is `every_variant_is_in_the_route_list`, a hand-written second list
/// pinned by an equality test — the deliberate redundancy this codebase sanctions, and the only
/// shape that can notice an omission.
///
/// AC3's guard walks this same list, which is what lets it assert both halves: 401 without a
/// credential, and something OTHER than 404 with one. ⚠️ Story 6b.2 measured why the positive half
/// is needed — `auth_deny` layers the fallback, so a misspelt path, an unmounted route and a typo
/// all answer 401 exactly like a real route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WriteRoute {
    /// `POST /ipam/subnet` — define a subnet, the entry the other two hang from.
    Subnet,
    /// `POST /ipam/range` — define a stretch of it and say what it is MEANT for.
    Range,
    /// `POST /ipam/address` — define one address.
    Address,
    /// `POST /ipam/subnet/delete` — remove a subnet. ⚠️ Its refusal is the DATABASE's: two foreign
    /// keys point at `ip_subnet`, so a subnet that still holds anything raises `ERROR 1451` with no
    /// count, no lock and no race.
    DeleteSubnet,
    /// `POST /ipam/range/delete` — remove a range. 🔴 Its refusal is COMPUTED, not raised: nothing
    /// relates an address to a range, so containment is arithmetic under the parent row's lock.
    DeleteRange,
    /// `POST /ipam/address/delete` — remove one defined address. Nothing points at it, so nothing
    /// can refuse it but the id being unknown.
    DeleteAddress,
    /// `POST /ipam/range/edit` — correct a range's bounds, its policy or its label.
    EditRange,
    /// `POST /ipam/address/edit` — correct an address or what the operator calls it.
    EditAddress,
    /// `POST /ipam/subnet/edit` — correct a subnet's LABEL and its VLAN, and nothing else (story
    /// 14.5). ⚠️ Never its base and never its prefix: the scope is carried by
    /// [`EditSubnetRequest`]'s fields, not by a sentence — story 14.4's arbitration.
    EditSubnet,
    /// `POST /ipam/release` — stop the plan holding an address the network was seen on (story
    /// 14.4b, the binding gesture `release` / « libérer »). 🔑 It writes a PLAN row
    /// (`address_release`) and never touches a sighting, which is what keeps the audit a reader.
    Release,
}

impl WriteRoute {
    /// Every write route this sub-router carries.
    pub(crate) const ALL: &'static [WriteRoute] = &[
        WriteRoute::Subnet,
        WriteRoute::Range,
        WriteRoute::Address,
        WriteRoute::DeleteSubnet,
        WriteRoute::DeleteRange,
        WriteRoute::DeleteAddress,
        WriteRoute::EditRange,
        WriteRoute::EditAddress,
        WriteRoute::Release,
        WriteRoute::EditSubnet,
    ];

    /// The status a write that went through earns.
    ///
    /// 🔴 **A correction answers 200, never 201, and this is a DECISION rather than a detail.**
    /// `every_route_reuses_the_shared_machinery` asserted `CREATED` for every route, so the cheap
    /// path was to let a deletion claim it had created something — a false statement in the
    /// protocol, kept true only by the guard demanding it. ⚠️ This project has shipped that shape
    /// twice: story 6b.4's route test asserted the raw UUID it was meant to remove, and story
    /// 14.2's asserted `offerable == 256` over the edge addresses. *A test that pins the wrong
    /// behaviour is a test that requires it*, so the guard became per-route instead.
    pub(crate) const fn success_status(self) -> StatusCode {
        match self {
            WriteRoute::Subnet | WriteRoute::Range | WriteRoute::Address => StatusCode::CREATED,
            WriteRoute::DeleteSubnet
            | WriteRoute::DeleteRange
            | WriteRoute::DeleteAddress
            | WriteRoute::EditRange
            | WriteRoute::EditAddress
            | WriteRoute::EditSubnet
            // 🔑 200 and not 201: a second release of one address WIDENS the row it already has
            // (`ipam_repo::release_address`), so the route cannot promise it created anything.
            | WriteRoute::Release => StatusCode::OK,
        }
    }

    /// The sentence a [`RepositoryError::NotFound`] earns, which NAMES THE RECORD THE ID MEANT.
    ///
    /// 🔴 **One sentence served every route until this story, and it was false on four of them**:
    /// `NotFound` answered *"that subnet is not in the plan"*, so an edit of a range the plan no
    /// longer holds told the operator their SUBNET was gone — and sent them to look in the wrong
    /// place. The same defect `already_defined` carried at 14.2b's review, one error variant over.
    /// ⚠️ It read *"false on five"* until the review: that is the count of routes this story ADDS,
    /// and `DeleteSubnet` is one of them where the old sentence was right. *Two populations, one
    /// number* — and under the other reading it is incoherent, the sentence having served three
    /// routes when it could not have been false on five.
    const fn unknown_record(self) -> Refusal {
        Refusal::new(
            StatusCode::NOT_FOUND,
            match self {
                // ⚠️ **The four routes for which a missing row can only be a SUBNET, and the reason
                // is different for each pair.** `Range` and `Address` are looked up through the
                // `subnet_id` their form carries, so the absent row is that parent. `Subnet` and
                // `DeleteSubnet` address a subnet directly. (This comment read *"the three
                // definitions receive a `subnet_id`"* until the review, which is false of `Subnet`
                // — it carries a CIDR and no parent at all — and silent about `DeleteSubnet`.)
                WriteRoute::Subnet
                | WriteRoute::Range
                | WriteRoute::Address
                | WriteRoute::DeleteSubnet => "ipam.refusal.unknown_subnet",
                WriteRoute::DeleteRange | WriteRoute::EditRange => "ipam.refusal.unknown_range",
                // A correction of a subnet addresses one directly, like its definition and its
                // delete — so an id naming no row is the same sentence.
                WriteRoute::EditSubnet => "ipam.refusal.unknown_subnet",
                WriteRoute::DeleteAddress | WriteRoute::EditAddress => {
                    "ipam.refusal.unknown_address"
                }
                // 🔑 **A release names an ADDRESS, not a record**, and the address needs no row to
                // exist: the write is an upsert and raises no `NotFound`. Reaching here from it is a
                // fault in this product — answered with the backend's STATUS as well as its
                // sentence, where the review found a dead arm pairing a 404 with that sentence.
                WriteRoute::Release => return backend(),
            },
        )
    }

    /// Whether this route's form carries a label the label rules can judge.
    ///
    /// 🔴 **A deletion names a record; it does not describe one.** Its form has no `label` field at
    /// all, and serde DROPS an unknown field rather than refusing it — so driving a blank or a
    /// control-character label at a delete route sends a field nothing reads, the write goes
    /// through, and a guard demanding 422 would red over a correct product. ⚠️ Without this
    /// predicate the reuse test would have been *fixed* by giving the delete forms a label they have
    /// no use for, which is a test shaping the product rather than measuring it.
    /// ⚠️ **`#[cfg(test)]`, because its one reader is a guard** — [`WriteRoute::paths`]'s own note,
    /// two functions down, records that a production-side list with no production reader is
    /// `never used` under `-D warnings`, *measured twice*. Left in production this reddened
    /// `clippy --workspace --all-targets`, which is what CI runs and what the local
    /// `clippy --workspace` alone cannot see.
    #[cfg(test)]
    pub(crate) const fn carries_label(self) -> bool {
        match self {
            WriteRoute::Subnet
            | WriteRoute::Range
            | WriteRoute::Address
            | WriteRoute::EditRange
            | WriteRoute::EditAddress
            | WriteRoute::EditSubnet => true,
            WriteRoute::DeleteSubnet
            | WriteRoute::DeleteRange
            | WriteRoute::DeleteAddress
            | WriteRoute::Release => false,
        }
    }

    /// Every path this sub-router carries — AC3's list, DERIVED from the variants.
    ///
    /// 🔴 **The criterion says `PATHS: &[&str]`, and a const of that shape was written first and
    /// then removed: clippy called it `never used`.** Nothing in production could read it — the
    /// router is built from the variants, because only a variant carries its handler — so the const
    /// existed for the guard alone, which is the compiler saying *this is a second spelling with no
    /// reader*. Pinning the two together with a test was the alternative; deriving them removes the
    /// question. 🔑 *The house rule keeps a redundancy a test pins; it does not ask for one where
    /// there need be none.* Same shape as `Screen::ALL`, `IpPolicy::ALL`, `FactKind::ALL`.
    ///
    /// ⚠️ **`#[cfg(test)]`, because it has exactly one reader and that reader is a guard.** Left in
    /// production it is `never used` under `-D warnings` — measured TWICE, first as a const and
    /// then as this function. `document::PATHS` is a const in production because ITS router really
    /// is built from it; here only a variant carries its handler. *Each list lives where its reader
    /// is, and neither pretends to a use it has not got.*
    #[cfg(test)]
    pub(crate) fn paths() -> Vec<&'static str> {
        Self::ALL.iter().map(|route| route.path()).collect()
    }

    /// The path the route is mounted at, and the path the guard probes.
    pub(crate) const fn path(self) -> &'static str {
        match self {
            WriteRoute::Subnet => "/ipam/subnet",
            WriteRoute::Range => "/ipam/range",
            WriteRoute::Address => "/ipam/address",
            WriteRoute::DeleteSubnet => "/ipam/subnet/delete",
            WriteRoute::DeleteRange => "/ipam/range/delete",
            WriteRoute::DeleteAddress => "/ipam/address/delete",
            WriteRoute::EditRange => "/ipam/range/edit",
            WriteRoute::EditAddress => "/ipam/address/edit",
            WriteRoute::Release => "/ipam/release",
            WriteRoute::EditSubnet => "/ipam/subnet/edit",
        }
    }

    /// The shape refusal this form earns — one sentence per form, naming ITS fields.
    const fn malformed(self) -> Refusal {
        Refusal::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            match self {
                WriteRoute::Subnet => "ipam.refusal.malformed_subnet",
                WriteRoute::Range => "ipam.refusal.malformed_range",
                WriteRoute::Address => "ipam.refusal.malformed_address",
                // 🔑 Five sentences and not one shared one, because the reuse guard asserts the
                // shape sentences are DISTINCT per route — which is exactly what stops a form
                // answering with a sentence naming fields it does not carry.
                WriteRoute::DeleteSubnet => "ipam.refusal.malformed_delete_subnet",
                WriteRoute::DeleteRange => "ipam.refusal.malformed_delete_range",
                WriteRoute::DeleteAddress => "ipam.refusal.malformed_delete_address",
                WriteRoute::EditRange => "ipam.refusal.malformed_edit_range",
                WriteRoute::EditAddress => "ipam.refusal.malformed_edit_address",
                WriteRoute::Release => "ipam.refusal.malformed_release",
                WriteRoute::EditSubnet => "ipam.refusal.malformed_edit_subnet",
            },
        )
    }

    /// The conflict a re-entered record earns, naming WHICH record is already there.
    ///
    /// 🔴 **One sentence served all three routes until story 14.2b's review, and it named the
    /// subnet**: a re-entered ADDRESS was told *"the plan already holds that subnet"* (measured,
    /// 409). `Constraint("unique")` carries no index name, so the route is what knows which record
    /// collided.
    const fn already_defined(self) -> Refusal {
        Refusal::new(
            StatusCode::CONFLICT,
            match self {
                // 🔑 Grouped by the RECORD rather than by the route, because the sentence names the
                // record that collided and a `unique` violation knows nothing about which gesture
                // provoked it.
                // 🔴 **THE DEFINITION AND THE CORRECTION SHARE THIS ONE, and story 14.4's
                // arbitration is what settles it rather than being contradicted by it.** That
                // arbitration says a refusal must name the RECORD the operator addressed; here both
                // gestures collide on the very same key — `ip_subnet_cidr_vlan`, minted by `0010` —
                // so a second sentence would not name a second record, it would name the same one
                // twice. ⚠️ What DID have to change is the sentence itself: since `0010` the CIDR
                // alone no longer decides, so *"the plan already holds that subnet"* was true and
                // under-informative in front of an operator looking at that CIDR under another tab.
                // It names the rule now. **Taken by me on delegation and recorded as mine so it can
                // be reversed at the right cost.**
                WriteRoute::Subnet | WriteRoute::EditSubnet => {
                    "ipam.refusal.already_defined_subnet"
                }
                WriteRoute::Range | WriteRoute::EditRange => "ipam.refusal.already_defined_range",
                WriteRoute::Address | WriteRoute::EditAddress => {
                    "ipam.refusal.already_defined_address"
                }
                // 🔴 **THE THREE DELETES ANSWER THE BACKEND SENTENCE, and this is the ONLY place
                // that says so.** A `DELETE` writes no row and can hit no unique key, so reaching
                // here from one is a fault in this product rather than a mistake an operator can
                // correct. ⚠️ The review found this function mapping them to *already defined*
                // while [`constraint_refusal`] routed them to [`backend`] — **two statements of one
                // delete's sentence, disagreeing**, with the doc here describing copy the product
                // never emits. One statement now, and the caller no longer decides.
                // 🔑 The release is an UPSERT on its own primary key, so it cannot collide either.
                WriteRoute::DeleteSubnet
                | WriteRoute::DeleteRange
                | WriteRoute::DeleteAddress
                | WriteRoute::Release => {
                    return backend();
                }
            },
        )
    }

    /// The method router mounted at [`WriteRoute::path`].
    fn handler(self) -> MethodRouter<IpamWriteState> {
        match self {
            WriteRoute::Subnet => post(define_subnet),
            WriteRoute::Range => post(define_range),
            WriteRoute::Address => post(define_address),
            WriteRoute::DeleteSubnet => post(delete_subnet),
            WriteRoute::DeleteRange => post(delete_range),
            WriteRoute::DeleteAddress => post(delete_address),
            WriteRoute::EditRange => post(edit_range),
            WriteRoute::EditAddress => post(edit_address),
            WriteRoute::Release => post(release_address),
            WriteRoute::EditSubnet => post(edit_subnet),
        }
    }
}

/// The `POST /ipam/subnet` request: `application/x-www-form-urlencoded`, ⚠️ because that is what
/// the vendored htmx 2.0.4 posts (measured at story 6.2: form-values encoding, zero `fetch(`, no
/// `json-enc` extension).
#[derive(Debug, Deserialize)]
pub(crate) struct DefineSubnetRequest {
    /// The subnet in CIDR notation, as an operator writes it — `192.0.2.0/24`.
    ///
    /// ⚠️ **NOT the stored spelling.** The store holds `192.000.002.000` so that lexicographic
    /// order is numeric order (story 14.1's arbitration); that padding is an implementation of
    /// ordering and never something an operator should type or read.
    pub(crate) cidr: String,
    /// What the operator calls this subnet. Required, and bounded by [`MAX_LABEL_CHARS`].
    pub(crate) label: String,
    /// The 802.1Q id the operator declares for it — OPTIONAL (story 14.5).
    ///
    /// 🔑 An absent field and an empty one both mean *none*, which the store holds as the sentinel
    /// `0`: a form that has never carried a VLAN and a form whose field was left blank are the same
    /// statement, and serde drops the field when the browser omits it.
    #[serde(default)]
    pub(crate) vlan: String,
}

/// The `POST /ipam/range` request.
#[derive(Debug, Deserialize)]
pub(crate) struct DefineRangeRequest {
    /// The subnet the range belongs to, by the id `/ipam`'s own selector carries.
    pub(crate) subnet_id: String,
    /// The first address of the range, as an operator writes it — `192.0.2.10`.
    pub(crate) first: String,
    /// The last address, inclusive.
    pub(crate) last: String,
    /// What the stretch is MEANT for, as one of the four binding tokens.
    pub(crate) policy: String,
    /// What the operator calls this range. Required, and bounded by [`MAX_LABEL_CHARS`].
    pub(crate) label: String,
}

/// The `POST /ipam/address` request.
#[derive(Debug, Deserialize)]
pub(crate) struct DefineAddressRequest {
    /// The subnet the address belongs to.
    pub(crate) subnet_id: String,
    /// The address itself, as an operator writes it.
    pub(crate) addr: String,
    /// What the operator calls it. Required, and bounded by [`MAX_LABEL_CHARS`].
    pub(crate) label: String,
}

/// The `POST /ipam/subnet/delete` request.
///
/// 🔑 **No `subnet_id` field, and its absence is the design**: the record being removed IS the
/// subnet, so there is nothing to send the browser back to. [`answer`] redirects to the plan itself.
#[derive(Debug, Deserialize)]
pub(crate) struct DeleteSubnetRequest {
    /// The subnet to remove, by the id `/ipam`'s own rail carries.
    pub(crate) id: String,
}

/// The `POST /ipam/range/delete` request.
///
/// 🔑 **One id, and no `subnet_id`** (Guy's decision 1, 2026-09-16). The field this form carried was
/// used for the redirect alone and was never compared with the range's actual parent; the adapter
/// now reads that parent off the row. See [`Written`] for what the review measured.
#[derive(Debug, Deserialize)]
pub(crate) struct DeleteRangeRequest {
    /// The range to remove.
    pub(crate) id: String,
}

/// The `POST /ipam/address/delete` request.
///
/// 🔑 One id, and no `subnet_id` — [`DeleteRangeRequest`]'s reason.
#[derive(Debug, Deserialize)]
pub(crate) struct DeleteAddressRequest {
    /// The defined address to remove.
    pub(crate) id: String,
}

/// The `POST /ipam/range/edit` request.
///
/// ⚠️ **Every field of the range, not only the changed ones.** A partial form would mean reading the
/// row to fill the gaps, and a read that decides what to write is the TOCTOU shape this module
/// refuses elsewhere by name — the operator's page holds the whole record, so the whole record is
/// what it sends.
#[derive(Debug, Deserialize)]
pub(crate) struct EditRangeRequest {
    /// The range being corrected.
    pub(crate) id: String,
    /// The first address of the range, as an operator writes it.
    pub(crate) first: String,
    /// The last address, inclusive.
    pub(crate) last: String,
    /// What the stretch is MEANT for, as one of the four binding tokens.
    pub(crate) policy: String,
    /// What the operator calls this range.
    pub(crate) label: String,
}

/// The `POST /ipam/address/edit` request.
#[derive(Debug, Deserialize)]
pub(crate) struct EditAddressRequest {
    /// The defined address being corrected.
    pub(crate) id: String,
    /// The address itself, as an operator writes it.
    pub(crate) addr: String,
    /// What the operator calls it.
    pub(crate) label: String,
}

/// The `POST /ipam/subnet/edit` request — the label and the VLAN, and NOTHING else.
///
/// 🔑 **The type is the scope**, and the mutation pass measured BOTH halves of what that buys —
/// where this comment first claimed only a tripwire, on story 5.12's habit of narrowing a promise.
/// Adding a CIDR field here and leaving it unread reds **clippy** (`field 'cidr' is never read`,
/// which CI runs with `--all-targets -D warnings`); adding it and READING it reds
/// `a_subnet_correction_carries_its_vlan_and_no_cidr`. Together they close the ordinary gesture in
/// both directions, and what stays open is someone editing the test too — which no guard reaches
/// and which is a decision rather than an accident. Story 6b.4b's idiom: close it in the type,
/// where a sentence is carried by nobody.
#[derive(Debug, Deserialize)]
pub(crate) struct EditSubnetRequest {
    /// The subnet being corrected.
    pub(crate) id: String,
    /// What the operator calls it.
    pub(crate) label: String,
    /// Its VLAN, or empty for none.
    #[serde(default)]
    pub(crate) vlan: String,
}

/// The `POST /ipam/release` request.
///
/// 🔑 **The address itself, and no id**: a release is keyed on the ADDRESS (`0009`), because the
/// sightings that hold it are keyed on the address too and no plan record need exist for it — most
/// held addresses are exactly the ones nothing in the plan names.
#[derive(Debug, Deserialize)]
pub(crate) struct ReleaseRequest {
    /// The address to release, as the findings list carries it.
    pub(crate) addr: String,
    /// The LAST instant the network had shown on it when the operator's page was rendered, as the
    /// finding carries it (RFC 3339) — what the release forgets up to (Guy, code review 2026-09-19).
    ///
    /// 🔴 **Not the press instant, and the review is why.** Every observation of a sweep is dated at
    /// the sweep's START (`arp_ping.rs`), so a release dated at the press forgot a host that answered
    /// in a sweep already running — and inside `static` offered its address until the next sweep. All
    /// three review layers reached it. *What the operator released is what they were shown*, and an
    /// instant taken from the data needs no clock, so a skewed clock and a database in another time
    /// zone stop mattering as well.
    pub(crate) seen_until: String,
}

/// What a write that went through answers with: the record it touched, and the plan to send the
/// operator back to.
///
/// 🔴 **`subnet_id` is READ BACK FROM THE RECORD on every route that addresses one by its own id,
/// and is never the value the form sent** (Guy's decision 1, 2026-09-16). Until this story the four
/// record-addressed forms carried a `subnet_id` beside the record's id, and the route used it as the
/// redirect target **without ever asking whether it was that record's subnet**. The review measured
/// both halves of the cost on the running binary: a mismatched pair removed an address from subnet A
/// and landed the operator on subnet B's plan, under a success sentence, looking at a page where
/// nothing had changed; and a malformed value in that field **refused a deletion it takes no part
/// in**. Reading the parent back removes the question instead of answering it — there is nothing
/// left to mismatch and nothing left to refuse.
///
/// 🔑 `None` means *the record IS the subnet*: `define_subnet` mints one and `delete_subnet` removes
/// one, and neither has a parent to return to. The two remaining definitions carry the form's
/// `subnet_id` and it is verified — **by the foreign key**, which is what the insert writes through:
/// a `subnet_id` naming no row cannot produce a written record at all.
#[derive(Debug)]
pub(crate) struct Written {
    /// The id of the record the gesture touched.
    pub(crate) id: String,
    /// The subnet to show the operator afterwards, or `None` when the record is itself a subnet.
    pub(crate) subnet_id: Option<String>,
}

impl Written {
    /// A write whose record is itself a subnet — there is no parent plan to return to.
    pub(crate) fn subnet(id: String) -> Self {
        Self {
            id,
            subnet_id: None,
        }
    }

    /// A write inside a subnet, which is where the operator is sent back to.
    pub(crate) fn inside(id: String, subnet_id: String) -> Self {
        Self {
            id,
            subnet_id: Some(subnet_id),
        }
    }
}

/// The port: one gesture of the plan, whole. The sub-router's state reaches the world only
/// through this, and whatever it needs to do so lives INSIDE the impl.
pub(crate) trait IpamWritePort: Send + Sync {
    /// Define a subnet and answer with the id it was given.
    ///
    /// # Errors
    ///
    /// Whatever the adapter refuses, as a [`RepositoryError`] — the handler maps it exhaustively.
    fn define_subnet(
        &self,
        subnet: Subnet,
        label: String,
        vlan: u16,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>>;

    /// Define a range inside a subnet and answer with the id it was given.
    ///
    /// # Errors
    ///
    /// Whatever the adapter refuses, as a [`RepositoryError`] — an unknown subnet, an overlap, a
    /// range outside its subnet, or the loser of a lock wait.
    fn define_range(
        &self,
        subnet_id: String,
        first: Ipv4Addr,
        last: Ipv4Addr,
        policy: opencmdb_core::ipam::IpPolicy,
        label: String,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>>;

    /// Define one address inside a subnet and answer with the id it was given.
    ///
    /// # Errors
    ///
    /// Whatever the adapter refuses — an unknown subnet, an address outside it, or the same
    /// address defined twice.
    fn define_address(
        &self,
        subnet_id: String,
        addr: Ipv4Addr,
        label: String,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>>;

    /// Remove a subnet, answering with the id that was removed.
    ///
    /// # Errors
    ///
    /// [`RepositoryError::NotFound`] when no subnet carries the id, or
    /// [`RepositoryError::Constraint`] `"foreign_key"` when it still holds ranges or addresses —
    /// the database's own refusal, on two foreign keys.
    fn delete_subnet(&self, id: String) -> BoxFuture<'_, Result<Written, RepositoryError>>;

    /// Remove a range, answering with the id that was removed **and the subnet it belonged to**.
    ///
    /// # Errors
    ///
    /// [`RepositoryError::NotFound`], or [`opencmdb_core::ipam::IpamError::RangeStillHoldsAddresses`]
    /// — **computed**, not raised: nothing relates an address to a range.
    fn delete_range(&self, id: String) -> BoxFuture<'_, Result<Written, RepositoryError>>;

    /// Remove one defined address, answering with the id that was removed **and its subnet**.
    ///
    /// # Errors
    ///
    /// [`RepositoryError::NotFound`] alone: nothing in the plan points at an address.
    fn delete_address(&self, id: String) -> BoxFuture<'_, Result<Written, RepositoryError>>;

    /// Correct a range's bounds, its policy and its label, answering with **its subnet**.
    ///
    /// # Errors
    ///
    /// [`RepositoryError::NotFound`], or the insert's own rules re-asked — outside its subnet,
    /// inverted bounds, or overlapping a SIBLING, which is the one rule whose scan differs — plus
    /// [`opencmdb_core::ipam::IpamError::RangeStillHoldsAddresses`] when the new bounds would
    /// abandon an address the old ones held (Guy's arbitration, 2026-09-16).
    fn edit_range(
        &self,
        id: String,
        first: Ipv4Addr,
        last: Ipv4Addr,
        policy: opencmdb_core::ipam::IpPolicy,
        label: String,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>>;

    /// Correct one defined address, or what the operator calls it, answering with **its subnet**.
    ///
    /// # Errors
    ///
    /// [`RepositoryError::NotFound`], an address outside its subnet, or the same address defined
    /// twice.
    fn edit_address(
        &self,
        id: String,
        addr: Ipv4Addr,
        label: String,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>>;

    /// Correct a subnet's label and its VLAN, answering with the subnet itself.
    ///
    /// # Errors
    ///
    /// [`RepositoryError::NotFound`] when the id names no row, or `Constraint("unique")` when the
    /// new VLAN is one another row of the same CIDR already holds.
    fn edit_subnet(
        &self,
        id: String,
        label: String,
        vlan: u16,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>>;

    /// Release one address — the plan forgets what the network showed on it up to `until`, the last
    /// sighting the operator was shown — answering with the address and **the innermost subnet
    /// containing it**, or none.
    ///
    /// # Errors
    ///
    /// [`opencmdb_core::ipam::IpamError::ReleaseOfADefinedAddress`] when the plan defines it (Guy's
    /// decision 4, 2026-09-19), or a backend failure.
    fn release_address(
        &self,
        addr: Ipv4Addr,
        until: opencmdb_core::observation::Timestamp,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>>;
}

/// The production wiring: the plan's gestures over a MariaDB pool, which lives HERE and not on
/// [`IpamWriteState`].
pub(crate) struct StoreIpamWrite {
    /// The pool, unreachable from any handler by type.
    pool: MySqlPool,
}

impl StoreIpamWrite {
    /// Wire the plan's gestures to a pool.
    pub(crate) fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }
}

impl IpamWritePort for StoreIpamWrite {
    fn define_subnet(
        &self,
        subnet: Subnet,
        label: String,
        vlan: u16,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
        Box::pin(async move {
            // 🔑 THE SERVER MINTS THE ID, v7 (Guy, 2026-09-12) — `document.rs:120`'s half of the
            // same decision. The other half, refusing an ARRIVING nil, belongs to the two routes
            // that receive a `subnet_id` from the browser; this one receives no id at all.
            let id = uuid::Uuid::now_v7().to_string();
            let mut conn = self.pool.acquire().await.map_err(crate::repo::classify)?;
            // 🔴 A TRANSACTION FOR ONE STATEMENT, and the review is why. This read *"one statement,
            // so no transaction … wrapping it would suggest a guarantee it does not need"* — and the
            // guarantee it needed was the budget's: in autocommit a statement waiting on the unique
            // key when the budget drops its future COMMITS once the key is free, under a sentence
            // telling the operator nothing was written. Inside a transaction the drop rolls it back.
            let mut tx = sqlx::Connection::begin(&mut *conn)
                .await
                .map_err(crate::repo::classify)?;
            let written = ipam_repo::insert_subnet(&mut *tx, &id, subnet, &label, vlan).await;
            settle(tx, written).await?;
            // A subnet IS the plan the operator returns to; there is no parent above it.
            Ok(Written::subnet(id))
        })
    }

    fn define_range(
        &self,
        subnet_id: String,
        first: Ipv4Addr,
        last: Ipv4Addr,
        policy: opencmdb_core::ipam::IpPolicy,
        label: String,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
        Box::pin(async move {
            let id = uuid::Uuid::now_v7().to_string();
            let mut conn = self.pool.acquire().await.map_err(crate::repo::classify)?;
            // ⚠️ THE TRANSACTION AND THE TWO LOCKS ARE `insert_range`'s OWN, deliberately: the
            // overlap rule is decided by a read, so the lock has to be taken by whatever performs
            // that read — and so is the one replay of a deadlock victim. Opening a transaction here
            // as well would turn `insert_range`'s into a sqlx SAVEPOINT, whose `commit` releases
            // the savepoint and commits nothing, and whose replay would run inside a transaction
            // InnoDB had already rolled back.
            ipam_repo::insert_range(&mut conn, &id, &subnet_id, first, last, policy, &label)
                .await?;
            // 🔑 The form's `subnet_id`, and it is VERIFIED — by the foreign key the insert writes
            // through: a `subnet_id` naming no row produces no record at all, so a written range
            // proves its parent. That is what the four record-addressed routes had no equivalent of.
            Ok(Written::inside(id, subnet_id))
        })
    }

    fn define_address(
        &self,
        subnet_id: String,
        addr: Ipv4Addr,
        label: String,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
        Box::pin(async move {
            let id = uuid::Uuid::now_v7().to_string();
            let mut conn = self.pool.acquire().await.map_err(crate::repo::classify)?;
            // 🔑 NO LOCK HERE, and the asymmetry with the range is not an oversight: an address is
            // refused twice by `ip_address_in_subnet`, a UNIQUE key, which needs no read to decide.
            // The range's rule compares a row against its SIBLINGS, which a `CHECK` cannot express
            // (`ERROR 1901`) and only a read can answer — *the lock is owed by the rule that reads,
            // not by the act of writing*.
            // 🔴 **But a TRANSACTION, which is not a lock**: measured at the review, an address
            // whose foreign-key check waited on a held parent row was written after the budget's
            // 503. Dropped inside a transaction, it is rolled back instead.
            let mut tx = sqlx::Connection::begin(&mut *conn)
                .await
                .map_err(crate::repo::classify)?;
            let written = ipam_repo::insert_address(&mut tx, &id, &subnet_id, addr, &label).await;
            settle(tx, written).await?;
            // The form's `subnet_id`, verified by the foreign key — `define_range`'s reason.
            Ok(Written::inside(id, subnet_id))
        })
    }

    fn delete_subnet(&self, id: String) -> BoxFuture<'_, Result<Written, RepositoryError>> {
        Box::pin(async move {
            let mut conn = self.pool.acquire().await.map_err(crate::repo::classify)?;
            // A TRANSACTION FOR ONE STATEMENT, on `define_subnet`'s measured precedent: the adapter
            // issues a single capped `DELETE` and owns no transaction, so in autocommit a statement
            // still waiting on a lock when the budget drops its future would COMMIT once the lock
            // came free — under a sentence telling the operator nothing was removed.
            let mut tx = sqlx::Connection::begin(&mut *conn)
                .await
                .map_err(crate::repo::classify)?;
            let written = ipam_repo::delete_subnet(&mut tx, &id).await;
            settle(tx, written).await?;
            // A removed subnet is no plan to return to; `answer` sends the operator to `/ipam`.
            Ok(Written::subnet(id))
        })
    }

    fn delete_range(&self, id: String) -> BoxFuture<'_, Result<Written, RepositoryError>> {
        Box::pin(async move {
            let mut conn = self.pool.acquire().await.map_err(crate::repo::classify)?;
            // ⚠️ NO TRANSACTION HERE, and the asymmetry with the subnet above is deliberate:
            // `delete_range` owns its own, because its refusal is DECIDED by a read and the lock has
            // to be held by whatever performs that read. Opening one here would turn the adapter's
            // into a sqlx SAVEPOINT, whose `commit` releases the savepoint and commits nothing.
            let subnet_id = ipam_repo::delete_range(&mut conn, &id).await?;
            Ok(Written::inside(id, subnet_id))
        })
    }

    fn delete_address(&self, id: String) -> BoxFuture<'_, Result<Written, RepositoryError>> {
        Box::pin(async move {
            let mut conn = self.pool.acquire().await.map_err(crate::repo::classify)?;
            // One statement, no read to decide anything — so the budget's transaction, as above.
            let mut tx = sqlx::Connection::begin(&mut *conn)
                .await
                .map_err(crate::repo::classify)?;
            let written = ipam_repo::delete_address(&mut tx, &id).await;
            let subnet_id = settle(tx, written).await?;
            Ok(Written::inside(id, subnet_id))
        })
    }

    fn edit_range(
        &self,
        id: String,
        first: Ipv4Addr,
        last: Ipv4Addr,
        policy: opencmdb_core::ipam::IpPolicy,
        label: String,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
        Box::pin(async move {
            let mut conn = self.pool.acquire().await.map_err(crate::repo::classify)?;
            // The adapter's own transaction, for `delete_range`'s reason: its sibling scan decides.
            let subnet_id =
                ipam_repo::update_range(&mut conn, &id, first, last, policy, &label).await?;
            Ok(Written::inside(id, subnet_id))
        })
    }

    fn edit_address(
        &self,
        id: String,
        addr: Ipv4Addr,
        label: String,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
        Box::pin(async move {
            let mut conn = self.pool.acquire().await.map_err(crate::repo::classify)?;
            // Likewise: `update_address` re-validates containment under the parent row's lock.
            let subnet_id = ipam_repo::update_address(&mut conn, &id, addr, &label).await?;
            Ok(Written::inside(id, subnet_id))
        })
    }

    fn edit_subnet(
        &self,
        id: String,
        label: String,
        vlan: u16,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
        Box::pin(async move {
            let mut conn = self.pool.acquire().await.map_err(crate::repo::classify)?;
            // The budget's transaction, for `delete_address`'s reason: the locked read and the
            // write must commit or roll back together.
            let mut tx = sqlx::Connection::begin(&mut *conn)
                .await
                .map_err(crate::repo::classify)?;
            let written = ipam_repo::update_subnet(&mut tx, &id, &label, vlan).await;
            settle(tx, written).await?;
            // A subnet IS the plan the operator returns to.
            Ok(Written::subnet(id))
        })
    }

    fn release_address(
        &self,
        addr: Ipv4Addr,
        until: opencmdb_core::observation::Timestamp,
    ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
        Box::pin(async move {
            // 🔑 **No clock**: the instant is the one the operator was SHOWN ([`ReleaseRequest`]).
            let mut conn = self.pool.acquire().await.map_err(crate::repo::classify)?;
            // The defined-address read and the write in one transaction, for `delete_address`'s
            // reason; the race the read leaves is harmless and `release_address`'s doc says why.
            let mut tx = sqlx::Connection::begin(&mut *conn)
                .await
                .map_err(crate::repo::classify)?;
            let written = ipam_repo::release_address(&mut tx, addr, until).await;
            let subnet_id = settle(tx, written).await?;
            Ok(Written {
                id: addr.to_string(),
                subnet_id,
            })
        })
    }
}

/// Commit a write that went through, and roll back one that was refused — explicitly.
///
/// ⚠️ **Defensive, and the second review measured how far.** A dropped sqlx transaction QUEUES its
/// rollback on the connection, and sqlx 0.9 pings a connection on its return to the pool, which
/// flushes it (`pool/connection.rs`, *"flush … transaction rollbacks"*). On this handler path the
/// connection is returned at once, so replacing the rollback below with `drop(tx)` left every test
/// green: it is carried by no test, and that is said rather than implied. It is kept so releasing the
/// locks does not depend on a pool's return path. (This doc read *"would keep its locks until the
/// connection was next used … measured"*, true of the race test's HELD connections and presented as
/// true of production.)
async fn settle<T>(
    tx: sqlx::Transaction<'_, sqlx::MySql>,
    written: Result<T, RepositoryError>,
) -> Result<T, RepositoryError> {
    match written {
        Ok(value) => tx
            .commit()
            .await
            .map_err(crate::repo::classify)
            .map(|()| value),
        Err(refused) => {
            if let Err(error) = tx.rollback().await {
                tracing::warn!(%error, "rolling back a refused plan write failed");
            }
            Err(refused)
        }
    }
}

/// The sub-router's state — deliberately NO pool field. See the module doc.
#[derive(Clone)]
pub(crate) struct IpamWriteState {
    /// The plan's gestures. A port, never a pool.
    port: Arc<dyn IpamWritePort>,
}

/// The production sub-router, wired to the store-backed port.
pub(crate) fn router(pool: MySqlPool) -> Router {
    router_with(Arc::new(StoreIpamWrite::new(pool)))
}

/// The sub-router over an explicit port — the seam tests use to drive a gesture without a
/// database.
pub(crate) fn router_with(port: Arc<dyn IpamWritePort>) -> Router {
    let mut router = Router::new();
    // Built from the variants, because only a variant carries its handler — and the list AC3's
    // guard walks is DERIVED from the same variants (`WriteRoute::paths`), so there is no second
    // list to pin.
    for route in WriteRoute::ALL {
        router = router.route(route.path(), route.handler());
    }
    router.with_state(IpamWriteState { port })
}

/// `POST /ipam/subnet` — define a subnet.
///
/// The order of the refusals is the order of the decisions, and the first one is the CSRF check
/// (story 6.2 §5): the 403 wins over every other refusal. ⚠️ It cannot literally run *before* the
/// parser — `form` is an axum extractor, so the body is parsed before this body executes — and what
/// holds is that no refusal path consults the parsed form before the Origin has been compared.
async fn define_subnet(
    State(state): State<IpamWriteState>,
    headers: HeaderMap,
    form: Result<Form<DefineSubnetRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return cross_origin().into_response();
    }
    let Ok(Form(request)) = form else {
        return WriteRoute::Subnet.malformed().into_response();
    };
    // 🔑 THE CIDR IS READ BEFORE THE LABEL, and the order is a decision rather than an accident.
    // A form wrong in two places can only be told about one of them, and the honest one to name is
    // the FIRST field the operator filled — the form is `cidr` then `label`, and a sentence about
    // the second while the first is unusable reads as though the first had been accepted.
    // ⚠️ T2 shipped the opposite order with nothing asserting either, which is what made this a
    // decision to take rather than a preference to keep.
    let subnet = match parse_cidr(&request.cidr) {
        Ok(subnet) => subnet,
        Err(refusal) => return refusal.into_response(),
    };
    let label = match checked_label(&request.label) {
        Ok(label) => label,
        Err(refusal) => return refusal.into_response(),
    };
    let vlan = match parse_vlan(&request.vlan, WriteRoute::Subnet) {
        Ok(vlan) => vlan,
        Err(refusal) => return refusal.into_response(),
    };
    let work = state.port.define_subnet(subnet, label, vlan);
    answer(
        within_budget(work).await,
        WriteRoute::Subnet,
        "ipam.done.subnet",
    )
}

/// `POST /ipam/range` — define a stretch of a subnet and say what it is MEANT for.
///
/// 🔑 **It reuses, it does not re-implement.** The Origin check, the label rules, the subnet-id
/// refusal, the exhaustive `RepositoryError` mapping and the budget are the same functions the
/// subnet route calls, and `every_route_reuses_the_shared_machinery` drives all three through the
/// same probes rather than trusting this sentence.
async fn define_range(
    State(state): State<IpamWriteState>,
    headers: HeaderMap,
    form: Result<Form<DefineRangeRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return cross_origin().into_response();
    }
    let route = WriteRoute::Range;
    let Ok(Form(request)) = form else {
        return route.malformed().into_response();
    };
    let subnet_id = match checked_subnet_id(&request.subnet_id, route) {
        Ok(id) => id,
        Err(refusal) => return refusal.into_response(),
    };
    let (first, last) = match (
        request.first.trim().parse::<Ipv4Addr>(),
        request.last.trim().parse::<Ipv4Addr>(),
    ) {
        (Ok(first), Ok(last)) => (first, last),
        _ => return route.malformed().into_response(),
    };
    let policy = match parse_policy(&request.policy) {
        Ok(policy) => policy,
        Err(refusal) => return refusal.into_response(),
    };
    let label = match checked_label(&request.label) {
        Ok(label) => label,
        Err(refusal) => return refusal.into_response(),
    };
    // 🔑 No `clone`: the port takes the id and hands it back inside [`Written`], so the redirect
    // target is what the store answered rather than what the handler still happened to hold.
    let work = state
        .port
        .define_range(subnet_id, first, last, policy, label);
    answer(within_budget(work).await, route, "ipam.done.range")
}

/// `POST /ipam/address` — define one address of a subnet.
async fn define_address(
    State(state): State<IpamWriteState>,
    headers: HeaderMap,
    form: Result<Form<DefineAddressRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return cross_origin().into_response();
    }
    let route = WriteRoute::Address;
    let Ok(Form(request)) = form else {
        return route.malformed().into_response();
    };
    let subnet_id = match checked_subnet_id(&request.subnet_id, route) {
        Ok(id) => id,
        Err(refusal) => return refusal.into_response(),
    };
    let Ok(addr) = request.addr.trim().parse::<Ipv4Addr>() else {
        return route.malformed().into_response();
    };
    let label = match checked_label(&request.label) {
        Ok(label) => label,
        Err(refusal) => return refusal.into_response(),
    };
    let work = state.port.define_address(subnet_id, addr, label);
    answer(within_budget(work).await, route, "ipam.done.address")
}

/// The id of the record a correction names, or the refusal it earns.
///
/// 🔑 **The same rule as [`checked_subnet_id`], under a name that says what it reads.** Every id in
/// the plan is a v7 UUID minted by the server, so the parse, the nil refusal and the canonical
/// re-serialisation are one rule — but a function called *subnet id* reading a RANGE's id is the
/// kind of true-when-written, false-later name this project keeps finding in its own reviews.
///
/// # Errors
///
/// The form's shape refusal when the id is not a UUID, or is the nil sentinel.
fn checked_record_id(raw: &str, route: WriteRoute) -> Result<String, Refusal> {
    let Ok(parsed) = raw.trim().parse::<uuid::Uuid>() else {
        return Err(route.malformed());
    };
    if parsed.is_nil() {
        return Err(route.malformed());
    }
    Ok(parsed.to_string())
}

/// `POST /ipam/subnet/delete` — remove a subnet.
///
/// ⚠️ **The refusal an operator is likeliest to meet here is the DATABASE's**, not this product's:
/// two foreign keys point at `ip_subnet`, so a subnet still holding ranges or addresses raises
/// `ERROR 1451` — which [`constraint_refusal`] must answer by NAME rather than as *no such subnet*.
async fn delete_subnet(
    State(state): State<IpamWriteState>,
    headers: HeaderMap,
    form: Result<Form<DeleteSubnetRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return cross_origin().into_response();
    }
    let route = WriteRoute::DeleteSubnet;
    let Ok(Form(request)) = form else {
        return route.malformed().into_response();
    };
    let id = match checked_record_id(&request.id, route) {
        Ok(id) => id,
        Err(refusal) => return refusal.into_response(),
    };
    let work = state.port.delete_subnet(id);
    answer(within_budget(work).await, route, "ipam.done.delete_subnet")
}

/// `POST /ipam/range/delete` — remove a range.
async fn delete_range(
    State(state): State<IpamWriteState>,
    headers: HeaderMap,
    form: Result<Form<DeleteRangeRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return cross_origin().into_response();
    }
    let route = WriteRoute::DeleteRange;
    let Ok(Form(request)) = form else {
        return route.malformed().into_response();
    };
    let id = match checked_record_id(&request.id, route) {
        Ok(id) => id,
        Err(refusal) => return refusal.into_response(),
    };
    let work = state.port.delete_range(id);
    answer(within_budget(work).await, route, "ipam.done.delete_range")
}

/// `POST /ipam/address/delete` — remove one defined address.
async fn delete_address(
    State(state): State<IpamWriteState>,
    headers: HeaderMap,
    form: Result<Form<DeleteAddressRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return cross_origin().into_response();
    }
    let route = WriteRoute::DeleteAddress;
    let Ok(Form(request)) = form else {
        return route.malformed().into_response();
    };
    let id = match checked_record_id(&request.id, route) {
        Ok(id) => id,
        Err(refusal) => return refusal.into_response(),
    };
    let work = state.port.delete_address(id);
    answer(within_budget(work).await, route, "ipam.done.delete_address")
}

/// `POST /ipam/subnet/edit` — correct a subnet's label and its VLAN (story 14.5).
async fn edit_subnet(
    State(state): State<IpamWriteState>,
    headers: HeaderMap,
    form: Result<Form<EditSubnetRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return cross_origin().into_response();
    }
    let route = WriteRoute::EditSubnet;
    let Ok(Form(request)) = form else {
        return route.malformed().into_response();
    };
    let id = match checked_record_id(&request.id, route) {
        Ok(id) => id,
        Err(refusal) => return refusal.into_response(),
    };
    let label = match checked_label(&request.label) {
        Ok(label) => label,
        Err(refusal) => return refusal.into_response(),
    };
    let vlan = match parse_vlan(&request.vlan, route) {
        Ok(vlan) => vlan,
        Err(refusal) => return refusal.into_response(),
    };
    let work = state.port.edit_subnet(id, label, vlan);
    answer(within_budget(work).await, route, "ipam.done.edit_subnet")
}

/// `POST /ipam/release` — stop the plan holding an address (story 14.4b).
///
/// 🔑 **The address is parsed and nothing else is asked of it here**: whether the plan DEFINES it is
/// the adapter's refusal (decision 4), and whether the network was ever seen on it is a question this
/// module may not ask — the plan's guard keeps the sightings behind `ipam_audit.rs`.
async fn release_address(
    State(state): State<IpamWriteState>,
    headers: HeaderMap,
    form: Result<Form<ReleaseRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return cross_origin().into_response();
    }
    let route = WriteRoute::Release;
    let Ok(Form(request)) = form else {
        return route.malformed().into_response();
    };
    let Ok(addr) = request.addr.trim().parse::<Ipv4Addr>() else {
        return route.malformed().into_response();
    };
    let Ok(until) = chrono::DateTime::parse_from_rfc3339(request.seen_until.trim()) else {
        return route.malformed().into_response();
    };
    let work = state
        .port
        .release_address(addr, until.with_timezone(&chrono::Utc));
    answer(within_budget(work).await, route, "ipam.done.release")
}

/// `POST /ipam/range/edit` — correct a range's bounds, its policy or its label.
///
/// 🔑 **Among the fields the OPERATOR filled, the order is the definition route's**: the bounds are
/// read before the policy and the policy before the label, because a form wrong in two places can
/// only be told about one of them and the honest one to name is the first they filled.
///
/// ⚠️ **The record's own id is read before all of them, and it is not one of those fields** — the
/// review found this sentence claiming the rule for the whole routine when it governs only its
/// operator-visible tail. A mangled hidden `id` is a form that is not the product's, and it earns
/// the shape refusal before any typed value is looked at.
async fn edit_range(
    State(state): State<IpamWriteState>,
    headers: HeaderMap,
    form: Result<Form<EditRangeRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return cross_origin().into_response();
    }
    let route = WriteRoute::EditRange;
    let Ok(Form(request)) = form else {
        return route.malformed().into_response();
    };
    let id = match checked_record_id(&request.id, route) {
        Ok(id) => id,
        Err(refusal) => return refusal.into_response(),
    };
    let (first, last) = match (
        request.first.trim().parse::<Ipv4Addr>(),
        request.last.trim().parse::<Ipv4Addr>(),
    ) {
        (Ok(first), Ok(last)) => (first, last),
        _ => return route.malformed().into_response(),
    };
    let policy = match parse_policy(&request.policy) {
        Ok(policy) => policy,
        Err(refusal) => return refusal.into_response(),
    };
    let label = match checked_label(&request.label) {
        Ok(label) => label,
        Err(refusal) => return refusal.into_response(),
    };
    let work = state.port.edit_range(id, first, last, policy, label);
    answer(within_budget(work).await, route, "ipam.done.edit_range")
}

/// `POST /ipam/address/edit` — correct one defined address, or what the operator calls it.
async fn edit_address(
    State(state): State<IpamWriteState>,
    headers: HeaderMap,
    form: Result<Form<EditAddressRequest>, FormRejection>,
) -> Response {
    if !crate::write_guard::same_origin(&headers) {
        return cross_origin().into_response();
    }
    let route = WriteRoute::EditAddress;
    let Ok(Form(request)) = form else {
        return route.malformed().into_response();
    };
    let id = match checked_record_id(&request.id, route) {
        Ok(id) => id,
        Err(refusal) => return refusal.into_response(),
    };
    let Ok(addr) = request.addr.trim().parse::<Ipv4Addr>() else {
        return route.malformed().into_response();
    };
    let label = match checked_label(&request.label) {
        Ok(label) => label,
        Err(refusal) => return refusal.into_response(),
    };
    let work = state.port.edit_address(id, addr, label);
    answer(within_budget(work).await, route, "ipam.done.edit_address")
}

/// Turn a port's answer into the operator's, sending the browser back to the plan it just changed.
///
/// 🔑 `HX-Redirect` for the reason story 6.4 adopted it: a narrow swap would leave the screen
/// asserting the state it had before the write. The browser goes back to `/ipam` selected on the
/// subnet concerned, which re-renders from the store — *one URL per state*, epic constraint 4.
fn answer(
    outcome: Result<Written, RepositoryError>,
    route: WriteRoute,
    done_key: &'static str,
) -> Response {
    match outcome {
        Ok(Written { id, subnet_id }) => {
            // 🔑 `None` means the record the gesture touched IS a subnet — `define_subnet` mints one
            // and `delete_subnet` removes one — so a defined subnet is its own redirect target.
            // ⚠️ **The target is no longer anything the browser can choose** (decision 1): the four
            // corrections read it off the record, and the two definitions inside a subnet carry a
            // `subnet_id` the foreign key vouched for by writing through it.
            // 🔴 **A RELEASE WITH NO CONTAINING SUBNET has no plan to return to**, and its `id` is an
            // ADDRESS — `/ipam?subnet=192.0.2.9` would render the unknown-subnet branch.
            let target = match (&subnet_id, route) {
                (Some(subnet), _) => Some(subnet.clone()),
                (None, WriteRoute::Release) => None,
                (None, _) => Some(id.clone()),
            };
            // 🔴 **A REMOVED SUBNET CANNOT BE REDIRECTED TO.** Every other gesture leaves the
            // operator looking at the plan it changed; this one leaves them looking at a plan that
            // no longer exists, and `/ipam?subnet=<the id just deleted>` would render the
            // unknown-subnet branch — the product telling them their subnet is missing one instant
            // after removing it at their request. The plan's own address is the honest target.
            let destination = match (&target, route) {
                (_, WriteRoute::DeleteSubnet) | (None, _) => "/ipam".to_string(),
                (Some(target), _) => format!("/ipam?subnet={target}"),
            };
            // 🔑 **The release's confirmation RIDES IN THE URL** (Guy, code review 2026-09-19, on
            // `/triage`'s `?documented=` precedent): `HX-Redirect` makes htmx navigate, so the body
            // below is never shown — and for the release it is the one sentence that says the
            // release LAPSES when the network answers again. `id` is the address the port parsed.
            let destination = if matches!(route, WriteRoute::Release) {
                let join = if destination.contains('?') { '&' } else { '?' };
                format!("{destination}{join}released={id}")
            } else {
                destination
            };
            tracing::info!(id = %id, subnet = ?target, key = done_key, "the plan was changed");
            (
                // 201 for the three definitions, 200 for the five corrections — a deletion that
                // answered *Created* would be a false statement in the protocol.
                route.success_status(),
                [(
                    axum::http::HeaderName::from_static("hx-redirect"),
                    destination,
                )],
                rust_i18n::t!(done_key).to_string(),
            )
                .into_response()
        }
        Err(error) => {
            let refusal = repository_refusal(&error, route);
            if refusal.status() == StatusCode::INTERNAL_SERVER_ERROR {
                tracing::error!(%error, "the addressing plan's write failed at the backend");
            }
            refusal.into_response()
        }
    }
}

/// The subnet id the form carried, or the refusal it earns.
///
/// 🔑 **THE NIL UUID IS REFUSED HERE, AT THE ROUTE** (Guy, 2026-09-12, §2's decision 3) — the other
/// half of `document.rs`'s pair, whose `:120` mints a v7 for the record being created and whose
/// `:195` refuses a nil ARRIVING from the client. Both halves are live on this route: it creates a
/// range (id minted here) and it carries a subnet id chosen in the browser.
///
/// ⚠️ **A TRIPWIRE, not a barrier**: the adapter still accepts the nil, measured at story 14.2 on
/// all three `insert_*`. Story 5.12's precedent, stated rather than implied.
///
/// ⚠️ The nil is folded into the form's shape sentence rather than given a key of its own
/// (`document.rs:195`'s precedent): it is a sentinel no operator types, so a sentence for it would
/// be a sentence spent on a hand-crafted request.
///
/// # Errors
///
/// The form's shape refusal when the id is not a UUID, or is the nil sentinel.
fn checked_subnet_id(raw: &str, route: WriteRoute) -> Result<String, Refusal> {
    let Ok(parsed) = raw.trim().parse::<uuid::Uuid>() else {
        return Err(route.malformed());
    };
    if parsed.is_nil() {
        return Err(route.malformed());
    }
    // Re-serialised canonical, so a braced, urn: or hyphenless spelling of a real id is harmless
    // before any SQL sees it (story 6.2 §2).
    Ok(parsed.to_string())
}

/// The VLAN the operator declared, or the refusal that says it is not one (story 14.5).
///
/// 🔑 **Empty means NONE, and none is the sentinel `0`** — the store's shape (`0010`), chosen after the
/// NULLable form was measured letting one CIDR in three times. An operator who leaves the field blank
/// is saying *this subnet is not on a VLAN*, which is a statement the plan can hold.
///
/// 🔴 **0 typed BY HAND is refused**, and that is not pedantry: 802.1Q reserves VID 0 for
/// *priority-tagged, no VLAN*, so an operator typing it means *none* — and the product must not let two
/// different gestures (typing 0, leaving it blank) reach the store as the same row through a path that
/// reads as if 0 were an ordinary VLAN. The refusal names the range, which is where they learn it.
///
/// ⚠️ **The whitespace is trimmed for the PARSE as well, and this sentence said the opposite until a
/// test of this story's own measured it** (`vlan=12 ` answered 200 under a criterion predicting 422).
/// Kept as it is rather than tightened: `parse_policy` refuses a padded token because a BINDING WORD
/// compared loosely is a word with two spellings, where a number wrapped in whitespace by a copy or by
/// a keyboard is a number the operator means. The false sentence was the defect; the behaviour is the
/// decision, and it is now measured on both sides.
///
/// # Errors
///
/// The form's 422 when the value is not a whole number in `1..=4094`.
fn parse_vlan(raw: &str, route: WriteRoute) -> Result<u16, Refusal> {
    // ⚠️ **The route is carried and answers nothing yet, and the second form ARRIVED IN THIS STORY
    // and chose the same sentence.** `unknown_record`'s idiom keeps the parameter so a third form
    // can earn its own refusal without every caller changing; what it buys today is measured at
    // zero, and saying so beats a comment predicting a split that already did not happen.
    let _ = route;
    let typed = raw.trim();
    if typed.is_empty() {
        return Ok(0);
    }
    match typed.parse::<u16>() {
        Ok(vlan) if (1..=4094).contains(&vlan) => Ok(vlan),
        _ => Err(Refusal::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "ipam.refusal.not_a_vlan",
        )),
    }
}

/// The policy the operator chose, or the refusal that says it is not one of the four.
///
/// 🔴 **NO `trim()`, and the first draft had one — its own test caught it.** A label is free text
/// an operator typed, where a stray space is a typo worth absorbing; a policy is a TOKEN from a
/// closed set, chosen by a control, where a stray space means the sender is not the form. Trimming
/// it would make the route's acceptance set larger than the binding table's, which is the one
/// property that table exists to hold — and `ascii_bin` being PAD SPACE, `'static '` is exactly the
/// value the schema needed an INTEGER comparison to refuse one layer down. *The same reflex that is
/// kindness on free text is a widened vocabulary on a token.*
///
/// ⚠️ **Not `ipam_repo::policy_from_token`, and the difference is the POPULATION.** That one reads a
/// token the STORE holds, so an unknown one is a row this build cannot render — a fault, answered
/// as a backend failure. This one reads a token the BROWSER sent, where an unknown one is an
/// ordinary form mistake. *The same string means different things depending on who wrote it.*
///
/// # Errors
///
/// A 422 naming the axis rather than paraphrasing the four binding words.
fn parse_policy(raw: &str) -> Result<opencmdb_core::ipam::IpPolicy, Refusal> {
    opencmdb_core::ipam::IpPolicy::ALL
        .into_iter()
        .find(|policy| policy.as_str() == raw)
        .ok_or_else(|| {
            Refusal::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "ipam.refusal.unknown_policy",
            )
        })
}

/// The label the operator typed, or the refusal that names which rule it broke.
///
/// # Errors
///
/// A 422 keyed on the rule: empty (§2's decision 4 — a plan of numbers with no words is not a
/// plan), carrying a control character, or longer than [`MAX_LABEL_CHARS`].
fn checked_label(raw: &str) -> Result<String, Refusal> {
    let label = raw.trim();
    // 🔴 NOTHING VISIBLE IS EMPTY, and `trim` alone let it through: the review posted
    // `%E2%80%8B%E2%80%8B`, got 201, and `/ipam` rendered a subnet with no name —
    // `"\u{200B}".trim().is_empty()` is `false` in Rust. The crate's own predicate, story 6b.2's.
    if !crate::carries_a_visible_glyph(label) {
        return Err(Refusal::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "ipam.refusal.label_empty",
        ));
    }
    // 🔴 A CONTROL CHARACTER IS REFUSED, NOT STORED: `%0A%0Dctrl%00` answered 201 and every `/ipam`
    // page then served a NUL byte, which `grep` took for a binary file. A label is one line of text.
    if label
        .chars()
        .any(|glyph| glyph.is_control() || breaks_the_line_or_its_direction(glyph))
    {
        return Err(Refusal::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "ipam.refusal.label_control",
        ));
    }
    if label.chars().count() > MAX_LABEL_CHARS {
        return Err(Refusal::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "ipam.refusal.label_too_long",
        ));
    }
    Ok(label.to_string())
}

/// A character that ends a line or reorders the text around it — refused in a label, which is one
/// line of plain text.
///
/// 🔴 `char::is_control` is category Cc alone, and the second review measured U+2028, U+2029 and
/// U+202E accepted and rendered. Directional MARKS (U+200E, U+200F) stay allowed — a label in Hebrew
/// or Arabic may legitimately carry one — while embeddings, overrides and isolates, which reorder
/// what surrounds them, do not.
const fn breaks_the_line_or_its_direction(glyph: char) -> bool {
    matches!(
        glyph,
        '\u{2028}' | '\u{2029}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
    )
}

/// Parse `192.0.2.0/24` into the subnet the arithmetic can answer for.
///
/// # Errors
///
/// A 422: the subnet form's own shape sentence when the text is not a CIDR at all, or the keyed
/// sentence of whichever [`IpamError`] [`Subnet::new`] raises.
fn parse_cidr(raw: &str) -> Result<Subnet, Refusal> {
    let malformed = || WriteRoute::Subnet.malformed();
    let (base, prefix) = raw.trim().split_once('/').ok_or_else(malformed)?;
    let base: Ipv4Addr = base.parse().map_err(|_| malformed())?;
    // ⚠️ A prefix that does not fit a `u8` — `/999` — is MALFORMED, while one that fits and cannot
    // belong to IPv4 — `/64` — gets `PrefixLengthNotInFamily`, which names the real rule. Two
    // sentences for what looks like one mistake, and the distinction is the honest one: the product
    // can only name a rule it can evaluate.
    let prefix: u8 = prefix.parse().map_err(|_| malformed())?;
    // The subnet route is the only caller of this parser, and it is the gesture whose sentence this
    // refusal will carry.
    Subnet::new(base, prefix).map_err(|error| ipam_refusal(&error, WriteRoute::Subnet))
}

/// The sentence a named database constraint earns, or `None` for a name nobody has mapped.
///
/// 🔑 **The `Option` is the whole point, and it exists because an explicit 500 and a fallthrough
/// 500 are the same ANSWER and not the same STATEMENT.** [`RepositoryError::Constraint`] carries a
/// `&'static str`, so no `match` over it can be exhaustive and the compiler cannot help; what
/// carries AC4's second half is `every_constraint_name_the_store_can_produce_is_mapped`, which
/// reads the names `repo::classify` can emit and asserts each one is `Some` here. Fold `check` into
/// the `_` arm and the answer to the operator does not change by one byte — and that test reds,
/// which is the difference between a decision and an omission.
fn constraint_refusal(name: &str, route: WriteRoute) -> Option<Refusal> {
    match name {
        // The likeliest refusal on this screen: the operator re-enters what the plan already holds.
        // It rides a UNIQUE key, and NO pre-read is taken — a check that commits separately from its
        // write is a TOCTOU hole, not a check. 🔴 The ROUTE names the record: the name carries no
        // index, and one sentence for all three told a re-entered address it was a subnet.
        // ⚠️ **A DELETE CANNOT RAISE IT AT ALL** — a statement that writes no row hits no unique
        // key — so the three deletes are not given a sentence explaining a re-entered record.
        // Reaching here from one would mean a unique violation on a `DELETE`, which is a fault in
        // this product rather than something an operator can correct: `check`'s own reasoning a few
        // lines down, met on a second constraint name. Enumerated rather than left to a `_`, for the
        // reason this file gives elsewhere — the `_` is what swallows a variant added tomorrow.
        "unique" => Some(route.already_defined()),
        // 🔴 **THE SAME CONSTRAINT NAME MEANS TWO OPPOSITE THINGS, and one sentence served both.**
        // On a DEFINITION, `1451` is a `subnet_id` naming no row: the parent is missing. On a
        // SUBNET DELETE it is the exact reverse — the parent is right there and its CHILDREN are
        // what refuse to let it go. Until this story both answered *"that subnet is not in the
        // plan"*, so an operator removing a populated subnet was told it did not exist while
        // looking at it. The route is what knows which direction the key was pointing.
        // ⚠️ **Enumerated, where the review found a `_`** — three lines below the `unique` arm whose
        // own comment forbids one, and in the arm this story exists to split. A compile probe put
        // the cost in figures: a ninth variant produces seven `error[E0004]`s and **this arm was in
        // none of them**, so a route added tomorrow would silently inherit *that subnet is not in
        // the plan* — the wrong-record defect, one arm below its fix. 🔑 The four record-addressed
        // routes cannot reach it today (a child `DELETE` violates no key, and neither `UPDATE`
        // touches `subnet_id`), so what was live was the hole and not the sentence; it is named
        // rather than left for the next route to discover.
        "foreign_key" => Some(match route {
            // The parent is right there and its CHILDREN refuse to let it go.
            WriteRoute::DeleteSubnet => {
                Refusal::new(StatusCode::CONFLICT, "ipam.refusal.subnet_still_holds")
            }
            // A definition whose `subnet_id` names no row: the parent is missing.
            WriteRoute::Range | WriteRoute::Address => {
                Refusal::new(StatusCode::NOT_FOUND, "ipam.refusal.unknown_subnet")
            }
            // A subnet definition writes no foreign key, and the four corrections address a record
            // by its own id without touching its parent — so a key violation from any of these is
            // ours, not the operator's.
            // ⚠️ `address_release` carries no foreign key at all (`0009`'s header), so a release
            // reaching here is ours as well.
            WriteRoute::Subnet
            | WriteRoute::DeleteRange
            | WriteRoute::DeleteAddress
            | WriteRoute::EditRange
            | WriteRoute::EditAddress
            | WriteRoute::EditSubnet
            | WriteRoute::Release => backend(),
        }),
        // 🔑 OURS, not the operator's, and mapped EXPLICITLY for that reason. Every `CHECK` on the
        // plan's three tables restates something the adapter already refuses — the canonical
        // spelling, the policy domain, the ordered bounds — so a `check` reaching here means a
        // value went round the adapter. That is a fault in this product, not a mistake the operator
        // can be told how to correct.
        "check" => Some(backend()),
        _ => None,
    }
}

/// Map a [`RepositoryError`] to the operator's sentence.
///
/// 🔑 **Exhaustive, with no `_` arm, and that is half of AC4's carrier**: a new VARIANT is an
/// `error[E0004]` here. ⚠️ The other half cannot be the compiler — [`RepositoryError::Constraint`]
/// carries a `&'static str`, which forces a `_` inside, so a new constraint NAME is invisible to
/// it. That half is a SET test over the names the adapter can produce.
pub(crate) fn repository_refusal(error: &RepositoryError, route: WriteRoute) -> Refusal {
    match error {
        RepositoryError::Ipam(ipam) => ipam_refusal(ipam, route),
        // The three names `classify` can produce. ⚠️ The `_` is what the SET test covers.
        // ⚠️ THE COMPILER STOPS HERE. `Constraint` carries a `&'static str`, so the match INSIDE
        // it needs a `_` and a new constraint NAME is invisible to `E0004`. That half is carried by
        // [`constraint_refusal`] answering `None`, which the SET test reads.
        RepositoryError::Constraint(name) => {
            constraint_refusal(name, route).unwrap_or_else(backend)
        }
        // 🔴 Split by route since story 14.4: this answered *"that subnet is not in the plan"* on
        // every route, so correcting a range the plan no longer holds reported the SUBNET missing
        // and sent the operator to look in the wrong place. [`WriteRoute::unknown_record`] names
        // the record the id actually meant.
        RepositoryError::NotFound => route.unknown_record(),
        // 🔴 Reachable since this story: `classify`'s `Contention` arm was dead until T1 repaired
        // it, and the plan's routes are the first writes in this product that CONTEND. 503 rather
        // than 409, because nothing about the request is wrong — the honest answer is *try again*.
        RepositoryError::Contention => {
            Refusal::new(StatusCode::SERVICE_UNAVAILABLE, "ipam.refusal.contention")
        }
        // Neither can arise here: both belong to the identity engine's write path, which this
        // module never enters. They are mapped rather than swept into a `_`, because the `_` is
        // what would swallow a variant added tomorrow.
        RepositoryError::InstantRegressed | RepositoryError::ContradictoryObservation => backend(),
        RepositoryError::Backend(_) => backend(),
    }
}

/// Map an [`IpamError`] to the operator's sentence — exhaustive, no `_` arm.
pub(crate) fn ipam_refusal(error: &IpamError, route: WriteRoute) -> Refusal {
    let (status, key) = match error {
        IpamError::RangeOutsideSubnet => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "ipam.refusal.range_outside_subnet",
        ),
        IpamError::RangeOverlapsAnother => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "ipam.refusal.range_overlaps",
        ),
        IpamError::RangeBoundsInverted => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "ipam.refusal.range_bounds_inverted",
        ),
        IpamError::AddressOutsideSubnet => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "ipam.refusal.address_outside_subnet",
        ),
        IpamError::BaseIsNotTheNetworkAddress => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "ipam.refusal.base_not_network",
        ),
        IpamError::PrefixLengthNotInFamily => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "ipam.refusal.prefix_not_in_family",
        ),
        IpamError::MalformedAddress => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "ipam.refusal.not_an_address",
        ),
        // 🔑 Story 14.4: the one refusal of this plan that is DECIDED rather than raised. A range
        // holds an address by arithmetic — `ip_address` references `ip_subnet` and never
        // `ip_range` — so no `ERROR 1451` answers it and no `CHECK` can express it; it is counted
        // under the parent row's lock and named here.
        //
        // 🔴 **409 AND NOT 422** (Guy's decision 3, 2026-09-16), which ALIGNS it with the subnet
        // delete's own refusal. Every other refusal above is about the CONTENT of the request —
        // outside its subnet, bounds inverted, not an address — and 422 is its status. This one says
        // nothing is wrong with the request: it is the STATE OF THE PLAN that opposes it, which is
        // what 409 means, and `constraint_refusal` already answers 409 when the database refuses a
        // populated subnet. Two refusals that both say *this container is not empty* answered two
        // different statuses, and nobody had decided that — it fell out of which layer raised them.
        //
        // 🔴 **AND THE GESTURE PICKS THE SENTENCE** (Guy's decision 2, 2026-09-16). Since the
        // review's repair, an EDIT that would move a range off an address it holds earns this same
        // rule — so the delete's sentence, which ends *"deleting the range would leave them with
        // nothing saying what that space is meant for"*, was served to an operator who had SHRUNK a
        // range. *A true sentence about the wrong gesture, which is worse than none* — this file's
        // own words, one route over, and the defect this story exists to remove. The route is what
        // knows which gesture asked, exactly as it does in [`WriteRoute::unknown_record`] and in
        // [`constraint_refusal`]; enumerated rather than left to a `_`, for their reason.
        IpamError::RangeStillHoldsAddresses => (
            StatusCode::CONFLICT,
            match route {
                WriteRoute::EditRange => "ipam.refusal.range_edit_would_abandon",
                WriteRoute::Subnet
                | WriteRoute::Range
                | WriteRoute::Address
                | WriteRoute::DeleteSubnet
                | WriteRoute::DeleteRange
                | WriteRoute::DeleteAddress
                | WriteRoute::EditAddress => "ipam.refusal.range_still_holds_addresses",
                // A release and a subnet correction touch no range, so this refusal reaching either
                // is a fault here — not a true sentence about the wrong gesture, which is what the
                // review found mapped.
                WriteRoute::Release | WriteRoute::EditSubnet => return backend(),
            },
        ),
        // 🔑 **409, for decision 3's reason**: nothing is wrong with the REQUEST — it is the STATE
        // OF THE PLAN (the address has a record) that opposes it. The sentence names the gesture
        // that does stop the plan holding a defined address: removing its record.
        IpamError::ReleaseOfADefinedAddress => {
            (StatusCode::CONFLICT, "ipam.refusal.release_of_defined")
        }
    };
    Refusal::new(status, key)
}

/// A backend failure: answered with a sentence that leaks none of its cause. The cause itself is
/// logged by the caller, which is what keeps this mapper pure.
const fn backend() -> Refusal {
    Refusal::new(StatusCode::INTERNAL_SERVER_ERROR, "ipam.refusal.backend")
}

/// The refusal a cross-origin write earns — a KEY, where `document.rs` still answers
/// [`crate::write_guard::CSRF_REFUSED_BODY`]'s English literal.
///
/// 🔴 **The review found that literal on these routes, under AC1's *"a keyed refusal body per
/// status in both locales"***: `hx-on::before-swap` swaps a 4xx into the page, so a French operator
/// behind a proxy that rewrites `Host` read *"cross-origin request refused"* on every write. The
/// documenting route's literal is story 6.1's and is registered rather than changed here.
const fn cross_origin() -> Refusal {
    Refusal::new(StatusCode::FORBIDDEN, "ipam.refusal.cross_origin")
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, header};
    use tower::ServiceExt;

    use super::*;

    /// A port that answers whatever it was built with, and records what it was asked.
    struct FakePort {
        answer: std::sync::Mutex<Option<Result<String, RepositoryError>>>,
        asked: std::sync::Mutex<Vec<(String, String)>>,
    }

    impl FakePort {
        /// The one answer this port was built with. Panics on a second call, which is the point:
        /// a test that reaches the port twice is a test whose refusal did not stop the handler.
        fn take_answer(&self) -> Result<String, RepositoryError> {
            self.answer
                .lock()
                .expect("the fake port's answer")
                .take()
                .expect("the fake port answers once per test")
        }

        fn answering(answer: Result<String, RepositoryError>) -> Arc<Self> {
            Arc::new(Self {
                answer: std::sync::Mutex::new(Some(answer)),
                asked: std::sync::Mutex::new(Vec::new()),
            })
        }
    }

    /// The subnet the fake reports as a record's real parent.
    ///
    /// 🔑 **It comes from the PORT, which is the point of decision 1**: the four record-addressed
    /// routes no longer receive a subnet from the form, so the redirect a test asserts can only have
    /// come from the store's answer. Before the repair the same expectation was satisfied by the
    /// request body echoing itself back.
    const FAKE_PARENT: &str = "01900000-0000-7000-8000-0000000000aa";

    impl IpamWritePort for FakePort {
        fn define_subnet(
            &self,
            subnet: Subnet,
            label: String,
            vlan: u16,
        ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
            // 🔑 **The VLAN is RECORDED here**, so a handler that parses it and forgets to pass it on
            // cannot leave every port assertion green — the story's validation named that shape.
            self.asked
                .lock()
                .expect("the fake port's log")
                .push((format!("{} vlan={vlan}", subnet.cidr()), label));
            let answer = self.take_answer();
            Box::pin(async move { answer.map(Written::subnet) })
        }

        fn edit_subnet(
            &self,
            id: String,
            label: String,
            vlan: u16,
        ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
            // The VLAN is recorded for `define_subnet`'s reason: a handler that parses it and drops
            // it must not leave the port assertions green.
            self.asked
                .lock()
                .expect("the fake port's log")
                .push((format!("{id} vlan={vlan}"), label.clone()));
            let answer = self.take_answer();
            // 🔑 A CORRECTION answers with the record the operator addressed, where a DEFINITION
            // answers with the id it just minted — so the fake echoes the REQUEST's id here and the
            // injected one there. Getting this wrong would have made the redirect assertion pass
            // over a handler that sent the browser somewhere else entirely.
            Box::pin(async move { answer.map(|_| Written::subnet(id)) })
        }

        fn define_range(
            &self,
            subnet_id: String,
            first: Ipv4Addr,
            last: Ipv4Addr,
            _policy: opencmdb_core::ipam::IpPolicy,
            label: String,
        ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
            self.asked
                .lock()
                .expect("the fake port's log")
                .push((format!("{subnet_id}:{first}-{last}"), label));
            let answer = self.take_answer();
            Box::pin(async move { answer.map(|id| Written::inside(id, subnet_id)) })
        }

        fn define_address(
            &self,
            subnet_id: String,
            addr: Ipv4Addr,
            label: String,
        ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
            self.asked
                .lock()
                .expect("the fake port's log")
                .push((format!("{subnet_id}:{addr}"), label));
            let answer = self.take_answer();
            Box::pin(async move { answer.map(|id| Written::inside(id, subnet_id)) })
        }

        // 🔴 **THE THREE DELETES LOG DISTINCT MARKERS, and the review is why**: all three pushed the
        // bare word `"delete"`, and `DeleteRange`/`DeleteAddress` also shared a body shape, a status
        // and — until decision 1 — a redirect. Wiring `/ipam/range/delete` to the address handler was
        // therefore invisible to every guard in this file. *Two routes a test double cannot tell
        // apart are two routes no test distinguishes.*
        fn delete_subnet(&self, id: String) -> BoxFuture<'_, Result<Written, RepositoryError>> {
            self.asked
                .lock()
                .expect("the fake port's log")
                .push((id, "delete_subnet".to_string()));
            let answer = self.take_answer();
            Box::pin(async move { answer.map(Written::subnet) })
        }

        fn delete_range(&self, id: String) -> BoxFuture<'_, Result<Written, RepositoryError>> {
            self.asked
                .lock()
                .expect("the fake port's log")
                .push((id, "delete_range".to_string()));
            let answer = self.take_answer();
            Box::pin(async move { answer.map(|id| Written::inside(id, FAKE_PARENT.to_string())) })
        }

        fn delete_address(&self, id: String) -> BoxFuture<'_, Result<Written, RepositoryError>> {
            self.asked
                .lock()
                .expect("the fake port's log")
                .push((id, "delete_address".to_string()));
            let answer = self.take_answer();
            Box::pin(async move { answer.map(|id| Written::inside(id, FAKE_PARENT.to_string())) })
        }

        fn edit_range(
            &self,
            id: String,
            first: Ipv4Addr,
            last: Ipv4Addr,
            _policy: opencmdb_core::ipam::IpPolicy,
            label: String,
        ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
            self.asked
                .lock()
                .expect("the fake port's log")
                .push((format!("{id}:{first}-{last}"), label));
            let answer = self.take_answer();
            Box::pin(async move { answer.map(|id| Written::inside(id, FAKE_PARENT.to_string())) })
        }

        fn edit_address(
            &self,
            id: String,
            addr: Ipv4Addr,
            label: String,
        ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
            self.asked
                .lock()
                .expect("the fake port's log")
                .push((format!("{id}:{addr}"), label));
            let answer = self.take_answer();
            Box::pin(async move { answer.map(|id| Written::inside(id, FAKE_PARENT.to_string())) })
        }

        fn release_address(
            &self,
            addr: Ipv4Addr,
            until: opencmdb_core::observation::Timestamp,
        ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
            self.asked
                .lock()
                .expect("the fake port's log")
                .push((addr.to_string(), format!("release until {until}")));
            let answer = self.take_answer();
            Box::pin(async move { answer.map(|id| Written::inside(id, FAKE_PARENT.to_string())) })
        }
    }

    /// A same-origin urlencoded POST to the subnet route. `Origin`/`Host` agree, so the CSRF check
    /// passes; the test that probes the check sets the headers itself.
    fn form_post(body: &str) -> Request<Body> {
        form_post_to(WriteRoute::Subnet, body)
    }

    /// The same, to any of the three routes.
    fn form_post_to(route: WriteRoute, body: &str) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri(route.path())
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::HOST, "opencmdb.example")
            .header(header::ORIGIN, "https://opencmdb.example")
            .body(Body::from(body.to_string()))
            .expect("a well-formed test request")
    }

    /// Drive one request through the sub-router over a fake port, and answer with the status and
    /// the body an operator would read.
    async fn drive(port: Arc<dyn IpamWritePort>, request: Request<Body>) -> (StatusCode, String) {
        let response = router_with(port)
            .oneshot(request)
            .await
            .expect("the sub-router answers");
        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("a readable body");
        (status, String::from_utf8_lossy(&body).into_owned())
    }

    /// The bodies are read from the resolver, never copied: pinning the literal here would recreate
    /// the two-spellings drift the keys exist to remove (story 6.4's finding, `document.rs`'s own
    /// note). What is asserted is WHICH key answered, and that is what makes a wrong mapping red.
    fn key(name: &str) -> String {
        rust_i18n::t!(name).to_string()
    }

    #[tokio::test]
    async fn a_cross_origin_post_is_refused_before_anything_else() {
        // 🔑 The port answers a SUCCESS. If the CSRF check did not win, this test would go green on
        // a 201 — which is exactly the ordering story 6.2 §5 buys and what this asserts.
        let port = FakePort::answering(Ok("id".to_string()));
        let request = Request::builder()
            .method("POST")
            .uri(WriteRoute::Subnet.path())
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .header(header::HOST, "opencmdb.example")
            .header(header::ORIGIN, "https://attacker.example")
            .body(Body::from("cidr=192.0.2.0/24&label=Office"))
            .expect("a well-formed test request");
        let (status, body) = drive(port.clone(), request).await;
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "a cross-origin write is refused"
        );
        assert_eq!(body, key("ipam.refusal.cross_origin"));
        assert!(
            port.asked.lock().expect("the log").is_empty(),
            "the port must not have been reached at all"
        );
    }

    #[tokio::test]
    async fn a_well_formed_definition_is_created_and_sends_the_browser_to_the_plan() {
        let port = FakePort::answering(Ok("01900000-0000-7000-8000-000000000001".to_string()));
        let response = router_with(port.clone())
            .oneshot(form_post("cidr=192.0.2.0/24&label=Office"))
            .await
            .expect("the sub-router answers");
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(
            response
                .headers()
                .get("hx-redirect")
                .expect("the browser is sent back to the plan")
                .to_str()
                .expect("an ASCII header"),
            "/ipam?subnet=01900000-0000-7000-8000-000000000001",
            "the redirect selects the subnet just defined — one URL per state"
        );
        assert_eq!(
            port.asked.lock().expect("the log").as_slice(),
            [("192.0.2.0/24 vlan=0".to_string(), "Office".to_string())],
            "the CIDR is parsed, the label trimmed and the VLAN carried — 0 is *none* (story 14.5), \
             and the port RECORDS it, so a handler that parses a VLAN and drops it cannot leave this \
             assertion green"
        );
    }

    /// **AC3 — the correction route carries the VLAN to the store, and carries NOTHING ELSE.**
    ///
    /// 🔴 **The second assertion is the one that measures the scope, and it is a CONTROL rather than
    /// a sentence**: the request sends a `cidr` and a `prefix_len` the type has no field for, and
    /// the port's log shows they reached nothing. Serde drops an unknown field, so this cannot fail
    /// by accident — what it pins is that nobody LATER adds those fields to
    /// [`EditSubnetRequest`] and quietly gives the route the power to move a subnet off its own
    /// children.
    ///
    /// ✅ **Measured, not asserted (M10-bis):** a `cidr` field added to the request AND read by the
    /// handler reds exactly this test, `726 passed; 1 failed`. Its twin M10 — the same field left
    /// unread — reds clippy instead. *Neither half alone would have said the scope is held.*
    #[tokio::test]
    async fn a_subnet_correction_carries_its_vlan_and_no_cidr() {
        let port = FakePort::answering(Ok("unread".to_string()));
        let response = router_with(port.clone())
            .oneshot(form_post_to(
                WriteRoute::EditSubnet,
                "id=01900000-0000-7000-8000-0000000000aa&label=Office&vlan=42&cidr=10.0.0.0/8&prefix_len=8",
            ))
            .await
            .expect("the sub-router answers");
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "a correction is not a mint"
        );
        let asked = port.asked.lock().expect("the log").clone();
        assert_eq!(
            asked.as_slice(),
            [(
                "01900000-0000-7000-8000-0000000000aa vlan=42".to_string(),
                "Office".to_string()
            )],
            "the correction must reach the store with its id, its label and its VLAN — and with no \
             trace of the CIDR the request tried to smuggle past a type that has no field for one"
        );
    }

    /// **AC3 — the correction is held to the SAME VLAN rules as the definition.**
    ///
    /// ⚠️ Driven through the ROUTE and not through `parse_vlan`, for this file's standing reason: a
    /// handler that parses the field and forgets to refuse on it leaves a unit test of the parser
    /// perfectly green. The port is asserted UNREACHED, which is what says the refusal happened
    /// before the write rather than after it.
    /// 🔴 **BOTH ROUTES THAT ASK FOR A VLAN, and it was ONE until the mutation pass said so.**
    /// Mutation M2 (`1..=4094` → `1..=4095`) was predicted to red two tests and reddened one:
    /// `ipam.refusal.not_a_vlan` was asserted at exactly one site in the whole file, and it was the
    /// CORRECTION's. The DEFINITION route — AC3's first half, shipped two commits earlier — had no
    /// test for its VLAN refusal at all. *A divergence between a prediction and a measurement is
    /// the finding*, and here the measurement was right and the prediction described a test that
    /// did not exist.
    #[tokio::test]
    async fn every_route_that_asks_for_a_vlan_refuses_one_outside_the_domain() {
        // ⚠️ `+1` is NOT here and was measured rather than reasoned about: Rust's `u16::from_str`
        // accepts a leading plus, so it answers 200 and writes VLAN 1 — which is the number the
        // operator typed. Left alone; listing it as a refusal would have pinned a behaviour the
        // product does not have.
        let bodies: [(WriteRoute, &str); 2] = [
            (WriteRoute::Subnet, "cidr=192.0.2.0/24&label=Office"),
            (
                WriteRoute::EditSubnet,
                "id=01900000-0000-7000-8000-0000000000aa&label=Office",
            ),
        ];
        for (route, prefix) in bodies {
            for raw in ["4095", "0", "-1", "ten", "4 095", "1.0"] {
                let port = FakePort::answering(Ok("unreached".to_string()));
                let (status, body) = drive(
                    port.clone(),
                    form_post_to(route, &format!("{prefix}&vlan={raw}")),
                )
                .await;
                assert_eq!(
                    status,
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "`{raw}` is not a VLAN this plan can hold, at `{}`",
                    route.path()
                );
                assert_eq!(
                    body,
                    key("ipam.refusal.not_a_vlan"),
                    "at `{}`",
                    route.path()
                );
                assert!(
                    port.asked.lock().expect("the log").is_empty(),
                    "`{raw}` was refused only after the store had been asked to write it, at `{}`",
                    route.path()
                );
            }
        }
    }

    /// ⚠️ **The two values that are NOT refused, kept beside the refusals rather than assumed.** An
    /// EMPTY field is *no VLAN* and must reach the store as the sentinel 0, never be refused as *not
    /// a number* — which is what an operator clearing the field would meet. And a number WRAPPED IN
    /// WHITESPACE is accepted, which is where this story found a false sentence of its own: the
    /// criterion predicted a 422 for `12 `, the route answered 200, and `parse_vlan`'s doc claimed a
    /// trim it did not describe. The behaviour is kept and the sentence corrected; the control is
    /// here so neither can drift again in silence.
    #[tokio::test]
    async fn an_empty_vlan_field_corrects_a_subnet_to_none() {
        let port = FakePort::answering(Ok("unread".to_string()));
        let (status, _) = drive(
            port.clone(),
            form_post_to(
                WriteRoute::EditSubnet,
                "id=01900000-0000-7000-8000-0000000000aa&label=Office&vlan=",
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            port.asked.lock().expect("the log").as_slice(),
            [(
                "01900000-0000-7000-8000-0000000000aa vlan=0".to_string(),
                "Office".to_string()
            )],
            "an emptied field means *none*, which the store spells 0"
        );

        let port = FakePort::answering(Ok("unread".to_string()));
        let (status, _) = drive(
            port.clone(),
            form_post_to(
                WriteRoute::EditSubnet,
                "id=01900000-0000-7000-8000-0000000000aa&label=Office&vlan=%2012%20",
            ),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::OK,
            "a padded number is still the number typed"
        );
        assert_eq!(
            port.asked.lock().expect("the log").as_slice(),
            [(
                "01900000-0000-7000-8000-0000000000aa vlan=12".to_string(),
                "Office".to_string()
            )],
            "the padding is trimmed and the VLAN reaches the store as 12"
        );
    }

    #[tokio::test]
    async fn a_body_that_is_not_the_form_is_refused_as_malformed() {
        let port = FakePort::answering(Ok("unreached".to_string()));
        let (status, body) = drive(port, form_post("subject=nothing-like-it")).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body, key("ipam.refusal.malformed_subnet"));
    }

    #[tokio::test]
    async fn an_empty_label_is_refused_by_its_own_rule() {
        // ⚠️ WHITESPACE, not the empty string: `label=` and `label=%20%20` are the same mistake,
        // and only the trimmed form catches the second.
        let port = FakePort::answering(Ok("unreached".to_string()));
        let (status, body) = drive(port, form_post("cidr=192.0.2.0/24&label=%20%20")).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body, key("ipam.refusal.label_empty"));
    }

    #[tokio::test]
    async fn a_label_over_the_column_is_refused_here_rather_than_by_the_driver() {
        let port = FakePort::answering(Ok("unreached".to_string()));
        let long = "e".repeat(MAX_LABEL_CHARS + 1);
        let (status, body) =
            drive(port, form_post(&format!("cidr=192.0.2.0/24&label={long}"))).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body, key("ipam.refusal.label_too_long"));
    }

    #[tokio::test]
    async fn a_label_at_the_column_in_accented_characters_is_accepted() {
        // 🔑 THE UNIT IS CHARACTERS, NOT BYTES. 120 accented characters are 240 bytes, and a
        // byte-counting bound would refuse a label the column accepts — which is the same class as
        // measuring a column in the wrong unit, one layer up.
        let port = FakePort::answering(Ok("id".to_string()));
        let accented = "é".repeat(MAX_LABEL_CHARS);
        let (status, _) = drive(
            port,
            form_post(&format!("cidr=192.0.2.0/24&label={accented}")),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::CREATED,
            "120 characters is 240 bytes and still 120 characters"
        );
    }

    #[tokio::test]
    async fn text_that_is_not_a_cidr_is_refused_as_malformed() {
        for spelling in [
            "192.0.2.0",
            "not-an-address/24",
            "192.0.2.0/",
            "192.0.2.0/999",
        ] {
            let port = FakePort::answering(Ok("unreached".to_string()));
            let (status, body) =
                drive(port, form_post(&format!("cidr={spelling}&label=Office"))).await;
            assert_eq!(
                status,
                StatusCode::UNPROCESSABLE_ENTITY,
                "`{spelling}` is not a subnet the product can evaluate"
            );
            assert_eq!(
                body,
                key("ipam.refusal.malformed_subnet"),
                "for `{spelling}`"
            );
        }
    }

    #[tokio::test]
    async fn a_prefix_outside_the_family_names_that_rule_and_not_malformed() {
        // 🔴 The distinction is deliberate and is the honest one: `/999` does not fit a `u8` and is
        // MALFORMED, `/64` fits and cannot belong to IPv4. The product names only a rule it can
        // evaluate. ⚠️ And this one is a REFUSAL rather than a lint because `Subnet::last` would
        // compute `32 - 64` and panic on subtract-with-overflow (story 14.1's measurement).
        let port = FakePort::answering(Ok("unreached".to_string()));
        let (status, body) = drive(port, form_post("cidr=192.0.2.0/64&label=Office")).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body, key("ipam.refusal.prefix_not_in_family"));
    }

    #[tokio::test]
    async fn a_base_carrying_host_bits_is_refused_and_the_sentence_names_the_subnet_meant() {
        let port = FakePort::answering(Ok("unreached".to_string()));
        let (status, body) = drive(port, form_post("cidr=192.0.2.5/24&label=Office")).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body, key("ipam.refusal.base_not_network"));
    }

    #[tokio::test]
    async fn a_subnet_the_plan_already_holds_is_a_conflict() {
        let port = FakePort::answering(Err(RepositoryError::Constraint("unique")));
        let (status, body) = drive(port, form_post("cidr=192.0.2.0/24&label=Office")).await;
        assert_eq!(
            status,
            StatusCode::CONFLICT,
            "the likeliest refusal on this screen: the operator re-enters what is already there"
        );
        assert_eq!(body, key("ipam.refusal.already_defined_subnet"));
    }

    #[tokio::test]
    async fn a_re_entered_address_is_told_it_is_the_address() {
        // 🔴 The review's measurement, as a test: the same `Constraint("unique")` on the address
        // route answered *"The plan already holds that subnet."*
        let port = FakePort::answering(Err(RepositoryError::Constraint("unique")));
        let (status, body) = drive(
            port,
            form_post_to(
                WriteRoute::Address,
                &a_valid_body(WriteRoute::Address, "Office"),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
        // The equality is the whole assertion: that the address sentence differs from the subnet's is
        // `every_refusal_the_handler_can_receive_names_a_rule`'s, and an `assert_ne!` repeating it
        // here could not fail on its own (the second review).
        assert_eq!(body, key("ipam.refusal.already_defined_address"));
    }

    /// A label an operator really types, in any script, is accepted.
    ///
    /// 🔑 The control that makes the refusals mean something: the second review's edge layer measured
    /// each of these accepted, and a wider refusal written in a hurry — every invisible code point,
    /// every combining mark — would refuse a family emoji, a decomposed accent or an Arabic label.
    #[test]
    fn real_labels_in_any_script_are_accepted() {
        for label in [
            "Bureau Genève",
            "東京オフィス",
            "📡 Wi-Fi",
            "משרד ראשי",
            "مكتب",
            "e\u{301}cole",
            "👨\u{200D}👩\u{200D}👧 lab",
            "❤\u{FE0F} rack",
            "Ὀδυσσεύς",
            "\u{5E9}\u{5E8}\u{5EA} \u{200F}2",
        ] {
            assert!(
                checked_label(label).is_ok(),
                "`{label}` is a label an operator types, and it was refused: {:?}",
                checked_label(label)
            );
        }
    }

    #[tokio::test]
    async fn contention_is_transient_and_says_so() {
        // 🔴 Unreachable before this story: `classify`'s `Contention` arm was dead code until T1
        // repaired it, because nothing in this product contended.
        let port = FakePort::answering(Err(RepositoryError::Contention));
        let (status, body) = drive(port, form_post("cidr=192.0.2.0/24&label=Office")).await;
        assert_eq!(
            status,
            StatusCode::SERVICE_UNAVAILABLE,
            "nothing about the request is wrong, so it is not a 4xx"
        );
        assert_eq!(body, key("ipam.refusal.contention"));
    }

    #[tokio::test]
    async fn a_backend_failure_leaks_none_of_its_cause() {
        let port = FakePort::answering(Err(RepositoryError::Backend(
            "Access denied for user 'root'@'10.0.0.7' (using password: YES)".to_string(),
        )));
        let (status, body) = drive(port, form_post("cidr=192.0.2.0/24&label=Office")).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        // 🔑 The equality IS the no-leak assertion: the body is the backend KEY's sentence and
        // nothing else. A `!body.contains("root")` beside it could not fail — a `Refusal` holds a
        // `&'static str` key and no room for the driver's text — and the review called it vacuous.
        assert_eq!(body, key("ipam.refusal.backend"));
    }

    /// Every refusal the plan's domain can raise renders a DISTINCT sentence.
    ///
    /// 🔴 Story 14.1's review measured why this is a property over `IpamError::ALL` and not a list
    /// written here: its predecessor hand-wrote a six-element array in its own body, so a seventh
    /// variant returning a VERBATIM duplicate left the whole suite green.
    #[test]
    fn every_domain_refusal_has_its_own_sentence_in_both_locales() {
        for locale in ["en", "fr"] {
            let mut seen: Vec<String> = Vec::new();
            for error in IpamError::ALL {
                // The delete's gesture, so the sentence read here is the one `DeleteRange` earns;
                // `the_edit_and_the_delete_do_not_share_a_sentence` covers the other gesture.
                let refusal = ipam_refusal(&error, WriteRoute::DeleteRange);
                // ⚠️ **409 for the one refusal about the PLAN'S STATE, 422 for the seven about the
                // REQUEST** (Guy's decision 3, 2026-09-16). This asserted 422 for all eight under a
                // message reading *"the operator's mistake, not the server's"* — true of both, and
                // not what separates them.
                // Story 14.4b adds a second of the first kind: a release of an address the plan
                // DEFINES is opposed by the plan's state, not by the request (decision 4).
                let expected = if matches!(
                    error,
                    IpamError::RangeStillHoldsAddresses | IpamError::ReleaseOfADefinedAddress
                ) {
                    StatusCode::CONFLICT
                } else {
                    StatusCode::UNPROCESSABLE_ENTITY
                };
                assert_eq!(
                    refusal.status(),
                    expected,
                    "{error:?} is the operator's mistake, not the server's — and a refusal about \
                     what the plan HOLDS is a conflict, where one about the request is not"
                );
                // 🔑 The key comes from the MAPPER, never from a copy of it written here. A test
                // that restates the mapping measures the copy: story 6.5's M8 — *a count is not a
                // set* — one degree over, where a second table would agree with itself.
                let sentence = rust_i18n::t!(refusal.key(), locale = locale).to_string();
                assert_ne!(
                    sentence,
                    refusal.key(),
                    "{error:?} has no `{locale}` translation, so the page renders its key name"
                );
                assert!(
                    !seen.contains(&sentence),
                    "two refusals read the same in `{locale}`, so the operator cannot tell them \
                     apart: {sentence}"
                );
                seen.push(sentence);
            }
        }
    }

    #[tokio::test]
    async fn the_cidr_is_read_before_the_label() {
        // A form wrong in BOTH places can only be told about one of them. Nothing pinned the order
        // when T2 shipped, so either answer would have passed; this is the decision, asserted.
        let port = FakePort::answering(Ok("unreached".to_string()));
        let (status, body) = drive(port, form_post("cidr=nonsense&label=")).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            body,
            key("ipam.refusal.malformed_subnet"),
            "the first field the operator filled is the one the refusal names"
        );
    }

    /// Every constraint NAME `repo::classify` can produce is mapped here by name.
    ///
    /// 🔑 **This is AC4's second carrier, and it exists because the compiler cannot be the first
    /// one twice.** An exhaustive `match` over `RepositoryError` makes a new VARIANT an `E0004`;
    /// [`RepositoryError::Constraint`] carries a `&'static str`, so a new NAME is invisible to it.
    /// Add `Constraint("range_overlap")` to `classify` and this test reds while every other test,
    /// clippy and all ten gates stay green.
    ///
    /// ⚠️ **It reads the WHOLE of `repo.rs`, test module included, deliberately.** Story 14.2's
    /// guard split on the first `#[cfg(test)]` and thereby read 382 bytes of 47 324; here the test
    /// module's own literals are the same three names, so including them costs nothing and the trap
    /// is not reachable at all. *A perimeter you do not have to compute is a perimeter that cannot
    /// be computed wrongly.*
    #[test]
    fn every_constraint_name_the_store_can_produce_is_mapped() {
        const NEEDLE: &str = "RepositoryError::Constraint(\"";
        let source = include_str!("repo.rs");
        let mut names: Vec<&str> = Vec::new();
        let mut rest = source;
        while let Some(at) = rest.find(NEEDLE) {
            let after = &rest[at + NEEDLE.len()..];
            let end = after.find('"').expect("a closed string literal");
            let name = &after[..end];
            if !names.contains(&name) {
                names.push(name);
            }
            rest = &after[end..];
        }
        // 🔴 THE MAPPING IS ASSERTED FIRST, AND THE ORDER IS THE FINDING. The floor came first in
        // the draft, so a new constraint name reddened on `left: 4, right: 3` — a count — and the
        // assertion written for the defect was never reached. Story 5.13 met the same shape and
        // this project has now met it five times: *an assertion that fires first decides what the
        // failure says*, and a count says nothing a reader can act on.
        for name in &names {
            for route in WriteRoute::ALL {
                assert!(
                    constraint_refusal(name, *route).is_some(),
                    "`Constraint(\"{name}\")` reaches `{}` and nobody has said what it means to \
                     the operator — map it by name, even when the answer is the backend sentence",
                    route.path()
                );
            }
        }
        // A floor equal to what is there, never under it (story 6b.7). It catches the other
        // direction: a name that DISAPPEARS, which leaves a mapping nothing can reach.
        assert_eq!(
            names.len(),
            3,
            "the names `classify` can produce changed: {names:?}"
        );
    }

    /// Every key this module can render resolves in BOTH locales.
    ///
    /// ⚠️ **A source scan reads the file's prose as well as its code** (story 14.2's finding, where
    /// a guard was satisfied by the comment narrating the defect). Here that is harmless and even
    /// useful: the needle is a double-quoted literal, so a key named in a doc comment inside
    /// backticks is not matched, and a key named in a literal that does NOT exist is a finding
    /// whichever line it sits on.
    #[test]
    fn every_key_this_module_can_render_resolves_in_both_locales() {
        let source = include_str!("ipam_write.rs");
        // 🔴 THE NEEDLE IS BUILT AT RUNTIME, and the first version was not — it found ITSELF, then
        // found the comment saying so. A scan of its own source matches the literal spelling the
        // scan is written with, so the needle came back as a key named after the prefix alone and
        // the test reddened on a key nobody wrote; the first repair explained the trap IN PROSE
        // CONTAINING THE SEQUENCE, and reddened again on the explanation. 🔑 *A guard that greps a
        // file greps its prose, and the better the prose explains the defect, the more reliably it
        // reproduces it* — story 14.2's finding, met twice in five minutes. Hence: the prefix is
        // assembled from a quote character and the namespace, and this comment names neither
        // adjacently.
        let needle = format!("{}ipam.", '"');
        let mut keys: Vec<&str> = Vec::new();
        let mut rest = source;
        while let Some(at) = rest.find(&needle) {
            let after = &rest[at + 1..];
            let end = after.find('"').expect("a closed string literal");
            let name = &after[..end];
            if !keys.contains(&name) {
                keys.push(name);
            }
            rest = &after[end..];
        }
        // A floor EQUAL to what is there (story 6b.7) — it read `>= 15` over 20-odd keys until the
        // review, which is a floor that tolerates losing a quarter of what it guards.
        // 🔑 42 → 45 at story 14.5, read off the list this prints: `ipam.refusal.not_a_vlan`, then
        // the tenth route's two — `ipam.refusal.malformed_edit_subnet` and `ipam.done.edit_subnet`.
        // ⚠️ The re-entry sentence is NOT a fourth: the correction shares the definition's key.
        assert_eq!(
            keys.len(),
            45,
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
    }

    /// Every refusal the HANDLER CAN RECEIVE names a rule the operator can read.
    ///
    /// 🔴 **The set is what the adapter hands back, not `IpamError::ALL`** — §1(c) measured that
    /// four of the refusals an operator meets first are not `IpamError`s at all: an empty plan is
    /// `NotFound`, a re-entered address is `Constraint("unique")`, the loser of a lock wait is
    /// `Contention` since T1 repaired `classify`, and anything else is `Backend`. *A set over
    /// `IpamError` is correct about `IpamError` and silent about the four.*
    #[test]
    fn every_refusal_the_handler_can_receive_names_a_rule() {
        let receivable = [
            RepositoryError::NotFound,
            RepositoryError::Constraint("unique"),
            RepositoryError::Constraint("foreign_key"),
            RepositoryError::Constraint("check"),
            RepositoryError::Contention,
            RepositoryError::Backend("1406: Data too long for column 'label'".to_string()),
            RepositoryError::Ipam(IpamError::RangeOverlapsAnother),
        ];
        // ⚠️ The review found two assertions here that could not fail — a status check restating the
        // mapper's construction, and `"1406"` absent from a sentence built from a key. They are gone;
        // what can fail is a key with no translation, and a sentence naming the wrong RECORD.
        for route in WriteRoute::ALL {
            for error in &receivable {
                let refusal = repository_refusal(error, *route);
                for locale in ["en", "fr"] {
                    let sentence = rust_i18n::t!(refusal.key(), locale = locale).to_string();
                    assert_ne!(
                        sentence,
                        refusal.key(),
                        "{error} renders its key name in `{locale}` at `{}`",
                        route.path()
                    );
                }
            }
        }
        // 🔴 **THE PROPERTY IS ABOUT THE RECORD, NOT THE ROUTE — and story 14.4 is where the two
        // stopped being the same thing.** 14.2b's review found one `unique` sentence serving three
        // records (a re-entered ADDRESS told the plan already held that SUBNET), and the guard
        // written for it asserted distinctness per ROUTE, which was exact only while routes and
        // records stood in bijection. They no longer do: `EditAddress` and `Address` concern the
        // SAME record and SHOULD answer the same sentence, so per-route distinctness would have
        // forbidden the correct behaviour — *a guard whose proxy outlives the thing it stood for*.
        //
        // ⚠️ And the three deletes cannot raise `unique` AT ALL: a `DELETE` hits no unique key. A
        // distinct sentence apiece would be copy no operator can ever read (story 5.12: detection
        // for a form that cannot execute is code carried by nothing), so they answer the backend
        // sentence on this file's own reasoning for `check` — a constraint that cannot arise
        // through the adapter is a fault here, not a mistake to explain.
        const BY_RECORD: [&[WriteRoute]; 3] = [
            // 🔑 The correction joins the definition rather than earning a sentence of its own:
            // story 14.5 gave both the SAME key to collide on, so one record is one sentence here
            // in the strict sense — see `already_defined`'s own comment.
            &[WriteRoute::Subnet, WriteRoute::EditSubnet],
            &[WriteRoute::Range, WriteRoute::EditRange],
            &[WriteRoute::Address, WriteRoute::EditAddress],
        ];
        // 🔑 The release joins the deletes: it is an UPSERT on its own primary key (story 14.4b).
        const CANNOT_COLLIDE: [WriteRoute; 4] = [
            WriteRoute::DeleteSubnet,
            WriteRoute::DeleteRange,
            WriteRoute::DeleteAddress,
            WriteRoute::Release,
        ];
        // 🔴 **A SET, NOT A COUNT** — story 6.5's M8 met one file over, and the review measured it
        // here: this summed the group lengths and compared the total against `ALL.len()`, so listing
        // one route TWICE and omitting another also reaches eight and passes. The message asserted
        // membership; the assertion checked cardinality.
        let mut classified: Vec<WriteRoute> = BY_RECORD.concat();
        classified.extend(CANNOT_COLLIDE);
        for route in WriteRoute::ALL {
            assert_eq!(
                classified.iter().filter(|listed| *listed == route).count(),
                1,
                "`{}` belongs to no record group and to no cannot-collide list — or to more than \
                 one — so nothing says which sentence it owes for a re-entered record",
                route.path()
            );
        }
        assert_eq!(
            classified.len(),
            WriteRoute::ALL.len(),
            "a route was classified here that the route list does not carry"
        );
        for locale in ["en", "fr"] {
            let mut by_record: Vec<String> = Vec::new();
            for group in BY_RECORD {
                let sentences: Vec<String> = group
                    .iter()
                    .map(|route| {
                        let refusal =
                            repository_refusal(&RepositoryError::Constraint("unique"), *route);
                        rust_i18n::t!(refusal.key(), locale = locale).to_string()
                    })
                    .collect();
                assert!(
                    sentences.windows(2).all(|pair| pair[0] == pair[1]),
                    "two routes concerning ONE record disagree about it in `{locale}`: {sentences:?}"
                );
                let sentence = sentences
                    .first()
                    .expect("every record group names at least one route")
                    .clone();
                assert!(
                    !by_record.contains(&sentence),
                    "two DIFFERENT records share a re-entry sentence in `{locale}`, which is the \
                     defect 14.2b measured: {sentence}"
                );
                by_record.push(sentence);
            }
            for route in CANNOT_COLLIDE {
                let refusal = repository_refusal(&RepositoryError::Constraint("unique"), route);
                assert_eq!(
                    refusal.key(),
                    "ipam.refusal.backend",
                    "`{}` cannot raise a unique key, so a sentence explaining a re-entered record \
                     there would be copy no operator can reach",
                    route.path()
                );
            }
        }
    }

    /// **AC9b — a write that never finishes answers, and does not hold the browser.**
    ///
    /// ⚠️ **The clock is PAUSED, and what that does and does not prove is said rather than left to
    /// be assumed.** `start_paused` makes tokio auto-advance to the next timer, so this measures
    /// the BRANCH — that the budget is armed, that it wraps the work, and that the operator gets a
    /// sentence — and it does NOT measure five real seconds. The alternative was a five-second
    /// test, which buys one number and costs it on every run for ever.
    ///
    /// 🔑 The port's future never completes, so without the budget this test would HANG rather
    /// than fail — which is the defect exactly: a handler with no budget does not answer wrongly,
    /// it does not answer.
    #[tokio::test(start_paused = true)]
    async fn a_write_that_never_finishes_answers_within_its_budget() {
        struct NeverAnswers;
        impl IpamWritePort for NeverAnswers {
            fn define_subnet(
                &self,
                _subnet: Subnet,
                _label: String,
                _vlan: u16,
            ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
                Box::pin(std::future::pending())
            }
            fn edit_subnet(
                &self,
                _id: String,
                _label: String,
                _vlan: u16,
            ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
                Box::pin(std::future::pending())
            }
            fn define_range(
                &self,
                _subnet_id: String,
                _first: Ipv4Addr,
                _last: Ipv4Addr,
                _policy: opencmdb_core::ipam::IpPolicy,
                _label: String,
            ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
                Box::pin(std::future::pending())
            }
            fn define_address(
                &self,
                _subnet_id: String,
                _addr: Ipv4Addr,
                _label: String,
            ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
                Box::pin(std::future::pending())
            }
            fn delete_subnet(
                &self,
                _id: String,
            ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
                Box::pin(std::future::pending())
            }
            fn delete_range(&self, _id: String) -> BoxFuture<'_, Result<Written, RepositoryError>> {
                Box::pin(std::future::pending())
            }
            fn delete_address(
                &self,
                _id: String,
            ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
                Box::pin(std::future::pending())
            }
            fn edit_range(
                &self,
                _id: String,
                _first: Ipv4Addr,
                _last: Ipv4Addr,
                _policy: opencmdb_core::ipam::IpPolicy,
                _label: String,
            ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
                Box::pin(std::future::pending())
            }
            fn edit_address(
                &self,
                _id: String,
                _addr: Ipv4Addr,
                _label: String,
            ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
                Box::pin(std::future::pending())
            }
            fn release_address(
                &self,
                _addr: Ipv4Addr,
                _until: opencmdb_core::observation::Timestamp,
            ) -> BoxFuture<'_, Result<Written, RepositoryError>> {
                Box::pin(std::future::pending())
            }
        }
        // 🔴 **EVERY ROUTE, and the review measured why**: this drove the subnet route alone, and
        // replacing `within_budget(work).await` with `work.await` in `define_range` — the only route
        // that takes locks — reddened nothing and hung nothing. The story said a missing budget
        // would HANG this test and so repeating it per route bought nothing; it was never run on the
        // other two. 🔑 The outer timeout is what turns that hang into a red: the paused clock
        // advances to it, and a handler with no budget never answers before it.
        for route in WriteRoute::ALL {
            let answered = tokio::time::timeout(
                IPAM_WRITE_BUDGET * 2,
                drive(
                    Arc::new(NeverAnswers),
                    form_post_to(*route, &a_valid_body(*route, "Office")),
                ),
            )
            .await;
            let Ok((status, body)) = answered else {
                panic!(
                    "`{}` did not answer within twice its budget: the handler has no budget, so a \
                     write that never finishes holds the browser for as long as the store likes",
                    route.path()
                );
            };
            assert_eq!(
                status,
                StatusCode::SERVICE_UNAVAILABLE,
                "`{}`: a write that outlives its budget is answered, not waited out",
                route.path()
            );
            assert_eq!(
                body,
                key("ipam.refusal.contention"),
                "`{}`: the operator is told the same thing as for a real lock wait — reload, and \
                 try again if the change is missing",
                route.path()
            );
        }
    }

    /// A body every field of which is valid, for each route.
    ///
    /// 🔑 The `match` is exhaustive, so a fourth route cannot be added without saying what a good
    /// request to it looks like — which is what makes the reuse test cover it automatically rather
    /// than by someone remembering.
    /// ⚠️ **`label` is IGNORED by the three deletes, and that is not an oversight**: their forms
    /// carry no label at all, so a label probe against them is a probe of a field that does not
    /// exist — serde drops an unknown field, the write goes through, and a guard demanding 422
    /// would fail over a correct product. That is why the reuse test partitions the label rules by
    /// [`WriteRoute::carries_label`] rather than driving every route through them.
    fn a_valid_body(route: WriteRoute, label: &str) -> String {
        let subnet = "01900000-0000-7000-8000-0000000000aa";
        let record = "01900000-0000-7000-8000-0000000000bb";
        match route {
            WriteRoute::Subnet => format!("cidr=192.0.2.0/24&label={label}"),
            WriteRoute::Range => format!(
                "subnet_id={subnet}&first=192.0.2.10&last=192.0.2.20&policy=static&label={label}"
            ),
            WriteRoute::Address => format!("subnet_id={subnet}&addr=192.0.2.9&label={label}"),
            // 🔴 **THE DELETE FORMS SEND A LABEL HERE, AND THAT IS THE REPAIR.** The label branch
            // below asserts that a delete route DROPS an unknown `label` and writes anyway — and
            // these arms interpolated none, so twelve byte-identical label-free requests were driven
            // through a branch whose comment claimed to be measuring serde's behaviour. *A
            // `continue` wearing an assertion*, which is the shape that comment says it exists to
            // avoid; three review layers reached it. The field is sent now, so the sentence beside
            // the assertion is true and measured rather than asserted about a request nobody made.
            WriteRoute::DeleteSubnet | WriteRoute::DeleteRange | WriteRoute::DeleteAddress => {
                format!("id={record}&label={label}")
            }
            WriteRoute::EditRange => {
                format!("id={record}&first=192.0.2.10&last=192.0.2.20&policy=static&label={label}")
            }
            WriteRoute::EditAddress => format!("id={record}&addr=192.0.2.9&label={label}"),
            // 🔑 A subnet correction addresses a SUBNET, so its id is the subnet's and its redirect
            // lands on the plan it just corrected. It carries no CIDR: the request type has none.
            WriteRoute::EditSubnet => format!("id={subnet}&label={label}&vlan=10"),
            // A release carries an address and no label; the label is sent for the deletes' reason.
            WriteRoute::Release => {
                format!("addr=192.0.2.9&seen_until=2026-09-01T10:00:00.000000Z&label={label}")
            }
        }
    }

    /// **AC1 — the second and third routes REUSE the machinery, and this drives all three through
    /// the same probes rather than trusting a comment that says so.**
    ///
    /// 🔴 The criterion's words are *"the others reuse it, and a TEST asserts the reuse rather than
    /// a comment claiming it"*. A comment can be true when written and false two commits later; a
    /// loop over `WriteRoute::ALL` covers a route the day it is added.
    ///
    /// ⚠️ What it asserts is that each route answers the SAME WAY to the same shared mistake — not
    /// that they call the same function, which no test can see. A second implementation that
    /// happened to behave identically would pass, and that is the honest limit of a behavioural
    /// guard. What narrows it is that the shape refusal is per-route and asserted DISTINCT, so a
    /// copy would have to reproduce the difference too.
    #[tokio::test]
    async fn every_route_reuses_the_shared_machinery() {
        let mut shape_sentences: Vec<String> = Vec::new();
        for route in WriteRoute::ALL {
            let route = *route;
            let at = route.path();

            // The Origin check, decided first: the port answers a SUCCESS, so a route that skipped
            // the check would go green on a 201.
            let cross = Request::builder()
                .method("POST")
                .uri(at)
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::HOST, "opencmdb.example")
                .header(header::ORIGIN, "https://attacker.example")
                .body(Body::from(a_valid_body(route, "Office")))
                .expect("a well-formed test request");
            let port = FakePort::answering(Ok("unreached".to_string()));
            let (status, body) = drive(port.clone(), cross).await;
            assert_eq!(
                status,
                StatusCode::FORBIDDEN,
                "`{at}` skipped the Origin check"
            );
            assert_eq!(body, key("ipam.refusal.cross_origin"), "at `{at}`");
            assert!(
                port.asked.lock().expect("the log").is_empty(),
                "`{at}` reached the port despite a cross-origin request"
            );

            // The label rules. 🔴 The last two are the review's measurements: a zero-width label
            // answered 201 and rendered no name, and a NUL was stored and then served in every page.
            for (label, key_name, what) in [
                ("%20%20", "ipam.refusal.label_empty", "a blank label"),
                (
                    "%E2%80%8B%E2%80%8B",
                    "ipam.refusal.label_empty",
                    "a label with nothing visible in it",
                ),
                (
                    "ctrl%00",
                    "ipam.refusal.label_control",
                    "a label carrying a control character",
                ),
                (
                    "a%E2%80%A8b",
                    "ipam.refusal.label_control",
                    "a label carrying a line separator",
                ),
                (
                    "Office%E2%80%AElive",
                    "ipam.refusal.label_control",
                    "a label carrying a direction override",
                ),
                (
                    "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
                    "ipam.refusal.label_too_long",
                    "a label past the column",
                ),
            ] {
                // 🔴 **A DELETION NAMES A RECORD; IT DOES NOT DESCRIBE ONE**, so its form has no
                // `label` field and there is no label rule for it to hold. ⚠️ This is ASSERTED
                // rather than skipped: a `continue` here would read exactly like a passing check,
                // which is the shape this project keeps finding in its own guards. What is measured
                // is the real behaviour — `a_valid_body` really does send `label=` at these three
                // routes, serde DROPS the unknown field, and the write goes through — so the day a
                // delete form gains a label, this assertion reds and the rules must be extended
                // rather than quietly not applying.
                //
                // ⚠️ **Until the review the body carried no `label` at all**, so this branch drove
                // identical label-free requests and the sentence above described a measurement
                // nobody had taken. *The claim was the defect, not the code.*
                if !route.carries_label() {
                    let port =
                        FakePort::answering(Ok("01900000-0000-7000-8000-0000000000bb".to_string()));
                    let (status, _) =
                        drive(port, form_post_to(route, &a_valid_body(route, label))).await;
                    // A literal, not `route.success_status()`: the three routes reaching this branch
                    // are deletions and a correction answers 200. Comparing against the mapper would
                    // restate its own output — the defect the review found one assertion down.
                    assert_eq!(
                        status,
                        StatusCode::OK,
                        "`{at}` carries no label, so {what} is a field nothing reads — if this form \
                         has gained one, the label rules must now cover it"
                    );
                    continue;
                }
                let port = FakePort::answering(Ok("unreached".to_string()));
                let (status, body) =
                    drive(port, form_post_to(route, &a_valid_body(route, label))).await;
                assert_eq!(
                    status,
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "`{at}` accepted {what}"
                );
                assert_eq!(
                    body,
                    key(key_name),
                    "`{at}` named the wrong rule for {what}"
                );
            }

            // The shape refusal, which is per-route and must stay so.
            let port = FakePort::answering(Ok("unreached".to_string()));
            let (status, body) = drive(port, form_post_to(route, "nothing=useful")).await;
            assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "at `{at}`");
            assert!(
                !shape_sentences.contains(&body),
                "`{at}` gives the same shape sentence as another form, so it names none of its own \
                 fields: {body}"
            );
            shape_sentences.push(body);

            // And the success path, including the redirect every route owes.
            let port = FakePort::answering(Ok("01900000-0000-7000-8000-0000000000bb".to_string()));
            let response = router_with(port)
                .oneshot(form_post_to(route, &a_valid_body(route, "Office")))
                .await
                .expect("the sub-router answers");
            // 🔴 **A SECOND, HAND-WRITTEN STATEMENT OF THE MAPPING — not `route.success_status()`,
            // which is exactly what `answer` returns.** The assertion compared the producer against
            // itself: `success_status` answering `OK` for every route, or `IM_A_TEAPOT` for every
            // route, left it green while its own message named the split it could not check. What it
            // still caught is a handler bypassing the mapper with a hardcoded status; what it could
            // not catch is the mapper being wrong. Two representations of one decision pinned by an
            // equality is the deliberate redundancy this codebase sanctions.
            let expected_status = match route {
                WriteRoute::Subnet | WriteRoute::Range | WriteRoute::Address => StatusCode::CREATED,
                WriteRoute::DeleteSubnet
                | WriteRoute::DeleteRange
                | WriteRoute::DeleteAddress
                | WriteRoute::EditRange
                | WriteRoute::EditAddress
                | WriteRoute::EditSubnet
                | WriteRoute::Release => StatusCode::OK,
            };
            assert_eq!(
                response.status(),
                expected_status,
                "`{at}` answered the wrong status for a write that went through: 201 belongs to \
                 the three definitions, 200 to the five corrections"
            );
            // 🔑 A REMOVED SUBNET HAS NO PLAN TO GO BACK TO, so it is the one route that redirects
            // to `/ipam` bare. Asserted per route rather than loosened to `starts_with("/ipam")`,
            // which would have passed for every route whatever it sent.
            let expected_redirect = match route {
                // A removed subnet has no plan to go back to.
                WriteRoute::DeleteSubnet => "/ipam".to_string(),
                // 🔑 A DEFINED subnet IS its own redirect target: its form carries no `subnet_id`,
                // so `answer` falls back to the id just minted. ⚠️ My first expectation said `…aa`
                // for every route but the delete, which `answer`'s own comment contradicts — *the
                // test was wrong and the product was right*, and it is recorded that way round
                // rather than quietly corrected.
                WriteRoute::Subnet => {
                    "/ipam?subnet=01900000-0000-7000-8000-0000000000bb".to_string()
                }
                // ⚠️ Enumerated, where the review found a `_` under a comment arguing against a
                // different loosening: a ninth route inherited this expectation in silence.
                // 🔑 For the four corrections this value now comes from the PORT (`FAKE_PARENT`),
                // not from the request body — which is decision 1 made visible in the test: before
                // it, the assertion was satisfied by the form echoing its own hidden field back.
                WriteRoute::Range
                | WriteRoute::Address
                | WriteRoute::DeleteRange
                | WriteRoute::DeleteAddress
                | WriteRoute::EditRange
                | WriteRoute::EditAddress
                // 🔑 A corrected subnet IS the plan the operator returns to, and the id it carries
                // is its own — so this value comes from the REQUEST here, where the four routes
                // above take it from the port. Said rather than blended into their comment.
                | WriteRoute::EditSubnet => {
                    "/ipam?subnet=01900000-0000-7000-8000-0000000000aa".to_string()
                }
                // A release goes back to the innermost subnet the PORT found for the address, and
                // carries its confirmation in the URL (`HX-Redirect` shows no body).
                WriteRoute::Release => {
                    "/ipam?subnet=01900000-0000-7000-8000-0000000000aa&released=01900000-0000-7000-8000-0000000000bb"
                        .to_string()
                }
            };
            let redirect = response
                .headers()
                .get("hx-redirect")
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_string();
            assert_eq!(
                redirect, expected_redirect,
                "`{at}` does not send the browser back to the plan it changed"
            );
        }
        assert_eq!(
            shape_sentences.len(),
            WriteRoute::ALL.len(),
            "every route was probed"
        );
    }

    /// The two routes that RECEIVE a subnet id refuse the nil sentinel, at the route.
    #[tokio::test]
    async fn a_nil_subnet_id_is_refused_before_the_store() {
        for (route, body) in [
            (
                WriteRoute::Range,
                "subnet_id=00000000-0000-0000-0000-000000000000&first=192.0.2.10&last=192.0.2.20&policy=static&label=Office",
            ),
            (
                WriteRoute::Address,
                "subnet_id=00000000-0000-0000-0000-000000000000&addr=192.0.2.9&label=Office",
            ),
        ] {
            let port = FakePort::answering(Ok("unreached".to_string()));
            let (status, _) = drive(port.clone(), form_post_to(route, body)).await;
            assert_eq!(
                status,
                StatusCode::UNPROCESSABLE_ENTITY,
                "`{}` accepted the nil UUID",
                route.path()
            );
            assert!(
                port.asked.lock().expect("the log").is_empty(),
                "`{}` handed the nil sentinel to the store",
                route.path()
            );
        }
    }

    /// **AC2's own repair, which until the review was carried by NOTHING.**
    ///
    /// 🔴 `"ipam.refusal.subnet_still_holds"` occurred exactly ONCE in all of `crates/` — the mapper
    /// itself. The only test that fed `Constraint("foreign_key")` asserted that whatever key came
    /// back RESOLVES in both locales, never which key or which status. Measured by mutation at the
    /// review: turning the repaired 409 back into the 404 this story exists to remove left 700
    /// tests, `clippy --all-targets` and all ten gates green. *The behaviour was right and nothing
    /// would have noticed it regressing.*
    #[tokio::test]
    async fn a_populated_subnet_is_refused_by_name_rather_than_as_a_missing_one() {
        let port = FakePort::answering(Err(RepositoryError::Constraint("foreign_key")));
        let (status, body) = drive(
            port,
            form_post_to(
                WriteRoute::DeleteSubnet,
                &a_valid_body(WriteRoute::DeleteSubnet, "Office"),
            ),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::CONFLICT,
            "a subnet that still holds records is a CONFLICT with the plan's state, not a 404"
        );
        assert_eq!(
            body,
            key("ipam.refusal.subnet_still_holds"),
            "the operator was told their subnet is not in the plan while they were looking at it"
        );
        // 🔑 **THE CONTROL IS WHAT MAKES THE PAIR MEAN ANYTHING**: the same database refusal on a
        // DEFINITION really is a missing parent, and there the 404 is the honest answer. Without
        // this half the assertion above would be satisfied by answering 409 everywhere.
        let port = FakePort::answering(Err(RepositoryError::Constraint("foreign_key")));
        let (status, body) = drive(
            port,
            form_post_to(
                WriteRoute::Range,
                &a_valid_body(WriteRoute::Range, "Office"),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body, key("ipam.refusal.unknown_subnet"));
    }

    /// **The same rule, two gestures, two sentences** (Guy's decision 2, 2026-09-16).
    ///
    /// 🔴 The review's repair made an EDIT that abandons an address earn the delete's refusal — and
    /// with it the delete's SENTENCE, so an operator who had shrunk a range read *"deleting the
    /// range would leave them with nothing…"*. `ipam_refusal` took no route where its two
    /// neighbours, [`constraint_refusal`] and [`WriteRoute::unknown_record`], both do.
    #[test]
    fn the_edit_and_the_delete_do_not_share_a_refusal_sentence() {
        let deleting = ipam_refusal(
            &IpamError::RangeStillHoldsAddresses,
            WriteRoute::DeleteRange,
        );
        let editing = ipam_refusal(&IpamError::RangeStillHoldsAddresses, WriteRoute::EditRange);
        assert_ne!(
            deleting.key(),
            editing.key(),
            "one sentence for both gestures is how an operator who shrank a range was told what \
             deleting it would do"
        );
        for locale in ["en", "fr"] {
            let edit_sentence = rust_i18n::t!(editing.key(), locale = locale).to_string();
            assert_ne!(
                edit_sentence,
                editing.key(),
                "the edit's refusal has no `{locale}` translation"
            );
            // The needle is the DELETE's own phrase in each language, so this reds the day the two
            // keys are collapsed back into one.
            for describing_a_deletion in ["deleting the range", "supprimer la plage"] {
                assert!(
                    !edit_sentence.to_lowercase().contains(describing_a_deletion),
                    "the edit's refusal describes a deletion in `{locale}`: {edit_sentence}"
                );
            }
        }
    }

    /// **`WriteRoute::ALL` is a hand-written array, and this is the only opinion in the module that
    /// is not derived from it.**
    ///
    /// 🔴 The module doc claimed that adding a route without mounting it is an `error[E0004]`. It is
    /// not: `router_with` iterates `ALL`, [`WriteRoute::paths`] maps over `ALL`, `main.rs`'s
    /// ten-path premise is built from `ALL`, and this story's own coverage assertion compares
    /// against `ALL.len()` — *they all agree because none of them has a second opinion to disagree
    /// with*. A variant left out of `ALL` is mounted by nothing and answers 404 behind the auth
    /// layer's 401, which story 6b.2 measured is indistinguishable from a typo.
    ///
    /// ⚠️ **Its limit, stated rather than implied: this is a TRIPWIRE, not a barrier** (story 5.12's
    /// precedent). A ninth variant produces an `E0004` at every `match` in this file, so the
    /// developer is forced past `path`, `handler`, `malformed`, `success_status` and the rest — and
    /// forced past nothing that names `ALL`. If they do not write it here either, this stays green.
    /// What it does carry is a route DELETED from `ALL` while its variant survives, and the two
    /// lists drifting in length.
    #[test]
    fn every_variant_is_in_the_route_list() {
        let written_out = [
            WriteRoute::Subnet,
            WriteRoute::Range,
            WriteRoute::Address,
            WriteRoute::DeleteSubnet,
            WriteRoute::DeleteRange,
            WriteRoute::DeleteAddress,
            WriteRoute::EditRange,
            WriteRoute::EditAddress,
            WriteRoute::Release,
            WriteRoute::EditSubnet,
        ];
        for route in written_out {
            assert!(
                WriteRoute::ALL.contains(&route),
                "`{}` exists and is mounted by nothing: it is missing from `WriteRoute::ALL`, \
                 which is the list `router_with` iterates",
                route.path()
            );
        }
        assert_eq!(
            WriteRoute::ALL.len(),
            written_out.len(),
            "`WriteRoute::ALL` and this second list disagree about how many routes the plan has"
        );
    }

    /// A policy outside the four binding words is refused, and the four are accepted.
    #[tokio::test]
    async fn only_the_four_binding_policies_are_accepted() {
        for policy in opencmdb_core::ipam::IpPolicy::ALL {
            let port = FakePort::answering(Ok("id".to_string()));
            let body = format!(
                "subnet_id=01900000-0000-7000-8000-0000000000aa&first=192.0.2.10&last=192.0.2.20&policy={}&label=Office",
                policy.as_str()
            );
            let (status, _) = drive(port, form_post_to(WriteRoute::Range, &body)).await;
            assert_eq!(status, StatusCode::CREATED, "`{policy}` is a binding word");
        }
        // ⚠️ `Static ` with a trailing space is the PAD SPACE trap one layer up: the schema's own
        // `IN (...)` accepts it, so the route must not.
        for rejected in ["structural", "Static", "static ", ""] {
            let port = FakePort::answering(Ok("unreached".to_string()));
            let body = format!(
                "subnet_id=01900000-0000-7000-8000-0000000000aa&first=192.0.2.10&last=192.0.2.20&policy={rejected}&label=Office"
            );
            let (status, sentence) = drive(port, form_post_to(WriteRoute::Range, &body)).await;
            assert_eq!(
                status,
                StatusCode::UNPROCESSABLE_ENTITY,
                "`{rejected}` was accepted as a policy"
            );
            assert_eq!(sentence, key("ipam.refusal.unknown_policy"));
        }
    }

    /// The list the router is built from is the list the guard walks — asserted, because the whole
    /// point of `WriteRoute` is that a route cannot be declared and left unmounted.
    #[tokio::test]
    async fn every_declared_route_is_mounted() {
        for route in WriteRoute::ALL {
            let port = FakePort::answering(Ok("id".to_string()));
            let response = router_with(port)
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(route.path())
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .header(header::HOST, "opencmdb.example")
                        .header(header::ORIGIN, "https://opencmdb.example")
                        .body(Body::from("cidr=192.0.2.0/24&label=Office"))
                        .expect("a well-formed test request"),
                )
                .await
                .expect("the sub-router answers");
            assert_ne!(
                response.status(),
                StatusCode::NOT_FOUND,
                "`{}` is declared and not mounted",
                route.path()
            );
        }
    }

    /// The lock-wait cap on the plan's statements sits BELOW the handler's budget, so the server gives
    /// up before the browser is told to (the second review, Guy 2026-09-15).
    #[test]
    fn the_plan_lock_wait_cap_is_below_the_write_budget() {
        assert!(
            crate::ipam_repo::tests::PLAN_LOCK_WAIT_SECONDS < IPAM_WRITE_BUDGET.as_secs(),
            "a cap at or past the budget lets the budget drop a write that is still waiting, and that \
             write keeps its connection until the server gives up"
        );
    }

    /// 🔴 **A write the budget drops leaves NO ROW BEHIND** — the review measured the opposite.
    ///
    /// The subnet and address writes ran in autocommit. With the parent row held for 15 s,
    /// `POST /ipam/address` answered 503 *"Nothing was written"* at 5.002 s, no row existed right
    /// after, the row WAS there once the holder committed, and the retry answered 409. The statement
    /// had already reached the server when its future was dropped.
    ///
    /// 🔑 This drops the port's future directly with a short `timeout`, which is what the budget
    /// does after five seconds, and holds each write on a lock its own statement needs: the parent
    /// row for the address (its foreign-key check), and an uncommitted copy of the same CIDR for the
    /// subnet (its unique check). Then it releases the lock, gives a statement that outlived its
    /// future time to land, and counts. In autocommit both counts are 1; inside a transaction the
    /// dropped write is rolled back with its connection.
    ///
    /// ⚠️ **What it measures is ROWS, not locks** (the second review): `COUNT(*)` is a non-locking
    /// read, so it cannot tell a rolled-back row from one inside a transaction still open. Whether a
    /// dropped write keeps its locks was measured on the booted binary instead — gone once the holder
    /// released, and since the same review the server gives up after the statement's lock-wait cap.
    /// And the holder is released BEFORE any premise is asserted, so a failing premise cannot hand a
    /// connection inside a raw `START TRANSACTION` back to the pool.
    #[tokio::test]
    async fn a_write_dropped_while_waiting_leaves_no_row_behind() {
        const PARENT: &str = "t-drop-parent";
        const HOLDER: &str = "t-drop-holder";
        const HELD_BASE: &str = "100.064.031.000";
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = crate::ipam_repo::tests::ipam_fixture().await else {
            return;
        };
        let cleanup = || async {
            for id in [PARENT, HOLDER] {
                crate::ipam_repo::tests::forget_subnet(&pool, id).await;
            }
            sqlx::query("DELETE FROM ip_subnet WHERE base = ? AND prefix_len = 24")
                .bind(HELD_BASE)
                .execute(&pool)
                .await
                .expect("the probe's own CIDR");
        };
        cleanup().await;
        let parent = Subnet::new("100.64.30.0".parse().expect("an address"), 24).expect("a subnet");
        ipam_repo::insert_subnet(&pool, PARENT, parent, "the parent", 0)
            .await
            .expect("the parent row");
        let port = StoreIpamWrite::new(pool.clone());
        let patience = std::time::Duration::from_millis(500);
        let mut holder = pool.acquire().await.expect("a holder connection");

        // The ADDRESS: a holder takes the parent row, so the insert's foreign-key check waits.
        sqlx::query("START TRANSACTION")
            .execute(&mut *holder)
            .await
            .expect("begin");
        sqlx::query("SELECT id FROM ip_subnet WHERE id = ? FOR UPDATE")
            .bind(PARENT)
            .execute(&mut *holder)
            .await
            .expect("hold the parent row");
        let dropped = tokio::time::timeout(
            patience,
            port.define_address(
                PARENT.to_string(),
                "100.64.30.7".parse().expect("an address"),
                "dropped".to_string(),
            ),
        )
        .await;
        let address_premise = (dropped.is_err(), format!("{dropped:?}"));
        sqlx::query("COMMIT")
            .execute(&mut *holder)
            .await
            .expect("release the parent row");
        assert!(
            address_premise.0,
            "premise: the address write must still be waiting on the held row when it is dropped, \
             or this measures nothing: {}",
            address_premise.1
        );

        // The SUBNET: a holder inserts the same CIDR without committing, so the unique check waits.
        sqlx::query("START TRANSACTION")
            .execute(&mut *holder)
            .await
            .expect("begin");
        sqlx::query(
            "INSERT INTO ip_subnet (id, base, prefix_len, label) VALUES (?, ?, 24, 'holder')",
        )
        .bind(HOLDER)
        .bind(HELD_BASE)
        .execute(&mut *holder)
        .await
        .expect("hold the CIDR");
        let dropped = tokio::time::timeout(
            patience,
            port.define_subnet(
                Subnet::new("100.64.31.0".parse().expect("an address"), 24).expect("a subnet"),
                "dropped".to_string(),
                0,
            ),
        )
        .await;
        let subnet_premise = (dropped.is_err(), format!("{dropped:?}"));
        sqlx::query("ROLLBACK")
            .execute(&mut *holder)
            .await
            .expect("release the CIDR");
        drop(holder);
        assert!(
            subnet_premise.0,
            "premise: the subnet write must still be waiting on the held CIDR when it is dropped: {}",
            subnet_premise.1
        );

        // A statement that outlived its future lands now, if it is going to.
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let (addresses,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM ip_address WHERE subnet_id = ?")
                .bind(PARENT)
                .fetch_one(&pool)
                .await
                .expect("counting addresses");
        let (subnets,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM ip_subnet WHERE base = ? AND prefix_len = 24")
                .bind(HELD_BASE)
                .fetch_one(&pool)
                .await
                .expect("counting subnets");
        cleanup().await;
        assert_eq!(
            addresses, 0,
            "the address the budget dropped was written anyway — under a sentence that could not \
             say so"
        );
        assert_eq!(
            subnets, 0,
            "the subnet the budget dropped was written anyway — under a sentence that could not say \
             so"
        );
    }
}
