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
    /// What it is for, in the operator's words.
    pub(crate) role: &'static str,
}

/// One sighting the example engine could not place on a device.
///
/// 🔑 It exists so the witness screen carries **two sections of different kinds**, which is what
/// lets AC2's *smallest unit* be demonstrated below screen level rather than asserted.
pub(crate) struct ExampleSighting {
    /// The address it answered on.
    pub(crate) ipv4: &'static str,
    /// Its hardware address, when it gave one.
    pub(crate) mac: &'static str,
    /// Why the example engine abstained, in the operator's words.
    pub(crate) reason: &'static str,
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
            role: "Storage",
        },
        ExampleDevice {
            id: "switch-core",
            name: "switch-core",
            ipv4: "192.0.2.2",
            mac: "00:00:5E:00:53:02",
            role: "Network",
        },
        ExampleDevice {
            id: "printer-hall",
            name: "printer-hall",
            ipv4: "192.0.2.31",
            mac: "00:00:5E:00:53:31",
            role: "Peripheral",
        },
    ]
}

/// The example sightings the engine could not place.
pub(crate) fn unplaced_sightings() -> Vec<ExampleSighting> {
    vec![
        ExampleSighting {
            ipv4: "192.0.2.57",
            mac: "00:00:5E:00:53:57",
            reason: "No declared record matches this address",
        },
        ExampleSighting {
            ipv4: "192.0.2.58",
            mac: "—",
            reason: "Answered without a hardware address",
        },
    ]
}
