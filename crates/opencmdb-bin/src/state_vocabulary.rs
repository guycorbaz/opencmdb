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

// ── The GESTURE axis, and the resolver-side half of AC2 (story 6b.10) ─────────────────────────

/// The binding glossary's GESTURE axis, transcribed: `(EN, FR)` exactly as the table carries them.
///
/// ⚠️ **From `ux-design-specification.md:1341-1351`, and the source is NAMED because the two
/// binding tables disagree.** The UX spec carries **eleven** gesture rows; `prd.md:993-1002`
/// carries **ten** — `attach`/« rattacher » is missing from the PRD's. Both documents call
/// themselves binding. This transcribes the superset and the divergence is registered rather than
/// silently resolved by picking one: reconciling two planning documents is a planning act.
///
/// 🔴 **Transcribed, not derived** — the same reason [`BINDING_STATE_AXIS`] gives: deriving it
/// from `app.yml` would make the check compare the locale file to itself, and the whole point is
/// that the copy must answer to a document written elsewhere. `CLAUDE.md` protects this
/// redundancy by name.
#[cfg(test)]
pub(crate) const BINDING_GESTURE_AXIS: [(&str, &str); 11] = [
    ("observed", "observé"),
    ("declared", "déclaré"),
    ("gap", "écart"),
    ("reconcile", "réconcilier"),
    // 🔴 The pair that is not a shared root, and the one this story exists for. The glossary's
    // own note: *"a pair needs ONE MEANING, not one root"*. `merge` is retired IN ENGLISH by
    // name; « Merger » is the fixed French translation of `document` and carries no such claim.
    ("document", "merger"),
    ("accept the gap", "accepter l'écart"),
    ("snooze", "mettre en veille"),
    ("attach", "rattacher"),
    ("exclude", "exclure"),
    ("triage", "triage"),
    ("source", "source"),
];

/// Retired terms, per locale, as they may appear in RESOLVED interface copy.
///
/// 🔴 **This restates `xtask`'s `copy_vocabulary::RETIRED`, and the duplication is FORCED rather
/// than sloppy**: `xtask` is a dependency of nobody (D56) and `opencmdb-bin` must not depend on
/// it, so the two carriers cannot share a constant even in principle. What keeps them in step is
/// [`the_two_carriers_agree_on_what_is_retired`], which reads the `xtask` source.
///
/// ⚠️ « Merger » is absent from the French row and must stay absent: it is BINDING.
#[cfg(test)]
const RETIRED_IN_COPY: [(&str, &[&str]); 2] = [
    (
        "en",
        &["merge", "merged", "merging", "drift", "ignore", "revert"],
    ),
    ("fr", &["ignorer", "ignore", "ignoré", "ignorée"]),
];

#[cfg(test)]
mod gesture_axis_tests {
    use super::*;

    /// Every `gesture.*` key whose gesture the glossary NAMES, and the row it answers to.
    ///
    /// 🔑 **The exemptions are enumerated, so a new gesture cannot ship without a decision.** A
    /// key added to `app.yml` under `gesture.` and listed in neither table below reds this test,
    /// which is the moment to ask whether the glossary needs a row — the question story 6b.7
    /// answered by refusing to extend the table *"prematurely, not wrongly"*.
    const GLOSSARY_BACKED: [(&str, &str); 5] = [
        ("gesture.document", "document"),
        ("gesture.accept_gap", "accept the gap"),
        ("gesture.snooze", "snooze"),
        ("gesture.attach", "attach"),
        ("gesture.exclude", "exclude"),
    ];

    /// `gesture.*` keys that are NOT glossary gestures, each with why.
    ///
    /// ⚠️ **Three of these are gestures with no binding row, and that is registered, not fixed**:
    /// extending a binding table is a planning act and Guy's (story 6b.7's precedent, where it was
    /// refused as *premature, not wrong*). `baseline` is a fourth of the same class, already owned
    /// by Epic 9.
    const NOT_A_GLOSSARY_GESTURE: [(&str, &str); 7] = [
        (
            "gesture.badge",
            "the *not yet* marker, not a gesture (story 6b.4b)",
        ),
        (
            "gesture.not_built",
            "the sentence under the bar (story 6b.4b)",
        ),
        (
            "gesture.badge_off",
            "the *built and switched off* marker, not a gesture (story 6.4's code review)",
        ),
        (
            "gesture.not_enabled",
            "its sentence, which names the switch rather than a term (story 6.4's code review)",
        ),
        (
            "gesture.resolve",
            "a gesture with NO row in either binding table — Epic 6 owns FR16's ranked candidates",
        ),
        (
            "gesture.check_now",
            "no row; an on-demand poll, FR6's scheduler (registered by story 6b.9)",
        ),
        (
            "gesture.export_log",
            "no row; Epic 13, which owns the incident axis (registered by story 6b.9)",
        ),
    ];

    /// The `gesture.*` key names the locale file carries, read from the FILE.
    fn gesture_keys() -> Vec<String> {
        // 🔴 Through a real parse, never a line scan. Until story 6b.10's code review this read
        // `line.strip_suffix(':')`, the exact parse arbitration 3(d) abolished for missing 7 of
        // 12 legal YAML shapes — inside this story, one file over from where it was abolished.
        // A nested `gesture:\n  document:` yields no key to a line scan, so its rendered value
        // was never linted by this carrier at all.
        crate::screens::locale_keys::key_paths()
            .into_iter()
            .filter(|name| name.starts_with("gesture."))
            .collect()
    }

    /// AC2 — **every gesture the glossary names is rendered with the glossary's own words, in both
    /// columns.**
    ///
    /// 🔴 This is the check that would have caught the defect story 6b.10 shipped against: the
    /// English column read `"Merge"`, the one word the binding tables retire by name, on
    /// `/triage`'s primary control — while the French « Merger » was right all along. The
    /// state axis has had this check since story 6b.6; the gesture axis had **none**.
    ///
    /// 🔑 **Through `t!` in both locales, never `set_locale`**, which is process-wide and makes
    /// the suite order-dependent (`page.rs` records that hazard twice).
    #[test]
    fn every_glossary_gesture_is_rendered_with_the_glossary_words() {
        for (key, gesture) in GLOSSARY_BACKED {
            let (en, fr) = BINDING_GESTURE_AXIS
                .iter()
                .find(|(term, _)| *term == gesture)
                .unwrap_or_else(|| panic!("{gesture:?} is transcribed from the binding table"));
            let rendered_en = rust_i18n::t!(key, locale = "en").to_lowercase();
            let rendered_fr = rust_i18n::t!(key, locale = "fr").to_lowercase();
            // 🔴 A `trim_end_matches(" the gap")` stood on BOTH operands until story 6b.10's
            // code review. It was measured a NO-OP — `gesture.accept_gap` renders "Accept the
            // gap", the glossary phrase exactly — so it weakened nothing today and would have
            // let a future `"Accept"` compare equal to `"accept the gap"`, which is the one
            // divergence this test exists to catch on the one multi-word row.
            assert_eq!(
                rendered_en.as_str(),
                *en,
                "{key} renders {rendered_en:?} in English and the binding glossary carries \
                 {en:?} — one term, one translation"
            );
            assert_eq!(
                rendered_fr, *fr,
                "{key} renders {rendered_fr:?} in French and the binding glossary carries {fr:?}"
            );
        }
    }

    /// AC2 — **no `gesture.*` key escapes the question.**
    ///
    /// Every key is either backed by a glossary row or listed as deliberately unbacked with its
    /// reason. A new one is neither, and reds here.
    #[test]
    fn every_gesture_key_is_either_glossary_backed_or_knowingly_not() {
        let keys = gesture_keys();
        assert_eq!(
            keys.len(),
            GLOSSARY_BACKED.len() + NOT_A_GLOSSARY_GESTURE.len(),
            "the locale file carries {keys:?}; every one must be accounted for above, and a new \
             gesture is the moment to ask whether the binding table needs a row"
        );
        for key in &keys {
            let backed = GLOSSARY_BACKED.iter().any(|(k, _)| k == key);
            let excused = NOT_A_GLOSSARY_GESTURE.iter().any(|(k, _)| k == key);
            assert!(backed ^ excused, "{key} is in neither list, or in both");
        }
    }

    /// AC2 — 🔴 **CARRIER 2: no RESOLVED value carries a retired term, in either language.**
    ///
    /// # Why this is not the `copy-vocabulary` gate over again
    ///
    /// The gate reads the FILE; this reads the RESOLVER, and story 6b.10's validation measured
    /// that the difference is not academic. Its first design claimed a YAML block scalar was what
    /// separated the two carriers — and with a real parse the gate reds on that too. **What
    /// actually separates them is the FALLBACK**: `rust-i18n` falls back to `en`, so a key with
    /// no `fr` half renders its ENGLISH value on a French screen. A retired French term reaching
    /// the operator that way exists **nowhere in the French column** for a file-reading gate to
    /// find, and only a check on the resolved string can see it.
    ///
    /// 🔑 *A guard that reads the source measures what was written, never what was served* —
    /// story 6b.4b's sentence, arrived at here from the locale axis.
    #[test]
    fn no_resolved_value_carries_a_retired_term() {
        // 🔴 Through a real parse — see `gesture_keys` above for why a line scan was wrong here.
        let keys = crate::screens::locale_keys::key_paths();
        // 🔑 The premise is DERIVED, not a floor: this carrier must read EVERY key the resolver
        // can resolve, and `every_key_carries_both_locales` counts the same population from the
        // same walker. A number here would tolerate the silent loss of everything above it.
        assert!(
            keys.len() > 200,
            "the premise: the whole locale file is walked ({} keys) — a scan that matched \
             nothing would assert nothing",
            keys.len()
        );
        // 🔴 **Default-ignorable characters are removed before matching.** Story 6b.10's code
        // review measured `"Nothing to mer<U+200B>ge yet."` walking past BOTH carriers with nine
        // gates and 720/720 green, the word intact in the shipped binary and intact on screen.
        // ⚠️ The set is an ENUMERATION and cannot claim the completeness of a property (story
        // 5.12) — a tripwire against the character a copy-paste carries, never a barrier. It
        // mirrors `xtask::copy_vocabulary::strip_invisible`; the frontier forbids sharing it,
        // and `the_two_carriers_agree_on_what_is_retired` is what keeps the pair honest.
        let visible = |text: &str| -> String {
            text.chars()
                .filter(|c| {
                    !matches!(u32::from(*c),
                        0x00AD
                        | 0x200B..=0x200F
                        | 0x2028..=0x202E
                        | 0x2060..=0x206F
                        | 0xFE00..=0xFE0F
                        | 0xFEFF
                        | 0xE0000..=0xE01EF
                    )
                })
                .collect()
        };
        let boundary = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric());
        for key in &keys {
            for (locale, retired) in RETIRED_IN_COPY {
                let rendered =
                    visible(&rust_i18n::t!(key.as_str(), locale = locale).to_lowercase());
                for term in retired {
                    let hit = rendered.match_indices(term).any(|(at, _)| {
                        boundary(rendered[..at].chars().next_back())
                            && boundary(rendered[at + term.len()..].chars().next())
                    });
                    assert!(
                        !hit,
                        "{key} RENDERS {rendered:?} in {locale}, which carries the retired term \
                         {term:?} — and the operator reads what is rendered, not what is written"
                    );
                }
            }
        }
    }

    /// 🔴 **AC2's glossary-uniqueness half, on the GESTURE axis — one word, one key.**
    ///
    /// The state axis has had this since story 6b.6 (`one_word_is_rendered_by_one_key` over
    /// `ObjectState::ALL`); the gesture axis had **none**, and story 6b.10's code review measured
    /// what that cost: setting `gesture.resolve`'s French half to « Merger » left 720/720 tests
    /// green and the `copy-vocabulary` gate green, with **two keys rendering one binding French
    /// word** — exactly what UX-DR64's *"glossary uniqueness"* forbids and what `prd.md:988`
    /// calls one word carrying two meanings.
    ///
    /// 🔑 Over EVERY `gesture.*` key and not merely the glossary-backed five: the defect's own
    /// specimen, `gesture.resolve`, has no binding row, so a check restricted to the glossary
    /// would have been placed exactly where that defect cannot occur.
    #[test]
    fn one_gesture_word_is_rendered_by_one_key() {
        for locale in ["en", "fr"] {
            let mut seen: std::collections::BTreeMap<String, String> =
                std::collections::BTreeMap::new();
            for key in gesture_keys() {
                let rendered = rust_i18n::t!(key.as_str(), locale = locale).to_lowercase();
                if let Some(first) = seen.get(&rendered) {
                    panic!(
                        "{locale}: `{first}` and `{key}` both render {rendered:?} — one term, \
                         one translation (UX-DR64). Two keys sharing a word make the interface \
                         say one thing about two different gestures."
                    );
                }
                seen.insert(rendered, key);
            }
            assert!(
                seen.len() > 5,
                "{locale}: the premise — the gesture axis was enumerated ({} keys)",
                seen.len()
            );
        }
    }

    /// 🔑 **The two carriers must not drift apart, and nothing else can hold them together.**
    ///
    /// `xtask` is a dependency of nobody (D56), so `opencmdb-bin` cannot import its denylist even
    /// in principle: the duplication is forced by the frontier, not chosen. This reads the `xtask`
    /// source so that a term added on one side and forgotten on the other is loud.
    ///
    /// ⚠️ The two lists are not required to be EQUAL — the gate lints key names as well and
    /// carries plural forms this side has no use for. What is required is that nothing this side
    /// forbids is absent over there, which is the direction that matters: the gate is the one
    /// that runs in CI on its own.
    ///
    /// 🔴 **Its first version searched the WHOLE gate file and reddened at once, on a legitimate
    /// line**: `"merger"` appears there as a NEGATIVE CONTROL in the word-boundary test, beside
    /// `emerged` and `submerge`. *An unbounded needle cannot tell a denylist entry from a mention
    /// of one* — story 6b.6's *"an oracle that counts a word counts every word that contains
    /// it"*, met on the first run of the guard written to keep two denylists in step. It now
    /// reads the `RETIRED` constant's own text and nothing else.
    #[test]
    fn the_two_carriers_agree_on_what_is_retired() {
        let source = include_str!("../../../xtask/src/copy_vocabulary.rs");
        let start = source
            .find("const RETIRED:")
            .expect("xtask's copy-vocabulary gate declares its denylist as `RETIRED`");
        let gate = &source[start
            ..start
                + source[start..]
                    .find("];")
                    .expect("the denylist is an array literal")];
        for (locale, terms) in RETIRED_IN_COPY {
            for term in terms {
                assert!(
                    gate.contains(&format!("\"{term}\"")),
                    "`{term}` is retired in {locale} here and absent from xtask's \
                     `copy_vocabulary::RETIRED` — the frontier forbids sharing the constant, so \
                     this test is the only thing keeping the two carriers in step"
                );
            }
        }
        // 🔴 **Bounded to the FRENCH column, and the unbounded form was measured wrong.**
        // Until story 6b.10's code review this searched the whole constant, so it forbade
        // `merger` in EVERY locale — and it reddened the moment Guy's arbitration 5 added the
        // noun to the ENGLISH column, where the French binding says nothing. It is the defect
        // this test's own doc narrates, met on a second axis: *an unbounded needle cannot tell
        // one locale's column from another's* any more than it can tell an entry from a mention.
        let fr_column = gate
            .find("(\"fr\",")
            .map(|at| &gate[at..])
            .expect("the gate declares a French column");
        let fr_column = &fr_column[..fr_column.find(']').expect("the column is an array")];
        assert!(
            !fr_column.contains("\"merger\""),
            "« Merger » is the BINDING French translation of `document` and must never reach the \
             FRENCH denylist — this is the row someone tidying the two lists together would add. \
             (English `merger` IS retired, by Guy's arbitration of 2026-08-22, and that is a \
             different column.) Column read: {fr_column}"
        );
    }
}
