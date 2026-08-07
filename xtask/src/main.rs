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
//!   - **fixtures** (D56): a lockfile for data, checked in BOTH directions — a listed
//!     artefact whose bytes changed is red, and a file present under `fixtures/` that
//!     nobody listed is red. `fixtures/MANIFEST.toml` carries sha256 + optional generator.
//!   - **file-size**: no source file over 2000 CODE lines (tests excluded — the count
//!     stops at the first top-level `#[cfg(test)]`, D56b). A file past the ceiling is
//!     split into modules, not grown. Names the offender and its count.
//!   - **float-free** (D13): no `f32`, no `f64` and no float literal in CODE under
//!     `crates/opencmdb-core/src/identity/` — *"if the output is a float, B has won in
//!     disguise"*. Comments are stripped first, so the architecture may be QUOTED there;
//!     the committed citation in `cascade.rs` is this gate's negative test case.
//!   - **views-hash** (informational): whether `architecture-views.md`'s `sourceSha256`
//!     still matches `architecture.md`. A mismatch means the views file is stale and
//!     should be regenerated at the next milestone — reported, never a hard failure.

// Documentation is a project rule (CLAUDE.md): every public item carries a doc comment.
// `warn` for now, graduating to `-D missing_docs` once the tree is clean.
#![deny(missing_docs)]

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

// ── Gate 4: file size (D56b) ────────────────────────────────────────────────
//
// ⚠️ This section sits ABOVE Gate 0 rather than between Gates 3 and 5, which is where its number
// says it belongs. That placement predates story 5.4b and is left alone deliberately: `run_ci` runs
// the gates 0,1,2,3,4,5 and every OTHER section is now in that order, so the one exception is
// visible instead of being hidden by renumbering.

/// The CODE-line ceiling per source file. Tests do not count (see [`code_line_count`]): the concern
/// is a module doing too much, not a well-covered one, and the house convention keeps tests inline
/// beside the code (D56b). A file that grows past this is asking to be split into modules.
const MAX_CODE_LINES: usize = 2000;

/// Count a Rust file's CODE lines — everything before the first top-level `#[cfg(test)]`.
///
/// The house convention (D56b) is exactly one trailing `#[cfg(test)] mod tests { … }` per file, so
/// "code" is the prefix up to that attribute; a file with no test module counts in full. This is a
/// heuristic, deliberately: it cannot be fooled by the convention it enforces, and a file that puts
/// test code ABOVE the marker would over-count itself (fail earlier), never under-count.
fn code_line_count(source: &str) -> usize {
    match source
        .lines()
        .position(|line| line.trim_start().starts_with("#[cfg(test)]"))
    {
        Some(at) => at,
        None => source.lines().count(),
    }
}

/// Gate: no source file exceeds [`MAX_CODE_LINES`] lines of non-test code (D56b file-size rule).
///
/// Walks `crates/*/src` and `xtask/src` for `.rs` files. Returns `(green, message)`; RED names each
/// offending file with its code-line count, so the fix — extract a module — is obvious.
fn gate_file_size(root: &Path) -> Result<(bool, String)> {
    let mut over: Vec<(String, usize)> = Vec::new();
    let mut checked = 0usize;
    let mut largest = 0usize;

    for base in [root.join("crates"), root.join("xtask/src")] {
        if !base.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&base)
            .into_iter()
            .filter_map(Result::ok)
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let source =
                std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?;
            let lines = code_line_count(&source);
            checked += 1;
            largest = largest.max(lines);
            if lines > MAX_CODE_LINES {
                let shown = p.strip_prefix(root).unwrap_or(p);
                over.push((shown.display().to_string(), lines));
            }
        }
    }

    if over.is_empty() {
        Ok((
            true,
            format!("{checked} file(s) under {MAX_CODE_LINES} code lines (largest: {largest})"),
        ))
    } else {
        over.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
        let findings: Vec<String> = over
            .iter()
            .map(|(f, n)| format!("{f}: {n} code lines > {MAX_CODE_LINES} — extract a module"))
            .collect();
        Ok((
            false,
            format!(
                "{} file(s) over the ceiling:\n      {}",
                over.len(),
                findings.join("\n      ")
            ),
        ))
    }
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

    let (g3, m3f) = gate_fixture_manifest(&root)?;
    report("fixtures", g3, &m3f);
    ok &= g3;

    let (g4, m4) = gate_file_size(&root)?;
    report("file-size", g4, &m4);
    ok &= g4;

    let (g5, m5) = gate_float_free(&root)?;
    report("float-free", g5, &m5);
    ok &= g5;

    let (g6, m6) = gate_declared_authorship(&root)?;
    report("authorship", g6, &m6);
    ok &= g6;

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

// ── Gate 3: the fixture MANIFEST, a lockfile for data (D56) ─────────────────

/// A lockfile for data (D56). Two directions, and the corpus is frozen only when BOTH hold:
///
/// * **Edited** — every artefact listed in `fixtures/MANIFEST.toml` must still hash to its
///   recorded sha256.
/// * **Added** — every file present under `fixtures/` must be listed. Without this the gate's
///   real guarantee is only "listed files are unchanged", which is not the same claim.
///
/// `fixtures/` lives at the workspace ROOT, outside every crate, so editing a trap reads as
/// "I am changing the spec", not "I am fixing a test" (D45).
fn gate_fixture_manifest(root: &Path) -> Result<(bool, String)> {
    let fixtures = root.join("fixtures");
    let manifest = fixtures.join("MANIFEST.toml");
    // Fail CLOSED in both directions. A corpus with no lock, and a lock with no corpus, are
    // both states this gate exists to forbid — reporting "nothing to check" on the deletion of
    // the thing being guarded is a guarantee the gate does not have.
    if !fixtures.exists() {
        return Ok((
            false,
            "fixtures/ is missing — the corpus this gate guards does not exist".into(),
        ));
    }
    if !manifest.exists() {
        return Ok((
            false,
            "fixtures/ exists but fixtures/MANIFEST.toml is missing — the corpus is unlocked"
                .into(),
        ));
    }
    let text =
        std::fs::read_to_string(&manifest).with_context(|| "reading fixtures/MANIFEST.toml")?;
    let parsed: Manifest = match toml::from_str(&text) {
        Ok(parsed) => parsed,
        // A manifest that does not parse is RED, never "no entries, skipped".
        Err(e) => return Ok((false, format!("fixtures/MANIFEST.toml does not parse: {e}"))),
    };

    let entries = corpus_entries(&fixtures)?;
    let findings = corpus_findings(&parsed, &entries, &|p| read_regular_file(&fixtures, p));

    if findings.is_empty() {
        // Named, not silent: the day a generated artefact enters the corpus, the gate says so.
        let generated = parsed
            .artefact
            .iter()
            .filter(|a| a.generator.is_some())
            .count();
        Ok((
            true,
            format!(
                "{} fixture(s) match their recorded sha256 ({generated} generated, {} hand-authored)",
                parsed.artefact.len(),
                parsed.artefact.len() - generated
            ),
        ))
    } else {
        Ok((
            false,
            format!(
                "{} finding(s):\n      {}",
                findings.len(),
                findings.join("\n      ")
            ),
        ))
    }
}

/// Read a corpus file, refusing anything that is not a regular file.
///
/// `std::fs::read` on a FIFO BLOCKS FOREVER, which would hang the gate rather than fail it —
/// a gate that never returns is worse than one that is wrong.
fn read_regular_file(fixtures: &Path, rel: &str) -> std::io::Result<Vec<u8>> {
    let path = fixtures.join(rel);
    let meta = std::fs::symlink_metadata(&path)?;
    if !meta.is_file() {
        return Err(std::io::Error::other(format!(
            "not a regular file ({:?})",
            meta.file_type()
        )));
    }
    std::fs::read(&path)
}

/// The lock itself. `deny_unknown_fields` throughout: a lockfile that tolerates a misspelled
/// key is not a lock (the rule stories 4.1 and 4.2 established for the corpus it guards).
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    #[serde(default)]
    artefact: Vec<Artefact>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Artefact {
    /// Corpus-relative, e.g. `scenario/replay/minimal.jsonl`.
    path: String,
    sha256: String,
    /// Absent for a hand-authored artefact — which is every artefact today. A format that
    /// could not express "nobody generated this" would be filled with lies.
    #[serde(default)]
    generator: Option<Generator>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Generator {
    name: String,
    version: String,
    /// The seed that reproduces the artefact byte for byte. Absent until a generator exists
    /// (ARCH-24 places it after the engine).
    #[serde(default)]
    seed: Option<u64>,
}

/// One thing found while walking the corpus.
#[derive(Debug, PartialEq, Eq)]
enum CorpusEntry {
    /// A regular file, corpus-relative.
    File(String),
    /// A symlink. Not followed (that would let the corpus reach outside itself), and NOT
    /// silently skipped either — an unlisted file that the gate cannot see is the failure
    /// mode this whole gate exists to prevent.
    Symlink(String),
    /// A path whose bytes are not valid UTF-8. It can never match a manifest entry, so saying
    /// so explicitly beats emitting a `U+FFFD` string no entry can ever equal.
    NotRepresentable(String),
}

/// Everything under `fixtures/`, without following symlinks and without swallowing errors.
///
/// A walk whose failure mode is "quietly saw less of the tree" is not a gate — the defect found
/// in story 4.1's path-discipline test, and strictly worse here.
///
/// Dot-files are skipped (decided 2026-07-21): a `.DS_Store`, a `.gitkeep` or a live editor
/// swap file would otherwise red a local run over files git does not track. **This is scoped to
/// the corpus walk, which is rooted at `fixtures/`.** BMad's `_bmad/`, `_bmad-output/` and
/// `.claude/` live at the REPOSITORY root and are unreachable from here — if this walk is ever
/// re-rooted, that stops being true and the skip list has to be revisited.
fn corpus_entries(fixtures: &Path) -> Result<Vec<CorpusEntry>> {
    let mut entries = Vec::new();
    let walk = walkdir::WalkDir::new(fixtures)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Skip dot-entries, but never the root itself (its own name may start with a dot).
            e.depth() == 0 || !e.file_name().to_str().is_some_and(|n| n.starts_with('.'))
        });
    for entry in walk {
        let entry = entry.with_context(|| "walking fixtures/")?;
        if entry.depth() == 0 {
            continue;
        }
        let rel_path = entry
            .path()
            .strip_prefix(fixtures)
            .with_context(|| "a walked path must sit under fixtures/")?;
        // `to_string_lossy` would map two different byte sequences onto one `U+FFFD` string that
        // no manifest entry can ever equal — a permanent, undiagnosable red.
        let Some(rel) = rel_path.to_str() else {
            entries.push(CorpusEntry::NotRepresentable(
                rel_path.to_string_lossy().into_owned(),
            ));
            continue;
        };
        // Separators are `/` on the platforms this project builds for; a `\` in a path here is
        // a filename BYTE, not a separator, and rewriting it would alias a smuggled `a\b.jsonl`
        // onto a legitimately listed `a/b.jsonl`.
        #[cfg(windows)]
        let rel = rel.replace('\\', "/");
        let rel = rel.to_string();

        let file_type = entry.file_type();
        if file_type.is_symlink() {
            entries.push(CorpusEntry::Symlink(rel));
        } else if file_type.is_file() {
            entries.push(CorpusEntry::File(rel));
        }
        // Directories carry no bytes to lock; anything else (fifo, socket) is caught on read.
    }
    entries.sort_by(|a, b| entry_path(a).cmp(entry_path(b)));
    Ok(entries)
}

fn entry_path(e: &CorpusEntry) -> &str {
    match e {
        CorpusEntry::File(p) | CorpusEntry::Symlink(p) | CorpusEntry::NotRepresentable(p) => p,
    }
}

/// Both directions of the lock, decided over already-gathered inputs so the whole gate is
/// unit-testable without touching a disk (D45).
fn corpus_findings(
    manifest: &Manifest,
    present: &[CorpusEntry],
    read: &dyn Fn(&str) -> std::io::Result<Vec<u8>>,
) -> Vec<String> {
    let mut findings = manifest_findings(&manifest.artefact, read);

    // A lock with zero entries is not a lock. The story holds its own discovery test to this
    // standard (`checked > 0`); the gate must not hold itself to a lower one.
    if manifest.artefact.is_empty() {
        findings.push(
            "fixtures/MANIFEST.toml lists no artefact — a lock with zero entries locks nothing"
                .to_string(),
        );
    }

    // A lockfile with a repeated key is malformed, and the success count would be a number the
    // gate cannot substantiate.
    let listed = manifest.listed_paths();
    if listed.len() != manifest.artefact.len() {
        let mut seen = std::collections::BTreeSet::new();
        for a in &manifest.artefact {
            if !seen.insert(&a.path) {
                findings.push(format!(
                    "fixtures/MANIFEST.toml: '{}' is listed more than once",
                    quote_path(&a.path)
                ));
            }
        }
    }
    findings.extend(orphan_findings(&listed, present));
    findings
}

/// Files present in the corpus that nobody listed, plus what the walk could not classify.
///
/// The exemptions are explicit and are compared on the FILE NAME, not as a string suffix: a
/// `NOT-A-README.md` must not inherit an exemption meant for `README.md`.
///
/// `MANIFEST.toml` is exempt because a lock cannot list itself — recording its own hash would
/// change the file and therefore the hash. That is a second exemption beyond the one the
/// acceptance criterion names; it is unavoidable, and it is recorded rather than assumed.
fn orphan_findings(
    listed: &std::collections::BTreeSet<String>,
    present: &[CorpusEntry],
) -> Vec<String> {
    let mut findings = Vec::new();
    for entry in present {
        let path = entry_path(entry);
        match entry {
            CorpusEntry::Symlink(_) => findings.push(format!(
                "fixtures/{}: is a symlink — the corpus must contain its own bytes, not point at \
                 someone else's",
                quote_path(path)
            )),
            CorpusEntry::NotRepresentable(_) => findings.push(format!(
                "fixtures/{}: the path is not valid UTF-8, so it can never match a manifest entry",
                quote_path(path)
            )),
            CorpusEntry::File(_) => {
                let name = Path::new(path).file_name().and_then(|n| n.to_str());
                let exempt = path == "MANIFEST.toml" || name == Some("README.md");
                if !exempt && !listed.contains(path) {
                    findings.push(format!(
                        "fixtures/{}: present but absent from MANIFEST.toml (orphan)",
                        quote_path(path)
                    ));
                }
            }
        }
    }
    findings
}

/// A path can contain a newline, which would otherwise inject a line into the report that reads
/// like a separate, benign gate finding.
fn quote_path(path: &str) -> String {
    if path.chars().any(|c| c.is_control()) {
        format!("{path:?}")
    } else {
        path.to_string()
    }
}

impl Manifest {
    fn listed_paths(&self) -> std::collections::BTreeSet<String> {
        self.artefact.iter().map(|a| a.path.clone()).collect()
    }
}

/// The edited direction, factored out of I/O so it is unit-tested without touching disk (D45):
/// recompute each artefact's sha256 and name it on a mismatch (compared case-insensitively —
/// `sha256_hex` is lowercase, but a hand-authored manifest may not be); name it too when it is
/// listed but unreadable. `read` resolves a path to its bytes.
fn manifest_findings(
    artefacts: &[Artefact],
    read: &dyn Fn(&str) -> std::io::Result<Vec<u8>>,
) -> Vec<String> {
    let mut findings = Vec::new();
    for artefact in artefacts {
        // A lockfile whose keys can escape the corpus is not a lock — the same containment
        // `fixture_path` applies on the reading side. `\` is checked too: it is a filename byte
        // on Unix, so `..\secret` must not slip through as an ordinary name.
        let p = Path::new(&artefact.path);
        let escapes = p.is_absolute()
            || artefact
                .path
                .split(['/', '\\'])
                .any(|c| c == ".." || c == ".");
        if escapes {
            findings.push(format!(
                "fixtures/MANIFEST.toml: path '{}' escapes the corpus (must be relative, no '..' or '.')",
                quote_path(&artefact.path)
            ));
            continue;
        }
        if artefact.path == "MANIFEST.toml" {
            findings.push(
                "fixtures/MANIFEST.toml: a lock cannot list itself — recording its own hash \
                 changes the file, so the entry can never be satisfied"
                    .to_string(),
            );
            continue;
        }
        // A lock that is itself corrupt and a fixture that was tampered with need opposite
        // repairs, so they must not share a diagnosis.
        if artefact.sha256.len() != 64 || !artefact.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            findings.push(format!(
                "fixtures/{}: recorded sha256 is not 64 hex characters — the LOCK is corrupt, \
                 not the fixture",
                quote_path(&artefact.path)
            ));
            continue;
        }
        // Decided 2026-07-21: a generator record exists to make the artefact REPRODUCIBLE, so
        // recording who generated it while omitting the seed records a provenance claim nobody
        // can check or re-run. This is a validation policy, adopted deliberately here rather
        // than inherited by accident from the story that adds the generator.
        if let Some(g) = &artefact.generator
            && g.seed.is_none()
        {
            findings.push(format!(
                "fixtures/{}: generator '{} {}' records no seed, so the artefact cannot be reproduced",
                quote_path(&artefact.path),
                g.name,
                g.version
            ));
        }
        match read(&artefact.path) {
            Ok(bytes) => {
                let actual = sha256_hex(&bytes);
                if !actual.eq_ignore_ascii_case(&artefact.sha256) {
                    // char-safe prefix: the recorded sha is unvalidated manifest text, so never
                    // byte-slice it (a multi-byte char at the boundary would panic).
                    let e: String = artefact.sha256.chars().take(12).collect();
                    findings.push(format!(
                        "fixtures/{}: sha256 mismatch (manifest {e}… ≠ file {}…)",
                        quote_path(&artefact.path),
                        &actual[..12]
                    ));
                }
            }
            Err(e) => findings.push(format!(
                "fixtures/{}: listed in MANIFEST.toml but unreadable ({})",
                quote_path(&artefact.path),
                e.kind()
            )),
        }
    }
    findings
}

// ── Gate 5: no float under identity/ (D13) ──────────────────────────────────

/// The subtree the float gate guards. D13 refuses a float at the DECISION boundary, and this is
/// where the decision lives.
const IDENTITY_DIR: &str = "crates/opencmdb-core/src/identity";

/// Rust's float type names. `f16`/`f128` are not stable yet; D13's clause is *"no float"*, not "no
/// `f64`", so the gate names all four rather than the two that exist today.
const FLOAT_TYPES: [&str; 4] = ["f32", "f64", "f16", "f128"];

/// The float token, or a float literal, in a line of CODE — comments stripped.
///
/// D13: *"**REFUSED: `rule -> confidence: f64`.** A float compares, averages, thresholds — and we
/// are back to invented weights via the back door. **If the output is a float, B has won in
/// disguise.**"* [architecture.md:956-958].
///
/// # What it catches, and what it does not
///
/// It strips comments (see [`strip_line_comment`]) and then looks for two shapes:
/// a **word-bounded** float type name from [`FLOAT_TYPES`] — a type position, `let x: f64`, `as
/// f64` — and a **float literal**, recognised by [`float_literal_kind`], which is a real
/// tokeniser rather than a substring search: `0.85`, `1e-3`, `1.`, `0.85f64`, `1f32`.
///
/// The word boundary is load-bearing in BOTH directions and each direction was measured: without it
/// `let x: f64` is caught but so is `fn a_f64_never_decides()` and `f32x4::splat(1)`, and an earlier
/// revision of this gate reddened on both. There is no substring fallback: a float type inside a
/// longer identifier is not a float type.
///
/// Known limits, stated because a comment asserting a checkable property gets checked:
/// - a float inside a **block comment** `/* … */` is NOT stripped and reds — a false POSITIVE. None
///   exists under the guarded subtree today; if one appears, that is the sentence to revisit.
/// - `#[doc = "…"]` is not stripped and would red on a float in its text. None exists under the
///   guarded subtree today.
/// - [`strip_line_comment`] tracks double quotes but not `'"'` char literals and not strings that
///   span lines, so a line containing an odd number of `"` can mis-classify a following `//`. None
///   exists under the guarded subtree today.
/// - a **hex float** (`0x1p3`, not stable Rust) is not recognised, and neither is a float built at
///   runtime from integers — a gate over source text cannot see the second one at all.
///
/// ⚠️ **The stripping does far more work than "tolerate one citation", and that was MEASURED.**
/// Removing it makes this gate report many offenders on the committed tree rather than one, because
/// a story reference in prose — `5.4b`, `4.6a`, `4.7a` — is a digit-dot-digit, so the literal rule
/// that catches `let confidence = 0.85;` also catches story numbers written in doc comments. The two
/// features are load-bearing together: the literal rule is what makes the gate catch an untyped
/// weight, and the stripping is what makes the literal rule usable at all. The gap is ASSERTED by
/// `the_stripping_is_what_makes_the_literal_rule_usable` — many offenders on the raw lines, zero on
/// the code parts — rather than quoted here as a figure, because a number in a comment rots and an
/// assertion does not. _(An earlier revision of this doc did quote one, and it was stale within the
/// same story: the doc said 47, the tree it described gave 45, and the tokeniser that replaced the
/// matcher moved the figure again. Three values for one sentence is the argument for asserting
/// instead of quoting.)_
fn line_has_float(line: &str) -> Option<&'static str> {
    let code = strip_line_comment(line);
    for token in FLOAT_TYPES {
        if contains_word(code, token) {
            return Some("float type");
        }
    }
    float_literal_kind(code)
}

/// The code part of a line: everything before a `//` that is not inside a string literal.
///
/// The naive `line.find("//")` this replaces truncated at a `//` **inside** a string, so
/// `let sep = "//"; let c: f64 = 0.85;` was reported clean — the missed float was not in the string,
/// it was the code after it. Measured before the fix.
fn strip_line_comment(line: &str) -> &str {
    let b = line.as_bytes();
    let mut in_string = false;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' if in_string => {
                i += 2;
                continue;
            }
            b'"' => in_string = !in_string,
            b'/' if !in_string && b.get(i + 1) == Some(&b'/') => return &line[..i],
            _ => {}
        }
        i += 1;
    }
    line
}

/// A float literal in a line of code, tokenised rather than pattern-matched.
///
/// A numeric literal starts where a digit is preceded by neither an identifier character nor a `.`
/// — the second exclusion is what keeps a nested tuple field access (`t.0.1`) and a dotted quad
/// (`"192.168.0.1"`) from reading as floats. The body then takes digits and `_`, at most **one**
/// dot, and an optional exponent; a second dot means the token is not a numeric literal at all (an
/// IP address, a version, `1..2`'s range operator). Finally the suffix is read: an empty suffix
/// makes it a float only if a dot or an exponent was seen, an `f32`/`f64`/`f16`/`f128` suffix makes
/// it a float regardless (`1f32`), and any other suffix means this was never a Rust numeric literal
/// (`5.4b` is a story number, `0xFF` is hex) and is not reported.
///
/// # Returns
///
/// The label to show the developer, or `None`. The label distinguishes a suffixed literal from a
/// bare one because the fix differs: one is a declared float, the other is an `f64` by inference.
fn float_literal_kind(code: &str) -> Option<&'static str> {
    let b = code.as_bytes();
    let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut i = 0;
    while i < b.len() {
        if !b[i].is_ascii_digit() || (i > 0 && (ident(b[i - 1]) || b[i - 1] == b'.')) {
            i += 1;
            continue;
        }

        let mut j = i;
        let mut dots = 0;
        let mut exponent = false;
        while j < b.len() {
            match b[j] {
                c if c.is_ascii_digit() => j += 1,
                b'_' => j += 1,
                b'.' if dots == 0 && b.get(j + 1) != Some(&b'.') => {
                    dots += 1;
                    j += 1;
                }
                b'.' => break,
                b'e' | b'E'
                    if !exponent
                        && (matches!(b.get(j + 1), Some(c) if c.is_ascii_digit())
                            || matches!(
                                (b.get(j + 1), b.get(j + 2)),
                                (Some(b'+') | Some(b'-'), Some(c)) if c.is_ascii_digit()
                            )) =>
                {
                    exponent = true;
                    j += if b[j + 1].is_ascii_digit() { 2 } else { 3 };
                }
                _ => break,
            }
        }
        // A second dot means this is not one numeric literal: `192.168.0.1`, `1.2.3`.
        if b.get(j) == Some(&b'.') && matches!(b.get(j + 1), Some(c) if c.is_ascii_digit()) {
            i = j + 1;
            continue;
        }

        let suffix_start = j;
        while j < b.len() && ident(b[j]) {
            j += 1;
        }
        let suffix = code[suffix_start..j].trim_start_matches('_');
        if FLOAT_TYPES.contains(&suffix) {
            return Some("float literal with an f32/f64 suffix");
        }
        if suffix.is_empty() && (dots == 1 || exponent) {
            return Some("bare float literal");
        }
        i = j.max(i + 1);
    }
    None
}

/// No float may reach a decision — D13's clause, held mechanically rather than by accident.
///
/// Before this gate the rule was true by measurement only: the whole workspace contained zero
/// `f32`/`f64` in code. A gate is what keeps it true.
///
/// It walks [`IDENTITY_DIR`] **recursively**, because the architecture's own source tree names a
/// future `identity/field_decision/` [architecture.md:3370-3372] and a flat read would go blind the
/// day it is created.
///
/// It **fails CLOSED** on two conditions, not one: the directory missing, and the directory present
/// but holding no `.rs` file. The DDL gate reports "nothing to check" when its directory is absent,
/// but that directory does not exist yet; this one does, so its disappearance is a finding — the
/// fixture gate's reasoning, *"reporting 'nothing to check' on the deletion of the thing being
/// guarded is a guarantee the gate does not have"*. The second condition is the likelier accident:
/// moving `cascade.rs` to a new module leaves `identity/` standing, and a gate that answers
/// `no float in code across 0 file(s)` to that is reporting a pass over nothing.
fn gate_float_free(root: &Path) -> Result<(bool, String)> {
    let dir = root.join(IDENTITY_DIR);
    if !dir.exists() {
        return Ok((
            false,
            format!(
                "{IDENTITY_DIR} is missing — the guarded subtree must exist for this gate to mean anything"
            ),
        ));
    }

    let mut offenders = Vec::new();
    let mut checked = 0usize;
    for entry in walkdir::WalkDir::new(&dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content =
            std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?;
        checked += 1;
        for (i, line) in content.lines().enumerate() {
            if let Some(what) = line_has_float(line) {
                let shown = p.strip_prefix(root).unwrap_or(p);
                offenders.push(format!("{}:{}: {what}", shown.display(), i + 1));
            }
        }
    }

    if checked == 0 {
        return Ok((
            false,
            format!(
                "{IDENTITY_DIR} holds no .rs file — the guarded code has moved and this gate is \
                 reporting a pass over nothing"
            ),
        ));
    }

    offenders.sort();
    if offenders.is_empty() {
        Ok((
            true,
            format!("no float in code across {checked} file(s) under {IDENTITY_DIR}"),
        ))
    } else {
        Ok((
            false,
            format!(
                "{} float(s) where a decision is made — D13 refuses it:\n      {}",
                offenders.len(),
                offenders.join("\n      ")
            ),
        ))
    }
}

// ── Gate 6: declared authorship (NFR5, story 5.12) ───────────────────────────────────────────

/// The table whose authorship NFR5 protects.
const DECLARED_TABLE: &str = "declared_attribute";

/// The subtrees this gate walks. `docker/` is NOT decoration: `seed-example.sql` writes a declared
/// row and is a file the operator is told to run, so a gate confined to `crates/` would be blind to
/// the only other writer in the product. Measured at story 5.12's validation.
const AUTHORSHIP_ROOTS: [&str; 2] = ["crates", "docker"];

/// The three sanctioned write sites, and nothing else may write a declared field.
///
/// Two are Rust functions, matched by their ENCLOSING `fn`; the third is a data file, matched by
/// path. ⚠️ **The story prescribed TWO sites** — the third is forced by the story's own requirement
/// to walk `docker/`, where `seed-example.sql` writes legitimately, with `'operator'` as its actor.
/// A deviation, stated rather than absorbed.
const SANCTIONED_FNS: [&str; 2] = [
    "insert_declared_attribute",
    "raw_declared_write_for_ddl_test",
];

/// The one data file allowed to write a declared row. See [`SANCTIONED_FNS`].
const SANCTIONED_FILE: &str = "docker/seed-example.sql";

/// The provenance columns a divergence computation may never read (FR13, NFR5's second clause).
///
/// **THREE, not two.** `origin_obs_id` is *"the adopted observation"* — it is *how the value was
/// obtained* at least as much as `origin` is, and story 5.12's first draft named only two.
const PROVENANCE_COLUMNS: [&str; 3] = ["origin_obs_id", "actor_id", "origin"];

/// One file's text, lowercased and whitespace-collapsed, plus the source line of every byte.
///
/// Whole-file rather than per-line — the difference from [`line_has_float`], and it is forced:
/// `INSERT INTO` and the table name may sit on two different lines, and a per-line matcher was
/// measured blind to exactly that. Line comments are stripped so the architecture may be quoted;
/// **block comments are not**, the same known limit `float-free` carries.
fn normalise_sql_text(content: &str, sql_comments: bool) -> (String, Vec<usize>) {
    let mut out = String::with_capacity(content.len());
    let mut lines = Vec::with_capacity(content.len());
    let mut prev_space = true;
    for (idx, raw) in content.lines().enumerate() {
        let code = strip_line_comment(raw);
        let code = if sql_comments {
            strip_sql_comment(code)
        } else {
            code
        };
        for ch in code.chars() {
            if ch.is_whitespace() {
                if !prev_space {
                    out.push(' ');
                    lines.push(idx + 1);
                    prev_space = true;
                }
            } else {
                out.push(ch.to_ascii_lowercase());
                lines.push(idx + 1);
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

/// A `.sql` line with its `--` comment removed, respecting single-quoted literals.
///
/// 🔴 Without this the gate was measured GREEN on a planted migration
/// (`UPDATE declared_attribute SET actor_id = 'engine'`) whose only defence was a `--` header line:
/// the comment ran into the statement under whitespace normalisation, so the fragment no longer
/// BEGAN with its verb. `strip_line_comment` handles `//` alone, which is right for Rust and blind
/// for SQL — and a `.sql` migration is the most natural home for a bulk author rewrite.
fn strip_sql_comment(line: &str) -> &str {
    let b = line.as_bytes();
    let mut in_string = false;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\'' => in_string = !in_string,
            b'-' if !in_string && b.get(i + 1) == Some(&b'-') => return &line[..i],
            _ => {}
        }
        i += 1;
    }
    line
}

/// Is the `declared_attribute` occurrence at `at` a TABLE reference rather than part of a longer
/// identifier?
///
/// `insert_declared_attribute` contains the table's name preceded by `_`; a backtick or a schema dot
/// does NOT disqualify it, which is what makes `` `declared_attribute` `` and
/// `opencmdb.declared_attribute` reachable — both measured green (i.e. invisible) before this.
fn is_table_reference(text: &str, at: usize) -> bool {
    let before = text[..at].chars().next_back();
    let after = text[at + DECLARED_TABLE.len()..].chars().next();
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
fn statement_before(text: &str, at: usize) -> &str {
    let start = text[..at].rfind([';', '"']).map_or(0, |i| i + 1);
    &text[start..at]
}

/// The `fn` a byte offset sits inside, if any — the unit [`SANCTIONED_FNS`] allowlists.
fn enclosing_fn(text: &str, at: usize) -> Option<&str> {
    let head = text[..at].rfind("fn ")?;
    let rest = &text[head + 3..];
    let end = rest.find(|c: char| !(c.is_alphanumeric() || c == '_'))?;
    Some(&rest[..end])
}

/// A projection with every parenthesised group removed.
///
/// ⚠️ Without it `SELECT COUNT(*) FROM declared_attribute` is read as a wildcard — measured on the
/// committed tree at `repo.rs:106`, the gate's first red and a FALSE POSITIVE. An aggregate's star
/// loads no column; `SELECT *` loads all three provenance columns. The distinction is the whole
/// difference between a gate and a nuisance.
fn outside_parens(projection: &str) -> String {
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

/// Every unsanctioned access to `declared_attribute` in one file's text.
///
/// `is_sanctioned_file` short-circuits the write half only: a data file may WRITE with a human
/// author, and no data file participates in the divergence computation.
fn authorship_findings(content: &str, is_sanctioned_file: bool, sql: bool) -> Vec<(usize, String)> {
    let (text, lines) = normalise_sql_text(content, sql);
    let mut findings = Vec::new();
    let mut from = 0usize;

    while let Some(rel) = text[from..].find(DECLARED_TABLE) {
        let at = from + rel;
        from = at + DECLARED_TABLE.len();
        if !is_table_reference(&text, at) {
            continue;
        }
        let stmt = statement_before(&text, at).trim_start();
        let line = lines.get(at).copied().unwrap_or(0);

        // `CREATE TABLE` is the schema's own definition, never a write of a value.
        if stmt.starts_with("create table") {
            continue;
        }

        // ── the WRITE half ──
        // `DELETE` is deliberately absent: NFR5 is about AUTHORSHIP, and a delete writes no author.
        // Including it was measured reddening the committed tree at two test-fixture sites.
        let verb = ["insert into", "insert", "update", "replace into", "replace"]
            .into_iter()
            .find(|v| stmt.starts_with(v));
        if let Some(verb) = verb {
            let sanctioned = is_sanctioned_file
                || enclosing_fn(&text, at).is_some_and(|f| SANCTIONED_FNS.contains(&f));
            if !sanctioned {
                findings.push((
                    line,
                    format!("`{verb} {DECLARED_TABLE}` outside the sanctioned write sites — NFR5"),
                ));
            }
            continue;
        }

        // ── the READ half (FR13: a divergence never consults HOW a value was obtained) ──
        if let Some(projection) = stmt.strip_prefix("select") {
            let projection = projection.split(" from ").next().unwrap_or(projection);
            if outside_parens(projection).contains('*') {
                findings.push((
                    line,
                    format!(
                        "`SELECT *` on {DECLARED_TABLE} loads all three provenance columns — FR13"
                    ),
                ));
            } else if let Some(col) = PROVENANCE_COLUMNS
                .into_iter()
                .find(|c| contains_word(projection, c))
            {
                findings.push((
                    line,
                    format!("a read of {DECLARED_TABLE} names `{col}` — FR13"),
                ));
            }
        }
    }
    findings
}

/// Gate 6 — no code path writes a declared field without a human author, and no divergence
/// computation reads how one was obtained (NFR5, FR13; story 5.12).
///
/// It walks `.rs` **and** `.sql` under [`AUTHORSHIP_ROOTS`]. A `.sql` migration was measured to be
/// the most natural home for a bulk author rewrite and entirely invisible to a `.rs`-only walk.
///
/// # Errors
///
/// If a guarded root is missing or a file cannot be read — this gate fails CLOSED, like its five
/// siblings: a pass reported over nothing is worse than a red.
fn gate_declared_authorship(root: &Path) -> Result<(bool, String)> {
    let mut offenders = Vec::new();
    let mut checked = 0usize;

    for sub in AUTHORSHIP_ROOTS {
        let dir = root.join(sub);
        if !dir.exists() {
            return Ok((
                false,
                format!(
                    "{sub}/ is missing — the guarded subtree must exist for this gate to mean anything"
                ),
            ));
        }
        for entry in walkdir::WalkDir::new(&dir)
            .into_iter()
            .filter_map(Result::ok)
        {
            let p = entry.path();
            let ext = p.extension().and_then(|e| e.to_str());
            if ext != Some("rs") && ext != Some("sql") {
                continue;
            }
            let content =
                std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))?;
            checked += 1;
            let shown = p.strip_prefix(root).unwrap_or(p);
            let shown = shown.display().to_string().replace('\\', "/");
            for (line, what) in
                authorship_findings(&content, shown == SANCTIONED_FILE, ext == Some("sql"))
            {
                offenders.push(format!("{shown}:{line}: {what}"));
            }
        }
    }

    if checked == 0 {
        return Ok((
            false,
            "no .rs or .sql file under the guarded roots — this gate is reporting a pass over nothing"
                .to_string(),
        ));
    }

    offenders.sort();
    if offenders.is_empty() {
        Ok((
            true,
            format!("declared authorship intact across {checked} file(s)"),
        ))
    } else {
        Ok((
            false,
            format!(
                "{} unsanctioned access(es) to {DECLARED_TABLE}:\n      {}",
                offenders.len(),
                offenders.join("\n      ")
            ),
        ))
    }
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

    // The D45 prove-to-red for the `ddl-collation` gate (D64 cond. 1): a bare text column
    // reds, a binary-collated one does not — the gate is trustworthy before any real
    // migration exists to exercise it. Extended below with the other accepted form,
    // the same-column toggle, and the remaining `is_text` variants.
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

    #[test]
    fn ddl_accepts_the_collate_binary_form() {
        // The heuristic accepts `_BIN` OR the literal `COLLATE BINARY`; lock the latter so a
        // regression that drops it cannot pass silently.
        assert!(text_column_without_binary_collation("  note TEXT COLLATE BINARY,").is_none());
        assert!(
            text_column_without_binary_collation("  tag VARCHAR(64) COLLATE BINARY NOT NULL,")
                .is_none()
        );
    }

    #[test]
    fn ddl_same_varchar_column_toggles_on_the_collation() {
        // AC #2 made literal: one column type, the binary collation is the only difference.
        assert!(text_column_without_binary_collation("  email VARCHAR(320) NOT NULL,").is_some());
        assert!(
            text_column_without_binary_collation(
                "  email VARCHAR(320) NOT NULL COLLATE latin1_bin,"
            )
            .is_none()
        );
    }

    #[test]
    fn ddl_flags_text_and_clob_and_char_variants() {
        // Guard the whole `is_text` set, not just VARCHAR: a bare TEXT / CLOB / leading CHAR
        // column each reds.
        assert!(text_column_without_binary_collation("  body TEXT,").is_some());
        assert!(text_column_without_binary_collation("  payload CLOB,").is_some());
        assert!(text_column_without_binary_collation("  CHAR(2) code NOT NULL,").is_some());
        // Known reflex-gate boundary (D53): ENUM/SET carry a collation but are outside the
        // current `is_text` set, so a bare ENUM is NOT flagged. Refining this is a D64
        // concern for when the first real migration is written — not this story.
        assert!(text_column_without_binary_collation("  kind ENUM('a','b') NOT NULL,").is_none());
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

    // ── fixture MANIFEST gate: a lockfile for data (D56) ─────────────────

    fn manifest(toml_text: &str) -> Manifest {
        toml::from_str(toml_text).expect("the test manifest must parse")
    }

    /// sha256("hello") — the fixed vector every byte-level test below is anchored on.
    const HELLO_SHA: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

    fn entry_of(path: &str) -> Vec<CorpusEntry> {
        vec![CorpusEntry::File(path.to_string())]
    }

    /// A private directory per test: a shared constant path races between concurrent runs and
    /// leaves a stale corpus behind when an assertion fails.
    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("opencmdb-xtask-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    #[test]
    fn fixtures_gate_reds_on_a_sha_mismatch() {
        let m = manifest(&format!(
            "[[artefact]]\npath = \"scenario/replay/a.jsonl\"\nsha256 = \"{HELLO_SHA}\"\n"
        ));
        let findings = manifest_findings(&m.artefact, &|_| Ok(b"tampered".to_vec()));
        assert_eq!(findings.len(), 1);
        assert!(
            findings[0].contains("scenario/replay/a.jsonl"),
            "{findings:?}"
        );
        assert!(findings[0].contains("sha256 mismatch"), "{findings:?}");
    }

    #[test]
    fn fixtures_gate_greens_when_bytes_match() {
        let m = manifest(&format!(
            "[[artefact]]\npath = \"a.jsonl\"\nsha256 = \"{HELLO_SHA}\"\n"
        ));
        assert!(manifest_findings(&m.artefact, &|_| Ok(b"hello".to_vec())).is_empty());
    }

    #[test]
    fn fixtures_gate_sha_compare_is_case_insensitive() {
        let m = manifest(&format!(
            "[[artefact]]\npath = \"a.jsonl\"\nsha256 = \"{}\"\n",
            HELLO_SHA.to_uppercase()
        ));
        assert!(manifest_findings(&m.artefact, &|_| Ok(b"hello".to_vec())).is_empty());
    }

    /// The recorded sha is unvalidated text: a multi-byte char at the 12-byte boundary must not
    /// panic the gate, and the finding must still NAME the offender.
    #[test]
    fn fixtures_gate_mismatch_prefix_is_char_safe() {
        let sixty_four_accents = "é".repeat(64);
        let m = manifest(&format!(
            "[[artefact]]\npath = \"traps/d.jsonl\"\nsha256 = \"{sixty_four_accents}\"\n"
        ));
        let findings = manifest_findings(&m.artefact, &|_| Ok(b"hello".to_vec()));
        assert_eq!(findings.len(), 1);
        assert!(findings[0].contains("traps/d.jsonl"), "{findings:?}");
    }

    /// A corrupt lock and a tampered fixture need opposite repairs, so they must not share a
    /// diagnosis.
    #[test]
    fn fixtures_gate_distinguishes_a_corrupt_lock_from_a_changed_fixture() {
        for bad in ["deadbeef", "", &"z".repeat(64)] {
            let m = manifest(&format!(
                "[[artefact]]\npath = \"a.jsonl\"\nsha256 = \"{bad}\"\n"
            ));
            let findings = manifest_findings(&m.artefact, &|_| Ok(b"hello".to_vec()));
            assert_eq!(findings.len(), 1, "{bad:?}");
            assert!(findings[0].contains("LOCK is corrupt"), "{findings:?}");
        }
    }

    #[test]
    fn fixtures_gate_flags_a_missing_file() {
        let m = manifest(&format!(
            "[[artefact]]\npath = \"gone.jsonl\"\nsha256 = \"{HELLO_SHA}\"\n"
        ));
        let findings = manifest_findings(&m.artefact, &|_| {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound))
        });
        assert_eq!(findings.len(), 1);
        assert!(findings[0].contains("gone.jsonl"), "{findings:?}");
        assert!(findings[0].contains("unreadable"), "{findings:?}");
    }

    /// A lockfile whose keys can escape the corpus is not a lock. `\` counts: it is a filename
    /// byte on Unix, not a separator.
    #[test]
    fn fixtures_gate_refuses_a_path_that_escapes_the_corpus() {
        for bad in ["/etc/passwd", "../outside.jsonl", "./a.jsonl", "..\\secret"] {
            let m = manifest(&format!(
                "[[artefact]]\npath = \"{}\"\nsha256 = \"{HELLO_SHA}\"\n",
                bad.replace('\\', "\\\\")
            ));
            let findings = manifest_findings(&m.artefact, &|_| Ok(b"hello".to_vec()));
            assert_eq!(findings.len(), 1, "{bad} must be refused");
            assert!(findings[0].contains("escapes the corpus"), "{findings:?}");
        }
    }

    /// Recording the lock's own hash changes the lock, so such an entry can never be satisfied.
    #[test]
    fn fixtures_gate_refuses_a_manifest_listing_itself() {
        let m = manifest(&format!(
            "[[artefact]]\npath = \"MANIFEST.toml\"\nsha256 = \"{HELLO_SHA}\"\n"
        ));
        let findings = manifest_findings(&m.artefact, &|_| Ok(b"hello".to_vec()));
        assert_eq!(findings.len(), 1);
        assert!(findings[0].contains("cannot list itself"), "{findings:?}");
    }

    #[test]
    fn fixtures_gate_manifest_refuses_an_unknown_key() {
        let bad = format!("[[artefact]]\npath = \"a\"\nsha256 = \"{HELLO_SHA}\"\nsah256 = \"x\"\n");
        assert!(toml::from_str::<Manifest>(&bad).is_err());
    }

    #[test]
    fn fixtures_gate_manifest_records_an_optional_generator() {
        let m = manifest(&format!(
            "[[artefact]]\npath = \"a\"\nsha256 = \"{HELLO_SHA}\"\n\n\
             [[artefact]]\npath = \"b\"\nsha256 = \"{HELLO_SHA}\"\n\
             generator = {{ name = \"xtask gen-fixtures\", version = \"0.1.1\", seed = 42 }}\n"
        ));
        assert!(m.artefact[0].generator.is_none());
        let g = m.artefact[1].generator.as_ref().expect("generator");
        assert_eq!(g.seed, Some(42));
        assert_eq!(g.name, "xtask gen-fixtures");
        assert!(manifest_findings(&m.artefact, &|_| Ok(b"hello".to_vec())).is_empty());
    }

    /// A generator without a seed is a provenance claim nobody can check or re-run.
    #[test]
    fn fixtures_gate_reds_on_a_generator_without_a_seed() {
        let m = manifest(&format!(
            "[[artefact]]\npath = \"a\"\nsha256 = \"{HELLO_SHA}\"\n\
             generator = {{ name = \"xtask gen-fixtures\", version = \"0.1.1\" }}\n"
        ));
        let findings = manifest_findings(&m.artefact, &|_| Ok(b"hello".to_vec()));
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings[0].contains("records no seed"), "{findings:?}");
    }

    /// The drift-in-the-ADD direction: a file nobody listed.
    #[test]
    fn fixtures_gate_reds_on_an_orphan_file() {
        let listed: std::collections::BTreeSet<String> = ["scenario/replay/a.jsonl".to_string()]
            .into_iter()
            .collect();
        let present = vec![
            CorpusEntry::File("scenario/replay/a.jsonl".into()),
            CorpusEntry::File("scenario/traps/sneaked-in.toml".into()),
        ];
        let findings = orphan_findings(&listed, &present);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].contains("sneaked-in.toml"), "{findings:?}");
        assert!(findings[0].contains("orphan"), "{findings:?}");
    }

    /// The exemptions are compared on the FILE NAME, so a near-miss must NOT inherit them.
    /// The previous version of this test only presented the two exempt names, which pinned
    /// "these are exempt" rather than "only these are".
    #[test]
    fn fixtures_gate_orphan_exemptions_do_not_extend_to_near_misses() {
        let listed = std::collections::BTreeSet::new();
        let present: Vec<CorpusEntry> = [
            "MANIFEST.toml",
            "README.md",
            "scenario/README.md",
            // Every one of these must be an orphan.
            "scenario/NOT-A-README.md",
            "scenario/evil-README.md",
            "scenario/README.md.bak",
            "scenario/readme.md",
            "nested/MANIFEST.toml",
        ]
        .into_iter()
        .map(|p| CorpusEntry::File(p.to_string()))
        .collect();
        let findings = orphan_findings(&listed, &present);
        assert_eq!(findings.len(), 5, "{findings:?}");
        for expected in [
            "NOT-A-README.md",
            "evil-README.md",
            "README.md.bak",
            "readme.md",
            "nested/MANIFEST.toml",
        ] {
            assert!(
                findings.iter().any(|f| f.contains(expected)),
                "{expected} must be an orphan: {findings:?}"
            );
        }
    }

    /// A symlink is not followed (it would let the corpus reach outside itself) and not skipped
    /// either — an unlisted file the gate cannot see is the failure this gate exists to prevent.
    #[test]
    fn fixtures_gate_reds_on_a_symlink_even_when_listed() {
        let listed: std::collections::BTreeSet<String> =
            ["link.jsonl".to_string()].into_iter().collect();
        let findings = orphan_findings(&listed, &[CorpusEntry::Symlink("link.jsonl".into())]);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(findings[0].contains("is a symlink"), "{findings:?}");
    }

    #[test]
    fn fixtures_gate_reds_on_a_path_that_is_not_utf8() {
        let findings = orphan_findings(
            &std::collections::BTreeSet::new(),
            &[CorpusEntry::NotRepresentable("bad\u{FFFD}name".into())],
        );
        assert_eq!(findings.len(), 1);
        assert!(findings[0].contains("not valid UTF-8"), "{findings:?}");
    }

    /// A path can carry a newline, which would otherwise inject a line into the report that
    /// reads like a separate, benign gate finding.
    #[test]
    fn fixtures_gate_quotes_a_control_character_in_a_path() {
        let findings = orphan_findings(
            &std::collections::BTreeSet::new(),
            &entry_of("evil\n      all good.jsonl"),
        );
        assert_eq!(findings.len(), 1);
        assert!(!findings[0].contains('\n'), "{findings:?}");
    }

    /// A repeated key makes the success count a number the gate cannot substantiate.
    #[test]
    fn fixtures_gate_reds_on_a_duplicate_manifest_path() {
        let m = manifest(&format!(
            "[[artefact]]\npath = \"a.jsonl\"\nsha256 = \"{HELLO_SHA}\"\n\n\
             [[artefact]]\npath = \"a.jsonl\"\nsha256 = \"{HELLO_SHA}\"\n"
        ));
        let findings = corpus_findings(&m, &entry_of("a.jsonl"), &|_| Ok(b"hello".to_vec()));
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert!(
            findings[0].contains("listed more than once"),
            "{findings:?}"
        );
    }

    // ── the gate itself, against a real corpus on disk ───────────────────
    // Every test above exercises a helper. These exercise `gate_fixture_manifest`, so a wiring
    // mistake between the two directions cannot ship green.

    fn write_corpus(dir: &Path, manifest_body: &str, files: &[(&str, &str)]) {
        let fixtures = dir.join("fixtures");
        std::fs::create_dir_all(&fixtures).unwrap();
        std::fs::write(fixtures.join("MANIFEST.toml"), manifest_body).unwrap();
        for (rel, body) in files {
            let p = fixtures.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, body).unwrap();
        }
    }

    #[test]
    fn gate_greens_on_a_consistent_corpus() {
        let dir = scratch("green");
        write_corpus(
            &dir,
            &format!("[[artefact]]\npath = \"scenario/a.jsonl\"\nsha256 = \"{HELLO_SHA}\"\n"),
            &[("scenario/a.jsonl", "hello")],
        );
        let (ok, msg) = gate_fixture_manifest(&dir).unwrap();
        assert!(ok, "{msg}");
        assert!(msg.contains("1 fixture(s)"), "{msg}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn gate_reds_on_an_orphan_on_disk() {
        let dir = scratch("orphan");
        write_corpus(
            &dir,
            &format!("[[artefact]]\npath = \"scenario/a.jsonl\"\nsha256 = \"{HELLO_SHA}\"\n"),
            &[("scenario/a.jsonl", "hello"), ("scenario/b.jsonl", "hi")],
        );
        let (ok, msg) = gate_fixture_manifest(&dir).unwrap();
        assert!(!ok, "{msg}");
        assert!(msg.contains("scenario/b.jsonl"), "{msg}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Decided 2026-07-21: editor and OS droppings must not red a local run. The skip is scoped
    /// to the corpus walk — BMad's directories live at the repository root, out of reach.
    #[test]
    fn gate_ignores_dot_files_in_the_corpus() {
        let dir = scratch("dotfiles");
        write_corpus(
            &dir,
            &format!("[[artefact]]\npath = \"scenario/a.jsonl\"\nsha256 = \"{HELLO_SHA}\"\n"),
            &[
                ("scenario/a.jsonl", "hello"),
                (".DS_Store", "junk"),
                ("scenario/.a.jsonl.swp", "vim"),
            ],
        );
        let (ok, msg) = gate_fixture_manifest(&dir).unwrap();
        assert!(ok, "{msg}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn gate_reds_when_the_manifest_does_not_parse() {
        let dir = scratch("badtoml");
        write_corpus(&dir, "[[artefact]\nbroken", &[]);
        let (ok, msg) = gate_fixture_manifest(&dir).unwrap();
        assert!(!ok, "{msg}");
        assert!(msg.contains("does not parse"), "{msg}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A corpus with no lock, and a lock with no corpus, are both states the gate forbids.
    #[test]
    fn gate_reds_when_the_corpus_or_its_lock_is_missing() {
        let dir = scratch("missing");
        let (ok, msg) = gate_fixture_manifest(&dir).unwrap();
        assert!(!ok, "deleting the corpus must not report success: {msg}");
        assert!(msg.contains("fixtures/ is missing"), "{msg}");

        std::fs::create_dir_all(dir.join("fixtures")).unwrap();
        let (ok, msg) = gate_fixture_manifest(&dir).unwrap();
        assert!(!ok, "{msg}");
        assert!(msg.contains("unlocked"), "{msg}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A lock with zero entries over a corpus that exists is vacuous — the same standard the
    /// trap-discovery test holds itself to.
    #[test]
    fn gate_reds_on_a_corpus_of_readmes_with_an_empty_lock() {
        let dir = scratch("vacuous");
        write_corpus(&dir, "# header only\n", &[("README.md", "prose")]);
        let (ok, msg) = gate_fixture_manifest(&dir).unwrap();
        assert!(
            !ok,
            "an empty lock over a real corpus must not be green: {msg}"
        );
        std::fs::remove_dir_all(&dir).ok();
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

    // ── The file-size gate (D56b) ────────────────────────────────────────────

    #[test]
    fn code_line_count_excludes_the_trailing_test_module() {
        // Ten code lines, then a test module of arbitrary length — only the ten count.
        let mut src = String::new();
        for i in 0..10 {
            src.push_str(&format!("fn f{i}() {{}}\n"));
        }
        src.push_str("#[cfg(test)]\nmod tests {\n");
        for _ in 0..500 {
            src.push_str("    // a very long test module\n");
        }
        src.push_str("}\n");
        assert_eq!(code_line_count(&src), 10, "the test module must not count");
    }

    #[test]
    fn code_line_count_counts_a_file_with_no_tests_in_full() {
        let src = "fn a() {}\nfn b() {}\nfn c() {}\n";
        assert_eq!(code_line_count(src), 3);
    }

    /// An indented `#[cfg(test)]` — a test module nested inside another item — still marks the
    /// boundary, because `trim_start` sees past the indentation.
    #[test]
    fn code_line_count_finds_an_indented_test_marker() {
        let src = "fn a() {}\n    #[cfg(test)]\n    mod inner { }\n";
        assert_eq!(code_line_count(src), 1);
    }

    /// The gate is GREEN on the real tree today — and, more importantly, it can be shown to fail:
    /// no gate that cannot fail is a gate (the discipline story 1.3 established).
    #[test]
    fn the_gate_is_green_now_and_can_be_shown_red() {
        let root = workspace_root();
        let (green, msg) = gate_file_size(&root).expect("the gate runs");
        assert!(green, "every source file is under the ceiling today: {msg}");
        assert!(msg.contains("under"), "{msg}");

        // Proven red: a synthetic file of 2001 code lines is over the ceiling. Driving the pure
        // counter is enough — `gate_file_size` is `code_line_count` plus a walk and a threshold.
        let huge = "fn x() {}\n".repeat(MAX_CODE_LINES + 1);
        assert!(
            code_line_count(&huge) > MAX_CODE_LINES,
            "a {}-line file must exceed the {MAX_CODE_LINES} ceiling",
            MAX_CODE_LINES + 1
        );
    }

    // ── declared-authorship gate (NFR5 / FR13, story 5.12) ───────────────────────────────────

    /// Wrap a fragment in an unsanctioned Rust function, the shape a future violation would take.
    fn in_unsanctioned_fn(sql: &str) -> String {
        format!("fn some_new_writer() {{\n    sqlx::query(\"{sql}\").execute(c).await?;\n}}\n")
    }

    /// 🔴 §4d — the WRITE evasion table. Every row was measured; four were HOLES in the first
    /// implementation and are closed here.
    #[test]
    fn the_write_matcher_survives_every_measured_evasion() {
        for sql in [
            "INSERT INTO declared_attribute (entity_id) VALUES (?)",
            "insert into declared_attribute (entity_id) VALUES (?)",
            "UPDATE declared_attribute SET actor_id = 'engine'",
            "REPLACE INTO declared_attribute (entity_id) VALUES (?)",
            "INSERT INTO  declared_attribute (entity_id) VALUES (?)",
            // 🔴 The four that were GREEN before this gate walked them properly.
            "INSERT INTO `declared_attribute` (entity_id) VALUES (?)",
            "INSERT INTO opencmdb.declared_attribute (entity_id) VALUES (?)",
            "INSERT declared_attribute (entity_id) VALUES (?)",
            "REPLACE declared_attribute (entity_id) VALUES (?)",
        ] {
            let findings = authorship_findings(&in_unsanctioned_fn(sql), false, false);
            assert!(!findings.is_empty(), "must RED: {sql}");
        }
    }

    /// 🔑 The newline-split case — the one structural difference from `float-free`.
    ///
    /// A per-line matcher is blind to it, which is why this gate normalises the WHOLE file.
    #[test]
    fn a_write_split_across_two_lines_still_reds() {
        let src = "fn w() {\n    let q = \"INSERT INTO\n         declared_attribute (a) VALUES (?)\";\n}\n";
        assert!(
            !authorship_findings(src, false, false).is_empty(),
            "a per-line matcher would miss this, and that is the whole reason for normalising"
        );
    }

    /// The `no` rows of §4d: what must stay green.
    #[test]
    fn the_write_matcher_leaves_the_legitimate_shapes_alone() {
        // The sanctioned adapter.
        let sanctioned = "fn insert_declared_attribute() {\n    sqlx::query(\"INSERT INTO declared_attribute (a) VALUES (?)\");\n}\n";
        assert!(
            authorship_findings(sanctioned, false, false).is_empty(),
            "the adapter is the sanctioned site"
        );

        // The sanctioned test helper.
        let helper = "fn raw_declared_write_for_ddl_test() {\n    sqlx::query(\"INSERT INTO declared_attribute (a) VALUES (?)\");\n}\n";
        assert!(
            authorship_findings(helper, false, false).is_empty(),
            "AC2's raw write has a named home"
        );

        // A data file on the allowlist.
        let seed =
            "INSERT INTO declared_attribute (entity_id, actor_id) VALUES ('x', 'operator');\n";
        assert!(
            authorship_findings(seed, true, true).is_empty(),
            "the seed file writes with a human author"
        );

        // DELETE is deliberately out of the verb list — it writes no author.
        let del = "fn cleanup() {\n    sqlx::query(\"DELETE FROM declared_attribute\");\n}\n";
        assert!(
            authorship_findings(del, false, false).is_empty(),
            "a DELETE writes no author (§4b)"
        );

        // The name in a doc comment.
        let doc =
            "/// Writes to declared_attribute via INSERT INTO declared_attribute.\nfn f() {}\n";
        assert!(
            authorship_findings(doc, false, false).is_empty(),
            "line comments are stripped"
        );

        // The schema's own definition.
        let ddl = "CREATE TABLE declared_attribute (\n  entity_id CHAR(36) NOT NULL\n);\n";
        assert!(
            authorship_findings(ddl, false, true).is_empty(),
            "CREATE TABLE writes no value"
        );

        // The function NAME contains the table name.
        let call = "fn caller() {\n    insert_declared_attribute(pool, a, b, c).await?;\n}\n";
        assert!(
            authorship_findings(call, false, false).is_empty(),
            "not a table reference"
        );
    }

    /// 🔴 §4e — the READ half, including the FALSE POSITIVE the naive matcher produced.
    #[test]
    fn the_read_matcher_names_all_three_provenance_columns_and_the_wildcard() {
        for sql in [
            "SELECT origin FROM declared_attribute",
            "SELECT actor_id FROM declared_attribute",
            // 🔴 The third column, which the story's first draft did not name.
            "SELECT origin_obs_id FROM declared_attribute",
            // 🔴 And the wildcard, which defeats every column-name rule.
            "SELECT * FROM declared_attribute",
        ] {
            let findings = authorship_findings(&in_unsanctioned_fn(sql), false, false);
            assert!(!findings.is_empty(), "must RED: {sql}");
        }
    }

    /// The `no` rows of §4e.
    #[test]
    fn the_read_matcher_leaves_the_sanctioned_read_and_the_aggregate_alone() {
        let ok =
            in_unsanctioned_fn("SELECT entity_id, attr_key, attr_value FROM declared_attribute");
        assert!(
            authorship_findings(&ok, false, false).is_empty(),
            "the divergence's own read"
        );

        // 🔴 Measured on the committed tree as this gate's FIRST red, and it was wrong:
        // an aggregate's star loads no column.
        let agg = in_unsanctioned_fn("SELECT COUNT(*) FROM declared_attribute");
        assert!(
            authorship_findings(&agg, false, false).is_empty(),
            "COUNT(*) is not SELECT * — the gate's first false positive, at repo.rs:106"
        );

        // 🔴 The false positive the naive backward search produced: a bare DELETE inheriting an
        // `origin` from an unrelated string literal above it.
        let phantom = "fn f() {\n    let a = \"INSERT INTO declared_attribute (origin, actor_id) VALUES (?, ?)\";\n    let b = \"DELETE FROM declared_attribute\";\n}\n";
        let findings = authorship_findings(phantom, false, false);
        assert!(
            findings.iter().all(|(_, w)| !w.contains("a read of")),
            "the `\"` bound is what stops a match spanning two literals; got {findings:?}"
        );
    }

    /// 🔑 The `format!` hole, PINNED rather than pretended away (D18).
    ///
    /// A text gate cannot see a table name assembled at runtime. Stating it is the difference
    /// between a known limit and a false promise.
    #[test]
    fn a_table_name_built_at_runtime_is_invisible_and_that_is_stated() {
        let src = "fn sneaky() {\n    let t = format!(\"declared_{}\", \"attribute\");\n    sqlx::query(&format!(\"INSERT INTO {t} (a) VALUES (?)\"));\n}\n";
        assert!(
            authorship_findings(src, false, false).is_empty(),
            "KNOWN LIMIT: a text gate cannot follow a name built at runtime"
        );
    }

    /// 🔴 A planted `.sql` MIGRATION reds — and it did NOT before `strip_sql_comment` existed.
    ///
    /// Measured during story 5.12's mutation pass: the gate walked the file (its count rose from 31
    /// to 32) and found nothing, because the `--` header ran into the statement under whitespace
    /// normalisation and the fragment no longer BEGAN with `update`. A migration is the most natural
    /// home for a bulk author rewrite, so this was the gate's largest remaining blind spot.
    #[test]
    fn a_bulk_author_rewrite_in_a_sql_migration_reds() {
        let sql = "-- a bulk author rewrite, the most natural home for one\n                   UPDATE declared_attribute SET actor_id = 'engine' WHERE origin = 'manual';\n";
        assert!(
            !authorship_findings(sql, false, true).is_empty(),
            "a `--` header must not shield the statement behind it"
        );
        // And the comment stripping must not swallow a legitimate hyphen inside a literal.
        let quoted = "INSERT INTO other_table (note) VALUES ('a -- not a comment');\n";
        assert!(
            authorship_findings(quoted, false, true).is_empty(),
            "a `--` inside a single-quoted literal is data, not a comment"
        );
    }

    /// 🔑 A commented-out write stays GREEN — and this is a DIVERGENCE from the story's prediction.
    ///
    /// Story 5.12 predicted this gate would inherit `float-free`'s block-comment false positive,
    /// since `strip_line_comment` strips `//` only. **Measured: it does not.** The reason is
    /// structural rather than lucky — this gate anchors on the START of a statement
    /// (`statement_before`), so a verb sitting mid-line inside `/* … */` matches no statement head.
    /// `float-free` searches anywhere in a line and therefore cannot make that distinction.
    ///
    /// Green is also the CORRECT answer: a commented-out write is not a code path, and NFR5 is
    /// about code paths. A divergence from a prediction is a finding, so it is recorded here rather
    /// than quietly enjoyed.
    #[test]
    fn a_write_inside_a_block_comment_stays_green() {
        let src = "fn f() {\n    /* INSERT INTO declared_attribute (a) VALUES (?) */\n}\n";
        assert!(
            authorship_findings(src, false, false).is_empty(),
            "a commented-out write is not a code path — and unlike float-free, this gate can tell"
        );
    }

    /// The allowlist must not match by prefix — a third site still reds.
    #[test]
    fn a_third_site_is_not_sanctioned_by_resembling_the_first() {
        let near = "fn insert_declared_attribute_v2() {\n    sqlx::query(\"INSERT INTO declared_attribute (a) VALUES (?)\");\n}\n";
        assert!(
            !authorship_findings(near, false, false).is_empty(),
            "an allowlist that matched by prefix would be float-free's failure again"
        );
    }

    /// The gate is green on the real tree, and it walks both extensions and both roots.
    #[test]
    fn the_authorship_gate_is_green_on_the_real_tree() {
        let root = workspace_root();
        let (ok, msg) = gate_declared_authorship(&root).expect("gate runs");
        assert!(ok, "the committed tree must be clean: {msg}");
        assert!(
            msg.contains("file(s)"),
            "it must say how many it checked: {msg}"
        );
    }

    // ── float-free gate (D13) ────────────────────────────────────────────────────────────────

    /// The line classifier, shape by shape.
    ///
    /// Every case below was MEASURED against the previous revision of this matcher, which searched
    /// for digit-dot-digit and for the bare substrings `f32`/`f64`. Five of them came back wrong:
    /// two legal float spellings passed (`1e-3`, `1.`) and three innocent lines reddened (a dotted
    /// quad, a nested tuple field, an identifier merely containing `f64`). They are pinned
    /// individually rather than by one example because each has its own reason.
    #[test]
    fn float_line_classifier_catches_types_suffixes_and_bare_literals() {
        // A float TYPE, word-bounded.
        assert_eq!(line_has_float("    let _x: f64 = 0.0;"), Some("float type"));
        assert!(line_has_float("fn w(x: f32) -> f32 { x }").is_some());
        assert!(line_has_float("    let y = n as f64;").is_some());

        // A suffix binds to the digits, so there is no word boundary before the `f`.
        assert_eq!(
            line_has_float("    let confidence = 0.85f64;"),
            Some("float literal with an f32/f64 suffix"),
            "a suffixed literal escapes a word-boundary match"
        );
        assert!(line_has_float("    let c = 1f32;").is_some());
        assert!(line_has_float("    let c = 0.5_f64;").is_some());

        // A bare literal carries no f32/f64 token at all, and is an f64 by inference. This is the
        // likeliest shape an invented weight actually takes.
        assert_eq!(
            line_has_float("    let confidence = 0.85;"),
            Some("bare float literal"),
            "a bare decimal literal is a float even with no type named"
        );
        assert!(line_has_float("    let c = 1_000.0;").is_some());

        // The two spellings the digit-dot-digit rule was measured to MISS. Both are `f64`.
        assert!(
            line_has_float("    let confidence = 1e-3;").is_some(),
            "exponent form carries no dot at all and was green before"
        );
        assert!(line_has_float("    let c = 2E10;").is_some());
        assert!(
            line_has_float("    let confidence = 1.;").is_some(),
            "a trailing dot is a float and was green before"
        );
        assert!(line_has_float("    let c = 2. * x;").is_some());

        // Comments are stripped: the architecture may be quoted.
        assert!(
            line_has_float("/// *\"REFUSED: `rule -> confidence: f64`\"* [architecture.md:956]")
                .is_none(),
            "a citation of D13 is prose, not a float"
        );
        assert!(line_has_float("//! the algebra refuses f64 outright").is_none());
        assert!(line_has_float("    let n = 1u32; // not an f64 either").is_none());

        // A `//` INSIDE a string is not a comment. Before the stripper tracked quotes, this line
        // truncated at the literal and the float after it was never seen.
        assert!(
            line_has_float("    let sep = \"//\"; let c: f64 = 0.85;").is_some(),
            "a // inside a string must not hide the code that follows it"
        );
        assert!(line_has_float("    let u = \"http://h/x\"; let c = 0.85;").is_some());

        // Things that merely look like floats. The last three reddened before the tokeniser.
        assert!(
            line_has_float("/// [architecture.md:967-974]").is_none(),
            "a line-number range is not a float"
        );
        assert!(
            line_has_float("    let a = t.0;").is_none(),
            "a tuple field access is not a float"
        );
        assert!(
            line_has_float("    let a = t.0.1;").is_none(),
            "a NESTED tuple field access is not a float either — it reddened before"
        );
        assert!(
            line_has_float("    for i in 1..32 {").is_none(),
            "a range is not a float"
        );
        assert!(line_has_float("    let s: u32 = 42;").is_none());
        assert!(
            line_has_float("        assert_eq!(ip, \"192.168.0.1\");").is_none(),
            "a dotted quad has three dots and is no numeric literal — it reddened before the \
             tokeniser replaced the digit-dot-digit search"
        );
        assert!(
            line_has_float("        let v = \"0.9.0\";").is_none(),
            "a three-part version is not a float"
        );
        assert!(
            line_has_float("    fn a_f64_never_decides() {}").is_none(),
            "an identifier merely CONTAINING f64 is not a float type — it reddened before"
        );
        assert!(
            line_has_float("    let v = f32x4::splat(1);").is_none(),
            "nor is a wider SIMD type whose name starts with f32"
        );
        assert!(
            line_has_float("        let s = \"story 5.4b\";").is_none(),
            "a story number has a suffix that is no Rust suffix, so it is not a literal"
        );
        assert!(
            line_has_float("    let x = 0xFF;").is_none(),
            "a hex integer is not a float"
        );
    }

    /// The stripping is not an optimisation, and this is the assertion that says so.
    ///
    /// Run the same two checks the gate runs, but on the RAW line instead of the code part, and the
    /// real tree yields many offenders where the gate yields none — because every story number and
    /// line-range citation in a doc comment is a digit-dot-digit. The exact figure is deliberately
    /// NOT written down: a count in a comment rots, and this assertion does not.
    #[test]
    fn the_stripping_is_what_makes_the_literal_rule_usable() {
        let root = workspace_root();
        let dir = root.join(IDENTITY_DIR);
        let mut raw_hits = 0usize;
        for entry in walkdir::WalkDir::new(&dir)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let content = std::fs::read_to_string(entry.path()).expect("readable");
            for line in content.lines() {
                let typed = FLOAT_TYPES.iter().any(|t| contains_word(line, t));
                if typed || float_literal_kind(line).is_some() {
                    raw_hits += 1;
                }
            }
        }

        let (green, msg) = gate_float_free(&root).expect("the gate runs");
        assert!(green, "the gate itself is green on the real tree: {msg}");
        assert!(
            raw_hits > 10,
            "without stripping the same checks must flood — they found only {raw_hits}, so either \
             the guarded subtree lost its doc comments or the stripping is no longer load-bearing"
        );
    }

    /// The gate itself, against a temp tree — the walk, the recursion, and the missing directory.
    ///
    /// Testing only [`line_has_float`] would leave all three untested while the gate read as
    /// covered, so this drives `gate_float_free` end to end.
    #[test]
    fn float_gate_walks_recursively_strips_comments_and_fails_closed() {
        let root = scratch("float-gate");
        let guarded = root.join(IDENTITY_DIR);
        let nested = guarded.join("field_decision");
        std::fs::create_dir_all(&nested).expect("nested dir");

        // A quotation of D13, which must NOT red — the regression the stripping exists for.
        std::fs::write(
            guarded.join("cascade.rs"),
            "/// *\"REFUSED: `rule -> confidence: f64`\"* [architecture.md:956-958]\npub fn ok() {}\n",
        )
        .expect("write");
        let (green, msg) = gate_float_free(&root).expect("the gate runs");
        assert!(green, "a float quoted in a doc comment must not red: {msg}");

        // A real float in a NESTED file: proves the walk recurses.
        std::fs::write(nested.join("weights.rs"), "pub fn w() -> f64 { 0.5 }\n").expect("write");
        let (green, msg) = gate_float_free(&root).expect("the gate runs");
        assert!(
            !green,
            "a float in a nested subdirectory must red — a flat read would go blind"
        );
        assert!(
            msg.contains("field_decision/weights.rs"),
            "the message names the offending file: {msg}"
        );

        // A bare literal, in the guarded directory itself.
        std::fs::remove_file(nested.join("weights.rs")).expect("rm");
        std::fs::write(guarded.join("score.rs"), "pub fn w() { let _c = 0.85; }\n").expect("write");
        let (green, msg) = gate_float_free(&root).expect("the gate runs");
        assert!(!green, "a bare float literal must red: {msg}");

        // FAILS CLOSED when the directory still stands but holds no Rust file — the likelier
        // accident of the two, and the one that used to report `0 file(s)` as a PASS.
        std::fs::remove_file(guarded.join("cascade.rs")).expect("rm");
        std::fs::remove_file(guarded.join("score.rs")).expect("rm");
        let (green, msg) = gate_float_free(&root).expect("the gate runs");
        assert!(
            !green,
            "a guarded subtree with no .rs file must RED, not pass over nothing: {msg}"
        );
        assert!(msg.contains("no .rs file"), "{msg}");

        // FAILS CLOSED when the guarded subtree is gone — the fixture gate's reasoning, not the
        // DDL gate's: this directory exists today, so its disappearance is a finding.
        std::fs::remove_dir_all(&guarded).expect("rm -r");
        let (green, msg) = gate_float_free(&root).expect("the gate runs");
        assert!(
            !green,
            "a missing guarded subtree must RED, not report 'nothing to check': {msg}"
        );
        assert!(msg.contains("missing"), "{msg}");

        let _ = std::fs::remove_dir_all(&root);
    }

    /// The gate is GREEN on the real tree — and the committed D13 citation is why that is a claim
    /// worth pinning rather than a tautology.
    ///
    /// `crates/opencmdb-core/src/identity/cascade.rs` quotes *"REFUSED: `rule -> confidence: f64`"*
    /// in a `///` block, and a naive line grep would red on it. This test is what keeps the stripping
    /// from being removed as "an optimisation".
    ///
    /// ⚠️ The premise is pinned to the citation's TEXT, not to the substring `f64`. It was pinned to
    /// the substring first, and that guard went inert inside the same story: `cascade.rs` gained
    /// further `f64` tokens in other doc comments, so deleting the citation left the assertion green
    /// while its message still claimed it would catch that — MEASURED by renaming the citation and
    /// watching the substring form stay green.
    ///
    /// The property the gate actually holds is *"no float token in CODE under the guarded subtree"*,
    /// never *"there is only one `f64` in the workspace"*. **No count is written here on purpose**:
    /// the workspace figure moved the moment this file gained a `FLOAT_TYPES` constant, and the first
    /// draft of this very paragraph quoted the pre-tokeniser number.
    #[test]
    fn float_gate_is_green_on_the_real_tree_despite_the_committed_d13_citation() {
        let root = workspace_root();
        let (green, msg) = gate_float_free(&root).expect("the gate runs");
        assert!(green, "the real tree carries no float in code: {msg}");
        assert!(msg.contains("no float in code"), "{msg}");

        let cascade = root.join(IDENTITY_DIR).join("cascade.rs");
        let source = std::fs::read_to_string(&cascade).expect("cascade.rs is readable");
        assert!(
            source.contains("confidence: f64"),
            "the citation this gate must tolerate has moved or gone — if it was removed on purpose, \
             this test's premise is what needs revisiting"
        );
    }
}
