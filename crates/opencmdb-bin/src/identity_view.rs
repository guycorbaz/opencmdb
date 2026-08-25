//! The identity engine's reach, shaped for the page — and why each of its lines carries no
//! documenting gesture.
//!
//! # Why this is a module of its own
//!
//! 🔴 **`page.rs` reached the `file-size` gate's 2000-line ceiling in story 6.4** — at 2033, with
//! the gate naming it. `CLAUDE.md`'s rule is *split, not grown*, and this is the split: the
//! identity section is the one part of the page with its own types, its own `identity.*` locale
//! namespace, its own producer (`repo::count_engine_reach`) and its own doctrine, so it comes out
//! whole rather than by line count. ⚠️ **Its TESTS stay in `page.rs`**, where the render helpers
//! they share with the rest of the page live; the ceiling counts the lines before the first
//! `#[cfg(test)]`, so a test module here would have bought nothing.
//!
//! # What the section is FOR, in one sentence
//!
//! It reports the identity engine's REACH — what the product saw and could not place — and every
//! line of it is a statement about the SOURCE, never a debt the operator owes.

use crate::repo::EngineReachRow;

/// One cause line of the identity engine's reach — *"N sightings, because …"*.
#[derive(Clone)]
pub(crate) struct IdentityCauseRow {
    /// Why the engine did not place them, in the operator's language.
    pub(crate) cause: String,
    /// How many sightings — never how many devices; the unit is stated on screen.
    pub(crate) count: i64,
    /// 🔴 **Why this line carries no documenting gesture** (story 6.4, AC2).
    ///
    /// The gesture belongs to `undeclared` — *observed, and no declared record claims it* — and
    /// this is not that population. ⚠️ **Saying nothing is not the same as saying no**: a count
    /// with no control beside it reads as a feature someone forgot, so the line says which
    /// answer it is waiting for instead.
    ///
    /// 🔑 It is a `String` and not an `Option<String>`, which is the same refusal story 6.4 made
    /// for the gesture's own route: an abstention line with no sentence would be a fourth state
    /// nothing needs, and the template would have to carry the choice.
    pub(crate) why: String,
}

/// One outcome the engine SETTLED without placing — `NoMatch`, and any token no variant names.
///
/// 🔑 **A type of its own rather than a reuse of [`IdentityCauseRow`], and the difference is
/// AC2's own word.** These lines are Guy's case ONE — *the software decided* — so nothing about
/// them awaits the operator and there is no gesture whose absence needs explaining. Sharing the
/// struct would have meant either an empty `why` (a state the abstention lines must never reach)
/// or copy for a question this half of the section does not raise.
#[derive(Clone)]
pub(crate) struct IdentitySettledRow {
    /// What the engine settled, in the operator's language.
    pub(crate) cause: String,
    /// How many sightings.
    pub(crate) count: i64,
}

/// The identity engine's reach, shaped for rendering.
///
/// # The unit is SIGHTINGS, and that is a decision rather than a caption
///
/// Every scan mints fresh `obs_id`s and the identity pass supersedes no engine link across passes,
/// so one machine seen ten times is ten rows. The number therefore counts SIGHTINGS, not devices,
/// and the surface says so on both sides of the pair.
///
/// 🔑 Naming the unit truthfully is what keeps the number from reading as a backlog: a figure that
/// rises because the product looked many times is the radar's range, not the operator's debt. ⚠️ It
/// does not make the UX bans MET — *"no growing counter"* and *"after six months of inaction it
/// reads the same number"* are still open, owned by Epic 6, and registered. A true unit does not
/// stop a number growing.
///
/// ⚠️ **The unit is TEMPORARY.** Epic 6 gives the population an identity, at which point *sighting*
/// stops being the honest word and the locale keys change with it. That rename is a scheduled
/// consequence, not a correction of a mistake.
///
/// # 🔴 Three cases, and they are the OPERATOR's three, not the engine's
///
/// Guy's taxonomy (2026-08-12), which is what decides where each outcome goes:
///
/// | case | what the engine wrote | who acts | the gesture |
/// |---|---|---|---|
/// | **no ambiguity** | `Match`, and also `NoMatch` | the software | none — it decided |
/// | **ambiguity** | `Abstained { Ambiguous }` | the operator lifts the doubt | choose among the candidates and their evidence (FR16) |
/// | **unknown** | `Abstained { AbsenceOfProof }` | *see below* | none here |
///
/// 🔑 **`NoMatch` is case ONE**, which is why it is neither placed nor listed among what awaits the
/// operator: *a rule FORBADE the pair* is a decision, not an absence. An earlier draft folded it
/// into `placed` through a bare `else`, so a refused placement was reported as a placement and the
/// page rendered *"every sighting was placed"* over it — found independently by all three review
/// layers.
///
/// 🔴 **THE THIRD ROW USED TO SAY *"the operator creates the entity — the documenting gesture"*,
/// AND STORY 6.4 IS WHAT REFUTED IT.** `AbsenceOfProof` is an identity verdict about whether two
/// sightings could be JOINED; FR13's population is `undeclared` — *observed, and no declared record
/// claims it* — which the PRD's binding glossary gives a row of its own. ⚠️ A sighting can be
/// `AbsenceOfProof` **and already fully declared**, so offering to create an entity here is the one
/// thing FR13's invariant exists to prevent. The gesture lives on the triage queue's `Nouveau` row,
/// where that population already is (Guy, 2026-08-24), and every line of this section now SAYS why
/// it carries none. **The answer to an identity abstention is a better SOURCE — Epic 11 — not a
/// record the operator writes.**
///
/// _(This table travelled unchanged out of `page.rs` when story 6.4 split this module out, three
/// hunks below its own refutation. A pure move is where a false sentence is least likely to be
/// re-read; the blind review layer found it from the diff alone.)_
///
/// ⚠️ **The ambiguity gesture does not exist** — it needs candidates nothing produces (Epic 6) —
/// and this view announces it no more than it announces the other: **announcing an absent gesture
/// is a promise; this section stays descriptive until the gesture is there** (Guy, 2026-08-12).
#[derive(Clone)]
pub(crate) struct IdentityView {
    /// Sightings the engine placed on an interface — case one, `Match`.
    pub(crate) placed: i64,
    /// Sightings it could not place — case two and case three together.
    pub(crate) not_placed: i64,
    /// Why, one line per cause — never one line per failure (FR16b).
    ///
    /// ⚠️ **The one-line-per-cause property belongs to the CALLER**, not to this type: it holds
    /// because `count_engine_reach` groups by cause in SQL. Feed this view two rows carrying the
    /// same cause and it renders two identical lines. Stated rather than enforced, because the only
    /// producer is the grouped read.
    pub(crate) causes: Vec<IdentityCauseRow>,
    /// Outcomes the engine SETTLED without placing — `NoMatch`, and any token no variant names.
    ///
    /// Rendered only when non-empty, and today it always is empty: `resolve` cannot produce a
    /// `NoMatch` (`placement_decision` only judges pairs inside one `join` group, which share their
    /// key by construction), and `repo::cause_token`'s exhaustive `match` is what writes the rest.
    /// It is counted and labelled rather than folded anywhere, on `identity_cause_line`'s
    /// precedent: the tolerant reader for the CAUSE token had a silent twin on the OUTCOME token,
    /// and this is that twin, made explicit.
    pub(crate) settled: Vec<IdentitySettledRow>,
    /// Has the engine seen anything at all? Distinguishes *"nothing yet"* from *"nothing unplaced"*.
    pub(crate) has_any: bool,
}

/// An identity abstention cause as the operator reads it: its label, **and why it carries no
/// documenting gesture**.
///
/// # 🔴 The two halves travel together because they answer one question
///
/// Story 6.4's AC2: every cause that is not `undeclared` carries no documenting gesture *and the
/// surface says why rather than staying silent*. Returning the label alone would let a caller
/// render a count with no control and no sentence — which is what the section did before, and
/// which reads as a feature someone forgot rather than as an answer.
///
/// ⚠️ **The tolerant arm gets a sentence too, and the type is what forced it.** The story budgeted
/// two keys, one per named cause; a third was needed the moment the pair became the return value,
/// because this function is total and its last arm renders a line like any other. *A cause the
/// product cannot name is still a cause the operator is looking at.*
///
/// # 🔴 This function is TOTAL, and refusing to fail is the whole point
///
/// `identity_link.abstention_cause` is a plain `VARCHAR(32)` with no `CHECK`, so the database can
/// hold a token no variant of [`opencmdb_core::identity::cascade::IdentityAbstentionCause`] names —
/// measured, an invented token inserts cleanly. And `page.rs`'s handlers turn any error into a `500`
/// for the WHOLE page, so a reader that failed here would take the gap display down with it, for one
/// unfamiliar row.
///
/// So an unrecognised token is **labelled and carried**, never dropped and never fatal. It is still
/// COUNTED by [`build_identity_view`]: a total that silently shrank would be the counter lying by
/// omission, which is worse than an unfamiliar word on the page.
///
/// ⚠️ **A `match` on tokens cannot be exhaustive over the enum**, and that is exactly why this is a
/// tripwire rather than a barrier: adding a variant breaks the WRITER ([`crate::repo::cause_token`],
/// an exhaustive `match` with no `_` arm) and breaks nothing here. A variant added with the minimal
/// repair therefore persists a token this function does not know — and the page renders it as
/// unrecognised instead of dying. That is the designed behaviour, not a gap in it.
///
/// The stronger closure — a DDL `CHECK` on the token domain — was weighed and refused for story
/// 5.14b: it moves the failure from the display to the WRITE, so a future variant would break the
/// identity pass rather than show an unfamiliar label. It is registered as the real closure.
pub(crate) fn identity_cause_line(token: &str) -> (String, String) {
    use rust_i18n::t;
    match token {
        "absence_of_proof" => (
            t!("identity.cause.absence_of_proof").to_string(),
            t!("identity.no_gesture.absence_of_proof").to_string(),
        ),
        "ambiguous" => (
            t!("identity.cause.ambiguous").to_string(),
            t!("identity.no_gesture.ambiguous").to_string(),
        ),
        other => (
            t!("identity.cause.unrecognised", token = other).to_string(),
            t!("identity.no_gesture.unrecognised").to_string(),
        ),
    }
}

/// PURE: shape the database's grouped reach rows into a renderable view.
///
/// Abstained rows are the not-placed population and each contributes one cause line; everything else
/// is placed. **One line per cause, never one line per failure** — FR16b's *"96 multi-interface
/// devices is not 96 failures, it is ONE question"*.
///
/// ⚠️ **An abstained row whose cause is NULL cannot exist**: `identity_link_rule_xor_cause` makes the
/// cause non-NULL exactly when `outcome = 'abstained'`. This function is nonetheless total over the
/// type, and the empty token then falls to the unrecognised label. That is totality, **not a guard** —
/// no test can red it, and it is not claimed as covering anything.
pub(crate) fn build_identity_view(rows: Vec<EngineReachRow>) -> IdentityView {
    use rust_i18n::t;

    let mut placed = 0i64;
    let mut not_placed = 0i64;
    let mut causes: Vec<IdentityCauseRow> = Vec::new();
    let mut settled: Vec<IdentitySettledRow> = Vec::new();
    let mut settled_count = 0i64;
    for row in rows {
        // 🔴 An explicit arm per outcome, and NO bare `else`. The `else` an earlier draft used sent
        // `no_match` — *a rule FORBADE this pair* — into `placed`, i.e. reported a refusal as a
        // success. `identity_link_outcome` admits exactly these three tokens; anything else can only
        // arrive from a store written by something other than `repo::outcome_token`, and it is
        // carried rather than folded, exactly as an unknown CAUSE token is.
        match row.outcome.as_str() {
            "match" => placed += row.count,
            "abstained" => {
                not_placed += row.count;
                let (cause, why) = identity_cause_line(row.cause.as_deref().unwrap_or_default());
                causes.push(IdentityCauseRow {
                    cause,
                    count: row.count,
                    why,
                });
            }
            "no_match" => {
                settled_count += row.count;
                settled.push(IdentitySettledRow {
                    cause: t!("identity.outcome.no_match").to_string(),
                    count: row.count,
                });
            }
            other => {
                settled_count += row.count;
                settled.push(IdentitySettledRow {
                    cause: t!("identity.outcome.unrecognised", token = other).to_string(),
                    count: row.count,
                });
            }
        }
    }
    IdentityView {
        placed,
        not_placed,
        causes,
        settled,
        has_any: placed + not_placed + settled_count > 0,
    }
}
