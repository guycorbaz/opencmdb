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

    let (g3, m3f) = gate_fixture_manifest(&root)?;
    report("fixtures", g3, &m3f);
    ok &= g3;

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
}
