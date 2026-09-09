//! `/sources` — what this instance's one source can and cannot see, shaped for the page.
//!
//! # Why this is a module of its own
//!
//! 🔴 **`page.rs` reached the `file-size` gate's 2000-line ceiling again** — at 2016, with the gate
//! naming it — when the reverse-DNS story gave the connector a third fact kind. `CLAUDE.md`'s rule
//! is *split, not grown*, and this is the split, on `identity_view.rs`'s precedent (story 6.4): the
//! sources screen has its own types, its own `sources.*` locale namespace, its own producer
//! (`arp_ping::observes_and_cannot_see`) and its own doctrine, so it comes out whole rather than by
//! line count. ⚠️ **Its TESTS stay in `page.rs`**, where the render helpers they share with the rest
//! of the page live; the ceiling counts the lines before the first `#[cfg(test)]`, so a test module
//! here would have bought nothing.
//!
//! # What the screen is FOR, in one sentence
//!
//! It reports what the source is BUILT to observe and what it is built NOT to observe — FR7's
//! static half — so that an operator asking *why is it not doing more?* gets an answer instead of
//! a silence.

use askama::Template;
use opencmdb_core::observation::FactKind;

use crate::page::relative_time;

/// One capability line: a fact kind the source does or does not observe, with its sentence.
pub(crate) struct KindLine {
    /// The kind's name, in the operator's language.
    pub(crate) label: String,
    /// What its presence or absence MEANS to the operator — the unlock framing, never the fault one.
    pub(crate) meaning: String,
}

/// Everything `/sources` renders: the product's real capability boundary, and its real freshness.
pub(crate) struct SourceView {
    /// The source's name, resolved — a TYPE name (see [`crate::arp_ping::SOURCE_NAME_KEY`]).
    pub(crate) name: String,
    /// Whether the product REFUSED this perimeter — measured with the connector's own parser.
    ///
    /// 🔴 A refused perimeter shown as an in-force one is the sharpest thing the code review found
    /// on this screen: the product already knows the configuration is bad, and said so in a log
    /// nobody reads. See [`build_sources`].
    pub(crate) refused: bool,
    /// The perimeter it was configured with.
    ///
    /// 🔑 **Not an `Option`, and the code review is why.** `build_sources` returns `None` outright
    /// when no perimeter is configured, so inside a `SourceView` this could never be absent — and
    /// the template carried a `when None` arm that could never execute. *A branch placed where the
    /// case cannot occur reads as handling and is none.*
    pub(crate) perimeter: String,
    /// The resolver the scan asks for a host's NAME, already rendered — the configured address,
    /// or the words for *the system resolver*.
    ///
    /// 🔴 **The boot refusal for `OPENCMDB_DNS_SERVER` is justified in the code by *"no screen
    /// anywhere would tell you otherwise"*, and the 2026-09-09 review measured that this stayed
    /// true after the variable shipped: the value reached the connector and no screen at all.**
    /// It is the difference between 2 names and 37 on the story's own field measurement, so the
    /// operator has to be able to see which resolver is being asked.
    pub(crate) resolver: String,
    /// What it is built to observe.
    pub(crate) observes: Vec<KindLine>,
    /// What it is built NOT to observe — the section AC1 requires to be real.
    pub(crate) cannot_see: Vec<KindLine>,
    /// How long ago anything was observed, or `None`.
    ///
    /// 🔴 **`None` is FOUR different states of the world and the screen says so.** Story 6b.8's
    /// validation booted the real binary four times against four fresh databases: never scanned,
    /// scanned-and-nobody-answered, an INVALID perimeter the product refused, and a blank one — all
    /// four leave `MAX(observed_at)` NULL. FR8's own distinction fails at boot level, so the copy
    /// states the ambiguity instead of picking one reading.
    pub(crate) last_observed: Option<String>,
}

/// The sources screen's body.
#[derive(Template)]
#[template(path = "_sources.html")]
pub(crate) struct SourcesBody {
    /// `None` when no source is configured at all — the case the story's first draft assumed away.
    pub(crate) source: Option<SourceView>,
    pub(crate) s: SourceStrings,
}

/// The copy `/sources` needs, resolved once.
pub(crate) struct SourceStrings {
    pub(crate) title: String,
    pub(crate) lede: String,
    pub(crate) observes_title: String,
    pub(crate) cannot_see_title: String,
    pub(crate) unlock: String,
    pub(crate) freshness_title: String,
    pub(crate) never: String,
    pub(crate) ambiguity: String,
    pub(crate) incident_axis: String,
    pub(crate) perimeter_label: String,
    /// The label of the resolver line.
    pub(crate) resolver_label: String,
    pub(crate) no_source: String,
    /// What the screen says when the product REFUSED the configured perimeter.
    pub(crate) refused: String,
}

/// Resolve `/sources`' copy.
pub(crate) fn source_strings() -> SourceStrings {
    SourceStrings {
        title: rust_i18n::t!("sources.title").to_string(),
        lede: rust_i18n::t!("sources.lede").to_string(),
        observes_title: rust_i18n::t!("sources.observes").to_string(),
        cannot_see_title: rust_i18n::t!("sources.cannot_see").to_string(),
        unlock: rust_i18n::t!("sources.unlock").to_string(),
        freshness_title: rust_i18n::t!("sources.freshness").to_string(),
        never: rust_i18n::t!("sources.never").to_string(),
        ambiguity: rust_i18n::t!("sources.ambiguity").to_string(),
        incident_axis: rust_i18n::t!("sources.incident_axis").to_string(),
        perimeter_label: rust_i18n::t!("sources.perimeter").to_string(),
        resolver_label: rust_i18n::t!("sources.resolver").to_string(),
        no_source: rust_i18n::t!("sources.no_source").to_string(),
        refused: rust_i18n::t!("sources.refused").to_string(),
    }
}

/// The i18n keys of one [`FactKind`]'s name and of what it means to the operator.
///
/// 🔑 **A `match` on a `FactKind`, and the `_` arm is FORCED by `#[non_exhaustive]`** — the compiler
/// cannot carry exhaustiveness across the crate boundary, so a wildcard is mandatory and is then
/// permanently silent.
///
/// 🔴 **The fallback returns a GENERIC pair, and the doc said *"the kind's `Debug` name"* until the
/// code review caught it two lines above the code that refutes it.** Every unmapped kind would render
/// *identically* — *"Unrecognised kind"* — with nothing on the page telling an operator or a
/// log-reading developer WHICH one appeared.
///
/// ⚠️ **And `FactKind::ALL`'s cross-crate guard does not protect THIS map.** It pins `ALL` against the
/// enum's declaration; it says nothing about whether each member has a key pair here. So an eighth
/// kind correctly added to `ALL` would satisfy that guard and still render *"Genre non reconnu"* on
/// `/sources` — *a guard placed where the defect cannot occur*, one field over. Closed by
/// [`crate::page::tests::every_fact_kind_has_its_own_sentence`], which reds on exactly that.
pub(crate) fn kind_keys(kind: FactKind) -> (&'static str, &'static str) {
    match kind {
        FactKind::Mac => ("kind.mac", "kind.mac.meaning"),
        FactKind::IpV4 => ("kind.ipv4", "kind.ipv4.meaning"),
        FactKind::Hostname => ("kind.hostname", "kind.hostname.meaning"),
        FactKind::DhcpLease => ("kind.dhcp_lease", "kind.dhcp_lease.meaning"),
        FactKind::Uplink => ("kind.uplink", "kind.uplink.meaning"),
        FactKind::OuiVendor => ("kind.oui_vendor", "kind.oui_vendor.meaning"),
        FactKind::Rtt => ("kind.rtt", "kind.rtt.meaning"),
        _ => ("kind.unknown", "kind.unknown.meaning"),
    }
}

/// Build one capability line.
pub(crate) fn kind_line(kind: FactKind) -> KindLine {
    let (label, meaning) = kind_keys(kind);
    KindLine {
        label: rust_i18n::t!(label).to_string(),
        meaning: rust_i18n::t!(meaning).to_string(),
    }
}

/// Build the sources view. Pure: the caller supplies the instant and the perimeter.
///
/// ⚠️ **No clock here** — `now` is a parameter, on the precedent of every view builder in this file.
pub(crate) fn build_sources(
    perimeter: Option<String>,
    dns_server: Option<std::net::IpAddr>,
    last: Option<chrono::DateTime<chrono::Utc>>,
    now: chrono::DateTime<chrono::Utc>,
) -> Option<SourceView> {
    // 🔴 **NO PERIMETER, NO SOURCE.** The story's first draft assumed one configured source
    // throughout; the validation measured that with `OPENCMDB_SCAN_CIDR` unset there are ZERO, and
    // the screen had no copy for it. Listing a source the product was never asked to build would be
    // the same fabrication the liveness arbitration refuses.
    // 🔑 The `?` on the field below is the ONLY place this is decided — it read `perimeter.as_ref()?`
    // here as well, and two refusals of one fact drift.
    let (observes, cannot_see) = crate::arp_ping::observes_and_cannot_see();
    // 🔴 **A PERIMETER THE PRODUCT REFUSED IS NOT A PERIMETER, and the screen said otherwise.**
    // Measured at the code review by booting with `OPENCMDB_SCAN_CIDR=nonsense`: the log carried
    // `ERROR invalid OPENCMDB_SCAN_CIDR — skipping scan`, and this screen rendered a full source
    // card reading *"Périmètre nonsense"* with the generic four-state sentence under it — **the
    // rejected string PRESENTED AS AN IN-FORCE VALUE**. That is worse than the ambiguity the story
    // registered: it is not *we cannot tell which of four*, it is *we are showing you a
    // configuration we already refused, as though it were live*.
    //
    // 🔑 `subnet_hosts` is the SAME parser the connector uses, so the screen and the scan agree by
    // construction rather than by two readings of one rule. ⚠️ `AppConfig::from_env` still does not
    // validate the CIDR — the refusal happens in a detached thread whose error nobody reads — and
    // moving it to boot time is registered rather than done here.
    let refused = crate::arp_ping::subnet_hosts(perimeter.as_deref().unwrap_or_default()).is_err();
    Some(SourceView {
        name: rust_i18n::t!(crate::arp_ping::SOURCE_NAME_KEY).to_string(),
        refused,
        perimeter: perimeter?,
        // Unset is not blank: the product asks the machine's own resolver, and saying so is not
        // the same as saying nothing. `AppConfig` already refused anything unparseable at boot,
        // so whatever arrives here is an address the operator chose.
        resolver: dns_server.map_or_else(
            || rust_i18n::t!("sources.resolver_system").to_string(),
            |server| server.to_string(),
        ),
        observes: observes.into_iter().map(kind_line).collect(),
        cannot_see: cannot_see.into_iter().map(kind_line).collect(),
        last_observed: last.map(|instant| relative_time(now, instant)),
    })
}
