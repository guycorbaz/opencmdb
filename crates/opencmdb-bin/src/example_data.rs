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
//! Change a slug here and you break a URL that story will ship. ⚠️ **Story 6b.6 SHIPPED that
//! route**, so the three slugs minted by 6b.3 are now live addresses, not a promise.

use crate::state_vocabulary::ObjectState;

/// What KIND of object a device is — the mock's filter axis.
///
/// ⚠️ **This is NOT `role_key`.** The role says what a device is FOR (storage, network,
/// peripheral); the kind says what it IS, and the mock filters on the kind. Story 6b.6's validation
/// measured that renaming one into the other loses an axis rather than saving a field.
///
/// 🔴 **Seven kinds and eight devices, and the count is load-bearing**: the mock ships seven
/// filters, and three devices would leave at least four of them rendering an empty table — a state
/// with no copy, no key and no marker decision. *A filter over nothing is a demonstration of
/// nothing.*
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeviceKind {
    /// A physical server.
    Server,
    /// A virtual machine.
    VirtualMachine,
    /// A container.
    Container,
    /// A purpose-built appliance.
    Appliance,
    /// Network equipment.
    Network,
    /// A printer.
    Printer,
    /// An operator's workstation.
    Workstation,
}

impl DeviceKind {
    /// Every kind, in the mock's filter order.
    pub(crate) const ALL: [DeviceKind; 7] = [
        DeviceKind::Server,
        DeviceKind::VirtualMachine,
        DeviceKind::Container,
        DeviceKind::Appliance,
        DeviceKind::Network,
        DeviceKind::Printer,
        DeviceKind::Workstation,
    ];

    /// The slug this kind is filtered by, in the query string.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            DeviceKind::Server => "server",
            DeviceKind::VirtualMachine => "vm",
            DeviceKind::Container => "container",
            DeviceKind::Appliance => "appliance",
            DeviceKind::Network => "network",
            DeviceKind::Printer => "printer",
            DeviceKind::Workstation => "workstation",
        }
    }

    /// The i18n KEY of the kind's name — a key, for the reason given on [`ExampleDevice::role_key`].
    pub(crate) fn label_key(self) -> &'static str {
        match self {
            DeviceKind::Server => "example.kind.server",
            DeviceKind::VirtualMachine => "example.kind.vm",
            DeviceKind::Container => "example.kind.container",
            DeviceKind::Appliance => "example.kind.appliance",
            DeviceKind::Network => "example.kind.network",
            DeviceKind::Printer => "example.kind.printer",
            DeviceKind::Workstation => "example.kind.workstation",
        }
    }
}

/// One compared field on the device record — the *field by field* block AC1 names.
pub(crate) struct ExampleField {
    /// The i18n KEY of the field's name.
    pub(crate) label_key: &'static str,
    /// What the operator documented, or `None` when nothing is declared for it.
    pub(crate) declared: Option<&'static str>,
    /// What the product observed, or `None` when nothing answered for it.
    pub(crate) observed: Option<&'static str>,
    /// What a SECOND source observed, when two disagree — [`ObjectState::Conflict`]'s whole shape.
    ///
    /// 🔴 **Found by the blind review layer, from the constants alone.** `Conflict` is defined as
    /// *"two observations disagree WITH EACH OTHER — source against source"*, and the first draft
    /// gave its field `observed: None`: the row rendered an em dash, identical to a `Gap` of
    /// absence, differing only by the colour of a pill. **A demonstration screen exists to show
    /// what a state LOOKS like, and that one looked like a different state.**
    pub(crate) observed_alt: Option<&'static str>,
    /// This field's own state.
    ///
    /// ⚠️ **Not every state of the axis is a FIELD state, and the distinction was found at this
    /// story's code review.** [`Concordant`](ObjectState::Concordant), [`Gap`](ObjectState::Gap)
    /// and [`Conflict`](ObjectState::Conflict) compare a declared value against an observed one, so
    /// they belong to a field. [`Ambiguous`](ObjectState::Ambiguous) is about which OBJECT a
    /// sighting belongs to and [`Undeclared`](ObjectState::Undeclared) about whether an object has
    /// a record at all: **both are properties of the device, not of one of its fields.** Putting
    /// `Ambiguous` on a field was a category error the screen made look ordinary. The glossary does
    /// not yet carry the distinction; it is registered rather than introduced here.
    pub(crate) state: ObjectState,
    /// Whether [`declared`](ExampleField::declared) and [`observed`](ExampleField::observed) hold
    /// i18n KEYS rather than factual values.
    ///
    /// 🔴 **An explicit flag and not a guess.** Most compared fields are addresses, MACs and
    /// serials — data, locale-neutral, printed as they are. The *role* is a translated word, and
    /// the first draft printed its key: the record rendered *"example.role.storage"* in both
    /// columns with 649 tests, eight gates and clippy green. A helper that resolved *"anything
    /// that looks like a key"* would have hidden the same mistake behind a heuristic; the flag
    /// makes each field say which kind of value it carries.
    ///
    /// ⚠️ **It protects ONE direction, measured.** Mutation M12 sets it to `true` on a field whose
    /// values are addresses and the whole suite stays **GREEN**: `rust_i18n::t!` renders an unknown
    /// key verbatim, so resolving `"192.0.2.10"` yields `"192.0.2.10"` and nothing changes on the
    /// screen. The flag catches *a key printed raw*; it does not catch *a fact needlessly
    /// resolved*, and the second costs nothing — but the promise must not be read as covering both.
    pub(crate) values_are_keys: bool,
}

/// One object this device hosts — FR29's *"Hosted here"*, ONE containment hop.
///
/// ⚠️ **The product has NO containment data of any kind** — five tables and not one relates two
/// objects — so this is example content in the strongest sense. FR29 is *one hop, no traversal*, and
/// ARCH-38 makes `hosts` lookup-only; **it is never called *Impact***.
///
/// 🔑 **Criticality is deliberately NOT a field here.** The mock carries one, and rendering it would
/// introduce a third vocabulary axis (critical / normal / low) with **no glossary row and no
/// producer** — which is precisely what AC2 says to register rather than introduce. Registered.
pub(crate) struct ExampleHosted {
    /// What it is called — a proper noun, therefore data and not a key.
    pub(crate) name: &'static str,
    /// The i18n KEY of what it is.
    pub(crate) kind_key: &'static str,
}

/// One component of the composite identity — the block AC1 names.
///
/// 🔑 *"Composite identity, not raw MAC"* (`prd.md:783`) is the product's founding identity claim,
/// and the example dataset carried nothing of the sort before this story: one MAC per device is the
/// very shape FR9 exists to replace.
pub(crate) struct ExampleIdentityPart {
    /// The i18n KEY of what this component is.
    pub(crate) label_key: &'static str,
    /// Its value — an address or a serial, therefore locale-neutral data.
    pub(crate) value: &'static str,
}

/// One line of the observation history — FR37, owned by story 6.19 and shown here as an example.
pub(crate) struct ExampleHistoryLine {
    /// When, as an ISO-ish instant: locale-neutral on purpose, so it needs no key.
    pub(crate) when: &'static str,
    /// The i18n KEY of what happened.
    pub(crate) what_key: &'static str,
}

/// One device in the example inventory.
pub(crate) struct ExampleDevice {
    /// The stable slug this device is addressed by — see the module doc: story 6b.6 routes on it.
    pub(crate) id: &'static str,
    /// What the operator would have named it.
    pub(crate) name: &'static str,
    /// What it IS — the mock's filter axis, distinct from [`ExampleDevice::role_key`].
    pub(crate) kind: DeviceKind,
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
    /// Where the reconciliation stands for this device, as the operator reads it.
    pub(crate) state: ObjectState,
    /// The i18n KEY of the qualifier after the state, or `None` for a bare state.
    ///
    /// ⚠️ Guy's arbitration, 2026-08-19: *"Écart · 2 champs"* is the word *écart* qualified, **not a
    /// sixth term** — see [`crate::state_vocabulary::QUALIFIER_SEPARATOR`].
    pub(crate) qualifier_key: Option<&'static str>,
    /// When it was last seen — locale-neutral, therefore not a key.
    pub(crate) last_seen: &'static str,
    /// The compared fields shown on its record.
    pub(crate) fields: &'static [ExampleField],
    /// What it hosts — FR29, one hop.
    pub(crate) hosted: &'static [ExampleHosted],
    /// Its composite identity, component by component.
    pub(crate) identity: &'static [ExampleIdentityPart],
    /// Its observation history — FR37.
    pub(crate) history: &'static [ExampleHistoryLine],
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

/// A device that hosts nothing — most of them, and the empty case the record must state.
const NO_HOSTED: &[ExampleHosted] = &[];

const NAS_FIELDS: &[ExampleField] = &[
    ExampleField {
        label_key: "example.field.ipv4",
        declared: Some("192.0.2.10"),
        observed: Some("192.0.2.10"),
        observed_alt: None,
        state: ObjectState::Concordant,
        values_are_keys: false,
    },
    ExampleField {
        label_key: "example.field.mac",
        declared: Some("00:00:5E:00:53:10"),
        observed: Some("00:00:5E:00:53:10"),
        observed_alt: None,
        state: ObjectState::Concordant,
        values_are_keys: false,
    },
    ExampleField {
        label_key: "example.field.role",
        declared: Some("example.role.storage"),
        observed: Some("example.role.storage"),
        observed_alt: None,
        state: ObjectState::Concordant,
        values_are_keys: true,
    },
];

const NAS_HOSTED: &[ExampleHosted] = &[
    ExampleHosted {
        name: "backup-nightly",
        kind_key: "example.hosted.service",
    },
    ExampleHosted {
        name: "media-library",
        kind_key: "example.hosted.service",
    },
];

const NAS_IDENTITY: &[ExampleIdentityPart] = &[
    ExampleIdentityPart {
        label_key: "example.identity.l2_domain",
        value: "lan-core",
    },
    ExampleIdentityPart {
        label_key: "example.identity.mac",
        value: "00:00:5E:00:53:10",
    },
    ExampleIdentityPart {
        label_key: "example.identity.mac",
        value: "00:00:5E:00:53:11",
    },
    ExampleIdentityPart {
        label_key: "example.identity.serial",
        value: "DOC-NAS-0001",
    },
];

const NAS_HISTORY: &[ExampleHistoryLine] = &[
    ExampleHistoryLine {
        when: "2026-08-19 08:14",
        what_key: "example.history.seen",
    },
    ExampleHistoryLine {
        when: "2026-08-17 04:02",
        what_key: "example.history.address_changed",
    },
    ExampleHistoryLine {
        when: "2026-07-30 11:20",
        what_key: "example.history.first_seen",
    },
];

const SWITCH_FIELDS: &[ExampleField] = &[
    ExampleField {
        label_key: "example.field.ipv4",
        declared: Some("192.0.2.3"),
        observed: Some("192.0.2.2"),
        observed_alt: None,
        state: ObjectState::Gap,
        values_are_keys: false,
    },
    ExampleField {
        label_key: "example.field.mac",
        declared: Some("00:00:5E:00:53:02"),
        observed: Some("00:00:5E:00:53:02"),
        observed_alt: None,
        state: ObjectState::Concordant,
        values_are_keys: false,
    },
    ExampleField {
        label_key: "example.field.role",
        declared: Some("example.role.network"),
        observed: Some("example.role.network"),
        observed_alt: None,
        state: ObjectState::Concordant,
        values_are_keys: true,
    },
];

const SWITCH_IDENTITY: &[ExampleIdentityPart] = &[
    ExampleIdentityPart {
        label_key: "example.identity.l2_domain",
        value: "lan-core",
    },
    ExampleIdentityPart {
        label_key: "example.identity.mac",
        value: "00:00:5E:00:53:02",
    },
];

const SWITCH_HISTORY: &[ExampleHistoryLine] = &[
    ExampleHistoryLine {
        when: "2026-08-19 08:14",
        what_key: "example.history.seen",
    },
    ExampleHistoryLine {
        when: "2026-08-19 06:00",
        what_key: "example.history.address_changed",
    },
];

const PRINTER_FIELDS: &[ExampleField] = &[
    ExampleField {
        label_key: "example.field.ipv4",
        declared: None,
        observed: Some("192.0.2.31"),
        observed_alt: None,
        state: ObjectState::Undeclared,
        values_are_keys: false,
    },
    ExampleField {
        label_key: "example.field.mac",
        declared: None,
        observed: Some("00:00:5E:00:53:31"),
        observed_alt: None,
        state: ObjectState::Undeclared,
        values_are_keys: false,
    },
];

const PRINTER_IDENTITY: &[ExampleIdentityPart] = &[
    ExampleIdentityPart {
        label_key: "example.identity.l2_domain",
        value: "lan-office",
    },
    ExampleIdentityPart {
        label_key: "example.identity.mac",
        value: "00:00:5E:00:53:31",
    },
];

const PRINTER_HISTORY: &[ExampleHistoryLine] = &[
    ExampleHistoryLine {
        when: "2026-08-19 08:12",
        what_key: "example.history.seen",
    },
    ExampleHistoryLine {
        when: "2026-08-19 08:12",
        what_key: "example.history.first_seen",
    },
];

const VM_FIELDS: &[ExampleField] = &[
    ExampleField {
        label_key: "example.field.ipv4",
        declared: Some("192.0.2.40"),
        observed: Some("192.0.2.41"),
        observed_alt: None,
        state: ObjectState::Gap,
        values_are_keys: false,
    },
    ExampleField {
        label_key: "example.field.mac",
        declared: Some("00:00:5E:00:53:40"),
        observed: Some("00:00:5E:00:53:41"),
        observed_alt: None,
        state: ObjectState::Gap,
        values_are_keys: false,
    },
    ExampleField {
        label_key: "example.field.role",
        declared: Some("example.role.application"),
        observed: Some("example.role.application"),
        observed_alt: None,
        state: ObjectState::Concordant,
        values_are_keys: true,
    },
];

const VM_HOSTED: &[ExampleHosted] = &[ExampleHosted {
    name: "billing-api",
    kind_key: "example.hosted.service",
}];

const VM_IDENTITY: &[ExampleIdentityPart] = &[
    ExampleIdentityPart {
        label_key: "example.identity.l2_domain",
        value: "lan-app",
    },
    ExampleIdentityPart {
        label_key: "example.identity.mac",
        value: "00:00:5E:00:53:41",
    },
];

const VM_HISTORY: &[ExampleHistoryLine] = &[
    ExampleHistoryLine {
        when: "2026-08-19 08:14",
        what_key: "example.history.seen",
    },
    ExampleHistoryLine {
        when: "2026-08-15 09:31",
        what_key: "example.history.address_changed",
    },
];

const CT_FIELDS: &[ExampleField] = &[
    ExampleField {
        label_key: "example.field.ipv4",
        declared: Some("192.0.2.42"),
        observed: Some("192.0.2.42"),
        observed_alt: None,
        state: ObjectState::Concordant,
        values_are_keys: false,
    },
    ExampleField {
        label_key: "example.field.role",
        declared: Some("example.role.application"),
        observed: Some("example.role.application"),
        observed_alt: None,
        state: ObjectState::Concordant,
        values_are_keys: true,
    },
];

const CT_IDENTITY: &[ExampleIdentityPart] = &[
    ExampleIdentityPart {
        label_key: "example.identity.l2_domain",
        value: "lan-app",
    },
    ExampleIdentityPart {
        label_key: "example.identity.mac",
        value: "00:00:5E:00:53:42",
    },
];

const CT_HISTORY: &[ExampleHistoryLine] = &[
    ExampleHistoryLine {
        when: "2026-08-19 08:14",
        what_key: "example.history.seen",
    },
    ExampleHistoryLine {
        when: "2026-08-01 15:05",
        what_key: "example.history.first_seen",
    },
];

const FW_FIELDS: &[ExampleField] = &[
    ExampleField {
        label_key: "example.field.ipv4",
        declared: Some("192.0.2.1"),
        observed: Some("192.0.2.1"),
        observed_alt: None,
        state: ObjectState::Concordant,
        values_are_keys: false,
    },
    ExampleField {
        label_key: "example.field.mac",
        declared: Some("00:00:5E:00:53:01"),
        observed: Some("00:00:5E:00:53:01"),
        observed_alt: Some("00:00:5E:00:53:F1"),
        state: ObjectState::Conflict,
        values_are_keys: false,
    },
];

const FW_IDENTITY: &[ExampleIdentityPart] = &[
    ExampleIdentityPart {
        label_key: "example.identity.l2_domain",
        value: "lan-edge",
    },
    ExampleIdentityPart {
        label_key: "example.identity.mac",
        value: "00:00:5E:00:53:01",
    },
];

const FW_HISTORY: &[ExampleHistoryLine] = &[
    ExampleHistoryLine {
        when: "2026-08-19 08:13",
        what_key: "example.history.seen",
    },
    ExampleHistoryLine {
        when: "2026-08-19 08:13",
        what_key: "example.history.conflicting_sources",
    },
];

const DESK_FIELDS: &[ExampleField] = &[
    ExampleField {
        label_key: "example.field.ipv4",
        declared: Some("192.0.2.77"),
        observed: Some("192.0.2.77"),
        observed_alt: None,
        state: ObjectState::Concordant,
        values_are_keys: false,
    },
    ExampleField {
        label_key: "example.field.mac",
        declared: Some("00:00:5E:00:53:77"),
        observed: Some("00:00:5E:00:53:77"),
        observed_alt: None,
        state: ObjectState::Concordant,
        values_are_keys: false,
    },
];

const DESK_IDENTITY: &[ExampleIdentityPart] = &[
    ExampleIdentityPart {
        label_key: "example.identity.l2_domain",
        value: "lan-office",
    },
    ExampleIdentityPart {
        label_key: "example.identity.mac",
        value: "00:00:5E:00:53:77",
    },
    ExampleIdentityPart {
        label_key: "example.identity.mac",
        value: "00:00:5E:00:53:78",
    },
];

const DESK_HISTORY: &[ExampleHistoryLine] = &[
    ExampleHistoryLine {
        when: "2026-08-19 07:58",
        what_key: "example.history.seen",
    },
    ExampleHistoryLine {
        when: "2026-08-19 07:58",
        what_key: "example.history.several_candidates",
    },
];

const SRV_FIELDS: &[ExampleField] = &[
    ExampleField {
        label_key: "example.field.ipv4",
        declared: Some("192.0.2.12"),
        observed: None,
        observed_alt: None,
        state: ObjectState::Gap,
        values_are_keys: false,
    },
    ExampleField {
        label_key: "example.field.mac",
        declared: Some("00:00:5E:00:53:12"),
        observed: None,
        observed_alt: None,
        state: ObjectState::Gap,
        values_are_keys: false,
    },
];

const SRV_HOSTED: &[ExampleHosted] = &[ExampleHosted {
    name: "ledger-worker",
    kind_key: "example.hosted.service",
}];

const SRV_IDENTITY: &[ExampleIdentityPart] = &[
    ExampleIdentityPart {
        label_key: "example.identity.l2_domain",
        value: "lan-core",
    },
    ExampleIdentityPart {
        label_key: "example.identity.mac",
        value: "00:00:5E:00:53:12",
    },
];

const SRV_HISTORY: &[ExampleHistoryLine] = &[
    ExampleHistoryLine {
        when: "2026-08-18 22:40",
        what_key: "example.history.last_seen",
    },
    ExampleHistoryLine {
        when: "2026-06-02 10:00",
        what_key: "example.history.first_seen",
    },
];

/// The example inventory — RFC 5737 documentation addresses and RFC 7042 documentation MACs.
///
/// ⚠️ The addresses are the ranges reserved FOR documentation on purpose: a screenshot of this
/// screen can be published, and a plausible-looking `192.168.1.x` in a manual is an address that
/// belongs to somebody.
///
/// 🔑 **Eight devices over seven kinds, and every one of the five states is represented** — the
/// filter axis and the state axis are both demonstrated by the data rather than by a caption.
pub(crate) fn devices() -> Vec<ExampleDevice> {
    vec![
        ExampleDevice {
            id: "nas-01",
            name: "nas-01",
            kind: DeviceKind::Server,
            ipv4: "192.0.2.10",
            mac: "00:00:5E:00:53:10",
            role_key: "example.role.storage",
            state: ObjectState::Concordant,
            qualifier_key: None,
            last_seen: "2026-08-19 08:14",
            fields: NAS_FIELDS,
            hosted: NAS_HOSTED,
            identity: NAS_IDENTITY,
            history: NAS_HISTORY,
        },
        ExampleDevice {
            id: "switch-core",
            name: "switch-core",
            kind: DeviceKind::Network,
            ipv4: "192.0.2.2",
            mac: "00:00:5E:00:53:02",
            role_key: "example.role.network",
            state: ObjectState::Gap,
            qualifier_key: Some("example.qualifier.one_field"),
            last_seen: "2026-08-19 08:14",
            fields: SWITCH_FIELDS,
            hosted: NO_HOSTED,
            identity: SWITCH_IDENTITY,
            history: SWITCH_HISTORY,
        },
        ExampleDevice {
            id: "printer-hall",
            name: "printer-hall",
            kind: DeviceKind::Printer,
            ipv4: "192.0.2.31",
            mac: "00:00:5E:00:53:31",
            role_key: "example.role.peripheral",
            state: ObjectState::Undeclared,
            qualifier_key: None,
            last_seen: "2026-08-19 08:12",
            fields: PRINTER_FIELDS,
            hosted: NO_HOSTED,
            identity: PRINTER_IDENTITY,
            history: PRINTER_HISTORY,
        },
        ExampleDevice {
            id: "vm-billing",
            name: "vm-billing",
            kind: DeviceKind::VirtualMachine,
            ipv4: "192.0.2.41",
            mac: "00:00:5E:00:53:41",
            role_key: "example.role.application",
            state: ObjectState::Gap,
            qualifier_key: Some("example.qualifier.two_fields"),
            last_seen: "2026-08-19 08:14",
            fields: VM_FIELDS,
            hosted: VM_HOSTED,
            identity: VM_IDENTITY,
            history: VM_HISTORY,
        },
        ExampleDevice {
            id: "ct-registry",
            name: "ct-registry",
            kind: DeviceKind::Container,
            ipv4: "192.0.2.42",
            mac: "00:00:5E:00:53:42",
            role_key: "example.role.application",
            state: ObjectState::Concordant,
            qualifier_key: None,
            last_seen: "2026-08-19 08:14",
            fields: CT_FIELDS,
            hosted: NO_HOSTED,
            identity: CT_IDENTITY,
            history: CT_HISTORY,
        },
        ExampleDevice {
            id: "fw-edge",
            name: "fw-edge",
            kind: DeviceKind::Appliance,
            ipv4: "192.0.2.1",
            mac: "00:00:5E:00:53:01",
            role_key: "example.role.security",
            state: ObjectState::Conflict,
            qualifier_key: None,
            last_seen: "2026-08-19 08:13",
            fields: FW_FIELDS,
            hosted: NO_HOSTED,
            identity: FW_IDENTITY,
            history: FW_HISTORY,
        },
        ExampleDevice {
            id: "desk-anna",
            name: "desk-anna",
            kind: DeviceKind::Workstation,
            ipv4: "192.0.2.77",
            mac: "00:00:5E:00:53:77",
            role_key: "example.role.workstation",
            state: ObjectState::Ambiguous,
            qualifier_key: None,
            last_seen: "2026-08-19 07:58",
            fields: DESK_FIELDS,
            hosted: NO_HOSTED,
            identity: DESK_IDENTITY,
            history: DESK_HISTORY,
        },
        ExampleDevice {
            id: "srv-app-02",
            name: "srv-app-02",
            kind: DeviceKind::Server,
            ipv4: "192.0.2.12",
            mac: "00:00:5E:00:53:12",
            role_key: "example.role.application",
            state: ObjectState::Gap,
            qualifier_key: Some("example.qualifier.presence"),
            last_seen: "2026-08-18 22:40",
            fields: SRV_FIELDS,
            hosted: SRV_HOSTED,
            identity: SRV_IDENTITY,
            history: SRV_HISTORY,
        },
    ]
}

/// Find the device a slug addresses, or `None` — the record route's whole lookup.
pub(crate) fn device_by_id(id: &str) -> Option<ExampleDevice> {
    devices().into_iter().find(|device| device.id == id)
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

/// One application in the example inventory — story 6b.7, Epic 15's frame.
///
/// # The vocabulary this struct carries, and the arbitration that shaped it
///
/// 🔴 **Five of the nouns this screen would name are in NO binding table**: `application`, `owner`,
/// `criticality`, `exposure` and `host`. Story 6b.6 registered the first three and deliberately did
/// not render them (`deferred-work.md:4267`); this story's own AC requires two of them, which is how
/// the register's owner assignment came to be falsified by the plan it was written under.
///
/// 🔑 **Guy's arbitration of 2026-08-20, option (c): render what the criterion NAMES and register the
/// rest.** So there is `owner` and there is `criticality_key` — and **there is no `exposure` field**,
/// because rendering a whole fourth axis with four values that no criterion asks for is what option
/// (b) would have cost. Extending the binding glossary was refused as **premature, not wrong**: it
/// would take five rows plus three value SCALES, and a scale is a new kind of row in a table whose
/// every row reads *one concept, one translation*. Epic 15 is where that closure belongs.
///
/// ⚠️ **`host` stays and is registered with the other four.** An application that runs nowhere is not
/// an application: it is FR28's containment, the device record already renders it as *Hosted here*,
/// and dropping it would gut the screen to satisfy a rule about vocabulary.
pub(crate) struct ExampleApp {
    /// What it is called — a proper noun, therefore data and not a key.
    pub(crate) name: &'static str,
    /// The slug of the device it runs on, or `None` when it runs outside the perimeter.
    ///
    /// 🔑 **An `Option`, because one row really has no host** — *Site vitrine* is hosted elsewhere,
    /// and it is the row of which AC2's *"declared and unobservable"* is most true. The first draft
    /// of this story prescribed a test requiring every host to name a device, which that row
    /// falsifies; the absence is modelled rather than papered over ([`ExampleSighting::mac`]'s
    /// precedent).
    pub(crate) host: Option<&'static str>,
    /// The version the operator documented — locale-neutral data.
    pub(crate) declared_version: &'static str,
    /// The version the host reported, or `None` when nothing evaluated it.
    pub(crate) observed_version: Option<&'static str>,
    /// Who answers for it — **data, not a key** (Guy, 2026-08-20).
    ///
    /// 🔑 An owner is a **proper noun**, like a device's [`ExampleDevice::name`]: *Comptabilité* is
    /// the name of a team, and translating it would rename a real thing. ⚠️ The opposite mistake is
    /// the one that ships silently — the validation measured that with the owners as literals
    /// **`/apps` renders five French words inside the English UI with the whole suite green**, which
    /// is story 6b.6's `role_key` defect verbatim. The distinction is *proper noun* against
    /// *classification*, and it is decided per field rather than per screen.
    pub(crate) owner: &'static str,
    /// The i18n KEY of how critical it is — **a key** (Guy, 2026-08-20).
    ///
    /// 🔑 Criticality is a **closed classification the product will one day compute**, so it is copy
    /// the operator reads in their own language. ⚠️ Its four values carry **no glossary row**, which
    /// is registered rather than introduced: see this struct's own doc.
    pub(crate) criticality_key: &'static str,
}

/// One subnet of the example IPAM screen — story 6b.7, Epic 14's frame.
///
/// # Why the occupancy is a LIST and never a formula
///
/// 🔴 `epics.md:2236` bans the mock's `(i * 37) % 256 < used`: *"a fake that varies is a fake no test
/// can pin and no screenshot can compare."* So a cell's state is a **membership lookup** against the
/// two committed lists below, never an arithmetic predicate — which makes the grid a function of data
/// a reviewer can read and a test can pin address by address.
///
/// ⚠️ The story's first draft feared the size of that (*"~110 literals … verbose but honest"*) and
/// offered to shrink the mock's numbers. Measured at validation, all three subnets come to **175
/// octets over 16 lines** after `cargo fmt`. The mock's counts are kept.
pub(crate) struct ExampleSubnet {
    /// What the query string carries for it.
    pub(crate) slug: &'static str,
    /// Its CIDR, as the operator reads it — locale-neutral data.
    pub(crate) cidr: &'static str,
    /// What the operator called it — a proper noun, therefore data (see [`ExampleApp::owner`]).
    pub(crate) name: &'static str,
    /// The first three octets, so a cell can name its own address.
    pub(crate) prefix: &'static str,
    /// The host octets that are occupied.
    pub(crate) used: &'static [u8],
    /// The host octets held back — a DHCP range, a static pool.
    pub(crate) reserved: &'static [u8],
}

/// The four states a cell of the occupancy grid can be in.
///
/// 🔴 **`Structural` exists because a /24 carries 256 ADDRESSES and 254 HOSTS**, and the mock does
/// not: it loops `for i = 0..256`, draws `.0` and `.255` as ordinary free cells, and its *next free
/// address* panel then names the NETWORK address. The validation reproduced that on a real build
/// before the defect was inherited. ⚠️ And the story's own first draft prescribed a test asserting
/// the counts *"sum to 256"*, which would have pinned the defect as the expected behaviour — *a test
/// that pins the ugly thing is a test that requires it* (story 6b.4).
///
/// 🔑 The occupancy line therefore counts over the **254 hosts**, and the two structural cells are in
/// none of its three numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CellState {
    /// Something answers there.
    Used,
    /// Held back deliberately — a DHCP range or a static pool.
    Reserved,
    /// Available.
    Free,
    /// The network or the broadcast address: not a host, and not the operator's to assign.
    Structural,
}

impl CellState {
    /// The CSS modifier this state renders through.
    ///
    /// 🔴 **A `&'static str` chosen by a `match`, and the classes in the template are LITERALS.**
    /// Epic 6b's constraint 5: a class *built* in Rust is invisible to
    /// `every_class_a_template_names_is_defined_in_the_stylesheet`, which skips any `class="…"`
    /// containing `{`. The shipped specimen is `_dashboard.html:58`'s `spark-h{{ height }}`, whose
    /// `.spark-h8` was missing from the sheet for a whole story. See the legend in
    /// `_ipam_example.html` for the deliberate redundancy that carries this guard.
    pub(crate) fn modifier(self) -> &'static str {
        match self {
            CellState::Used => "ipam-cell-used",
            CellState::Reserved => "ipam-cell-reserved",
            CellState::Free => "ipam-cell-free",
            CellState::Structural => "ipam-cell-structural",
        }
    }

    /// The i18n KEY of the word that names this state to a screen reader and in the legend.
    pub(crate) fn label_key(self) -> &'static str {
        match self {
            CellState::Used => "ipam.state.used",
            CellState::Reserved => "ipam.state.reserved",
            CellState::Free => "ipam.state.free",
            CellState::Structural => "ipam.state.structural",
        }
    }
}

impl ExampleSubnet {
    /// What state the host octet `octet` is in.
    ///
    /// 🔑 A lookup, never a predicate — see this struct's own doc for the ban it obeys.
    pub(crate) fn state_of(&self, octet: u8) -> CellState {
        if octet == 0 || octet == 255 {
            CellState::Structural
        } else if self.used.contains(&octet) {
            CellState::Used
        } else if self.reserved.contains(&octet) {
            CellState::Reserved
        } else {
            CellState::Free
        }
    }

    /// The lowest host address nothing occupies, or `None` when the subnet is full.
    ///
    /// 🔑 **Derived from the same lists the grid renders**, so the address this names cannot be one
    /// the grid draws as occupied — which a test pins. ⚠️ It starts at 1 and stops at 254: the mock's
    /// `findIndex` started at 0 and answered with the network address.
    pub(crate) fn next_free(&self) -> Option<u8> {
        (1..=254).find(|octet| self.state_of(*octet) == CellState::Free)
    }

    /// How many host addresses are in each of the three operator-facing states.
    ///
    /// # Returns
    ///
    /// `(used, reserved, free)`, counted over the **254 hosts** — the structural pair is in none of
    /// them. 🔑 The occupancy line renders these, so the line and the grid are **two representations
    /// of one fact**: a deliberate redundancy in the sense `CLAUDE.md` sanctions, and pinned by a
    /// test rather than trusted. The mock computes its line from independent scalars and can
    /// therefore disagree with the cells it drew.
    pub(crate) fn occupancy(&self) -> (usize, usize, usize) {
        let mut counts = (0, 0, 0);
        for octet in 1..=254u8 {
            match self.state_of(octet) {
                CellState::Used => counts.0 += 1,
                CellState::Reserved => counts.1 += 1,
                CellState::Free => counts.2 += 1,
                CellState::Structural => unreachable!("1..=254 excludes the structural pair"),
            }
        }
        counts
    }
}

/// The office subnet's occupied hosts — **including every address the inventory ships**, so the two
/// screens describe one network rather than two.
const OFFICE_USED: &[u8] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 31, 41, 42, 51, 52, 53,
    54, 55, 56, 57, 58, 59, 60, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 101, 102, 103, 104, 105,
    106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124,
    125, 126, 127, 128, 129, 130, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213,
    214, 215, 216, 217, 218, 219, 220, 221, 222, 223,
];

/// The office subnet's DHCP range.
const OFFICE_RESERVED: &[u8] = &[
    240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253,
];

/// The workshop subnet's occupied hosts.
const WORKSHOP_USED: &[u8] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 100, 101, 102,
    103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118,
];

/// The workshop subnet's DHCP range.
const WORKSHOP_RESERVED: &[u8] = &[247, 248, 249, 250, 251, 252, 253, 254];

/// The guest subnet's occupied hosts.
const GUEST_USED: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

/// The guest subnet's DHCP range.
const GUEST_RESERVED: &[u8] = &[251, 252, 253, 254];

/// The example subnets, in the mock's order.
///
/// ⚠️ **Three RFC 5737 blocks, one per subnet** — TEST-NET-1, TEST-NET-2 and TEST-NET-3 — for the
/// reason the module doc gives: a published screenshot must name nobody's network. The mock uses
/// `192.168.10/20/30`, which are addresses that belong to somebody.
pub(crate) fn subnets() -> Vec<ExampleSubnet> {
    vec![
        ExampleSubnet {
            slug: "office",
            cidr: "192.0.2.0/24",
            name: "Bureau",
            prefix: "192.0.2",
            used: OFFICE_USED,
            reserved: OFFICE_RESERVED,
        },
        ExampleSubnet {
            slug: "workshop",
            cidr: "198.51.100.0/24",
            name: "Atelier",
            prefix: "198.51.100",
            used: WORKSHOP_USED,
            reserved: WORKSHOP_RESERVED,
        },
        ExampleSubnet {
            slug: "guest",
            cidr: "203.0.113.0/24",
            name: "Invités",
            prefix: "203.0.113",
            used: GUEST_USED,
            reserved: GUEST_RESERVED,
        },
    ]
}

/// The subnet a slug names, or `None` when it names none.
pub(crate) fn subnet_by_slug(slug: &str) -> Option<ExampleSubnet> {
    subnets().into_iter().find(|subnet| subnet.slug == slug)
}

/// The example address conflict — FR24, and the pair the IPAM panel names.
///
/// 🔴 **The panel is titled *« Conflit d'adresse »* and the qualifier is load-bearing** (Guy,
/// 2026-08-20). The binding glossary's `conflict` row means *two observations disagree with each
/// other — source against source*; **two appliances answering on one address is a different fact**,
/// and `prd.md:988` forbids one word carrying two meanings. The qualifier disambiguates without
/// minting a term, exactly as *"Écart · 2 champs"* qualifies `écart`. The collision itself is
/// registered, not settled here.
///
/// 🔑 Both devices EXIST in [`devices()`] and the address is one the office grid draws as **used** —
/// the mock names a ninth device that is in no dataset, at an address its own grid may not occupy.
/// ⚠️ Neither device carries [`ObjectState::Conflict`], deliberately: that state is the OTHER
/// conflict, and demonstrating both on one pair would blur the distinction the title exists to make.
pub(crate) struct ExampleAddressConflict {
    /// The disputed address.
    pub(crate) ipv4: &'static str,
    /// The slug of the device declared at it.
    pub(crate) declared_device: &'static str,
    /// The slug of the device also answering there.
    pub(crate) observed_device: &'static str,
    /// When the second device's lease was seen — locale-neutral data.
    pub(crate) lease_seen: &'static str,
}

/// The address conflict the example IPAM screen shows.
pub(crate) fn address_conflict() -> ExampleAddressConflict {
    ExampleAddressConflict {
        ipv4: "192.0.2.41",
        declared_device: "vm-billing",
        observed_device: "desk-anna",
        lease_seen: "2026-08-19 14:02",
    }
}

/// The example applications, in the mock's order.
///
/// 🔑 **Every host names a device [`devices()`] really ships.** The mock's own hosts (`DOCKER-01`,
/// `VM-AD-01`, …) name machines that exist in no dataset here, so seven of the eight rows are
/// re-pointed; the eighth has no host at all and says so.
pub(crate) fn apps() -> Vec<ExampleApp> {
    vec![
        ExampleApp {
            name: "Nextcloud",
            host: Some("ct-registry"),
            declared_version: "28.0.4",
            observed_version: Some("29.0.1"),
            owner: "Direction",
            criticality_key: "example.criticality.high",
        },
        ExampleApp {
            name: "Active Directory",
            host: Some("srv-app-02"),
            declared_version: "2022",
            // 🔑 The one row nothing evaluated — an absence, resolved in the view layer.
            observed_version: None,
            owner: "Prestataire IT",
            criticality_key: "example.criticality.critical",
        },
        ExampleApp {
            name: "Sage 50",
            host: Some("vm-billing"),
            declared_version: "2024.1",
            observed_version: Some("2024.1"),
            owner: "Comptabilité",
            criticality_key: "example.criticality.critical",
        },
        ExampleApp {
            name: "Traefik",
            host: Some("ct-registry"),
            declared_version: "3.0",
            observed_version: Some("3.0.4"),
            owner: "Prestataire IT",
            criticality_key: "example.criticality.high",
        },
        ExampleApp {
            name: "Sauvegarde Hyper Backup",
            host: Some("nas-01"),
            declared_version: "4.1",
            observed_version: Some("4.2"),
            owner: "Direction",
            criticality_key: "example.criticality.critical",
        },
        ExampleApp {
            name: "GLPI",
            host: Some("ct-registry"),
            declared_version: "10.0.15",
            observed_version: Some("10.0.15"),
            owner: "Prestataire IT",
            criticality_key: "example.criticality.medium",
        },
        ExampleApp {
            name: "Supervision caméras",
            host: Some("desk-anna"),
            declared_version: "5.2",
            observed_version: Some("5.2"),
            owner: "Atelier",
            criticality_key: "example.criticality.low",
        },
        ExampleApp {
            name: "Site vitrine",
            // 🔴 Hosted OUTSIDE the perimeter, on no device at all — the row of which AC2's
            // *"declared and unobservable"* is most true, and the counterexample to any test
            // requiring every application to name a device.
            host: None,
            declared_version: "2024",
            observed_version: None,
            owner: "Marketing",
            criticality_key: "example.criticality.low",
        },
    ]
}

/// One row of the example alert list — Epic 16's frame (story 6b.8).
///
/// # 🔑 The three kinds come from FR30, not from the mock
///
/// The reference mock invents severities (*critical · high · …*), which would be **a fourth value
/// set with no glossary row** — the shape story 6b.7 arbitrated one story earlier, and the register
/// already carries five nouns and three value sets owed to Epic 15. `prd.md:941` names the three
/// alerts the product will actually raise: *an unknown device appearing, a documented IP unseen for
/// N days, and an IP conflict.* **Those are the rows**, and they introduce no vocabulary the plan
/// does not already bind.
///
/// ⚠️ **No severity field.** Not an omission: a severity is a judgement, and the product has no
/// producer for one.
pub(crate) struct ExampleAlert {
    /// The i18n KEY of what happened — a key, for the reason given on [`ExampleDevice::role_key`].
    pub(crate) what_key: &'static str,
    /// What it is about — an address or a device name, therefore locale-neutral data.
    pub(crate) subject: &'static str,
    /// When, as an ISO-ish instant: locale-neutral, so it needs no key.
    pub(crate) when: &'static str,
}

/// The example alerts, one per kind FR30 names.
pub(crate) fn alerts() -> Vec<ExampleAlert> {
    vec![
        ExampleAlert {
            what_key: "example.alert.unknown_device",
            subject: "192.0.2.58",
            when: "2026-08-19 08:14",
        },
        ExampleAlert {
            what_key: "example.alert.unseen",
            subject: "printer-hall",
            when: "2026-08-18 22:40",
        },
        ExampleAlert {
            what_key: "example.alert.address_conflict",
            subject: "192.0.2.41",
            when: "2026-08-19 14:02",
        },
    ]
}

/// One step of the example commissioning walk-through.
///
/// 🔴 **The mock frames this as an ONBOARDING and the UX specification corrects that framing**:
/// *"bootstrap is a MODE, not an onboarding. Filing it under 'first run' was a design error: the
/// wall recurs on every large migration… the baselining flow stays available for life, gated by
/// VOLUME, never by a `first_run` flag"* (F11). So the copy here speaks of a **baseline that can be
/// adopted whenever the volume calls for it**, never of a first day.
///
/// ⚠️ **Every field that an operator READS is a key** — story 6b.6 measured a French UI rendering
/// English literals with the whole suite green, and story 6b.3 measured a real key from the wrong
/// namespace resolving to the wrong word.
pub(crate) struct ExampleStep {
    /// The step's ordinal, as the mock shows it (`01`…`04`). Locale-neutral.
    pub(crate) number: &'static str,
    /// The i18n key of the step's title.
    pub(crate) title_key: &'static str,
    /// The i18n key of the line under it.
    pub(crate) detail_key: &'static str,
    /// The i18n key of its status word.
    pub(crate) status_key: &'static str,
}

/// The example commissioning steps — Epic 9's frame (story 6b.9).
///
/// ⚠️ The addresses are RFC 5737 documentation space, as everywhere in this dataset, so a published
/// screenshot names nobody's network.
pub(crate) fn commissioning_steps() -> Vec<ExampleStep> {
    vec![
        ExampleStep {
            number: "01",
            title_key: "example.step.database",
            detail_key: "example.step.database.detail",
            status_key: "example.step.done",
        },
        ExampleStep {
            number: "02",
            title_key: "example.step.source",
            detail_key: "example.step.source.detail",
            status_key: "example.step.done",
        },
        ExampleStep {
            number: "03",
            title_key: "example.step.perimeter",
            detail_key: "example.step.perimeter.detail",
            status_key: "example.step.done",
        },
        ExampleStep {
            number: "04",
            title_key: "example.step.first_pass",
            detail_key: "example.step.first_pass.detail",
            status_key: "example.step.done",
        },
    ]
}

/// The example baseline block: what an initial discovery would offer to adopt.
///
/// 🔑 **The three figures partition**: total = consistent + divergent, and the template asserts it
/// nowhere because [`crate::example_screens::commissioning_body`]'s own test does. A demonstration
/// whose numbers do not add up teaches the operator to distrust the real ones.
pub(crate) struct ExampleBaseline {
    /// Objects observed by the initial discovery.
    pub(crate) total: u32,
    /// Those the baselining can adopt without a decision.
    pub(crate) consistent: u32,
    /// Those carrying a divergence the baselining cannot settle alone.
    pub(crate) divergent: u32,
}

/// The example baseline figures.
pub(crate) fn commissioning_baseline() -> ExampleBaseline {
    ExampleBaseline {
        total: 412,
        consistent: 383,
        divergent: 29,
    }
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
    /// Every application runs on a device this dataset really ships — or on none, and says so.
    ///
    /// 🔴 **Seven of the mock's eight hosts name machines that exist in no dataset here**
    /// (`DOCKER-01`, `VM-AD-01`, …), so they are re-pointed. Without this guard the applications
    /// screen would link to `/devices/DOCKER-01`, which serves the *unknown device* page — a 200,
    /// therefore invisible to every status check.
    #[test]
    fn every_application_runs_on_a_device_that_exists_or_on_none() {
        let slugs: Vec<&str> = devices().iter().map(|device| device.id).collect();
        let mut hosted = 0_usize;
        for app in apps() {
            let Some(host) = app.host else { continue };
            assert!(
                slugs.contains(&host),
                "{} is hosted on {host:?}, which names no device — the row would link to the \
                 unknown-device page, and that answers 200",
                app.name
            );
            hosted += 1;
        }
        // 🔑 The `None` row is the point of the `Option`, so BOTH cases are asserted present: a
        // dataset where every host were `None` would pass the loop above and demonstrate nothing.
        assert_eq!(
            hosted,
            apps().len() - 1,
            "exactly one application has no host"
        );
        assert!(
            apps().iter().any(|app| app.host.is_none()),
            "the dataset must keep the application hosted outside the perimeter — it is the row \
             of which *declared and unobservable* is most true"
        );
    }

    /// 🔴 **The two lists are DISJOINT, and this guard exists because a mutation came back green.**
    ///
    /// [`ExampleSubnet::state_of`] tests `used` before `reserved`, so an octet in both is silently
    /// resolved as *used*: the mutation that put `41` into the office subnet's reserved list — a
    /// deliberate corruption of the data — changed no cell, no count and no test. **The lists are
    /// not orthogonal, and a priority order hides a contradiction in the dataset rather than
    /// showing it.**
    ///
    /// 🔑 What that measured is not that the guards are weak but that the DATA can be incoherent
    /// without saying so. A subnet claiming an address is both occupied and held back is a defect;
    /// the priority order is a rendering decision and must not double as a repair.
    #[test]
    fn no_octet_is_both_occupied_and_reserved() {
        for subnet in subnets() {
            for octet in subnet.reserved {
                assert!(
                    !subnet.used.contains(octet),
                    "{}: .{octet} is in both lists. `state_of` resolves it silently as `used`, so \
                     the contradiction renders as an ordinary cell and no count moves",
                    subnet.cidr
                );
            }
        }
    }

    /// The occupancy line's three numbers are counts over the same data the grid draws.
    ///
    /// 🔴 **A /24 carries 256 ADDRESSES and 254 HOSTS**, and the mock conflates them: it draws
    /// `.0` and `.255` as ordinary free cells and computes its line as `256 - used - reserved` from
    /// independent scalars, which can disagree with the cells it drew. ⚠️ This story's own first
    /// draft prescribed a test asserting the three numbers *"sum to 256"* — which would have pinned
    /// the defect as the expected behaviour.
    #[test]
    fn the_occupancy_line_counts_the_same_cells_the_grid_draws() {
        for subnet in subnets() {
            let (used, reserved, free) = subnet.occupancy();
            let drawn = |wanted: CellState| {
                (0..=255u8)
                    .filter(|octet| subnet.state_of(*octet) == wanted)
                    .count()
            };
            assert_eq!(used, drawn(CellState::Used), "{}: used", subnet.cidr);
            assert_eq!(
                reserved,
                drawn(CellState::Reserved),
                "{}: reserved",
                subnet.cidr
            );
            assert_eq!(free, drawn(CellState::Free), "{}: free", subnet.cidr);
            assert_eq!(
                used + reserved + free,
                254,
                "{}: the three counts cover the HOSTS, and a /24 has 254 of them — not 256",
                subnet.cidr
            );
            assert_eq!(
                drawn(CellState::Structural),
                2,
                "{}: the network and broadcast addresses are drawn and counted apart",
                subnet.cidr
            );
        }
    }

    /// The address the *next free* panel names is one the grid draws as free, and is a host.
    ///
    /// 🔴 The mock's `findIndex` starts at 0, so its panel names the NETWORK address. Reproduced on
    /// a real build at this story's validation before it could be inherited.
    #[test]
    fn the_next_free_address_is_a_free_host() {
        for subnet in subnets() {
            let octet = subnet.next_free().expect("no example subnet is full");
            assert_eq!(
                subnet.state_of(octet),
                CellState::Free,
                "{}: the panel would name an address the grid draws as occupied",
                subnet.cidr
            );
            assert!(
                (1..=254).contains(&octet),
                "{}: {octet} is the network or the broadcast address, which is not the \
                 operator's to assign",
                subnet.cidr
            );
            // The LOWEST free host, not merely some free host: a scan that skipped one would still
            // satisfy the two assertions above.
            for lower in 1..octet {
                assert_ne!(
                    subnet.state_of(lower),
                    CellState::Free,
                    "{}: {lower} is free and lower than the address the panel names",
                    subnet.cidr
                );
            }
        }
    }

    /// A subnet with no free host answers `None`, and the copy for it exists.
    ///
    /// ⚠️ **The RENDER of that copy is carried by nothing, and saying so is the point.** No committed
    /// subnet is full, so `ipam.next_free_none` reaches no screen in any test — a review layer
    /// measured the path unreachable rather than merely untested. Putting a full subnet in the
    /// example dataset to exercise one sentence would be shaping the demonstration around the test,
    /// so what is guarded here is the FUNCTION; the sentence's rendering waits for Epic 14, where a
    /// full subnet is an ordinary state rather than a fixture. **Registered.**
    #[test]
    fn a_full_subnet_has_no_next_free_address() {
        let full: Vec<u8> = (1..=254).collect();
        let subnet = ExampleSubnet {
            slug: "full",
            cidr: "192.0.2.0/24",
            name: "Plein",
            prefix: "192.0.2",
            used: full.leak(),
            reserved: &[],
        };
        assert_eq!(subnet.next_free(), None, "a full subnet offers no address");
        let (used, reserved, free) = subnet.occupancy();
        assert_eq!((used, reserved, free), (254, 0, 0));
        // The control: the committed subnets are NOT full, so the assertion above is not satisfied
        // by a `next_free` that answers `None` for everything.
        for committed in subnets() {
            assert!(
                committed.next_free().is_some(),
                "{}: the example subnets must keep a free address, or the panel this story ships \
                 shows its empty state on every screen",
                committed.cidr
            );
        }
    }

    /// The conflict panel names two devices that exist, at an address the grid draws as used.
    ///
    /// 🔑 The mock names `PC-COMPTA-02`, a ninth device in no dataset, at an address its own grid
    /// may not occupy. ⚠️ And **neither device carries [`ObjectState::Conflict`]** on purpose: that
    /// state is the OTHER conflict — *source against source* — and demonstrating both on one pair
    /// would blur the very distinction the panel's qualified title exists to make.
    #[test]
    fn the_conflict_names_devices_that_exist_at_an_address_the_grid_occupies() {
        let conflict = address_conflict();
        for slug in [conflict.declared_device, conflict.observed_device] {
            assert!(
                device_by_id(slug).is_some(),
                "the conflict names {slug:?}, which is no device"
            );
            assert_ne!(
                device_by_id(slug).expect("just checked").state,
                ObjectState::Conflict,
                "{slug:?} carries the state `conflict`, which is source-against-source — using it \
                 here blurs the distinction the panel's title is qualified to make"
            );
        }
        assert_ne!(
            conflict.declared_device, conflict.observed_device,
            "a device does not conflict with itself"
        );
        let office = subnet_by_slug("office").expect("the office subnet");
        let octet: u8 = conflict
            .ipv4
            .rsplit('.')
            .next()
            .and_then(|last| last.parse().ok())
            .expect("the conflict address ends in an octet");
        assert!(
            conflict.ipv4.starts_with(office.prefix),
            "the conflict must sit in a subnet the grid draws"
        );
        assert_eq!(
            office.state_of(octet),
            CellState::Used,
            "the grid must draw the disputed address as occupied — a conflict on a cell shown \
             free is a screen contradicting itself"
        );
    }

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
        // 🔴 **EVERY key-bearing field, and it was TWO until story 6b.7.** This loop read
        // `role_key` and `reason_key` only, while the dataset already carried five more —
        // `DeviceKind::label_key`, `ExampleField::label_key`, `ExampleHosted::kind_key`,
        // `ExampleIdentityPart::label_key` and `ExampleHistoryLine::what_key` — none of them
        // covered. ⚠️ It read as strong coverage because it is the FIRST thing to red when a
        // `role_key` is broken; an enumeration that catches the case you test is exactly the shape
        // *"a guard placed where the defect cannot occur"* takes when the defect moves one field
        // over. Closing the five is in scope here because this story is what makes the hole
        // load-bearing: `criticality_key` would have been the eighth uncovered field.
        for kind in DeviceKind::ALL {
            assert_key(kind.label_key(), "example.kind.");
        }
        for device in devices() {
            assert_key(device.role_key, "example.role.");
            if let Some(qualifier) = device.qualifier_key {
                assert_key(qualifier, "example.qualifier.");
            }
            for field in device.fields {
                assert_key(field.label_key, "example.field.");
            }
            for hosted in device.hosted {
                assert_key(hosted.kind_key, "example.hosted.");
            }
            for part in device.identity {
                assert_key(part.label_key, "example.identity.");
            }
            for line in device.history {
                assert_key(line.what_key, "example.history.");
            }
        }
        for sighting in unplaced_sightings() {
            assert_key(sighting.reason_key, "example.reason.");
        }
        for app in apps() {
            // 🔑 `criticality_key` and NOT `owner`: Guy's arbitration of 2026-08-20 makes
            // criticality a closed classification (a key) and an owner a proper noun (data). The
            // distinction is decided per field, and this loop is where the key half is enforced.
            assert_key(app.criticality_key, "example.criticality.");
        }
        assert!(
            checked >= 80,
            "the premise: the dataset carries at least eighty translated strings ({checked} seen) \
             — **87 at story 6b.7**, over the seven field kinds now covered rather than the two \
             this guard used to read. ⚠️ A floor is only a guard while it is near what is there: \
             `every_key_carries_both_locales` sits at 47 for a file carrying 184, which is \
             registered. This one is set just under the measured figure so it catches a mass \
             deletion without reddening when one row moves"
        );
    }
}
