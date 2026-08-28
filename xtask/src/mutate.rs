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
//! recurred across THREE epics — 5, 6b and 6 — by the rows of the story's own table. *(It said
//! "six consecutive epics"; the code review counted.)*
//!
//! # What it promises, and what it does not
//!
//! It refuses to report a number it did not honestly obtain. Every outcome below is a REFUSAL
//! rather than a count, and each one was a recorded defect first:
//!
//! - the anchor matched **nothing**, or matched **more than once** (a replace-all mutates the
//!   second oracle as well as the code — measured at 6 red where the honest figure is 14);
//! - the replacement changed **nothing** (`sed` with itself);
//! - the tree did not **compile**, detected by a line-anchored `error[EDDDD]` **and only when the
//!   run produced no `test result:` line at all** — a failing test whose output quotes a
//!   diagnostic is not a compile failure, and the code review measured the driver calling one;
//! - the run was **filtered** — measured by cargo's own `filtered out` count, not by counting
//!   flags, because `cargo test -- A B` still runs a handful of tests and exits 0;
//! - the **file** lay outside the workspace, where no carrier can see it;
//! - a test target produced **no `test result:` line at all**, which a summing driver reports as
//!   *"0 passed, 0 failed"*;
//! - the **store** was absent — or set and UNUSABLE, which is worse: the store-backed tests then
//!   panic rather than skip, so the red you would read is the harness's.
//!
//! ⚠️ **And it takes NO BASELINE unless `--baseline` is given**, so a test already red before the
//! mutation confirms a `red` prediction on its own. Opt-in by Guy's arbitration of 2026-08-26; the
//! run says so every time it is not used.
//!
//! ⚠️ **AND WHAT IT CANNOT MEASURE, STATED because silence is what produces the green**: the two
//! BROWSER gates (`a11y/axe-gate.mjs`, `a11y/kbd-probe.mjs`) are NOT driven. A mutation to
//! `assets/`, to `templates/`, or to anything whose carrier is a computed page is **invisible
//! here** — measured: inverting an arrow in `app.js` leaves the whole Rust suite and all nine
//! gates green while the keyboard gate reports nine failures. Those gates need a
//! rebuild-boot-seed cycle, are not idempotent (the last block writes to the store), and answer
//! 0/1/2 rather than pass/fail, which is a second story's worth of apparatus. **Run them by
//! hand** — `cargo xtask mutate --help` carries the full recipe, which is `ci.yml`'s rather than
//! a paraphrase of it. ⚠️ The paraphrase that stood here omitted `npm ci`, the credentials and
//! `AXE_REQUIRE_QUEUE`/`AXE_REQUIRE_GESTURE` — without the last two an empty queue reads as a
//! PASS, which is the *green on residue* defect story 6b.11 closed.
//!
//! 🔑 **An interrupted run leaves its snapshot on DISK**, at `target/xtask-mutate/`, and the next
//! run refuses until it is dealt with. ⚠️ The residual this replaces said the file was
//! *"recoverable from git"* — false on exactly the dirty tree AC7 declares normal, where
//! `git checkout --` is recorded defect row 7.
//!
//! ⚠️ **It is a TRIPWIRE against the ordinary mistake, never a barrier against a determined one**
//! (story 5.12's narrowing). Nothing stops an author writing a shell line instead.

use std::path::{Path, PathBuf};
use std::process::Command;

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
    /// binary collation reds `cargo xtask ci` alone with every test still green.
    Red {
        /// How many tests failed.
        tests: usize,
        /// Did `clippy --all-targets -D warnings` red?
        clippy: bool,
        /// Did the gates red?
        gates: bool,
    },
    /// Everything passed, on all three carriers.
    Green,
    /// The tree did not compile.
    CompileFailure(String),
    /// The driver cannot honestly report — with the reason, which is the whole point.
    CannotMeasure(String),
}

/// What the driver found when it looked for a store.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Store {
    /// `DATABASE_URL` is unset: every store-backed test passes by RETURNING and the totals are
    /// identical to a real run.
    Absent,
    /// Set, and something answers there.
    Reachable(String),
    /// Set and unusable — which is WORSE than absent: the store-backed tests do not skip, they
    /// panic, and the resulting red has nothing to do with the mutation.
    Unusable(String),
}

impl Store {
    /// Why this run may not be recorded, or `None` if it may.
    pub(crate) fn refusal(&self) -> Option<String> {
        match self {
            Self::Reachable(_) => None,
            Self::Absent => Some(
                "DATABASE_URL is unset, so every store-backed test passes by RETURNING and the \
                 totals are identical to a real run — the clock beside each carrier is the only \
                 tell. This run may not be recorded for anything the store carries."
                    .to_string(),
            ),
            Self::Unusable(why) => Some(format!(
                "DATABASE_URL is set and unusable ({why}), which is worse than unset: the \
                 store-backed tests PANIC rather than skip, so the red you would read is the \
                 harness's and not the mutation's. Measured at the code review: a dead port gave \
                 a dozen failures and a run killed at over ten minutes."
            )),
        }
    }
}

impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Absent => write!(f, "ABSENT"),
            Self::Reachable(e) => write!(f, "reachable at {e} (connected)"),
            Self::Unusable(why) => write!(f, "UNUSABLE — {why}"),
        }
    }
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
    let results = test_results(output);
    // 🔴 **A COMPILE FAILURE PRODUCES NO `test result:` LINE AT ALL, and that is what tells it
    // apart from a test whose OUTPUT contains a diagnostic.** This ran `compiler_error` over the
    // whole merged stream first, and the code review measured what that costs: a planted panic
    // message carrying `error[E0308]` made the driver report `CompileFailure` for a tree that
    // compiled, and under `--expect compile-fail` it printed ✅ and exited 0. Cargo replays every
    // failing test's output at column 0, so the anchor AC5 exists to make precise was being
    // applied to program output, where it means nothing. ⚠️ It also short-circuited AC3: the
    // check preceded the target count, so the run was never read.
    if results.is_empty()
        && let Some(error) = compiler_error(output)
    {
        return Outcome::CompileFailure(error);
    }
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

/// The `host:port` a `DATABASE_URL` names, or `None` when it names none.
///
/// ⚠️ **PARSING ONLY.** It said *"it probes the TCP endpoint"* until the code review, and probed
/// nothing at all — see [`store_is_reachable`], which is what the word now means.
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

/// Is the store actually there? AC8 — the same counts mean different things with and without one.
///
/// 🔴 **IT CONNECTS, and until the code review it did not.** The driver printed
/// `store: reachable at 127.0.0.1:13405` from the SHAPE of a string, and the review measured what
/// that costs: pointed at a dead port it printed the same sentence, the store-backed tests then
/// **panicked** rather than skipping (`MySqlPool::connect(&url).expect("connect")`), and the run
/// was killed at over ten minutes with a dozen failures that would have been folded into the
/// mutation's count and matched against a `--expect red`. *A driver asserting a fact it did not
/// measure* is the one thing this module exists to stop.
///
/// ⚠️ A TCP handshake is not a schema check: it says the endpoint answers, not that the migrations
/// ran. That is the honest limit of a probe this cheap, and it is enough for the question at hand
/// — *did the store-backed tests run at all, or pass by returning?*
fn store_is_reachable(endpoint: &str) -> bool {
    use std::net::{TcpStream, ToSocketAddrs};
    let Ok(mut addrs) = endpoint.to_socket_addrs() else {
        return false;
    };
    addrs.any(|addr| TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)).is_ok())
}

/// The command line, echoed verbatim beside every count it produces (AC4, AC6).
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

/// Restore `path` from `snapshot`, and verify byte-for-byte that it landed.
///
/// # The mtime, and what actually carries it
///
/// A restore that preserves the timestamp leaves cargo serving a **stale artefact** — measured on
/// a plain `.rs` file: `git status` clean, `Finished in 0.04s` with no `Compiling` line, nine
/// tests still failing. Story 6b.7 recorded that as an askama-template property; it is a cargo
/// FINGERPRINT property and reaches every file this driver touches.
///
/// 🔴 **But `std::fs::write` already advances the mtime, and this function used to call
/// `set_modified` on top of it as if that were the carrier.** Two review layers measured
/// independently that deleting the extra call leaves the suite green — *a guard placed where the
/// defect cannot occur*, and the plant recorded against it (`set_modified(UNIX_EPOCH)`, pushing
/// the mtime BACKWARDS) was a mutation nobody would make. ⚠️ The recorded defect was
/// `shutil.copy2`, which PRESERVES metadata; `fs::write` does not, so the class is unreachable
/// here. The redundant call is gone and the test below pins the property that really holds.
fn restore(path: &Path, snapshot: &str) -> Result<()> {
    std::fs::write(path, snapshot).with_context(|| format!("restoring {}", path.display()))?;
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
    /// Measure the UNMUTATED tree first? (`--baseline`)
    ///
    /// 🔴 **Without it, a pre-existing red confirms a `red` prediction** — and AC7 deliberately
    /// removed the dirty-tree refusal, so the driver is designed for exactly the tree where a
    /// test may already be red for reasons that have nothing to do with the anchor. The code
    /// review found the driver knew the word *baseline* only in a refusal MESSAGE and never
    /// measured one.
    ///
    /// ⚠️ **Opt-in, by Guy's arbitration of 2026-08-26**, over measuring it every time: the
    /// baseline doubles the cost of every mutation, and the cost should fall on whoever needs it.
    /// The refusal below states the limit whenever it is not used, so nobody has to remember.
    pub(crate) baseline: bool,
}

/// Apply, measure over the three cargo-side carriers, restore, and compare against the prediction.
///
/// # Errors
///
/// Only for conditions that make the run impossible (an unreadable file, a cargo that will not
/// start). Everything else is an [`Outcome`] — a refusal is a RESULT here, not an error.
pub(crate) fn run_mutation(
    root: &Path,
    m: &Mutation,
    gates: impl Fn(&Path) -> Result<(bool, String)>,
) -> Result<u8> {
    let path = root.join(&m.file);
    // 🔴 **CONFINED TO THE WORKSPACE.** `root.join(file)` discards `root` entirely for an absolute
    // path and does not normalise `..`, and the review measured the consequence end to end: the
    // driver mutated a file in `/tmp`, ran all three carriers, restored it, and delivered a
    // verdict — on a file no carrier can see. *A mutation outside the workspace measures nothing,
    // and the honest answer is to say so rather than to run.*
    let inside = path
        .canonicalize()
        .with_context(|| format!("resolving {}", path.display()))?;
    let root_real = root
        .canonicalize()
        .with_context(|| format!("resolving the workspace root {}", root.display()))?;
    if !inside.starts_with(&root_real) {
        println!(
            "🔴 OUTSIDE THE WORKSPACE: {} resolves to {}, which no carrier walks.",
            m.file.display(),
            inside.display()
        );
        return Ok(CANNOT_MEASURE);
    }
    let snapshot = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {} to snapshot it", path.display()))?;

    // 🔴 **THE SNAPSHOT GOES TO DISK BEFORE THE MUTATION, at a path anyone can find.** T9 required
    // this and was ticked without it; the review measured the cost. The restore runs after the
    // carriers return, so a Ctrl-C during a multi-minute cargo run — or a panic — leaves the tree
    // mutated with the snapshot in a dead process's memory. ⚠️ And the residual written in its
    // place said the file was *"recoverable from git"*, which is FALSE on exactly the tree AC7
    // declares normal: this driver deliberately runs on dirty trees, and `git checkout --` on one
    // is recorded defect row 7 — nine uncommitted keys lost. The review's own layer re-enacted it
    // in one command while cleaning up.
    let vault = root.join("target/xtask-mutate");
    let stash = vault.join(format!(
        "{}.snapshot",
        m.file.to_string_lossy().replace('/', "%")
    ));
    if stash.exists() {
        println!(
            "🔴 A SNAPSHOT FROM AN EARLIER RUN IS STILL HERE: {}",
            stash.display()
        );
        println!(
            "   That run did not finish, so {} may still be MUTATED.",
            m.file.display()
        );
        println!("   Compare them, restore by hand, and delete the snapshot — do NOT use");
        println!("   `git checkout`: on a dirty tree it destroys uncommitted work (defect row 7).");
        return Ok(CANNOT_MEASURE);
    }
    std::fs::create_dir_all(&vault).with_context(|| format!("creating {}", vault.display()))?;
    std::fs::write(&stash, &snapshot).with_context(|| format!("writing {}", stash.display()))?;

    // 🔑 THE BASELINE FIRST, on the UNMUTATED tree, so a pre-existing red is caught before the
    // mutation can be credited with it. A baseline that is not clean is a REFUSAL: measuring a
    // mutation against a tree that was already broken tells you nothing about either.
    if m.baseline {
        println!("── baseline, on the UNMUTATED tree ──");
        match measure(root, m, &gates)? {
            Outcome::Green => println!("── baseline clean; applying the mutation ──"),
            other => {
                println!("🔴 THE BASELINE IS NOT CLEAN: {other:?}");
                println!("   Nothing measured after this could be attributed to the mutation.");
                return Ok(CANNOT_MEASURE);
            }
        }
    }

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

    // 🔴 **THE DRIVER SHOWS ITS WORK, and until the code review it showed none.** AC1, §0b row 4
    // and T2 all require the applied diff, and row 4 is the FOUR-occurrence family — *a mutation
    // named for one thing and applied to another* (5.14 ×2, 5.12, 6b.4). The success path printed
    // nothing at all: the review drove an anchor that landed only inside a COMMENT and got a
    // transcript indistinguishable from a real code mutation. The exit code makes the PREDICTION
    // mechanical; this is what makes the MUTATION auditable.
    // 🔴 **OFF BY ONE ON ITS FIRST RUN, and the tell was in the output it printed.** This read
    // `snapshot[..hit].lines().count()`, which is the number of COMPLETE lines before the hit —
    // i.e. one less than the line the hit is on — so the diff showed the line ABOVE the change,
    // identical on both sides. ⚠️ *An identical `-`/`+` pair is the shape of a diff that is
    // looking at the wrong line*, and this block exists to close the four-occurrence family *a
    // mutation named for one thing and applied to another*: printing the wrong line would have
    // fed that class rather than closed it. Newlines before the hit, plus one, is exact.
    let hit = snapshot.find(&m.anchor).unwrap_or(0);
    let line_no = snapshot[..hit].matches('\n').count() + 1;
    println!("APPLIED at {}:{line_no} (1 site)", m.file.display());
    for (marker, text) in [("-", &snapshot), ("+", &mutated)] {
        let line = text.lines().nth(line_no - 1).unwrap_or("").trim_end();
        println!("   {marker} {}", &line[..line.len().min(160)]);
    }

    let result = measure(root, m, &gates);
    restore(&path, &snapshot)?;
    // Only once the file is provably back does the stash go: while it exists, it is the recovery.
    std::fs::remove_file(&stash).with_context(|| format!("clearing {}", stash.display()))?;
    let outcome = result?;

    if !m.baseline {
        println!(
            "⚠️  no baseline was measured (--baseline): a test already red before this mutation \
             would confirm a `red` prediction on its own."
        );
    }

    println!("\nPREDICTED: {:?}", m.expect);
    println!("MEASURED:  {outcome:?}");
    Ok(match &outcome {
        Outcome::CannotMeasure(why) => {
            println!("🔴 CANNOT MEASURE: {why}");
            CANNOT_MEASURE
        }
        // 🔴 **An UNPREDICTED compile failure is `2`, never `1`.** It exited 1 until the code
        // review — *the finding* — against AC9 and against this module's own `--help`, in the same
        // commit. A tree that did not build measured nothing about any guard, so it is *the driver
        // could not measure* and not *the product moved*. ⚠️ A PREDICTED one still matches and
        // exits 0: that is a legitimate thing to predict (story 6.1's M4, story 6.4's `E0004`).
        Outcome::CompileFailure(error) if !m.expect.matches(&outcome) => {
            println!("🔴 CANNOT MEASURE: the tree did not compile — {error}");
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
fn measure(
    root: &Path,
    m: &Mutation,
    gates: &impl Fn(&Path) -> Result<(bool, String)>,
) -> Result<Outcome> {
    // 🔑 Three states, not two, and the middle one used to pass. A URL that is set but unusable —
    // a dead port, a wrong host, an empty string — reported *"set, but its endpoint could not be
    // read"*, which is not the string `"ABSENT"`, so the requirement passed and the run proceeded.
    // The comparison is on an enum now rather than on a rendered sentence, so a copy-edit of the
    // message cannot turn the guard off.
    let store = match std::env::var("DATABASE_URL") {
        Err(_) => Store::Absent,
        Ok(url) => match store_endpoint(&url) {
            None => Store::Unusable(format!("{url:?} names no host:port")),
            Some(endpoint) => {
                if store_is_reachable(&endpoint) {
                    Store::Reachable(endpoint)
                } else {
                    Store::Unusable(format!("nothing answers at {endpoint}"))
                }
            }
        },
    };
    println!("store: {store}");
    if m.require_store
        && let Some(why) = store.refusal()
    {
        return Ok(Outcome::CannotMeasure(why));
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
    // ⚠️ The verdict is NAMED and, when it reds, its diagnostics are shown. It printed the command
    // and then nothing: the author learned `clippy: true` from the final line and had to re-run
    // clippy by hand to find out what it had said.
    println!("   clippy: {}", if clippy_red { "RED" } else { "green" });
    if clippy_red {
        for line in output.lines().filter(|l| l.starts_with("error")).take(5) {
            println!("   | {line}");
        }
    }

    let test = ["test", "--workspace", "--locked", "--no-fail-fast"];
    println!("$ {}", shown("cargo", &test));
    let test_clock = std::time::Instant::now();
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
    // 🔴 **THE CLOCK, which T6 required and which shipped missing — and worse, the driver used to
    // DELETE cargo's own.** It re-printed each line as `N passed, M failed`, dropping
    // `finished in 0.21s`; with the store probe asserting a fact it had not measured, both tells
    // that a store-backed run really executed were gone together, in the story that named AC8 for
    // them. Cargo's line goes through whole now, and the wall clock of the carrier beside it.
    for line in output.lines().filter(|l| l.starts_with("test result:")) {
        println!("   {line}");
    }
    println!("   (cargo test: {:.2?} wall)", test_clock.elapsed());

    // 🔴 **THE GATES RUN THROUGH A NESTED CARGO, and this is Guy's arbitration of 2026-08-26
    // taken on a measurement.** They ran IN-PROCESS until the code review, which meant they read
    // the already-compiled binary — this process's own text, built BEFORE the mutation — while
    // clippy and `cargo test` rebuilt xtask with it. Both directions were measured: setting
    // `MAX_CODE_LINES` to 20 printed *"✅ file-size 41 file(s) under 2000 code lines"*, quoting
    // the constant it had just replaced; and a STALE binary manufactured a gate red on a pristine
    // tree and the driver labelled it *the finding*. ⚠️ Mutating a gate is the second most common
    // shape in this project (5.12, 6.3, 6b.10), so the stale carrier hit exactly the case it was
    // added for. The nested run costs a rebuild; a carrier reading pre-mutation code costs the
    // truth.
    //
    // 🔑 And `cargo xtask ci` is a THIRD carrier, not a summary of the other two — measured:
    // dropping a binary collation from a migration leaves the tests green and reds
    // `ddl-collation` alone.
    let (gates_green, gate_output) = gates(root)?;
    if let Some(error) = compiler_error(&gate_output) {
        return Ok(Outcome::CompileFailure(error));
    }
    if !gates_green {
        for line in gate_output.lines().filter(|l| l.contains("🔴")) {
            println!("   | {line}");
        }
    }

    Ok(fold(outcome, clippy_red, gates_green))
}

/// Fold the three carriers into one outcome.
///
/// 🔑 **FOLDED, never collapsed**: a red on any carrier is a red, and the result says WHICH.
/// Reporting only the test count would hide the two carriers the workspace suite provably does not
/// subsume — a dead binding in a test module reds `clippy --all-targets` alone, a migration losing
/// its binary collation reds the gates alone.
pub(crate) fn fold(tests_outcome: Outcome, clippy_red: bool, gates_green: bool) -> Outcome {
    let tests = match tests_outcome {
        Outcome::Red { tests, .. } => tests,
        _ => 0,
    };
    if tests == 0 && !clippy_red && gates_green {
        Outcome::Green
    } else {
        Outcome::Red {
            tests,
            clippy: clippy_red,
            gates: !gates_green,
        }
    }
}

/// The `cargo xtask ci` gates, through a NESTED cargo so they read the mutated tree.
///
/// ⚠️ The count is deliberately not written here: it was `nine` until story 6.5 added a tenth, and
/// a number in a doc comment is a number nobody re-measures.
///
/// 🔴 Guy's arbitration of 2026-08-26. The in-process form read this process's own text, built
/// before the mutation — see [`measure`] for the two measurements that settled it. It is injected
/// rather than called directly so the driver's own tests can drive the fold without a workspace.
///
/// # Errors
///
/// Only if cargo cannot be started; a RED gate is a value, not an error.
pub(crate) fn gates_through_cargo(root: &Path) -> Result<(bool, String)> {
    let args = ["run", "--quiet", "-p", "xtask", "--locked", "--", "ci"];
    println!("$ {}", shown("cargo", &args));
    let (status, output) = run(root, "cargo", &args)?;
    Ok((status == Some(0), output))
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
    let mutation = parse(args)?;
    if mutation.file.as_os_str().is_empty() {
        return Ok(CLEAN); // `--help` was asked for and printed; no mutation was run.
    }
    run_mutation(root, &mutation, gates_through_cargo)
}

/// The PURE half: argv in, a strict [`Mutation`] out.
///
/// 🔑 Split out at the code review because `from_args` was reachable only through `main.rs` and
/// tested by nothing, so the strict `targets`, the strict `require_store` and the `--expect`
/// requirement were all carried by a sentence — the review planted each and the suite stayed
/// green.
///
/// # Errors
///
/// A missing, repeated or unrecognised flag.
fn parse(args: &[String]) -> Result<Mutation> {
    let mut file = None;
    let mut anchor = None;
    let mut replacement = None;
    let mut expect = None;
    let mut baseline = false;
    let mut seen: Vec<&str> = Vec::new();
    let mut rest = args.iter();
    while let Some(flag) = rest.next() {
        // ⚠️ A repeated flag is a REFUSAL, not last-wins. Recorded defect row 11 is *"a batch edit
        // lost three repairs to one failed anchor, SILENTLY"*, and a driver that silently prefers
        // one of two `--anchor`s is the same shape one level up.
        if seen.contains(&flag.as_str()) {
            bail!("{flag} given twice — which of the two did you mean?");
        }
        seen.push(flag.as_str());
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
            "--baseline" => baseline = true,
            "--help" | "-h" => {
                println!("{USAGE}");
                return Ok(Mutation {
                    file: PathBuf::new(),
                    anchor: String::new(),
                    replacement: String::new(),
                    expect: Expect::Green,
                    targets: EXPECTED_TEST_TARGETS,
                    require_store: true,
                    baseline: false,
                });
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
        baseline,
    };
    Ok(mutation)
}

/// What `--help` prints, including the limit AC10 requires it to state.
const USAGE: &str = "\
usage: cargo xtask mutate --file <path> --anchor <text> --replacement <text> --expect <what>

  --expect green | red | red:N | compile-fail   (required — predict BEFORE you run)
  --baseline                                    measure the UNMUTATED tree first (see below)

exit: 0 the outcome matches the prediction (and for --help, which runs no mutation)
      1 it contradicts it — that is the finding
      2 the driver could not honestly measure: anchor missed or multi-matched, no-op, a file
        outside the workspace, an UNPREDICTED compile failure, a filtered run, a lost test
        target, no store or an unusable one, a baseline that was not clean, restore failed

⚠️ NO BASELINE unless --baseline is given. A test already red before the mutation would confirm
   a `red` prediction on its own, and this driver deliberately runs on dirty trees.

⚠️ IT DOES NOT DRIVE THE BROWSER GATES. A mutation to assets/, templates/, or anything whose
   carrier is a computed page is INVISIBLE here — measured: inverting an arrow in app.js leaves
   the whole Rust suite and every `cargo xtask ci` gate green while a11y/kbd-probe.mjs reports
   nine failures.

   Run them by hand. This recipe is ci.yml's rather than a paraphrase, because the omissions are
   what bite: without AXE_REQUIRE_QUEUE an empty queue reads as a PASS, which is the \"green on
   residue\" defect story 6b.11 closed.

     cargo build --workspace --locked
     DATABASE_URL=... OPENCMDB_BASIC_USER=ci OPENCMDB_BASIC_PASSWORD=ci-not-a-secret \\
       OPENCMDB_DOCUMENT_ENABLED=1 ./target/debug/opencmdb &
     until curl -fsS --max-time 5 -o /dev/null http://127.0.0.1:8080/healthz; do sleep 1; done
     mysql ... < a11y/seed.sql        # AFTER the boot: the binary owns the migrations
     npm --prefix a11y ci             # without it node exits on ITS code, not the gate's
     OPENCMDB_BASIC_USER=ci OPENCMDB_BASIC_PASSWORD=ci-not-a-secret \\
       AXE_REQUIRE_QUEUE=1 AXE_REQUIRE_GESTURE=1 node a11y/axe-gate.mjs
     mysql ... < a11y/seed.sql        # RE-SEED: kbd-probe writes; it is not idempotent
     OPENCMDB_BASIC_USER=ci OPENCMDB_BASIC_PASSWORD=ci-not-a-secret node a11y/kbd-probe.mjs

   Both answer 0 clean / 1 the product / 2 the gate could not run.";

#[cfg(test)]
mod tests {
    use super::*;

    /// 🔴 **The applied diff names the line the change is ON.** It printed the line ABOVE it on
    /// its first run — `snapshot[..hit].lines().count()` is the count of COMPLETE lines before the
    /// hit — so the two halves came out identical, which is the shape of a diff looking at the
    /// wrong line. ⚠️ This block exists to close *a mutation named for one thing and applied to
    /// another*; printing the wrong line would have fed that family rather than closed it.
    #[test]
    fn the_reported_line_is_the_one_the_anchor_is_on() {
        let text = "one\ntwo\nTARGET\nfour\n";
        let hit = text.find("TARGET").expect("the anchor");
        assert_eq!(
            text[..hit].matches('\n').count() + 1,
            3,
            "TARGET is on line 3; the count of complete lines before it is 2, which is the \
             off-by-one this test exists for"
        );
        assert_eq!(text.lines().nth(3 - 1), Some("TARGET"));
    }

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

    /// 🔴 **A FAILING TEST WHOSE OUTPUT QUOTES A DIAGNOSTIC IS NOT A COMPILE FAILURE**, and the
    /// code review measured the driver calling one. `compiler_error` ran over the merged
    /// stdout+stderr of `cargo test`, where cargo replays every panic at column 0 — so a planted
    /// panic message carrying `error[E0308]` was reported as `CompileFailure`, and under
    /// `--expect compile-fail` the driver printed ✅ and exited **0** over a tree that compiled.
    /// ⚠️ It also short-circuited AC3: the check preceded the target count, so the run was never
    /// read at all.
    ///
    /// 🔑 **The discriminator is structural**: a compile failure produces NO `test result:` line.
    #[test]
    fn a_test_that_prints_a_diagnostic_is_not_a_compile_failure() {
        let red_with_a_quote = line(495, 1, 0)
            + "error[E0308]: a TEST printed this; the tree compiles fine
" + &line(161, 0, 0)
            + &line(92, 0, 0)
            + &line(0, 0, 0);
        assert_eq!(
            read_run(&red_with_a_quote, Some(101), 4),
            Outcome::Red {
                tests: 1,
                clippy: false,
                gates: false
            },
            "one test failed and the tree compiled — reporting a compile failure here is a \
             PLAUSIBLE WRONG ANSWER, which is the one thing this module promises never to give"
        );
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
    /// `cargo test -- A B`**, which ran seven tests of the 741 there were then, and exited 0.
    /// So a driver counting
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
    /// collation reds `cargo xtask ci` alone with every test green.
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

    /// **The restore leaves cargo able to see it** — the property, pinned where it really lives.
    ///
    /// 🔴 **This test's plant was refuted at the code review and the guard was moved rather than
    /// re-argued.** It stood over an explicit `set_modified(SystemTime::now())` whose doc called
    /// it load-bearing; two layers measured that deleting the call leaves the suite green, because
    /// `std::fs::write` advances the mtime on its own. ⚠️ *A guard placed where the defect cannot
    /// occur* — and the plant recorded against it pushed the mtime BACKWARDS, which is a mutation
    /// nobody would make. What is asserted now is what `restore` genuinely guarantees.
    #[test]
    fn a_restore_leaves_a_timestamp_cargo_will_act_on() {
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
    /// *the whole suite passing, exit 0* without a store and *1 failed, exit 101* with one. The clock is the
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

    /// 🔴 **`from_args` was called by ONE line of `main.rs` and by NO test**, so the strict
    /// `targets`, the strict `require_store` and the `--expect` requirement were all carried by
    /// nothing — the review planted each of the three and the suite stayed green.
    #[test]
    fn the_command_line_builds_the_strict_mutation_and_refuses_a_missing_prediction() {
        let argv: Vec<String> = [
            "--file",
            "a.rs",
            "--anchor",
            "x",
            "--replacement",
            "y",
            "--expect",
            "red:3",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
        let m = parse(&argv).expect("a complete command line parses");
        assert_eq!(
            m.targets, EXPECTED_TEST_TARGETS,
            "strict, and not settable from the CLI"
        );
        assert!(m.require_store, "strict, and not settable from the CLI");
        assert!(!m.baseline, "opt-in");
        assert_eq!(m.expect, Expect::Red(Some(3)));

        let no_expect: Vec<String> = ["--file", "a.rs", "--anchor", "x", "--replacement", "y"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        assert!(
            parse(&no_expect).is_err(),
            "a prediction written after the fact is not a prediction — a missing --expect is a \
             refusal, and the review measured this defaulting silently to green"
        );

        let twice: Vec<String> = [
            "--file",
            "a.rs",
            "--file",
            "b.rs",
            "--anchor",
            "x",
            "--replacement",
            "y",
            "--expect",
            "green",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
        assert!(
            parse(&twice).is_err(),
            "a repeated flag is a refusal, not last-wins"
        );
    }

    /// 🔴 **The gates verdict is FOLDED into the outcome** — the review planted it discarded and
    /// the whole xtask suite stayed green, because the only test that reaches `measure` passed a
    /// stub that always says *green*. A stub that says *red* is what measures the fold.
    #[test]
    fn a_red_from_the_gates_alone_reaches_the_outcome() {
        let clean = line(503, 0, 0) + &line(161, 0, 0) + &line(92, 0, 0) + &line(0, 0, 0);
        let tests_green = read_run(&clean, Some(0), 4);
        assert_eq!(
            tests_green,
            Outcome::Green,
            "the premise: the TESTS are green"
        );

        let folded = fold(tests_green, false, false);
        assert_eq!(
            folded,
            Outcome::Red {
                tests: 0,
                clippy: false,
                gates: true
            },
            "a gate red with every test green is still a RED, and the outcome says which carrier \
             — measured: dropping a binary collation from a migration reds `ddl-collation` alone"
        );
        assert!(Expect::parse("red").expect("red").matches(&folded));
    }

    /// 🔴 **The store refusal was reachable by no test**, and the review measured its branch
    /// deletable with the suite green. Worse, the middle state PASSED: a `DATABASE_URL` that is
    /// set and unusable is not the string `"ABSENT"`, so the run proceeded — and the store-backed
    /// tests then PANIC rather than skip, so the red is the harness's.
    #[test]
    fn a_store_that_is_set_and_unusable_is_refused_like_an_absent_one() {
        assert!(
            Store::Reachable("127.0.0.1:13405".into())
                .refusal()
                .is_none()
        );
        let absent = Store::Absent.refusal().expect("unset is a refusal");
        assert!(absent.contains("passes by RETURNING"), "{absent}");
        let dead = Store::Unusable("nothing answers at 127.0.0.1:1".into())
            .refusal()
            .expect("set-and-unusable is a refusal TOO");
        assert!(
            dead.contains("PANIC"),
            "and it says why it is WORSE than absent, which is the half that used to pass: {dead}"
        );
        assert!(
            !store_is_reachable("127.0.0.1:1"),
            "and the probe CONNECTS — it read the shape of a URL and called it reachable until \
             the code review"
        );
    }

    /// 🔑 **THE END-TO-END, over a synthetic crate with its own `CARGO_TARGET_DIR`.**
    ///
    /// Story 5.12's finding is why this exists: its whole gate body was deletable with the xtask
    /// suite green, because every test attacked the helper and none drove the thing.
    ///
    /// 🔴 **AND IT SHIPPED GATED BEHIND AN ENV VAR THAT NOTHING SET, so the finding reproduced
    /// itself inside the test written to prevent it.** Three review layers measured the same
    /// thing: `measure()`'s entire body — and `run_mutation`'s, and `from_args`' — was deletable
    /// with 92 tests green, and `XTASK_MUTATE_E2E` appeared nowhere in `.github/`. ⚠️ The
    /// disclosure that it was gated is not a defence: *"the suite reports 92 either way, so the
    /// gate is the clock"* answers whether you can TELL it ran, never whether the code is
    /// carried. It runs unconditionally now; measured cost below. ⚠️ And story
    /// 6b.11's is why the tree is SYNTHETIC: *a gate green over the real tree says nothing about
    /// its own tests*. A nested cargo run does not deadlock (measured, 5.98 s) but it fights the
    /// outer run for `target/`, so this one is given a target directory of its own.
    #[test]
    fn the_driver_drives_a_real_cargo_run_end_to_end() {
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
            baseline: false,
        };
        let code =
            run_mutation(&dir, &mutation, |_| Ok((true, String::new()))).expect("the driver runs");
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
