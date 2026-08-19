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
}

/// The copy these two screens need, resolved once into the operator's language.
///
/// 🔴 Resolved HERE and not in the template, because [`crate::example_data`] holds i18n KEYS rather
/// than sentences — see [`crate::example_data::ExampleDevice::role_key`] for the defect that taught
/// us the difference, which was found by looking at the screen and by nothing else.
pub(crate) struct ExampleStrings {
    /// The example marker's badge — the field name is shared with `_example_marker.html`.
    example_badge: String,
    /// The example marker's sentence.
    example_sentence: String,
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
        example_badge: rust_i18n::t!("example.badge").to_string(),
        example_sentence: rust_i18n::t!("example.sentence").to_string(),
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
        unplaced_title: rust_i18n::t!("devices.unplaced_title").to_string(),
        unplaced_reason: rust_i18n::t!("devices.unplaced_reason").to_string(),
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
                .map(|field| FieldRow {
                    label: rust_i18n::t!(field.label_key).to_string(),
                    declared: field.declared.unwrap_or(dash).to_string(),
                    observed: field.observed.unwrap_or(dash).to_string(),
                    state: StateView::new(field.state, None),
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
}
