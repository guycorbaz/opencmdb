//! xtask — `cargo xtask <cmd>`. All CI gates, in Rust, not YAML (D56).
//!
//! `cargo xtask ci` runs every gate and exits non-zero if any is RED:
//!   - **frontier** (D47): the dependency graph IS the frontier. `opencmdb-core` (the
//!     domain, where an error is DOMAIN DATA) must not resolve `anyhow`, `axum`, `sqlx`
//!     or `askama`; `opencmdb-bin` must not resolve `xtask` (a dependency of nobody).
//!     Reads `cargo tree` — the GRAPH, never the manifest text. A reflex gate (D53).
//!   - **ddl-collation** (D64 condition 1): every text column in a migration carries an
//!     explicit binary collation. No allowlist — the absence IS the mechanism. A reflex
//!     gate (D53), not a proof; it bites on a real migration once one exists.
//!   - **vocabulary** (D65): retired terms must not survive. Volet A — retired *code*
//!     identifiers (`pending_accept`, `reverting`, `accept-as-declared`) absent from
//!     `crates/`. Volet B — CO-PRESENCE across the planning docs: a body that holds a
//!     RETIRED term with its LIVE replacement nowhere is a stale document, and reds.
//!   - **views-hash** (informational): whether `architecture-views.md`'s `sourceSha256`
//!     still matches `architecture.md`. A mismatch means the views file is stale and
//!     should be regenerated at the next milestone — reported, never a hard failure.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anyhow::{Context, Result};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("ci") => match run_ci() {
            Ok(true) => ExitCode::SUCCESS,
            Ok(false) => ExitCode::FAILURE,
            Err(e) => {
                eprintln!("xtask ci: error: {e:#}");
                ExitCode::FAILURE
            }
        },
        Some(other) => {
            eprintln!("xtask: unknown command {other:?}\nusage: cargo xtask ci");
            ExitCode::FAILURE
        }
        None => {
            eprintln!("usage: cargo xtask ci");
            ExitCode::FAILURE
        }
    }
}

/// Workspace root = the parent of xtask's own manifest directory.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask always has a parent directory")
        .to_path_buf()
}

fn run_ci() -> Result<bool> {
    let root = workspace_root();
    println!("cargo xtask ci — gates (D56/D65)\n");
    let mut ok = true;

    let (g0, m0) = gate_dependency_frontier(&root)?;
    report("frontier", g0, &m0);
    ok &= g0;

    let (g1, m1) = gate_ddl_collation(&root)?;
    report("ddl-collation", g1, &m1);
    ok &= g1;

    let (g2, m2) = gate_vocabulary(&root)?;
    report("vocabulary", g2, &m2);
    ok &= g2;

    let m3 = check_views_hash(&root)?;
    println!("  ℹ  {:<14} {m3}", "views-hash");

    println!(
        "\n{}",
        if ok {
            "✅ all gates green"
        } else {
            "🔴 one or more gates RED"
        }
    );
    Ok(ok)
}

fn report(name: &str, ok: bool, msg: &str) {
    println!("  {} {name:<14} {msg}", if ok { "✅" } else { "🔴" });
}

// ── Gate 0: dependency frontier (D47) ───────────────────────────────────────

/// The domain crate cannot name what touches the outside world. An error in
/// `opencmdb-core` is DOMAIN DATA, not an `anyhow` string (D47).
const CORE_FORBIDDEN: &[&str] = &["anyhow", "axum", "sqlx", "askama"];

/// The frontier is the resolved dependency GRAPH, not a manifest rule. Reads `cargo tree`,
/// so a `Cargo.toml` comment that merely names a banned crate never reaches the detector.
fn gate_dependency_frontier(root: &Path) -> Result<(bool, String)> {
    let core_tree = cargo_tree(root, "opencmdb-core")?;
    let bin_tree = cargo_tree(root, "opencmdb-bin")?;
    let offenders = frontier_offenders(&core_tree, &bin_tree);
    if offenders.is_empty() {
        Ok((
            true,
            "domain graph clean; xtask depended on by nobody".into(),
        ))
    } else {
        Ok((
            false,
            format!(
                "{} finding(s):\n      {}",
                offenders.len(),
                offenders.join("\n      ")
            ),
        ))
    }
}

/// The decision, factored out of I/O so it is unit-tested on synthetic trees (D45): given
/// each crate's DIRECT-dependency tree, name every frontier crossing. `opencmdb-core` must
/// not directly resolve a `CORE_FORBIDDEN` crate; NO product crate may resolve `xtask`
/// (a dependency of nobody — D56).
fn frontier_offenders(core_tree: &str, bin_tree: &str) -> Vec<String> {
    let mut offenders = Vec::new();
    let core_crates = crates_present_in_tree(core_tree);
    for banned in CORE_FORBIDDEN {
        if core_crates.contains(*banned) {
            offenders.push(format!(
                "opencmdb-core depends on forbidden crate '{banned}'"
            ));
        }
    }
    for (crate_name, tree) in [
        ("opencmdb-core", &core_crates),
        ("opencmdb-bin", &crates_present_in_tree(bin_tree)),
    ] {
        if tree.contains("xtask") {
            offenders.push(format!("{crate_name} depends on forbidden crate 'xtask'"));
        }
    }
    offenders
}

/// Shell `cargo tree` for `pkg`'s DIRECT dependencies and return its stdout.
/// - `--depth 1`: direct deps only — D47 is about what core can *name* (`use anyhow`), which
///   needs a DIRECT dependency; a transitive `anyhow` is unusable by core and must not red.
/// - `-e normal`: drops dev/build edges (so `xtask`, a dep of nobody, never falsely trips).
/// - `--charset utf8`: pin the glyph set the parser strips — immune to a `[term] charset`
///   config or a future cargo default, so the gate can never go silently green on mis-parse.
/// - `--locked`: keeps the check side-effect-free.
fn cargo_tree(root: &Path, pkg: &str) -> Result<String> {
    let out = Command::new(env!("CARGO"))
        .current_dir(root)
        .args([
            "tree",
            "-p",
            pkg,
            "-e",
            "normal",
            "--depth",
            "1",
            "--charset",
            "utf8",
            "--locked",
        ])
        .output()
        .with_context(|| format!("running `cargo tree -p {pkg}`"))?;
    if !out.status.success() {
        anyhow::bail!(
            "`cargo tree -p {pkg}` failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    String::from_utf8(out.stdout).with_context(|| format!("`cargo tree -p {pkg}` stdout not UTF-8"))
}

/// Extract the set of crate names from `cargo tree` text. Each line is
/// `<tree glyphs> <name> v<version> [(*)|(proc-macro)|(path)]`. Strip the leading glyphs,
/// take the first whitespace token as the crate name. WHOLE-token by construction — a
/// `<name> v…` shape means `anyhow-macros` is its own token and never reads as `anyhow`.
fn crates_present_in_tree(tree: &str) -> HashSet<String> {
    let mut names = HashSet::new();
    for line in tree.lines() {
        // Drop tree-drawing glyphs and indentation; the crate name is the first token left.
        let stripped = line
            .trim_start_matches(|c: char| c.is_whitespace() || matches!(c, '│' | '├' | '└' | '─'));
        if let Some(name) = stripped.split_whitespace().next()
            && !name.is_empty()
        {
            names.insert(name.to_string());
        }
    }
    names
}

// ── Gate 1: DDL binary collation (D64 condition 1) ──────────────────────────

fn gate_ddl_collation(root: &Path) -> Result<(bool, String)> {
    let mig = root.join("crates/opencmdb-bin/migrations");
    if !mig.exists() {
        return Ok((true, "no migrations/ yet — nothing to check".into()));
    }
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&mig)
        .into_iter()
        .filter_map(Result::ok)
    {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("sql") {
            continue;
        }
        let content =
            std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?;
        for (i, line) in content.lines().enumerate() {
            if let Some(col) = text_column_without_binary_collation(line) {
                offenders.push(format!("{}:{}: {col}", p.display(), i + 1));
            }
        }
    }
    if offenders.is_empty() {
        Ok((
            true,
            "every text column carries an explicit binary collation".into(),
        ))
    } else {
        Ok((
            false,
            format!(
                "{} text column(s) without a binary collation:\n      {}",
                offenders.len(),
                offenders.join("\n      ")
            ),
        ))
    }
}

/// Reflex heuristic: a line that declares a text-typed column with no binary collation.
fn text_column_without_binary_collation(line: &str) -> Option<String> {
    let l = line.trim();
    if l.starts_with("--") || l.is_empty() {
        return None;
    }
    let up = l.to_uppercase();
    let is_text = up.contains("VARCHAR")
        || up.contains("TEXT")
        || up.contains(" CHAR")
        || up.starts_with("CHAR")
        || up.contains("CLOB");
    if !is_text {
        return None;
    }
    let has_binary_collation = up.contains("_BIN") || up.contains("COLLATE BINARY");
    if has_binary_collation {
        None
    } else {
        Some(l.trim_end_matches(',').to_string())
    }
}

// ── Gate 2: retired vocabulary (D65) ────────────────────────────────────────

/// The planning documents in volet-B scope. Missing files are skipped.
const DOCS: &[&str] = &[
    "_bmad-output/planning-artifacts/prd.md",
    "_bmad-output/planning-artifacts/ux-design-specification.md",
    "_bmad-output/planning-artifacts/architecture.md",
    "_bmad-output/planning-artifacts/architecture-views.md",
    "_bmad-output/planning-artifacts/product-brief-opencmdb.md",
    "_bmad-output/planning-artifacts/product-brief-opencmdb-distillate.md",
    "docs/project-context.md",
];

/// (retired term, its live replacement(s)). Co-presence: a doc holding the retired term
/// with NONE of the replacements is stale. The replacement's presence and the correct
/// repair are the same act, so the red has exactly one repair (D45).
const PAIRS: &[(&str, &[&str])] = &[
    ("pending_accept", &["pending_commit"]),
    ("reverting", &["failed", "in_queue"]),
    ("accept-as-declared", &["accept-gap", "document"]),
    ("ignore", &["exclude"]),
];

/// Retired *code* identifiers — unambiguous, no legitimate other meaning in Rust, so
/// safe to forbid outright in `crates/`. `ignore` is deliberately absent: it is a real
/// Rust token (`#[ignore]`), and its doc-level check is covered by co-presence above.
const CODE_RETIRED: &[&str] = &[
    "pending_accept",
    "reverting",
    "accept_as_declared",
    "accept-as-declared",
];

fn gate_vocabulary(root: &Path) -> Result<(bool, String)> {
    let mut red = Vec::new();

    // Volet B — co-presence across the planning docs (body only; the frontmatter is a
    // journal and may record old names).
    for rel in DOCS {
        let p = root.join(rel);
        if !p.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&p).with_context(|| format!("reading {rel}"))?;
        let body = strip_frontmatter(&content).to_lowercase();
        for (retired, repls) in PAIRS {
            let has_retired = contains_word(&body, &retired.to_lowercase());
            let has_repl = repls
                .iter()
                .any(|r| contains_word(&body, &r.to_lowercase()));
            if has_retired && !has_repl {
                red.push(format!(
                    "{rel}: contains '{retired}' but none of its replacement(s) {repls:?}"
                ));
            }
        }
    }

    // Volet A — retired identifiers in product code (crates/ only; xtask defines the
    // denylist and would match itself).
    let crates = root.join("crates");
    if crates.exists() {
        for entry in walkdir::WalkDir::new(&crates)
            .into_iter()
            .filter_map(Result::ok)
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let content = std::fs::read_to_string(p)?.to_lowercase();
            for term in CODE_RETIRED {
                if contains_word(&content, &term.to_lowercase()) {
                    let shown = p.strip_prefix(root).unwrap_or(p);
                    red.push(format!("{}: retired identifier '{term}'", shown.display()));
                }
            }
        }
    }

    if red.is_empty() {
        Ok((true, "co-presence green across docs; code clean".into()))
    } else {
        Ok((
            false,
            format!("{} finding(s):\n      {}", red.len(), red.join("\n      ")),
        ))
    }
}

/// Drop a leading YAML frontmatter block delimited by `---` lines. Body sections may
/// themselves contain `---` rules, so only the FIRST block is stripped.
fn strip_frontmatter(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.first().map(|l| l.trim_end()) == Some("---")
        && let Some(rel_end) = lines.iter().skip(1).position(|l| l.trim_end() == "---")
    {
        let body_start = rel_end + 2; // +1 for skip(1), +1 to pass the closing ---
        return lines[body_start..].join("\n");
    }
    content.to_string()
}

/// Whole-token containment (case handled by the caller lowercasing both sides). `-` and
/// `_` count as token characters, so `accept-as-declared` matches as a unit and `ignore`
/// does not match inside `ignored`.
fn contains_word(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let bytes = haystack.as_bytes();
    let nlen = needle.len();
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        let i = start + pos;
        let before_ok = i == 0 || !is_token_char(bytes[i - 1]);
        let after = i + nlen;
        let after_ok = after >= bytes.len() || !is_token_char(bytes[after]);
        if before_ok && after_ok {
            return true;
        }
        start = i + 1;
    }
    false
}

fn is_token_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

// ── Check 3: views-hash staleness (informational) ───────────────────────────

fn check_views_hash(root: &Path) -> Result<String> {
    let src = root.join("_bmad-output/planning-artifacts/architecture.md");
    let views = root.join("_bmad-output/planning-artifacts/architecture-views.md");
    if !src.exists() || !views.exists() {
        return Ok("source or views file missing — skipped".into());
    }
    let hash = sha256_hex(&std::fs::read(&src)?);
    let views_content = std::fs::read_to_string(&views)?;
    match extract_frontmatter_field(&views_content, "sourceSha256") {
        Some(d) if d == hash => Ok(format!("CURRENT ({}…)", &hash[..12])),
        Some(d) => Ok(format!(
            "STALE — regenerate at next milestone (source {}… ≠ declared {}…)",
            &hash[..12],
            &d[..12.min(d.len())]
        )),
        None => Ok("no sourceSha256 in views frontmatter — cannot verify".into()),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn extract_frontmatter_field(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    content.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&prefix)
            .map(|rest| rest.trim().trim_matches(['\'', '"']).trim().to_string())
    })
}

// ── Tests: prove each gate can go RED, not only GREEN (D45) ──────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn copresence_findings(body: &str) -> Vec<&'static str> {
        let body = body.to_lowercase();
        PAIRS
            .iter()
            .filter(|(r, repls)| {
                contains_word(&body, &r.to_lowercase())
                    && !repls
                        .iter()
                        .any(|x| contains_word(&body, &x.to_lowercase()))
            })
            .map(|(r, _)| *r)
            .collect()
    }

    #[test]
    fn vocabulary_reds_on_a_stale_doc() {
        // retired term, replacement nowhere -> the definition of stale -> RED
        assert_eq!(
            copresence_findings("the state is pending_accept, full stop"),
            vec!["pending_accept"]
        );
    }

    #[test]
    fn vocabulary_greens_when_the_replacement_is_present() {
        // a doc that narrates its own rename contains both words -> GREEN
        assert!(copresence_findings("renamed pending_accept to pending_commit").is_empty());
    }

    #[test]
    fn word_boundary_ignores_substrings() {
        assert!(!contains_word("this is ignored prose", "ignore")); // "ignored" != "ignore"
        assert!(contains_word("please ignore this", "ignore"));
        assert!(!contains_word("pending_accept_extended", "pending_accept"));
        assert!(contains_word("state = pending_accept;", "pending_accept"));
    }

    #[test]
    fn frontmatter_is_stripped_but_body_rules_survive() {
        let doc =
            "---\ntitle: x\nsourceSha256: abc\n---\nintro\n\n---\n\nbody has pending_accept\n";
        let body = strip_frontmatter(doc);
        assert!(body.contains("pending_accept"));
        assert!(body.contains("intro")); // a --- rule in the body does not truncate it
        assert!(!body.contains("sourceSha256"));
    }

    #[test]
    fn ddl_flags_bare_text_column_and_passes_a_collated_one() {
        assert!(
            text_column_without_binary_collation("  hostname VARCHAR(255) NOT NULL,").is_some()
        );
        assert!(
            text_column_without_binary_collation(
                "  id CHAR(36) CHARACTER SET ascii COLLATE ascii_bin,"
            )
            .is_none()
        );
        assert!(text_column_without_binary_collation("  count INTEGER NOT NULL,").is_none());
        assert!(text_column_without_binary_collation("  -- a comment about TEXT").is_none());
    }

    // ── frontier gate (D47) ──────────────────────────────────────────────

    const CLEAN_CORE: &str = "\
opencmdb-core v0.1.0 (/w/crates/opencmdb-core)
├── chrono v0.4.45
├── serde v1.0.228
├── thiserror v2.0.18
└── uuid v1.24.0";
    const CLEAN_BIN: &str = "\
opencmdb-bin v0.1.0 (/w/crates/opencmdb-bin)
├── anyhow v1.0.103
└── opencmdb-core v0.1.0 (/w/crates/opencmdb-core)";

    #[test]
    fn frontier_flags_a_forbidden_dep_in_core() {
        // A forbidden crate resolved in core's graph -> RED, and the production message
        // NAMES the crate (AC #2). Asserts the real gate path, not a duplicated helper.
        let core = "\
opencmdb-core v0.1.0 (/w/crates/opencmdb-core)
├── anyhow v1.0.103
├── serde v1.0.228
└── uuid v1.24.0";
        assert_eq!(
            frontier_offenders(core, CLEAN_BIN),
            vec!["opencmdb-core depends on forbidden crate 'anyhow'"]
        );
    }

    #[test]
    fn frontier_is_clean_on_the_real_core_deps() {
        // core's actual direct deps (chrono/serde/thiserror/uuid) -> zero findings.
        assert!(frontier_offenders(CLEAN_CORE, CLEAN_BIN).is_empty());
    }

    #[test]
    fn frontier_token_match_rejects_lookalikes() {
        // `anyhow-macros` is its own crate token — it must never read as `anyhow`.
        let core = "\
opencmdb-core v0.1.0 (/w)
└── anyhow-macros v0.1.0";
        assert!(frontier_offenders(core, CLEAN_BIN).is_empty());
        let names = crates_present_in_tree(core);
        assert!(names.contains("anyhow-macros"));
        assert!(!names.contains("anyhow"));
    }

    #[test]
    fn frontier_flags_xtask_in_bin() {
        // xtask is a dependency of nobody (D56) — its presence in bin's tree reds.
        let bin = "\
opencmdb-bin v0.1.0 (/w/crates/opencmdb-bin)
├── anyhow v1.0.103
├── opencmdb-core v0.1.0 (/w/crates/opencmdb-core)
└── xtask v0.1.0 (/w/xtask)";
        assert_eq!(
            frontier_offenders(CLEAN_CORE, bin),
            vec!["opencmdb-bin depends on forbidden crate 'xtask'"]
        );
    }

    #[test]
    fn frontier_flags_xtask_in_core() {
        // "dependency of nobody" holds for core too, not only bin.
        let core = "\
opencmdb-core v0.1.0 (/w/crates/opencmdb-core)
└── xtask v0.1.0 (/w/xtask)";
        assert_eq!(
            frontier_offenders(core, CLEAN_BIN),
            vec!["opencmdb-core depends on forbidden crate 'xtask'"]
        );
    }

    #[test]
    fn frontier_glyph_stripping_extracts_the_name() {
        // The crate name survives every tree-drawing prefix cargo emits.
        let names =
            crates_present_in_tree("root v0.1.0\n├── a v1.0.0\n│   └── b v2.0.0\n└── c v3.0.0 (*)");
        for expected in ["root", "a", "b", "c"] {
            assert!(names.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn frontmatter_field_extraction() {
        let fm = "---\nsourceSha256: 'deadbeef'\nother: 1\n---\nbody\n";
        assert_eq!(
            extract_frontmatter_field(fm, "sourceSha256").as_deref(),
            Some("deadbeef")
        );
        assert_eq!(extract_frontmatter_field(fm, "missing"), None);
    }
}
