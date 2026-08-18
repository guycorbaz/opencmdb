//! The `observed-immutable` gate — story 6.3, NFR5's second assertion.
//!
//! `observation_record` is the measuring instrument. *"Documenting a field sets the declared value
//! **and leaves the observation record bit-for-bit unchanged, with the link intact**"*
//! (`prd.md:1214-1217`), and D2 puts it plainly: *"the observed record does not move a single
//! byte"*, one of the six conditions whose loss turns adoption into a destructive merge.
//!
//! # Why a GATE and not only a test
//!
//! The story ships behavioural guards too — they compare the row across a documenting gesture,
//! byte for byte. But on the tree that introduced them **there is no `UPDATE observation_record`
//! anywhere**, so those guards cannot fail: they run what exists, and the violation is what a
//! future story would ADD. *You cannot measure the absence of code by running code* — story
//! 5.12's sentence, and this gate is its second application.
//!
//! Neither carrier subsumes the other, and that is MEASURED rather than argued: a write planted in
//! a new **uncalled** function leaves the **product** suite green (`opencmdb-bin` + `opencmdb-core`)
//! and reds this gate (story 6.3's mutation M1b).
//! ⚠️ It does NOT leave the whole WORKSPACE suite green — [`crate::tests`]'s
//! `the_observed_gate_is_green_on_the_real_tree` reds too, because that test IS this gate run
//! against the real tree. The wider sentence stood here until story 6.3's own code review; it was
//! refuted by that story's own Debug Log, in the same commit.
//!
//! # What it reds on
//!
//! Every OVERWRITING form of a write to `observation_record`, in `.rs` **and** `.sql` under
//! [`crate::AUTHORSHIP_ROOTS`]: a governing `UPDATE`, a governing `REPLACE` / `REPLACE INTO`, and
//! an `INSERT … ON DUPLICATE KEY UPDATE`.
//!
//! 🔴 **`UPDATE` alone was measured insufficient.** The first draft named that verb only, and both
//! validation layers planted `INSERT INTO observation_record … ON DUPLICATE KEY UPDATE raw =
//! VALUES(raw)` and `REPLACE INTO observation_record …` — both **modify an existing row**, and
//! both came back GREEN, because the keyword governing the reference is `insert into` /
//! `replace into`. Story 5.12's own probe corpus carries that shape (`e12_on_dup_key.rs`); it reds
//! *there* only because the authorship gate flags every non-`select` verb. Restricting to `update`
//! would have re-opened, for this table, the hole 5.12 had closed for its own — and
//! `ON DUPLICATE KEY UPDATE` is the ordinary gesture (*"make the ingest idempotent"*), not an
//! adversary's.
//!
//! # The allowlist is EMPTY, and that is the point
//!
//! Measured on the committed tree: **zero occurrences**, `docker/` included. Where the
//! authorship gate needed four sanctioned sites, this one needs none — and **an empty allowlist is
//! the one form of allowlist nobody can quietly widen.** If a future story finds itself adding the
//! first entry here, that is the moment to ask whether the invariant still holds, not a formality.
//!
//! # What it deliberately does NOT red on
//!
//! - **`DELETE`.** NFR5 is about the observed record not being ALTERED under the scanner's hand; a
//!   bulk delete is data loss, a different invariant, and this gate does not hold it. The same call
//!   was taken for `declared_attribute` at story 5.12 (`deferred-work.md:2872-2877`). ⚠️ Here it is
//!   also concrete: `docker/seed-example.sql:24` carries a live `DELETE FROM observation_record`
//!   (the demo's idempotent cleanup), so the verb would red the committed tree and buy an
//!   exemption for a shipped file that has no test.
//! - **`INSERT` on its own.** Ingestion is how observations come into being (`repo::insert_observation`,
//!   `docker/seed-example.sql:35`). An append is not an overwrite.
//! - **`identity_link` and `interface`.** The engine legitimately UPDATEs both —
//!   `repo::close_identity_link` is how a placement is superseded, `widen_interface_seen_window`
//!   how a window grows. A gate over a table the product rightly mutates would need an allowlist as
//!   long as its callers, which is a gate that means nothing. *"The link is intact"* is therefore
//!   carried by a behavioural comparison only, and the story says so.
//! - **A write through a VIEW.** Measured at story 6.3's code review: `CREATE VIEW obs_v AS SELECT
//!   … FROM observation_record;` then `UPDATE obs_v SET raw = …` → **GREEN**. The `UPDATE` never
//!   names the guarded table, and MariaDB updates a simple view through to its base table. **No
//!   text matcher can close this** — it needs the schema, not the source — so it is named as a
//!   residual class rather than chased.
//! - **`RENAME TABLE` and `ALTER TABLE … DROP COLUMN`.** Measured GREEN. A shadow-table swap
//!   (`RENAME TABLE observation_record TO old, shadow TO observation_record`) replaces a table's
//!   whole contents without any row-level verb firing — *an overwrite in spirit*. Whether a gate
//!   should reach table-level replacement at all is a SCOPE question both gates share, and it is
//!   registered rather than answered here.
//! - **`declared_attribute`.** That is [`crate::gate_declared_authorship`]'s table. ⚠️ And D15's
//!   sibling rule — *"`declared_attribute.entity_id` is NEVER updated. Ever. No UPDATE"*, which
//!   `architecture.md:1064-1069` calls *"the most dangerous line of SQL in this project"* — is held
//!   by NEITHER gate today. It is registered against the entity/device schema story.
//!
//! # The promise, at its width and no wider
//!
//! **A TRIPWIRE against a good-faith change, never a barrier against a determined one.** Read it as
//! *"a future story will not add such a write by accident"*, never as *"such a write cannot
//! exist"*. Story 5.12's residual classes are inherited verbatim and not re-litigated here: a table
//! name assembled at runtime (`format!("observation_{}", …)`) is invisible to any text matcher, and
//! guard neutralisation — editing this file — is closed by a database privilege rather than by a
//! gate. That `GRANT` is registered as the real closure for both gates.
//!
//! # Its FALSE POSITIVES, named so nobody closes them with an allowlist
//!
//! A tripwire that reds on legitimate code gets deleted, so its false positives matter as much as
//! its holes. Both of these were measured at story 6.3's code review:
//!
//! - **Ordinary prose.** A string literal reading *"refusing to update observation_record
//!   automatically"* REDS. Comments are stripped; string content is not, and a text matcher cannot
//!   tell prose from SQL. **If you hit this, rephrase the message** — do not add the first entry to
//!   an allowlist whose emptiness is the property above.
//! - **A filter-only JOIN.** `UPDATE identity_link il JOIN observation_record o ON … SET
//!   il.valid_to = NOW(6) WHERE o.raw IS NOT NULL` REDS, though the `SET` touches `identity_link`
//!   alone. The verdict follows the nearest governing verb and never the `SET` clause. ⚠️ The
//!   corpus's *"engine supersede"* negative uses the SUBQUERY form, so an ordinary rewrite to a
//!   join would red a non-violation; `o28_join_filter_only.sql` pins this to its ACTUAL behaviour
//!   rather than to the one we would prefer.
//!
//! # Where its tests live
//!
//! In `main.rs`'s trailing test module rather than in a `#[cfg(test)] mod tests` here, against
//! D56b's house convention — deliberately, to reuse `scratch()` and `workspace_root()`, which live
//! there. Stated because an undiscussed deviation from a house rule reads as an oversight.

use std::path::Path;

use anyhow::{Context, Result};

use crate::{
    AUTHORSHIP_ROOTS, governing_keyword, is_table_reference_of, normalise_sql_text,
    statement_after_of, statement_before,
};

/// The guarded table.
const OBSERVED_TABLE: &str = "observation_record";

/// The phrase that turns an `INSERT` into an overwrite. Matched in the statement AFTER the table
/// reference, which is where MariaDB's grammar puts it.
const ON_DUPLICATE: &str = "on duplicate key update";

/// Every overwriting access to `observation_record` in one file's text, as `(line, message)`.
///
/// The line is the SOURCE line the reference sits on. ⚠️ Story 5.12 paid for that precision: its
/// offset→line map counted CHARACTERS while the caller indexed BYTES, so any multibyte literal
/// shifted every later finding — a gate that sends the reader to the wrong line spends the trust it
/// just earned, and **a pinned boolean proves THAT a gate fires and never WHERE.**
pub(crate) fn observed_immutable_findings(
    content: &str,
    sql_comments: bool,
) -> Vec<(usize, String)> {
    let (text, lines) = normalise_sql_text(content, sql_comments);
    let mut findings = Vec::new();
    let mut from = 0usize;

    while let Some(rel) = text[from..].find(OBSERVED_TABLE) {
        let at = from + rel;
        from = at + OBSERVED_TABLE.len();
        if !is_table_reference_of(&text, at, OBSERVED_TABLE) {
            continue;
        }
        let line = lines.get(at).copied().unwrap_or(0);
        let before = statement_before(&text, at).trim_start();
        let after = statement_after_of(&text, at, OBSERVED_TABLE);

        // `CREATE TABLE`, a foreign-key clause, a bare mention and a `SELECT … FROM` all govern
        // nothing this gate is about.
        let Some(keyword) = governing_keyword(before) else {
            continue;
        };

        let verdict = match keyword {
            "update" | "replace" | "replace into" | "create or replace table" => Some(keyword),
            // An append that overwrites on collision is an overwrite, whatever it is spelled.
            "insert" | "insert into" if after.contains(ON_DUPLICATE) => Some(ON_DUPLICATE),
            _ => None,
        };
        let Some(named) = verdict else {
            continue;
        };
        findings.push((
            line,
            format!("`{named} {OBSERVED_TABLE}` — an observation is immutable (NFR5, FR11)"),
        ));
    }
    findings
}

/// The gate: no code path overwrites an observation record.
///
/// # Errors
///
/// Fails closed — a missing root or a walk that reads zero files is a RED, not a pass. A gate that
/// greens because it found nothing to look at is the decoration D18 forbids.
pub(crate) fn gate_observed_immutable(root: &Path) -> Result<(bool, String)> {
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
            for (line, message) in observed_immutable_findings(&content, ext == Some("sql")) {
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
            format!("observations immutable across {walked} file(s)"),
        ));
    }
    Ok((
        false,
        format!(
            "{} overwriting access(es) to {OBSERVED_TABLE}:\n      {}",
            findings.len(),
            findings.join("\n      ")
        ),
    ))
}
