//! Gate 9 (story 6b.10): per-locale forbidden-word lint over the translation file.
//!
//! GAP-HUNT PROTOTYPE — naive parse on purpose, to measure what it misses.

use std::path::Path;

use anyhow::{Context, Result};

/// (locale, forbidden word) — a word retired in ONE language and possibly binding in another.
pub(crate) const LOCALE_RETIRED: &[(&str, &str)] = &[("en", "merge")];

/// Retired words forbidden in a KEY NAME, whatever the locale.
pub(crate) const KEY_RETIRED: &[&str] = &["merge"];

/// A finding: the 1-based line and what was wrong.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Finding {
    /// 1-based line number in the locale file.
    pub(crate) line: usize,
    /// Human-readable cause.
    pub(crate) what: String,
}

/// Naive line-based parse: `  en: "…"` / `  fr: "…"`, key names are `name:` at column 0.
pub(crate) fn findings_naive(yaml: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for (i, line) in yaml.lines().enumerate() {
        let n = i + 1;
        let t = line.trim_end();
        if let Some(rest) = t.strip_prefix("  ") {
            if let Some((loc, val)) = rest.split_once(": ") {
                let val = val.trim().trim_matches('"');
                for (l, word) in LOCALE_RETIRED {
                    if *l == loc && contains_word(&val.to_lowercase(), word) {
                        out.push(Finding {
                            line: n,
                            what: format!("{loc} value {val:?} carries retired word '{word}'"),
                        });
                    }
                }
            }
        } else if let Some(name) = t.strip_suffix(':')
            && !name.starts_with(' ')
            && !name.starts_with('#')
        {
            for word in KEY_RETIRED {
                if contains_word(&name.to_lowercase(), word) {
                    out.push(Finding {
                        line: n,
                        what: format!("key name {name:?} carries retired word '{word}'"),
                    });
                }
            }
        }
    }
    out
}

/// Whole-word containment on ASCII word boundaries.
pub(crate) fn contains_word(hay: &str, needle: &str) -> bool {
    let b = hay.as_bytes();
    let nb = needle.as_bytes();
    if nb.is_empty() || b.len() < nb.len() {
        return false;
    }
    let is_w = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    for i in 0..=(b.len() - nb.len()) {
        if &b[i..i + nb.len()] == nb {
            let before = i == 0 || !is_w(b[i - 1]);
            let after = i + nb.len() == b.len() || !is_w(b[i + nb.len()]);
            if before && after {
                return true;
            }
        }
    }
    false
}

/// The gate entry point.
///
/// # Errors
/// Returns an error if the locale file cannot be read.
pub(crate) fn gate_copy_vocabulary(root: &Path) -> Result<(bool, String)> {
    let rel = "crates/opencmdb-bin/locales/app.yml";
    let p = root.join(rel);
    let yaml = std::fs::read_to_string(&p).with_context(|| format!("reading {rel}"))?;
    let f = findings_yaml(&yaml)?;
    if f.is_empty() {
        Ok((true, "locale file clean of retired vocabulary".into()))
    } else {
        Ok((
            false,
            format!(
                "{} finding(s):\n      {}",
                f.len(),
                f.iter()
                    .map(|x| format!("{rel}:{}: {}", x.line, x.what))
                    .collect::<Vec<_>>()
                    .join("\n      ")
            ),
        ))
    }
}


/// A located (key path, locale, value) triple, from a REAL YAML parse.
#[derive(Debug, Clone)]
pub(crate) struct Entry {
    /// Dotted key path as `rust-i18n` resolves it.
    pub(crate) path: String,
    /// The 1-based line the scalar starts on.
    pub(crate) line: usize,
    /// The scalar's value.
    pub(crate) value: String,
}

/// A `MarkedEventReceiver` that flattens the document into dotted paths with line marks.
#[derive(Default)]
struct Flatten {
    stack: Vec<String>,
    /// `true` while the next scalar is a mapping KEY.
    expect_key: Vec<bool>,
    pending_key: Option<String>,
    entries: Vec<Entry>,
    keys: Vec<(String, usize)>,
}

impl yaml_rust2::parser::MarkedEventReceiver for Flatten {
    fn on_event(&mut self, ev: yaml_rust2::Event, mark: yaml_rust2::scanner::Marker) {
        use yaml_rust2::Event as E;
        match ev {
            E::MappingStart(..) => {
                if let Some(k) = self.pending_key.take() {
                    self.stack.push(k);
                }
                self.expect_key.push(true);
            }
            E::MappingEnd => {
                self.expect_key.pop();
                self.stack.pop();
            }
            E::SequenceStart(..) => {
                if let Some(k) = self.pending_key.take() {
                    self.stack.push(k);
                }
            }
            E::SequenceEnd => {
                self.stack.pop();
            }
            E::Scalar(val, ..) => {
                let in_key_position = *self.expect_key.last().unwrap_or(&false);
                if in_key_position {
                    let mut path = self.stack.clone();
                    path.push(val.clone());
                    self.keys.push((path.join("."), mark.line()));
                    self.pending_key = Some(val);
                    if let Some(f) = self.expect_key.last_mut() {
                        *f = false;
                    }
                } else {
                    if let Some(k) = self.pending_key.take() {
                        let mut path = self.stack.clone();
                        path.push(k);
                        self.entries.push(Entry {
                            path: path.join("."),
                            line: mark.line(),
                            value: val,
                        });
                    }
                    if let Some(f) = self.expect_key.last_mut() {
                        *f = true;
                    }
                }
            }
            _ => {}
        }
    }
}

/// Flatten a locale document into its located scalar entries and its located key paths.
///
/// # Errors
/// Returns an error if the document is not valid YAML.
pub(crate) fn flatten(yaml: &str) -> Result<(Vec<Entry>, Vec<(String, usize)>)> {
    let mut sink = Flatten::default();
    let mut parser = yaml_rust2::parser::Parser::new_from_str(yaml);
    parser
        .load(&mut sink, true)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok((sink.entries, sink.keys))
}

/// The real parse: walk the document, not its lines.
pub(crate) fn findings_yaml(yaml: &str) -> Result<Vec<Finding>> {
    let (entries, keys) = flatten(yaml)?;
    let mut out = Vec::new();
    for e in &entries {
        // The last segment is the locale; everything before it is the i18n key.
        let Some((key, locale)) = e.path.rsplit_once('.') else {
            continue;
        };
        for (l, word) in LOCALE_RETIRED {
            if *l == locale && contains_word(&e.value.to_lowercase(), word) {
                out.push(Finding {
                    line: e.line,
                    what: format!("{key} / {locale} value {:?} carries '{word}'", e.value),
                });
            }
        }
    }
    for (path, line) in &keys {
        // Skip the leaf locale segments themselves.
        if path.ends_with(".en") || path.ends_with(".fr") {
            continue;
        }
        for word in KEY_RETIRED {
            if contains_word(&path.to_lowercase(), word) {
                out.push(Finding {
                    line: *line,
                    what: format!("key name {path:?} carries retired word '{word}'"),
                });
            }
        }
    }
    out.sort_by_key(|f| f.line);
    out.dedup_by(|a, b| a.line == b.line && a.what == b.what);
    Ok(out)
}

#[cfg(test)]
mod probes {
    use super::*;

    /// (name, yaml) — every one carries the retired word `merge` in its EN value or key name.
    const SHAPES: &[(&str, &str)] = &[
        ("plain double-quoted", "gesture.x:\n  en: \"Merge\"\n  fr: \"Merger\"\n"),
        ("single-quoted", "gesture.x:\n  en: 'Merge'\n  fr: 'Merger'\n"),
        ("unquoted plain", "gesture.x:\n  en: Merge\n  fr: Merger\n"),
        ("block scalar |", "gesture.x:\n  en: |\n    Merge the rows\n  fr: \"Merger\"\n"),
        ("block scalar >", "gesture.x:\n  en: >\n    Merge the rows\n  fr: \"Merger\"\n"),
        ("block scalar |-", "gesture.x:\n  en: |-\n    Merge\n  fr: \"Merger\"\n"),
        ("escaped quote", "gesture.x:\n  en: \"say \\\"Merge\\\" now\"\n  fr: \"Merger\"\n"),
        ("flow mapping", "gesture.x: {en: Merge, fr: Merger}\n"),
        ("deeper indent", "gesture.x:\n    en: \"Merge\"\n    fr: \"Merger\"\n"),
        ("plain continuation", "gesture.x:\n  en: A long\n    Merge sentence\n  fr: \"Merger\"\n"),
        ("quoted key name", "\"gesture.merge\":\n  en: \"Document\"\n  fr: \"Merger\"\n"),
        ("nested key name", "gesture:\n  merge:\n    en: \"Document\"\n    fr: \"Merger\"\n"),
    ];

    #[test]
    fn which_shapes_the_naive_parse_sees() {
        let mut missed = Vec::new();
        for (name, yaml) in SHAPES {
            let f = findings_naive(yaml);
            println!(
                "{:<26} -> {}",
                name,
                if f.is_empty() {
                    "MISSED (reads GREEN)".to_string()
                } else {
                    format!("caught line {}", f[0].line)
                }
            );
            if f.is_empty() {
                missed.push(*name);
            }
        }
        println!("\nMISSED {} of {}: {:?}", missed.len(), SHAPES.len(), missed);
    }

    #[test]
    fn which_shapes_the_yaml_parse_sees() {
        let mut missed = Vec::new();
        for (name, yaml) in SHAPES {
            let f = findings_yaml(yaml).expect("valid yaml");
            println!(
                "{:<26} -> {}",
                name,
                if f.is_empty() {
                    "MISSED (reads GREEN)".to_string()
                } else {
                    format!("caught line {}", f[0].line)
                }
            );
            if f.is_empty() {
                missed.push(*name);
            }
        }
        println!("\nYAML PARSE MISSED {} of {}: {:?}", missed.len(), SHAPES.len(), missed);
    }

    /// The FR half must NOT be flagged — « Merger » is binding.
    #[test]
    fn the_french_binding_word_is_not_flagged() {
        let f = findings_naive("gesture.document:\n  en: \"Document\"\n  fr: \"Merger\"\n");
        assert!(f.is_empty(), "the FR binding word must not red: {f:?}");
    }
}
