//! Reading Rust source the way a source-scanning guard needs to: as CODE, not as prose.
//!
//! 🔴 **Two guards in this crate carried a comment stripper of their own, and story 14.2b's second
//! review defeated both with ordinary code**: a `/* … */` comment quoting the very SQL a guard looks
//! for; a string literal containing `/*` (`"/ipam/*"`), which a naive stripper takes for the start of
//! a comment and so drops the rest of the file; a SQL string spanning two lines, which a per-line
//! stripper reads as outside any string; a `'"'` char literal, which flips a quote counter. One
//! tokenizer here, with each of those traps as a test, instead of one hand-rolled stripper per guard.
//!
//! ⚠️ Test-only. `xtask`'s `sql_text.rs` closed the same class for the gates at story 6.5, and a
//! binary crate cannot depend on `xtask` — which is how the lesson failed to reach these guards.
//!
//! ⚠️ **A tokenizer, not a parser**: it knows comments, string, raw-string, byte-string and char
//! literals, and tells a char literal from a lifetime by shape. A macro that builds tokens, or code
//! the compiler never sees, is beyond it — the guards that use it are tripwires, never barriers.

/// `source` with every comment removed and every literal kept whole.
///
/// What a guard reads when the thing it looks for lives INSIDE a literal — a SQL statement.
pub(crate) fn code_only(source: &str) -> String {
    scan(source, Literals::Keep)
}

/// `source` with every comment removed and the CONTENTS of every string and char literal blanked.
///
/// What a guard reads when it looks for a PATH or an identifier, so a word inside a message or a SQL
/// string cannot be mistaken for code.
pub(crate) fn code_without_literals(source: &str) -> String {
    scan(source, Literals::Blank)
}

/// What to do with a literal's contents.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Literals {
    /// Keep them as written.
    Keep,
    /// Replace every character but a newline with a space.
    Blank,
}

/// The one pass both views share.
fn scan(source: &str, literals: Literals) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut out = String::with_capacity(source.len());
    let mut at = 0;
    while at < chars.len() {
        let here = chars[at];
        let next = chars.get(at + 1).copied();

        // A line comment, doc comments included.
        if here == '/' && next == Some('/') {
            while at < chars.len() && chars[at] != '\n' {
                at += 1;
            }
            continue;
        }

        // A block comment. Rust's nest, so the depth is counted; newlines are kept so a line number
        // read off the result still means something.
        if here == '/' && next == Some('*') {
            let mut depth = 0usize;
            while at < chars.len() {
                if chars[at] == '/' && chars.get(at + 1) == Some(&'*') {
                    depth += 1;
                    at += 2;
                } else if chars[at] == '*' && chars.get(at + 1) == Some(&'/') {
                    depth -= 1;
                    at += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    if chars[at] == '\n' {
                        out.push('\n');
                    }
                    at += 1;
                }
            }
            out.push(' ');
            continue;
        }

        // A raw string, `r"…"` or `r#"…"#` (a `br` prefix leaves its `b` as ordinary code first).
        if here == 'r' && !follows_an_identifier(&chars, at) && matches!(next, Some('"' | '#')) {
            let mut quote = at + 1;
            let mut hashes = 0usize;
            while chars.get(quote) == Some(&'#') {
                hashes += 1;
                quote += 1;
            }
            if chars.get(quote) == Some(&'"') {
                let mut end = quote + 1;
                while end < chars.len()
                    && !(chars[end] == '"'
                        && (1..=hashes).all(|h| chars.get(end + h) == Some(&'#')))
                {
                    end += 1;
                }
                out.extend(&chars[at..=quote]);
                emit(&mut out, &chars[quote + 1..end.min(chars.len())], literals);
                let close = (end + 1 + hashes).min(chars.len());
                out.extend(&chars[end.min(chars.len())..close]);
                at = close;
                continue;
            }
        }

        // A string or byte string, with its escapes.
        if here == '"' {
            let mut end = at + 1;
            while end < chars.len() && chars[end] != '"' {
                end += if chars[end] == '\\' { 2 } else { 1 };
            }
            out.push('"');
            emit(&mut out, &chars[at + 1..end.min(chars.len())], literals);
            if end < chars.len() {
                out.push('"');
            }
            at = end + 1;
            continue;
        }

        // A char literal — `'x'`, `'\n'`, `'\u{202E}'`, `'"'` — as opposed to a lifetime `'a`.
        if here == '\'' {
            let closing = if next == Some('\\') {
                chars[at + 2..]
                    .iter()
                    .position(|c| *c == '\'')
                    .map(|p| at + 2 + p)
            } else if chars.get(at + 2) == Some(&'\'') {
                Some(at + 2)
            } else {
                None
            };
            if let Some(end) = closing {
                out.push('\'');
                emit(&mut out, &chars[at + 1..end], literals);
                out.push('\'');
                at = end + 1;
                continue;
            }
        }

        out.push(here);
        at += 1;
    }
    out
}

/// Whether the character before `at` continues an identifier, so an `r` there is not a raw prefix.
fn follows_an_identifier(chars: &[char], at: usize) -> bool {
    at > 0 && (chars[at - 1].is_alphanumeric() || chars[at - 1] == '_')
}

/// A literal's contents, kept or blanked.
fn emit(out: &mut String, contents: &[char], literals: Literals) {
    for c in contents {
        out.push(match (literals, *c) {
            (Literals::Keep, c) | (Literals::Blank, c @ '\n') => c,
            (Literals::Blank, _) => ' ',
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_block_comment_quoting_what_a_guard_seeks_is_gone() {
        // The second review's G1 in its other spelling: 618 tests green over a missing lock.
        let code = code_only("/* \"x FOR UPDATE\", */\nlet sql = \"SELECT x\";");
        assert!(!code.contains("FOR UPDATE"), "{code}");
        assert!(code.contains("\"SELECT x\""), "{code}");
    }

    #[test]
    fn a_nested_block_comment_is_gone_whole() {
        let code = code_only("keep /* a /* b */ still comment */ after");
        assert!(!code.contains("still comment"), "{code}");
        assert!(code.contains("keep") && code.contains("after"), "{code}");
    }

    #[test]
    fn a_slash_star_inside_a_string_does_not_swallow_the_file() {
        // The edge layer's P3: `"/ipam/*"` blinded the plan guard to everything after it.
        let code = code_only("const P: &str = \"/ipam/*\";\nfn f() { crate::seen::x(); }");
        assert!(code.contains("crate::seen::x()"), "{code}");
        assert!(code.contains("\"/ipam/*\""), "{code}");
    }

    #[test]
    fn a_line_comment_goes_and_a_double_slash_in_a_string_stays() {
        let code = code_only("let u = \"http://x\"; // gone\nkept");
        assert!(code.contains("http://x") && code.contains("kept"), "{code}");
        assert!(!code.contains("gone"), "{code}");
    }

    #[test]
    fn a_string_spanning_two_lines_is_still_a_string() {
        // The blind layer's case: a per-line stripper reads the second line as outside any string.
        let code = code_only("q(\"INSERT \\\n // not a comment\");\n// gone");
        assert!(code.contains("// not a comment"), "{code}");
        assert!(!code.contains("gone"), "{code}");
    }

    #[test]
    fn an_escaped_quote_does_not_end_the_string() {
        let code = code_only("let s = \"a\\\" // still string\"; // gone");
        assert!(code.contains("// still string"), "{code}");
        assert!(!code.contains("gone"), "{code}");
    }

    #[test]
    fn a_quote_char_literal_does_not_open_a_string() {
        let code = code_only("let q = '\"'; // gone\nkept");
        assert!(!code.contains("gone") && code.contains("kept"), "{code}");
    }

    #[test]
    fn a_lifetime_is_not_a_char_literal() {
        let code = code_only("fn f<'a>(x: &'a str) {} // gone\nkept");
        assert!(code.contains("&'a str") && code.contains("kept"), "{code}");
        assert!(!code.contains("gone"), "{code}");
    }

    #[test]
    fn a_raw_string_keeps_its_slashes_and_its_quotes() {
        let code = code_only("let r = r#\"a \" // b\"#; // gone\nkept");
        assert!(code.contains("// b"), "{code}");
        assert!(!code.contains("gone") && code.contains("kept"), "{code}");
    }

    #[test]
    fn blanking_hides_a_word_inside_a_literal_and_keeps_the_code() {
        let words = code_without_literals("let m = \"repo::x\"; crate::seen::y(); let c = 'r';");
        assert!(!words.contains("repo"), "{words}");
        assert!(words.contains("crate::seen::y()"), "{words}");
        assert_eq!(
            words.len(),
            code_only("let m = \"repo::x\"; crate::seen::y(); let c = 'r';").len(),
            "blanking keeps every offset, so a position read off one view holds in the other"
        );
    }
}
