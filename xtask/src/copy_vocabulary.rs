//! Gate 9 — **`copy-vocabulary`**: the binding glossary's retired terms may not survive in the
//! interface copy, in either language (D65 volet C, story 6b.10, AC2).
//!
//! # The seam this closes, and how long it stood open
//!
//! `crates/opencmdb-bin/locales/app.yml` has carried the sentence *"the D65 vocabulary/
//! forbidden-word gate **can later** lint these strings"* in its own header since story 3.8, in
//! July. Story 3.8's own record notes the seam twice more. Nothing read the file: [`crate`]'s
//! `gate_vocabulary` walks seven planning documents (volet B) and every `.rs` under `crates/`
//! (volet A), and it `continue`s on any extension that is not `rs`.
//!
//! And volet A could not have carried this even if it had read the file, because its denylist is
//! **locale-blind**. The term this gate was written for is `merge`, which the binding tables
//! retire **in English by name** — *"it names the forbidden operation; the founding pillar is
//! linked, never merged"* — while the FRENCH « Merger » is the fixed translation of `document`
//! and carries no such claim. One word, forbidden in one column and binding in the other: a flat
//! list cannot say that.
//!
//! ⚠️ **When this gate was first run on the committed tree it reddened, twice, on the product's
//! primary control**: `gesture.merge` rendered `"Merge"` in English on `/triage`'s main button.
//! The French had been right all along. That is story 6b.6's *"Drift"* defect one story later on
//! a different term, and Guy's arbitration 1 of 2026-08-21 renamed the key to `gesture.document`
//! and the value to `Document` before this gate could land.
//!
//! # Scope: this file, and deliberately not `crates/`
//!
//! The glossary's first column is *"EN (docs, API, **code**)"*, which reads like a mandate to
//! lint every identifier in the workspace. **It is not taken, and the refusal is measured rather
//! than preferred**: `merge` has ~310 legitimate whole-word occurrences under `crates/`, among
//! them `Expectation::MustMerge` and 87 `must-merge` tokens spread across **fourteen
//! sha256-locked fixture files** which cannot be renamed without reddening the `fixtures` gate.
//! `must-merge` is a trap-corpus pole: the corpus has to be able to SAY that two observations
//! must be merged, precisely because the product does not merge them.
//!
//! 🔑 So the rule is narrower than the column and truer to it: **what the glossary forbids is a
//! retired term as the name of the operator's gesture, and the locale file is where gesture names
//! live.** ⚠️ Stated as a limit rather than a property: this gate cannot stop a future story
//! naming a route `/merge` or a handler `merge_entity`. That closure is volet A's denylist, and
//! taking it means deciding about `MustMerge` first.
//!
//! # Why a real YAML parse, and why that is not a trade-off
//!
//! 🔴 **A denylist that fails to parse a value reads GREEN.** The failure direction is the whole
//! argument: for a *detector*, a missed value is a false negative — a hole — never a false alarm.
//! Story 6b.10's validation built both parses and measured a naive line split missing **seven of
//! twelve legal YAML shapes** (`|`, `>`, `|-` block scalars, a flow mapping, a four-space indent,
//! a plain multi-line continuation, a nested key), while two shapes the story had *suspected*
//! — single quotes and escaped quotes — were caught by both.
//!
//! `yaml-rust2` was already in `Cargo.lock` through `config`, so removing that whole class cost
//! **one line in the lockfile and no crate in the supply chain**. There was nothing to trade.
//!
//! ⚠️ **A real parser has its own silent failure mode**, and the validation hit it: a hand-rolled
//! event walker dropped a key from the parse and the gate then reported ✅ over a live violation.
//! [`Entries::completeness`] is the answer — the parse must account for every scalar the document
//! contains, or the gate refuses to report green at all.

use std::path::Path;

use anyhow::{Context, Result};
use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser};
use yaml_rust2::scanner::Marker;

/// The one file this gate reads, relative to the workspace root.
pub(crate) const LOCALE_FILE: &str = "crates/opencmdb-bin/locales/app.yml";

/// A retired term, and the locale column it is retired IN.
///
/// 🔴 **The locale dimension is the point.** `merge` is retired in English and its French
/// counterpart « Merger » is the *binding* translation of `document`; a flat denylist would
/// either miss the English defect or red on the correct French. `KEY_NAMES` is the pseudo-locale
/// for the identifier column — the glossary's *"EN (docs, API, code)"* applied to key paths.
///
/// ⚠️ **`drift` is here and `merge` is not, in the FRENCH column** — the asymmetry is the
/// glossary's, not this table's. And `drift` is forbidden **only in this file**: under `crates/`
/// the English word is used legitimately throughout ("the two spellings cannot drift", "an
/// anti-drift"), so a workspace-wide ban would be wrong. In interface copy it can only mean the
/// synonym for `gap` that story 6b.6 retired.
const RETIRED: &[(&str, &[&str])] = &[
    (
        KEY_NAMES,
        &[
            "merge",
            "merges",
            "merged",
            "merging",
            "drift",
            "ignore",
            "revert",
            "accept-as-declared",
            "accept_as_declared",
        ],
    ),
    (
        "en",
        &[
            "merge",
            "merges",
            "merged",
            "merging",
            "drift",
            "drifts",
            "ignore",
            "ignores",
            "revert",
            "reverts",
            "accept-as-declared",
        ],
    ),
    // « Merger » is BINDING here and must never join this list. French `ignore`/`ignorer` is the
    // retired verb — the same retirement as the English one, in the language it was retired for.
    ("fr", &["ignorer", "ignore", "ignorez", "ignorée", "ignoré"]),
];

/// The pseudo-locale under which a KEY PATH is linted rather than a translated value.
const KEY_NAMES: &str = "<key>";

/// One scalar the locale file carries: a translated value, or the key path that holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The dotted key path, e.g. `gesture.document`.
    pub(crate) key: String,
    /// The locale column, or [`KEY_NAMES`] when this entry IS the key path.
    pub(crate) locale: String,
    /// The text linted — the value, or the key path itself.
    pub(crate) text: String,
    /// The 1-based line the scalar sits on, so a finding says WHERE.
    pub(crate) line: usize,
}

/// What a parse of the locale file yielded, with enough bookkeeping to prove it saw everything.
#[derive(Debug, Default)]
pub(crate) struct Entries {
    /// Every translated value, plus one [`KEY_NAMES`] entry per VALUE-BEARING key path.
    pub(crate) entries: Vec<Entry>,
    /// Every scalar the parser emitted, keys and values alike — the completeness counter.
    scalars: usize,
    /// How many mapping keys the parser read — locale keys (`en`, `fr`) included.
    keys: usize,
    /// How many value scalars it read.
    values: usize,
}

impl Entries {
    /// Whether the parse accounted for every scalar in the document.
    ///
    /// 🔴 **A gate that goes blind must not be able to report green**, and this is the only thing
    /// standing between that and a silent hole. Story 6b.10's validation wrote an event walker
    /// that dropped a key, and the gate then reported ✅ over a live violation of the very rule it
    /// exists to enforce — the denylist failure mode arrived at from the inside.
    ///
    /// Every scalar is either a mapping key or a mapping value, and this parse records one entry
    /// per key plus one per value, so the two counts must add up exactly.
    pub(crate) fn completeness(&self) -> std::result::Result<(), String> {
        let values = self.values;
        if self.keys + values == self.scalars {
            Ok(())
        } else {
            Err(format!(
                "the parse accounted for {} key(s) and {values} value(s) against {} scalar(s) in \
                 the document — it went blind somewhere, and a denylist that cannot see a value \
                 reports GREEN over it",
                self.keys, self.scalars
            ))
        }
    }
}

/// Collects `(key path, locale, value, line)` from the YAML event stream.
///
/// # Why an event receiver rather than [`yaml_rust2::YamlLoader`]
///
/// The loader yields the structure and throws the MARKS away, and story 5.12's finding is that
/// **a pinned boolean proves THAT a gate fires and never WHERE**: its `AUTHORSHIP_PROBES` pinned
/// booleans while the offset→line map was broken, and the corpus reported the right line by
/// accident. A finding that cannot name a line sends the reader to a 958-line file.
struct Collector {
    /// One frame per open mapping: whether the next scalar is a key, and the key it belongs to.
    stack: Vec<Frame>,
    /// The dotted path of enclosing mapping keys.
    path: Vec<String>,
    /// What has been collected so far.
    out: Entries,
    /// Where each key path was DECLARED, so a key-name finding names the declaration and not the
    /// first translation under it.
    ///
    /// ⚠️ **Only value-bearing key paths are linted as key names, and the first version of this
    /// gate got that wrong in a way worth recording**: it emitted one key entry per mapping key,
    /// which counts `en` and `fr` too, and its green message then read *"862 key(s)"* over a file
    /// carrying **284**. Nothing was mis-detected — but a gate that reports a false count in the
    /// sentence announcing its own success is the defect this story exists to remove, one level
    /// up. *A number in a gate's message is a claim like any other.*
    key_lines: std::collections::BTreeMap<String, usize>,
}

/// One open mapping in the YAML document.
struct Frame {
    /// `true` when the next scalar in this mapping is a KEY, `false` when it is a value.
    expecting_key: bool,
    /// The key the next value belongs to.
    key: String,
}

impl Collector {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            path: Vec::new(),
            out: Entries::default(),
            key_lines: std::collections::BTreeMap::new(),
        }
    }

    /// The dotted path of `key` inside the currently open mappings.
    fn path_of(&self, key: &str) -> String {
        if self.path.is_empty() {
            key.to_string()
        } else {
            format!("{}.{key}", self.path.join("."))
        }
    }
}

impl MarkedEventReceiver for Collector {
    fn on_event(&mut self, event: Event, mark: Marker) {
        match event {
            Event::MappingStart(..) => {
                // 🔴 A nested mapping is the VALUE of the key just read, so that key joins the
                // path AND the parent goes back to expecting a key. Forgetting the second half is
                // exactly the defect story 6b.10's validation hit: the parent's flag stayed on
                // `value`, the next real key was swallowed, and the gate went blind in silence.
                if let Some(frame) = self.stack.last_mut() {
                    let key = std::mem::take(&mut frame.key);
                    frame.expecting_key = true;
                    self.path.push(key);
                }
                self.stack.push(Frame {
                    expecting_key: true,
                    key: String::new(),
                });
            }
            Event::MappingEnd => {
                self.stack.pop();
                self.path.pop();
            }
            // A sequence cannot appear in this file's shape, but it must not corrupt the stack if
            // one is ever added: its scalars are values, and it opens no key context.
            Event::SequenceStart(..) => {
                if let Some(frame) = self.stack.last_mut() {
                    frame.expecting_key = true;
                }
            }
            Event::Scalar(value, ..) => {
                self.out.scalars += 1;
                let Some(frame) = self.stack.last_mut() else {
                    return;
                };
                if frame.expecting_key {
                    frame.key = value.clone();
                    frame.expecting_key = false;
                    self.out.keys += 1;
                    let key = self.path_of(&value);
                    self.key_lines.entry(key).or_insert_with(|| mark.line());
                } else {
                    frame.expecting_key = true;
                    self.out.values += 1;
                    // A top-level scalar pair (`_version: 2`) has no locale: its key IS the path,
                    // and an empty locale matches no denylist row, so it is counted and not
                    // linted. Anything nested is `<key path>: { <locale>: <text> }`.
                    let (key, locale) = if self.path.is_empty() {
                        (frame.key.clone(), String::new())
                    } else {
                        (self.path.join("."), frame.key.clone())
                    };
                    self.out.entries.push(Entry {
                        key,
                        locale,
                        text: value,
                        line: mark.line(),
                    });
                }
            }
            _ => {}
        }
    }
}

/// Parse the locale file into linted entries.
///
/// # Errors
///
/// Returns an error when the YAML does not parse. A file this gate cannot read is a RED, never a
/// skip: the alternative is a denylist that reports green because it gave up.
pub(crate) fn entries_of(yaml: &str) -> Result<Entries> {
    let mut collector = Collector::new();
    let mut parser = Parser::new_from_str(yaml);
    parser
        .load(&mut collector, false)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("parsing the locale file")?;
    // 🔑 One key-name entry per VALUE-BEARING key path — never one per mapping key, which would
    // lint `en` and `fr` as identifiers and inflate the count by a factor of three.
    let mut paths: Vec<String> = collector
        .out
        .entries
        .iter()
        .map(|entry| entry.key.clone())
        .collect();
    paths.sort();
    paths.dedup();
    for key in paths {
        let line = collector.key_lines.get(&key).copied().unwrap_or(0);
        collector.out.entries.push(Entry {
            text: key.clone(),
            key,
            locale: KEY_NAMES.to_string(),
            line,
        });
    }
    Ok(collector.out)
}

/// Whether `haystack` contains `needle` as a WORD.
///
/// 🔴 **The boundary set is not `gate_vocabulary`'s and the difference was measured.**
/// `contains_word` there treats `_` as a word character, which is right for a Rust identifier and
/// wrong here, where `_` and `.` are this file's own separators: under that matcher
/// `gesture.merge` reds and **`gesture.merge_all` passes**, on a file where 107 of 284 keys carry
/// an underscore. A retired term is a word wherever it is not glued to a letter or a digit.
fn contains_word(haystack: &str, needle: &str) -> bool {
    let haystack = haystack.to_lowercase();
    let needle = needle.to_lowercase();
    let boundary = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric());
    haystack.match_indices(&needle).any(|(at, _)| {
        boundary(haystack[..at].chars().next_back())
            && boundary(haystack[at + needle.len()..].chars().next())
    })
}

/// One retired term found in the interface copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Finding {
    /// The key the term was found under.
    pub(crate) key: String,
    /// The locale column, or [`KEY_NAMES`].
    pub(crate) locale: String,
    /// The retired term itself.
    pub(crate) term: String,
    /// The 1-based line, so the message says where to go.
    pub(crate) line: usize,
}

/// Every retired term the interface copy carries, located.
pub(crate) fn findings(entries: &Entries) -> Vec<Finding> {
    let mut found = Vec::new();
    for entry in &entries.entries {
        for (locale, terms) in RETIRED {
            if *locale != entry.locale {
                continue;
            }
            for term in *terms {
                if contains_word(&entry.text, term) {
                    found.push(Finding {
                        key: entry.key.clone(),
                        locale: entry.locale.clone(),
                        term: (*term).to_string(),
                        line: entry.line,
                    });
                }
            }
        }
    }
    found
}

/// Run the gate.
///
/// # Errors
///
/// Returns an error when the locale file cannot be read or parsed — never a green skip.
pub(crate) fn gate_copy_vocabulary(root: &Path) -> Result<(bool, String)> {
    let path = root.join(LOCALE_FILE);
    let yaml = std::fs::read_to_string(&path).with_context(|| {
        format!("reading {LOCALE_FILE} — the gate has nothing to lint without it")
    })?;
    let entries = entries_of(&yaml)?;
    if let Err(why) = entries.completeness() {
        return Ok((false, format!("{LOCALE_FILE}: {why}")));
    }
    // 🔴 **Zero entries is a REFUSAL, not a pass**, and it is categorically different from a
    // small number rather than a floor that will rot. A locale file that parses but yields
    // nothing — truncated to a bare scalar, emptied, replaced — would otherwise be reported as
    // *"no retired term in 0 key(s)"*, which is a denylist announcing success over a file it
    // could not see. *A gate with nothing to lint has not passed; it has gone blind.*
    if entries.entries.is_empty() {
        return Ok((
            false,
            format!(
                "{LOCALE_FILE}: parsed to NOTHING — a gate with nothing to lint has not passed"
            ),
        ));
    }
    let found = findings(&entries);
    if found.is_empty() {
        let keys = entries
            .entries
            .iter()
            .filter(|e| e.locale == KEY_NAMES)
            .count();
        return Ok((
            true,
            format!(
                "no retired term in {keys} key(s) and {} translated value(s)",
                entries.entries.len() - keys
            ),
        ));
    }
    let lines: Vec<String> = found
        .iter()
        .map(|f| {
            format!(
                "{LOCALE_FILE}:{}: `{}` carries the retired term '{}' in {}",
                f.line, f.key, f.term, f.locale
            )
        })
        .collect();
    Ok((
        false,
        format!(
            "{} finding(s):\n      {}",
            found.len(),
            lines.join("\n      ")
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A private directory per test — `main.rs`'s own reason: a shared constant path races
    /// between concurrent runs and leaves a stale corpus behind when an assertion fails.
    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("opencmdb-copy-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("crates/opencmdb-bin/locales")).expect("scratch dir");
        dir
    }

    /// Run the WHOLE gate over a planted locale file, exactly as `cargo xtask ci` runs it.
    fn gate_over(tag: &str, yaml: &str) -> (bool, String) {
        let root = scratch(tag);
        std::fs::write(root.join(LOCALE_FILE), yaml).expect("planting the locale file");
        gate_copy_vocabulary(&root).expect("the gate reads its own planted file")
    }

    /// The probe corpus: a locale document, and the LINE the gate must name — or `None` for a
    /// document it must pass.
    ///
    /// 🔴 **Located verdicts, in BOTH directions, and story 5.12 is why.** Its `AUTHORSHIP_PROBES`
    /// pinned a BOOLEAN — *does it red* — while the gate's offset→line map was broken and reported
    /// line 0 for a write on line 2. Neither the probes nor ten mutations could catch it: *a
    /// pinned boolean proves THAT a gate fires and never WHERE.* Those 29 verdicts became located
    /// ones, and this corpus is born that way.
    ///
    /// ⚠️ **And the GREEN rows are half the corpus, not padding.** A denylist that silently widens
    /// is a denylist whose stated limits have gone stale; the French rows below are what stop
    /// someone "tidying" « Merger » onto the list and breaking the binding translation.
    const PROBES: &[(&str, &str, Option<usize>)] = &[
        // ── must RED ──────────────────────────────────────────────────────────────────────
        (
            "e01 the committed defect: `merge` in the English column",
            "gesture.merge:\n  en: \"Merge\"\n  fr: \"Merger\"\n",
            Some(2),
        ),
        (
            "e02 the key NAME alone, values clean",
            "gesture.merge:\n  en: \"Document\"\n  fr: \"Merger\"\n",
            Some(1),
        ),
        (
            "e03 snake_case around the term — `gate_vocabulary`'s matcher passes this",
            "gesture.merge_all:\n  en: \"Document\"\n  fr: \"Merger\"\n",
            Some(1),
        ),
        (
            "e04 a BLOCK SCALAR, which a naive line split walks past",
            "triage.hint:\n  en: |\n    You may merge the two records.\n  fr: \"Fusion\"\n",
            Some(3),
        ),
        (
            "e05 a folded block scalar",
            "triage.hint:\n  en: >\n    Nothing to ignore here.\n  fr: \"Rien\"\n",
            Some(3),
        ),
        (
            "e06 a FLOW MAPPING on one line",
            "triage.hint: {en: \"Merge them\", fr: \"Fusionner\"}\n",
            Some(1),
        ),
        (
            "e07 a NESTED key path — it resolves identically and defeats a line-shape parse",
            "gesture:\n  merge:\n    en: \"Document\"\n    fr: \"Merger\"\n",
            Some(2),
        ),
        (
            "e08 four-space indent",
            "triage.hint:\n    en: \"Please revert it\"\n    fr: \"Annulez\"\n",
            Some(2),
        ),
        (
            "e09 a plain multi-line continuation",
            "triage.hint:\n  en: You may safely\n    ignore this row\n  fr: \"Rien\"\n",
            Some(2),
        ),
        (
            "e10 single quotes",
            "triage.hint:\n  en: 'Drift detected'\n  fr: \"Écart\"\n",
            Some(2),
        ),
        (
            "e11 an escaped quote inside the value",
            "triage.hint:\n  en: \"the \\\"merge\\\" button\"\n  fr: \"le bouton\"\n",
            Some(2),
        ),
        (
            "e12 the FRENCH retired verb, in the French column",
            "gesture.skip:\n  en: \"Exclude\"\n  fr: \"Ignorer\"\n",
            Some(3),
        ),
        (
            "e13 `drift`, the synonym story 6b.6 retired for the product's core term",
            "triage.kind.ecart:\n  en: \"Drift\"\n  fr: \"Écart\"\n",
            Some(2),
        ),
        // ── must stay GREEN, and each row says what it protects ───────────────────────────
        (
            "g01 « Merger » is the BINDING French translation of `document` — never a finding",
            "gesture.document:\n  en: \"Document\"\n  fr: \"Merger\"\n",
            None,
        ),
        (
            "g02 the live gesture vocabulary, untouched",
            "gesture.exclude:\n  en: \"Exclude\"\n  fr: \"Exclure\"\n",
            None,
        ),
        (
            "g03 `emerged` GLUES the term to a letter — a word, not a substring",
            "triage.note:\n  en: \"A pattern emerged\"\n  fr: \"Un motif\"\n",
            None,
        ),
        (
            "g04 an English word inside a FRENCH value is not linted under the English column",
            "triage.note:\n  en: \"Nothing\"\n  fr: \"Le mode merge est absent\"\n",
            None,
        ),
        (
            "g05 the top-level `_version` pair has no locale and is not copy",
            "_version: 2\ntriage.note:\n  en: \"Fine\"\n  fr: \"Bien\"\n",
            None,
        ),
    ];

    /// Every probe gets the verdict it names, and a red one names its LINE.
    #[test]
    fn every_probe_gets_its_located_verdict() {
        for (index, (label, yaml, expected)) in PROBES.iter().enumerate() {
            let found = findings(&entries_of(yaml).unwrap_or_else(|e| {
                panic!("{label}: the gate must PARSE every legal shape, and it refused: {e}")
            }));
            match expected {
                None => assert!(
                    found.is_empty(),
                    "{label}: expected GREEN, got {found:?} — a denylist that widens silently is \
                     a denylist whose stated limits have gone stale"
                ),
                Some(line) => {
                    assert!(
                        !found.is_empty(),
                        "{label}: expected a finding and the gate reported none — for a DENYLIST \
                         a missed value is a hole, never a false alarm"
                    );
                    assert!(
                        found.iter().any(|f| f.line == *line),
                        "{label}: the gate found {found:?} but none of them names line {line} — a \
                         pinned boolean proves THAT a gate fires and never WHERE (story 5.12)"
                    );
                }
            }
            // The index is read so a row cannot be silently dropped from the middle.
            assert!(index < PROBES.len());
        }
        let red = PROBES.iter().filter(|p| p.2.is_some()).count();
        let green = PROBES.iter().filter(|p| p.2.is_none()).count();
        assert_eq!(
            (red, green),
            (13, 5),
            "both poles are exercised, and the count is here so a deleted row is loud"
        );
    }

    /// 🔴 **The gate END TO END, not its helper — and story 5.12 is why this test exists.**
    ///
    /// Its review found that *"the whole body of `gate_declared_authorship` was deletable with the
    /// xtask suite green"*, because every test attacked the finding helper directly while nothing
    /// ran the gate. The probes above call [`findings`]; this one calls
    /// [`gate_copy_vocabulary`] over a planted tree, so the wiring between them is covered too.
    #[test]
    fn the_gate_itself_reds_and_says_where() {
        let (ok, message) = gate_over("red", "gesture.merge:\n  en: \"Merge\"\n  fr: \"Merger\"\n");
        assert!(!ok, "the gate must RED: {message}");
        assert!(
            message.contains(LOCALE_FILE),
            "the file is named: {message}"
        );
        assert!(message.contains(":2:"), "the LINE is named: {message}");
        assert!(message.contains("merge"), "the term is named: {message}");

        let (ok, message) = gate_over(
            "green",
            "gesture.document:\n  en: \"Document\"\n  fr: \"Merger\"\n",
        );
        assert!(ok, "and it passes correct copy: {message}");
        assert!(
            message.contains("1 key(s)") && message.contains("2 translated value(s)"),
            "the green message counts what it actually read: {message}"
        );
    }

    /// 🔴 **The completeness refusal THROUGH THE GATE — and this test exists because the
    /// mutation that should have produced it came back GREEN.**
    ///
    /// Story 6b.10's own T4 pass ran M18: *delete the `completeness()` call from
    /// [`gate_copy_vocabulary`]*. It reddened **nothing**. The sibling test below asserts on
    /// `completeness()` directly and is entirely CORRECT about what it tests; what nothing tested
    /// was that the gate ever calls it. That is story 5.12's finding verbatim — *"the whole body
    /// of `gate_declared_authorship` was deletable with the xtask suite green, because every test
    /// attacked the helper"* — reproduced inside the gate written with that lesson in hand, and
    /// caught only by running the mutation.
    ///
    /// 🔑 A document that PARSES and yields nothing is the reachable case: a bare top-level
    /// scalar. Without both refusals the gate reports *"no retired term in 0 key(s)"* — success,
    /// announced over a file it could not read.
    #[test]
    fn a_locale_file_the_gate_cannot_see_into_is_a_red_through_the_gate() {
        let (ok, message) = gate_over("blind-scalar", "just a bare scalar\n");
        assert!(!ok, "a document with no entries must RED: {message}");
        assert!(
            message.contains("nothing to lint") || message.contains("went blind"),
            "and it must say WHY, not merely fail: {message}"
        );

        let (ok, message) = gate_over("blind-empty", "");
        assert!(!ok, "an EMPTY locale file must RED too: {message}");

        // The control: the same gate over a document it CAN see is green, so the two reds above
        // are about blindness and not about the gate refusing everything.
        let (ok, _) = gate_over("blind-control", "a.b:\n  en: \"x\"\n  fr: \"y\"\n");
        assert!(ok, "the control passes, so the reds above mean something");
    }

    /// ⚠️ A file the gate cannot read or parse is a RED, never a green skip.
    #[test]
    fn an_unreadable_or_unparseable_locale_file_is_not_a_pass() {
        let missing = gate_copy_vocabulary(&scratch("missing"));
        assert!(
            missing.is_err(),
            "a gate with nothing to lint must not report green"
        );
        assert!(entries_of("gesture.merge:\n  en: \"a\"\n fr: \"b\"\n").is_err());
    }

    /// 🔴 **The completeness assertion, and it is what stops the gate going blind in silence.**
    ///
    /// Story 6b.10's validation wrote an event walker that dropped a key from the parse; the gate
    /// then reported ✅ over a live violation. This asserts the counter is load-bearing rather than
    /// decorative: a parse that loses a scalar must be refused, not reported.
    #[test]
    fn a_parse_that_loses_a_scalar_refuses_to_report_green() {
        let honest = entries_of("a:\n  en: \"x\"\n  fr: \"y\"\n").expect("parses");
        assert!(honest.completeness().is_ok());
        let mut blinded = entries_of("a:\n  en: \"x\"\n  fr: \"y\"\n").expect("parses");
        blinded.values -= 1;
        let refusal = blinded
            .completeness()
            .expect_err("a lost scalar is refused");
        assert!(refusal.contains("went blind"), "{refusal}");
    }

    /// The word matcher's boundary set is this FILE's, not a Rust identifier's.
    ///
    /// 🔴 Measured at story 6b.10's validation: `gate_vocabulary`'s `contains_word` counts `_` as
    /// a word character — right for an identifier, wrong here — so `gesture.merge` reds and
    /// **`gesture.merge_all` passes**, on a file where 107 of 284 keys carry an underscore.
    #[test]
    fn the_separators_of_this_file_are_word_boundaries() {
        for glued in [
            "gesture.merge",
            "gesture.merge_all",
            "merge-all",
            "a merge b",
        ] {
            assert!(contains_word(glued, "merge"), "{glued} carries the word");
        }
        for apart in ["emerged", "submerge", "merger", "mergeable"] {
            assert!(
                !contains_word(apart, "merge"),
                "{apart} glues it to a letter — a word, not a substring"
            );
        }
    }

    /// 🔑 **The per-locale asymmetry, asserted rather than described.** One word, forbidden in one
    /// column and BINDING in the other — the property no flat denylist can express, and the whole
    /// reason this gate is not a row in `gate_vocabulary`.
    #[test]
    fn one_word_is_forbidden_in_english_and_binding_in_french() {
        let both =
            entries_of("gesture.document:\n  en: \"Merge\"\n  fr: \"Merger\"\n").expect("parses");
        let found = findings(&both);
        assert_eq!(found.len(), 1, "exactly one column is at fault: {found:?}");
        assert_eq!(found[0].locale, "en");
        assert_eq!(found[0].line, 2, "the ENGLISH line, not the French one");
    }
}
