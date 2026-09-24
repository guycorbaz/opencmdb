//! L2 — device grouping, and the rules that argue about it.
//!
//! L1 asks whether two observations are the same INTERFACE; L2 asks whether two interfaces are the
//! same DEVICE [architecture.md:891]. The trap matrix splits on exactly that line: *"multi-NIC
//! false-split = L1 correct, **L2 failed to group**"* [architecture.md:893].
//!
//! # What lives here, and what deliberately does not
//!
//! The RULES. The candidate generator stays in [`crate::identity::blocking`] — *a blocker that
//! consults a rule is that rule's echo* — and keeping the two in different files is what makes that
//! separation visible while reading. ⚠️ **It is visibility, not constraint**: story 6.6 measured
//! that adding a [`crate::identity::cascade::decide`] call inside the blocker leaves the whole
//! suite, clippy and every gate green. Read the separation as a convention, never as a guarantee.
//!
//! # 🔴 A rule needs FACTS, and an `L2CandidatePair` carries none
//!
//! [`crate::identity::blocking::L2CandidatePair`] holds two [`crate::identity::l1::L1Key`]s, and an
//! `L1Key` is `(L2DomainId, MacAddr)` — no hostname, no uplink, no [`Fact`] at all. That is
//! deliberate: it is what makes a narrowing key inexpressible inside the blocker. So a rule cannot
//! take a candidate pair; it takes an [`L2Side`] per side, which is the interface WITH the
//! observations that landed on it.
//!
//! ⚠️ **Resolving `ObsId`s back to observations is the CALLER's job, not this module's.**
//! [`crate::identity::l1::join`] returns `BTreeMap<L1Key, BTreeSet<ObsId>>`; mapping those ids back
//! against the original slice belongs to whoever holds that slice. Story 6.12 is the first such
//! caller; this module answers with what it is handed.
//!
//! # 🔴 A `decide` at this level must receive L2 verdicts ONLY
//!
//! Combining L1's verdict for the same pair with an L2 rule's is the obvious gesture and it
//! **erases this level**. Measured on the committed corpus: two observations on different
//! interfaces carry different MACs, so `l1::verdict_for_pair` returns `Disqualifying` via
//! `l1-distinct-mac`, `decide` gives `Disqualifying` absolute priority, and the result is
//! `NoMatch { rule: "l1-distinct-mac" }` — in which no L2 rule can ever be named.
//!
//! 🔴 _(This said **"always returns `Disqualifying`"** until story 6.7's code review built the
//! counterexample: a MULTI-HOMED observation carrying two MACs belongs to BOTH interfaces, so the
//! same observation can stand on both sides — and `verdict_for_pair` then returns `Decisive` via
//! `l1-exact-mac`, which composed with an L2 `Opposes` gives `Abstained { Ambiguous }`, not
//! `NoMatch`. The warning holds and its **"always" did not**; an unreachable-today counterexample
//! in a sentence written to guide story 6.12 is exactly the kind that stops being unreachable when
//! 6.12 arrives.)_
//!
//! `l1.rs` says the two organs do not consult each other; **nothing had ever said it about
//! `decide`'s ARGUMENT**, and this paragraph is that sentence. The invariant is registered against
//! story 6.12, which should consider closing it in a TYPE rather than in prose.
//!
//! # ⚠️ How a wrong rule id is caught here, and how it is NOT
//!
//! [`crate::trap::Expectation`] names a rule and the gate compares it — but **not for a verdict set
//! that only OPPOSES**. D13's ratified arbitration (GitHub issue #54) makes `>= 1 Opposes` with no
//! `Disqualifying` abstain on `AbsenceOfProof`, and an abstention carries **no rule**, so the
//! gate's rule comparison never fires. Measured end to end: corrupting this module's rule id leaves
//! the trap PASSING.
//!
//! ⚠️ **The same hole on the other rule, through a DIFFERENT arm — measured, not inherited.** A
//! verdict set that only SUPPORTS lands on the weak-evidence row and abstains on
//! `IdentityAbstentionCause::Ambiguous`. Different cause, different arm of the table, and the same
//! consequence: no rule is named, so a misspelled id is invisible to the trap path there too. Story
//! 6.9 re-measured that rather than reading it off this paragraph, because *the same conclusion
//! reached by a different mechanism is a second measurement and not a corollary*.
//!
//! The id is therefore pinned by a DOUBLE-LITERAL test — the constant against the corpus's own
//! spelling — which is L1's idiom.
//!
//! ⚠️ **It is not the SOLE carrier, measured**: corrupting the constant reds **two** tests, the
//! double-literal one and the corpus walk, which filters on this id and finds nothing. _(Three
//! documents called it "the only carrier" until story 6.7's code review, while the story's own
//! mutation table recorded `M5 … red 2` three lines away. A claim of sole carriership is worth
//! exactly the mutation that checked it — and this one had been checked and then mis-stated.)_

use std::collections::BTreeSet;

use crate::identity::blocking::L2CandidatePair;
use crate::identity::cascade::{RuleVerdict, Verdict};
use crate::observation::{Fact, MacAddr, ObsId, Observation};
use crate::trap::RuleId;

/// The rule id, spelled exactly as the committed corpus spells it.
///
/// Three committed traps name it. ⚠️ Only **two** of them can be answered at this level: the third,
/// `cloned-mac-must-not-merge`, carries the same MAC on both observations, so the L1 join collapses
/// them onto ONE interface and no pair exists to judge. Guy's arbitration of 2026-08-30 accepts
/// that; the structural reading it would need is registered against story 6.11.
pub const L2_DIFFERENT_HOSTNAME: &str = "l2-different-hostname";

/// One side of an L2 candidate pair — the observations that landed on ONE interface.
///
/// # Why a type rather than a bare slice
///
/// The rules below take two of these, and *which* side is which carries no meaning: naming the
/// shape once gives the invariant a home and stops a caller from passing a slice of something else.
/// It holds no key: no rule here reads one, and a field nothing reads is an invention (D45).
#[derive(Debug, Clone)]
pub struct L2Side<'a> {
    /// The observations that landed on this interface. May be empty — see [`hostnames_of`].
    pub observations: Vec<&'a Observation>,
}

impl<'a> L2Side<'a> {
    /// Build a side from the observations that landed on one interface.
    pub fn new(observations: Vec<&'a Observation>) -> Self {
        Self { observations }
    }
}

/// The hostnames this side actually offers, normalised — an empty set meaning *no name observed*.
///
/// # Absence and emptiness are ONE case, and the corpus says so
///
/// `fixtures/scenario/traps/hostname-absence.toml`'s header states the equivalence this implements:
/// *"MISSING and EMPTY are both the absence of a signal: an empty string is not a matchable value
/// (`"" == ""` is not hostname agreement), a byte-present empty name counts as NO observed value,
/// and a name that stops resolving opposes nothing."* A null hostname is unrepresentable — `Fact`'s
/// `name` is a `String`, not an `Option<String>` — so the format cannot even pose the non-case.
///
/// # ⚠️ Case is folded, in ASCII, and the decision is the point
///
/// Compared case-SENSITIVELY, `NAS-01` and `nas-01` argue that one machine is two devices — D20's
/// named bug produced by doing nothing in particular. D10 puts this comparison in Rust rather than
/// in SQL (*"comparison never descends into SQL"*), so the rule owns it.
///
/// **ASCII lowercasing, not [`str::to_lowercase`]**: full Unicode case folding has traps of its own
/// (the Turkish dotless ı, final sigma) and a DNS label is ASCII.
///
/// ⚠️ **The risk INTRODUCED is stated too, not only the risk avoided** — the first draft gave one
/// side of that trade and the code review caught it. `to_ascii_lowercase` leaves every non-ASCII
/// byte alone, so `NAS-Ö1` and `nas-ö1` stay two names and can **oppose**: D20's bug, in the one
/// direction this choice does not cover. It is accepted because a hostname whose capitalisation is
/// non-ASCII is not a case this product has met, and because the alternative trades a rare wrong
/// split for the Turkish-ı wrong MERGE, which is worse — *a false merge cannot be undone by looking
/// harder.* **No committed trap exercises case in either direction**, so both halves ship with a
/// synthetic guard and this paragraph.
///
/// # 🔴 A name must carry at least one alphanumeric character, and that is a PROPERTY
///
/// `"  "` counts as absent — but trimming is not enough, and this repository has the measurement to
/// prove it: `"\u{200B}".trim().is_empty()` is **`false`** in Rust, so a zero-width space survives
/// as a "name". Story 6.7's code review measured the consequence — `"\u{200B}"` against
/// `"\u{2062}"` **OPPOSED**, and so did `"---"` against `"..."`. *Two non-signals arguing
/// confidently that one machine is two devices* — D20's named bug, reached through a channel a trim
/// does not close.
///
/// The guard is therefore **a property, never a list**: a name must contain at least one ASCII
/// alphanumeric character. That is principled rather than defensive — RFC 1035 requires a DNS label
/// to start with a letter — and it closes the invisible-character class and the punctuation class
/// together **for a name made ENTIRELY of noise**, where an enumeration of invisible code points could
/// only ever close the ones someone thought of. *An enumeration cannot claim the completeness of a
/// property* (story 5.12's sentence, applied again).
///
/// 🔴 **AND THE WHOLE-STRING QUALIFICATION IS LOAD-BEARING — measured, this claim was too broad.** A
/// zero-width space ATTACHED TO A REAL NAME survives: `trim` does not remove U+200B and the property
/// only asks for *at least one* alphanumeric, so `"\u{200B}nas-01"` and `"nas-01"` are two names.
/// [`verdict_for_hostname_agreement`] answers `Neutral` there, which costs a merge and errs safely —
/// but [`verdict_for_hostname`] answers **`Opposes`**, which is D20's named bug in the direction story
/// 6.7's review closed **for the whole-string case only**. ⚠️ Not live through the shipped connector
/// (`reverse_dns::sanitise` refuses U+200B) and live for any other producer and for this function as a
/// domain primitive. Found by story 6.9's edge-case review layer; registered with story 6.12, which is
/// the first caller that can reach it.
///
/// ⚠️ **A second limit, stated**: a name of only non-ASCII letters — a purely Cyrillic or CJK hostname —
/// carries no ASCII alphanumeric and is therefore read as absent. That is a REFUSAL TO SPEAK, never
/// a false `Opposes`, so it errs on D20's safe side; and no committed trap exercises it. (Re-measured
/// by the same layer: `"Нас"` against `"Нас"` is `Neutral` under BOTH rules — the stated limit
/// working.)
///
/// ⚠️ **A third, and the alphanumeric property cannot reach it at all**: a name that is syntactically
/// perfect and identifies nobody. Measured — `localhost`, `unknown`, `android` and `1` each agree with
/// themselves and yield **`Supports`**, and `reverse_dns::sanitise` carries no denylist of
/// non-identifying names. Latent today because a lone `Supports` cannot merge; **the decision that
/// changes that is the one Guy deferred to Epic 6's retrospective**, so it is registered rather than
/// left for that retrospective to discover.
///
/// # ⚠️ [`crate::observation::HostnameSource`] is deliberately IGNORED
///
/// The rule does not care whether a name came from DHCP, DNS, mDNS or NetBIOS. Weighting by source
/// is exactly the invention D20 refuses, and a silence about a field is not a decision about it —
/// hence this sentence.
pub fn hostnames_of(side: &L2Side<'_>) -> BTreeSet<String> {
    side.observations
        .iter()
        .flat_map(|observation| observation.facts.iter())
        .filter_map(|fact| match fact {
            Fact::Hostname { name, .. } => {
                let trimmed = name.trim();
                // A name must SAY something. `trim` alone leaves zero-width spaces and pure
                // punctuation standing, and two such "names" opposed each other — measured.
                if trimmed.chars().any(|c| c.is_ascii_alphanumeric()) {
                    Some(trimmed.to_ascii_lowercase())
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect()
}

/// `l2-different-hostname` — two interfaces whose names cannot both be right argue against being
/// one device.
///
/// # The verdict
///
/// [`Verdict::Opposes`] when **both** sides offer a name and the two name sets are **disjoint**;
/// [`Verdict::Neutral`] otherwise. This is **the first producer of `Opposes` in this codebase**.
///
/// # 🔴 Why `Neutral` and not `Opposes` on absence — D20's lock
///
/// *"The rule that wrongly `Opposes` should return `Neutral` — it does not KNOW, it BELIEVES it
/// knows; nine parasitic abstentions out of ten are that"* [architecture.md:1409]. A missing name
/// is not a disagreement, and *absence is derived, never observed, so absence can never oppose* —
/// the `hostname-absence` family's own words. ⚠️ **No trap in that family names this rule**: it
/// constrains this rule by not letting it fire, which is a negative requirement carried by tests
/// here rather than by a trap turning green.
///
/// # ⚠️ Disjointness, on a side that offers several names
///
/// An interface is a group of observations, so a side may offer more than one name — a renamed
/// host, DHCP churn. `Opposes` requires the sets to share **nothing**: a partial overlap stays
/// `Neutral`, because a rule that opposes on a partial overlap claims to know which name is current,
/// and it does not.
///
/// ⚠️ **The committed corpus cannot exercise this.** Measured over every replay stream, exactly one
/// interface carries two different names — and it is the one `cloned-mac` collapses onto, which has
/// no L2 pair at all. So the multi-name behaviour is **reachable in production and unexercised by
/// the corpus**, and it ships with a synthetic guard rather than a claim.
///
/// # Evidence
///
/// On `Opposes`, both sides' [`crate::observation::ObsId`]s, **sorted** — so the evidence of a pair
/// does not depend on which side was the left argument, on `verdict_for_pair`'s measured precedent.
///
/// ⚠️ **It is NOT de-duplicated**, and that is stated rather than discovered: a side holding one
/// observation twice, or a multi-homed observation standing on both sides, puts an `ObsId` in twice.
/// Unreachable through the corpus today and reachable by a caller building a malformed group —
/// story 6.12's plumbing is where that becomes possible.
/// A `Neutral` legitimately carries none: D19's *"a rule that fires without leaving its `rule_id`
/// in the database is a rule we cannot debug"* is about a verdict that ARGUES.
pub fn verdict_for_hostname(a: &L2Side<'_>, b: &L2Side<'_>) -> RuleVerdict {
    let names_a = hostnames_of(a);
    let names_b = hostnames_of(b);

    // 🔴 D20'S LOCK, and it is one line that four tests were seen red without.
    //
    // `BTreeSet::is_disjoint` says TRUE of two empty sets, so a rule that only asks "are the names
    // disjoint?" OPPOSES on absence — the exact bug D20 names, reached by doing nothing in
    // particular. Both sides must actually OFFER a name before disagreement can mean anything.
    let both_sides_offer_a_name = !names_a.is_empty() && !names_b.is_empty();
    let opposes = both_sides_offer_a_name && names_a.is_disjoint(&names_b);

    if opposes {
        RuleVerdict {
            rule: RuleId(L2_DIFFERENT_HOSTNAME.to_string()),
            verdict: Verdict::Opposes,
            evidence: evidence_of(a, b),
        }
    } else {
        RuleVerdict {
            rule: RuleId(L2_DIFFERENT_HOSTNAME.to_string()),
            verdict: Verdict::Neutral,
            evidence: Vec::new(),
        }
    }
}

/// The observations both sides stand on, **sorted** — what a verdict that ARGUES leaves behind.
///
/// D19: *"a rule that fires without leaving its `rule_id` in the database is a rule we cannot
/// debug"*. The sort is what makes the evidence of a pair independent of which side was the left
/// argument, on `verdict_for_pair`'s measured precedent.
///
/// # 🔑 Why this is a function and not two inline blocks
///
/// It was two, and the mutation driver is what said so: story 6.9's prove-to-red **refused** the
/// mutation that drops the sort with `ANCHOR MATCHED 2 TIMES`, the second site being
/// [`verdict_for_hostname`]'s own copy. *Two representations of one convention with nothing tying
/// them* — replacing both would have repaired the guard the mutation was meant to red, which is
/// exactly what establishes that they were independent. They are one now, so a mutation of the order
/// reds every rule that argues.
///
/// ⚠️ **It is NOT de-duplicated**, and that is stated rather than discovered: a side holding one
/// observation twice, or a multi-homed observation standing on both sides, puts an [`ObsId`] in
/// twice. Unreachable through the corpus today; **story 6.12's plumbing is where it becomes
/// possible**, and it is registered there rather than guarded here.
fn evidence_of(a: &L2Side<'_>, b: &L2Side<'_>) -> Vec<ObsId> {
    let mut evidence: Vec<_> = a
        .observations
        .iter()
        .chain(b.observations.iter())
        .map(|observation| observation.obs_id)
        .collect();
    evidence.sort();
    evidence
}

/// The rule id, spelled exactly as the committed corpus spells it.
///
/// **Exactly one committed trap names it** — `shared-hardware-vm-must-merge`, whose two observations
/// carry distinct locally-administered MACs and the shared hostname `doc-vm-alpha`, so the L1 join
/// gives two distinct interfaces and a pair exists to judge. ⚠️ Unlike
/// [`L2_DIFFERENT_HOSTNAME`]'s three, none is lost to a collapse.
///
/// # ⚠️ What reds when this constant is corrupted, and what does not
///
/// The string `"l2-hostname-agrees"` appears at **four hand-authored test sites** keyed on the corpus
/// TOML rather than on this constant (`cascade.rs`, `l1_runner.rs`, `trap_gate.rs`, and `fixtures.rs`'s
/// `expected()` second oracle) — **measured: not one of the four reds when this constant is
/// corrupted**, because none of them reads it.
///
/// What DOES red is two sites, and they red by two different mechanisms: the **double-literal pin** in
/// `fixtures.rs`, which compares this constant against the corpus's own spelling, and that file's
/// **corpus walk**, which filters on the CONSTANT and therefore iterates **zero** times — so its red
/// comes from its terminal naming assertion and from nothing else. Without that assertion the pin is
/// the sole carrier. *A claim of sole carriership is worth exactly the mutation that checked it* — this
/// one was checked, in both directions.
///
/// 🔴 _This section read "the constant is one literal among five, and only two of them are tied to it".
/// Three things were wrong and the blind review layer found all three from the diff: the count was a
/// BASELINE figure (this story adds a sixth occurrence, the pin's own literal); "only two of them"
/// named nothing inside its own enumeration, since the two that red are neither of the five listed;
/// and the walk is **not a literal** but a reader of the constant, which is why it iterates zero times
/// instead of failing a comparison — *wrong in kind, not merely in count*. `fixtures.rs` also appeared
/// on BOTH sides of one enumeration._
pub const L2_HOSTNAME_AGREES: &str = "l2-hostname-agrees";

/// `l2-hostname-agrees` — two interfaces reporting the same name argue for being one device.
///
/// # The verdict
///
/// [`Verdict::Supports`] when **both** sides offer a name and the two name sets are **equal**;
/// [`Verdict::Neutral`] otherwise. This is **the first producer of `Supports` in this codebase** —
/// story 6.8 was to have been, and Guy's reorder of 2026-09-23 moved it here because `Fact::Uplink`
/// has no connector producer while `Fact::Hostname` has had one since PR #143.
///
/// # 🔴 Why `Neutral` and not `Supports` on absence — D20's lock, and the CORPUS wrote it first
///
/// `BTreeSet` compares **equal** when both sides are empty, so a rule that only asks *"are the names
/// the same?"* **SUPPORTS ON ABSENCE** — two interfaces that name nothing arguing they are one
/// machine. `fixtures/scenario/traps/hostname-absence.toml:15` prescribes the lock in so many words,
/// two epics before this rule existed: *"an empty string is not a matchable value (`"" == ""` is not
/// hostname agreement)"*.
///
/// ⚠️ **And no trap can red it.** Measured over the trap-named pairs, the unlocked form flips **8 of
/// 25** to `Supports` on total absence — six being `must-not-merge` traps — and **not one is caught**, because a lone false `Supports` still yields `Abstained { Ambiguous }` and
/// `(must-not-merge, Abstained)` is `score`'s load-bearing PASS cell. *The lock is carried by
/// synthetic guards alone, and the tests below are all that stand between the product and it.*
///
/// 🔑 **The stakes are not symmetric with [`verdict_for_hostname`]'s.** A false `Opposes` leaves two
/// rows where one belongs and the operator can still act; *a false merge cannot be undone by looking
/// harder* (Guy, story 6.7).
///
/// # ⚠️ EQUALITY, not a shared name — Guy's decision of 2026-09-24
///
/// A side is a group of observations, so it may offer several names. `Supports` requires the sets to
/// be **equal**: a partial overlap stays `Neutral` here exactly as it stays `Neutral` in
/// [`verdict_for_hostname`], so the two rules are silent together in the middle. The argument is the
/// false-merge asymmetry above and **not** symmetry for its own sake — a non-empty *intersection*
/// would assert that one shared name settles which name is current, which is the sentence
/// [`verdict_for_hostname`]'s own doc refuses, read backwards.
///
/// ⚠️ **Two tempting arguments for this reading were measured and do NOT hold**, and are recorded so
/// nobody re-derives the decision from them: *Supports and Opposes are mutually exclusive* is true
/// under **every** reading anyone would write (0 violations over 64 adversarial subset pairs, under
/// equality, intersection, subset and the unlocked form alike), because supporting implies sharing a
/// name implies not being disjoint; and the emptiness lock above is a **cost** of this reading rather
/// than a reason for it, since `!is_disjoint` is already `false` for two empty sets.
///
/// ⚠️ **The committed corpus cannot separate the two readings**: every side in it offers at most one
/// name, so they agree on all 26 rows and diverge on 30 of the 64 synthetic pairs. The tests below
/// carry the decision, and a partial-overlap population is what makes it reversible at the right
/// price.
///
/// # ⚠️ What this rule's silence is, and what it is not
///
/// Where a name is absent, the silence is **data availability**: `prd.md:880` measures hostname
/// *"unusable on nearly half of known clients (absent on 71, empty string on 11, and never null)"*,
/// and because this rule needs **both** sides of a pair to carry one, availability enters **squared**
/// — on the reference LAN, ~70 % per interface is ~49 % per pair.
///
/// Where two sources spell one name differently, the silence is **normalisation**, which is a
/// different thing and is not closed here: an FQDN against a short label is `Neutral` here while
/// [`verdict_for_hostname`] confidently **`Opposes`**. Nothing is live today — the one connector that
/// emits a name strips the trailing dot at `reverse_dns.rs:137` — but *that is a property of one
/// connector and not of [`hostnames_of`]*, so the question is registered with story 6.12 rather than
/// answered by a sentence.
///
/// # 🔴 A `Supports` cannot make a merge, and the trap that names this rule therefore FAILS
///
/// [`crate::identity::cascade::decide`] reaches `Conclusion::Match` through one arm only, which
/// requires a `Decisive`; a lone `Supports` lands on the row `architecture.md:972` calls **weak
/// evidence** and abstains as `Ambiguous`, which `score` scores a **fail** against a `must-merge`
/// expectation. So `shared-hardware-vm-must-merge` — and story 6.8's two traps, which expect a merge
/// from a `Supports` rule just as this one does — cannot score a pass under the present algebra.
///
/// ⚠️ **Guy's decision of 2026-09-24 is that this story MEASURES that and does not decide it**: the
/// rule's verdict is `Supports` under every option on the table, so no code here depends on the
/// answer — only the trap's score does. *What makes a merge at L2* is Epic 6's retrospective's, which
/// may edit `epics.md` where a story may not.
pub fn verdict_for_hostname_agreement(a: &L2Side<'_>, b: &L2Side<'_>) -> RuleVerdict {
    let names_a = hostnames_of(a);
    let names_b = hostnames_of(b);

    // 🔴 D20'S LOCK, MIRRORED — and the mirror is the dangerous half.
    //
    // Two empty `BTreeSet`s are EQUAL, so without this line absence SUPPORTS: two interfaces that
    // name nothing would argue they are one machine. `hostname-absence.toml:15` asked for this line
    // before the rule existed, and no trap in the corpus can red its loss.
    let both_sides_offer_a_name = !names_a.is_empty() && !names_b.is_empty();
    let supports = both_sides_offer_a_name && names_a == names_b;

    if supports {
        RuleVerdict {
            rule: RuleId(L2_HOSTNAME_AGREES.to_string()),
            verdict: Verdict::Supports,
            evidence: evidence_of(a, b),
        }
    } else {
        RuleVerdict {
            rule: RuleId(L2_HOSTNAME_AGREES.to_string()),
            verdict: Verdict::Neutral,
            evidence: Vec::new(),
        }
    }
}

/// The IANA virtual-router MAC block, five octets exact — `00:00:5e:00:01:xx`.
///
/// D13 reads it as a STRUCTURAL FACT rather than an inference (`architecture.md:1054-1055`): *"VRRP/HSRP
/// MAC prefixes are **IANA-reserved** … **A reading, not an inference**"*, the low bits carrying the VRID.
///
/// # 🔴 FIVE octets, and the width is load-bearing against the product's own shipped dataset
///
/// `example_data.rs` carries thirty-odd production MACs of the form `00:00:5E:00:53:xx` — RFC 7042
/// *documentation* addresses, which share the IANA OUI `00:00:5e` and differ only at the **fifth** octet.
/// **A predicate written on three octets would disqualify the entire example inventory**, and one written
/// on four would admit the IANA VRRP **IPv6** block `00:00:5e:00:02:xx`, which no committed fixture
/// exercises. ⚠️ Story 6.11's validation measured the four-octet widening leaving the whole suite, clippy,
/// ten gates **and its own equality pin** green — because that pin enumerated the *last* octet and never
/// crossed the boundary. *An enumeration cannot claim the completeness of a property.*
///
/// # ⚠️ The set is CLOSED, and HSRP is out BY DECISION rather than by omission
///
/// `vrrp-virtual-mac.toml`'s privacy record: *"The allowlist is CLOSED: HSRP (00:00:0c:07:ac — a Cisco
/// OUI, not IANA) **and HSRPv2** stay out until a fixture commits those octets, each with its own
/// prove-to-red."* Both are refused here, and a negative probe measures it — validation added HSRP to a
/// draft of this predicate and the whole suite stayed GREEN.
const IANA_VIRTUAL_ROUTER_PREFIX: [u8; 5] = [0x00, 0x00, 0x5e, 0x00, 0x01];

/// Whether this hardware address is an IANA virtual-router address.
///
/// A property of the BYTES and of nothing else. ⚠️ **It deliberately does not consult
/// `Fact::Mac::locally_administered`**: story 6.11's validation made a draft read that flag instead and
/// the whole suite stayed GREEN, because the fixture's flag happens to agree — *so a connector reporting
/// it wrongly would silently disable the disqualification and nothing would notice*.
/// [`crate::observation::MacAddr::is_locally_administered`]'s own doc calls the bytes *"the ground truth a
/// connector's reported flag can be cross-checked against"*, and this function is on the ground-truth
/// side of that sentence.
fn is_iana_virtual_router_mac(addr: MacAddr) -> bool {
    addr.0[..5] == IANA_VIRTUAL_ROUTER_PREFIX
}

/// The rule id, spelled exactly as the committed corpus spells it.
///
/// **One committed trap names it**: `vrrp-virtual-mac-must-not-merge-master`, the VIP against the master
/// of the minute.
///
/// # 🔴 Why a RULE identifier names something that is NOT a rule
///
/// Guy's arbitration of 2026-08-12 is *there is no rule*, and `epics.md`'s criterion says so in capitals:
/// the disqualification is **a structural fact read at ingestion, not a rule that scores**. But
/// [`crate::identity::cascade::Conclusion::NoMatch`] carries a [`RuleId`], and D19 wants the id left
/// behind — *a rule that fires without one is undebuggable*. Measured, there is exactly one door:
/// `NoMatch { rule }` is reachable only from a `RuleVerdict` carrying [`Verdict::Disqualifying`].
///
/// 🔑 **So the thing that is not a rule must be a `RuleVerdict`, and what makes it not a rule is not its
/// arity but WHAT IT READS** — see [`verdict_for_virtual_mac`]. The sentence this section owes was already
/// written one crate away, at `neighbour.rs:167-172`, and it is quoted rather than re-derived: *"D13 calls
/// it disqualifying **as a grouping anchor** — which is an L2 judgement about how much a MAC is worth, not
/// an L1 judgement about whether it is an address at all."*
pub const L2_VIRTUAL_MAC_PREFIX: &str = "l2-virtual-mac-prefix";

/// `l2-virtual-mac-prefix` — an IANA virtual-router address can never anchor a grouping.
///
/// # The verdict
///
/// [`Verdict::Disqualifying`] when **either** key's hardware address sits in the IANA virtual-router
/// block; [`Verdict::Neutral`] otherwise. **The second producer of `Disqualifying` in this codebase and
/// the first at L2** — the only other is `l1-distinct-mac`.
///
/// # 🔑 Why this takes a PAIR OF KEYS, which is what makes it a READING and not a rule
///
/// Guy's arbitration of 2026-09-24. The distinction is **not arity — it is what the function READS**:
///
/// | | takes | reads |
/// |---|---|---|
/// | a RULE | two [`L2Side`]s | the FACTS the sides carry |
/// | a READING | one [`L2CandidatePair`] | the KEY, and nothing else |
///
/// **A function that cannot reach a [`Fact`] cannot score, and the compiler checks that** rather than a
/// doc comment promising it. `L1Key` is `(L2DomainId, MacAddr)`, so the key is all this needs.
///
/// 🔴 **The obvious alternative was BUILT and MEASURED WRONG.** A reading over an [`L2Side`] must dig MACs
/// out of the side's `facts`, and an observation bearing several MACs stands on several interfaces — so it
/// answers `Disqualifying` for a perfectly ordinary interface because *some observation standing on it
/// also bore a virtual MAC*, **which is the ordinary VRRP geometry**: a router port bears its own address
/// and the VIP. ⚠️ And the committed corpus cannot tell the two shapes apart — 0 disagreements over all 31
/// interfaces in every trap-named stream — so this decision is carried by synthetic guards alone.
///
/// # ⚠️ *Either* key, and what that does NOT mean
///
/// Story 6.11's contexting argued that the reading must be two-sided *"because the distinction it exists to
/// draw is whether the two interfaces share the virtual MAC — one gateway re-seen versus the VIP against
/// its master"*. 🔴 **That reason is false and implementing it is what showed so**: the failover
/// re-sighting is `l1-exact-mac`'s business and its two observations land on the **same** interface, so
/// they form no pair at all and this function never sees them. *What makes the pair the right argument is
/// that being a grouping ANCHOR is a property of the pair, not that the predicate compares the two sides
/// — it does not.*
///
/// **Both keys virtual is `Disqualifying` too**, and that is deliberate: two VIPs are two virtual
/// gateways, so the pair is no more an anchor than a VIP against its master.
///
/// # ⚠️ The evidence is EMPTY, and that is the truthful value rather than a D19 lapse
///
/// [`RuleVerdict::evidence`] is documented as *"the observations the rule READ to say it"*. This reading
/// read a key and no observation, so an empty vector is what is true; attaching ids it never looked at
/// would be the invention D19 exists to prevent. ⚠️ **The cost is stated rather than hidden**: an operator
/// meeting this refusal sees the rule and not the pair, and the pair is the caller's to record. Story 6.12
/// is the first caller with both in hand.
///
/// # 🔴 The verdict this produces can never be NAMED beside an L1 `Disqualifying`, for ANY trap, ever
///
/// [`crate::identity::cascade::decide`] names the **lexicographically smallest** `Disqualifying`, and
/// every `l1-*` id sorts before every `l2-*` id. So a vector mixing L1's verdict for the same pair in
/// answers `NoMatch { rule: "l1-distinct-mac" }` and this rule is invisible. ⚠️ `l2.rs`'s module doc states
/// that as a per-case argument measured on one family; **it is a total property of the naming convention
/// meeting `min()`**, measured by story 6.11's validation and registered.
///
/// So this verdict reaches a `NoMatch` naming itself **only in an L2-only vector**, which is what story
/// 6.12's plumbing owes. ✅ *Measured sufficient at unit level today: `Neutral` · `Neutral` · this
/// `Disqualifying` gives `NoMatch { l2-virtual-mac-prefix }`, hence `Refused`, which `score` passes.*
pub fn verdict_for_virtual_mac(pair: &L2CandidatePair) -> RuleVerdict {
    let disqualifying =
        is_iana_virtual_router_mac(pair.low().1) || is_iana_virtual_router_mac(pair.high().1);

    RuleVerdict {
        rule: RuleId(L2_VIRTUAL_MAC_PREFIX.to_string()),
        verdict: if disqualifying {
            Verdict::Disqualifying
        } else {
            Verdict::Neutral
        },
        // Empty under BOTH verdicts, and for one reason: this reading read no observation. See the doc.
        evidence: Vec::new(),
    }
}

/// Tests for the L2 rules, over SYNTHETIC inputs only.
///
/// Nothing here reads `fixtures/` — this crate may not touch the filesystem (D47). The
/// corpus-driven half lives in `opencmdb-bin`'s `fixtures.rs`, beside the corpus walks.
///
/// # A deliberate duplication, which a DRY pass may not collapse
///
/// The helpers below re-declare the spellings `l1.rs` and `blocking.rs` use in their own test
/// modules. Those are private to their files and unreachable from here; the alternative is a
/// `pub(crate)` test-helper surface, which is a wider change than this story wants.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::{
        ConnectorId, HostnameSource, L2DomainId, MacAddr, ObsId, Scope, Timestamp, VantageId,
    };
    use uuid::Uuid;

    fn ts() -> Timestamp {
        chrono::DateTime::parse_from_rfc3339("2026-08-30T10:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    /// One observation carrying a MAC and, when `hostname` is `Some`, that name.
    fn observation(n: u128, last: u8, hostname: Option<&str>) -> Observation {
        let mut facts = vec![Fact::Mac {
            addr: MacAddr([0x00, 0x11, 0x22, 0x33, 0x44, last]),
            locally_administered: false,
        }];
        if let Some(name) = hostname {
            facts.push(Fact::Hostname {
                name: name.to_string(),
                source: HostnameSource::Dhcp,
            });
        }
        Observation {
            obs_id: ObsId::from_uuid(Uuid::from_u128(n)),
            connector_id: ConnectorId::from_uuid(Uuid::from_u128(0xC0)),
            observed_at: ts(),
            scope: Scope {
                l2_domain: L2DomainId::from_uuid(Uuid::from_u128(1)),
                vantage: VantageId::from_uuid(Uuid::from_u128(2)),
            },
            facts,
            raw: None,
        }
    }

    fn side(observations: &[Observation]) -> L2Side<'_> {
        L2Side::new(observations.iter().collect())
    }

    // ---- `l2-virtual-mac-prefix` (story 6.11): the reading, not a rule ----

    /// The IANA virtual-router address the committed corpus uses, VRID 10.
    fn virtual_mac() -> MacAddr {
        MacAddr([0x00, 0x00, 0x5e, 0x00, 0x01, 0x0a])
    }

    /// An ordinary documentation address — RFC 7042, the shape `example_data.rs` ships.
    fn ordinary_mac(last: u8) -> MacAddr {
        MacAddr([0x02, 0x00, 0x5e, 0x00, 0x53, last])
    }

    fn key(addr: MacAddr) -> crate::identity::l1::L1Key {
        (L2DomainId::from_uuid(Uuid::from_u128(1)), addr)
    }

    fn pair(a: MacAddr, b: MacAddr) -> L2CandidatePair {
        L2CandidatePair::new(key(a), key(b)).expect("two distinct addresses are two interfaces")
    }

    /// AC1's core: the VIP against its master is refused, and the verdict NAMES this reading.
    #[test]
    fn a_virtual_router_address_disqualifies_the_pair_and_names_itself() {
        let verdict = verdict_for_virtual_mac(&pair(virtual_mac(), ordinary_mac(0x8c)));

        assert_eq!(
            verdict.verdict,
            Verdict::Disqualifying,
            "an IANA virtual-router address can never anchor a grouping"
        );
        assert_eq!(
            verdict.rule.0, L2_VIRTUAL_MAC_PREFIX,
            "a verdict that refuses must say which reading refused, or the refusal is undebuggable (D19)"
        );
    }

    /// 🔴 The oracle AC1 must name, and the WEAK one it must not.
    ///
    /// Validation measured that `Disqualifying` → `Opposes` leaves *"the trap scores a pass"* GREEN,
    /// because a lone `Opposes` abstains on `AbsenceOfProof` and `(must-not-merge, Abstained)` is also a
    /// PASS cell. ⚠️ **So "the trap passes" cannot tell this reading firing from it never firing at all** —
    /// only the conclusion can, and only over an L2-ONLY vector.
    #[test]
    fn an_l2_only_vector_concludes_no_match_naming_this_reading() {
        use crate::identity::cascade::{Conclusion, decide};
        use crate::identity::l1::CURRENT_RULESET_VERSION;

        let vip = [observation(1, 0x01, Some("doc-rtr-alpha"))];
        let master = [observation(2, 0x02, Some("doc-rtr-alpha"))];
        let vector = vec![
            verdict_for_hostname(&side(&vip), &side(&master)),
            verdict_for_hostname_agreement(&side(&vip), &side(&master)),
            verdict_for_virtual_mac(&pair(virtual_mac(), ordinary_mac(0x8c))),
        ];

        let decision = decide(vector, CURRENT_RULESET_VERSION);

        assert_eq!(
            decision.conclusion,
            Conclusion::NoMatch {
                rule: RuleId(L2_VIRTUAL_MAC_PREFIX.to_string())
            },
            "the STRONG oracle: the conclusion is a refusal and it names this reading — 'the trap \
             passes' is satisfied by a reading that never fired"
        );
    }

    /// AC5: the width is FIVE octets, and the probe varies the FOURTH and FIFTH.
    ///
    /// 🔴 Validation widened a draft from five octets to four — admitting the IANA VRRP **IPv6** block
    /// `00:00:5e:00:02:xx` — and left the whole suite, clippy, ten gates **and an equality pin** GREEN,
    /// because that pin enumerated the *last* octet and never crossed the boundary. This one crosses it.
    ///
    /// ⚠️ And the fifth octet is what separates the block from `example_data.rs`'s thirty-odd
    /// `00:00:5E:00:53:xx` documentation MACs: **a three-octet predicate disqualifies the whole example
    /// inventory.**
    #[test]
    fn the_prefix_is_five_octets_and_the_neighbouring_blocks_are_not_it() {
        assert!(
            is_iana_virtual_router_mac(MacAddr([0x00, 0x00, 0x5e, 0x00, 0x01, 0x0a])),
            "VRID 10 of the IPv4 virtual-router block is the case the corpus commits"
        );
        for fifth in [0x00u8, 0x02, 0x53, 0xff] {
            assert!(
                !is_iana_virtual_router_mac(MacAddr([0x00, 0x00, 0x5e, 0x00, fifth, 0x0a])),
                "only the fifth octet 0x01 is the virtual-router block; 0x02 is IANA's IPv6 block and \
                 0x53 is RFC 7042 documentation, which the product's own example inventory is full of"
            );
        }
        assert!(
            !is_iana_virtual_router_mac(MacAddr([0x00, 0x00, 0x5f, 0x00, 0x01, 0x0a])),
            "the fourth-from-last octet is part of the prefix too"
        );
    }

    /// AC7: the allowlist is CLOSED, and HSRP and HSRPv2 are out BY DECISION.
    ///
    /// 🔴 Validation added HSRP to a draft of the predicate — the widening `vrrp-virtual-mac.toml` forbids
    /// in writing — and the whole suite, clippy and ten gates stayed GREEN. This is the one line that
    /// measures the limit instead of stating it.
    #[test]
    fn hsrp_and_hsrpv2_are_out_by_decision_and_a_probe_says_so() {
        assert!(
            !is_iana_virtual_router_mac(MacAddr([0x00, 0x00, 0x0c, 0x07, 0xac, 0x0a])),
            "HSRP's 00:00:0c:07:ac is a CISCO OUI, not IANA — out until a fixture commits it, with its \
             own prove-to-red"
        );
        assert!(
            !is_iana_virtual_router_mac(MacAddr([0x00, 0x00, 0x0c, 0x9f, 0xf0, 0x0a])),
            "HSRPv2 likewise — and the corpus names BOTH, where this story's contexting quoted only one"
        );
    }

    /// AC9: the BYTES decide, never the connector's reported flag.
    ///
    /// 🔴 Validation made a draft consult `Fact::Mac::locally_administered` and the suite stayed GREEN,
    /// because the fixture's flag happens to agree. A connector reporting it wrongly would silently
    /// disable the disqualification. ⚠️ Here the flag and the bytes CONTRADICT each other — `0x00` has the
    /// U/L bit clear — and the bytes must still win.
    #[test]
    fn the_bytes_decide_even_when_a_connector_would_report_otherwise() {
        let addr = virtual_mac();
        assert!(
            !addr.is_locally_administered(),
            "the premise: an IANA virtual-router address has the U/L bit CLEAR, so a connector \
             reporting 'locally administered' about it would be reporting the opposite of the bytes"
        );
        assert_eq!(
            verdict_for_virtual_mac(&pair(addr, ordinary_mac(0x8c))).verdict,
            Verdict::Disqualifying,
            "the reading is a property of the bytes; no reported flag can turn it off"
        );
    }

    /// AC8's other half: two virtual addresses are two virtual gateways, not one.
    #[test]
    fn two_virtual_router_addresses_are_still_not_a_grouping_anchor() {
        let vrid_10 = MacAddr([0x00, 0x00, 0x5e, 0x00, 0x01, 0x0a]);
        let vrid_20 = MacAddr([0x00, 0x00, 0x5e, 0x00, 0x01, 0x14]);

        assert_eq!(
            verdict_for_virtual_mac(&pair(vrid_10, vrid_20)).verdict,
            Verdict::Disqualifying,
            "two VIPs are two virtual gateways; the pair is no more an anchor than a VIP against its \
             master"
        );
    }

    /// Two ordinary addresses: the reading stays QUIET, which is most of its job.
    #[test]
    fn two_ordinary_addresses_are_left_alone() {
        assert_eq!(
            verdict_for_virtual_mac(&pair(ordinary_mac(0x8c), ordinary_mac(0x8d))).verdict,
            Verdict::Neutral,
            "a reading that spoke about every pair would be a rule, and a bad one"
        );
    }

    /// AC6: the evidence is EMPTY under both verdicts, and that is the truthful value.
    ///
    /// [`RuleVerdict::evidence`] is *"the observations the rule READ to say it"*. This reading read a KEY.
    /// Attaching ids it never looked at would be the invention D19 exists to prevent — and the cost, that
    /// an operator sees the rule without the pair, is the caller's to close (story 6.12).
    #[test]
    fn the_reading_cites_no_observation_because_it_read_none() {
        for (a, b) in [
            (virtual_mac(), ordinary_mac(0x8c)),
            (ordinary_mac(0x8c), ordinary_mac(0x8d)),
        ] {
            let verdict = verdict_for_virtual_mac(&pair(a, b));
            assert!(
                verdict.evidence.is_empty(),
                "a reading of the key read no observation, so it cites none — {:?}",
                verdict.verdict
            );
            assert_eq!(
                verdict.rule.0, L2_VIRTUAL_MAC_PREFIX,
                "and it names itself either way, so a reader can see WHICH reading was consulted"
            );
        }
    }

    // ---- D20's lock: the rule must STAY QUIET where it does not know ----

    /// 🔴 **The story's centre.** A missing name is not a disagreement.
    ///
    /// D20: *"the rule that wrongly `Opposes` should return `Neutral` — it does not KNOW, it
    /// BELIEVES it knows; nine parasitic abstentions out of ten are that."* And the
    /// `hostname-absence` family's own words: *absence is derived, never observed, so absence can
    /// never oppose.*
    #[test]
    fn a_side_with_no_hostname_is_neutral_never_opposes() {
        let left = [observation(1, 0x01, Some("doc-nas-01"))];
        let right = [observation(2, 0x02, None)];

        let verdict = verdict_for_hostname(&side(&left), &side(&right));

        assert_eq!(
            verdict.verdict,
            Verdict::Neutral,
            "a side that offers no name cannot disagree with one; opposing here is D20's named bug"
        );
        assert!(
            verdict.evidence.is_empty(),
            "a Neutral argues nothing and owes no evidence"
        );
    }

    /// An EMPTY name is the same case as a missing one — the corpus pins that equivalence.
    #[test]
    fn an_empty_hostname_is_an_absence_not_a_value() {
        let left = [observation(1, 0x01, Some("doc-nas-01"))];
        let right = [observation(2, 0x02, Some(""))];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "an empty name is the absence of a signal wearing a value's clothes"
        );
    }

    /// Both sides empty is still absence — `"" == ""` is not hostname agreement, and two absences
    /// are not a disagreement either.
    #[test]
    fn two_empty_hostnames_oppose_nothing() {
        let left = [observation(1, 0x01, Some("   "))];
        let right = [observation(2, 0x02, None)];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "neither side observed a name, so nothing was compared"
        );
    }

    // ---- What the rule DOES say ----

    #[test]
    fn two_present_and_different_hostnames_oppose() {
        let left = [observation(1, 0x01, Some("doc-vm-alpha"))];
        let right = [observation(2, 0x02, Some("doc-vm-beta"))];

        let verdict = verdict_for_hostname(&side(&left), &side(&right));

        assert_eq!(
            verdict.verdict,
            Verdict::Opposes,
            "two names that cannot both be right argue against one device"
        );
        assert_eq!(
            verdict.rule.0, L2_DIFFERENT_HOSTNAME,
            "the verdict names the rule that produced it"
        );
        assert_eq!(
            verdict.evidence.len(),
            2,
            "a verdict that ARGUES leaves both sides' observations behind (D19)"
        );
    }

    #[test]
    fn the_same_hostname_opposes_nothing() {
        let left = [observation(1, 0x01, Some("doc-nas-01"))];
        let right = [observation(2, 0x02, Some("doc-nas-01"))];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "agreement is not this rule's business; it only ever opposes or stays quiet"
        );
    }

    /// The evidence is SORTED, so it does not depend on which side was the left argument.
    #[test]
    fn the_evidence_does_not_depend_on_the_argument_order() {
        let left = [observation(9, 0x09, Some("doc-a"))];
        let right = [observation(1, 0x01, Some("doc-b"))];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).evidence,
            verdict_for_hostname(&side(&right), &side(&left)).evidence,
            "sorted evidence, or one logical pair produces two unequal verdicts"
        );
    }

    // ---- The two decisions the committed corpus CANNOT exercise ----

    /// ⚠️ Case is folded. **No committed trap exercises this in either direction** — see
    /// [`hostnames_of`]'s doc for why the decision is ASCII folding and what its limit is.
    #[test]
    fn hostnames_differing_only_in_case_are_the_same_name() {
        let left = [observation(1, 0x01, Some("NAS-01"))];
        let right = [observation(2, 0x02, Some("nas-01"))];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "two sources reporting one machine with different capitalisation must not be argued \
             into two devices — D20's bug reached by doing nothing in particular"
        );
    }

    /// ⚠️ A side may offer SEVERAL names. A partial overlap stays quiet: opposing there would claim
    /// to know which name is current. **Unexercised by the corpus** — the only multi-named interface
    /// it holds is the one `cloned-mac` collapses onto, which has no L2 pair.
    #[test]
    fn a_partial_overlap_of_name_sets_stays_neutral() {
        let left = [
            observation(1, 0x01, Some("doc-old-name")),
            observation(2, 0x01, Some("doc-shared")),
        ];
        let right = [observation(3, 0x02, Some("doc-shared"))];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "the sets share a name, so the rule does not know which is current and says so"
        );
    }

    /// Disjoint MULTI-name sets still oppose — the overlap rule is not an excuse to go quiet.
    #[test]
    fn disjoint_multi_name_sets_still_oppose() {
        let left = [
            observation(1, 0x01, Some("doc-a")),
            observation(2, 0x01, Some("doc-b")),
        ];
        let right = [observation(3, 0x02, Some("doc-c"))];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Opposes,
            "nothing is shared, so nothing suggests one device"
        );
    }

    /// 🔴 **Why AC3 cannot be carried by the trap gate, made executable rather than quoted.**
    ///
    /// This is the reason the rule id is pinned by a double-literal test in `opencmdb-bin` instead
    /// of by `run_trap`: an `Opposes`-only verdict set abstains, and **an abstention names no
    /// rule**, so nothing downstream can compare the id the trap expects against the id the rule
    /// carried. Misspell [`L2_DIFFERENT_HOSTNAME`] and the gate stays silent.
    ///
    /// D13's ratified arbitration, GitHub issue #54 — not a defect, a decision. It has been in the
    /// algebra since story 5.4b and was invisible for want of a producer; this module is the first
    /// producer, so this is where it becomes observable.
    #[test]
    fn an_opposes_only_verdict_abstains_and_names_no_rule() {
        use crate::identity::cascade::{Conclusion, IdentityAbstentionCause, decide};
        use crate::identity::l1::CURRENT_RULESET_VERSION;

        let left = [observation(1, 0x01, Some("doc-vm-alpha"))];
        let right = [observation(2, 0x02, Some("doc-vm-beta"))];
        let verdict = verdict_for_hostname(&side(&left), &side(&right));
        assert_eq!(
            verdict.verdict,
            Verdict::Opposes,
            "the premise of this test"
        );

        let decision = decide(vec![verdict], CURRENT_RULESET_VERSION);

        assert!(
            matches!(
                decision.conclusion,
                Conclusion::Abstained {
                    cause: IdentityAbstentionCause::AbsenceOfProof
                }
            ),
            "`>= 1 Opposes` with no `Disqualifying` abstains on AbsenceOfProof (D13, issue #54), \
             got {:?}",
            decision.conclusion
        );
        assert_eq!(
            decision.rule(),
            None,
            "and an abstention names NO rule — which is exactly why a misspelled id is invisible \
             to the trap gate, and why AC3 ships on a double literal instead"
        );
    }

    /// 🔴 **Two non-signals must not argue.** Found by story 6.7's code review, which measured
    /// `"\u{200B}"` opposing `"\u{2062}"` and `"---"` opposing `"..."` — D20's bug reached through
    /// a channel `trim` does not close, because `"\u{200B}".trim().is_empty()` is `false`.
    ///
    /// The guard is a PROPERTY — a name must carry at least one ASCII alphanumeric — never a list of
    /// invisible code points, which could only close the ones someone thought of.
    #[test]
    fn a_name_carrying_no_alphanumeric_is_not_a_name() {
        let invisible = [observation(1, 0x01, Some("\u{200B}"))];
        let other_invisible = [observation(2, 0x02, Some("\u{2062}"))];
        assert_eq!(
            verdict_for_hostname(&side(&invisible), &side(&other_invisible)).verdict,
            Verdict::Neutral,
            "two invisible characters are two non-signals, and non-signals do not disagree"
        );

        let dashes = [observation(3, 0x03, Some("---"))];
        let dots = [observation(4, 0x04, Some("..."))];
        assert_eq!(
            verdict_for_hostname(&side(&dashes), &side(&dots)).verdict,
            Verdict::Neutral,
            "pure punctuation is a placeholder, not a name — a DNS label must start with a letter"
        );
    }

    /// The CONTROL that gives the guard above its meaning: the property must not swallow real names.
    #[test]
    fn a_name_with_one_alphanumeric_is_still_a_name() {
        let left = [observation(1, 0x01, Some("-a-"))];
        let right = [observation(2, 0x02, Some("-b-"))];

        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Opposes,
            "one alphanumeric is enough to be a name; the guard refuses noise, not punctuation"
        );
    }

    #[test]
    fn a_side_with_no_observations_at_all_is_neutral() {
        let right = [observation(1, 0x01, Some("doc-nas-01"))];

        assert_eq!(
            verdict_for_hostname(&L2Side::new(Vec::new()), &side(&right)).verdict,
            Verdict::Neutral,
            "an empty side observed nothing; it cannot disagree"
        );
    }

    // ---- `l2-hostname-agrees` (story 6.9): D20's lock on the DANGEROUS side ----

    /// 🔴 **The story's centre, and the corpus asked for it two epics early.**
    ///
    /// Two empty `BTreeSet`s are EQUAL, so without the emptiness check this rule SUPPORTS on
    /// absence — two interfaces that name nothing arguing they are one machine.
    /// `hostname-absence.toml:15` prescribes it in so many words: *"an empty string is not a
    /// matchable value (`"" == ""` is not hostname agreement)"*.
    ///
    /// ⚠️ **No trap in the corpus can red this.** Measured over all 26 trap-named pairs, the
    /// unlocked form flips 8 of them to `Supports` on total absence and not one is caught, because a
    /// lone false `Supports` abstains as `Ambiguous` and `(must-not-merge, Abstained)` is `score`'s
    /// PASS cell. This test and its two neighbours are the only carriers.
    #[test]
    fn two_absent_hostnames_do_not_agree() {
        let left = [observation(1, 0x01, None)];
        let right = [observation(2, 0x02, None)];

        assert_eq!(
            verdict_for_hostname_agreement(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "two silences are not an agreement, and a false merge cannot be undone by looking harder"
        );
    }

    /// The same lock reached through a channel `trim` does not close.
    ///
    /// `"\u{200B}".trim().is_empty()` is `false` in Rust, so a zero-width space survives trimming.
    /// [`hostnames_of`]'s ASCII-alphanumeric property is what makes both sides read as ABSENT here;
    /// without it two invisible "names" would agree.
    ///
    /// 🔴 **BOTH SIDES CARRY THE SAME CODE POINT, and that one character is the whole test.** It first
    /// carried U+200B against U+2062 — story 6.7's population, mirrored mechanically — and **for
    /// EQUALITY that is the wrong shape**: two DIFFERENT invisible strings give unequal sets, so with
    /// the alphanumeric property deleted this test stayed **GREEN** while its punctuation neighbour
    /// reddened. Measured, with the neighbours as controls: invisible **GREEN**, punctuation **RED**,
    /// absent **GREEN**. 🔑 *Disjointness wants two different non-names; equality wants the same
    /// non-name twice — the mirror of a rule needs the mirror of its population, and mirroring the
    /// code while copying the data is how a guard ends up unable to fail.* Found by the review layer
    /// that had the diff and nothing else.
    #[test]
    fn two_invisible_hostnames_do_not_agree() {
        let left = [observation(1, 0x01, Some("\u{200B}"))];
        let right = [observation(2, 0x02, Some("\u{200B}"))];

        assert_eq!(
            verdict_for_hostname_agreement(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "an invisible character is not a name, so two of them are not an agreement"
        );
    }

    /// ⚠️ **Pure punctuation, which the story's first draft did not name.**
    ///
    /// The guard is a PROPERTY (*a name carries at least one ASCII alphanumeric*) and not a list of
    /// invisible code points, so it closes this class in the same act — *an enumeration cannot claim
    /// the completeness of a property*. Two IDENTICAL punctuation strings are the sharp case: set
    /// equality would hold on them.
    #[test]
    fn two_identical_punctuation_strings_do_not_agree() {
        let left = [observation(1, 0x01, Some("---"))];
        let right = [observation(2, 0x02, Some("---"))];

        assert_eq!(
            verdict_for_hostname_agreement(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "two identical non-names are still not names; equality of noise is not agreement"
        );
    }

    /// ⚠️ **Kept, and deliberately NOT claimed as a carrier of the lock above.**
    ///
    /// Under set equality this case is already `Neutral` by inequality — `{doc-nas-01}` is not `{}` —
    /// so dropping the emptiness check leaves this test GREEN. Measured: that mutation reds its **three**
    /// neighbours and not this one. It is here because it catches other mutations, and the story names
    /// **which three** carry D20's lock rather than counting four.
    ///
    /// ⚠️ _This said "its two neighbours" and "which two", figures inherited from the validation layer's
    /// tree, where the pure-punctuation guard did not yet exist. The mutation table beside it recorded
    /// `red 3` all along — **one count published three incompatible ways in one story**, and the blind
    /// review layer reconciled them by reproducing the mutation on paper._
    #[test]
    fn one_named_side_and_one_silent_side_do_not_agree() {
        let left = [observation(1, 0x01, Some("doc-nas-01"))];
        let right = [observation(2, 0x02, None)];

        assert_eq!(
            verdict_for_hostname_agreement(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "one side naming nothing cannot agree with one that does"
        );
    }

    // ---- what the rule DOES say ----

    /// AC1's positive, and AC4's real carrier: the VERDICT names this rule.
    ///
    /// ⚠️ The rule-id assertion is here on purpose. Without it, a `Supports` arm emitting
    /// [`L2_DIFFERENT_HOSTNAME`] ships with the whole suite green — measured, and the double-literal
    /// pin in `fixtures.rs` cannot see it, because that pin compares a constant with the corpus's
    /// spelling and never looks at a verdict.
    #[test]
    fn two_present_and_equal_hostnames_agree() {
        let left = [observation(1, 0x01, Some("doc-vm-alpha"))];
        let right = [observation(2, 0x02, Some("doc-vm-alpha"))];

        let verdict = verdict_for_hostname_agreement(&side(&left), &side(&right));

        assert_eq!(
            verdict.verdict,
            Verdict::Supports,
            "two interfaces reporting one name argue for being one device"
        );
        assert_eq!(
            verdict.rule.0, L2_HOSTNAME_AGREES,
            "a verdict that argues must name the rule that argued, not its sibling"
        );
    }

    /// Case folding and surrounding whitespace, on the agreement side.
    ///
    /// [`hostnames_of`] folds ASCII case and trims, so these are ONE name. The mirror of
    /// `hostnames_differing_only_in_case_are_the_same_name`, which measures that
    /// [`verdict_for_hostname`] does not oppose them; here the same normalisation must let them
    /// AGREE, and a rule blind to one direction would be the D20 bug on the merge side.
    #[test]
    fn case_and_whitespace_do_not_stop_two_names_agreeing() {
        let left = [observation(1, 0x01, Some("  NAS-01\t"))];
        let right = [observation(2, 0x02, Some("nas-01"))];

        assert_eq!(
            verdict_for_hostname_agreement(&side(&left), &side(&right)).verdict,
            Verdict::Supports,
            "one name spelled two ways is one name, on this rule as on its sibling"
        );
    }

    // ---- AC7: the population that carries Guy's decision of 2026-09-24 ----

    /// 🔴 **The decision's ONLY carrier, and the committed corpus cannot supply it.**
    ///
    /// Guy chose set EQUALITY over a non-empty intersection. A partial overlap is where the two
    /// readings diverge: equality says `Neutral`, intersection says `Supports`. Measured — with
    /// `!names_a.is_disjoint(&names_b)` in place of the equality, the whole suite stayed GREEN before
    /// this test existed, and so did a third plausible spelling (`is_subset`).
    ///
    /// ⚠️ Every side in the committed corpus offers at most one name, so the two readings agree on all
    /// 26 trap-named pairs and diverge on 30 of 64 synthetic ones. *A decision a story takes out loud
    /// and no guard can red is a decision the next story reverses silently.*
    #[test]
    fn a_partial_overlap_of_name_sets_does_not_agree() {
        let left = [
            observation(1, 0x01, Some("doc-old")),
            observation(2, 0x01, Some("doc-shared")),
        ];
        let right = [observation(3, 0x02, Some("doc-shared"))];

        assert_eq!(
            verdict_for_hostname_agreement(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "a rule that supports on a partial overlap claims to know which name is current"
        );
    }

    /// The same decision on its other shape — a subset rather than a crossing overlap, IN BOTH
    /// ORIENTATIONS.
    ///
    /// `is_subset` is the third spelling a developer reaches for, and it was measured GREEN across the
    /// whole suite. Equality refuses it: a side that names strictly more is not the same side.
    ///
    /// 🔴 **Both orientations are asserted because ONE of them is not enough, and my own prove-to-red
    /// is what said so.** `names_a.is_subset(&names_b)` is directional: with the superset on the LEFT
    /// it answers `false` and this test would have stayed GREEN under the very mutation it names. *A
    /// guard written for a directional operator and exercised in one direction is a guard placed where
    /// half the defect cannot occur* — the epic's dominant class, caught here by predicting the
    /// mutation before running it rather than by reading.
    #[test]
    fn a_superset_of_names_does_not_agree_with_its_subset() {
        let many = [
            observation(1, 0x01, Some("doc-a")),
            observation(2, 0x01, Some("doc-b")),
        ];
        let few = [observation(3, 0x02, Some("doc-a"))];

        assert_eq!(
            verdict_for_hostname_agreement(&side(&many), &side(&few)).verdict,
            Verdict::Neutral,
            "one side naming strictly more is not agreement, under equality"
        );
        assert_eq!(
            verdict_for_hostname_agreement(&side(&few), &side(&many)).verdict,
            Verdict::Neutral,
            "nor is it agreement the other way round — and this is the orientation a subset test \
             would answer TRUE on"
        );
    }

    /// 🔴 **EQUAL CARDINALITY, CROSSING SETS — the one population that closes Guy's arbitration, and
    /// the two tests above do NOT close it.**
    ///
    /// Both divergence guards above are **2-vs-1 in cardinality**, so *cardinality alone* separates the
    /// readings there — and a reading that gates on `len()` first therefore passes them. Measured by
    /// the edge-case review layer: `names_a.len() == names_b.len() && !names_a.is_disjoint(&names_b)`
    /// left **1 060 tests, clippy `--all-targets` and all ten gates GREEN**, while answering
    /// `Supports` on `{doc-a, doc-b}` against `{doc-a, doc-c}` — *a crossing partial overlap of equal
    /// size*, which is exactly the "one shared name settles which name is current" semantics Guy
    /// refused.
    ///
    /// 🔑 *The both-orientations repair one test up was the right instinct applied to the WRONG AXIS:
    /// it hardened DIRECTION and left CARDINALITY open.* This test closes the axis that was open, and
    /// it is the only guard here that distinguishes equality from every intersection-flavoured reading
    /// at once.
    #[test]
    fn two_name_sets_of_equal_size_that_merely_cross_do_not_agree() {
        let left = [
            observation(1, 0x01, Some("doc-a")),
            observation(2, 0x01, Some("doc-b")),
        ];
        let right = [
            observation(3, 0x02, Some("doc-a")),
            observation(4, 0x02, Some("doc-c")),
        ];

        assert_eq!(
            verdict_for_hostname_agreement(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "sharing one name out of two is not agreement, however equal the two counts are — and a \
             reading that compares sizes before contents passes every other guard here"
        );
        assert_eq!(
            verdict_for_hostname(&side(&left), &side(&right)).verdict,
            Verdict::Neutral,
            "nor is it opposition: the sets are not disjoint, so both rules stay silent together"
        );
    }

    /// 🔑 **What the decision BUYS, stated as the property that actually discriminates.**
    ///
    /// On a partial overlap the two rules are silent TOGETHER — neither supports nor opposes.
    /// ⚠️ This replaces the mutual-exclusivity property the story first prescribed, which was
    /// measured to hold under **every** reading (0 violations over 64 adversarial subset pairs) and
    /// therefore could not red on the decision it was cited to carry: *a guard placed where the defect
    /// cannot occur reads as coverage and is none*.
    #[test]
    fn on_a_partial_overlap_both_hostname_rules_stay_silent() {
        let left = [
            observation(1, 0x01, Some("doc-old")),
            observation(2, 0x01, Some("doc-shared")),
        ];
        let right = [observation(3, 0x02, Some("doc-shared"))];
        let (a, b) = (side(&left), side(&right));

        assert_eq!(
            verdict_for_hostname_agreement(&a, &b).verdict,
            Verdict::Neutral,
            "agreement needs the sets to be equal"
        );
        assert_eq!(
            verdict_for_hostname(&a, &b).verdict,
            Verdict::Neutral,
            "opposition needs the sets to share nothing; a partial overlap is neither"
        );
    }

    // ---- AC6: what a verdict that ARGUES leaves behind ----

    /// D19, on the merge side: a `Supports` carries both sides' observations, sorted.
    ///
    /// Measured: returning an empty vector here left the whole suite green before this test existed.
    /// The sort is shared with [`verdict_for_hostname`] through [`evidence_of`], which is why a
    /// mutation of the order now reds every rule that argues rather than one of two copies.
    #[test]
    fn an_agreeing_verdict_carries_its_observations_in_order() {
        let left = [observation(9, 0x01, Some("doc-vm-alpha"))];
        let right = [observation(3, 0x02, Some("doc-vm-alpha"))];

        let forward = verdict_for_hostname_agreement(&side(&left), &side(&right));
        let backward = verdict_for_hostname_agreement(&side(&right), &side(&left));

        assert!(
            !forward.evidence.is_empty(),
            "a verdict that ARGUES leaves its observations behind (D19)"
        );
        assert_eq!(
            forward.evidence, backward.evidence,
            "the evidence of a pair does not depend on which side was the left argument"
        );
        assert!(
            forward.evidence.windows(2).all(|w| w[0] <= w[1]),
            "sorted, so the order is a property of the pair and not of the call"
        );
    }

    /// A `Neutral` carries none, and it names this rule anyway.
    ///
    /// The convention [`verdict_for_hostname`] established: D19 is about a verdict that ARGUES, and a
    /// `Neutral` does not — but the rule id is what lets a reader see WHICH rule declined.
    #[test]
    fn a_neutral_agreement_carries_no_evidence_and_still_names_its_rule() {
        let left = [observation(1, 0x01, Some("doc-a"))];
        let right = [observation(2, 0x02, Some("doc-b"))];

        let verdict = verdict_for_hostname_agreement(&side(&left), &side(&right));

        assert_eq!(verdict.verdict, Verdict::Neutral);
        assert!(
            verdict.evidence.is_empty(),
            "a verdict that does not argue leaves nothing behind"
        );
        assert_eq!(
            verdict.rule.0, L2_HOSTNAME_AGREES,
            "a Neutral still says which rule declined"
        );
    }

    // ---- the fact §0.0 rests on, made executable ----

    /// 🔴 **A `Supports` CANNOT MAKE A MERGE, and this test is what says so when D13 changes.**
    ///
    /// [`crate::identity::cascade::decide`] reaches `Match` through one arm only, and that arm needs a
    /// `Decisive`. A lone `Supports` lands on the row `architecture.md:972` calls *weak evidence* and
    /// abstains as `Ambiguous` — which `score` scores a **fail** against a `must-merge` expectation,
    /// the cell D18 calls cowardice.
    ///
    /// 🔑 So `shared-hardware-vm-must-merge`, the one trap that names this rule, cannot score a pass;
    /// nor can story 6.8's two, which expect a merge from a `Supports` rule just as this one does.
    /// **Guy's decision of 2026-09-24: this story measures that and does not decide it** — *what makes
    /// a merge at L2* is Epic 6's retrospective's, which may edit `epics.md`.
    ///
    /// ⚠️ It is also AC4's reason, re-measured rather than inherited from story 6.7: that story's
    /// abstention was `AbsenceOfProof` from an `Opposes`, this one is `Ambiguous` from a `Supports` —
    /// **two different rows of the table** — and both carry no rule, so `run_trap`'s `(Some, Some)`
    /// comparison never fires and a misspelled id cannot make a trap red.
    #[test]
    fn a_supports_only_verdict_abstains_as_ambiguous_and_names_no_rule() {
        use crate::identity::cascade::{Conclusion, IdentityAbstentionCause, decide};
        use crate::identity::l1::CURRENT_RULESET_VERSION;

        let left = [observation(1, 0x01, Some("doc-vm-alpha"))];
        let right = [observation(2, 0x02, Some("doc-vm-alpha"))];
        let verdict = verdict_for_hostname_agreement(&side(&left), &side(&right));
        assert_eq!(
            verdict.verdict,
            Verdict::Supports,
            "the premise of this test"
        );

        let decision = decide(vec![verdict], CURRENT_RULESET_VERSION);

        assert_eq!(
            decision.conclusion,
            Conclusion::Abstained {
                cause: IdentityAbstentionCause::Ambiguous
            },
            "a lone Supports is weak evidence: it abstains, so no composition of this rule alone \
             can answer a must-merge trap, and no misspelled id can be caught through the trap path"
        );
        // ⚠️ NO `!matches!(…, Match { .. })` ASSERTION HERE, and its absence is deliberate. One stood
        // here and it was **strictly implied** by the `assert_eq!` above — if the conclusion IS
        // `Abstained{Ambiguous}` it cannot also be a `Match` — so it could not fail unless its
        // predecessor already had, and its message asserted two things this test does not measure.
        // Story 6.9's blind review layer found it. *A second assertion that the first one entails is
        // not a second carrier.*
        //
        // 🔑 And the RULE-ABSENCE half of this test's own name is carried by the TYPE, not by a line
        // here: `Conclusion::Abstained { cause }` has no rule field, so the struct literal above would
        // not compile if it gained one. That is a better carrier than an assertion, and saying which
        // carries what is the point.
    }
}
