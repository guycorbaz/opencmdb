//! The `entity-id-immutable` gate — D15, and the line of SQL the architecture calls the most
//! dangerous in this project.
//!
//! > **THE RULE: `declared_attribute.entity_id` is NEVER updated. Ever. No UPDATE.**
//! >
//! > *A `declared_attribute` is a human's testimony about a referent. If the referent changes, the
//! > meaning of the testimony changes. Rewriting `entity_id` is making a human say something they
//! > never said, about an object they never saw. **It is not moving data, it is falsification.**
//! > `UPDATE declared_attribute SET entity_id = ?` is the most dangerous line of SQL in this
//! > project — **and it looks like a routine refactor.*** (`architecture.md:1089-1095`)
//!
//! # Why a gate, and not a test
//!
//! Because the property is the ABSENCE of a code path, and *you cannot measure the absence of code
//! by running code* (story 5.12). A test exercises what exists; the violation is what a future
//! story ADDS. Story 5.12's rule governs here in full — unlike `device`'s column list, which is
//! bounded, present in one place and therefore measurable by a test that reads the applied schema.
//!
//! ⚠️ **It was held by NEITHER existing gate, and the register said so before this story was
//! written**: `authorship` guards WHO a declared write claims to be, and `observed-immutable`
//! guards a different table. An `UPDATE declared_attribute SET entity_id = ?` at a sanctioned site
//! passed both. **Owner: story 6.5** — *"where `entity_id` acquires meaning"*.
//!
//! # It is BROADER than D15, by decision
//!
//! D15 forbids updating one COLUMN. This gate refuses **any `UPDATE` of `declared_attribute`**, and
//! the reason is a rule this project has paid for twice: a matcher that must parse a `SET` clause to
//! tell `SET entity_id = ?` from `SET attr_value = ?` is a matcher that will be wrong in both
//! directions — story 5.12's authorship gate spent a day on exactly that class, and story 6.5's
//! validation then measured the neighbouring `ddl-collation` matcher passing four of five planted
//! violations because `_BIN` anywhere on a line satisfied it.
//!
//! **The widening costs nothing today, measured**: there is no `UPDATE declared_attribute` anywhere
//! under `crates/` or `docker/`. The day the identity-migration mechanism needs one — D15's own
//! mechanism moves source attributes to `state='pending_migration'` — that story reopens this gate
//! deliberately, which is the coupling we want rather than a gate that silently already allowed it.
//!
//! # And it is a TRIPWIRE, never a barrier
//!
//! Story 5.12's narrowing, fourth application. It reads the source text of the files under
//! [`crate::AUTHORSHIP_ROOTS`]; a query assembled at runtime, a stored procedure, or a hand typed at
//! a `mysql` prompt is outside it and always will be. Read it as *"a future story will not add such
//! a write by accident"*, never as *"such a write cannot exist"*. The barrier is a database
//! privilege, registered as the real closure since story 5.12 and still unbuilt.

use std::path::Path;

use anyhow::{Context, Result};

use crate::{
    AUTHORSHIP_ROOTS, governing_keyword, is_table_reference_of, normalise_sql_text,
    statement_before,
};

/// The guarded table.
const DECLARED_TABLE: &str = "declared_attribute";

/// Every `UPDATE` of `declared_attribute` in one file's text, as `(line, message)`.
///
/// The line is the SOURCE line the table reference sits on — story 5.12 paid for that precision
/// with a character/byte offset map that sent the reader to the wrong line, and **a pinned boolean
/// proves THAT a gate fires and never WHERE.**
pub(crate) fn entity_id_immutable_findings(
    content: &str,
    sql_comments: bool,
) -> Vec<(usize, String)> {
    let (text, lines) = normalise_sql_text(content, sql_comments);
    let mut findings = Vec::new();
    let mut from = 0usize;

    while let Some(rel) = text[from..].find(DECLARED_TABLE) {
        let at = from + rel;
        from = at + DECLARED_TABLE.len();
        if !is_table_reference_of(&text, at, DECLARED_TABLE) {
            continue;
        }
        let line = lines.get(at).copied().unwrap_or(0);
        let before = statement_before(&text, at).trim_start();

        // A `CREATE TABLE`, a `SELECT … FROM`, an `INSERT` and a bare mention all govern something
        // this gate is not about. `INSERT` is the authorship gate's subject, not D15's.
        let Some(keyword) = governing_keyword(before) else {
            continue;
        };
        if !matches!(keyword, "update") {
            continue;
        }
        findings.push((
            line,
            format!(
                "`update {DECLARED_TABLE}` — a declared attribute is a human's testimony about a \
                 referent, and rewriting it is falsification, not data movement (D15)"
            ),
        ));
    }
    findings
}

/// The gate: no code path updates a declared attribute.
///
/// # Errors
///
/// Fails closed — a missing root or a walk that reads zero files is a RED, not a pass. *A gate that
/// greens because it found nothing to look at is decoration* (D18), and story 6b.11 measured a
/// browser gate reporting success over a surface it never reached.
pub(crate) fn gate_entity_id_immutable(root: &Path) -> Result<(bool, String)> {
    let mut findings: Vec<String> = Vec::new();
    let mut walked = 0usize;

    for sub in AUTHORSHIP_ROOTS {
        let dir = root.join(sub);
        if !dir.exists() {
            return Ok((
                false,
                format!("{sub}/ is missing — the gate cannot vouch for a tree it did not read"),
            ));
        }
        for entry in walkdir::WalkDir::new(&dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());
            if ext != Some("rs") && ext != Some("sql") {
                continue;
            }
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("reading {}", path.display()))?;
            walked += 1;
            let shown = path.strip_prefix(root).unwrap_or(path);
            let shown = shown.display().to_string().replace('\\', "/");
            for (line, message) in entity_id_immutable_findings(&content, ext == Some("sql")) {
                findings.push(format!("{shown}:{line}: {message}"));
            }
        }
    }
    findings.sort();

    if walked == 0 {
        return Ok((
            false,
            "read zero files — a gate that greens on an empty walk is decoration".into(),
        ));
    }
    if findings.is_empty() {
        return Ok((
            true,
            format!("declared attributes never re-pointed across {walked} file(s)"),
        ));
    }
    Ok((
        false,
        format!(
            "{} update(s) of {DECLARED_TABLE}:\n      {}",
            findings.len(),
            findings.join("\n      ")
        ),
    ))
}
