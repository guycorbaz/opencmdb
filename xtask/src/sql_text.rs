//! The SQL-text reader the three SQL gates share — normalisation, comment stripping, and the
//! statement bounds each gate asks its questions inside.
//!
//! 🔴 **It lives in its own file since story 6.5, and the reason is a measurement rather than
//! tidiness.** `xtask/src/main.rs` reached **exactly 2000** code lines when the tenth gate was
//! registered — the `file-size` ceiling `CLAUDE.md` sets, whose rule is *"a file approaching the
//! ceiling is split into modules, not grown"*. These helpers are what three gates share, so they
//! are the coherent piece to lift out.
//!
//! 🔴 **And they had a defect worth the move.** [`strip_comments`] tracked only the single quote as
//! a string delimiter, so a `/*` INSIDE a double-quoted Rust string opened a block comment nothing
//! closed — and **every line after it in that file went invisible to `authorship`,
//! `observed-immutable` and `entity-id-immutable` alike**. The live specimen was
//! `diagnostic.rs`'s `"/assets/*"`, an ordinary route pattern. Story 6.5's code review measured it
//! with a whole-tree sweep: one violation appended to every walked file, **45 planted, 44 named,
//! one file blind**. After the fix, 45 named and none blind.
//!
//! ⚠️ **One helper stayed behind**: `contains_word`, which is not about SQL text and is used by
//! gates that never touch it.

use crate::is_token_char;

/// Where a block comment stands when one line ends and the next begins.
///
/// Block comments cross lines, so this cannot be a per-line decision — the review measured
/// `/* housekeeping */ UPDATE declared_attribute …` passing the gate untouched.
#[derive(Default)]
struct CommentState {
    /// Inside an ordinary `/* … */`, whose body is not code.
    in_plain: bool,
    /// Nesting depth inside MariaDB's executable `/*! … */`, whose body IS code.
    exec_depth: usize,
    /// Inside a Rust raw string, holding its `#` count — `Some(0)` for `r"…"`.
    raw_hashes: Option<usize>,
}

/// Stands in for a `"` that is INSIDE a Rust string literal rather than delimiting one.
///
/// 🔴 [`statement_before`] and [`statement_after_of`] bound a statement at `;` or `"`, and that `"`
/// bound is load-bearing — without it a finding spans two literals and the gate invents phantoms.
/// But a `"` can also sit *inside* the SQL, and then the bound truncates the statement instead of
/// ending it. Measured: `… FROM declared_attribute WHERE note = \"n\" AND actor_id = ?` passed the
/// gate while the same read WITHOUT the quoted literal reddened — the read half loses every
/// predicate after the quote, and predicates are where provenance columns live.
///
/// This is the ordinary Rust spelling, not a trick: `\"` is how anyone writes a quote inside a
/// query. So the two forms Rust actually allows — the escape and the raw string — are rewritten to
/// this sentinel, which is neither a bound nor a token character.
const QUOTE_IN_LITERAL: char = '\u{1}';

/// One line with its comments removed, carrying [`CommentState`] to the next.
///
/// 🔑 The two block forms are handled the OPPOSITE way round, and that is the point:
/// `/*!50000 INSERT … */` is MariaDB's executable comment — the server runs its body — so the
/// markers are dropped and the body KEPT, while an ordinary `/* … */` is dropped whole. Reversing
/// them would either invent a write out of a commented-out one or go blind to a real one; the
/// review measured the gate doing the second.
///
/// `'` is tracked so that a `--` or a `/*` sitting inside a SQL literal stays data.
fn strip_comments(line: &str, sql: bool, state: &mut CommentState) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut in_literal = false;
    // 🔴 Story 6.5's code review: a `/*` INSIDE a double-quoted Rust string opened a block comment
    // that nothing closed, and **every line after it in that file went invisible to all three SQL
    // gates**. The live specimen was `diagnostic.rs:1141`'s `"/assets/*"` — an ordinary route
    // pattern, not an adversary's probe: the `*` is consumed by the opener, so the following `"`
    // cannot close what it opened. Measured with a whole-tree sweep: one violation appended to
    // every walked file, 45 planted, **44 named, one file blind**.
    //
    // ⚠️ The same blindness covered `//` and, in `.sql`, `--` inside a string. Tracking the
    // double quote is one flag and closes the class; it is NOT an enumeration of shapes.
    let mut in_dquote = false;
    let mut i = 0;
    while i < chars.len() {
        // A raw string swallows everything — comments included — until its own terminator.
        if let Some(hashes) = state.raw_hashes {
            if chars[i] == '"'
                && chars[i + 1..]
                    .iter()
                    .take(hashes)
                    .filter(|c| **c == '#')
                    .count()
                    == hashes
            {
                state.raw_hashes = None;
                out.push('"');
                i += 1 + hashes;
            } else {
                out.push(if chars[i] == '"' {
                    QUOTE_IN_LITERAL
                } else {
                    chars[i]
                });
                i += 1;
            }
            continue;
        }
        if state.in_plain {
            if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                state.in_plain = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        match chars[i] {
            // `\"` is a quote inside a literal, never a delimiter.
            '\\' if chars.get(i + 1) == Some(&'"') => {
                out.push(QUOTE_IN_LITERAL);
                i += 2;
            }
            // `r"`, `r#"`, `r##"` … open a raw string, where an unescaped `"` is data.
            'r' if !in_literal
                && chars[i + 1..]
                    .iter()
                    .position(|c| *c != '#')
                    .is_some_and(|k| chars.get(i + 1 + k) == Some(&'"'))
                && !matches!(chars.get(i.wrapping_sub(1)), Some(c) if is_token_char(*c as u8)) =>
            {
                let hashes = chars[i + 1..].iter().position(|c| *c != '#').unwrap_or(0);
                state.raw_hashes = Some(hashes);
                out.push('"');
                i += hashes + 2;
            }
            '\'' if !in_dquote => {
                in_literal = !in_literal;
                out.push('\'');
                i += 1;
            }
            // The double quote delimits a string in both languages this walks. Tracking it is what
            // stops a comment opener INSIDE one from blinding the rest of the file.
            '"' if !in_literal => {
                in_dquote = !in_dquote;
                out.push('"');
                i += 1;
            }
            '/' if !in_literal && !in_dquote && chars.get(i + 1) == Some(&'/') => break,
            '-' if !in_literal && !in_dquote && sql && chars.get(i + 1) == Some(&'-') => break,
            // `#` is a comment to MariaDB and to nothing else here, so it is stripped for `.sql`
            // only — in Rust it opens an attribute. Without it a `.sql` file's `# note` read as
            // CODE, which reds on a sentence ABOUT the rule: the shape probe `e05` exists to
            // protect, one comment syntax over.
            '#' if !in_literal && !in_dquote && sql => break,
            '/' if !in_literal && !in_dquote && chars.get(i + 1) == Some(&'*') => {
                if chars.get(i + 2) == Some(&'!') {
                    state.exec_depth += 1;
                    i += 3;
                    while chars.get(i).is_some_and(char::is_ascii_digit) {
                        i += 1;
                    }
                } else {
                    state.in_plain = true;
                    i += 2;
                }
            }
            '*' if !in_literal
                && !in_dquote
                && state.exec_depth > 0
                && chars.get(i + 1) == Some(&'/') =>
            {
                state.exec_depth -= 1;
                i += 2;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// Characters that occupy no width and separate no token — a zero-width space, a joiner, a BOM.
///
/// 🔴 They are DELETED rather than treated as whitespace. `INSERT` with a zero-width space inside
/// it is one word to the server and two to a whitespace-collapsing matcher; deleting restores the
/// word, collapsing would split it. The review's `e06` planted one at the head of a statement and
/// the gate went green.
///
/// ⚠️ **This is an ENUMERATION, and an enumeration cannot claim the completeness of a property.**
/// It named five ranges and missed the variation selectors and three invisible operators —
/// measured, `INS<U+FE0F>ERT` and `INS<U+2063>ERT` both passing. The ranges are widened ONCE, to
/// the blocks Unicode reserves for zero-width and formatting characters, and deliberately not to
/// exhaustion: chasing an adversary through the code-point space is a race
/// [`gate_declared_authorship`]'s stated promise has already declined. What closes that class is a
/// database privilege, not a longer list here. `e37` pins the widening; nothing pins completeness,
/// because nothing could.
pub(crate) fn is_invisible(ch: char) -> bool {
    matches!(
        ch,
        '\u{00ad}'
            | '\u{061c}'
            | '\u{180e}'
            | '\u{200b}'..='\u{200f}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206f}'
            | '\u{fe00}'..='\u{fe0f}'
            | '\u{feff}'
    )
}

/// One file's text, lowercased and whitespace-collapsed, plus the source line of every byte.
///
/// Whole-file rather than per-line — the difference from [`line_has_float`], and it is forced:
/// `INSERT INTO` and the table name may sit on two different lines, and a per-line matcher was
/// measured blind to exactly that. Comments — line AND block — are stripped, so the architecture
/// may be quoted; see [`strip_comments`] for the one comment form whose body survives.
pub(crate) fn normalise_sql_text(content: &str, sql_comments: bool) -> (String, Vec<usize>) {
    let mut out = String::with_capacity(content.len());
    let mut lines = Vec::with_capacity(content.len());
    let mut prev_space = true;
    let mut state = CommentState::default();
    for (idx, raw) in content.lines().enumerate() {
        let code = strip_comments(raw, sql_comments, &mut state);
        for ch in code.chars() {
            if is_invisible(ch) {
                continue;
            }
            if ch.is_whitespace() {
                if !prev_space {
                    out.push(' ');
                    lines.push(idx + 1);
                    prev_space = true;
                }
            } else {
                out.push(ch.to_ascii_lowercase());
                // 🔴 One entry per BYTE, not per char. `authorship_findings` indexes this map with
                // a byte offset into `out`, and `to_ascii_lowercase` leaves non-ASCII intact — so a
                // single multibyte character in any string literal shifted every later line number
                // by (bytes − chars). Measured on a 50-emoji literal: the gate reported line **0**
                // for a write on line 2. Not a detection hole — the finding still reds — but a gate
                // that sends the reader to the wrong line spends the trust it just earned.
                for _ in 0..ch.len_utf8() {
                    lines.push(idx + 1);
                }
                prev_space = false;
            }
        }
        if !prev_space {
            out.push(' ');
            lines.push(idx + 1);
            prev_space = true;
        }
    }
    (out, lines)
}

/// Is the `declared_attribute` occurrence at `at` a TABLE reference rather than part of a longer
/// identifier?
///
/// `insert_declared_attribute` contains the table's name preceded by `_`; a backtick or a schema dot
/// does NOT disqualify it, which is what makes `` `declared_attribute` `` and
/// `opencmdb.declared_attribute` reachable — both measured green (i.e. invisible) before this.
/// 🔑 **Parameterised by TABLE at story 6.3.** It hard-coded `DECLARED_TABLE.len()`, which — with
/// [`statement_after_of`] — was the whole cost of serving a second table, measured at that story's
/// validation. Everything else in this apparatus was already table-agnostic.
pub(crate) fn is_table_reference_of(text: &str, at: usize, table: &str) -> bool {
    let before = text[..at].chars().next_back();
    let after = text[at + table.len()..].chars().next();
    let head_ok = !matches!(before, Some(c) if c.is_alphanumeric() || c == '_');
    let tail_ok = !matches!(after, Some(c) if c.is_alphanumeric() || c == '_');
    head_ok && tail_ok
}

/// The statement fragment a table reference belongs to.
///
/// Bounded by the nearest preceding `;` **or** `"` — whichever is closer. The `"` bound is what
/// stops a match spanning two string literals: without it, a bare `DELETE FROM declared_attribute`
/// was measured inheriting an `origin` from an unrelated INSERT twenty-four lines above, and the
/// gate reported two phantom findings on the clean tree.
pub(crate) fn statement_before(text: &str, at: usize) -> &str {
    &text[statement_start(text, at)..at]
}

/// The last statement bound strictly before `at`, **skipping a `;` that sits inside a
/// single-quoted SQL string**.
///
/// 🔴 Added at story 6.3's code review, on a measurement WITH its control. The bound was the first
/// `;` whatever its context, so
/// `INSERT INTO observation_record … VALUES (1, 'a;payload') ON DUPLICATE KEY UPDATE …` captured
/// nothing after the reference and the overwrite went **undetected** — while the same line without
/// the semicolon reddened. ⚠️ **The hole was measured to be INHERITED by
/// [`gate_declared_authorship`]'s read half** (`… WHERE raw = 'a;b' AND actor_id = 'x'` → no
/// finding; control → one), which is why the fix lives in the SHARED helper rather than in one
/// gate. `raw` is an opaque blob by design and `ON DUPLICATE KEY UPDATE` is the ordinary
/// idempotent-ingest gesture, so this was the good-faith path, not an adversary's.
///
/// `"` still bounds unconditionally: it ends a Rust string literal, and that bound is what stops a
/// finding spanning two literals (story 5.12). Byte scanning is safe — `;`, `'` and `"` are ASCII,
/// and a UTF-8 continuation byte can never equal them.
fn statement_start(text: &str, at: usize) -> usize {
    let bytes = text.as_bytes();
    let mut in_quote = false;
    let mut last = 0usize;
    for (i, b) in bytes.iter().enumerate().take(at) {
        match b {
            b'\'' => in_quote = !in_quote,
            b';' if !in_quote => last = i + 1,
            b'"' => {
                last = i + 1;
                in_quote = false;
            }
            _ => {}
        }
    }
    last
}

/// The first statement bound at or after `from`, under [`statement_start`]'s rule.
fn statement_end(text: &str, from: usize) -> usize {
    let bytes = text.as_bytes();
    let mut in_quote = false;
    for (i, b) in bytes.iter().enumerate().skip(from) {
        match b {
            b'\'' => in_quote = !in_quote,
            b';' if !in_quote => return i,
            b'"' => return i,
            _ => {}
        }
    }
    text.len()
}

/// The rest of the statement, AFTER the table reference, under the same bounds.
///
/// 🔴 The review's largest read-half hole: the first implementation inspected only what stood
/// BEFORE the table name, so `WHERE actor_id = 'scanner'`, `ORDER BY origin_obs_id` and a join
/// predicate on `d.origin` all passed. A provenance column read in a predicate is read (FR13).
///
/// 🔑 **Parameterised by TABLE at story 6.3** — see [`is_table_reference_of`].
pub(crate) fn statement_after_of<'t>(text: &'t str, at: usize, table: &str) -> &'t str {
    let from = at + table.len();
    &text[from..statement_end(text, from)]
}

/// The `fn` a byte offset sits inside, if any — the unit [`SANCTIONED_SITES`] pairs with a path.
///
/// 🔴 It was an unbounded `rfind("fn ")`, and the review defeated it twice: a nested
/// `fn insert_declared_attribute() {}` **whose body had already closed** sanctioned everything
/// after it, and the same name inside a string literal did the same. So a candidate must look like
/// a declaration — its name followed by `(` or a generic list — and its braces must still be OPEN
/// where the reference sits. The innermost such `fn` wins.
pub(crate) fn enclosing_fn(text: &str, at: usize) -> Option<&str> {
    let mut found = None;
    let mut from = 0usize;
    while let Some(rel) = text[from..at].find("fn ") {
        let head = from + rel;
        from = head + 3;
        if text[..head]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
        {
            continue;
        }
        let rest = &text[head + 3..];
        let Some(end) = rest.find(|c: char| !(c.is_alphanumeric() || c == '_')) else {
            continue;
        };
        if end == 0 {
            continue;
        }
        let after = rest[end..].trim_start();
        if !(after.starts_with('(') || after.starts_with('<')) {
            continue;
        }
        let Some(open) = rest.find('{') else { continue };
        let limit = at - (head + 3);
        if open >= limit {
            continue;
        }
        let mut depth = 0i32;
        let mut closed = false;
        for (i, ch) in rest[open..limit].char_indices() {
            let _ = i;
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        closed = true;
                        break;
                    }
                }
                _ => {}
            }
        }
        if !closed {
            found = Some(&rest[..end]);
        }
    }
    found
}

/// A projection with every parenthesised group removed.
///
/// ⚠️ Without it `SELECT COUNT(*) FROM declared_attribute` is read as a wildcard — measured on the
/// committed tree at `repo.rs:106`, the gate's first red and a FALSE POSITIVE. An aggregate's star
/// loads no column; `SELECT *` loads all three provenance columns. The distinction is the whole
/// difference between a gate and a nuisance.
pub(crate) fn outside_parens(projection: &str) -> String {
    let mut out = String::with_capacity(projection.len());
    let mut depth = 0usize;
    for ch in projection.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    out
}

/// The verbs that put a value into the guarded table without passing through a human author.
///
/// `DELETE` and `TRUNCATE` are deliberately ABSENT: NFR5 is about AUTHORSHIP and a removal writes
/// no author — including them was measured reddening the committed tree at two fixture sites.
/// `RENAME TABLE` and `ALTER … DROP CONSTRAINT` are absent because they touch no ROW at all; they
/// neutralise the guard, and a text matcher is the wrong closure for that (probes `e14`, `e31`).
///
/// ⚠️ **`CREATE OR REPLACE TABLE` is the entry that does NOT follow from that criterion, and saying
/// so is the point.** By the authorship test it belongs with `e14` and `e31` — it writes no row
/// under a false author either; it destroys the table and every declared value in it. It reds
/// anyway, for two reasons that must not be confused:
///
/// 1. **The red is incident, not decided.** Mutation M19b removed this entry and `e22` stayed RED,
///    because the `REPLACE` hiding inside the phrase governs the same reference. The entry earns
///    its place by NAMING the finding correctly — a message saying `replace` for a statement that
///    drops the table sends the reader after the wrong thing.
/// 2. **The red is kept on purpose.** The gesture annihilates the guarded table, and it lives in a
///    `.sql` migration — the place this story measured to be the most natural home for a bulk
///    rewrite. Keeping a red that the criterion does not demand is a decision; pretending the
///    criterion demanded it would be a false sentence, and the review caught this file writing one.
const WRITE_VERBS: [&str; 7] = [
    "create or replace table",
    "insert into",
    "insert",
    "load data",
    "replace into",
    "replace",
    "update",
];

/// Does `needle` sit at `at` in `hay` as a whole token?
pub(crate) fn is_word_at(hay: &str, at: usize, len: usize) -> bool {
    let b = hay.as_bytes();
    let before_ok = at == 0 || !is_token_char(b[at - 1]);
    let after_ok = at + len >= b.len() || !is_token_char(b[at + len]);
    before_ok && after_ok
}

/// The keyword that governs a table reference: the write verb or `select` whose match ENDS latest
/// before it.
///
/// 🔑 The first implementation anchored on the statement's HEAD, and the review walked through it
/// six ways — a CTE (`WITH x AS (SELECT origin FROM …)`), a subquery, and a trigger body whose
/// `INSERT INTO` sits fifty characters into a `CREATE TRIGGER`. What governs a reference is the
/// nearest preceding keyword, not the first one in the statement.
///
/// At equal end offsets the LONGER keyword wins, so `CREATE OR REPLACE TABLE` is not reported as
/// the `REPLACE` hiding inside it.
pub(crate) fn governing_keyword(stmt: &str) -> Option<&'static str> {
    let mut best: Option<(usize, &'static str)> = None;
    for kw in WRITE_VERBS.into_iter().chain(["select"]) {
        let mut from = 0usize;
        while let Some(rel) = stmt[from..].find(kw) {
            let at = from + rel;
            from = at + kw.len();
            if !is_word_at(stmt, at, kw.len()) {
                continue;
            }
            let end = at + kw.len();
            let better = match best {
                None => true,
                Some((e, k)) => end > e || (end == e && kw.len() > k.len()),
            };
            if better {
                best = Some((end, kw));
            }
        }
    }
    best.map(|(_, kw)| kw)
}
