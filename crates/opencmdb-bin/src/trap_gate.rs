//! The metrics harness — scores the trap corpus, and exists BEFORE any engine does (Story 4.6b).
//!
//! Not named `metrics`: `crate::metrics` is the Prometheus `/metrics` handler (D66), an unrelated
//! thing. This is the release gate's harness — it reads the committed trap corpus, feeds each trap
//! and its answer to the pure scoring algebra in `opencmdb_core::score` (story 4.6a), and reports
//! `{discovered, scored, failures}`.
//!
//! # It scores answers; it never runs a producer
//!
//! D19's build order is *"the metrics harness BEFORE the engine — a metric written after the engine
//! is bent to fit the engine"*. The structural guarantee here — the true and narrow one — is that
//! the harness **never calls a producer**, and that its shape is fixed by 4.6a's algebra: a future
//! engine must conform to the [`Outcome`] type, so the engine fits the metric, not the reverse.
//!
//! It scores a map of already-produced [`Outcome`]s, keyed by [`TrapId`]. A
//! `BTreeMap<TrapId, Outcome>` is DATA — no `poll`, no behaviour, no trait to stub — but do NOT
//! read that as "the metric can never be influenced by an engine": once Epic 5 fills this map from
//! engine output, the numbers depend on that output, exactly as D18 intends. What cannot happen is
//! the harness being SHAPED by the engine, because it consumes a fixed type and runs no producer.
//! The map is empty today; the vacuous run below is what that emptiness looks like.
//!
//! That is why AC1's "must not take an engine parameter" is honoured while AC6's "drive it over a
//! corpus whose traps are paired with outcomes" is still possible: an outcome is a result, not a
//! producer.
//!
//! # Vacuously green is not the same as green
//!
//! With no answers, every discovered trap is **discovered and not scored** — it produces no record.
//! `failures = 0` then, and the gate is green, but `scored = 0` and `discovered = 3` together say
//! plainly that nothing was measured. Without `discovered`, a function with an empty body would
//! report `{0, 0}` and pass — the exact vacuity story 4.1 removed from the fixtures gate
//! (`no fixtures — skipped`). A null engine that ABSTAINED on everything would be RED, not green:
//! D18's middle column demolishes it. Vacuously green means nothing ran, never "an abstainer ran".
//!
//! # `4.6a`'s [`ScoredRecord`] is not produced here
//!
//! It is exercised only by hand-built values until an engine exists (story 4.6a's own note). This
//! harness tallies; it does not persist a record per trap. That join is 4.6c/Epic-5 work.
//!
//! Wired into no runtime path — the release gate is not `/healthz`. `#![allow(dead_code)]` for the
//! same reason `fixtures.rs` and `arp_ping.rs` carry it: it is used by tests and by a later story,
//! not by the running binary.
//!
//! # Where it does NOT live, and why the number is not yet a CI gate
//!
//! The architecture places this at `xtask/src/gen_metrics.rs`, but the corpus reader (`read_traps`)
//! is in `bin`, and `xtask` cannot reach it without depending on `opencmdb-bin` — dragging sqlx,
//! axum and askama into the dev-tool runner (D56 makes xtask a dependency of nobody, and the reverse
//! has never been sanctioned). So the harness lives beside the reader it needs, and publishing its
//! number from `cargo xtask ci` is deferred with the obstacle recorded, not forced.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use opencmdb_core::score::{Outcome, Tally};
use opencmdb_core::trap::TrapId;

use crate::fixtures::{FixtureError, read_traps};

/// What one run of the corpus established: how many traps were found, how many had an answer to
/// score, and how many of those failed — per D18 column, inside the [`Tally`].
///
/// The number that blocks a release is [`Report::failures`] and it must be zero. `discovered` and
/// `scored` are not a fraction and are never divided — they exist so a reader can tell a passing
/// gate from a gate that measured nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    discovered: usize,
    tally: Tally,
}

impl Report {
    /// How many traps the walk found in the corpus. Zero means the harness never opened anything —
    /// the vacuity `discovered` exists to make visible.
    pub fn discovered(&self) -> usize {
        self.discovered
    }

    /// How many discovered traps had an answer to score. Zero with a non-zero `discovered` is the
    /// honest state before any engine exists: found, not measured.
    pub fn scored(&self) -> usize {
        self.tally.scored()
    }

    /// **The number that blocks a release. It must be zero.** Per D18 column inside [`Self::tally`].
    pub fn failures(&self) -> usize {
        self.tally.failures()
    }

    /// The per-column tally, for a caller that wants to know WHICH column fell.
    pub fn tally(&self) -> &Tally {
        &self.tally
    }

    /// The gate's verdict, as a method rather than a comment a caller must reconstruct.
    ///
    /// D18's one number — truth-table failures = 0 — plus a floor: **a run that discovered NOTHING
    /// does not pass.** An empty or wrong-but-present directory is vacuity, and `failures == 0` over
    /// zero traps must not read as success. A real corpus with no engine yet (discovered > 0,
    /// scored == 0) DOES pass — AC1 defines that as green; `scored()` is what tells a reader it was
    /// vacuous, not this predicate.
    pub fn passed(&self) -> bool {
        self.discovered > 0 && self.failures() == 0
    }
}

impl fmt::Display for Report {
    /// All three numbers on one line, so "0 failures" can never be read as "the gate passed" when
    /// nothing was scored (AC3).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} trap(s) discovered, {} scored, {} truth-table failure(s)",
            self.discovered,
            self.scored(),
            self.failures()
        )
    }
}

/// Score the trap corpus rooted at `traps_root` against a map of already-produced answers.
///
/// `traps_root` is a parameter, never [`crate::fixtures`]'s baked constant — that is what lets a
/// test point the harness at a scratch corpus (AC4). Discovery walks it for `.toml` trap files;
/// each trap is read and validated through [`read_traps`].
///
/// `answers` maps a [`TrapId`] to the [`Outcome`] something produced for it. A trap with no entry
/// is discovered and not scored. Today the map is empty for a real run; a test supplies contradicting
/// answers to prove the gate can be red.
///
/// **One interaction to know:** [`read_traps`] resolves each trap's `replay` field against the
/// BAKED corpus root, not against `traps_root`. So a scratch trap corpus may only reference replay
/// streams that exist in the committed corpus (e.g. `scenario/replay/minimal.jsonl`). That is
/// enough for AC6 — a scratch trap varies its expectation, not its stream — and it is a real limit,
/// recorded in `deferred-work.md`.
pub fn score_corpus(
    traps_root: &Path,
    answers: &BTreeMap<TrapId, Outcome>,
) -> Result<Report, FixtureError> {
    let mut tally = Tally::default();
    // Every trap id seen so far, and the file it came from. `TrapFile::validate` enforces
    // uniqueness WITHIN a file; a `TrapId` is the key an answer is scored against, so one id in two
    // files would score a single outcome twice — the mirror of the cross-stream `obs_id` rule.
    let mut seen: BTreeMap<TrapId, PathBuf> = BTreeMap::new();
    // Which answers were actually used. A key matching no discovered trap is a producer emitting an
    // outcome the gate would otherwise ignore silently — a walk that quietly sees less.
    let mut used: std::collections::BTreeSet<TrapId> = std::collections::BTreeSet::new();

    for trap_file in discover_trap_files(traps_root)? {
        let traps = read_traps(&trap_file)?;
        for trap in &traps.trap {
            if let Some(first) = seen.insert(trap.id.clone(), trap_file.clone()) {
                return Err(FixtureError::DuplicateTrapId {
                    trap: trap.id.0.clone(),
                    first,
                    second: trap_file.clone(),
                });
            }
            if let Some(outcome) = answers.get(&trap.id) {
                tally.record(&trap.expect, outcome);
                used.insert(trap.id.clone());
            }
        }
    }

    // An answer for a trap that does not exist is a producer/corpus mismatch, not a silent no-op.
    let unmatched: Vec<&TrapId> = answers.keys().filter(|id| !used.contains(*id)).collect();
    if let Some(orphan) = unmatched.first() {
        return Err(FixtureError::AnswerForUnknownTrap {
            trap: orphan.0.clone(),
            count: unmatched.len(),
        });
    }

    Ok(Report {
        discovered: seen.len(),
        tally,
    })
}

/// Walk `root` recursively for `.toml` trap files, in sorted order.
///
/// It refuses a symlink and a foreign extension, and does NOT swallow a read error — *"walks that
/// quietly see less"* were the recurring defect of stories 4.1 and 4.3, so a subtree it cannot read
/// is an error, not a smaller result. `README.md` is exempt at any depth, exactly as the corpus
/// lock's orphan rule exempts it, so documenting a directory does not turn the gate red.
///
/// This is the harness's OWN walk, not the `#[cfg(test)]` walks in `fixtures.rs`: those are test
/// helpers (`walk_replay_streams` scans replay streams, not traps), and promoting one would move
/// its callers for no gain here.
fn discover_trap_files(root: &Path) -> Result<Vec<PathBuf>, FixtureError> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|source| FixtureError::Io {
            path: dir.clone(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| FixtureError::Io {
                path: dir.clone(),
                source,
            })?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|source| FixtureError::Io {
                path: path.clone(),
                source,
            })?;
            if file_type.is_symlink() {
                // A symlink neither smuggles a file in nor is walked out of the corpus, but it must
                // not pass unnoticed either. `Io` with a synthetic error keeps the one error type.
                return Err(FixtureError::Io {
                    path: path.clone(),
                    source: std::io::Error::other(format!(
                        "the corpus must contain its own bytes, not a symlink: {}",
                        path.display()
                    )),
                });
            }
            if file_type.is_dir() {
                stack.push(path);
                continue;
            }
            if path.file_name().and_then(|n| n.to_str()) == Some("README.md") {
                continue;
            }
            let is_toml = path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("toml"));
            if !is_toml {
                return Err(FixtureError::Io {
                    path: path.clone(),
                    source: std::io::Error::other(format!(
                        "only .toml trap files and README.md belong in a trap corpus: {}",
                        path.display()
                    )),
                });
            }
            found.push(path);
        }
    }
    // Sorted so a discovery run is deterministic regardless of readdir order.
    found.sort();
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::fixtures_dir;
    use opencmdb_core::gap::AbstentionCause;
    use opencmdb_core::score::Column;
    use opencmdb_core::trap::RuleId;

    fn committed_traps_root() -> PathBuf {
        fixtures_dir().join("scenario/traps")
    }

    /// A private scratch directory per test. A shared constant path races between concurrent
    /// `cargo test` runs — the same reasoning `fixtures.rs`'s own `scratch_dir` states.
    fn scratch_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("opencmdb-trap-gate-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    // ── The committed corpus, no answers: vacuously green, and visibly so (AC1, AC2, AC3, AC5) ──

    #[test]
    fn the_committed_corpus_is_discovered_and_scored_by_nothing() {
        let report = score_corpus(&committed_traps_root(), &BTreeMap::new())
            .expect("the committed corpus reads");

        // Discovered is what makes the zeros honest: `example.toml` carries three traps.
        assert_eq!(report.discovered(), 3, "the walk must open the corpus");
        assert_eq!(
            report.scored(),
            0,
            "no answer producer exists, so nothing is scored"
        );
        assert_eq!(
            report.failures(),
            0,
            "and a gate that scored nothing has no failures"
        );
    }

    /// The vacuous state must not read like a passing gate — all three numbers on one line.
    #[test]
    fn the_report_says_plainly_that_nothing_was_scored() {
        let report = score_corpus(&committed_traps_root(), &BTreeMap::new()).unwrap();
        let rendered = report.to_string();
        assert!(rendered.contains("3 trap(s) discovered"), "{rendered}");
        assert!(rendered.contains("0 scored"), "{rendered}");
        assert!(rendered.contains("0 truth-table failure(s)"), "{rendered}");
    }

    /// A discovered-but-unscored trap is counted in `discovered`, never dropped, and never scored
    /// as a phantom pass (AC5). One committed trap gets an answer; the other two are still counted.
    #[test]
    fn a_trap_with_no_answer_is_discovered_but_not_scored() {
        let mut answers = BTreeMap::new();
        // A correct answer for one trap, so `scored` is 1 while `discovered` stays 3.
        answers.insert(
            TrapId("example-must-abstain".into()),
            Outcome::Abstained {
                cause: AbstentionCause::NoObservedValue,
            },
        );
        let report = score_corpus(&committed_traps_root(), &answers).unwrap();
        assert_eq!(report.discovered(), 3);
        assert_eq!(report.scored(), 1, "only the answered trap is scored");
        assert_eq!(report.failures(), 0, "and its answer is correct");
    }

    // ── The gate can be shown to fail, per D18 column (AC6) ──────────────────

    /// A scratch trap file referencing a COMMITTED replay stream, so `read_traps`' obs_id
    /// cross-check (which resolves the stream against the baked root) still passes.
    fn write_scratch_traps(dir: &Path, body: &str) -> PathBuf {
        let path = dir.join("scratch-traps.toml");
        std::fs::write(&path, body).unwrap();
        path
    }

    /// One scratch corpus, one trap per D18 column, each paired with a CONTRADICTING answer — and a
    /// failure counted in each column. This is the demonstration that the gate is red-able: *"a gate
    /// that cannot be shown to fail is decoration."*
    #[test]
    fn each_column_can_be_driven_red() {
        let dir = scratch_dir("trap-gate-red");
        // All three traps judge observations that exist in the committed `minimal.jsonl`, so
        // `read_traps` validates them; only the expectation differs across the three.
        write_scratch_traps(
            &dir,
            r#"
[[trap]]
id = "red-must-merge"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "a scratch trap that expects a merge, so an abstention fails the must-merge column."
expect = { must-merge = { rule = "l1-exact-mac" } }

[[trap]]
id = "red-must-not-merge"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "a scratch trap that forbids a merge, so a merge fails the must-not-merge column."
expect = { must-not-merge = { rule = "l1-distinct-mac" } }

[[trap]]
id = "red-must-abstain"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000002"]
reason = "a scratch trap that expects an abstention, so a decision fails the must-abstain column."
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        );

        let mut answers = BTreeMap::new();
        // must-merge, answered with an abstention → cowardice, the middle column.
        answers.insert(
            TrapId("red-must-merge".into()),
            Outcome::Abstained {
                cause: AbstentionCause::NoObservedValue,
            },
        );
        // must-not-merge, answered with a merge → the false merge.
        answers.insert(
            TrapId("red-must-not-merge".into()),
            Outcome::Merged {
                rule: RuleId("l2-uplink-agrees".into()),
            },
        );
        // must-abstain, answered with a merge → a guess on the ambiguous case.
        answers.insert(
            TrapId("red-must-abstain".into()),
            Outcome::Merged {
                rule: RuleId("l2-uplink-agrees".into()),
            },
        );

        let report = score_corpus(&dir, &answers).expect("the scratch corpus reads");
        assert_eq!(report.discovered(), 3);
        assert_eq!(report.scored(), 3);
        assert_eq!(
            report.failures(),
            3,
            "one failure in each of D18's three columns"
        );
        assert_eq!(report.tally().failures_in(Column::MustMerge), 1);
        assert_eq!(report.tally().failures_in(Column::MustNotMerge), 1);
        assert_eq!(report.tally().failures_in(Column::MustAbstain), 1);

        std::fs::remove_dir_all(&dir).ok();
    }

    // ── The walk sees everything and swallows nothing (AC2, Task 2) ──────────

    #[test]
    fn the_walk_refuses_a_foreign_extension() {
        let dir = scratch_dir("trap-gate-foreign");
        std::fs::write(dir.join("not-a-trap.txt"), "x").unwrap();
        let err = score_corpus(&dir, &BTreeMap::new())
            .expect_err("a non-.toml file in a trap corpus must fail the walk");
        assert!(err.to_string().contains("only .toml"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn the_walk_exempts_readme_at_any_depth() {
        let dir = scratch_dir("trap-gate-readme");
        std::fs::write(dir.join("README.md"), "prose about the corpus").unwrap();
        std::fs::write(
            dir.join("t.toml"),
            r#"
[[trap]]
id = "only-trap"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000002"]
reason = "a single valid trap alongside a README that the walk must not choke on."
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        )
        .unwrap();
        let report = score_corpus(&dir, &BTreeMap::new()).expect("a README must be exempt");
        assert_eq!(report.discovered(), 1);
        std::fs::remove_dir_all(&dir).ok();
    }

    /// A missing root errors — and the message says the path was unreadable, not merely "Io", so a
    /// caller can tell a disk failure from a corpus authoring mistake.
    #[test]
    fn a_missing_root_is_an_error_not_an_empty_result() {
        let missing = scratch_dir("trap-gate-missing").join("nope");
        let err = score_corpus(&missing, &BTreeMap::new())
            .expect_err("a walk that cannot read its root must not report zero traps");
        match &err {
            FixtureError::Io { path, .. } => {
                assert!(path.ends_with("nope"), "names the path: {err}")
            }
            other => panic!("expected an Io error naming the unreadable root, got {other:?}"),
        }
    }

    /// An empty-but-present directory is vacuity, not a pass: `discovered == 0`, and `passed()` is
    /// false. Without this, any caller pointing the harness at the wrong existing directory reads a
    /// green gate. The committed-corpus test proves the positive; this proves the floor.
    #[test]
    fn an_empty_corpus_does_not_pass() {
        let dir = scratch_dir("trap-gate-empty");
        let report = score_corpus(&dir, &BTreeMap::new()).expect("an empty directory reads");
        assert_eq!(report.discovered(), 0);
        assert_eq!(report.failures(), 0);
        assert!(
            !report.passed(),
            "0 failures over 0 traps is vacuity, never a pass"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// `passed()` is the D18 gate — `failures == 0` — plus a floor of `discovered > 0`. The floor
    /// is what an EMPTY corpus fails; a real corpus with no engine yet still PASSES, because AC1
    /// defines the vacuous-over-a-real-corpus run as green (the `scored` number is what tells a
    /// human it was vacuous, not `passed()`). A run with a failure does not pass.
    #[test]
    fn passed_is_the_failures_gate_with_a_discovered_floor() {
        // Vacuous over the committed corpus: discovered, nothing scored, and GREEN (AC1).
        let vacuous = score_corpus(&committed_traps_root(), &BTreeMap::new()).unwrap();
        assert!(
            vacuous.passed(),
            "a real corpus with no engine yet is green (AC1)"
        );
        assert_eq!(
            vacuous.scored(),
            0,
            "…and `scored` is what shows it was vacuous"
        );
    }

    /// A run mixing a CORRECT answer and a WRONG one — the discriminating case. The offending trap
    /// is second, behind the valid one, so a harness that stopped after the first would still be
    /// caught. Only the wrong answer enters `failures`; the correct one stays out.
    #[test]
    fn a_correct_answer_stays_out_of_failures_while_a_wrong_one_enters() {
        let dir = scratch_dir("trap-gate-mixed");
        write_scratch_traps(
            &dir,
            r#"
[[trap]]
id = "mixed-correct"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000002"]
reason = "a valid trap answered correctly, so it must not appear in the failure count."
expect = { must-abstain = { cause = "NoObservedValue" } }

[[trap]]
id = "mixed-wrong"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "a must-merge trap answered with an abstention, so it must fail the must-merge column."
expect = { must-merge = { rule = "l1-exact-mac" } }
"#,
        );
        let mut answers = BTreeMap::new();
        answers.insert(
            TrapId("mixed-correct".into()),
            Outcome::Abstained {
                cause: AbstentionCause::NoObservedValue,
            },
        );
        answers.insert(
            TrapId("mixed-wrong".into()),
            Outcome::Abstained {
                cause: AbstentionCause::NoObservedValue,
            },
        );
        let report = score_corpus(&dir, &answers).expect("the mixed corpus reads");
        assert_eq!(report.scored(), 2);
        assert_eq!(report.failures(), 1, "only the wrong answer fails");
        assert_eq!(report.tally().failures_in(Column::MustMerge), 1);
        assert_eq!(report.tally().failures_in(Column::MustAbstain), 0);
        assert!(!report.passed());
        std::fs::remove_dir_all(&dir).ok();
    }

    // ── The two corpus-integrity guards this story's review added ────────────

    /// The symlink guard, which was correct but unproven. A symlink in the corpus errors the walk
    /// rather than being followed — the *"walks that quietly see less"* defect of 4.1/4.3.
    #[test]
    #[cfg(unix)]
    fn the_walk_refuses_a_symlink() {
        let dir = scratch_dir("trap-gate-symlink");
        // A valid trap first, and the symlink second — a walk that stopped early would miss it.
        std::fs::write(
            dir.join("real.toml"),
            r#"
[[trap]]
id = "real"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000002"]
reason = "a valid trap alongside a symlink the walk must refuse rather than follow."
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        )
        .unwrap();
        std::os::unix::fs::symlink("/etc/hostname", dir.join("link.toml")).unwrap();
        let err = score_corpus(&dir, &BTreeMap::new())
            .expect_err("a symlink in the corpus must fail the walk");
        assert!(err.to_string().contains("symlink"), "{err}");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Discovery is recursive: a trap in a NESTED directory is found. Nothing else exercised the
    /// descent, so "at any depth" was an untested claim.
    #[test]
    fn discovery_descends_into_subdirectories() {
        let dir = scratch_dir("trap-gate-nested");
        let nested = dir.join("family").join("randomized-mac");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(
            nested.join("deep.toml"),
            r#"
[[trap]]
id = "deep"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000002"]
reason = "a trap two directories down, to prove the walk descends rather than scanning the top."
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        )
        .unwrap();
        let report = score_corpus(&dir, &BTreeMap::new()).expect("a nested corpus reads");
        assert_eq!(report.discovered(), 1, "the nested trap must be found");
        std::fs::remove_dir_all(&dir).ok();
    }

    /// One trap id in two files is refused across the corpus, naming both — `read_traps` only
    /// dedups within a file, and one id scored against one answer twice is a false gate.
    #[test]
    fn a_trap_id_repeated_across_two_files_is_refused() {
        let dir = scratch_dir("trap-gate-dup-id");
        let trap = |id: &str| {
            format!(
                r#"
[[trap]]
id = "{id}"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000002"]
reason = "a trap whose id is deliberately duplicated in a sibling file to trip the guard."
expect = {{ must-abstain = {{ cause = "NoObservedValue" }} }}
"#
            )
        };
        std::fs::write(dir.join("a.toml"), trap("shared-id")).unwrap();
        std::fs::write(dir.join("b.toml"), trap("shared-id")).unwrap();
        let err = score_corpus(&dir, &BTreeMap::new())
            .expect_err("a trap id in two files must be refused");
        match &err {
            FixtureError::DuplicateTrapId { trap, .. } => assert_eq!(trap, "shared-id"),
            other => panic!("expected DuplicateTrapId, got {other:?}"),
        }
        assert!(
            err.to_string().contains("a.toml"),
            "names the first file: {err}"
        );
        assert!(
            err.to_string().contains("b.toml"),
            "names the second file: {err}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// An answer for a trap that does not exist is refused, not silently dropped — a producer
    /// emitting an outcome the gate cannot place is a mismatch.
    #[test]
    fn an_answer_for_an_unknown_trap_is_refused() {
        let mut answers = BTreeMap::new();
        answers.insert(
            TrapId("no-such-trap".into()),
            Outcome::Merged {
                rule: RuleId("l1-exact-mac".into()),
            },
        );
        let err = score_corpus(&committed_traps_root(), &answers)
            .expect_err("an answer naming no trap must be refused");
        match &err {
            FixtureError::AnswerForUnknownTrap { trap, count } => {
                assert_eq!(trap, "no-such-trap");
                assert_eq!(*count, 1);
            }
            other => panic!("expected AnswerForUnknownTrap, got {other:?}"),
        }
    }
}
