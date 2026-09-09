//! The inventory — what the operator has DOCUMENTED, read from the store.
//!
//! # 🔴 Why this screen exists: documenting a machine used to make it disappear
//!
//! Until now `/devices` showed eight invented machines and none of yours. An operator who pressed
//! the one live control in this product — *Add*, on a `Nouveau` triage row — watched the question
//! leave the queue and land **nowhere they could see**. The gesture worked and its result was
//! invisible, which is a worse dead end than a screen that says it is empty.
//!
//! It was named as point 3 of the plan the 2026-08-30 project review left, and it got heavier the
//! same week: since the reverse-DNS story the gesture writes TWO fields, not one.
//!
//! # What is in it, and what is deliberately not
//!
//! **The declared side, and only that.** A row here is an entity the operator documented; an
//! address the network shows and nobody has claimed is a QUESTION, and questions live in the
//! triage queue. Listing both here would put the same host on two screens under two meanings —
//! *the gap is the product*, and an inventory that already contains the gap is not an inventory.
//!
//! ⚠️ **There is no drill-in, and that is today's data speaking rather than a design.** A
//! documented entity carries at most two fields (`ipv4`, and `hostname` since the reverse-DNS
//! story), so the row shows everything the store knows and a record page would repeat it. The day
//! an entity carries an owner, a criticality or a group — Epic 6's and Epic 15's — the record
//! earns its own page and `Screen::Device` stops being an example.

use std::collections::BTreeMap;

use askama::Template;

use opencmdb_core::observation::Fact;

use crate::repo::{DeclaredProvenance, ObservedBatch};

/// One documented entity, as the inventory shows it.
pub(crate) struct InventoryRow {
    /// The entity's id — the row's stable anchor, never displayed.
    pub(crate) id: String,
    /// The name it answers to, when one was documented; empty otherwise.
    ///
    /// ⚠️ Shown whole here, unlike the triage queue, which abbreviates to the first label to keep
    /// a dense row on one line. This is a TABLE with a column of its own: the reason to shorten
    /// does not apply, and the value shown is then exactly the value stored.
    pub(crate) name: String,
    /// The address the entity is recognised by — the perimeter key, so never empty.
    pub(crate) ipv4: String,
    /// How many fields are declared for it, already rendered with its noun.
    pub(crate) fields: String,
    /// How the declaration was made — adopted from a sighting, or entered by hand.
    pub(crate) origin: String,
    /// When it was documented, in the operator's language.
    pub(crate) documented: String,
    /// When the network last showed this address, or the words for *never seen*.
    ///
    /// 🔑 **This is the column that makes the inventory worth reading.** A documented machine the
    /// network has not shown for a month is the product's own subject: the declared side says it
    /// is there and the observed side has stopped agreeing. It is not a gap — nothing contradicts
    /// anything — and it is not nothing either.
    pub(crate) seen: String,
    /// Seconds since that sighting, for the ordering. `i64::MAX` when never seen.
    pub(crate) age_seconds: i64,
}

/// The inventory screen's state.
pub(crate) struct InventoryView {
    /// The documented entities, freshest first.
    pub(crate) rows: Vec<InventoryRow>,
    /// The count, already rendered — or empty when there is nothing to count.
    pub(crate) total: String,
}

/// The copy this screen renders, resolved once.
pub(crate) struct InventoryStrings {
    /// The section's heading.
    ///
    /// 🔴 It read *Inventory* / *Inventaire* until a look at the page: the navigation GROUP is
    /// already called that and the entry beside it is *Devices*, so one screen carried three words
    /// for one idea and the real section shared a name with a group. *Your records* says whose
    /// they are, which is the one thing that distinguishes this section from the example list
    /// under it.
    pub(crate) title: String,
    /// The sentence under it.
    pub(crate) lede: String,
    /// What the screen says when nothing has been documented yet, before the link.
    ///
    /// 🔴 **Split in two so the link sits INSIDE the sentence.** It was one string followed by a
    /// bare `<a>Triage</a>`, which served *"…and press Add. **Triage**"* — a word stapled after a
    /// full stop, not a door — and the guard asserted that shape, so a test REQUIRED it. ⚠️ It
    /// also spelled the button's label as a literal, and that label was renamed once already
    /// (« Merger » → « Ajouter », v0.3.1): the sentence now describes the control instead of
    /// quoting it, so the next rename cannot make it false.
    pub(crate) none_before: String,
    /// The rest of that sentence, after the link.
    pub(crate) none_after: String,
    /// The name column.
    pub(crate) col_name: String,
    /// The address column.
    pub(crate) col_ipv4: String,
    /// The declared-fields column.
    pub(crate) col_fields: String,
    /// The provenance column.
    pub(crate) col_origin: String,
    /// The documented-at column.
    pub(crate) col_documented: String,
    /// The last-seen column.
    pub(crate) col_seen: String,
    /// What a row shows in the name column when no name was documented.
    pub(crate) unnamed: String,
    /// The name of the screen the empty state links to.
    ///
    /// 🔴 It was `title` — so the empty state read *"…press Add. **Inventory**"*, a door labelled
    /// with the room you are already in. Found by reading the served page; no assertion could see
    /// it, because the link and its text were both present and both resolved.
    pub(crate) triage: String,
}

/// The inventory body, as it is served.
#[derive(Template)]
#[template(path = "_inventory.html")]
pub(crate) struct InventoryBody {
    /// The screen's state.
    pub(crate) inventory: InventoryView,
    /// Its copy.
    pub(crate) s: InventoryStrings,
}

/// This screen's copy, resolved from the operator's locale.
pub(crate) fn inventory_strings() -> InventoryStrings {
    InventoryStrings {
        title: rust_i18n::t!("inventory.title").to_string(),
        lede: rust_i18n::t!("inventory.lede").to_string(),
        none_before: rust_i18n::t!("inventory.none_before").to_string(),
        none_after: rust_i18n::t!("inventory.none_after").to_string(),
        col_name: rust_i18n::t!("inventory.col_name").to_string(),
        col_ipv4: rust_i18n::t!("inventory.col_ipv4").to_string(),
        col_fields: rust_i18n::t!("inventory.col_fields").to_string(),
        col_origin: rust_i18n::t!("inventory.col_origin").to_string(),
        col_documented: rust_i18n::t!("inventory.col_documented").to_string(),
        col_seen: rust_i18n::t!("inventory.col_seen").to_string(),
        unnamed: rust_i18n::t!("inventory.unnamed").to_string(),
        triage: rust_i18n::t!("nav.triage").to_string(),
    }
}

/// PURE: shape the declared rows, their provenance and the observations into the inventory.
///
/// 🔑 **`now` is a PARAMETER.** Every builder in this product takes its instant rather than reading
/// one: `chrono`'s `clock` feature is off workspace-wide, which stops one SPELLING and not the act
/// — `std::time::SystemTime::now()` compiles freely inside a pure function. Story 6b.4's M6
/// measured that, and the guard is a test rather than the feature flag.
pub(crate) fn build_inventory(
    declared: Vec<(String, String, String)>,
    provenance: &[DeclaredProvenance],
    observations: &[ObservedBatch],
    now: chrono::DateTime<chrono::Utc>,
) -> InventoryView {
    // Group the declared attributes by entity, preserving the store's order within each.
    let mut entities: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (entity_id, key, value) in declared {
        entities.entry(entity_id).or_default().push((key, value));
    }

    // 🔴 **The freshest sighting PER ADDRESS, computed once.** The first draft scanned every
    // observation for every entity, so the cost was `entities × observations` — on the screen the
    // same review found had no store budget. Measured by the edge layer at 300 entities and 50 000
    // observations: **3.7 s**, and sixteen concurrent requests delayed `/healthz` by **7.57 s**,
    // which is the route a container orchestrator polls with a three-to-five-second patience.
    //
    // ⚠️ The read itself is still unbounded (issue #150) and this does not bound it: it removes
    // the multiplication, not the scan.
    //
    // 🔑 It keys on the SAME thing `crate::page::in_perimeter` matches — an `IpV4` fact whose
    // rendered address equals the entity's — so the freshness shown here and the comparison behind
    // the queue cannot disagree about which sightings belong to whom.
    let mut freshest: BTreeMap<String, &ObservedBatch> = BTreeMap::new();
    for batch in observations {
        for fact in &batch.facts {
            let Fact::IpV4 { addr } = fact else { continue };
            let entry = freshest.entry(addr.to_string());
            match entry {
                std::collections::btree_map::Entry::Vacant(slot) => {
                    slot.insert(batch);
                }
                std::collections::btree_map::Entry::Occupied(mut slot) => {
                    if batch.observed_at > slot.get().observed_at {
                        slot.insert(batch);
                    }
                }
            }
        }
    }

    let mut rows: Vec<InventoryRow> = Vec::new();
    for (entity_id, attrs) in entities {
        let value_of = |key: &str| {
            attrs
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v.clone())
                .unwrap_or_default()
        };
        // An entity with no `ipv4` is not addressable and cannot be reconciled either — the
        // perimeter key is what an entity IS recognised by. Skipped rather than rendered with an
        // empty address, which would be a row about nothing.
        let ipv4 = value_of("ipv4");
        if ipv4.is_empty() {
            continue;
        }

        // The entity's most recent declared write — the same rule the triage pane applies.
        //
        // ⚠️ **Provenance is per FIELD and this column shows ONE word for the entity.** The
        // documenting gesture writes two fields in one transaction, so they share `updated_at` and
        // the tie must not be decided by the store's row order: `attr_key` breaks it, which makes
        // the column deterministic without pretending it is complete. An entity whose `ipv4` was
        // adopted and whose `hostname` was entered by hand shows one of the two, and the day a
        // field-level provenance matters this column is where it lands. Registered.
        let newest_write = provenance
            .iter()
            .filter(|p| p.entity_id == entity_id)
            .max_by_key(|p| (p.updated_at, p.attr_key.clone()));

        let newest_sighting = freshest.get(&ipv4).copied();

        rows.push(InventoryRow {
            id: entity_id,
            name: value_of("hostname"),
            ipv4,
            fields: crate::page::counted_fields("inventory.n_fields", attrs.len()),
            // ⚠️ **Words, never an empty cell** — the rule the name and last-seen columns already
            // follow in this same table, and the review found these two breaking it: with no
            // provenance row they rendered `<td class="muted"></td>`, which reads as a value the
            // product failed to render rather than as a fact it does not hold.
            origin: newest_write.map_or_else(
                || rust_i18n::t!("inventory.origin_unknown").to_string(),
                |p| {
                    rust_i18n::t!(match p.origin.as_str() {
                        "adopted" => "inventory.origin_adopted",
                        _ => "inventory.origin_manual",
                    })
                    .to_string()
                },
            ),
            documented: newest_write.map_or_else(
                || rust_i18n::t!("meta.never_seen").to_string(),
                |p| crate::page::relative_time(now, p.updated_at),
            ),
            seen: newest_sighting.map_or_else(
                || rust_i18n::t!("meta.never_seen").to_string(),
                |b| crate::page::relative_time(now, b.observed_at),
            ),
            age_seconds: newest_sighting
                .map_or(i64::MAX, |b| (now - b.observed_at).num_seconds().max(0)),
        });
    }

    // Freshest first, then by address so the order is total and does not depend on the store's.
    // Freshest first, then by address. ⚠️ Two entities may declare the SAME address (registered
    // for the queue by story 6b.4 and inherited here), in which case both keys are equal and the
    // tie falls to `sort_by`'s stability over the entity-id order of the `BTreeMap` above —
    // deterministic, but by a mechanism worth naming rather than calling the order *total*.
    // ⚠️ The address compares as a STRING, so `192.0.2.9` follows `192.0.2.10`. Cosmetic, and only
    // within one freshness.
    rows.sort_by(|a, b| {
        a.age_seconds
            .cmp(&b.age_seconds)
            .then_with(|| a.ipv4.cmp(&b.ipv4))
    });

    let total = if rows.is_empty() {
        String::new()
    } else {
        crate::page::counted_fields("inventory.n_entities", rows.len())
    };
    InventoryView { rows, total }
}
