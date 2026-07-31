//! The identity cascade — the vocabulary it speaks, the type it returns, and the algebra between.
//!
//! D13 fixes the cascade's shape: **all rules are evaluated** (never first-match-wins), each yields
//! an enumerated verdict, and the verdicts combine by an **algebra, never a sum** — *"if the output
//! is a float, B has won in disguise"* [architecture.md:956-965]. This file holds that vocabulary —
//! [`Verdict`], [`RuleVerdict`], [`Conclusion`], [`Decision`], [`RulesetVersion`] and
//! [`IdentityAbstentionCause`] — **and the algebra itself, [`decide`]**, which combines a verdict set
//! into a conclusion over D13's six-row table plus the one input class that table leaves uncovered.
//!
//! **Rules now produce verdicts, and `decide` has a caller** — [`crate::identity::l1`], which joins
//! observations on the scope-qualified key and answers a pair with one [`RuleVerdict`]. ⚠️ Be exact
//! about how far that goes: L1 emits **three** of the five verdicts — [`Verdict::Decisive`],
//! [`Verdict::Disqualifying`] and [`Verdict::Neutral`]. **[`Verdict::Supports`] and
//! [`Verdict::Opposes`] still have no producer**, and gain one with Epic 6's `l2-*` rules.
//!
//! The vocabulary is chosen before the engine on purpose — the same order D19 imposed on the metrics
//! harness in Epic 4, *"a metric written after the engine is bent to fit the engine"* — and because
//! story 4.6a recorded the abstention half of the choice as Epic 5's to make.
//!
//! # Four judgements, four types, and none of them is the others
//!
//! | type | whose judgement | of what | variants | owner |
//! |---|---|---|---|---|
//! | [`Verdict`] | ONE rule | one candidate pair | 5 | this file, story 5.4 |
//! | [`Conclusion`] | the CASCADE | a whole verdict set | 3 | this file, story 5.4 |
//! | [`crate::score::Outcome`] | the trap harness | what something ANSWERED about a trap | 3 | story 4.6a |
//! | [`crate::score::TrapVerdict`] | the trap RUNNER | whether one trap passed | 3 | story 4.7a |
//!
//! ⚠️ **The table is about JUDGEMENTS, not about names.** Two types that carry the word *verdict*
//! are deliberately not rows in it, because neither is a judgement: [`RuleVerdict`] is the
//! `(rule, verdict, evidence)` triple — an ELEMENT that carries a [`Verdict`] — and
//! [`crate::score::VerdictVectorEntry`] is the harness-side placeholder for that same triple,
//! uninhabited until story 5.7 unifies the two.
//!
//! [`Conclusion`] and [`crate::score::Outcome`] look redundant and are not. `Outcome` is what a
//! harness writes down about *any* answer, including a hand-authored one in a test; `Conclusion` is
//! what the engine concluded, and it never travels alone — it travels inside a [`Decision`], with
//! its verdict vector and its ruleset version. **Nothing converts between them**, and the day they
//! meet is story 5.7's; see [`Decision`] for why no `From` impl exists.

use crate::observation::ObsId;
use crate::trap::RuleId;

/// What ONE rule says about ONE candidate pair — D13's enumerated verdict.
///
/// D13 writes this enum out by name: `enum Verdict { Decisive, Supports, Neutral, Opposes,
/// Disqualifying }` [architecture.md:964]. The five variants and their spelling are the
/// architecture's, not a paraphrase.
///
/// ⚠️ **Not [`crate::score::TrapVerdict`]**, which is what the trap RUNNER says about one trap.
/// This is what ONE RULE says about ONE candidate pair. The two names are four letters apart and
/// the judgements are not comparable; the module doc above tabulates all four.
///
/// # This type is the vocabulary; [`decide`] is what combines it
///
/// D13's contract is that **all rules are evaluated** and that *"verdicts combine by an **algebra,
/// not a sum**"* [architecture.md:960-961], because *"REFUSED: `rule -> confidence: f64` … if the
/// output is a float, B has won in disguise"* [architecture.md:956-958]. The six-row table at
/// [architecture.md:967-974] is implemented by [`decide`], in this file, and
/// [`crate::identity::l1`] is what calls it. ⚠️ **Only three of the five variants have a producer**
/// — `Decisive`, `Disqualifying` and `Neutral`, all from L1; `Supports` and `Opposes` gain one with
/// Epic 6's `l2-*` rules.
///
/// ⚠️ **The six rows do not cover every input.** Enumerated over the PRESENCE of each variant, the
/// table leaves exactly one class unanswered: at least one `Opposes`, with no `Decisive`, no
/// `Supports` and no `Disqualifying`. It is not *"only `Neutral` / nothing"* and it is not
/// *"`Supports` AND `Opposes`"*. **[`decide`] carries the enumeration and Guy's arbitration of it**
/// (2026-07-29: it abstains for absence of proof); this doc names the gap so a reader does not
/// mistake five variants and six rows for a complete specification.
///
/// # Why there is no ordering
///
/// D20 refuses evidence STRENGTH as a default and specifies the shape of its return if it ever
/// comes back: *"if strength returns, it returns as an ORDINAL, not a weight: `Opposes(Weak) |
/// Opposes(Strong)`. The enum grows, the table grows, it stays finite and exhaustively enumerable.
/// No float decides."* [architecture.md:1374-1376], under a four-condition ADR
/// [architecture.md:1378-1394]. Deriving `Ord` here would let two verdicts be COMPARED by magnitude
/// today, which is the move that ADR exists to gate.
// Deliberately absent, each with a reason: `Serialize`/`Deserialize` (nothing persists a verdict —
// story 5.9 if it persists one), `PartialOrd`/`Ord` (see the doc above — D20's ADR owns it),
// `Display` (story 5.14 renders through the `t!()` seam), and `#[non_exhaustive]` (never: it would
// force a `_` arm in `opencmdb-bin`, which is a DIFFERENT crate, and destroy the `error[E0004]`
// that makes a sixth variant break every match there). ⚠️ It would NOT weaken story 5.4b's table:
// `#[non_exhaustive]` is inert within the defining crate, and 5.4b's table lives in this one. The
// loss is downstream exhaustiveness, and that is the whole of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// This rule alone settles the pair — the strongest thing a rule may say.
    ///
    /// It does not settle the DECISION: D13's table pairs a `Decisive` with `Opposes` to produce
    /// `Ambiguous`, the cloned-MAC case [architecture.md:971]. A rule is decisive about its own
    /// evidence, never about the outcome.
    Decisive,
    /// This rule argues FOR the pair being one interface, without settling it.
    Supports,
    /// This rule has nothing to say about this pair.
    ///
    /// The honest answer of a rule that does not KNOW — D20 names the opposite as the common bug:
    /// *"the rule that wrongly `Opposes` should return `Neutral`: it does not KNOW, it BELIEVES it
    /// knows… nine parasitic abstentions out of ten are that"* [architecture.md:1383-1387].
    Neutral,
    /// This rule argues AGAINST the pair, without disqualifying it.
    Opposes,
    /// This rule forbids the pair outright — D13's *"absolute priority, short-circuits
    /// everything"* [architecture.md:969].
    ///
    /// Its two committed instances are STRUCTURAL facts read at ingestion, never scored: the
    /// IANA-reserved VRRP/HSRP MAC prefixes and the U/L bit — *"**Both are** `Disqualifying` as
    /// grouping anchors, known at ingestion."* [architecture.md:1002]. D13 is explicit that
    /// *"confusing an IANA fact with scoring turns a fact into a probability — and that is how
    /// weights get invented"* [architecture.md:996-998].
    Disqualifying,
}

impl Verdict {
    /// Every variant, so a caller can exercise the whole vocabulary without listing it.
    ///
    /// # What the witness below guarantees, and what it does not
    ///
    /// Adding a variant makes the `match` non-exhaustive — **`error[E0004]`, which stops the
    /// build** and forces a human decision at this exact site. That is the guarantee.
    ///
    /// **It does not mechanically force the new variant into the array, and that was MEASURED on
    /// this idiom's first instance ([`IdentityAbstentionCause::all`]), not assumed.** Repairing only
    /// the `error[E0004]` — adding a bare arm — leaves this function returning the old list while
    /// the whole suite stays green. Widening the literal *without* widening the return type is a
    /// separate `error[E0308]`; the two errors are **alternatives along one repair path, never
    /// simultaneous**. The array length is pinned at `5` in the signature so the second error exists
    /// at all.
    ///
    /// The residue is registered with an owner (story 5.14) rather than closed here: the two
    /// candidate closures were built and measured on the sibling enum and both were rejected.
    pub fn all() -> [Self; 5] {
        let all = [
            Self::Decisive,
            Self::Supports,
            Self::Neutral,
            Self::Opposes,
            Self::Disqualifying,
        ];
        // ⚠️ If you arrived here from an error[E0004] after adding a variant: adding a bare
        // `Self::NewThing => {}` arm silences this and is the WRONG repair — `all()` would then
        // return a list missing your variant, and every test that loops over it would skip the
        // variant in silence (measured on the sibling enum: the suite stays green). Add it to the
        // literal above and widen the return type. And read D20 first: a sixth verdict is the
        // "evidence strength" question, which needs an ADR before any code.
        for v in all {
            match v {
                Self::Decisive => {}
                Self::Supports => {}
                Self::Neutral => {}
                Self::Opposes => {}
                Self::Disqualifying => {}
            }
        }
        all
    }
}

/// One entry of the complete verdict vector: the `(rule, verdict, evidence)` triple.
///
/// D13: *"**explanation is free** (the list of `(rule, verdict, evidence)` IS the explanation)"*
/// [architecture.md:977-978]. D18 makes recording the whole list a requirement rather than a
/// courtesy: *"The harness records, for every case, the COMPLETE VERDICT VECTOR, not just the
/// outcome… **the anti-drift is not discipline, it is a data requirement.**"*
/// [architecture.md:1396-1399].
///
/// # One producer, and what it is held to
///
/// [`crate::identity::l1::verdict_for_pair`] builds these. The fields remain `pub` with no
/// constructor — the [`crate::score::ScoredRecord`] precedent — so the two validations a producer
/// would want are honoured **by that producer**, not by the type:
///
/// - a [`Decision`]'s conclusion names a rule its own vector contains — by construction for
///   everything [`decide`] returns, and a test says so;
/// - a verdict which ARGUES leaves non-empty evidence — stated and tested on the L1 producer, which
///   is where it can red at all, since [`decide`] never reads [`Self::evidence`].
///
/// ⚠️ Both hold for what the L1 producer emits. **A `RuleVerdict` built by struct literal elsewhere
/// is still unconstrained**, and that residue stays registered.
///
/// # Its relationship to [`crate::score::VerdictVectorEntry`]
///
/// That type is the HARNESS-side placeholder for this same triple, and it is **uninhabited** so
/// that `ScoredRecord::verdict_vector` is provably empty until the trap runner records a real run.
/// A producer now exists, but nothing yet feeds the harness — **story 5.7 owns the unification**.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleVerdict {
    /// The rule that spoke. A [`RuleId`] rather than an enum: five of the seven rule names the
    /// committed corpus writes are `l2-*`, which Epic 6 designs, so closing the enum here would
    /// enumerate rules nobody has specified.
    pub rule: RuleId,
    /// What it said.
    pub verdict: Verdict,
    /// The observations the rule read to say it, by the stable id stories 4.1/4.2 froze — never a
    /// line number, which *"would silently shift under the truth"*.
    ///
    /// This is the SMALLEST evidence that is not invented. The architecture requires a firing rule
    /// to leave evidence behind — *"a rule that fires without leaving its `rule_id` in the database
    /// is a rule we cannot debug in production"* [architecture.md:1309-1310] — and **shapes it
    /// nowhere**. A richer payload (the fact values, the candidate pair, a rendered sentence) is a
    /// design nothing asks for: the L1 producer fills this, on a verdict that ARGUES, with both
    /// sides' [`ObsId`]s sorted — a [`Verdict::Neutral`] legitimately carries none — and no
    /// consumer reads more.
    pub evidence: Vec<ObsId>,
}

/// The version of the rule set a [`Decision`] was produced under.
///
/// D14: *"**`ruleset_version` is mandatory:** without it, improving the engine is **a silent data
/// migration — the worst kind**."* [architecture.md:1044-1045]. D20 says what an increment means:
/// *"Any reintroduction increments `ruleset_version`; existing links are not recomputed (they carry
/// the version they were decided under)."* [architecture.md:1392-1393].
///
/// # The constant lives with the rules; there is still no default and no ordering
///
/// A ruleset now exists — L1's two rules — so the constant naming its version does too:
/// [`crate::identity::l1::CURRENT_RULESET_VERSION`]. It lives beside the rules it versions rather
/// than here, so that a value asserting "these rules are there" sits where they are.
///
/// **There is still no `Default`, and that is load-bearing**: the version arrives as a parameter
/// when a producer builds a [`Decision`], and the absent `Default` is what makes it unforgettable —
/// removing the field breaks every construction site and every read.
///
/// **No value is refused, including `RulesetVersion(0)`.** D14's "mandatory" is about PRESENCE, not
/// meaning, and validating a number against nothing would be an invention. What the L1 producer
/// states instead is the weaker true thing: the version it emits is not that meaningless zero.
///
/// **No `PartialOrd`/`Ord`**: the first consumer that ORDERS two versions is persistence (D20's
/// "existing links are not recomputed" is a claim about which version a row carries, not a
/// comparison anything performs today). Story 5.9 adds it if it needs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RulesetVersion(
    /// The version number itself. Opaque: nothing derives meaning from its value yet.
    pub u32,
);

/// What the cascade concluded — D13's three-way decision.
///
/// D13: *"candidate generation (blocking) -> verdicts -> **three-way decision** (`match` /
/// `no-match` / **`ambiguous` -> abstain**)"* [architecture.md:931-932].
///
/// # A decision names a RULE; an abstention names a CAUSE
///
/// This mirrors [`crate::score::Outcome`]'s shape deliberately, so that [`Decision::rule`] can
/// return `None` for an abstention **by construction** — the property
/// [`crate::score::run_trap`] already relies on to leave an abstention out of its `(verdict, rule)`
/// assertion without a runtime guard. Story 4.2 fixed the rule half in the truth format:
/// *"Every decision names a rule; only an abstention names a cause"* (`trap.rs`), because *"a
/// refusal without a named rule cannot be told apart from an engine that simply found nothing, and
/// that is undebuggable in production"*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Conclusion {
    /// These observations describe one interface, and this rule settled it — D13's row *"a
    /// `Decisive`, no `Opposes`"* [architecture.md:970].
    Match {
        /// The rule that settled it.
        rule: RuleId,
    },
    /// These observations describe different interfaces, and this rule FORBADE the pair — D13's row
    /// *"any `Disqualifying`"* [architecture.md:969].
    ///
    /// ⚠️ **D13's `NoMatch` covers two situations and only ONE of them lands here.** The other —
    /// *"only `Neutral` / nothing → `NoMatch` (absence of proof)"* [architecture.md:974] — has **no
    /// rule to name**, so it cannot be represented by this variant and becomes
    /// [`Self::Abstained`] with [`IdentityAbstentionCause::AbsenceOfProof`]. That fork is why this
    /// variant carries a rule and cannot be given an optional one: an engine that mapped all of
    /// `NoMatch` onto a refusal *"would fail every honest `must-abstain` trap"*, the exact case D18
    /// says must NOT be gated. **[`decide`] is what chooses the side**: a `Disqualifying` present
    /// lands here and names its rule; its absence, with nothing arguing for the pair, abstains.
    NoMatch {
        /// The rule that opposed the pair — never the rule that was merely tempting.
        rule: RuleId,
    },
    /// The cascade took no decision, for this cause.
    Abstained {
        /// Why it did not conclude.
        cause: IdentityAbstentionCause,
    },
}

/// What the identity cascade returns: the conclusion, the explanation, and the ruleset version.
///
/// A struct rather than an enum, and the reason is D14. `ruleset_version` is carried **once**, so
/// "mandatory" rests on one declaration instead of on three variants agreeing — and a variant added
/// to [`Conclusion`] later cannot forget it. [`crate::score::Outcome`] is an enum because it has no
/// common field to carry; the two types differ in envelope and agree in algebra, which is exactly
/// what [`Self::rule`] pins.
///
/// # One producer, and still no consumer
///
/// [`crate::identity::l1::decide_pair`] produces these. There is still **no blocker**: the pair it
/// answers arrives from its caller, and candidate generation is the next story's.
/// **No `From<Decision> for Outcome` exists in either direction**: mapping the engine's return onto
/// the harness's record is a decision about the release gate, and it belongs to story 5.7 with a
/// story behind it — not to a silent conversion. The same refusal, for the same reason, kept the
/// two abstention vocabularies unbridged in story 5.3.
///
/// # No float, anywhere
///
/// D13's universal rule: *"floats may RANK, never DECIDE… The moment a float decides,
/// explainability and the truth table die the same day"* [architecture.md:988-990]. No field here
/// is a float and none is a magnitude: [`RulesetVersion`] is an identifier, [`Verdict`] is
/// enumerated and unordered. There is no ranking value at all, because L1 is a deterministic lookup
/// with nothing to rank; D13's milli-units corollary [architecture.md:991-993] binds the day one
/// appears. **`cargo xtask ci`'s `float-free` gate holds this mechanically** — it reds on an `f32`,
/// an `f64` or a bare float literal in code anywhere under `identity/`.
///
/// # What is representable and not refused
///
/// The fields are `pub` with no constructor ([`crate::score::ScoredRecord`]'s precedent), so a
/// `Decision` whose [`Self::conclusion`] names a rule **absent from its own
/// [`Self::verdict_vector`]** compiles, and so does a [`Conclusion::Match`] with an empty vector —
/// *"merged, with no explanation"*, which is what D13's *"the list … IS the explanation"* exists to
/// prevent.
///
/// **Through [`decide`] both are unreachable, and that is by construction rather than by a check**:
/// it selects the named rule FROM the vector it then returns, and an empty vector abstains rather
/// than matching. A test walks all 32 verdict subsets and asserts it.
///
/// ⚠️ **Be precise about WHICH emptiness that closes: the empty VECTOR, not empty EVIDENCE.**
/// [`decide`] never reads [`RuleVerdict::evidence`], so a single
/// `RuleVerdict { verdict: Decisive, evidence: vec![] }` still yields a [`Conclusion::Match`] whose
/// explanation explains nothing — D13's clause defeated one level below the case closed here. That
/// invariant can only be stated by a rule that FIRES, and one now does:
/// [`crate::identity::l1::verdict_for_pair`] carries both sides' [`ObsId`]s on every verdict that
/// ARGUES, with a test that reds if it stops. ⚠️ **That closes it for the L1 producer, not for the
/// type** — a struct literal is still unconstrained. _(This paragraph closed both emptinesses in
/// one sentence until the 2026-07-30 review.)_
///
/// ⚠️ **A struct literal built anywhere else is still unconstrained**, because the fields are `pub`
/// and there is no constructor; that residue is registered with story 5.9, the first story that
/// reconstructs a `Decision` from somewhere other than `decide`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// What the cascade concluded.
    pub conclusion: Conclusion,
    /// The complete verdict vector — D18's word, and D18's data requirement. Every rule that spoke,
    /// what it said, and what it read.
    pub verdict_vector: Vec<RuleVerdict>,
    /// The version of the ruleset that produced this decision (D14).
    pub ruleset_version: RulesetVersion,
}

impl Decision {
    /// The rule a DECISION named, or `None` for an abstention — the mirror of
    /// [`crate::score::Outcome::rule`].
    ///
    /// A match names the rule that settled the pair; a refusal names the rule that FORBADE it. An
    /// abstention took no decision, so it carries a cause and no rule — **the type says so**, which
    /// is what lets a future consumer leave an abstention out of a `(verdict, rule)` comparison by
    /// construction rather than by a runtime guard.
    ///
    /// It lives on `Decision` and not on [`Conclusion`], and there is no `Conclusion::rule()`: the
    /// consumer that needs it holds a decision, so an accessor on the inner enum would have no
    /// caller. Same no-consumer argument that keeps serde off these types.
    pub fn rule(&self) -> Option<&RuleId> {
        match &self.conclusion {
            Conclusion::Match { rule } | Conclusion::NoMatch { rule } => Some(rule),
            Conclusion::Abstained { .. } => None,
        }
    }
}

/// Combine a verdict set into a [`Decision`] — D13's algebra, as a TOTAL pure function.
///
/// D13: **all rules are evaluated** (never first-match-wins), each yields an enumerated verdict, and
/// *"verdicts combine by an **algebra, not a sum**"* [architecture.md:960-961]. This is that algebra.
/// It reads nothing but its arguments: no clock, no I/O, no SQL — *"the engine is a pure function"*
/// [architecture.md:3302], and *"the engine never touches the clock"* [architecture.md:3364].
///
/// # It is TOTAL, and the table it implements is not
///
/// Every `Vec<RuleVerdict>` gets an answer — the empty one, and one naming the same rule twice.
/// There is no `Result`, no `panic!` and no `unwrap`: there is no error to carry.
///
/// D13's table has six rows. Enumerated over the PRESENCE of each verdict it leaves **exactly one
/// input class unanswered**, and this is the enumeration, so a reader can re-derive the gap rather
/// than trust it (`Disqualifying` absent throughout — it short-circuits):
///
/// The last column is what **this function returns**, which is not always the words D13 uses — the
/// `D13 row` column carries those. The two differ on exactly one row and the difference is
/// deliberate: see [`Conclusion::NoMatch`]'s doc.
///
/// | `Decisive` | `Supports` | `Opposes` | D13 row (its words) | what `decide` returns |
/// |---|---|---|---|---|
/// | ✔ | ✔ | ✔ | `:971` Ambiguous | `Abstained { Ambiguous }` |
/// | ✔ | ✔ | ✗ | `:970` Match | `Match { rule }` |
/// | ✔ | ✗ | ✔ | `:971` Ambiguous | `Abstained { Ambiguous }` |
/// | ✔ | ✗ | ✗ | `:970` Match | `Match { rule }` |
/// | ✗ | ✔ | ✔ | `:973` Ambiguous | `Abstained { Ambiguous }` |
/// | ✗ | ✔ | ✗ | `:972` Ambiguous | `Abstained { Ambiguous }` |
/// | ✗ | ✗ | ✔ | **NONE** | **arbitrated below** |
/// | ✗ | ✗ | ✗ | `:974` NoMatch (absence of proof) | `Abstained { AbsenceOfProof }` ⚠️ |
///
/// ⚠️ marks the one row where the engine's answer is not D13's word. `Conclusion::NoMatch` carries a
/// rule and this class has none to name, so mapping it there is not expressible; D18 forbids gating
/// an honest abstention behind a refusal. _(This column read `NoMatch` for that row until the
/// 2026-07-30 review: a faithful quote of D13 sitting in a column that means the return value, in a
/// table whose own preamble invites re-derivation.)_
///
/// **Guy's arbitration, 2026-07-29: `>=1 Opposes` alone concludes
/// [`IdentityAbstentionCause::AbsenceOfProof`]** — nothing argues FOR the merge, so there is no merge
/// to refuse, and D13 deliberately reserves the refusal-that-names-a-rule for `Disqualifying`. The
/// correction to D13 itself is a MILESTONE edit of `architecture.md`, never a story task; it is
/// registered with a GitHub issue.
///
/// # Which rule a decision names
///
/// A decision names ONE rule while several may be `Disqualifying` or `Decisive` at once. The one
/// named is the **lexicographically smallest [`RuleId`] among the verdicts that qualified for that
/// arm** — the `Disqualifying` ones for a refusal, the `Decisive` ones for a match.
///
/// ⚠️ **That is a tiebreak nobody designed**, and it is chosen because it is the only
/// order-independent rule available that invents nothing. D13 refuses the alternative by name —
/// *"which rule fires first? The one written first. **That is not a decision, it is an accident of
/// file order**"* [architecture.md:936-937].
///
/// ⚠️ **Rules now exist, and L1 still supplies no priority — because it supplies no tie.**
/// [`crate::identity::l1`] emits **exactly one** verdict per pair, so its two rules never appear in
/// one vector and the tiebreak is never consulted on an L1 decision. Designing a priority between
/// them would be ordering two things that cannot meet. The tiebreak keeps its byte-order placeholder
/// and its three costs below; the first vector that can hold two verdicts is Epic 6's, and that is
/// where a designed priority gets an input. Registered.
///
/// ⚠️ **It is not free of consequence, though, and calling it "semantically empty" was wrong.**
/// [`RuleId`] is a `String` with a derived `Ord`, so this is BYTE order, and three consequences
/// follow that a reader must know before relying on it:
/// - it is only empty WITHIN a tier: `l1-distinct-mac` is not "more disqualifying" than
///   `l1-exact-mac`, true — but across tiers `l1-*` always beats `l2-*`, so the accident of file
///   order has been replaced by a systematic preference that merely looks deliberate;
/// - it is case-sensitive: `L1-exact` sorts before `l1-anything`, because uppercase bytes are lower;
///   and
/// - it **flips tiers at ten**: `l10-x` sorts before `l2-x`. The day rule numbering reaches double
///   digits, which rule a decision names changes without anyone editing this function.
///
/// _(This paragraph claimed the tiebreak had "no semantic content" until the 2026-07-30 review,
/// illustrated with the one example — two rules in the same tier — where the claim holds.)_
///
/// # What it does not do
///
/// It does not refuse a vector naming the same rule twice, and it does not validate
/// `ruleset_version` — including `RulesetVersion(0)`, which D14's "mandatory" does not forbid since
/// that is about PRESENCE. Both are claims about what a PRODUCER emits, so they are stated on the
/// producer's side. ⚠️ The L1 producer emits one verdict per pair, so "one verdict per rule" holds
/// there **trivially**: the property with content needs a producer that can emit several, which is
/// Epic 6's. That half stays open.
///
/// The conclusion and the vector are built together here, so a conclusion naming a rule absent from
/// its own vector is unreachable **through this function** — the rule is selected FROM the vector
/// that is then returned, and `decide(vec![], _)` abstains rather than matching. A `Decision` built
/// by struct literal elsewhere is still unconstrained (the fields are `pub`); that residue is
/// registered with story 5.9.
pub fn decide(verdict_vector: Vec<RuleVerdict>, ruleset_version: RulesetVersion) -> Decision {
    let has = |w: Verdict| verdict_vector.iter().any(|rv| rv.verdict == w);

    // ⚠️ A `match` on a tuple, NOT an `if`/`else if` chain, and that is load-bearing: it is what
    // makes deleting an arm an `error[E0004]` instead of a silent behaviour change. An `if`-chain
    // with a trailing `else` was measured to swallow the loss of the arbitration arm while every
    // one of the sixteen classes kept its answer.
    //
    // ⚠️ The first two slots hold the SELECTED RULE, not a boolean, so the presence test and the
    // selection are ONE act: `Some(rule)` is simultaneously "a Disqualifying exists" and "here it
    // is". No arm then holds an `Option` it must prove `Some`, which is why this function has no
    // unreachable branch to justify. It was written with booleans first, and the two dead arms that
    // shape produced both mapped an impossible state onto `AbsenceOfProof` — a legitimate answer
    // for two other classes, so a broken invariant would have degraded a refusal into an
    // honest-looking abstention with nothing observable.
    let conclusion = match (
        smallest_rule_with(&verdict_vector, Verdict::Disqualifying),
        smallest_rule_with(&verdict_vector, Verdict::Decisive),
        has(Verdict::Supports),
        has(Verdict::Opposes),
    ) {
        // architecture.md:969 — "any Disqualifying → NoMatch, absolute priority, short-circuits
        // everything". Checked first, so the priority is structural and not an accident of order.
        (Some(rule), _, _, _) => Conclusion::NoMatch { rule },

        // architecture.md:970 — "a Decisive, no Opposes → Match".
        (None, Some(rule), _, false) => Conclusion::Match { rule },

        // architecture.md:971 — "a Decisive, >=1 Opposes → Ambiguous", the cloned-MAC case.
        (None, Some(_), _, true) => Conclusion::Abstained {
            cause: IdentityAbstentionCause::Ambiguous,
        },

        // architecture.md:972 — "no Decisive, >=1 Supports, no Opposes → Ambiguous" (weak evidence).
        (None, None, true, false) => Conclusion::Abstained {
            cause: IdentityAbstentionCause::Ambiguous,
        },

        // architecture.md:973 — "Supports AND Opposes → Ambiguous" (conflict).
        (None, None, true, true) => Conclusion::Abstained {
            cause: IdentityAbstentionCause::Ambiguous,
        },

        // ⚠️ NOT a D13 row — this is the class the table does not cover, and there is no
        // architecture line to cite. Guy's arbitration, 2026-07-29: `>=1 Opposes` with nothing
        // arguing for the merge abstains for absence of proof. See this function's doc.
        (None, None, false, true) => Conclusion::Abstained {
            cause: IdentityAbstentionCause::AbsenceOfProof,
        },

        // architecture.md:974 — "only Neutral / nothing → NoMatch (absence of proof)", which this
        // engine answers as `Abstained { AbsenceOfProof }` and not as `NoMatch`: that variant
        // carries a rule and this class has none to name. See `Conclusion::NoMatch`'s own doc — the
        // fork is deliberate and D18 is why. `Neutral` is not in the tuple because no row tests it:
        // it is what is left when nothing else spoke.
        (None, None, false, false) => Conclusion::Abstained {
            cause: IdentityAbstentionCause::AbsenceOfProof,
        },
    };

    Decision {
        conclusion,
        verdict_vector,
        ruleset_version,
    }
}

/// The lexicographically smallest [`RuleId`] among the verdicts carrying `wanted`, if any.
///
/// The tiebreak of [`decide`], factored out so the two rule-naming arms cannot drift apart.
///
/// The `Option` is not a nuisance to be discharged: [`decide`] matches on it directly, so `None`
/// **is** how "no rule carries this verdict" is expressed and `Some(rule)` is simultaneously the
/// presence test and its answer. That is why no caller needs `expect()` and no path panics.
///
/// ⚠️ `min()` is byte order over a `String` id — see [`decide`]'s doc for what that costs.
fn smallest_rule_with(verdicts: &[RuleVerdict], wanted: Verdict) -> Option<RuleId> {
    verdicts
        .iter()
        .filter(|rv| rv.verdict == wanted)
        .map(|rv| rv.rule.clone())
        .min()
}

/// Why the identity cascade did not conclude — the engine's own abstention vocabulary.
///
/// # Two variants, and the two D13 rows that produce none
///
/// D13's algebra is a table of six conditions over the verdict set [architecture.md:967-974]. Four
/// of those rows produce an abstention and are covered by the two variants below — three by
/// [`Self::Ambiguous`], one by [`Self::AbsenceOfProof`]. The remaining **two** are accounted for
/// here, so a reader does not find an uncovered row and add a variant for it:
///
/// - `any Disqualifying → NoMatch` [architecture.md:969] is an **active opposition**: a rule
///   opposes, so the answer names that rule and is a [`crate::score::Outcome::Refused`], never an
///   abstention.
/// - `a Decisive, no Opposes → Match` [architecture.md:970] is a [`crate::score::Outcome::Merged`].
///
/// `NoMatch` is therefore reached by two different rows — the `Disqualifying` one just named, which
/// is a decision, and the absence-of-proof one below — and **exactly one of them has no rule to
/// name**: that row is [`Self::AbsenceOfProof`].
///
/// ⚠️ **Seven input classes, not six.** D13's table has six rows, but enumerated over the PRESENCE of
/// each verdict it leaves one class uncovered — `>=1 Opposes` alone — which Guy arbitrated onto
/// [`Self::AbsenceOfProof`] on 2026-07-29. So: seven classes, five of them abstentions, two variants.
/// [`decide`] carries the enumeration and is where the arbitration lives.
///
/// A third variant is a **finding**, not a tidy-up. In particular this enum does not reproduce
/// `OutOfPerimeter`: the cascade's table has no such row, and D16 names the failure mode of
/// carrying causes across — *"if `Ambiguous` means both 'real conflict' and 'unmodelled case', it
/// means nothing, and the operator learns to ignore it"* [architecture.md:1112-1115].
///
/// # The two sides of a trap now speak different vocabularies, and that is safe by construction
///
/// [`crate::gap::AbstentionCause`] is the RECONCILIATION vocabulary (`OutOfPerimeter`,
/// `NoObservedValue`, `ConflictingObservations`). It says why comparing a *declared field* against
/// observations did not conclude, and in the trap corpus it lives on the **expectation** side —
/// [`crate::trap::Expectation::MustAbstain`] carries it, and three committed trap files write it.
/// This enum lives on the **outcome** side, in
/// [`crate::score::Outcome::Abstained`], and says why *the cascade* did not conclude: a question
/// about a verdict set, not about a field.
///
/// The two are never compared, and the reason is a mechanism rather than a promise:
/// [`crate::score::score`]'s 3×3 matches `Outcome::Abstained { .. }` with `..` and cannot read the
/// payload at all, and [`crate::score::run_trap`] compares rules only where BOTH sides return
/// `Some`, which an abstention never does — [`crate::score::Outcome::rule`] returns `None` for one
/// **by type**. There is no comparison here to go asymmetric. A test says so
/// (`the_two_abstention_vocabularies_are_never_compared`, in `score`'s test module).
///
/// If some future story wants to map one onto the other, it needs a decision and a story — not a
/// silent `impl From`, which is why none exists.
// Deliberately absent, each with an owner: `Serialize`/`Deserialize` (nothing persists a cause yet
// — story 5.9 if it persists one), `PartialOrd`/`Ord` (nothing orders or keys one — story 5.14 if
// it groups by cause), `Display` (story 5.14 renders through the `t!()` seam, not through
// `Display`), and `#[non_exhaustive]` (never: `opencmdb-bin` is another crate, so it would force a
// `_` arm on every downstream match and destroy the `error[E0004]` that makes a new variant break
// its consumers).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityAbstentionCause {
    /// The verdict set points both ways, or too weakly to point at all.
    ///
    /// Three of D13's rows produce it: *"a `Decisive`, >=1 `Opposes`"* — **the cloned-MAC case**
    /// [architecture.md:971] — *"no `Decisive`, >=1 `Supports`, no `Opposes`"* (weak evidence,
    /// `:972`), and *"`Supports` AND `Opposes`"* (conflict, `:973`).
    ///
    /// Whether those three should later become three variants is **open, with an owner**: story
    /// 5.14 owns the operator-facing grouping and is the first place a split can be justified by a
    /// consumer rather than by symmetry. Registered in `deferred-work.md`.
    Ambiguous,
    /// Nothing argues FOR the pair — so there is no merge to refuse and none to make.
    ///
    /// **Two input classes produce it, and only one of them is a D13 row:**
    /// - *"only `Neutral` / nothing → `NoMatch` (absence of proof)"* [architecture.md:974], where
    ///   nothing argues either way;
    /// - **`>=1 Opposes` with no `Decisive`, no `Supports` and no `Disqualifying`** — the class D13's
    ///   table leaves uncovered, arbitrated onto this variant by Guy on 2026-07-29. Here something
    ///   *does* argue, against; what is absent is anything arguing FOR, which is why the answer is an
    ///   abstention and not a refusal. See [`decide`], which carries the enumeration.
    ///
    /// *(This doc read "nothing in the verdict set argues either way" until [`decide`] existed —
    /// true of the D13 row alone, and falsified by the arbitrated class the same function serves.)*
    ///
    /// It is a separate variant because it is the half of `NoMatch` with no rule to name, and D18
    /// is explicit that this case must NOT be gated: *"an engine that abstains because there is NOT
    /// ENOUGH SIGNAL is being honest… We do not gate that"*. Mapping it to a refusal instead would
    /// fail every honest `must-abstain` trap.
    AbsenceOfProof,
}

impl IdentityAbstentionCause {
    /// Every variant, so a caller can exercise the whole vocabulary without listing it.
    ///
    /// # What the witness below guarantees, and what it does not
    ///
    /// Adding a variant makes the `match` non-exhaustive — **`error[E0004]`, which stops the
    /// build** and forces a human decision at this exact site. That is the guarantee, and it is the
    /// one that matters: given that a third variant is a finding rather than a routine addition
    /// (see this type's own doc), an error that refuses to compile IS the mechanism.
    ///
    /// **It does not mechanically force the new variant into the array, and that was measured, not
    /// assumed.** Repairing only the `error[E0004]` — adding a bare arm — leaves this function
    /// returning the old list while the whole suite stays green. Widening the literal *without*
    /// widening the return type is a separate `error[E0308]`; the two errors are alternatives along
    /// one repair path, never simultaneous. The array length is still pinned at `2` in the
    /// signature so the second error exists at all.
    ///
    /// Whether to close that path with a single-source construction is registered with an owner
    /// (story 5.14) rather than decided here: the alternatives measured for it either make adding a
    /// variant *frictionless*, which is the opposite of what this vocabulary wants, or cost a
    /// dependency in the domain crate.
    pub fn all() -> [Self; 2] {
        let all = [Self::Ambiguous, Self::AbsenceOfProof];
        // ⚠️ If you arrived here from an error[E0004] after adding a variant: adding a bare
        // `Self::NewThing => {}` arm silences this and is the WRONG repair — `all()` would then
        // return a list missing your variant, and every test that loops over it would skip the
        // variant in silence (measured: the suite stays green). Add it to the literal above and
        // widen the return type. Better still: re-read this type's doc first — a third variant is
        // a finding, not a routine addition.
        for c in all {
            match c {
                Self::Ambiguous => {}
                Self::AbsenceOfProof => {}
            }
        }
        all
    }
}

/// Tests for this file's types.
///
/// # Where a test lives, decided once
///
/// **A test lives with the item whose CLAIM it pins; the items it merely READS are dependencies,
/// imported and not owned.** Story 5.3's review left this convention to "whoever next adds a test
/// to `cascade.rs`", and these are those tests. It decides two cases:
///
/// - `a_decision_names_a_rule_and_an_abstention_does_not` and
///   `the_conclusion_mirrors_the_outcomes_rule_shape` pin claims about [`Decision`]'s shape.
///   [`crate::score::Outcome`] is the dependency the second one reads to express the mirror, not
///   its subject. Both belong here.
/// - `an_abstention_names_no_rule_whatever_its_cause` (story 5.3) pins a claim about the abstention
///   VOCABULARY — *"an abstention has no rule to name, for EVERY cause"* — and reads
///   `Outcome::rule()` as the mechanism that expresses it. Its subject is
///   [`IdentityAbstentionCause`], so it stays here too, by the convention rather than by fiat.
/// - Story 5.4b's six tests of [`decide`] pin claims about a function defined in this file, so the
///   convention places them here without a judgement call. [`crate::score::Outcome`] is not read by
///   any of them.
///
/// The truth-table tests of `score()` stay in `score.rs` under the same rule: their subject is
/// `score`, and this module's types are what they read.
///
/// # The oracle is deliberate redundancy
///
/// `expected_conclusion` restates D13's table independently of [`decide`]. That is the protected kind
/// of duplication this project names by hand (`fixtures.rs`'s `expected()`, `score.rs`'s
/// `Column::as_str()`), and a DRY pass must not collapse it: an expectation computed by calling the
/// code under test proves nothing.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::Outcome;
    use uuid::Uuid;

    fn obs(n: u128) -> ObsId {
        ObsId::from_uuid(Uuid::from_u128(n))
    }

    fn rule(name: &str) -> RuleId {
        RuleId(name.into())
    }

    /// The variant SET, not its length: on `[Self; 2]` the length is a tautology the return type
    /// already guarantees, so asserting it would prove nothing. What this pins is *which* two —
    /// renaming a variant, or swapping one for another, reds here.
    ///
    /// ⚠️ **What it does NOT catch, stated because the name reads stronger than the body:** a
    /// third variant added to the enum leaves `Ambiguous` and `AbsenceOfProof` in `all()`, so this
    /// test stays green. What stops that is [`IdentityAbstentionCause::all`]'s `error[E0004]`, and
    /// its doc says exactly how far that goes. "Exactly" in this name is carried by the `[Self; 2]`
    /// return type, not by the two assertions below.
    #[test]
    fn the_vocabulary_is_exactly_ambiguous_and_absence_of_proof() {
        let all = IdentityAbstentionCause::all();
        assert!(
            all.contains(&IdentityAbstentionCause::Ambiguous),
            "Ambiguous is D13's three-row abstention and must be in the witness"
        );
        assert!(
            all.contains(&IdentityAbstentionCause::AbsenceOfProof),
            "AbsenceOfProof is D13's no-rule-to-name row and must be in the witness"
        );
    }

    /// An abstention has no rule to name — for EVERY cause, by construction rather than by
    /// convention. This is what lets `run_trap` leave an abstention out of the `(verdict, rule)`
    /// assertion, and it must not depend on which cause was chosen.
    #[test]
    fn an_abstention_names_no_rule_whatever_its_cause() {
        for cause in IdentityAbstentionCause::all() {
            assert_eq!(
                Outcome::Abstained { cause }.rule(),
                None,
                "an abstention with cause {cause:?} must name no rule"
            );
        }
    }

    /// D13 names five verdicts [architecture.md:964] and this pins that set — renaming one, or
    /// swapping one for another, reds here.
    ///
    /// ⚠️ **What it does NOT catch, stated because the name reads stronger than the body:** a SIXTH
    /// variant added to the enum leaves the five in `all()`, so this test stays green. What stops
    /// that is [`Verdict::all`]'s `error[E0004]`, and its doc says exactly how far that goes. The
    /// length is carried by the `[Self; 5]` return type, not by the assertions below, so asserting
    /// it here would prove nothing.
    #[test]
    fn the_five_verdicts_of_d13_are_present() {
        let all = Verdict::all();
        for expected in [
            Verdict::Decisive,
            Verdict::Supports,
            Verdict::Neutral,
            Verdict::Opposes,
            Verdict::Disqualifying,
        ] {
            assert!(
                all.contains(&expected),
                "{expected:?} is one of D13's five verdicts and must be in the witness"
            );
        }
    }

    /// A decision names a rule; an abstention names a cause and no rule — for EVERY cause, not one
    /// hand-picked variant. This is the property that lets a consumer skip an abstention in a
    /// `(verdict, rule)` comparison by construction.
    #[test]
    fn a_decision_names_a_rule_and_an_abstention_does_not() {
        let matched = Decision {
            conclusion: Conclusion::Match {
                rule: rule("l1-exact-mac"),
            },
            verdict_vector: Vec::new(),
            ruleset_version: RulesetVersion(1),
        };
        assert_eq!(
            matched.rule(),
            Some(&rule("l1-exact-mac")),
            "a Match names the rule that settled the pair"
        );

        let refused = Decision {
            conclusion: Conclusion::NoMatch {
                rule: rule("l1-distinct-mac"),
            },
            verdict_vector: Vec::new(),
            ruleset_version: RulesetVersion(1),
        };
        assert_eq!(
            refused.rule(),
            Some(&rule("l1-distinct-mac")),
            "a NoMatch names the rule that forbade the pair"
        );

        for cause in IdentityAbstentionCause::all() {
            let abstained = Decision {
                conclusion: Conclusion::Abstained { cause },
                verdict_vector: Vec::new(),
                ruleset_version: RulesetVersion(1),
            };
            assert_eq!(
                abstained.rule(),
                None,
                "an abstention with cause {cause:?} took no decision, so it names no rule"
            );
        }
    }

    /// The claim the epic makes about `Decision` — *"the same shape `Outcome` mirrors, so
    /// `run_trap`'s existing assertion needs no runtime guard"* — made executable.
    ///
    /// Conclusion by counterpart outcome: a rule is named on exactly the same two of three. The
    /// ENVELOPES differ deliberately (`Decision` is a struct carrying the ruleset version,
    /// `Outcome` an enum), so what is asserted is the algebra, not the layout.
    #[test]
    fn the_conclusion_mirrors_the_outcomes_rule_shape() {
        let r = rule("l1-exact-mac");
        let pairs: [(&str, Conclusion, Outcome); 3] = [
            (
                "match / merged",
                Conclusion::Match { rule: r.clone() },
                Outcome::Merged { rule: r.clone() },
            ),
            (
                "no-match / refused",
                Conclusion::NoMatch { rule: r.clone() },
                Outcome::Refused { rule: r.clone() },
            ),
            (
                "abstained / abstained",
                Conclusion::Abstained {
                    cause: IdentityAbstentionCause::Ambiguous,
                },
                Outcome::Abstained {
                    cause: IdentityAbstentionCause::Ambiguous,
                },
            ),
        ];

        for (label, conclusion, outcome) in pairs {
            let decision = Decision {
                conclusion,
                verdict_vector: Vec::new(),
                ruleset_version: RulesetVersion(1),
            };
            assert_eq!(
                decision.rule().is_some(),
                outcome.rule().is_some(),
                "{label}: Decision and Outcome must agree on whether a rule is named"
            );
        }
    }

    /// `RuleVerdict` really is a TRIPLE and `Decision` really carries a vector of them — three
    /// entries, three different verdicts, three distinct non-empty evidence lists.
    ///
    /// ⚠️ **What this does NOT prove, stated because the assertions look stronger than they are.**
    /// There is no trip: the vector is `clone()`d into the literal and compared with itself, so
    /// every assertion below reads a `pub` field with no code between it and the value the test
    /// wrote. **Measured**: replacing `RuleVerdict`'s derived `PartialEq` with a total
    /// `fn eq(&self, _: &Self) -> bool { true }` leaves the whole crate green. This test pins the
    /// SHAPE — it reds when a field is removed or retyped (story 5.4's M4: three `error[E0560]`
    /// plus one `error[E0609]`) — and it pins no BEHAVIOUR, because a `pub` field has no code to
    /// break. The behavioural version needs a producer, and one now exists: `identity::l1`'s tests
    /// assert the triple on a verdict this crate BUILT rather than on a literal — see
    /// `an_arguing_verdict_never_ships_empty_evidence`, which reds when the producer stops filling
    /// evidence. This test keeps its narrower job, and its limits are stated rather than fixed.
    #[test]
    fn the_verdict_vector_carries_the_whole_triple_in_order() {
        let vector = vec![
            RuleVerdict {
                rule: rule("l1-exact-mac"),
                verdict: Verdict::Decisive,
                evidence: vec![obs(1), obs(2)],
            },
            RuleVerdict {
                rule: rule("l2-hostname-agrees"),
                verdict: Verdict::Supports,
                evidence: vec![obs(3)],
            },
            RuleVerdict {
                rule: rule("l2-different-switch"),
                verdict: Verdict::Opposes,
                evidence: vec![obs(4), obs(5)],
            },
        ];
        let decision = Decision {
            conclusion: Conclusion::Abstained {
                cause: IdentityAbstentionCause::Ambiguous,
            },
            verdict_vector: vector.clone(),
            ruleset_version: RulesetVersion(7),
        };

        assert_eq!(
            decision.verdict_vector, vector,
            "the verdict vector is carried verbatim, in order, with every field intact"
        );
        assert_eq!(
            decision.verdict_vector[1].verdict,
            Verdict::Supports,
            "each entry keeps its own verdict"
        );
        assert_eq!(
            decision.verdict_vector[2].evidence,
            vec![obs(4), obs(5)],
            "each entry keeps its own evidence"
        );
        // Asserted rather than merely asserted ABOUT: the three evidence lists being distinct is
        // what stops this test passing on three identical ones, which is the vacuous shape the
        // story banned outright when it refused `Uuid::nil()` three times.
        assert!(
            decision.verdict_vector[0].evidence != decision.verdict_vector[1].evidence
                && decision.verdict_vector[1].evidence != decision.verdict_vector[2].evidence
                && decision.verdict_vector[0].evidence != decision.verdict_vector[2].evidence,
            "the three evidence lists are pairwise distinct, so no entry can stand in for another"
        );
        assert_eq!(
            decision.ruleset_version,
            RulesetVersion(7),
            "the ruleset version is carried alongside, not derived"
        );
    }

    // ── story 5.4b: the verdict algebra ──────────────────────────────────────────────────────

    /// D13's table, read directly and restated here as a SECOND, INDEPENDENT ORACLE.
    ///
    /// ⚠️ **Deliberate redundancy — do not collapse this into [`decide`] with a DRY pass.** It is the
    /// same protected shape as `fixtures.rs`'s `expected()`, which restates the corpus bytes rather
    /// than reading them through the code under test. A test that called `decide` to compute its own
    /// expectation would prove nothing at all.
    ///
    /// It is written from the four PRESENCE booleans, in D13's own row order, and returns the full
    /// [`Conclusion`] — **rule included**, because the rule is the half of a conclusion an arm can
    /// silently get wrong.
    ///
    /// ⚠️ The `unreachable!()` arms below are RIGHT here and would be wrong in [`decide`], and the
    /// difference is not style. This is a test: a broken premise should abort loudly. `decide` is the
    /// domain: a broken premise must not become a plausible answer — which is why it now matches on
    /// `Option<RuleId>` directly and has no such arm to fill in at all.
    fn expected_conclusion(v: &[RuleVerdict]) -> Conclusion {
        let has = |w: Verdict| v.iter().any(|rv| rv.verdict == w);
        // The lexicographically smallest rule among the verdicts carrying `w` — restated, not shared
        // with the implementation.
        let smallest = |w: Verdict| {
            v.iter()
                .filter(|rv| rv.verdict == w)
                .map(|rv| rv.rule.clone())
                .min()
        };

        if has(Verdict::Disqualifying) {
            // architecture.md:969 — "any Disqualifying → NoMatch, absolute priority".
            match smallest(Verdict::Disqualifying) {
                Some(rule) => Conclusion::NoMatch { rule },
                None => unreachable!("has(Disqualifying) implies a Disqualifying verdict exists"),
            }
        } else if has(Verdict::Decisive) && !has(Verdict::Opposes) {
            // architecture.md:970 — "a Decisive, no Opposes → Match".
            match smallest(Verdict::Decisive) {
                Some(rule) => Conclusion::Match { rule },
                None => unreachable!("has(Decisive) implies a Decisive verdict exists"),
            }
        } else if has(Verdict::Decisive) {
            // architecture.md:971 — "a Decisive, >=1 Opposes → Ambiguous" (the cloned-MAC case).
            Conclusion::Abstained {
                cause: IdentityAbstentionCause::Ambiguous,
            }
        } else if has(Verdict::Supports) {
            // architecture.md:972 (weak evidence) and :973 (Supports AND Opposes, conflict) — with
            // no Decisive, both land on Ambiguous whatever Opposes does.
            Conclusion::Abstained {
                cause: IdentityAbstentionCause::Ambiguous,
            }
        } else {
            // architecture.md:974 — "only Neutral / nothing → NoMatch (absence of proof)", AND the
            // class D13's table does not cover: >=1 Opposes with nothing arguing for the merge.
            // Guy's arbitration, 2026-07-29.
            Conclusion::Abstained {
                cause: IdentityAbstentionCause::AbsenceOfProof,
            }
        }
    }

    /// The number of input classes the totality walk must cover: one per subset of the verdicts.
    ///
    /// ⚠️ **Derived from `Verdict::all()`, never written as a literal.** A hard-coded `32` was
    /// measured to be the wrong kind of constant: adding a sixth `Verdict` variant makes
    /// `Verdict::all()` red with `error[E0004]` and forces the array to grow, but a literal `32`
    /// keeps the walk at half the real class count while the test keeps the name
    /// `decide_is_total_over_every_one_of_d13s_input_classes`. The compiler cannot catch that — this
    /// function is what does. (A `const` cannot: `Verdict::all()` is not a `const fn`, and making it
    /// one to serve a test would be the test dictating the domain's API.)
    fn input_classes() -> u32 {
        1 << Verdict::all().len()
    }

    /// A subset of the verdicts, as one `RuleVerdict` each with a distinct rule.
    ///
    /// Rule names are derived from the index rather than taken from a fixed array, so this helper
    /// cannot go stale when [`Verdict`] gains a variant. _(It used a scrambled `const NAMES: [&str;
    /// 5]` until the 2026-07-30 review, justified by a claim that could not be true: because every
    /// subset carries at most ONE verdict of any given kind, no rule-naming arm ever has two
    /// candidates, so "smallest rule" and "first in the vector" are indistinguishable here whatever
    /// the names are — as this test's own "What it does NOT cover" paragraph says four lines below.)_
    fn subset(mask: u32) -> Vec<RuleVerdict> {
        Verdict::all()
            .into_iter()
            .enumerate()
            .filter(|(i, _)| mask & (1 << i) != 0)
            .map(|(i, verdict)| RuleVerdict {
                rule: rule(&format!("r-{i}")),
                verdict,
                evidence: vec![obs(100 + i as u128)],
            })
            .collect()
    }

    /// `decide` is TOTAL over D13's table: every subset of the verdicts gets an answer, and it is the
    /// one D13's own text gives — including the empty vector, which is the `mask == 0` case.
    ///
    /// This is the "every input class, not a sample" the epic asks for: `input_classes()` presence
    /// combinations, each checked against [`expected_conclusion`], an oracle written from the
    /// architecture rather than from the code under test. The bound is derived from `Verdict::all()`
    /// so this stays "every class" when a variant is added — see `input_classes`.
    ///
    /// ⚠️ **Every mismatch is collected and reported together, not asserted per iteration.** A bare
    /// `assert_eq!` inside the loop aborts on the first one, which hides HOW MANY classes a change
    /// moved — and that count is this test's coverage. Measured: it is the difference between "M1
    /// reds" and "M1 reds on exactly 2 of 32".
    ///
    /// ⚠️ **What it does NOT cover: the tiebreak.** Each subset carries at most one verdict of any
    /// given kind, so no arm ever has two candidates and `min()` over a singleton is order-free.
    /// `the_named_rule_does_not_depend_on_arrival_order` is the only coverage that property has.
    #[test]
    fn decide_is_total_over_every_one_of_d13s_input_classes() {
        let mut mismatches = Vec::new();
        let mut exercised: Vec<Verdict> = Vec::new();

        let classes = input_classes();

        for mask in 0..classes {
            let vector = subset(mask);
            for rv in &vector {
                if !exercised.contains(&rv.verdict) {
                    exercised.push(rv.verdict);
                }
            }
            let expected = expected_conclusion(&vector);
            let decision = decide(vector.clone(), RulesetVersion(3));

            if decision.conclusion != expected {
                mismatches.push(format!(
                    "subset {mask:05b} {:?}: expected {expected:?}, got {:?}",
                    vector.iter().map(|rv| rv.verdict).collect::<Vec<_>>(),
                    decision.conclusion
                ));
            }
            assert_eq!(
                decision.verdict_vector, vector,
                "subset {mask:05b}: the verdict vector is carried through, not rebuilt"
            );
            assert_eq!(
                decision.ruleset_version,
                RulesetVersion(3),
                "subset {mask:05b}: the version is passed through verbatim"
            );
        }

        assert!(
            mismatches.is_empty(),
            "{} of {classes} input classes disagree with D13's table:\n  {}",
            mismatches.len(),
            mismatches.join("\n  ")
        );

        // ⚠️ The walk agreeing with the oracle says NOTHING about how much of the space it walked:
        // halving `classes` was MEASURED to leave all 100 core tests green, because the oracle agrees
        // on every class the walk still visits. This is what makes "every input class" falsifiable —
        // a bound too small stops exercising the highest-bit variant, and that reds here.
        for verdict in Verdict::all() {
            assert!(
                exercised.contains(&verdict),
                "{verdict:?} never appeared in any of the {classes} subsets walked, so this test \
                 does not cover the space its name claims — check that the bound is derived from \
                 Verdict::all() and that subset() maps every index"
            );
        }
    }

    /// The class D13's six rows leave uncovered — `>=1 Opposes`, no `Decisive`, no `Supports`, no
    /// `Disqualifying` — abstains for absence of proof.
    ///
    /// It is inside the 32-subset walk already; it gets its own name because it is the ONE answer no
    /// architecture line backs. It is Guy's arbitration of 2026-07-29: nothing argues FOR the merge,
    /// so there is no merge to refuse, and D13 reserves the refusal-that-names-a-rule for
    /// `Disqualifying`.
    #[test]
    fn the_input_class_d13_does_not_cover_abstains_for_absence_of_proof() {
        let vector = vec![RuleVerdict {
            rule: rule("l2-different-switch"),
            verdict: Verdict::Opposes,
            evidence: vec![obs(1)],
        }];

        let decision = decide(vector, RulesetVersion(1));

        assert_eq!(
            decision.conclusion,
            Conclusion::Abstained {
                cause: IdentityAbstentionCause::AbsenceOfProof
            },
            "an Opposes with nothing arguing for the merge abstains; it does not refuse, because a \
             refusal names a rule and D13 reserves that to Disqualifying"
        );
    }

    /// The rule a decision names does not depend on the order the verdicts arrived in — D13's *"the
    /// one written first… is not a decision, it is an accident of file order"*.
    ///
    /// ⚠️ Three verdicts of the SAME rule-naming kind, and their rule names supplied in an order that
    /// is NOT their lexicographic order. Both are load-bearing: a mixed vector would conclude
    /// `Abstained`, which names no rule, so every permutation would agree whatever the tiebreak did —
    /// the test would pass with "first in the vector" and prove nothing.
    #[test]
    fn the_named_rule_does_not_depend_on_arrival_order() {
        let mk = |name: &str| RuleVerdict {
            rule: rule(name),
            verdict: Verdict::Disqualifying,
            evidence: vec![obs(7)],
        };
        // Supplied c, a, b — lexicographic order differs from vector order.
        let names = ["c", "a", "b"];
        let permutations = [
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ];

        for order in permutations {
            let vector: Vec<RuleVerdict> = order.iter().map(|&i| mk(names[i])).collect();
            let decision = decide(vector, RulesetVersion(1));

            assert_eq!(
                decision.conclusion,
                Conclusion::NoMatch { rule: rule("a") },
                "arrival order {order:?} must not change the rule named: the smallest RuleId wins, \
                 not the first one in the vector"
            );
        }
    }

    /// A refusal names the rule that FORBADE the pair — never the smallest rule in the vector.
    ///
    /// The failing implementation this pins is a real one: taking `min()` over the WHOLE vector once
    /// and reusing it in both rule-naming arms is deterministic, order-independent, and leaves the
    /// named rule inside its own verdict vector — so it satisfies every other test here while
    /// answering `NoMatch { rule: "a" }`, a refusal naming the rule that argued FOR the merge.
    #[test]
    fn a_disqualifying_names_the_disqualifying_rule_not_the_smallest_one() {
        let vector = vec![
            RuleVerdict {
                rule: rule("a"),
                verdict: Verdict::Decisive,
                evidence: vec![obs(1)],
            },
            RuleVerdict {
                rule: rule("z"),
                verdict: Verdict::Disqualifying,
                evidence: vec![obs(2)],
            },
        ];

        let decision = decide(vector, RulesetVersion(1));

        assert_eq!(
            decision.conclusion,
            Conclusion::NoMatch { rule: rule("z") },
            "the refusal names the Disqualifying rule; 'a' argued FOR the pair and must not be the \
             rule a refusal cites"
        );
    }

    /// Everything `decide` returns is coherent: a named rule is in the vector it travels with, and
    /// an empty input can never come back as a `Match`.
    ///
    /// This is the invariant story 5.4 registered as unenforceable at the type level — `Decision`'s
    /// fields are `pub` — and it holds here **by construction**: `decide` selects the rule FROM the
    /// vector it then returns. A struct literal built elsewhere is still unconstrained; that residue
    /// is registered with story 5.9.
    #[test]
    fn a_named_rule_is_always_present_in_the_vector_it_travels_with() {
        for mask in 0..input_classes() {
            let decision = decide(subset(mask), RulesetVersion(1));
            if let Some(named) = decision.rule() {
                assert!(
                    decision.verdict_vector.iter().any(|rv| &rv.rule == named),
                    "subset {mask:05b}: conclusion names {named:?}, which is absent from its own \
                     verdict vector — 'merged, with no explanation'"
                );
            }
        }

        let empty = decide(Vec::new(), RulesetVersion(1));
        assert_eq!(
            empty.conclusion,
            Conclusion::Abstained {
                cause: IdentityAbstentionCause::AbsenceOfProof
            },
            "an empty verdict set abstains for absence of proof and can never be a Match: \
             'merged, with no explanation' is unreachable through decide()"
        );
    }

    /// A vector naming the same rule twice does not break totality — `decide` answers.
    ///
    /// `("a", Decisive)` + `("a", Opposes)` is the register's own example: ONE rule contradicting
    /// itself, fabricating D13's *"a `Decisive`, >=1 `Opposes`"* conflict row on its own. ⚠️ This
    /// pins TOTALITY, not refusal. Refusing a duplicated rule needs a PRODUCER that can emit two
    /// verdicts and does not; `identity::l1` emits exactly ONE per pair, so it cannot duplicate a
    /// rule and cannot exercise the refusal either. That half stays open, with Epic 6.
    ///
    /// _(This test also called `decide` twice and asserted the two answers matched, under the
    /// heading "answers the same way every time". Removed by the 2026-07-30 review: `decide` is a
    /// `fn` with no state and no interior mutability, so that equality is guaranteed by the language
    /// and no mutation of the body could red it. Order-independence — the property that assertion
    /// looked like it was testing — is covered by `the_named_rule_does_not_depend_on_arrival_order`.)_
    #[test]
    fn a_duplicated_rule_id_does_not_break_totality() {
        let vector = vec![
            RuleVerdict {
                rule: rule("a"),
                verdict: Verdict::Decisive,
                evidence: vec![obs(1)],
            },
            RuleVerdict {
                rule: rule("a"),
                verdict: Verdict::Opposes,
                evidence: vec![obs(2)],
            },
        ];

        let decision = decide(vector, RulesetVersion(1));

        assert_eq!(
            decision.conclusion,
            Conclusion::Abstained {
                cause: IdentityAbstentionCause::Ambiguous
            },
            "one rule contradicting itself is D13's Decisive-plus-Opposes row; decide answers rather \
             than refusing, because refusing is a claim about a producer that can emit two verdicts"
        );
    }
}
