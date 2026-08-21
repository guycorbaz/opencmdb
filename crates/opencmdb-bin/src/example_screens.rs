//! The example screens: the inventory and the device record (story 6b.6).
//!
//! # Why a module of its own, and not more of [`crate::page`]
//!
//! ⚠️ `page.rs` stood at **1575 code lines of the 2000** the `file-size` gate allows, and story 6b.4
//! alone had added 533. `CLAUDE.md`'s rule is *split, not grown*, and a story that delivers two
//! screens is the one that must obey it rather than discover the gate. The boundary is not
//! arbitrary: **[`crate::page`] owns the screens backed by the store; this module owns the screens
//! backed by [`crate::example_data`]**, which is the same line Epic 6b's constraint 1 already draws.
//!
//! 🔑 It carries its OWN strings struct rather than borrowing `page`'s flat one. That keeps the
//! shared struct from growing by twenty fields for screens it does not render, and it means the two
//! halves cannot silently start depending on each other's copy.

use askama::Template;
use serde::Deserialize;

use crate::example_data::{self, DeviceKind, ExampleDevice};
use crate::state_vocabulary::{ObjectState, term_of};

/// What the query string may carry on a demonstration screen.
///
/// 🔑 One type for every such screen rather than one per screen: the router's loop is generic over
/// [`crate::screens::Screen`], and a per-screen extractor would mean leaving that loop — which is
/// what keeps the marker impossible to forget.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ScreenQuery {
    /// The device kind the inventory is filtered by, as [`DeviceKind::slug`] spells it.
    pub(crate) kind: Option<String>,
    /// The subnet the occupancy grid shows, as [`example_data::ExampleSubnet::slug`] spells it.
    ///
    /// ⚠️ **One type for every demonstration screen means `/devices?subnet=x` parses and is
    /// ignored**, and `/ipam?kind=x` likewise. That is accepted rather than overlooked: the router's
    /// loop is generic over [`crate::screens::Screen`], and a per-screen extractor would mean leaving
    /// that loop — which is what keeps the marker impossible to forget. `serde` ignores unknown
    /// fields here already, so the shared type widens nothing.
    pub(crate) subnet: Option<String>,
}

/// The copy these two screens need, resolved once into the operator's language.
///
/// 🔴 Resolved HERE and not in the template, because [`crate::example_data`] holds i18n KEYS rather
/// than sentences — see [`crate::example_data::ExampleDevice::role_key`] for the defect that taught
/// us the difference, which was found by looking at the screen and by nothing else.
pub(crate) struct ExampleStrings {
    /// The inventory's heading.
    devices_title: String,
    /// The accessible name of the filter bar.
    filter_label: String,
    /// The *all kinds* filter's label.
    filter_all: String,
    /// Column: the device's name.
    devices_name: String,
    /// Column: what it is.
    devices_kind: String,
    /// Column: the address.
    devices_ipv4: String,
    /// Column: the hardware address.
    devices_mac: String,
    /// Column: what it is for.
    devices_role: String,
    /// Column: where reconciliation stands.
    devices_state: String,
    /// Column: when it was last observed.
    devices_seen: String,
    /// What the inventory says when a filter matches nothing.
    devices_none: String,
    /// The second section's heading — sightings the engine did not place.
    unplaced_title: String,
    /// Column: why a sighting was not placed.
    unplaced_reason: String,
    /// The record's *field by field* heading.
    record_fields: String,
    /// Column: the declared value.
    record_declared: String,
    /// Column: the observed value.
    record_observed: String,
    /// The record's *Hosted here* heading (FR29).
    record_hosted: String,
    /// What the record says when the device hosts nothing.
    record_hosted_none: String,
    /// The record's composite-identity heading.
    record_identity: String,
    /// The record's observation-history heading (FR37).
    record_history: String,
    /// The link back to the inventory.
    record_back: String,
    /// What the record says for a slug no device carries.
    record_unknown: String,
}

/// Resolve the copy both screens share.
fn example_strings() -> ExampleStrings {
    ExampleStrings {
        devices_title: rust_i18n::t!("devices.title").to_string(),
        filter_label: rust_i18n::t!("devices.filter_label").to_string(),
        filter_all: rust_i18n::t!("devices.filter_all").to_string(),
        devices_name: rust_i18n::t!("devices.name").to_string(),
        devices_kind: rust_i18n::t!("devices.kind").to_string(),
        devices_ipv4: rust_i18n::t!("devices.ipv4").to_string(),
        devices_mac: rust_i18n::t!("devices.mac").to_string(),
        devices_role: rust_i18n::t!("devices.role").to_string(),
        devices_state: rust_i18n::t!("devices.state").to_string(),
        devices_seen: rust_i18n::t!("devices.seen").to_string(),
        devices_none: rust_i18n::t!("devices.none").to_string(),
        unplaced_title: rust_i18n::t!("unplaced.title").to_string(),
        unplaced_reason: rust_i18n::t!("unplaced.reason").to_string(),
        record_fields: rust_i18n::t!("record.fields").to_string(),
        record_declared: rust_i18n::t!("record.declared").to_string(),
        record_observed: rust_i18n::t!("record.observed").to_string(),
        record_hosted: rust_i18n::t!("record.hosted").to_string(),
        record_hosted_none: rust_i18n::t!("record.hosted_none").to_string(),
        record_identity: rust_i18n::t!("record.identity").to_string(),
        record_history: rust_i18n::t!("record.history").to_string(),
        record_back: rust_i18n::t!("record.back").to_string(),
        record_unknown: rust_i18n::t!("record.unknown").to_string(),
    }
}

/// A state as the operator reads it: the word, its qualifier, and the class that colours it.
///
/// 🔑 The word and the modifier come from the same [`ObjectState`], so a pill cannot carry one
/// state's colour over another state's word.
pub(crate) struct StateView {
    /// The glossary term, resolved.
    term: String,
    /// The qualifier after it, already prefixed with the separator, or empty.
    ///
    /// ⚠️ Guy's arbitration: *"Écart · 2 champs"* is the word *écart* qualified, not a sixth term.
    qualifier: String,
    /// The CSS modifier — `statepill-*`, never a literal.
    modifier: &'static str,
}

impl StateView {
    /// Build the view of a state and its optional qualifier key.
    fn new(state: ObjectState, qualifier_key: Option<&'static str>) -> Self {
        let rendered = rust_i18n::t!(state.key()).to_string();
        StateView {
            // 🔑 `term_of` and not the raw string: if a translation ever carries the separator, the
            // term is what precedes it — the same rule the glossary check applies, applied by the
            // code that renders rather than only by the code that checks.
            term: term_of(&rendered).to_string(),
            qualifier: qualifier_key
                .map(|key| {
                    format!(
                        "{}{}",
                        crate::state_vocabulary::QUALIFIER_SEPARATOR,
                        rust_i18n::t!(key)
                    )
                })
                .unwrap_or_default(),
            modifier: state.modifier(),
        }
    }
}

/// One filter in the inventory's bar.
pub(crate) struct FilterView {
    /// What the query string carries for it — empty for *all kinds*.
    slug: &'static str,
    /// Its label, resolved.
    label: String,
    /// Whether it is the one in force.
    active: bool,
}

/// One row of the example inventory.
pub(crate) struct DeviceRow {
    /// The stable slug, which is also the record's address.
    id: &'static str,
    name: &'static str,
    kind: String,
    ipv4: &'static str,
    mac: &'static str,
    role: String,
    state: StateView,
    last_seen: &'static str,
}

/// One example sighting the engine did not place, with its reason resolved.
pub(crate) struct SightingRow {
    ipv4: &'static str,
    /// The hardware address, or the typographic placeholder when the sighting gave none.
    ///
    /// 🔑 The absence is resolved HERE and not stored as a sentinel in the dataset: `example_data`
    /// holds facts, this struct holds what is displayed.
    mac: String,
    reason: String,
}

/// The inventory screen's body.
#[derive(Template)]
#[template(path = "_devices_example.html")]
struct Inventory {
    filters: Vec<FilterView>,
    devices: Vec<DeviceRow>,
    sightings: Vec<SightingRow>,
    s: ExampleStrings,
}

/// Build one inventory row from a dataset entry.
fn row_of(device: &ExampleDevice) -> DeviceRow {
    DeviceRow {
        id: device.id,
        name: device.name,
        kind: rust_i18n::t!(device.kind.label_key()).to_string(),
        ipv4: device.ipv4,
        mac: device.mac,
        role: rust_i18n::t!(device.role_key).to_string(),
        state: StateView::new(device.state, device.qualifier_key),
        last_seen: device.last_seen,
    }
}

/// Render the example inventory, filtered by kind when the query names one.
///
/// ⚠️ An unrecognised `kind` narrows to nothing rather than silently showing everything: a filter
/// that ignores its input is the shape story 6b.4's review caught on `?sort=`, and *"showing
/// everything"* would tell the operator their filter matched when it did not.
///
/// # Panics
///
/// Never in practice: the template is compiled into the binary by askama and its inputs are
/// constants, so a failure here would be a compile error rather than a run-time one.
pub(crate) fn inventory_body(query: &ScreenQuery) -> String {
    let selected = query.kind.as_deref().filter(|slug| !slug.is_empty());
    let s = example_strings();
    let mut filters = vec![FilterView {
        slug: "",
        // The struct's own field, not a second resolution of the same key: two readers of one
        // fact drift, which is the shape story 6b.2's M12 shipped.
        label: s.filter_all.clone(),
        active: selected.is_none(),
    }];
    filters.extend(DeviceKind::ALL.iter().map(|kind| FilterView {
        slug: kind.slug(),
        label: rust_i18n::t!(kind.label_key()).to_string(),
        active: selected == Some(kind.slug()),
    }));
    Inventory {
        filters,
        devices: example_data::devices()
            .iter()
            .filter(|device| selected.is_none_or(|slug| device.kind.slug() == slug))
            .map(row_of)
            .collect(),
        sightings: example_data::unplaced_sightings()
            .into_iter()
            .map(|sighting| SightingRow {
                ipv4: sighting.ipv4,
                // An em dash, locale-neutral on purpose: a typographic placeholder for an absent
                // value, not a word, so it needs no key and reads the same in both languages.
                mac: sighting.mac.unwrap_or("—").to_string(),
                reason: rust_i18n::t!(sighting.reason_key).to_string(),
            })
            .collect(),
        s,
    }
    .render()
    .expect("the inventory template and its struct are compiled together")
}

/// One compared field on the record.
pub(crate) struct FieldRow {
    label: String,
    /// The declared value, or the placeholder when nothing is declared for it.
    declared: String,
    /// The observed value, or the placeholder when nothing answered.
    observed: String,
    state: StateView,
}

/// One object the device hosts — FR29, one containment hop.
pub(crate) struct HostedRow {
    name: &'static str,
    kind: String,
}

/// One component of the composite identity.
pub(crate) struct IdentityRow {
    label: String,
    value: &'static str,
}

/// One line of the observation history — FR37.
pub(crate) struct HistoryRow {
    when: &'static str,
    what: String,
}

/// The device record's body.
#[derive(Template)]
#[template(path = "_device_record.html")]
struct Record {
    name: &'static str,
    kind: String,
    ipv4: &'static str,
    state: StateView,
    fields: Vec<FieldRow>,
    hosted: Vec<HostedRow>,
    identity: Vec<IdentityRow>,
    history: Vec<HistoryRow>,
    s: ExampleStrings,
}

/// What a slug no device carries gets: a page that says so.
#[derive(Template)]
#[template(path = "_device_unknown.html")]
struct UnknownDevice {
    s: ExampleStrings,
}

/// Render the record of the device a slug addresses, or `None` when no device carries it.
///
/// # Panics
///
/// Never in practice, for the reason given on [`inventory_body`].
pub(crate) fn record_body(id: &str) -> Option<String> {
    let device = example_data::device_by_id(id)?;
    // A typographic placeholder for an absent value — locale-neutral, therefore not a key.
    let dash = "—";
    Some(
        Record {
            name: device.name,
            kind: rust_i18n::t!(device.kind.label_key()).to_string(),
            ipv4: device.ipv4,
            state: StateView::new(device.state, device.qualifier_key),
            fields: device
                .fields
                .iter()
                .map(|field| {
                    // 🔴 The flag, not a heuristic: the *role* row's values are KEYS and every
                    // other row's are facts. The first draft printed the key — see
                    // `no_i18n_key_reaches_the_screen` for the two ways that stayed green.
                    let show = |value: Option<&'static str>| match (value, field.values_are_keys) {
                        (None, _) => dash.to_string(),
                        (Some(value), true) => rust_i18n::t!(value).to_string(),
                        (Some(value), false) => value.to_string(),
                    };
                    FieldRow {
                        label: rust_i18n::t!(field.label_key).to_string(),
                        declared: show(field.declared),
                        // 🔑 Two observed values on one line when two sources disagree. A
                        // `Conflict` row whose observed column held an em dash was measured
                        // indistinguishable from an absence — the blind review layer saw it in the
                        // constants, and the screen exists to show what a state LOOKS like.
                        observed: match field.observed_alt {
                            Some(other) => {
                                format!("{} ≠ {}", show(field.observed), show(Some(other)))
                            }
                            None => show(field.observed),
                        },
                        state: StateView::new(field.state, None),
                    }
                })
                .collect(),
            hosted: device
                .hosted
                .iter()
                .map(|hosted| HostedRow {
                    name: hosted.name,
                    kind: rust_i18n::t!(hosted.kind_key).to_string(),
                })
                .collect(),
            identity: device
                .identity
                .iter()
                .map(|part| IdentityRow {
                    label: rust_i18n::t!(part.label_key).to_string(),
                    value: part.value,
                })
                .collect(),
            history: device
                .history
                .iter()
                .map(|line| HistoryRow {
                    when: line.when,
                    what: rust_i18n::t!(line.what_key).to_string(),
                })
                .collect(),
            s: example_strings(),
        }
        .render()
        .expect("the record template and its struct are compiled together"),
    )
}

/// The body a slug no device carries gets.
///
/// # Panics
///
/// Never in practice, for the reason given on [`inventory_body`].
pub(crate) fn unknown_device_body() -> String {
    UnknownDevice {
        s: example_strings(),
    }
    .render()
    .expect("the unknown-device template and its struct are compiled together")
}

/// The copy the applications screen needs, resolved once into the operator's language.
///
/// 🔑 Its own struct, for the reason this module's doc gives: a shared flat struct would grow by a
/// dozen fields for screens it does not render, and the halves would start depending on each other's
/// copy.
pub(crate) struct AppsStrings {
    /// The screen's heading.
    title: String,
    /// The sentence AC2 exists for — *nothing will ever observe them*.
    lede: String,
    /// Column: the application.
    name: String,
    /// Column: where it runs.
    host: String,
    /// Column: the documented version.
    declared: String,
    /// Column: the version the host reported.
    observed: String,
    /// Column: who answers for it.
    owner: String,
    /// Column: how critical it is.
    criticality: String,
    /// What the *host* cell says when the application runs outside the perimeter.
    no_host: String,
}

/// Resolve the applications screen's copy.
fn apps_strings() -> AppsStrings {
    AppsStrings {
        title: rust_i18n::t!("apps.title").to_string(),
        lede: rust_i18n::t!("apps.lede").to_string(),
        name: rust_i18n::t!("apps.name").to_string(),
        host: rust_i18n::t!("apps.host").to_string(),
        declared: rust_i18n::t!("apps.declared").to_string(),
        observed: rust_i18n::t!("apps.observed").to_string(),
        owner: rust_i18n::t!("apps.owner").to_string(),
        criticality: rust_i18n::t!("apps.criticality").to_string(),
        no_host: rust_i18n::t!("apps.no_host").to_string(),
    }
}

/// One row of the example application inventory, resolved for rendering.
pub(crate) struct AppRow {
    name: &'static str,
    /// The host's name, or the sentence that says it runs outside the perimeter.
    host: String,
    /// The record's address when there is a host, so the row links where the inventory does.
    host_id: Option<&'static str>,
    declared: &'static str,
    /// The observed version, or the typographic placeholder for *nothing evaluated it*.
    observed: String,
    owner: &'static str,
    /// The criticality, RESOLVED — it is a key (Guy, 2026-08-20), unlike [`AppRow::owner`].
    criticality: String,
}

/// The applications screen's body.
#[derive(Template)]
#[template(path = "_apps_example.html")]
struct Apps {
    apps: Vec<AppRow>,
    s: AppsStrings,
}

/// Render the example application inventory.
///
/// ⚠️ The query is taken and deliberately unused: this screen offers no filter, and taking the
/// parameter keeps every demonstration screen on one signature. *Rust does not lint an unused
/// function parameter* — story 6b.6 threaded a query through three signatures and left the arm
/// passing `Default::default()`, so the underscore here is a statement, not a habit.
///
/// # Panics
///
/// Never in practice, for the reason given on [`inventory_body`].
pub(crate) fn apps_body(_query: &ScreenQuery) -> String {
    let s = apps_strings();
    let devices = example_data::devices();
    Apps {
        apps: example_data::apps()
            .into_iter()
            .map(|app| AppRow {
                name: app.name,
                host: match app.host {
                    Some(slug) => devices
                        .iter()
                        .find(|device| device.id == slug)
                        .map(|device| device.name.to_string())
                        // Unreachable while the guard below holds; a slug naming no device is a
                        // dataset defect, and saying so beats rendering the raw slug.
                        .unwrap_or_else(|| slug.to_string()),
                    None => s.no_host.clone(),
                },
                host_id: app.host,
                declared: app.declared_version,
                // The same em dash the inventory uses for an absent MAC: locale-neutral, therefore
                // no key, and one placeholder across the product.
                observed: app.observed_version.unwrap_or("—").to_string(),
                owner: app.owner,
                criticality: rust_i18n::t!(app.criticality_key).to_string(),
            })
            .collect(),
        s,
    }
    .render()
    .expect("the applications template and its struct are compiled together")
}

/// The copy the IPAM screen needs, resolved once.
pub(crate) struct IpamStrings {
    /// The screen's heading.
    title: String,
    /// *One cell, one address.*
    lede: String,
    /// The accessible name of the subnet selector.
    selector_label: String,
    /// The accessible name of the grid itself.
    grid_label: String,
    /// The *next free address* panel's heading.
    next_free: String,
    /// The *address conflict* panel's heading — qualified on purpose (Guy, 2026-08-20).
    conflict_title: String,
    /// The sentence explaining the conflict.
    conflict_lede: String,
    /// What the row of the device declared at the address says.
    conflict_declared: String,
    /// What the row of the device also answering says.
    conflict_observed: String,
    /// The link into the triage.
    conflict_link: String,
    /// What the screen says for a subnet slug nothing names.
    unknown_subnet: String,
    /// The occupancy line, already assembled with its three counts.
    occupancy: String,
}

/// One cell of the occupancy grid.
pub(crate) struct CellView {
    /// Its accessible name — the address and its state, which AC1 requires **per cell**.
    label: String,
    /// The CSS modifier, from [`example_data::CellState::modifier`].
    modifier: &'static str,
}

/// One tab of the subnet selector.
pub(crate) struct SubnetTab {
    slug: &'static str,
    label: String,
    active: bool,
}

/// One line of the conflict panel.
pub(crate) struct ConflictLine {
    /// The device's name.
    name: &'static str,
    /// Its record's address.
    id: &'static str,
    /// What is said about it — declared here, or seen with a lease.
    note: String,
}

/// The IPAM screen's body.
#[derive(Template)]
#[template(path = "_ipam_example.html")]
struct Ipam {
    tabs: Vec<SubnetTab>,
    /// `None` when the query names a subnet that does not exist — see [`ipam_body`].
    cells: Option<Vec<CellView>>,
    /// The four state words, in [`example_data::CellState`]'s order.
    ///
    /// 🔑 **Words only — the legend's CSS classes are LITERALS in the template**, on purpose: they
    /// are what makes the cells' Rust-chosen modifiers visible to the stylesheet guard. Carrying the
    /// modifier here as well would let a tidy-up render the legend from it, which is exactly the
    /// change the validation measured turning the guard green over a colourless grid.
    legend: Vec<String>,
    next_free: String,
    conflict: Vec<ConflictLine>,
    conflict_ipv4: &'static str,
    s: IpamStrings,
}

/// Render the example subnet occupancy.
///
/// 🔴 **An unrecognised `subnet` shows NO grid and says so**, which is the policy the sibling screen
/// already states: [`inventory_body`]'s doc reads *"an unrecognised `kind` narrows to nothing rather
/// than silently showing everything: a filter that ignores its input is the shape story 6b.4's review
/// caught on `?sort=`"*. The validation's prototype silently served the first subnet, which would
/// have shipped **two screens with opposite policies for one gesture**.
///
/// # Panics
///
/// Never in practice, for the reason given on [`inventory_body`].
pub(crate) fn ipam_body(query: &ScreenQuery) -> String {
    let subnets = example_data::subnets();
    let asked = query.subnet.as_deref().filter(|slug| !slug.is_empty());
    let selected = match asked {
        Some(slug) => example_data::subnet_by_slug(slug),
        None => example_data::subnets().into_iter().next(),
    };
    let conflict = example_data::address_conflict();
    let devices = example_data::devices();
    let name_of = |slug: &str| -> &'static str {
        devices
            .iter()
            .find(|device| device.id == slug)
            .map(|device| device.name)
            // Unreachable while `the_conflict_panel_names_devices_that_exist` holds.
            .unwrap_or("—")
    };

    let occupancy = match &selected {
        Some(subnet) => {
            let (used, reserved, free) = subnet.occupancy();
            rust_i18n::t!(
                "ipam.occupancy",
                used = used.to_string(),
                reserved = reserved.to_string(),
                free = free.to_string()
            )
            .to_string()
        }
        None => String::new(),
    };
    let next_free = match &selected {
        Some(subnet) => match subnet.next_free() {
            Some(octet) => format!("{}.{octet}", subnet.prefix),
            None => rust_i18n::t!("ipam.next_free_none").to_string(),
        },
        None => String::new(),
    };

    let s = IpamStrings {
        title: rust_i18n::t!("ipam.title").to_string(),
        lede: rust_i18n::t!("ipam.lede").to_string(),
        selector_label: rust_i18n::t!("ipam.selector_label").to_string(),
        grid_label: rust_i18n::t!("ipam.grid_label").to_string(),
        next_free: rust_i18n::t!("ipam.next_free").to_string(),
        conflict_title: rust_i18n::t!("ipam.conflict_title").to_string(),
        conflict_lede: rust_i18n::t!("ipam.conflict_lede").to_string(),
        conflict_declared: rust_i18n::t!("ipam.conflict_declared").to_string(),
        conflict_observed: rust_i18n::t!("ipam.conflict_observed").to_string(),
        conflict_link: rust_i18n::t!("ipam.conflict_link").to_string(),
        unknown_subnet: rust_i18n::t!("ipam.unknown_subnet").to_string(),
        occupancy,
    };

    Ipam {
        tabs: subnets
            .iter()
            .map(|subnet| SubnetTab {
                slug: subnet.slug,
                label: format!("{} · {}", subnet.cidr, subnet.name),
                active: selected
                    .as_ref()
                    .is_some_and(|chosen| chosen.slug == subnet.slug),
            })
            .collect(),
        cells: selected.as_ref().map(|subnet| {
            (0..=255u8)
                .map(|octet| {
                    let state = subnet.state_of(octet);
                    CellView {
                        // 🔑 The address AND its state, per cell — AC1's literal requirement. The
                        // mock puts one label on the container and a `title` on each cell, which
                        // names nothing to a screen reader.
                        label: format!(
                            "{}.{octet} · {}",
                            subnet.prefix,
                            rust_i18n::t!(state.label_key())
                        ),
                        modifier: state.modifier(),
                    }
                })
                .collect()
        }),
        legend: [
            example_data::CellState::Used,
            example_data::CellState::Reserved,
            example_data::CellState::Free,
            example_data::CellState::Structural,
        ]
        .into_iter()
        .map(|state| rust_i18n::t!(state.label_key()).to_string())
        .collect(),
        next_free,
        conflict: vec![
            ConflictLine {
                name: name_of(conflict.declared_device),
                id: conflict.declared_device,
                note: s.conflict_declared.clone(),
            },
            ConflictLine {
                name: name_of(conflict.observed_device),
                id: conflict.observed_device,
                note: format!(
                    "{} · {}",
                    s.conflict_observed,
                    rust_i18n::t!("ipam.conflict_lease", at = conflict.lease_seen)
                ),
            },
        ],
        conflict_ipv4: conflict.ipv4,
        s,
    }
    .render()
    .expect("the IPAM template and its struct are compiled together")
}

/// The copy the alert list needs, resolved once.
pub(crate) struct AlertStrings {
    /// The screen's heading.
    title: String,
    /// The sentence saying what this list is and is not.
    lede: String,
    /// Column: what happened.
    what: String,
    /// Column: what it is about.
    subject: String,
    /// Column: when.
    when: String,
}

/// One rendered alert row.
pub(crate) struct AlertRow {
    what: String,
    subject: &'static str,
    when: &'static str,
}

/// The alert list's body.
#[derive(Template)]
#[template(path = "_alerts_example.html")]
struct Alerts {
    alerts: Vec<AlertRow>,
    s: AlertStrings,
}

/// Render the example alert list — Epic 16's frame.
///
/// ⚠️ The query is taken and unused, for the reason given on [`apps_body`].
///
/// # Panics
///
/// Never in practice, for the reason given on [`inventory_body`].
pub(crate) fn alerts_body(_query: &ScreenQuery) -> String {
    Alerts {
        alerts: example_data::alerts()
            .into_iter()
            .map(|alert| AlertRow {
                what: rust_i18n::t!(alert.what_key).to_string(),
                subject: alert.subject,
                when: alert.when,
            })
            .collect(),
        s: AlertStrings {
            title: rust_i18n::t!("alerts.title").to_string(),
            lede: rust_i18n::t!("alerts.lede").to_string(),
            what: rust_i18n::t!("alerts.what").to_string(),
            subject: rust_i18n::t!("alerts.subject").to_string(),
            when: rust_i18n::t!("alerts.when").to_string(),
        },
    }
    .render()
    .expect("the alerts template and its struct are compiled together")
}

/// One rendered commissioning step.
pub(crate) struct StepRow {
    /// Its ordinal.
    number: &'static str,
    /// Its title.
    title: String,
    /// The line under it.
    detail: String,
    /// Its status word.
    status: String,
}

/// The commissioning screen's copy.
struct CommissioningStrings {
    title: String,
    lede: String,
    baseline_title: String,
    baseline_total: String,
    baseline_consistent: String,
    baseline_divergent: String,
    baseline_action: String,
    baseline_threshold: String,
    /// The *à venir* badge, from the SAME key `/triage` and `/diagnostic` use — one vocabulary for
    /// one state, which is what the glossary's *one term, one translation* rule means in the UI.
    gesture_badge: String,
}

/// The commissioning body.
#[derive(Template)]
#[template(path = "_commissioning_example.html")]
struct Commissioning {
    steps: Vec<StepRow>,
    total: u32,
    consistent: u32,
    divergent: u32,
    s: CommissioningStrings,
}

/// Render the example commissioning walk-through and its baselining block — Epic 9's frame.
///
/// 🔴 **The primary control is NOT live and is labelled**, through the same mechanism story 6b.4b
/// built for `/triage`: adopting a baseline writes to `declared_attribute` for every consistent
/// object, which is Epic 9's, and a live-looking button here would be a promise.
///
/// ⚠️ The query is taken and unused, for the reason given on [`apps_body`].
///
/// # Panics
///
/// Never in practice, for the reason given on [`inventory_body`].
pub(crate) fn commissioning_body(_query: &ScreenQuery) -> String {
    let baseline = example_data::commissioning_baseline();
    Commissioning {
        steps: example_data::commissioning_steps()
            .into_iter()
            .map(|step| StepRow {
                number: step.number,
                title: rust_i18n::t!(step.title_key).to_string(),
                detail: rust_i18n::t!(step.detail_key).to_string(),
                status: rust_i18n::t!(step.status_key).to_string(),
            })
            .collect(),
        total: baseline.total,
        consistent: baseline.consistent,
        divergent: baseline.divergent,
        s: CommissioningStrings {
            title: rust_i18n::t!("commissioning.title").to_string(),
            lede: rust_i18n::t!("commissioning.lede").to_string(),
            baseline_title: rust_i18n::t!("commissioning.baseline").to_string(),
            baseline_total: rust_i18n::t!("commissioning.total").to_string(),
            baseline_consistent: rust_i18n::t!("commissioning.consistent").to_string(),
            baseline_divergent: rust_i18n::t!("commissioning.divergent").to_string(),
            baseline_action: rust_i18n::t!("commissioning.action").to_string(),
            baseline_threshold: rust_i18n::t!("commissioning.threshold").to_string(),
            gesture_badge: rust_i18n::t!("gesture.badge").to_string(),
        },
    }
    .render()
    .expect("the commissioning template and its struct are compiled together")
}

/// Every word of `html`'s visible TEXT that looks like an i18n key rather than a translation.
///
/// # Why this is a shared helper since story 6b.7
///
/// 🔴 The guard that used it was an **ENUMERATION of pages** — `/devices`, the eight records, the
/// unknown page — so the two screens story 6b.7 adds were outside its population entirely. Measured
/// at that story's validation: a `criticality_key` naming a key that does not exist rendered the key
/// name on `/apps` with **zero tests red**, while the control (the same defect on a device's
/// `role_key`) reddened two. *rust-i18n renders an unknown key verbatim, and
/// `every_key_carries_both_locales` asks only whether the keys IN `app.yml` have two languages — a
/// key absent from the file is not in its population at all.*
///
/// 🔑 So the detector lives here and `main.rs`'s route-table guard applies it to the real HTTP body
/// of **every** screen. This module keeps its own caller for the record pages, which no route in
/// `Screen::ALL` reaches.
///
/// ⚠️ **Case-INSENSITIVE, and the widening is story 6b.6's code review.** It required every segment
/// to be lowercase, so `Zorp.Kind.bogus` went through unseen. The discriminator against false
/// positives is **two segments carrying a letter**, not the case: it separates `example.role.storage`
/// from a version like `v0.1.1` (one lettered segment) and from an address like `192.0.2.10` (none),
/// both of which are legitimate text on these very screens.
#[cfg(test)]
pub(crate) fn key_names_in_text(html: &str) -> Vec<String> {
    fn keyish(word: &str) -> bool {
        let word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '_');
        let parts: Vec<&str> = word.split('.').collect();
        parts.len() >= 2
            && parts.iter().all(|part| {
                !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            })
            && parts
                .iter()
                .filter(|part| part.chars().any(|c| c.is_ascii_alphabetic()))
                .count()
                >= 2
    }

    visible_text(html)
        .split_whitespace()
        .filter(|word| keyish(word))
        .map(str::to_string)
        .collect()
}

/// What the operator actually reads: `html` with every tag removed.
///
/// 🔴 **Separated from [`key_names_in_text`] because a premise assertion was measured counting the
/// wrong thing.** The guard below accumulated `html.split_whitespace().count()` — raw markup, so
/// every `href`, `class` and tag name counted as an inspected word — under a message claiming
/// *"rendered words were inspected"*. Its floor was then satisfied by markup whatever the page said.
/// Found by the review layer that had the diff and nothing else. *A counter that stops measuring
/// what its message names is a floor nobody re-reads*, which is the family this very file's
/// neighbouring guard warns about.
///
/// ⚠️ Tag-depth and not a parser: it is a test helper over templates this repository controls, and
/// it must stay conservative — text it wrongly discards is text this guard stops checking.
#[cfg(test)]
pub(crate) fn visible_text(html: &str) -> String {
    // Only the TEXT: an `href`, a `class` and an Askama comment legitimately carry dotted tokens,
    // and the operator reads none of them.
    let mut text = String::new();
    let mut depth = 0_usize;
    for c in html.chars() {
        match c {
            // 🔴 **A SPACE AT EVERY TAG BOUNDARY, and it took a live database to find out why.**
            // Without it the extractor joins across tags with nothing between them, so
            // `<p>…from an example.</p><p>No source is configured…` becomes `…example.No source…`
            // and `example.No` reads as a dotted i18n key. Story 6b.8's `/sources` reddened the
            // route-level guard on exactly that — **a FALSE POSITIVE manufactured by the helper**,
            // on copy that is correct.
            // 🔑 The cost of the fix is nil and the cost of the bug was a guard that cries wolf:
            // *a check that fails for the wrong reason is worth nothing*, and one that fails often
            // enough for the wrong reason gets deleted by whoever meets it next.
            '<' => {
                if depth == 0 {
                    text.push(' ');
                }
                depth += 1;
            }
            '>' => depth = depth.saturating_sub(1),
            _ if depth == 0 => text.push(c),
            _ => {}
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_vocabulary::{BINDING_STATE_AXIS, QUALIFIER_SEPARATOR};
    use std::collections::BTreeSet;

    /// Every `class="…"` literal a template names, with any Askama expression removed.
    fn body_of(query: &ScreenQuery) -> String {
        inventory_body(query)
    }

    /// 🔴 **Every state word this module can SERVE is a term of the binding glossary — in both
    /// languages, and read off the RENDERED page.**
    ///
    /// # What this adds to its namespace-derived twin, and what defeats it
    ///
    /// [`crate::state_vocabulary`]'s check compares the enum to the table. This one compares **what
    /// was served**. Story 6b.6's validation built it and measured **four ways it is worth
    /// nothing**, each closed here:
    ///
    /// 1. the population was **EMPTY** — the only screen rendering state words was `/triage`, which
    ///    is skipped without a database, so the guard was green and measured nothing. *The example
    ///    inventory is what gives it a non-empty population on a machine with no database*, and the
    ///    count assertion below is what keeps that true;
    /// 2. it resolved in **English** only, because the test process never sets a locale — so a
    ///    French-only glossary would have reddened on every word. ⚠️ That is **not** closed by
    ///    setting the locale here: `set_locale` is process-wide and this story removed the one test
    ///    that called it after measuring a reproducible race. The French side is carried by the
    ///    namespace-derived twin, which resolves explicitly and renders nothing;
    /// 3. a wrapping `<strong>` inside the pill defeated the extractor — so the extraction takes
    ///    the pill's whole text and strips tags rather than stopping at the first `<`;
    /// 4. 🔴 **AC1's own deliverable defeated it**: a filter bar naming state words with no class
    ///    at all left it green. That one is NOT closed and cannot be — see the limit below.
    ///
    /// ⚠️ **The limit, written rather than implied** (story 5.12's narrowing, fourth application):
    /// this is a **tripwire against a state word rendered through [`StateView`], never a barrier
    /// against a word typed into a template**. Its namespace-derived twin is blind to any literal.
    /// **Neither form alone is complete, and both were measured defeated.**
    #[test]
    fn every_state_word_served_is_a_term_of_the_binding_glossary() {
        fn pill_terms(html: &str) -> Vec<String> {
            html.split(r#"<span class="statepill "#)
                .skip(1)
                .filter_map(|tail| tail.split_once('>').map(|(_, rest)| rest))
                .filter_map(|rest| rest.split_once("</span>").map(|(text, _)| text))
                .map(|text| {
                    // Strip any nested tag: a `<strong>` around the word was measured defeating an
                    // extractor that stopped at the first `<`.
                    let mut out = String::new();
                    let mut depth = 0_usize;
                    for c in text.chars() {
                        match c {
                            '<' => depth += 1,
                            '>' => depth = depth.saturating_sub(1),
                            _ if depth == 0 => out.push(c),
                            _ => {}
                        }
                    }
                    term_of(out.trim()).to_string()
                })
                .collect()
        }

        // ⚠️ **The AMBIENT locale, and `set_locale` is deliberately NOT called.** It is
        // process-wide, and story 6b.6 removed the one test that used it after measuring a
        // reproducible race with its control (18 reds in 60 at two threads, 0 in 30 at one) —
        // adding one back here to read the other language would reintroduce exactly that.
        // 🔑 **The other language is covered by the namespace-derived twin**, which resolves with
        // an explicit locale and needs no render. Splitting the two properties across the two
        // guards is what lets both stay race-free; neither alone is complete, which the module doc
        // already says for a different reason.
        let permitted: Vec<String> = BINDING_STATE_AXIS
            .iter()
            .map(|row| row.0.to_string())
            .collect();
        let mut served = 0_usize;
        let mut pages = vec![body_of(&ScreenQuery::default())];
        for device in example_data::devices() {
            pages.push(record_body(device.id).expect("every listed device has a record"));
        }
        for html in &pages {
            for term in pill_terms(html) {
                assert!(
                    permitted.contains(&term.to_lowercase()),
                    "{term:?} reached the operator and the binding glossary's state axis carries \
                     {permitted:?} — a state word not in the table is a word INTRODUCED rather \
                     than registered, which is what AC2 forbids"
                );
                served += 1;
            }
        }
        assert!(
            served >= 20,
            "the premise: at least twenty state words were actually SERVED ({served} seen). A \
             guard over an empty population is green and measures nothing — which is exactly what \
             this one did before the example inventory carried states"
        );
    }

    /// ⚠️ A qualifier reaches the operator, and it is NOT a term.
    ///
    /// 🔑 Guy's arbitration of 2026-08-19 is only worth something if the product actually renders
    /// one: an exact-membership check would red on the mock's own copy, and this asserts that the
    /// case it was arbitrated for is live rather than hypothetical.
    #[test]
    fn a_qualified_state_reaches_the_screen_and_stays_one_term() {
        let html = body_of(&ScreenQuery::default());
        assert!(
            html.contains(QUALIFIER_SEPARATOR),
            "no state on the inventory carries a qualifier, so Guy's arbitration guards nothing"
        );
    }

    /// 🔴 **Every `statepill-*` modifier is defined in the stylesheet — because the generic guard
    /// structurally cannot see them.**
    ///
    /// `every_class_a_template_names_is_defined_in_the_stylesheet` skips any `class="…"` containing
    /// a brace, and the pill's class is `class="statepill {{ … }}"`. Story 6b.4b registered that as
    /// a general limit; story 6b.6's validation measured it **live on this widget, with a control**:
    /// `class="statepill statepill-{{ id }}"` with both undefined stays **GREEN**, while the bare
    /// `class="statepill"` **reds**. *A widget whose class the generic guard cannot see needs a
    /// specific one*, and this iterates the ENUM rather than reading the template, so it cannot
    /// inherit the same blindness.
    #[test]
    fn every_state_pill_modifier_is_defined_in_the_stylesheet() {
        let css = include_str!("../assets/app.css");
        for state in ObjectState::ALL {
            let rule = format!(".{}", state.modifier());
            assert!(
                css.contains(&rule),
                "{state:?} renders {rule} and app.css defines no rule for it — the pill would take \
                 the base class alone and every state would look identical, which no test can see \
                 and only a look can catch"
            );
        }
    }

    /// The filter narrows, and an unrecognised kind narrows to nothing rather than to everything.
    #[test]
    fn the_filter_narrows_and_says_so_when_it_matches_nothing() {
        let all = body_of(&ScreenQuery::default());
        let printers = body_of(&ScreenQuery {
            kind: Some("printer".into()),
            // Explicit rather than `..Default::default()`: a spread absorbs the NEXT field
            // silently, and the compiler-forced revisit is the point.
            subnet: None,
        });
        assert!(all.contains("nas-01") && all.contains("printer-hall"));
        assert!(
            printers.contains("printer-hall") && !printers.contains("nas-01"),
            "the printer filter must narrow to printers"
        );
        // ⚠️ An unrecognised value narrows to NOTHING and says so. Story 6b.4's review measured a
        // `?sort=bogus` silently ignored; here "showing everything" would tell the operator their
        // filter matched when it did not.
        let bogus = body_of(&ScreenQuery {
            kind: Some("no-such-kind".into()),
            // Explicit rather than `..Default::default()`: a spread absorbs the NEXT field
            // silently, and the compiler-forced revisit is the point.
            subnet: None,
        });
        assert!(
            !bogus.contains("nas-01"),
            "an unknown kind must not show everything"
        );
        assert!(
            bogus.contains(rust_i18n::t!("devices.none").to_string().as_str()),
            "an empty filter must SAY it is empty rather than render a blank table"
        );
    }

    /// 🔑 Every kind the filter bar offers matches at least one device.
    ///
    /// ⚠️ Story 6b.6's validation measured that the dataset's three devices could not feed the
    /// mock's seven filters — **at least four would render an empty table**, a state with no copy,
    /// no key and no marker decision. *A filter over nothing is a demonstration of nothing.*
    #[test]
    fn no_filter_the_bar_offers_is_empty() {
        for kind in DeviceKind::ALL {
            let html = body_of(&ScreenQuery {
                kind: Some(kind.slug().into()),
                // Explicit rather than `..Default::default()`: a spread absorbs the NEXT field
                // silently, and the compiler-forced revisit is the point.
                subnet: None,
            });
            assert!(
                !html.contains(rust_i18n::t!("devices.none").to_string().as_str()),
                "the bar offers {kind:?} and no example device carries it, so that filter renders \
                 an empty table"
            );
        }
    }

    /// Every state of the glossary is demonstrated by the dataset, so the screen shows the axis.
    #[test]
    fn every_state_of_the_axis_is_demonstrated_by_a_device() {
        let states: BTreeSet<&'static str> = example_data::devices()
            .iter()
            .map(|device| device.state.modifier())
            .collect();
        for state in ObjectState::ALL {
            assert!(
                states.contains(state.modifier()),
                "{state:?} is a row of the binding glossary and no example device is in it, so \
                 the screen that exists to show the axis does not show it"
            );
        }
    }

    /// A slug no device carries gets a page that says so and does NOT echo the slug.
    #[test]
    fn an_unknown_slug_is_said_rather_than_echoed() {
        assert!(record_body("does-not-exist").is_none());
        let html = unknown_device_body();
        assert!(html.contains(rust_i18n::t!("record.unknown").to_string().as_str()));
        // 🔴 The body is built from a TEMPLATE with no slug in it at all. The obvious
        // implementation — `format!` into `Html` — was measured serving
        // `/devices/%3Cscript%3E…` back with a 200 at this story's validation.
        assert!(!html.contains("does-not-exist"));
    }

    /// The record carries the four blocks AC1 names, and names *Hosted here* rather than *Impact*.
    #[test]
    fn the_record_carries_the_four_blocks_ac1_names() {
        let html = record_body("nas-01").expect("nas-01 is in the example dataset");
        for key in [
            "record.fields",
            "record.hosted",
            "record.identity",
            "record.history",
        ] {
            let heading = rust_i18n::t!(key).to_string();
            assert!(
                html.contains(heading.as_str()),
                "the record must carry {key}: {heading:?}"
            );
        }
        // ⚠️ FR29/UX-DR29: one containment hop, and **never called *Impact***.
        assert!(
            !html.to_lowercase().contains("impact"),
            "FR29's panel is *Hosted here* and must never be called Impact — the word names a \
             traversal this product does not do"
        );
        // The composite identity is composite: `prd.md:783` is *"not raw MAC"*, and a record
        // showing one MAC would illustrate the very shape FR9 exists to replace.
        let parts = example_data::device_by_id("nas-01")
            .expect("nas-01 is in the example dataset")
            .identity
            .len();
        assert!(
            parts >= 3,
            "a COMPOSITE identity needs more than one component ({parts} seen)"
        );
    }

    /// A device that hosts nothing SAYS so rather than rendering an empty table.
    #[test]
    fn a_device_that_hosts_nothing_says_so() {
        let html = record_body("switch-core").expect("switch-core is in the example dataset");
        assert!(html.contains(rust_i18n::t!("record.hosted_none").to_string().as_str()));
    }

    /// 🔴 **Every i18n key spelled as a literal in the view code EXISTS — and this was found by
    /// LOOKING at the screen, by nothing else.**
    ///
    /// # The defect
    ///
    /// Story 6b.6's first render put **`devices.unplaced_title`** and **`devices.unplaced_reason`**
    /// on the operator's screen, in place of the two headings they were meant to resolve: the keys
    /// are `unplaced.title` and `unplaced.reason`, and the new module invented a `devices.` prefix
    /// for them. `rust-i18n` renders an unknown key **verbatim**, so nothing failed — 649 tests,
    /// eight gates and clippy all green over a page showing its own key names.
    ///
    /// 🔑 **Why no existing guard could see it.** `every_key_carries_both_locales` reads `app.yml`
    /// and asks *"does every key have two languages?"* — a key that is not in the file is not in
    /// its population at all. `the_example_copy_is_translated_rather_than_typed` checks the
    /// dataset's `example.*` keys, not a template heading. And story 6b.6's own state-word guard
    /// covers state words. *Three guards over copy, and a heading rendering its own key passed all
    /// three.*
    ///
    /// ⚠️ **The limit, written**: it reads `t!("…")` with a LITERAL key. A key held in a variable —
    /// which is how [`crate::example_data`]'s fields work, and which
    /// `the_example_copy_is_translated_rather_than_typed` covers instead — is invisible here. The
    /// two guards are complements, and neither is a barrier.
    #[test]
    fn every_literal_key_in_the_view_code_resolves() {
        let mut checked = 0_usize;
        for (file, source) in [
            ("example_screens.rs", include_str!("example_screens.rs")),
            ("page.rs", include_str!("page.rs")),
            ("screens.rs", include_str!("screens.rs")),
        ] {
            // ⚠️ A BOUNDARY, not a substring: the first draft split on `t!("` and matched inside
            // `format!(".{}"`, reporting `".{}"` as a missing key. *A matcher without a boundary
            // finds the language it is written in.*
            for (at, _) in source.match_indices("t!(\"") {
                let before = source[..at].chars().next_back();
                if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                    continue;
                }
                let Some((key, _)) = source[at + 4..].split_once('"') else {
                    continue;
                };
                // A key is a dotted path with no interpolation in it.
                if !key.contains('.') || key.contains(' ') || key.contains('{') {
                    continue;
                }
                assert_ne!(
                    rust_i18n::t!(key),
                    key,
                    "{file} resolves {key:?} and no such key exists — `rust-i18n` renders an \
                     unknown key verbatim, so this reaches the operator's screen as its own name \
                     while every test stays green"
                );
                checked += 1;
            }
        }
        assert!(
            checked >= 60,
            "the premise: at least sixty literal keys were checked ({checked} seen) — a scan that \
             matched nothing would assert nothing"
        );
    }

    /// 🔴 **No i18n KEY reaches the operator's screen — the render-side half, and the only guard
    /// that could have caught either of this story's two i18n defects.**
    ///
    /// # Two defects, one class, three guards blind to both
    ///
    /// Story 6b.6 shipped two of these in one afternoon and **both were found by looking at the
    /// page**, with 649 tests, eight gates and clippy green over each:
    ///
    /// - two section headings resolved `devices.unplaced_*`, keys that do not exist, so the page
    ///   showed its own key names — closed by
    ///   [`every_literal_key_in_the_view_code_resolves`](tests::every_literal_key_in_the_view_code_resolves);
    /// - the record's *Rôle* row carried `example.role.storage` **as a VALUE**, because the dataset
    ///   stores keys and that column printed them raw. That one is invisible to the literal-key
    ///   guard (the key is data, not a `t!` argument) **and** to
    ///   `the_example_copy_is_translated_rather_than_typed` (which checks the fields it knows,
    ///   `role_key` and `reason_key`, not a new struct's).
    ///
    /// 🔑 **The two are one class — an i18n key on the screen — and only a check on the SERVED
    /// PAGE spans it.** Every guard that reads the source, the dataset or the locale file is
    /// looking at one end of a pipe whose other end is what the operator sees. *A guard that never
    /// reads the render cannot see what was rendered.*
    ///
    /// ⚠️ **The limit**: it recognises a key by SHAPE — a dotted lowercase token with no space. A
    /// key spelled unlike a key is invisible, and so is a legitimate value that looks like one
    /// (none today: the dataset's values are addresses, MACs, serials and proper nouns).
    /// AC1 — **every cell of the grid carries its own `aria-label`**, and it names the address.
    ///
    /// 🔴 **Measured carried by NOTHING before this test existed**: at the story's validation,
    /// replacing every `aria-label` with the mock's own `title` left **zero tests red**, and the
    /// repository then held no axe-core, no headless browser and **not one assertion on an `aria-*`
    /// attribute at all**. ⚠️ Past tense on purpose — story 6b.7 adds three, this one and the two
    /// below, and the sentence was written in the present until a review layer pointed out that its
    /// own commit falsifies it. `epics.md:316` names this grid by name as a WCAG 2.1 AA key view,
    /// and the epic's axe-core gate (`epics.md:2108`) is still owed by nobody's story.
    #[test]
    fn every_cell_of_the_grid_carries_its_own_aria_label() {
        let html = ipam_body(&ScreenQuery::default());
        let cells = html.matches("<li class=\"ipam-cell").count();
        assert_eq!(
            cells, 256,
            "a /24 is drawn as 256 cells — 254 hosts plus the network and broadcast addresses"
        );
        // One label per cell, counted on the CELLS rather than on the document: the grid's own
        // container carries one too, and a count over the whole body would be satisfied by it.
        let labelled = html
            .match_indices("<li class=\"ipam-cell")
            .filter(|(at, _)| {
                let rest = &html[*at..];
                let end = rest.find('>').expect("a tag closes");
                rest[..end].contains("aria-label=\"")
            })
            .count();
        assert_eq!(
            labelled, 256,
            "AC1: an `aria-label` PER CELL. A `title` is not an accessible name for a list item, \
             and one label on the container names the grid rather than the address"
        );
        for octet in [0_u8, 1, 41, 254, 255] {
            let expected = format!("192.0.2.{octet} · ");
            assert!(
                html.contains(&expected),
                "the label of .{octet} must name its own address, not its position"
            );
        }
    }

    /// The grid is a LIST laid out by CSS Grid, and never the mock's `role="img"`.
    ///
    /// 🔑 `role="img"` is *Children Presentational: True*, so 256 labels inside it are announced as
    /// one sentence; and `aria-label` on a bare `<div>` maps to `generic`, where ARIA 1.2 prohibits
    /// it outright. `role="list"` is explicit because `list-style: none` is known to drop the role
    /// in Safari/VoiceOver. Chrome 151's accessibility tree was read at validation to confirm the
    /// `listitem` keeps its name.
    #[test]
    fn the_grid_is_a_labelled_list_and_not_a_presentational_image() {
        let html = ipam_body(&ScreenQuery::default());
        assert!(
            html.contains("class=\"ipam-grid\" role=\"list\""),
            "the grid must keep its list role explicitly"
        );
        assert!(
            !html.contains("role=\"img\""),
            "role=\"img\" makes the whole subtree presentational and swallows all 256 labels"
        );
    }

    /// 🔴 An unrecognised `subnet` shows **no grid** and says so — the sibling screen's policy.
    ///
    /// [`inventory_body`]'s own doc: *"an unrecognised `kind` narrows to nothing rather than
    /// silently showing everything: a filter that ignores its input is the shape story 6b.4's review
    /// caught on `?sort=`"*. The validation's prototype served the first subnet instead, which would
    /// have shipped **two screens with opposite policies for one gesture**.
    #[test]
    fn an_unrecognised_subnet_shows_no_grid_and_says_so() {
        let html = ipam_body(&ScreenQuery {
            kind: None,
            subnet: Some("no-such-subnet".into()),
        });
        assert!(
            !html.contains("ipam-grid"),
            "a subnet nothing names must not draw somebody else's grid"
        );
        assert!(
            html.contains(&rust_i18n::t!("ipam.unknown_subnet").to_string()),
            "and it must say why the grid is absent"
        );
        // The control: a slug that DOES name a subnet draws one, so the assertion above is not
        // satisfied by a screen that never draws a grid at all.
        let known = ipam_body(&ScreenQuery {
            kind: None,
            subnet: Some("workshop".into()),
        });
        assert!(known.contains("ipam-grid"), "the control must draw a grid");
        assert!(
            known.contains("198.51.100."),
            "and it must be the workshop's own addresses — a selector that parses its input and \
             then ignores it renders the first subnet, and Rust does not lint an unused parameter"
        );
    }

    /// The selector offers every subnet, marks the one in force, and never says `aria-current="page"`.
    ///
    /// ⚠️ **Measured invisible before this test**: `exactly_one_entry_is_current_on_each_screen`
    /// renders the shell with an EMPTY body, so a second `aria-current="page"` inside a screen's
    /// content is outside its population. Two of them in one document is an ARIA error, and the
    /// inventory's filter bar already established `"true"` as the idiom for a control — which makes
    /// `"page"` a coin flip for whoever writes the next selector.
    #[test]
    fn the_subnet_selector_marks_one_tab_and_never_claims_to_be_the_page() {
        let html = ipam_body(&ScreenQuery::default());
        for subnet in example_data::subnets() {
            assert!(
                html.contains(&format!("/ipam?subnet={}", subnet.slug)),
                "{} must be offered by the selector",
                subnet.cidr
            );
        }
        // ⚠️ **The negative comes FIRST, and the order is a measurement.** With the count above it,
        // swapping `"true"` for `"page"` reddened on *"exactly one tab is in force"* — a true
        // failure naming the wrong cause, and the assertion that exists for this defect was never
        // reached. Story 5.13's assertion-order finding, which this project has now met five times.
        assert!(
            !html.contains("aria-current=\"page\""),
            "`page` belongs to the shell's navigation; a second one inside the body is an ARIA \
             error on a WCAG key view"
        );
        assert_eq!(
            html.matches("aria-current=\"true\"").count(),
            1,
            "exactly one tab is in force"
        );
    }

    /// 🔴 The legend names every cell modifier as a **literal**, and that redundancy is deliberate.
    ///
    /// `every_class_a_template_names_is_defined_in_the_stylesheet` skips any class attribute
    /// carrying an Askama expression, so the cells — whose modifier comes from a Rust `match` — are
    /// invisible to it. **Measured at validation**: write the legend the DRY way, from the same
    /// `CellState` data the cells use, and deleting `.ipam-cell-used` from the sheet leaves the
    /// whole suite green while every occupied cell ships with no colour. That is
    /// `_dashboard.html:58`'s `spark-h8` reproduced, *caused by the tidy gesture*.
    ///
    /// 🔑 This test is what stops the tidy-up: the literals are load-bearing, and their loss is a
    /// silent loss of coverage rather than a visible one.
    #[test]
    fn the_legend_names_every_cell_modifier_as_a_literal() {
        let template = include_str!("../templates/_ipam_example.html");
        for state in [
            example_data::CellState::Used,
            example_data::CellState::Reserved,
            example_data::CellState::Free,
            example_data::CellState::Structural,
        ] {
            let literal = format!("ipam-cell {}\"", state.modifier());
            assert!(
                template.contains(&literal),
                "the legend must name {:?} as a literal class: it is the only way the stylesheet \
                 guard can see a modifier the cells choose in Rust",
                state.modifier()
            );
        }
    }

    /// AC2 — the applications screen says, on the screen, that the owner and the criticality are
    /// declared and that nothing will ever observe them.
    #[test]
    fn the_apps_screen_states_that_owner_and_criticality_are_unobservable() {
        let html = apps_body(&ScreenQuery::default());
        let lede = rust_i18n::t!("apps.lede").to_string();
        assert!(
            html.contains(&lede),
            "AC2's sentence is the reason this screen exists and belongs ON it, not only in the \
             story file"
        );
        for header in ["apps.owner", "apps.criticality"] {
            assert!(
                html.contains(&rust_i18n::t!(header).to_string()),
                "{header} must be a column of the table"
            );
        }
        for app in example_data::apps() {
            assert!(
                html.contains(app.owner),
                "{} must show its owner — a proper noun, rendered as data",
                app.name
            );
            assert!(
                html.contains(&rust_i18n::t!(app.criticality_key).to_string()),
                "{} must show its criticality — a classification, rendered from a key",
                app.name
            );
        }
    }

    /// 🔴 The screen renders **no exposure column** — Guy's arbitration of 2026-08-20, option (c).
    ///
    /// ⚠️ A negative assertion, deliberately: the mock has the column, the copy for it is one line
    /// away, and the reason not to ship it is a vocabulary decision that lives in a document. This
    /// is what makes the decision survive the next person who compares the screen to the mock.
    #[test]
    fn the_apps_screen_renders_no_exposure_column() {
        let html = apps_body(&ScreenQuery::default());
        for word in ["Exposition", "Exposure", "Reverse proxy", "Hors périmètre"] {
            assert!(
                !html.contains(word),
                "{word:?} is on the screen. Exposure is a fourth vocabulary axis with four values \
                 and no glossary row; option (b) — render it and register it — was refused. \
                 Rendering it needs a new arbitration, not a column"
            );
        }
    }

    /// An application hosted outside the perimeter says so rather than linking nowhere.
    #[test]
    fn an_application_without_a_host_says_so_rather_than_linking_nowhere() {
        let html = apps_body(&ScreenQuery::default());
        assert!(
            html.contains(&rust_i18n::t!("apps.no_host").to_string()),
            "the row hosted outside the perimeter must say so"
        );
        for app in example_data::apps() {
            let Some(host) = app.host else { continue };
            assert!(
                html.contains(&format!("/devices/{host}")),
                "{} must link to its host's record, as the inventory does",
                app.name
            );
        }
    }

    #[test]
    fn no_i18n_key_reaches_the_screen() {
        // 🔑 The detector is [`key_names_in_text`], shared with `main.rs`'s route-table guard since
        // story 6b.7 — see its doc for the enumeration this list used to be, and for what that cost.
        // This caller keeps the RECORD pages, which no route in `Screen::ALL` reaches.
        let mut pages = vec![("/devices", inventory_body(&ScreenQuery::default()))];
        for device in example_data::devices() {
            pages.push((
                "/devices/{id}",
                record_body(device.id).expect("every listed device has a record"),
            ));
        }
        pages.push(("/devices/unknown", unknown_device_body()));

        let mut checked = 0_usize;
        for (screen, html) in &pages {
            let keys = key_names_in_text(html);
            assert!(
                keys.is_empty(),
                "{screen} renders {keys:?}, which are i18n KEYS and not words — the operator \
                 reads the key's name where the translation belongs, and no guard over the \
                 source, the dataset or the locale file can see it"
            );
            checked += visible_text(html).split_whitespace().count();
        }
        assert!(
            checked >= 200,
            "the premise: at least two hundred rendered words were inspected ({checked} seen)"
        );
    }
}
