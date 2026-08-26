//! `cargo xtask mutate` — the mutation driver that cannot lie (story 6.4b).
//!
//! # Why this exists at all
//!
//! 🔴 **Every mutation pass this project ran until 2026-08-26 was driven by a throw-away script
//! written into a scratchpad and deleted with it**, and fifteen recorded defects came out of that:
//! two filters that ran nothing, a `head -8` that turned 18 red into *"unreachable"*, a
//! `git checkout --` that destroyed nine uncommitted keys, a `sed` replacing a string with itself,
//! a restore that left cargo serving a stale artefact, a commit over a red clippy and another over
//! a red suite. **There was no artefact for a fix to land in**, which is why the same defect
//! recurred in six consecutive epics.
//!
//! # What it promises, and what it does not
//!
//! It refuses to report a number it did not honestly obtain. Every outcome below is a REFUSAL
//! rather than a count, and each one was a recorded defect first:
//!
//! - the anchor matched **nothing**, or matched **more than once** (a replace-all mutates the
//!   second oracle as well as the code — measured at 6 red where the honest figure is 14);
//! - the replacement changed **nothing** (`sed` with itself);
//! - the tree did not **compile**, detected by a line-anchored `error[EDDDD]` and never by a bare
//!   `^error`, which cargo prints on *every* red run as `error: test failed`;
//! - the run was **filtered** — measured by cargo's own `filtered out` count, not by counting
//!   flags, because `cargo test -- A B` still runs seven tests of 741 and exits 0;
//! - a test target produced **no `test result:` line at all**, which a summing driver reports as
//!   *"0 passed, 0 failed"*;
//! - the **store** was absent, which changes the verdict while leaving the counts identical.
//!
//! ⚠️ **AND WHAT IT CANNOT MEASURE, STATED because silence is what produces the green**: the two
//! BROWSER gates (`a11y/axe-gate.mjs`, `a11y/kbd-probe.mjs`) are NOT driven. A mutation to
//! `assets/`, to `templates/`, or to anything whose carrier is a computed page is **invisible
//! here** — measured: inverting an arrow in `app.js` leaves 741 tests and nine gates green while
//! the keyboard gate reports nine failures. Those gates need a rebuild-boot-seed cycle, are not
//! idempotent (the last block writes to the store), and answer 0/1/2 rather than pass/fail, which
//! is a second story's worth of apparatus. **Run them by hand**, re-seeding between runs:
//!
//! ```text
//! cargo build --workspace --locked && ./target/debug/opencmdb &      # with the env CI uses
//! mysql … < a11y/seed.sql                                            # AFTER the boot
//! node a11y/axe-gate.mjs   # then re-seed, then:   node a11y/kbd-probe.mjs
//! ```
//!
//! ⚠️ **It is a TRIPWIRE against the ordinary mistake, never a barrier against a determined one**
//! (story 5.12's narrowing). Nothing stops an author writing a shell line instead.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use anyhow::{Context, Result, bail};

/// How many `test result:` lines a full workspace run must produce — three test targets plus the
/// doctest target.
///
/// 🔑 **It is an EXPECTED COUNT and not a floor**, because the defect it exists for produces
/// *fewer* lines with no other sign: `cargo test --workspace` stops at the first failing crate, so
/// a mutation reddening `opencmdb-bin` silently never runs `opencmdb-core` — measured at 8 red
/// where the honest figure is 17. `--no-fail-fast` is what restores them, and this count is what
/// notices when something else takes them away.
const EXPECTED_TEST_TARGETS: usize = 4;

/// What happened when the driver tried to apply the mutation.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Applied {
    /// The anchor is not in the file.
    Missed,
    /// The anchor is in the file more than once, and replacing all of them is a different
    /// mutation from the one that was predicted.
    Multi(usize),
    /// The anchor was replaced at exactly one site and the file really changed.
    Once,
    /// The replacement is textually identical to the anchor, so nothing moved.
    NoOp,
}

/// What the run measured, or why it could not.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// Something reddened, and the driver says WHICH carrier.
    ///
    /// 🔑 Three carriers, three fields, because they are not summaries of one another: a dead
    /// binding in a test module reds `clippy --all-targets` alone, and a migration losing its
    /// binary collation reds `cargo xtask ci` alone with 741 tests still green.
    Red {
        /// How many tests failed.
        tests: usize,
        /// Did `clippy --all-targets -D warnings` red?
        clippy: bool,
        /// Did the nine gates red?
        gates: bool,
    },
    /// Everything passed, on all three carriers.
    Green,
    /// The tree did not compile.
    CompileFailure(String),
    /// The driver cannot honestly report — with the reason, which is the whole point.
    CannotMeasure(String),
}

/// What the author predicted before running. AC9: this is compared, never merely printed.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Expect {
    /// Red, optionally with the exact count predicted.
    Red(Option<usize>),
    /// Nothing reddens.
    Green,
    /// The tree stops compiling.
    CompileFailure,
}

impl Expect {
    /// Parse `red`, `red:12`, `green`, `compile-fail`.
    ///
    /// # Errors
    ///
    /// Any other spelling, so a typo is a refusal rather than a silently different prediction.
    pub(crate) fn parse(text: &str) -> Result<Self> {
        match text {
            "green" => Ok(Self::Green),
            "compile-fail" => Ok(Self::CompileFailure),
            "red" => Ok(Self::Red(None)),
            other => match other.strip_prefix("red:") {
                Some(n) => Ok(Self::Red(Some(
                    n.parse()
                        .with_context(|| format!("red:{n} is not a count"))?,
                ))),
                None => bail!("--expect must be green, red, red:N or compile-fail (got {other:?})"),
            },
        }
    }

    /// Does the measured outcome match what was predicted?
    ///
    /// ⚠️ `Red(None)` matches any non-zero count deliberately: a prediction of *"it reds"* is
    /// weaker than *"it reds 12"* and the driver must not invent the stronger one.
    pub(crate) fn matches(&self, outcome: &Outcome) -> bool {
        match (self, outcome) {
            (Self::Green, Outcome::Green) => true,
            (Self::CompileFailure, Outcome::CompileFailure(_)) => true,
            (
                Self::Red(None),
                Outcome::Red {
                    tests,
                    clippy,
                    gates,
                },
            ) => *tests > 0 || *clippy || *gates,
            (Self::Red(Some(want)), Outcome::Red { tests, .. }) => want == tests,
            _ => false,
        }
    }
}

/// Apply the mutation to `text`, refusing every shape that has produced a false report.
pub(crate) fn apply(text: &str, anchor: &str, replacement: &str) -> (Applied, String) {
    if anchor == replacement {
        return (Applied::NoOp, text.to_string());
    }
    let hits = text.matches(anchor).count();
    match hits {
        0 => (Applied::Missed, text.to_string()),
        1 => {
            let mutated = text.replacen(anchor, replacement, 1);
            if mutated == text {
                (Applied::NoOp, mutated)
            } else {
                (Applied::Once, mutated)
            }
        }
        n => (Applied::Multi(n), text.to_string()),
    }
}

/// Does this output carry a COMPILER error?
///
/// 🔴 **Anchored on `error[EDDDD]` at line start, and this is the whole of story 5.14b's recorded
/// trap**: matching `^error` counts cargo's own `error: test failed, to rerun pass …` trailer,
/// which it prints on **every** red run — story 6.4's driver did exactly that and reported a
/// compiler-carried red on seven of its eight runs when not one was one.
pub(crate) fn compiler_error(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let rest = line.strip_prefix("error[E")?;
        let (digits, tail) = rest.split_at(rest.find(']')?);
        if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) && tail.starts_with(']')
        {
            Some(line.to_string())
        } else {
            None
        }
    })
}

/// One parsed `test result:` line.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TestResult {
    /// How many passed.
    pub(crate) passed: usize,
    /// How many failed.
    pub(crate) failed: usize,
    /// How many the filter excluded — non-zero means this run is not comparable with a full one.
    pub(crate) filtered_out: usize,
}

/// Read every `test result:` line out of a cargo run.
///
/// 🔑 It reads the WHOLE output. A bounded read is the recorded defect (`head -8` turned 18 red
/// into *"unreachable in a full run"*), and the caller checks the COUNT of lines against
/// [`EXPECTED_TEST_TARGETS`] — so a run that lost a target says so instead of shrinking a number.
pub(crate) fn test_results(output: &str) -> Vec<TestResult> {
    output
        .lines()
        .filter(|line| line.starts_with("test result:"))
        .filter_map(|line| {
            let field = |name: &str| -> Option<usize> {
                let at = line.find(&format!(" {name}"))?;
                line[..at]
                    .rsplit(|c: char| !c.is_ascii_digit())
                    .find(|s| !s.is_empty())?
                    .parse()
                    .ok()
            };
            Some(TestResult {
                passed: field("passed")?,
                failed: field("failed")?,
                filtered_out: field("filtered out")?,
            })
        })
        .collect()
}

/// Turn one cargo run into an outcome, refusing wherever the numbers cannot be trusted.
pub(crate) fn read_run(output: &str, status: Option<i32>, targets: usize) -> Outcome {
    if let Some(error) = compiler_error(output) {
        return Outcome::CompileFailure(error);
    }
    let results = test_results(output);
    if results.len() != targets {
        return Outcome::CannotMeasure(format!(
            "{} `test result:` line(s) where {targets} are due — a target produced \
             none, which a summing driver reports as \"0 passed, 0 failed\". Did the run stop at \
             the first failing crate (--no-fail-fast), or did the tree fail to compile?",
            results.len()
        ));
    }
    if let Some(filtered) = results.iter().find(|r| r.filtered_out > 0) {
        return Outcome::CannotMeasure(format!(
            "the run was FILTERED ({} test(s) excluded) — its counts are not comparable with a \
             full-suite figure, and `cargo test -- A B` reaches this while exiting 0",
            filtered.filtered_out
        ));
    }
    let failed: usize = results.iter().map(|r| r.failed).sum();
    if failed == 0 && status != Some(0) {
        return Outcome::CannotMeasure(format!(
            "no test failed and the process exited {status:?} — something reddened that this \
             driver did not read"
        ));
    }
    if failed == 0 {
        Outcome::Green
    } else {
        Outcome::Red {
            tests: failed,
            clippy: false,
            gates: false,
        }
    }
}

/// Is a store reachable? AC8 — the same counts mean different things with and without one.
///
/// ⚠️ It probes the TCP endpoint rather than the schema: the concern is *did the store-backed
/// tests run at all*, and without `DATABASE_URL` they pass by returning while the totals stay
/// identical. The clock is the only other tell, and a driver that prints counts prints no clock.
pub(crate) fn store_endpoint(url: &str) -> Option<String> {
    let rest = url.split("://").nth(1)?;
    let authority = rest.split('/').next()?;
    let host_port = authority.rsplit('@').next()?;
    if host_port.contains(':') {
        Some(host_port.to_string())
    } else {
        None
    }
}

/// The command line, echoed verbatim beside every count it produces (AC2).
fn shown(program: &str, args: &[&str]) -> String {
    format!("{program} {}", args.join(" "))
}

/// Run a command from the workspace root, capturing stdout and stderr together.
///
/// 🔑 **The status comes from the process** (AC4). `cmd | grep` yields grep's status, which is how
/// a commit went in over a red clippy (6b.1) and another over a red suite (6b.10).
fn run(root: &Path, program: &str, args: &[&str]) -> Result<(Option<i32>, String)> {
    let out = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("running {}", shown(program, args)))?;
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    Ok((out.status.code(), text))
}

/// Restore `path` from `snapshot` AND advance its mtime.
///
/// 🔴 **The mtime is not housekeeping.** A byte-identical restore that preserves the timestamp
/// leaves cargo serving a **stale artefact**: measured on a plain `.rs` file, `git status` clean,
/// `Finished in 0.04s` with no `Compiling` line, and nine tests still failing. Story 6b.7 recorded
/// this as an askama-template property; it is a cargo FINGERPRINT property and applies to every
/// file this driver will ever touch.
fn restore(path: &Path, snapshot: &str) -> Result<()> {
    std::fs::write(path, snapshot).with_context(|| format!("restoring {}", path.display()))?;
    let file = std::fs::File::options()
        .write(true)
        .open(path)
        .with_context(|| format!("reopening {} to advance its mtime", path.display()))?;
    file.set_modified(SystemTime::now())
        .with_context(|| format!("advancing the mtime of {}", path.display()))?;
    let back = std::fs::read_to_string(path)?;
    if back != snapshot {
        bail!(
            "RESTORE FAILED for {} — the file on disk is not what was snapshotted",
            path.display()
        );
    }
    Ok(())
}

/// The driver's exit contract, on `a11y/*.mjs`'s precedent (story 6b.11's arbitration 1).
pub(crate) const CLEAN: u8 = 0;
/// The mutation applied and the outcome CONTRADICTS the prediction — the finding.
pub(crate) const CONTRADICTED: u8 = 1;
/// The driver could not honestly measure. Never confused with the two above.
pub(crate) const CANNOT_MEASURE: u8 = 2;

/// Everything one invocation needs.
pub(crate) struct Mutation {
    /// The file to mutate — exactly one, and the only file the driver writes.
    pub(crate) file: PathBuf,
    /// The text to find.
    pub(crate) anchor: String,
    /// What to put in its place.
    pub(crate) replacement: String,
    /// What the author predicted BEFORE running.
    pub(crate) expect: Expect,
    /// How many `test result:` lines a complete run must produce.
    ///
    /// ⚠️ **Not settable from the command line, and that is deliberate.** [`from_args`] always
    /// builds the strict value; the field exists so this module's own tests can drive a synthetic
    /// one-crate tree. A flag here would be a hole: an author whose run lost a target could
    /// silence the refusal instead of asking why.
    pub(crate) targets: usize,
    /// Must a store be reachable? Same rule as `targets`: strict from the command line, relaxed
    /// only for a synthetic tree that has no store-backed test to lose.
    pub(crate) require_store: bool,
}

/// Apply, measure over the three cargo-side carriers, restore, and compare against the prediction.
///
/// # Errors
///
/// Only for conditions that make the run impossible (an unreadable file, a cargo that will not
/// start). Everything else is an [`Outcome`] — a refusal is a RESULT here, not an error.
pub(crate) fn run_mutation(root: &Path, m: &Mutation, ci: impl Fn() -> Result<bool>) -> Result<u8> {
    let path = root.join(&m.file);
    let snapshot = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {} to snapshot it", path.display()))?;

    let (applied, mutated) = apply(&snapshot, &m.anchor, &m.replacement);
    match applied {
        Applied::Missed => {
            println!(
                "🔴 ANCHOR MISSED in {} — nothing was mutated.",
                m.file.display()
            );
            println!("   A run on an UNMUTATED tree measures nothing, and its green is the green");
            println!("   of the baseline. (rustfmt reflowing the code is how this happened last.)");
            return Ok(CANNOT_MEASURE);
        }
        Applied::Multi(n) => {
            println!(
                "🔴 ANCHOR MATCHED {n} TIMES in {} — refused.",
                m.file.display()
            );
            println!(
                "   Replacing all of them is a DIFFERENT mutation from the one predicted, and"
            );
            println!(
                "   this project keeps deliberate second oracles: replacing both sites of one"
            );
            println!(
                "   rule id measured 6 red where the honest figure is 14, because the mutation"
            );
            println!("   repaired the guard it was meant to red.");
            return Ok(CANNOT_MEASURE);
        }
        Applied::NoOp => {
            println!(
                "🔴 NO-OP in {} — the replacement changes nothing.",
                m.file.display()
            );
            return Ok(CANNOT_MEASURE);
        }
        Applied::Once => {}
    }
    std::fs::write(&path, &mutated).with_context(|| format!("mutating {}", path.display()))?;

    let result = measure(root, m, &ci);
    restore(&path, &snapshot)?;
    let outcome = result?;

    println!("\nPREDICTED: {:?}", m.expect);
    println!("MEASURED:  {outcome:?}");
    Ok(match &outcome {
        Outcome::CannotMeasure(why) => {
            println!("🔴 CANNOT MEASURE: {why}");
            CANNOT_MEASURE
        }
        _ if m.expect.matches(&outcome) => {
            println!("✅ the outcome matches the prediction");
            CLEAN
        }
        _ => {
            println!("🔴 THE OUTCOME CONTRADICTS THE PREDICTION — that is the finding.");
            CONTRADICTED
        }
    })
}

/// The three cargo-side carriers, in the order that fails cheapest first.
fn measure(root: &Path, m: &Mutation, ci: &impl Fn() -> Result<bool>) -> Result<Outcome> {
    let store = match std::env::var("DATABASE_URL") {
        Ok(url) => store_endpoint(&url).map_or_else(
            || "set, but its endpoint could not be read".to_string(),
            |e| format!("reachable at {e}"),
        ),
        Err(_) => "ABSENT".to_string(),
    };
    println!("store: {store}");
    if m.require_store && store == "ABSENT" {
        return Ok(Outcome::CannotMeasure(
            "DATABASE_URL is unset, so every store-backed test passes by RETURNING and the totals \
             are identical to a real run — measured: one mutation gives 741 passed without a store \
             and 1 failed with one. This run may not be recorded."
                .to_string(),
        ));
    }

    // 🔑 clippy FIRST, over --all-targets: it is the carrier that sees what CI sees. A dead
    // binding in a test module leaves `cargo build` and `cargo test` both green and reds here —
    // which is exactly the red CI run story 6b.10 shipped.
    let clippy = [
        "clippy",
        "--workspace",
        "--all-targets",
        "--locked",
        "--",
        "-D",
        "warnings",
    ];
    println!("$ {}", shown("cargo", &clippy));
    let (status, output) = run(root, "cargo", &clippy)?;
    if let Some(error) = compiler_error(&output) {
        return Ok(Outcome::CompileFailure(error));
    }
    let clippy_red = status != Some(0);

    let test = ["test", "--workspace", "--locked", "--no-fail-fast"];
    println!("$ {}", shown("cargo", &test));
    let (status, output) = run(root, "cargo", &test)?;
    let outcome = read_run(&output, status, m.targets);
    if let Outcome::CannotMeasure(_) | Outcome::CompileFailure(_) = outcome {
        // 🔑 The TAIL is for diagnosis and never for a count — AC3 forbids reading a NUMBER
        // through a bounded window, not showing a human where to look. Measured while writing
        // this module: the first end-to-end refused with *"did the tree fail to compile?"* when
        // the real cause was a missing `Cargo.lock` under `--locked`, which the tail names at
        // once.
        println!("   last lines of what was read:");
        for line in output
            .lines()
            .rev()
            .take(6)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            println!("   | {line}");
        }
        return Ok(outcome);
    }
    for result in test_results(&output) {
        println!(
            "   test result: {} passed, {} failed",
            result.passed, result.failed
        );
    }

    // 🔑 The gates IN-PROCESS rather than through a nested `cargo run`: this binary IS xtask, so
    // the gates read the mutated tree directly. A nested cargo works but fights the outer run for
    // `target/`. ⚠️ And `cargo xtask ci` is a THIRD carrier, not a summary of the other two —
    // measured: dropping a binary collation from a migration leaves 741 tests green and reds the
    // `ddl-collation` gate alone.
    println!("$ cargo xtask ci  (in-process)");
    let gates_green = ci()?;

    // 🔑 The three carriers are FOLDED, never collapsed: a red on any of them is a red, and the
    // outcome says which. Reporting only the test count would hide the two carriers that the
    // workspace suite provably does not subsume.
    let tests = match outcome {
        Outcome::Red { tests, .. } => tests,
        _ => 0,
    };
    Ok(if tests == 0 && !clippy_red && gates_green {
        Outcome::Green
    } else {
        Outcome::Red {
            tests,
            clippy: clippy_red,
            gates: !gates_green,
        }
    })
}

/// Parse the command line and run one mutation.
///
/// ```text
/// cargo xtask mutate --file <path> --anchor <text> --replacement <text> --expect <what>
/// ```
///
/// `--expect` is `green`, `red`, `red:N` or `compile-fail`, and it is **required**: a prediction
/// written after the fact is not a prediction, and the retrospective credits the prediction with
/// catching every one of the recorded defects.
///
/// # Errors
///
/// A missing or unrecognised flag, which is answered with [`CANNOT_MEASURE`] rather than with a
/// count — a driver that guesses at its own arguments is the first thing that can lie.
pub(crate) fn from_args(args: &[String], root: &Path) -> Result<u8> {
    let mut file = None;
    let mut anchor = None;
    let mut replacement = None;
    let mut expect = None;
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        let mut value = || -> Result<String> {
            rest.next()
                .cloned()
                .with_context(|| format!("{flag} needs a value"))
        };
        match flag.as_str() {
            "--file" => file = Some(PathBuf::from(value()?)),
            "--anchor" => anchor = Some(value()?),
            "--replacement" => replacement = Some(value()?),
            "--expect" => expect = Some(Expect::parse(&value()?)?),
            "--help" | "-h" => {
                println!("{USAGE}");
                return Ok(CLEAN);
            }
            other => bail!("unknown flag {other:?}\n{USAGE}"),
        }
    }
    let mutation = Mutation {
        file: file.context("--file is required")?,
        anchor: anchor.context("--anchor is required")?,
        replacement: replacement.context("--replacement is required")?,
        expect: expect.context(
            "--expect is required: a prediction written after the fact is not a prediction",
        )?,
        targets: EXPECTED_TEST_TARGETS,
        require_store: true,
    };
    run_mutation(root, &mutation, crate::run_ci)
}

/// What `--help` prints, including the limit AC10 requires it to state.
const USAGE: &str = "\
usage: cargo xtask mutate --file <path> --anchor <text> --replacement <text> --expect <what>

  --expect green | red | red:N | compile-fail   (required — predict BEFORE you run)

exit: 0 the outcome matches the prediction
      1 it contradicts it — that is the finding
      2 the driver could not honestly measure (anchor missed or multi-matched, no-op,
        compile failure, filtered run, a lost test target, no store, restore failed)

⚠️ IT DOES NOT DRIVE THE BROWSER GATES. A mutation to assets/, templates/, or anything whose
   carrier is a computed page is INVISIBLE here — measured: inverting an arrow in app.js leaves
   741 tests and nine gates green while a11y/kbd-probe.mjs reports nine failures. Run those by
   hand, re-seeding the store between runs: they are not idempotent.";

#[cfg(test)]
mod tests {
    use super::*;

    /// 🔴 **The recorded defect (6.4): rustfmt reflowed the code, the anchor stopped matching, the
    /// script carried on, and the green of an UNMUTATED tree was read as a result.**
    #[test]
    fn an_anchor_that_matches_nothing_is_refused_and_the_text_is_untouched() {
        let (applied, text) = apply("fn a() {}\n", "fn b()", "fn c()");
        assert_eq!(applied, Applied::Missed);
        assert_eq!(text, "fn a() {}\n", "and nothing was written");
    }

    /// 🔴 **The measured cost of replace-all (6.4b's validation): `\"l1-exact-mac\"` occurs twice in
    /// `l1.rs` — the production constant AND `CORPUS_EXACT_MAC`, the independent second oracle this
    /// project protects under *deliberate redundancy*. Replacing both measured 6 red where the
    /// honest figure is 14, because the mutation REPAIRED the guard it was meant to red.**
    #[test]
    fn an_anchor_that_matches_twice_is_refused_rather_than_replaced_everywhere() {
        let source = "const A: &str = \"x\";\nconst ORACLE: &str = \"x\";\n";
        let (applied, text) = apply(source, "\"x\"", "\"broken\"");
        assert_eq!(applied, Applied::Multi(2));
        assert_eq!(text, source, "a refusal writes nothing at all");
    }

    /// 🔴 Story 6b.6's two `sed` failures: a replacement identical to the anchor, and a pattern
    /// that could not match — both reported `0 red` off a tree nothing had touched.
    #[test]
    fn a_replacement_that_changes_nothing_is_a_no_op_and_not_a_green() {
        assert_eq!(apply("fn a() {}", "fn a", "fn a").0, Applied::NoOp);
    }

    /// 🔴 **Story 5.14b's recorded trap, and the mandate's literal remedy.** Cargo prints
    /// `error: test failed, to rerun pass …` at line start on EVERY red run, so a `^error`
    /// matcher reports a compiler-carried red on every mutation that reds anything — which story
    /// 6.4's own driver did, on seven of its eight runs.
    #[test]
    fn cargos_own_trailer_is_not_a_compiler_error() {
        let red_run = "test result: FAILED. 495 passed; 8 failed; 0 filtered out\n\
                       error: test failed, to rerun pass `-p opencmdb-bin --bin opencmdb`\n";
        assert_eq!(
            compiler_error(red_run),
            None,
            "a bare `error:` is cargo's trailer, not a compiler error — anchoring on `^error` is \
             the defect this test exists for"
        );
        assert!(
            compiler_error("error[E0308]: mismatched types\n").is_some(),
            "and a real one is `error[EDDDD]` at line start"
        );
        assert_eq!(
            compiler_error("  error[E0308]: indented, so not at line start\n"),
            None
        );
        assert_eq!(compiler_error("error[EXX]: not digits\n"), None);
    }

    /// A `test result:` line, as cargo writes it.
    fn line(passed: usize, failed: usize, filtered: usize) -> String {
        format!(
            "test result: ok. {passed} passed; {failed} failed; 0 ignored; 0 measured; \
             {filtered} filtered out; finished in 0.01s\n"
        )
    }

    /// 🔴 **`cargo test --workspace` STOPS AT THE FIRST FAILING CRATE** — measured on this tree at
    /// 8 red where `--no-fail-fast` gives 17, with the missing nine in a crate that never ran.
    /// Nothing in the output says a window was applied; the run simply ends. **The count of
    /// `test result:` lines is what notices.**
    #[test]
    fn a_lost_test_target_is_a_refusal_and_never_a_count() {
        let one_crate = line(495, 8, 0);
        match read_run(&one_crate, Some(101), 4) {
            Outcome::CannotMeasure(why) => {
                assert!(
                    why.contains("1 `test result:` line(s) where 4 are due"),
                    "{why}"
                );
            }
            other => panic!("a lost target must be a refusal, got {other:?}"),
        }
    }

    /// 🔴 A tree that does not compile emits **zero** `test result:` lines and exits 101 — the same
    /// code as a test failure. A summing driver reports *"0 passed, 0 failed"*, which is the exact
    /// sentence the anchor above exists to make impossible.
    #[test]
    fn a_tree_that_does_not_compile_is_its_own_outcome() {
        let broken = "error[E0425]: cannot find function `nope` in this scope\n\
                      error: could not compile `opencmdb-bin` (bin \"opencmdb\" test)\n";
        assert!(matches!(
            read_run(broken, Some(101), 4),
            Outcome::CompileFailure(_)
        ));
    }

    /// 🔴 **On cargo 1.96 `cargo test A B` fails loudly — the form that is still SILENT is
    /// `cargo test -- A B`**, which runs seven tests of 741 and exits 0. So a driver counting
    /// `--filter` flags closes nothing, and the instrument is cargo's own `filtered out` count.
    #[test]
    fn a_filtered_run_is_refused_however_the_filter_arrived() {
        let filtered = line(2, 0, 501) + &line(5, 0, 156) + &line(0, 0, 77) + &line(0, 0, 1);
        match read_run(&filtered, Some(0), 4) {
            Outcome::CannotMeasure(why) => assert!(why.contains("FILTERED"), "{why}"),
            other => panic!("a filtered run is not comparable, got {other:?}"),
        }
    }

    /// A clean full run is green, and a red one names its count.
    #[test]
    fn a_complete_run_is_read_in_full() {
        let clean = line(503, 0, 0) + &line(161, 0, 0) + &line(77, 0, 0) + &line(0, 0, 0);
        assert_eq!(read_run(&clean, Some(0), 4), Outcome::Green);
        let red = line(495, 8, 0) + &line(152, 9, 0) + &line(77, 0, 0) + &line(0, 0, 0);
        assert_eq!(
            read_run(&red, Some(101), 4),
            Outcome::Red {
                tests: 17,
                clippy: false,
                gates: false
            },
            "17 — the figure `--no-fail-fast` restores, summed across every target"
        );
    }

    /// ⚠️ A run where nothing failed and the process still reddened is a REFUSAL: something the
    /// driver does not read went wrong, and reporting green would be inventing a result.
    #[test]
    fn a_green_suite_over_a_red_process_is_refused() {
        let clean = line(503, 0, 0) + &line(161, 0, 0) + &line(77, 0, 0) + &line(0, 0, 0);
        assert!(matches!(
            read_run(&clean, Some(101), 4),
            Outcome::CannotMeasure(_)
        ));
    }

    /// 🔑 **The prediction is COMPARED, not printed beside the result for a human to notice.** The
    /// retrospective credits a contradicted prediction with catching every recorded defect; this is
    /// that instrument made mechanical.
    #[test]
    fn the_prediction_is_compared_and_a_bare_red_does_not_pin_a_count() {
        let red12 = Outcome::Red {
            tests: 12,
            clippy: false,
            gates: false,
        };
        assert!(Expect::parse("red").expect("red").matches(&red12));
        assert!(Expect::parse("red:12").expect("red:12").matches(&red12));
        assert!(
            !Expect::parse("red:11").expect("red:11").matches(&red12),
            "a predicted COUNT is pinned exactly — story 6.4's M1b was recorded at 3 and measured 6"
        );
        assert!(!Expect::parse("green").expect("green").matches(&red12));
        assert!(
            Expect::parse("green")
                .expect("green")
                .matches(&Outcome::Green)
        );
        assert!(
            Expect::parse("compile-fail")
                .expect("compile-fail")
                .matches(&Outcome::CompileFailure("error[E0425]".into()))
        );
        assert!(Expect::parse("nonsense").is_err(), "a typo is a refusal");
    }

    /// 🔴 **A carrier the workspace suite does not subsume still counts as red.** Measured: a dead
    /// binding in a test module reds `clippy --all-targets` alone; a migration losing its binary
    /// collation reds `cargo xtask ci` alone with 741 tests green.
    #[test]
    fn a_red_on_any_carrier_is_a_red_and_the_outcome_says_which() {
        let clippy_only = Outcome::Red {
            tests: 0,
            clippy: true,
            gates: false,
        };
        assert!(Expect::parse("red").expect("red").matches(&clippy_only));
        assert!(
            !Expect::parse("red:1").expect("red:1").matches(&clippy_only),
            "and a count still means TESTS — a clippy red is not one test"
        );
    }

    /// 🔴 **Story 6b.7's defect, reproduced and closed.** A byte-identical restore that preserves
    /// the mtime leaves cargo serving a stale artefact: `git status` clean, source identical, nine
    /// tests still failing. It is a cargo FINGERPRINT property, not an askama one.
    #[test]
    fn a_restore_advances_the_mtime_so_cargo_cannot_serve_a_stale_artefact() {
        let dir = std::env::temp_dir().join(format!("xtask-mutate-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch");
        let file = dir.join("subject.rs");
        std::fs::write(&file, "fn a() {}\n").expect("write");
        let before = std::fs::metadata(&file)
            .expect("meta")
            .modified()
            .expect("mtime");

        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&file, "fn MUTATED() {}\n").expect("mutate");
        restore(&file, "fn a() {}\n").expect("restore");

        assert_eq!(std::fs::read_to_string(&file).expect("read"), "fn a() {}\n");
        let after = std::fs::metadata(&file)
            .expect("meta")
            .modified()
            .expect("mtime");
        assert!(
            after > before,
            "the mtime must ADVANCE: a byte-perfect restore that keeps the old timestamp leaves \
             cargo believing its artefact is current ({before:?} -> {after:?})"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// ⚠️ A restore that does not land is a hard failure, not a warning.
    #[test]
    fn a_restore_that_does_not_land_is_a_failure() {
        let missing = Path::new("/proc/definitely/not/writable/subject.rs");
        assert!(restore(missing, "x").is_err());
    }

    /// 🔴 **The store changes the verdict and NOT the counts** — measured: one mutation gives
    /// *741 passed, exit 0* without a store and *1 failed, exit 101* with one. The clock is the
    /// only other tell, and a driver that prints counts prints no clock.
    #[test]
    fn the_store_endpoint_is_read_from_the_url() {
        assert_eq!(
            store_endpoint("mysql://root:pw@127.0.0.1:13405/opencmdb_test").as_deref(),
            Some("127.0.0.1:13405")
        );
        assert_eq!(
            store_endpoint("mysql://host/db"),
            None,
            "no port, no endpoint"
        );
        assert_eq!(store_endpoint("nonsense"), None);
    }

    /// 🔑 **THE END-TO-END, over a synthetic crate with its own `CARGO_TARGET_DIR`.**
    ///
    /// Story 5.12's finding is why this exists: its whole gate body was deletable with the xtask
    /// suite green, because every test attacked the helper and none drove the thing. ⚠️ And story
    /// 6b.11's is why the tree is SYNTHETIC: *a gate green over the real tree says nothing about
    /// its own tests*. A nested cargo run does not deadlock (measured, 5.98 s) but it fights the
    /// outer run for `target/`, so this one is given a target directory of its own.
    #[test]
    fn the_driver_drives_a_real_cargo_run_end_to_end() {
        if std::env::var("XTASK_MUTATE_E2E").is_err() {
            eprintln!("skipping the end-to-end: set XTASK_MUTATE_E2E=1 (it invokes cargo)");
            return;
        }
        let dir = std::env::temp_dir().join(format!("xtask-mutate-e2e-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).expect("scratch");
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"subject\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\
             [workspace]\n",
        )
        .expect("manifest");
        std::fs::write(
            dir.join("src/lib.rs"),
            "pub fn answer() -> u8 { 42 }\n\
             #[cfg(test)]\nmod t {\n  #[test] fn it_is_42() { assert_eq!(super::answer(), 42); }\n}\n",
        )
        .expect("lib");
        // ⚠️ `--locked` REFUSES to create a lock file, and the driver passes it on every carrier
        // (this project's own rule: `Cargo.lock` is committed, always `--locked`). Without this
        // the run produces no `test result:` line at all and the driver refuses — correctly, and
        // for a cause its message could not name. That refusal is why the tail is printed below.
        std::fs::write(
            dir.join("Cargo.lock"),
            "version = 3

[[package]]
name = \"subject\"
version = \"0.0.0\"
",
        )
        .expect("lock");
        unsafe { std::env::set_var("CARGO_TARGET_DIR", dir.join("target")) };

        let mutation = Mutation {
            file: PathBuf::from("src/lib.rs"),
            anchor: "42 }".to_string(),
            replacement: "41 }".to_string(),
            expect: Expect::Red(Some(1)),
            targets: 2, // the lib test target and the doctest target
            require_store: false,
        };
        let code = run_mutation(&dir, &mutation, || Ok(true)).expect("the driver runs");
        assert_eq!(
            code, CLEAN,
            "the mutation reds exactly one test, which is what was predicted"
        );
        assert!(
            std::fs::read_to_string(dir.join("src/lib.rs"))
                .expect("read back")
                .contains("42 }"),
            "and the tree is RESTORED — a driver that leaves its mutation behind poisons every \
             run after it"
        );
        unsafe { std::env::remove_var("CARGO_TARGET_DIR") };
        std::fs::remove_dir_all(&dir).ok();
    }
}
