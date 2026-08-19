//! The example dataset: what the demonstration screens show, and where it comes from.
//!
//! # Why this is a module of CONSTANTS and not a fixture file
//!
//! 🔴 Epic 6b's constraint 1 forbids a demonstration screen to open a database connection, and
//! story 6b.2 made that enforceable by giving those screens a `Router<()>` — a handler there
//! cannot take `State<MySqlPool>` because it does not compile. **A dataset that had to be READ
//! would break that carrier**, whether it were read from the store or from disk. So the data is
//! in the binary, and the guard stays a property of the type rather than of anyone's discipline.
//!
//! ⚠️ **It is not `fixtures/`, and must not drift into it.** The committed corpus under
//! `fixtures/` is the identity engine's evidence — sha256-locked by a gate, and every artefact
//! there is an input to a MEASUREMENT. This is decoration for a screen. Confusing the two would
//! put a demo string behind a lock that exists for something else entirely.
//!
//! 🔑 **Every device carries a STABLE id**, and that is owed rather than decorative: the register
//! records that story 6b.6's `/devices/{id}` "needs an id, which needs 6b.3's example dataset".
//! Change a slug here and you break a URL that story will ship.

/// One device in the example inventory.
pub(crate) struct ExampleDevice {
    /// The stable slug this device is addressed by — see the module doc: story 6b.6 routes on it.
    pub(crate) id: &'static str,
    /// What the operator would have named it.
    pub(crate) name: &'static str,
    /// Its address inside the example network.
    pub(crate) ipv4: &'static str,
    /// Its hardware address.
    pub(crate) mac: &'static str,
    /// The i18n KEY of what it is for.
    ///
    /// 🔴 A key and not a sentence, and the reason was found by LOOKING rather than by testing:
    /// the first draft carried English literals here, so a French operator read *"Storage"* and
    /// *"Network"* under a French interface — an NFR26 violation that the whole suite passed over,
    /// because a literal is not a key and `every_key_carries_both_locales` can only see keys.
    /// *Example data is still operator-visible copy.*
    pub(crate) role_key: &'static str,
}

/// One sighting the example engine could not place on a device.
///
/// 🔑 It exists so the witness screen carries **two sections of different kinds**, which is what
/// lets AC2's *smallest unit* be demonstrated below screen level rather than asserted.
pub(crate) struct ExampleSighting {
    /// The address it answered on.
    pub(crate) ipv4: &'static str,
    /// Its hardware address — `None` when the sighting gave none, which is the case the second
    /// example row exists to show.
    ///
    /// 🔑 An `Option` and not a sentinel string. It carried a hard `"—"` literal until story 6b.3's
    /// code review, under a doc comment reading *"when it gave one"* — an `Option`'s sentence over
    /// a field that had no absent case. A false doc is a defect here, and the placeholder is a
    /// DISPLAY decision, taken in [`crate::page`] where every other resolution happens.
    pub(crate) mac: Option<&'static str>,
    /// The i18n KEY of why the example engine abstained — a key, for the reason given on
    /// [`ExampleDevice::role_key`].
    pub(crate) reason_key: &'static str,
}

/// The example inventory — RFC 5737 documentation addresses and RFC 7042 documentation MACs.
///
/// ⚠️ The addresses are the ranges reserved FOR documentation on purpose: a screenshot of this
/// screen can be published, and a plausible-looking `192.168.1.x` in a manual is an address that
/// belongs to somebody.
pub(crate) fn devices() -> Vec<ExampleDevice> {
    vec![
        ExampleDevice {
            id: "nas-01",
            name: "nas-01",
            ipv4: "192.0.2.10",
            mac: "00:00:5E:00:53:10",
            role_key: "example.role.storage",
        },
        ExampleDevice {
            id: "switch-core",
            name: "switch-core",
            ipv4: "192.0.2.2",
            mac: "00:00:5E:00:53:02",
            role_key: "example.role.network",
        },
        ExampleDevice {
            id: "printer-hall",
            name: "printer-hall",
            ipv4: "192.0.2.31",
            mac: "00:00:5E:00:53:31",
            role_key: "example.role.peripheral",
        },
    ]
}

/// The example sightings the engine could not place.
pub(crate) fn unplaced_sightings() -> Vec<ExampleSighting> {
    vec![
        ExampleSighting {
            ipv4: "192.0.2.57",
            mac: Some("00:00:5E:00:53:57"),
            reason_key: "example.reason.no_declared_match",
        },
        ExampleSighting {
            ipv4: "192.0.2.58",
            mac: None,
            reason_key: "example.reason.no_hardware_address",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 🔴 **Every operator-visible sentence in the example dataset is a KEY, and every key
    /// RESOLVES.**
    ///
    /// # Why this guard exists, and what found the defect it prevents
    ///
    /// Not a test. **Looking at the screen.** The first draft of this module carried English
    /// literals — a French operator read *"Storage"*, *"Network"* and *"No declared record matches
    /// this address"* under a fully French interface — and the whole suite was GREEN, because
    /// `every_key_carries_both_locales` scans `app.yml` and a literal never reaches it. An NFR26
    /// violation with no possible carrier on the locale side. *Example data is operator-visible
    /// copy, and being fictional does not make it exempt.*
    ///
    /// 🔑 **It asserts THREE different things, and they catch different mistakes.** That the value
    /// LOOKS like a key catches the literal (measured: M8 reds). That it RESOLVES is aimed at the
    /// typo, since `rust-i18n` renders an unknown key as its own name (M8b). That it sits in the
    /// namespace of ITS OWN FIELD catches the key that is real, resolves, and says the wrong thing.
    ///
    /// 🔴 **The third half is story 6b.3's code review, and it was measured missing.** The guard
    /// checked SHAPE and RESOLVABILITY and never WHICH key: setting `role_key` to `"example.badge"`
    /// — a real key, from the marker's namespace — left **all 613 tests green, all eight
    /// `cargo xtask ci` gates green and clippy clean**, while `/devices` rendered *"Exemple"* in the
    /// Role column where *"Stockage"* belongs. An operator-visible wrong label surviving the whole
    /// apparatus, in the guard added to close an operator-visible i18n defect — *a guard placed
    /// where the defect cannot occur reads as coverage and is none*, Epic 5's dominant class,
    /// reproduced inside its own remedy.
    ///
    /// ⚠️ M8b first came back GREEN and the green was an artefact, worth recording because it cost
    /// real work: the mutation ran against a tree that **did not compile**, and the driver grepped
    /// for `FAILED` test lines, which a compile failure does not produce. *A mutation that does not
    /// build measures nothing, and a filter that cannot see the difference reports it as a pass.*
    #[test]
    fn the_example_copy_is_translated_rather_than_typed() {
        let mut checked = 0_usize;
        // `namespace` is the field's OWN prefix, not `example.` — see the third half above.
        let mut assert_key = |key: &str, namespace: &str| {
            assert!(
                key.starts_with("example.") && !key.contains(' '),
                "{key:?} is a literal, not a key: a sentence typed here renders in English under \
                 a French interface, and no locale guard can see it"
            );
            assert!(
                key.starts_with(namespace),
                "{key:?} is a real key that resolves, and it is the WRONG one: this field's copy \
                 lives under {namespace:?}. A key borrowed from another namespace renders a \
                 plausible word in the operator's language and no shape check can see it"
            );
            assert_ne!(
                rust_i18n::t!(key),
                key,
                "{key:?} resolves to its own name — an unknown key is rendered verbatim by \
                 `rust-i18n`, which would put the key's text on the operator's screen"
            );
            checked += 1;
        };
        for device in devices() {
            assert_key(device.role_key, "example.role.");
        }
        for sighting in unplaced_sightings() {
            assert_key(sighting.reason_key, "example.reason.");
        }
        assert!(
            checked >= 5,
            "the premise: the dataset carries at least five translated strings ({checked} seen) \
             — an empty dataset would assert nothing"
        );
    }
}
