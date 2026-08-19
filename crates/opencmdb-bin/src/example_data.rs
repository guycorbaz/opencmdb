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
