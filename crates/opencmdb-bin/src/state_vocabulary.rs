//! The STATE axis of the canonical glossary, in code.
//!
//! # What this module is, and what it is not
//!
//! The canonical glossary (`prd.md` and `ux-design-specification.md`, *"Terminology — one term, one
//! translation"*) carried **eleven binding rows and every one of them a GESTURE** — what the
//! operator DOES. Story 6b.6 is the first story that must name what an object **IS**, and Guy added
//! a second axis to the binding table on 2026-08-19 rather than let a story invent words beside it.
//!
//! 🔑 **The five states ARE Guy's three-case taxonomy of 2026-08-12, and the mapping is exact**:
//! *no ambiguity → the software decides* ([`Concordant`](ObjectState::Concordant),
//! [`Gap`](ObjectState::Gap), [`Conflict`](ObjectState::Conflict)); *ambiguity → the operator lifts
//! the doubt* ([`Ambiguous`](ObjectState::Ambiguous)); *unknown → the operator creates the entity*
//! ([`Undeclared`](ObjectState::Undeclared)). A sixth state fitting none of the three cases means
//! the taxonomy changed, not that this enum is short.
//!
//! ⚠️ **This is DISPLAY vocabulary and it lives in `opencmdb-bin` on purpose.** The states name
//! engine outcomes, but a glossary is about what the operator reads; putting it in the domain crate
//! would make `opencmdb-core` hold i18n keys, which D47 exists to prevent.
//!
//! # The limit of the check below, written rather than implied
//!
//! [`ObjectState::every_state_word_is_in_the_binding_glossary`] is a **tripwire against the author
//! who names a state through this enum, never a barrier against a word typed straight into a
//! template** — story 5.12's narrowing, third application. Its render-side complement
//! (`crate::example_screens`) covers the other direction and is blind to anything the author does
//! not mark. **Neither form alone is complete, and both were measured defeated at this story's
//! validation.**

/// One state an object can be in, as the operator reads it.
///
/// Every variant is a row of the binding glossary's state axis. Adding one without adding the row
/// reds [`ObjectState::every_state_word_is_in_the_binding_glossary`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ObjectState {
    /// Declared and observed agree, field by field. The software decided; nothing is asked.
    Concordant,
    /// They differ. ⚠️ **The same binding pair as the gesture axis' `gap`/« écart », not a second
    /// term** — which is why this renders through one key and no `state.ecart` was minted.
    Gap,
    /// Two observations disagree **with each other** — source against source, a different question
    /// from [`Gap`](ObjectState::Gap).
    Conflict,
    /// Several possible identities for one object. The operator lifts the doubt.
    Ambiguous,
    /// Observed, and no declared record claims it. The operator creates the entity (FR13).
    Undeclared,
}

impl ObjectState {
    /// Every state, in the glossary's order.
    ///
    /// ⚠️ **`#[cfg(test)]` because nothing in production iterates the states yet** — the inventory
    /// filters by KIND, not by state. It is gated rather than kept alive by a throw-away caller,
    /// which is the shape story 6b.3's arbitration refused when it measured that one production
    /// line silences `dead_code` and buys nothing. 🔑 The variant-to-`ALL` hole this opens is the
    /// one story 6b.3 closed for `Screen`: `every_variant_of_a_navigated_enum_is_listed_in_all`
    /// scans this file's source and now covers `ObjectState` too.
    #[cfg(test)]
    pub(crate) const ALL: [ObjectState; 5] = [
        ObjectState::Concordant,
        ObjectState::Gap,
        ObjectState::Conflict,
        ObjectState::Ambiguous,
        ObjectState::Undeclared,
    ];

    /// The i18n key this state renders through.
    ///
    /// 🔑 [`Gap`](ObjectState::Gap) reuses the key that already renders *« Écart »* on the triage
    /// queue instead of minting a `state.ecart` beside it. **One term, one meaning, one key** —
    /// UX-DR64's *"glossary uniqueness"* breaks on two keys for one word, and the story's
    /// validation raised that fork before it could be discovered at the gate.
    pub(crate) fn key(self) -> &'static str {
        match self {
            ObjectState::Concordant => "state.concordant",
            ObjectState::Gap => "triage.kind.ecart",
            ObjectState::Conflict => "state.conflict",
            ObjectState::Ambiguous => "state.ambiguous",
            ObjectState::Undeclared => "state.undeclared",
        }
    }

    /// The CSS modifier the state pill carries, so a state can be coloured without a literal.
    pub(crate) fn modifier(self) -> &'static str {
        match self {
            ObjectState::Concordant => "statepill-concordant",
            ObjectState::Gap => "statepill-gap",
            ObjectState::Conflict => "statepill-conflict",
            ObjectState::Ambiguous => "statepill-ambiguous",
            ObjectState::Undeclared => "statepill-undeclared",
        }
    }
}

/// The binding glossary's state axis, transcribed: `(EN, FR)` exactly as the table carries them.
///
/// ⚠️ A TEST ORACLE, and `#[cfg(test)]` for that reason — the same shape as `fixtures.rs`'s
/// `expected()`, which restates the corpus bytes so a guard has something independent to compare
/// against. `CLAUDE.md` protects that redundancy by name.
///
/// 🔴 **Transcribed, not derived.** It is a second, independent representation of the planning
/// document — the deliberate redundancy `CLAUDE.md` protects, and the only thing that can catch a
/// key whose translation has drifted away from the binding pair. Deriving it from the locale file
/// would make the check compare the locale file to itself.
#[cfg(test)]
pub(crate) const BINDING_STATE_AXIS: [(&str, &str); 5] = [
    ("concordant", "concordant"),
    ("gap", "écart"),
    ("conflict", "conflit"),
    ("ambiguous", "ambigu"),
    ("undeclared", "non déclaré"),
];

/// The separator after which a state string carries a QUALIFIER rather than a new term.
///
/// ⚠️ **Guy's arbitration, 2026-08-19.** The reference mock renders *"Écart · 1 champ"*,
/// *"Écart · 2 champs"* and *"Écart · présence"*: the term is *écart* and what follows qualifies it.
/// An exact-membership check would **red on the mock's own copy**, so the check matches the term
/// before this separator. Refused: seven glossary rows (the table would then describe rendering)
/// and banning the suffix (the operator loses *how many fields diverge*).
pub(crate) const QUALIFIER_SEPARATOR: &str = " · ";

/// The term a rendered state string carries — everything before [`QUALIFIER_SEPARATOR`].
pub(crate) fn term_of(rendered: &str) -> &str {
    rendered
        .split(QUALIFIER_SEPARATOR)
        .next()
        .unwrap_or(rendered)
        .trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 🔴 **Every state this enum can render is a row of the BINDING glossary, in both languages.**
    ///
    /// # Why this exists, and what it does NOT promise
    ///
    /// AC2 requires that *"every word is checked against the canonical glossary"*. Until 2026-08-19
    /// there was nothing to check against: the table's eleven rows were all gestures. Guy added the
    /// state axis; this is the check.
    ///
    /// 🔑 **It asserts BOTH directions.** A state whose word is not in the table is a word
    /// introduced rather than registered — the thing AC2 forbids. A table row no state renders is a
    /// glossary that has outrun the product, which is the failure the other direction catches and
    /// which a one-way check reads as a pass.
    ///
    /// ⚠️ **It is blind to a word typed straight into a template**, and its render-side complement
    /// is blind to a word this enum never renders. See the module doc: neither is a barrier.
    #[test]
    fn every_state_word_is_in_the_binding_glossary() {
        assert_eq!(
            ObjectState::ALL.len(),
            BINDING_STATE_AXIS.len(),
            "the enum and the transcribed table must have the same number of rows, or one of them \
             has gained a state the other has not"
        );
        for (state, (en, fr)) in ObjectState::ALL.iter().zip(BINDING_STATE_AXIS) {
            let rendered_en = rust_i18n::t!(state.key(), locale = "en").to_string();
            let rendered_fr = rust_i18n::t!(state.key(), locale = "fr").to_string();
            assert_eq!(
                term_of(&rendered_en).to_lowercase(),
                en,
                "{state:?} renders {rendered_en:?} in English and the binding glossary carries \
                 {en:?} — a state word that is not in the table is a word INTRODUCED rather than \
                 registered, which is what AC2 forbids"
            );
            assert_eq!(
                term_of(&rendered_fr).to_lowercase(),
                fr,
                "{state:?} renders {rendered_fr:?} in French and the binding glossary carries \
                 {fr:?}"
            );
        }
    }

    /// 🔑 **`Gap` renders through the key that already existed, and this measures it.**
    ///
    /// The story's validation raised the fork before it could be discovered at a gate: minting a
    /// `state.ecart` beside `triage.kind.ecart` puts **two keys behind one French word**, and
    /// UX-DR64's *"glossary uniqueness"* breaks on exactly that. The glossary settles it — `gap` is
    /// the SAME binding pair on both axes — and this asserts the settlement rather than trusting it.
    #[test]
    fn one_word_is_rendered_by_one_key() {
        let mut seen: Vec<(String, &str)> = Vec::new();
        for state in ObjectState::ALL {
            let word = rust_i18n::t!(state.key(), locale = "fr").to_string();
            if let Some((_, other)) = seen.iter().find(|(w, _)| *w == word) {
                panic!(
                    "{word:?} is rendered by two keys, {:?} and {other:?} — one term, one meaning, \
                     one key: UX-DR64's glossary uniqueness cannot hold with two",
                    state.key()
                );
            }
            seen.push((word, state.key()));
        }
        assert_eq!(seen.len(), 5, "the premise: five states were compared");
    }

    /// A qualifier after the separator is a rendering detail and must not become a second term.
    #[test]
    fn a_qualifier_is_not_a_new_term() {
        assert_eq!(term_of("Écart · 2 champs"), "Écart");
        assert_eq!(term_of("Écart · présence"), "Écart");
        assert_eq!(term_of("Concordant"), "Concordant");
        // ⚠️ A hyphen is NOT the separator: only the mock's middle dot qualifies, so a word that
        // merely contains punctuation is still compared whole.
        assert_eq!(term_of("Non déclaré"), "Non déclaré");
    }
}
