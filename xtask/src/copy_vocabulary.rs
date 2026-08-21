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
    let f = findings_naive(&yaml);
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
