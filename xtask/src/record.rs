//! `cargo xtask record <story-file>` — the story's RECORD, checked against the tree (story 14.4c).
//!
//! 🔑 **Why it exists** (Epic 14's partial retrospective, Guy, 2026-09-19): record defects were found in
//! all seven Epic 14 stories, and story 14.4's fourth review round found defects ONLY in the record —
//! three agent layers checking that a number written in a file equals the real one. *Keep the three
//! layers on the CODE; check the RECORD mechanically.*
//!
//! # The convention it reads — and ONLY that
//!
//! One `## Record` block per story file (a line equal to `## Record`, outside code fences, exactly
//! once), one entry per physical line, every line keyed (an optional leading `- ` is allowed, so the
//! block renders as a list):
//!
//! ```text
//! - live-count: bin=717 core=191 xtask=99
//! - base: <the commit the branch forked from>
//! - registered: <a phrase of each row the branch ADDS to deferred-work.md>
//! - file: <each repo-relative path the branch touches>
//! ```
//!
//! 🔴 **It never reads the story's prose**, on purpose: *a guard that greps a file greps its prose*
//! (14.2, 14.2b, 14.4), and the paragraph explaining a defect would satisfy it. The price is stated
//! rather than hidden — a claim written in prose is not checked. What the command adds for that case
//! is a LIST: it always prints the register rows the branch added, so a reviewer compares a list
//! instead of believing a sentence.
//!
//! # The exit contract — the mutation driver's (story 6.4b)
//!
//! `0` the record matches the tree · `1` it does not, each mismatch named · `2` it could not honestly
//! run: no block or several, a line that parses as nothing, a dirty tree, a `base:` that is not the
//! branch point, a build or a `git` command that failed.
//!
//! ⚠️ **After the squash merge the check is meaningless** — on `master` the merge-base with `master` is
//! `HEAD` and the diff is empty. It runs on the story branch's LAST commit before the merge.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

/// The record matches the tree.
pub(crate) const MATCHES: u8 = 0;
/// The record says something the tree does not.
pub(crate) const MISMATCH: u8 = 1;
/// The check could not honestly run.
pub(crate) const CANNOT_CHECK: u8 = 2;

/// The register a story's `registered:` lines are checked against.
pub(crate) const REGISTER: &str = "_bmad-output/implementation-artifacts/deferred-work.md";

/// The branch every story forks from.
pub(crate) const MAIN_BRANCH: &str = "master";

/// The three test targets the live count names, and the `cargo test` arguments that list each alone.
///
/// 🔴 **Three invocations and not `--workspace`**, measured by the story's validation: under
/// `--workspace` the target NAMES (`Running …`) go to stderr and the COUNTS to stdout, so a capture that
/// concatenates the two can pair them only by position — and two targets are both `src/main.rs`. Each
/// invocation lists exactly one target, which also drops the doc-test target by construction.
pub(crate) const TARGETS: &[(&str, &[&str])] = &[
    (
        "bin",
        &[
            "test",
            "-p",
            "opencmdb-bin",
            "--bins",
            "--locked",
            "--",
            "--list",
        ],
    ),
    (
        "core",
        &[
            "test",
            "-p",
            "opencmdb-core",
            "--lib",
            "--locked",
            "--",
            "--list",
        ],
    ),
    (
        "xtask",
        &["test", "-p", "xtask", "--bins", "--locked", "--", "--list"],
    ),
];

/// A story's `## Record` block, parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Record {
    /// The live count, by target name.
    pub(crate) counts: BTreeMap<String, usize>,
    /// The commit the branch forked from, as written.
    pub(crate) base: String,
    /// One phrase per register row the branch adds.
    pub(crate) registered: Vec<String>,
    /// Every path the branch touches.
    pub(crate) files: BTreeSet<String>,
}

/// Collapse every run of whitespace to one space — register rows wrap at ~100 columns, so a phrase
/// that spans a line break is invisible to a literal search (story 5.14b's split-needle trap,
/// measured again by this story's validation on a real row).
pub(crate) fn normalised(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Find the ONE `## Record` block and parse it.
///
/// # Errors
///
/// A sentence saying why the block cannot be read — zero blocks, several, a line that is not one of
/// the four keys, a malformed count, a missing `base:` — which the caller turns into exit `2`.
pub(crate) fn parse_block(story: &str) -> std::result::Result<Record, String> {
    let lines: Vec<&str> = story.lines().collect();
    let mut in_fence = false;
    let mut headings = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence && line.trim_end() == "## Record" {
            headings.push(at);
        }
    }
    let start = match headings.as_slice() {
        [one] => one + 1,
        [] => return Err("no `## Record` block — this story's record cannot be checked".into()),
        many => {
            return Err(format!(
                "{} `## Record` headings — the block must be unique, or which one is the record?",
                many.len()
            ));
        }
    };
    let mut counts = None;
    let mut base = None;
    let mut registered = Vec::new();
    let mut files = BTreeSet::new();
    for line in lines[start..].iter().take_while(|l| !l.starts_with('#')) {
        let entry = line.trim();
        if entry.is_empty() {
            continue;
        }
        let entry = entry.strip_prefix("- ").unwrap_or(entry);
        let Some((key, value)) = entry.split_once(':') else {
            return Err(format!("the block line {line:?} is not `key: value`"));
        };
        let value = value.trim().trim_matches('`').trim();
        match key.trim() {
            "live-count" => counts = Some(parse_counts(value)?),
            "base" => base = Some(value.to_string()),
            "registered" => registered.push(normalised(value)),
            "file" => {
                files.insert(value.to_string());
            }
            other => {
                return Err(format!(
                    "the block key {other:?} is not one this checker knows"
                ));
            }
        }
    }
    Ok(Record {
        counts: counts.ok_or("the block carries no `live-count:`")?,
        base: base.ok_or("the block carries no `base:`")?,
        registered,
        files,
    })
}

/// Parse `bin=717 core=191 xtask=99`.
fn parse_counts(value: &str) -> std::result::Result<BTreeMap<String, usize>, String> {
    let mut counts = BTreeMap::new();
    for pair in value.split_whitespace() {
        let Some((name, count)) = pair.split_once('=') else {
            return Err(format!("`{pair}` in `live-count:` is not `target=count`"));
        };
        let count = count
            .parse()
            .map_err(|_| format!("`{pair}` in `live-count:` does not end in a number"))?;
        counts.insert(name.to_string(), count);
    }
    Ok(counts)
}

/// Read the `N tests, M benchmarks` line of one `-- --list` run.
pub(crate) fn listed_count(stdout: &str) -> Option<usize> {
    let mut found = stdout
        .lines()
        .filter_map(|line| line.strip_suffix(" benchmarks"))
        .filter_map(|line| line.split_once(" test"))
        .filter_map(|(count, _)| count.trim().parse().ok());
    let first = found.next()?;
    found.next().is_none().then_some(first)
}

/// The register's ROWS: a line starting `- ` at column 0 and its indented continuation lines, each
/// row normalised. Everything else — headings, prose — is not a row.
pub(crate) fn register_rows(register: &str) -> Vec<String> {
    let mut rows: Vec<String> = Vec::new();
    let mut current: Option<String> = None;
    for line in register.lines() {
        if let Some(rest) = line.strip_prefix("- ") {
            if let Some(row) = current.take() {
                rows.push(normalised(&row));
            }
            current = Some(rest.to_string());
        } else if line.starts_with("  ") && current.is_some() {
            if let Some(row) = current.as_mut() {
                row.push(' ');
                row.push_str(line.trim());
            }
        } else if let Some(row) = current.take() {
            rows.push(normalised(&row));
        }
    }
    if let Some(row) = current {
        rows.push(normalised(&row));
    }
    rows
}

/// Compare the `registered:` phrases with the register at the base and at `HEAD`.
///
/// 🔴 **Each rule closes a lie the story's validation BUILT and measured passing the first draft**:
/// a phrase must be ABSENT from the base register (or it names an OLD row — measured exit 0 before);
/// present in EXACTLY ONE row at `HEAD`, a row of its own (so a one-letter phrase, which occurs
/// everywhere, is refused); and the number of phrases must EQUAL the rows the branch added, NET —
/// counting `+- ` diff lines counted an edited row as a registration (measured: `031e2d7`, three old
/// rows re-marked ✅ read as three new ones).
///
/// Returns every mismatch as a sentence; empty means the registrations hold.
pub(crate) fn registration_mismatches(
    phrases: &[String],
    base_register: &str,
    head_register: &str,
) -> Vec<String> {
    let base = normalised(base_register);
    let base_rows = register_rows(base_register);
    let head_rows = register_rows(head_register);
    let mut mismatches = Vec::new();
    let mut claimed_rows = BTreeSet::new();
    for phrase in phrases {
        if base.contains(phrase.as_str()) {
            mismatches.push(format!(
                "`registered: {phrase}` is already in the register at the base — it names a row \
                 this branch did not add"
            ));
            continue;
        }
        let holding: Vec<usize> = head_rows
            .iter()
            .enumerate()
            .filter(|(_, row)| row.contains(phrase.as_str()))
            .map(|(at, _)| at)
            .collect();
        match holding.as_slice() {
            [one] => {
                if !claimed_rows.insert(*one) {
                    mismatches.push(format!(
                        "`registered: {phrase}` names a row another `registered:` line already \
                         claims — two claims, one row"
                    ));
                }
            }
            [] => mismatches.push(format!(
                "`registered: {phrase}` is in no row of the register — a registration with no row"
            )),
            many => mismatches.push(format!(
                "`registered: {phrase}` occurs in {} rows — a phrase must name ONE row",
                many.len()
            )),
        }
    }
    let added = head_rows.len().saturating_sub(base_rows.len());
    if phrases.len() != added {
        mismatches.push(format!(
            "{} `registered:` line(s) for {added} row(s) the branch added to the register (net)",
            phrases.len()
        ));
    }
    mismatches
}

/// Compare the `file:` lines with the paths the branch touches, in BOTH directions.
pub(crate) fn file_list_mismatches(
    listed: &BTreeSet<String>,
    touched: &BTreeSet<String>,
) -> Vec<String> {
    let mut mismatches: Vec<String> = touched
        .difference(listed)
        .map(|path| format!("`{path}` is touched by the branch and not in the File List"))
        .collect();
    mismatches.extend(
        listed
            .difference(touched)
            .map(|path| format!("`{path}` is in the File List and not touched by the branch")),
    );
    mismatches
}

/// Everything the check needs from outside the story file — a SEAM, so the end-to-end test drives it
/// over a scratch repository and a scratch crate instead of this workspace.
///
/// ⚠️ The target directory is passed to the child `cargo` with `Command::env`, never set in this
/// process: the mutation driver's end-to-end already sets `CARGO_TARGET_DIR` process-wide, and two
/// tests doing that race each other.
pub(crate) struct Checked<'a> {
    /// The repository and workspace root.
    pub(crate) root: &'a Path,
    /// The live count's targets and how to list each.
    pub(crate) targets: &'a [(&'a str, &'a [&'a str])],
    /// Where the child `cargo` builds, when not its default.
    pub(crate) target_dir: Option<PathBuf>,
}

/// Run a command in `root` and answer its stdout, or an error naming it.
fn output(root: &Path, program: &str, args: &[&str], target_dir: Option<&Path>) -> Result<String> {
    let mut command = Command::new(program);
    command.args(args).current_dir(root);
    if let Some(dir) = target_dir {
        command.env("CARGO_TARGET_DIR", dir);
    }
    let out = command
        .output()
        .with_context(|| format!("starting `{program} {}`", args.join(" ")))?;
    if !out.status.success() {
        bail!(
            "`{program} {}` exited {:?}: {}",
            args.join(" "),
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Check one story's record against the tree. Prints every mismatch and the register rows the branch
/// added; answers the exit code.
///
/// # Errors
///
/// A command that could not run at all — answered by the caller as `2`.
pub(crate) fn check(story_path: &Path, env: &Checked<'_>) -> Result<u8> {
    let git = |args: &[&str]| output(env.root, "git", args, None);
    let story = std::fs::read_to_string(env.root.join(story_path))
        .with_context(|| format!("reading {}", story_path.display()))?;
    let record = match parse_block(&story) {
        Ok(record) => record,
        Err(why) => {
            println!("🔴 CANNOT CHECK: {why}");
            return Ok(CANNOT_CHECK);
        }
    };
    // 🔴 A DIRTY TREE makes the three checks read three different trees: `--list` compiles the
    // working tree while `git diff` reads commits (the validation measured the story's own untracked
    // file reddening "listed and not touched").
    if !git(&["status", "--porcelain"])?.trim().is_empty() {
        println!("🔴 CANNOT CHECK: the tree is dirty — commit first, so every check reads HEAD");
        return Ok(CANNOT_CHECK);
    }
    // 🔴 THE BASE IS COMPUTED, never taken on trust: a `base:` one commit up the branch shrank both
    // diffs and hid a touched file — measured exit 0 on the first draft.
    let fork = git(&["merge-base", "HEAD", MAIN_BRANCH])?
        .trim()
        .to_string();
    let written = git(&[
        "rev-parse",
        "--verify",
        &format!("{}^{{commit}}", record.base),
    ])
    .map(|sha| sha.trim().to_string())
    .unwrap_or_default();
    if written != fork {
        println!(
            "🔴 CANNOT CHECK: `base: {}` is not the branch point with `{MAIN_BRANCH}` ({fork}) — a \
             later base hides what the branch touched first. (A branch that merged `{MAIN_BRANCH}` \
             into itself moved its branch point: rebase, do not merge.)",
            record.base
        );
        return Ok(CANNOT_CHECK);
    }

    let mut mismatches = Vec::new();
    let names: BTreeSet<&str> = env.targets.iter().map(|(name, _)| *name).collect();
    let written_names: BTreeSet<&str> = record.counts.keys().map(String::as_str).collect();
    if names != written_names {
        mismatches.push(format!(
            "`live-count:` names {written_names:?} where the targets are {names:?}"
        ));
    }
    for (name, args) in env.targets {
        let listed = output(env.root, "cargo", args, env.target_dir.as_deref())?;
        let Some(real) = listed_count(&listed) else {
            println!(
                "🔴 CANNOT CHECK: `cargo {}` printed no single count line",
                args.join(" ")
            );
            return Ok(CANNOT_CHECK);
        };
        match record.counts.get(*name) {
            Some(claimed) if *claimed == real => {}
            Some(claimed) => mismatches.push(format!(
                "the live count says {name}={claimed}; the tree lists {name}={real}"
            )),
            None => {}
        }
    }

    let range = format!("{fork}...HEAD");
    let touched: BTreeSet<String> = git(&["diff", "--no-renames", "--name-only", &range])?
        .lines()
        .map(str::to_string)
        .collect();
    mismatches.extend(file_list_mismatches(&record.files, &touched));

    let at = |rev: &str| git(&["show", &format!("{rev}:{REGISTER}")]).unwrap_or_default();
    let (base_register, head_register) = (at(&fork), at("HEAD"));
    let base_rows: BTreeSet<String> = register_rows(&base_register).into_iter().collect();
    println!(
        "rows this branch added OR CHANGED in the register (an edited row reads as new text; the \
         claims are checked against the NET count of rows added):"
    );
    for row in register_rows(&head_register)
        .iter()
        .filter(|row| !base_rows.contains(*row))
    {
        println!("   + {}", &row[..row.len().min(140)]);
    }
    mismatches.extend(registration_mismatches(
        &record.registered,
        &base_register,
        &head_register,
    ));

    if mismatches.is_empty() {
        println!("✅ the record matches the tree");
        Ok(MATCHES)
    } else {
        for mismatch in &mismatches {
            println!("🔴 {mismatch}");
        }
        Ok(MISMATCH)
    }
}

/// The subcommand's entry point.
///
/// # Errors
///
/// A missing argument, or a command that could not run — both `2` for the caller.
pub(crate) fn from_args(args: &[String], root: &Path) -> Result<u8> {
    let [story] = args else {
        bail!("usage: cargo xtask record <story-file>");
    };
    check(
        Path::new(story),
        &Checked {
            root,
            targets: TARGETS,
            target_dir: None,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCK: &str = "# Story\n\nprose that says registered and 999 tests\n\n## Record\n\n\
        - live-count: bin=717 core=191 xtask=99\n- base: abc123\n\
        - registered: brand new row about widgets\n- file: xtask/src/record.rs\n\n\
        ## Dev Agent Record\n";

    #[test]
    fn the_block_is_read_and_the_prose_is_not() {
        let record = parse_block(BLOCK).expect("a readable block");
        assert_eq!(record.counts.get("bin"), Some(&717));
        assert_eq!(record.base, "abc123");
        assert_eq!(
            record.registered,
            vec!["brand new row about widgets".to_string()]
        );
        assert!(record.files.contains("xtask/src/record.rs"));
        // `## Dev Agent Record` is not `## Record`, and neither is a heading inside a code fence.
        let fenced = format!("```\n## Record\n- base: x\n```\n{BLOCK}");
        assert!(
            parse_block(&fenced).is_ok(),
            "a fenced example is not the block"
        );
    }

    #[test]
    fn a_missing_duplicated_or_unkeyed_block_cannot_be_checked() {
        assert!(
            parse_block("# Story\n## Dev Agent Record\n").is_err(),
            "no block"
        );
        assert!(
            parse_block(&format!("{BLOCK}\n## Record\n- base: y\n")).is_err(),
            "two blocks"
        );
        let wrapped = BLOCK.replace(
            "- registered: brand new row about widgets\n",
            "- registered: a phrase that wraps\n  onto a second line\n",
        );
        assert!(
            parse_block(&wrapped).is_err(),
            "a wrapped entry is refused, not read as another key"
        );
        assert!(parse_block(&BLOCK.replace("xtask=99", "xtask=lots")).is_err());
        assert!(parse_block(&BLOCK.replace("- base: abc123\n", "")).is_err());
    }

    #[test]
    fn one_list_run_gives_one_count() {
        assert_eq!(
            listed_count("a: test\n717 tests, 0 benchmarks\n"),
            Some(717)
        );
        assert_eq!(listed_count("1 test, 0 benchmarks\n"), Some(1));
        assert_eq!(
            listed_count("717 tests, 0 benchmarks\n1 test, 0 benchmarks\n"),
            None,
            "two targets in one run cannot be told apart — refused"
        );
    }

    const BASE_REGISTER: &str =
        "## Old\n\n- ⚠️ **An old row about gadgets** owned by\n  someone.\n";

    fn head_register(extra: &str) -> String {
        format!("{BASE_REGISTER}\n## New\n\n{extra}")
    }

    #[test]
    fn a_new_row_claimed_once_holds() {
        let head = head_register("- ⚠️ **A brand new row about\n  widgets**, owned by 14.5.\n");
        assert!(
            registration_mismatches(
                &[normalised("brand new row about widgets")],
                BASE_REGISTER,
                &head
            )
            .is_empty(),
            "a phrase split by a wrap is found once whitespace is normalised"
        );
    }

    #[test]
    fn the_lies_the_validation_built_are_refused() {
        let head = head_register("- ⚠️ **A brand new row about widgets**, owned by 14.5.\n");
        let old =
            registration_mismatches(&[normalised("old row about gadgets")], BASE_REGISTER, &head);
        assert!(
            old.iter()
                .any(|m| m.contains("already in the register at the base")),
            "a phrase naming an OLD row: {old:?}"
        );
        let short = registration_mismatches(&["o".to_string()], BASE_REGISTER, &head);
        assert!(
            !short.is_empty(),
            "a one-letter phrase names no single new row: {short:?}"
        );
        let absent =
            registration_mismatches(&[normalised("a row nobody wrote")], BASE_REGISTER, &head);
        assert!(absent.iter().any(|m| m.contains("in no row")), "{absent:?}");
        let unclaimed = registration_mismatches(&[], BASE_REGISTER, &head);
        assert!(
            unclaimed
                .iter()
                .any(|m| m.contains("0 `registered:` line(s) for 1 row")),
            "an added row nobody claimed: {unclaimed:?}"
        );
    }

    #[test]
    fn an_edited_row_is_not_a_registration() {
        // The old row re-marked ✅ — an EDIT, which counting `+- ` diff lines read as a new row.
        let head = BASE_REGISTER.replace("An old row", "✅ CLOSED — An old row");
        assert!(
            registration_mismatches(&[normalised("A brand new row")], BASE_REGISTER, &head)
                .iter()
                .any(|m| m.contains("for 0 row(s)")),
            "the edit adds no row, so a claim of one reds"
        );
        assert!(registration_mismatches(&[], BASE_REGISTER, &head).is_empty());
    }

    #[test]
    fn the_file_list_is_compared_in_both_directions() {
        let set = |items: &[&str]| items.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>();
        let mismatches = file_list_mismatches(&set(&["a.rs", "c.rs"]), &set(&["a.rs", "b.rs"]));
        assert_eq!(mismatches.len(), 2);
        assert!(mismatches[0].contains("`b.rs` is touched"));
        assert!(mismatches[1].contains("`c.rs` is in the File List and not touched"));
    }

    /// 🔴 **END TO END through `check`, over a scratch repository and a scratch crate** — story
    /// 5.12's lesson: a checker whose helpers are tested and whose body is not is carried by nothing.
    /// One honest record exits 0; each planted lie exits what the contract says.
    #[test]
    fn the_checker_drives_a_real_repository_end_to_end() {
        let root = std::env::temp_dir().join(format!("xtask-record-e2e-{}", std::process::id()));
        let target =
            std::env::temp_dir().join(format!("xtask-record-target-{}", std::process::id()));
        std::fs::remove_dir_all(&root).ok();
        std::fs::create_dir_all(root.join("src")).expect("scratch");
        std::fs::create_dir_all(root.join("_bmad-output/implementation-artifacts"))
            .expect("scratch");
        let git = |args: &[&str]| {
            let status = Command::new("git")
                .args([
                    "-c",
                    "user.name=t",
                    "-c",
                    "user.email=t@t",
                    "-c",
                    "commit.gpgsign=false",
                ])
                .args(args)
                .current_dir(&root)
                .output()
                .expect("git runs");
            assert!(status.status.success(), "git {args:?}: {status:?}");
            String::from_utf8_lossy(&status.stdout).trim().to_string()
        };
        let write = |path: &str, text: &str| std::fs::write(root.join(path), text).expect("write");
        let lib = |tests: usize| {
            let mut text = String::from("pub fn one() -> u8 { 1 }\n#[cfg(test)]\nmod t {\n");
            for n in 0..tests {
                text.push_str(&format!(
                    "  #[test] fn t{n}() {{ assert_eq!(super::one(), 1); }}\n"
                ));
            }
            text + "}\n"
        };
        write(
            "Cargo.toml",
            "[package]\nname = \"subject\"\nversion = \"0.0.0\"\nedition = \"2021\"\n[workspace]\n",
        );
        write(
            "Cargo.lock",
            "version = 3\n\n[[package]]\nname = \"subject\"\nversion = \"0.0.0\"\n",
        );
        write("src/lib.rs", &lib(2));
        write(REGISTER, BASE_REGISTER);
        git(&["init", "-q", "-b", MAIN_BRANCH]);
        git(&["add", "-A"]);
        git(&["commit", "-q", "-m", "base"]);
        let fork = git(&["rev-parse", "HEAD"]);
        git(&["switch", "-q", "-c", "story"]);
        write("src/lib.rs", &lib(3));
        write(
            REGISTER,
            &head_register("- ⚠️ **A brand new row about widgets**, owned by 14.5.\n"),
        );
        let story = |count: usize, base: &str, registered: &str, files: &[&str]| {
            let mut text = format!(
                "# Story\n\n## Record\n\n- live-count: lib={count}\n- base: {base}\n\
                 - registered: {registered}\n"
            );
            for file in files {
                text.push_str(&format!("- file: {file}\n"));
            }
            text
        };
        let all = ["src/lib.rs", REGISTER, "story.md"];
        write(
            "story.md",
            &story(3, &fork, "brand new row about widgets", &all),
        );
        git(&["add", "-A"]);
        git(&["commit", "-q", "-m", "the story"]);
        let targets: &[(&str, &[&str])] =
            &[("lib", &["test", "--lib", "--locked", "--", "--list"])];
        let checked = Checked {
            root: &root,
            targets,
            target_dir: Some(target.clone()),
        };
        let run = || check(Path::new("story.md"), &checked).expect("the checker runs");
        assert_eq!(run(), MATCHES, "the honest record matches the tree");

        // Each plant is committed on its own, measured, then undone by a fresh commit of the honest
        // record — so the tree is never dirty when `check` runs (a dirty tree is its own refusal).
        let plant = |text: String, expected: u8, why: &str| {
            write("story.md", &text);
            git(&["commit", "-q", "-am", why]);
            assert_eq!(run(), expected, "{why}");
            write(
                "story.md",
                &story(3, &fork, "brand new row about widgets", &all),
            );
            git(&["commit", "-q", "-am", "restore"]);
        };
        plant(
            story(2, &fork, "brand new row about widgets", &all),
            MISMATCH,
            "a wrong count",
        );
        plant(
            story(3, &fork, "a row nobody wrote", &all),
            MISMATCH,
            "a registration with no row",
        );
        plant(
            story(
                3,
                &fork,
                "brand new row about widgets",
                &["src/lib.rs", "story.md"],
            ),
            MISMATCH,
            "a File List missing a touched file",
        );
        // 🔴 The phrase keeps the row's own CASE: written `an old row …` it matched no row at all, and
        // the plant reddened through the *"in no row"* rule instead of the one it names — mutation X4
        // measured it (red 1 where 2 was predicted). *A plant named for one thing, carried by another.*
        plant(
            story(3, &fork, "old row about gadgets", &all),
            MISMATCH,
            "a phrase naming an OLD row",
        );
        let later = git(&["rev-parse", "HEAD"]);
        plant(
            story(3, &later, "brand new row about widgets", &all),
            CANNOT_CHECK,
            "base = a later commit",
        );
        plant("# Story, no block\n".to_string(), CANNOT_CHECK, "no block");

        write("src/lib.rs", &lib(4));
        assert_eq!(run(), CANNOT_CHECK, "a dirty tree cannot be checked");

        std::fs::remove_dir_all(&root).ok();
        std::fs::remove_dir_all(&target).ok();
    }
}
