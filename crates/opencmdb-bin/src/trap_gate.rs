//! The metrics harness — scores the trap corpus, and exists BEFORE any engine does (Story 4.6b).
//!
//! Not named `metrics`: `crate::metrics` is the Prometheus `/metrics` handler (D66), an unrelated
//! thing. This is the release gate's harness — it reads the committed trap corpus, feeds each trap
//! and its answer to the pure scoring algebra in `opencmdb_core::score` (story 4.6a), and reports
//! `{discovered, scored, failures}`.
//!
//! # [`score_corpus`] scores answers; it never runs a producer
//!
//! D19's build order is *"the metrics harness BEFORE the engine — a metric written after the engine
//! is bent to fit the engine"*. The structural guarantee — the true and narrow one — is that
//! [`score_corpus`] **never calls a producer**, and that its shape is fixed by 4.6a's algebra: an
//! engine must conform to the [`Outcome`] type, so the engine fits the metric, not the reverse.
//!
//! ⚠️ **Since story 5.7 a producer EXISTS, and it is deliberately not in this file.**
//! [`crate::l1_runner`] runs the real L1 engine over a trap corpus and returns the map
//! [`score_corpus`] takes. It lives in its own module precisely so the sentence above stays a
//! FILE-level property instead of degrading into a per-function promise on the day it first has
//! something to promise about. The seam between the two is a `BTreeMap<TrapId, Answer>`, which is
//! DATA — no `poll`, no behaviour, no trait to stub — and [`score_corpus`] takes no engine
//! parameter, no callback and no closure.
//! _(This paragraph said the seam is a `BTreeMap<TrapId, Outcome>` and that `score_corpus`'s
//! signature and body are "unchanged by that story". True of story 5.7; story 5.8 widened the
//! map's VALUE to [`Answer`] and with it the signature — the arity, and the guarantee above, are
//! what stayed. The twin of this sentence in `l1_runner.rs` was corrected in 5.8's own commit and
//! this copy was missed, which is the defect its AC8 warns about.)_
//!
//! Do NOT read the guarantee as "the metric can never be influenced by an engine": now that the map
//! is filled from engine output, the numbers depend on that output, exactly as D18 intends. What
//! cannot happen is the harness being SHAPED by the engine, because it consumes a fixed type and
//! runs no producer.
//!
//! That is why AC1's "must not take an engine parameter" is honoured while AC6's "drive it over a
//! corpus whose traps are paired with outcomes" is still possible: an outcome is a result, not a
//! producer.
//!
//! [`Answer`]: opencmdb_core::score::Answer
//! [`Outcome`]: opencmdb_core::score::Outcome
//!
//! # Story 5.8 widened the map's VALUE, and that does not spend 4.6b's AC1 either
//!
//! [`score_corpus`] now takes `&BTreeMap<TrapId, Answer>` where it took
//! `&BTreeMap<TrapId, Outcome>`. Its **arity is unchanged**, and so is the guarantee above: an
//! [`Answer`] is still DATA — no trait, no callback, no closure, nothing this file can call. 4.6b's
//! AC1 asks that the harness *"must not require an engine to exist"* (`epics.md:1055`), and it does
//! not: an empty map still scores nothing and still passes.
//!
//! What the wider value buys is the one thing absence cannot express — *"a producer ran and could
//! not ask, for this reason"*. Before it, a trap left the denominator with no reason attached and a
//! green gate could mean *"we did not ask the question"*. `epics.md`'s story 5.8 forbids exactly
//! that, and [`Report::passed`] is now blocked by [`Report::unanswered`] as it is by the other
//! three buckets.
//!
//! # Vacuously green is not the same as green
//!
//! With no answers, every discovered trap is **discovered and not scored** — it produces no record.
//! `failures = 0` then, and the gate is green, but `scored = 0` and `discovered = 3` together say
//! plainly that nothing was measured. That is no longer the committed corpus's state — it scores 13
//! of 24 since story 5.7, and since story 5.8 the other **11 are a blocking bucket**
//! ([`Report::unanswered`]), so the committed gate does not pass and will not until Epic 6
//! implements `l2-*`. An empty answers map still looks the old way, and that is deliberate: 4.6b's
//! AC1 keeps it green, with [`Report::unaccounted`] naming the state. Without `discovered`, a
//! function with an empty body would
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

use opencmdb_core::score::{Answer, Column, Tally, TrapVerdict, UnanswerableCause, run_trap};
use opencmdb_core::trap::{IncompleteFamily, RuleId, Trap, TrapId, incomplete_families};

use crate::fixtures::{FixtureError, read_traps};

/// One trap whose verdict was RIGHT but whose decision fired the WRONG rule (story 4.7a).
///
/// D46b's surviving criterion: *"same output, different reason… BOTH JOBS GREEN… the worst kind"*.
/// The truth-table [`Tally`] passes this trap — its verdict is correct — so the mismatch is carried
/// here, separately, and it names BOTH rules so a red gate is debuggable without opening the corpus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMismatch {
    /// The trap whose answer reached the right column by the wrong rule.
    pub trap: TrapId,
    /// The D18 column the trap belongs to — always a decision column (`must-merge`/`must-not-merge`),
    /// because only a decision carries a rule to be wrong.
    pub column: Column,
    /// The rule the trap's author said must fire (or oppose the merge).
    pub expected: RuleId,
    /// The rule the answer actually fired.
    pub actual: RuleId,
}

/// One trap a producer RAN and could not put to its engine at all (story 5.8).
///
/// The sibling of [`RuleMismatch`], and the fourth bucket `epics.md` requires: *"counted as NOT
/// PASSING in a fourth named bucket, beside truth-table failures, rule mismatches and incomplete
/// families — they never silently leave the denominator"*.
///
/// ⚠️ **Not an abstention and not a failure.** A truth-table failure means the engine answered
/// WRONG; a wrong rule means it answered right for the wrong reason; this means it was **never
/// asked**. All three block, and they are three different pieces of news — which is why this is a
/// bucket of its own rather than a tenth cell or a fourth column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unanswered {
    /// The trap the producer declined to put to its engine.
    pub trap: TrapId,
    /// The D18 column the trap belongs to, so a red gate is readable per column without reopening
    /// the corpus. Unlike [`RuleMismatch::column`] this may be **any** of the three: a trap can be
    /// unanswerable in the `must-abstain` column, where no decision rule exists to be wrong.
    pub column: Column,
    /// Why the producer could not ask. Its three variants are the measured classes — 8 / 2 / 1 over
    /// the committed corpus.
    pub cause: UnanswerableCause,
}

/// What one run of the corpus established: how many traps were found, how many had an answer to
/// score, how many of those failed the truth table — per D18 column, inside the [`Tally`] — which
/// ones reached the right verdict by the wrong rule (story 4.7a), which trap FAMILIES were tested
/// in only one decision form (story 4.7b), and which traps a producer RAN and could not ask about
/// at all (story 5.8).
///
/// The numbers that block a release are [`Report::failures`], [`Report::rule_mismatches`],
/// [`Report::incomplete_families`] AND [`Report::unanswered`]; all must be empty. `discovered` and
/// `scored` are not a fraction and are never divided — they exist so a reader can tell a passing
/// gate from a gate that measured nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    discovered: usize,
    tally: Tally,
    rule_mismatches: Vec<RuleMismatch>,
    incomplete_families: Vec<IncompleteFamily>,
    unanswered: Vec<Unanswered>,
}

impl Report {
    /// How many traps the walk found in the corpus. Zero means the harness never opened anything —
    /// the vacuity `discovered` exists to make visible.
    pub fn discovered(&self) -> usize {
        self.discovered
    }

    /// How many discovered traps had an answer to score.
    ///
    /// Zero with a non-zero `discovered` was the honest state before any engine existed: found, not
    /// measured. Since story 5.7 the committed corpus scores **13 of 24** — the thirteen traps whose
    /// expected rule is `l1-*`. The gap is not an error: eleven traps are unanswerable at this
    /// cascade level and stay in `discovered` on purpose, so the exclusion is visible. Since story
    /// 5.8 those eleven are also [`Self::unanswered`] — a bucket that BLOCKS — so the gap is no
    /// longer merely visible, it is what keeps the committed gate from passing.
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

    /// The traps that reached the right verdict by the wrong rule (story 4.7a). Empty is the
    /// passing state; each entry names both rules. Separate from [`Self::failures`] on purpose — a
    /// wrong rule is not a truth-table failure (the verdict passed), it is D46b's distinct
    /// *"same output, different reason"* failure.
    pub fn rule_mismatches(&self) -> &[RuleMismatch] {
        &self.rule_mismatches
    }

    /// The trap families tested in only one decision form (story 4.7b). Empty is the passing state;
    /// each entry names the family and which pole it has. Separate from the two failure buckets above
    /// — a one-sided family is a corpus-SHAPE defect, orthogonal to any answer: it means the gate was
    /// never shown it can fail the family the other way.
    pub fn incomplete_families(&self) -> &[IncompleteFamily] {
        &self.incomplete_families
    }

    /// The traps a producer RAN and could not put to its engine (story 5.8). Empty is the passing
    /// state; each entry names the trap, its column and WHY.
    ///
    /// The fourth blocking bucket, and the one that stops a green gate from meaning *"we did not
    /// ask the question"*. Separate from the three above because it is separate news: not a wrong
    /// answer, not a right answer for the wrong reason, not a one-sided family — **no answer at
    /// all**.
    pub fn unanswered(&self) -> &[Unanswered] {
        &self.unanswered
    }

    /// How many traps were unanswerable IN ONE COLUMN.
    ///
    /// The counterpart of [`Tally::scored_in`], and the two are meant to be read together:
    /// **when the answers map accounts for every discovered trap** (`unaccounted() == 0`),
    /// `scored_in(c) + unanswered_in(c)` is how many traps the corpus carries in that column, so a
    /// trap that vanished from BOTH is the one thing the pair can catch and `discovered` alone
    /// cannot.
    ///
    /// ⚠️ The qualifier is load-bearing, not hedging: with an EMPTY map over the committed corpus —
    /// the state 4.6b's AC1 keeps green — all three columns read `0 + 0` against 10 / 11 / 3, and
    /// the identity is false in every one of them. It holds of a TOTAL map, which is what
    /// `l1_runner::l1_answers` produces.
    pub fn unanswered_in(&self, column: Column) -> usize {
        self.unanswered
            .iter()
            .filter(|u| u.column == column)
            .count()
    }

    /// Discovered traps that were neither scored nor declared unanswerable — **reported, and
    /// deliberately NOT blocking.**
    ///
    /// This is what an EMPTY answers map looks like: 4.6b's AC1 requires the harness to be *"GREEN
    /// vacuously… it must not require an engine to exist"*, so a trap simply absent from the map is
    /// not bucketed and does not block. `scored() == 0` beside `discovered() > 0` is what tells a
    /// reader that run measured nothing, exactly as before story 5.8.
    ///
    /// It is an accessor and nothing renders it. The question it makes measurable — *should a
    /// non-empty but PARTIAL map block, i.e. should a producer that ran be required to account for
    /// every discovered trap?* — is registered in `deferred-work.md` rather than decided here:
    /// deciding it would overturn an epic-level acceptance criterion.
    pub fn unaccounted(&self) -> usize {
        self.discovered
            .saturating_sub(self.scored())
            .saturating_sub(self.unanswered.len())
    }

    /// The gate's verdict, as a method rather than a comment a caller must reconstruct.
    ///
    /// D18's one number — truth-table failures = 0 — plus D46b's `(verdict, rule)` criterion (no
    /// wrong-rule trap, story 4.7a), plus the corpus-completeness criterion (no one-sided family,
    /// story 4.7b), plus the unanswerable bucket (story 5.8), plus a floor: **a run that discovered
    /// NOTHING does not pass.** An empty or wrong-but-present directory is vacuity, and
    /// `failures == 0` over zero traps must not read as success.
    ///
    /// # "No producer ran" and "a producer declined" are different, and only the second blocks
    ///
    /// A real corpus with an EMPTY answers map (discovered > 0, scored == 0, unanswered == 0) still
    /// DOES pass — 4.6b's AC1 defines that as green, and `scored()` is what tells a reader it was
    /// vacuous, not this predicate. What story 5.8 added blocks the other case: a producer that RAN
    /// and named traps it could not ask about. The distinction is carried by the map itself — an
    /// absent key is the first, an `Answer::Unanswerable` is the second — so this predicate never
    /// has to guess which happened.
    pub fn passed(&self) -> bool {
        self.discovered > 0
            && self.failures() == 0
            && self.rule_mismatches.is_empty()
            && self.incomplete_families.is_empty()
            && self.unanswered.is_empty()
    }
}

impl fmt::Display for Report {
    /// All three numbers on one line, so "0 failures" can never be read as "the gate passed" when
    /// nothing was scored (4.6b AC3). **Three** count suffixes follow on that SAME first line, in a
    /// fixed order — `", K wrong-rule failure(s)"` (story 4.7a), `", J incomplete-famil{y|ies}"`
    /// (story 4.7b), then `", N unanswerable trap(s)"` (story 5.8) — each appended only when
    /// non-zero, so the line alone can never read as a pass while
    /// [`Report::passed`] is false, and the nominal first line stays byte-for-byte unchanged (its
    /// 4.6b-asserted substrings are stable). The order is fixed so the rendered string is
    /// deterministic. Each wrong-rule mismatch then follows on its own line (naming both rules), then
    /// each incomplete family on its own line (naming the family and the missing pole).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} trap(s) discovered, {} scored, {} truth-table failure(s)",
            self.discovered,
            self.scored(),
            self.failures()
        )?;
        // Each distinct red must also show on the first line, in a FIXED order (wrong-rule, then
        // incomplete-family): a reader — or a `grep` — that trusts only that line would otherwise
        // read "0 truth-table failure(s)" as a pass while `passed()` is false. Appended only when
        // non-empty; the substrings above are added-to, never rewritten, so the 4.6b guard holds.
        if !self.rule_mismatches.is_empty() {
            write!(f, ", {} wrong-rule failure(s)", self.rule_mismatches.len())?;
        }
        if !self.incomplete_families.is_empty() {
            let n = self.incomplete_families.len();
            let noun = if n == 1 {
                "incomplete-family"
            } else {
                "incomplete-families"
            };
            write!(f, ", {n} {noun}")?;
        }
        // Story 5.8's suffix is THIRD and last, so the two above stay byte-identical and every
        // substring assertion 4.6b/4.7a/4.7b wrote on them keeps passing. It carries a NOUN, as
        // both its siblings do — `", 1 unanswerable"` alone read as an adjective with nothing to
        // qualify.
        if !self.unanswered.is_empty() {
            write!(f, ", {} unanswerable trap(s)", self.unanswered.len())?;
        }
        for mismatch in &self.rule_mismatches {
            write!(
                f,
                "\n  wrong rule: trap `{}` ({}): expected rule `{}`, got `{}`",
                mismatch.trap.0,
                mismatch.column.as_str(),
                mismatch.expected.0,
                mismatch.actual.0
            )?;
        }
        for family in &self.incomplete_families {
            let poles = match (family.has_merge, family.has_not_merge) {
                (true, false) => "has must-merge, missing must-not-merge".to_string(),
                (false, true) => "has must-not-merge, missing must-merge".to_string(),
                // The abstain-only case (DR1): a family with no decision pole at all.
                _ => "has neither pole (needs must-merge and must-not-merge)".to_string(),
            };
            write!(f, "\n  incomplete family `{}`: {poles}", family.family.0)?;
        }
        for unanswered in &self.unanswered {
            let why = match &unanswered.cause {
                UnanswerableCause::LevelNotImplemented { expected } => format!(
                    "its author named rule `{}`, at a cascade level this engine does not implement",
                    expected.0
                ),
                UnanswerableCause::NoLevelToRouteOn => {
                    "its expectation names a cause and no rule, so there is no level to route on"
                        .to_string()
                }
                UnanswerableCause::NoPairUnderJudgement => {
                    "it does not name a pair, so no identity question can be formed".to_string()
                }
            };
            write!(
                f,
                "\n  unanswerable: trap `{}` ({}): {why}",
                unanswered.trap.0,
                unanswered.column.as_str()
            )?;
        }
        // NFR4's status, tied to the bucket rather than written unconditionally — and VENTILATED
        // by cause, because one closer does not close all three classes.
        //
        // ⚠️ This sentence used to attribute the whole count to *"this cascade level — closed by
        // Epic 6"*. Story 5.8's own code review measured that false: `NoLevelToRouteOn` and
        // `NoPairUnderJudgement` do not depend on a level at all (`Expectation::rule()` is `None`
        // for any `must-abstain`, at every level present and future), so Epic 6 takes the bucket
        // from 11 to 3 and the old sentence would have gone on naming as its closer the epic that
        // had just shipped. Each class now names the story that actually closes it, and a class
        // with no members renders nothing — so the line narrows itself as the work lands instead of
        // relying on someone remembering to edit it.
        if !self.unanswered.is_empty() {
            let level = self
                .unanswered
                .iter()
                .filter(|u| matches!(u.cause, UnanswerableCause::LevelNotImplemented { .. }))
                .count();
            let no_level = self.unanswered.len() - level;
            write!(
                f,
                "\n  NFR4 NOT MET at this epic: D18 places the gate at the DEVICE level, and {} \
                 trap(s) went unanswered.",
                self.unanswered.len()
            )?;
            if level > 0 {
                write!(
                    f,
                    " {level} at a cascade level this engine does not implement (closed by Epic 6)."
                )?;
            }
            if no_level > 0 {
                write!(
                    f,
                    " {no_level} with no level to route on at all (closed by story 5.14 / Epic 6, \
                     where an abstention first has a producer the corpus can judge)."
                )?;
            }
        }
        Ok(())
    }
}

/// Score the trap corpus rooted at `traps_root` against a map of already-produced answers.
///
/// `traps_root` is a parameter, never [`crate::fixtures`]'s baked constant — that is what lets a
/// test point the harness at a scratch corpus (AC4). Discovery walks it for `.toml` trap files;
/// each trap is read and validated through [`read_traps`].
///
/// `answers` maps a [`TrapId`] to the [`Answer`] a producer gave: either an [`Outcome`] it reached,
/// or a named reason it could not ask at all. A trap with **no entry** is discovered and neither
/// scored nor bucketed — the vacuous state 4.6b's AC1 keeps green ([`Report::unaccounted`]).
///
/// [`Outcome`]: opencmdb_core::score::Outcome
///
/// **One interaction to know:** [`read_traps`] resolves each trap's `replay` field against the
/// BAKED corpus root, not against `traps_root`. So a scratch trap corpus may only reference replay
/// streams that exist in the committed corpus (e.g. `scenario/replay/minimal.jsonl`). That is
/// enough for AC6 — a scratch trap varies its expectation, not its stream — and it is a real limit,
/// recorded in `deferred-work.md`.
pub fn score_corpus(
    traps_root: &Path,
    answers: &BTreeMap<TrapId, Answer>,
) -> Result<Report, FixtureError> {
    let mut tally = Tally::default();
    let mut rule_mismatches: Vec<RuleMismatch> = Vec::new();
    let mut unanswered: Vec<Unanswered> = Vec::new();
    // Every discovered trap, OWNED — each file's `TrapFile` is a local that drops at the end of its
    // loop iteration, so a borrow could not survive to the family check at the end of the walk. The
    // family-completeness check (story 4.7b) is answer-INDEPENDENT: it is about corpus SHAPE, so it
    // runs over every discovered trap regardless of the `answers` map.
    let mut all_traps: Vec<Trap> = Vec::new();
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
            if let Some(answer) = answers.get(&trap.id) {
                match answer {
                    Answer::Answered(outcome) => {
                        // The truth-table path is UNCHANGED from 4.6b (story 4.7a AC3: the rule
                        // assertion is layered on, not folded in). `record` uses the rule-blind
                        // `score()`, so a wrong-rule trap — whose verdict is right — records a PASS
                        // here and never enters `failures`.
                        tally.record(&trap.expect, outcome);
                        // The `(verdict, rule)` assertion, beside the tally. It fires WrongRule only
                        // on a verdict pass with a decision on both sides, so a trap is never in
                        // both buckets.
                        if let TrapVerdict::WrongRule { expected, actual } =
                            run_trap(&trap.expect, outcome)
                        {
                            rule_mismatches.push(RuleMismatch {
                                trap: trap.id.clone(),
                                column: Column::of(&trap.expect),
                                expected,
                                actual,
                            });
                        }
                    }
                    // 🔴 The producer RAN and could not ask. It touches NEITHER the tally nor
                    // `run_trap`: an unanswerable trap produces no `Score`, so it can never pass a
                    // column — least of all `must-abstain`, where recording it as an abstention
                    // would pass (story 5.8's `Answer` doc carries that measurement).
                    Answer::Unanswerable { cause } => {
                        unanswered.push(Unanswered {
                            trap: trap.id.clone(),
                            column: Column::of(&trap.expect),
                            cause: cause.clone(),
                        });
                    }
                }
                // BOTH arms: a declined trap is an answer the producer gave about a trap, so an
                // `Unanswerable` naming no discovered trap must be refused by the same check.
                used.insert(trap.id.clone());
            }
            all_traps.push(trap.clone());
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
        rule_mismatches,
        incomplete_families: incomplete_families(&all_traps),
        unanswered,
    })
}

/// Walk `root` recursively for `.toml` trap files, in sorted order.
///
/// It refuses a symlink and a foreign extension, and does NOT swallow a read error — *"walks that
/// quietly see less"* were the recurring defect of stories 4.1 and 4.3, so a subtree it cannot read
/// is an error, not a smaller result. `README.md` is exempt at any depth, exactly as the corpus
/// lock's orphan rule exempts it, so documenting a directory does not turn the gate red.
///
/// This is the harness's OWN walk, not one of the `#[cfg(test)]` walks in `fixtures.rs`. The trap
/// tree's test-side walk is `fixtures::walk_trap_files` (story 5.2 hoisted it out of
/// `every_trap_file_in_the_corpus_is_valid`, which had walked inline). The two stay SEPARATE
/// deliberately — this one is the scoring harness's discovery path and returns `Result` where the
/// test walk panics — but they are held to the same RULES, and a change to any of them belongs in
/// both: dot-entries skipped, `README.md` exempt at any depth, symlinks refused, foreign
/// extensions refused, sorted order. Promoting either into the other would move its callers for no
/// gain here.
///
/// **`pub(crate)` since story 5.7**, so [`crate::l1_runner`] walks the corpus through THIS function
/// instead of writing a third walk. The tree already carries two, and their divergence on three
/// points is a registered defect from story 5.2's review; a third would be the same mistake a third
/// time. The alternative — `fixtures::walk_trap_files` — is `#[cfg(test)]` and takes **no root**
/// (it hardcodes the committed corpus), so it neither exists in a production build nor could be
/// pointed at a scratch corpus.
pub(crate) fn discover_trap_files(root: &Path) -> Result<Vec<PathBuf>, FixtureError> {
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
            // Tooling scratch is not corpus. `fixtures/scenario/traps/.claude/.cc-writes` already
            // exists in a working tree (created 2026-07-26, empty — the only reason discovery
            // stayed green), and the foreign-extension refusal below would have accused the CORPUS
            // of a defect the moment any tool wrote a file under it. Measured before the skip
            // landed (story 5.2): one `probe.txt` there reds `every_trap_file_in_the_corpus_is_valid`
            // AND six tests in this module — including `an_answer_for_an_unknown_trap_is_refused`,
            // which expects an error and got `Io` instead of `AnswerForUnknownTrap`, because
            // discovery fails before `score_corpus` ever validates an answer.
            //
            // Story 5.1 closed this class on the REPLAY tree only; this is the same one line on
            // the trap tree, in both of its walks. The cost is named as 5.1 named it: a
            // `.hidden.toml` is no longer discovered — acceptable because the corpus never hides
            // an artefact and `MANIFEST.toml` lists every one by its visible name. `xtask`'s own
            // corpus walk has skipped dot-entries since 2026-07-21, so the lock and the orphan
            // check stay consistent with both walks in both directions.
            if entry
                .file_name()
                .to_str()
                .is_some_and(|n| n.starts_with('.'))
            {
                continue;
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
    use crate::l1_runner::{answer_trap, l1_answers};
    use opencmdb_core::identity::cascade::IdentityAbstentionCause;
    use opencmdb_core::score::Column;
    // `Outcome` left production use with story 5.8: `score_corpus` now matches on `Answer` and
    // never names the inner type. An import kept alive only by this module is an `unused_imports`
    // ERROR in `cargo clippy --workspace -- -D warnings`, the form CI runs, and invisible under
    // `--all-targets` — so it belongs here, as `l1_runner.rs` already does for the rule ids.
    use opencmdb_core::score::Outcome;
    use opencmdb_core::trap::RuleId;
    use std::collections::BTreeSet;

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

    /// An EMPTY answers map scores nothing — the vacuity `discovered` exists to make visible.
    ///
    /// _(Renamed in story 5.7. It read
    /// `the_committed_corpus_is_discovered_and_scored_by_nothing`, which stated a claim about the
    /// CORPUS that story 5.7 falsifies: `l1_runner` now answers thirteen of its traps. What this
    /// test pins is a property of its own CALL — it supplies no answers — and the name now says
    /// so.)_
    #[test]
    fn an_empty_answers_map_scores_nothing_over_the_committed_corpus() {
        let report = score_corpus(&committed_traps_root(), &BTreeMap::new())
            .expect("the committed corpus reads");

        // Discovered is what makes the zeros honest: `example.toml` carries three traps,
        // `randomized-mac.toml` (story 4.9) adds two, `multi-nic.toml` (story 4.10) two more,
        // `shared-hardware-vm.toml` (story 4.11) three, `cloned-mac.toml` (story 4.12) two,
        // `dhcp-churn.toml` (story 4.13) two, `vrrp-virtual-mac.toml` (story 4.14) three,
        // `hostname-collision.toml` (story 4.15) two, `docker-veth.toml` (story 4.16) two and
        // `hostname-absence.toml` (story 4.17) three — twenty-four in the committed corpus.
        assert_eq!(report.discovered(), 24, "the walk must open the corpus");
        assert_eq!(
            report.scored(),
            0,
            "this call supplies no answers, so nothing is scored — a producer now EXISTS \
             (`crate::l1_runner`) and the test below runs the corpus through it"
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
        assert!(rendered.contains("24 trap(s) discovered"), "{rendered}");
        assert!(rendered.contains("0 scored"), "{rendered}");
        assert!(rendered.contains("0 truth-table failure(s)"), "{rendered}");
    }

    /// A discovered-but-unscored trap is counted in `discovered`, never dropped, and never scored
    /// as a phantom pass (AC5). One committed trap gets an answer; the other two are still counted.
    #[test]
    fn a_trap_with_no_answer_is_discovered_but_not_scored() {
        let mut answers = BTreeMap::new();
        // A correct answer for one trap, so `scored` is 1 while `discovered` stays 24.
        answers.insert(
            TrapId("example-must-abstain".into()),
            Answer::Answered(Outcome::Abstained {
                cause: IdentityAbstentionCause::Ambiguous,
            }),
        );
        let report = score_corpus(&committed_traps_root(), &answers).unwrap();
        assert_eq!(report.discovered(), 24);
        assert_eq!(report.scored(), 1, "only the answered trap is scored");
        assert_eq!(report.failures(), 0, "and its answer is correct");
    }

    // ── The corpus scored by the REAL engine (story 5.7) ─────────────────────
    //
    // These tests live here and not in `l1_runner.rs` because their subject is the REPORT: what
    // the harness makes of the answers. WHICH traps the runner answers is the runner's own claim
    // and is pinned there.

    /// The committed corpus, answered by the real L1 engine.
    ///
    /// `scored` has read **0** since story 4.6b, over nine stories — the honest state while no
    /// producer existed. It reads 13 here, and every one of the thirteen passes **both halves of
    /// story 4.7a's assertion: the truth table AND the rule**.
    ///
    /// ⚠️ The `incomplete_families()` assertion below is NOT a property of those thirteen answers
    /// and is not claimed as one. [`incomplete_families`] is computed over ALL discovered traps
    /// (`score_corpus` passes it `all_traps`), so it says the corpus SHAPE is unchanged by this
    /// story and nothing whatever about the engine. It has to be independent: the runner answers 2
    /// of `hostname-absence`'s 3 traps and 1 of `vrrp-virtual-mac`'s 3, so a completeness check
    /// computed over the SCORED traps would be non-empty here.
    #[test]
    fn the_committed_corpus_is_scored_by_the_l1_engine() {
        let answers = l1_answers(&committed_traps_root()).expect("the runner answers the corpus");
        let report = score_corpus(&committed_traps_root(), &answers).expect("the corpus scores");

        assert_eq!(
            report.discovered(),
            24,
            "the unanswered traps stay in the denominator — the exclusion is visible, never silent"
        );
        assert_eq!(
            report.scored(),
            13,
            "the thirteen traps whose expected rule is `l1-*`"
        );
        assert_eq!(report.failures(), 0, "no truth-table failure");
        assert!(
            report.rule_mismatches().is_empty(),
            "and none of them reached the right verdict by the wrong rule: {:?}",
            report.rule_mismatches()
        );
        assert!(
            report.incomplete_families().is_empty(),
            "the corpus shape is unchanged by this story: {:?}",
            report.incomplete_families()
        );
        // 🔴 FLIPPED by story 5.8, and the flip is the deliverable rather than a regression. Until
        // then this asserted a PASS while eleven of the corpus's twenty-four traps had never been
        // put to any engine — D18's *"a gate that cannot fall is decoration"*. The eleven are now a
        // blocking bucket; `epics.md:416` says NFR4 stays RED and is closed by Epic 6.
        assert!(
            !report.passed(),
            "the gate must NOT pass while eleven traps were never asked — the three buckets above \
             are all empty, so the fourth is the only thing blocking it: {report}"
        );
    }

    // ── The fourth bucket (story 5.8) ─────────────────────────────────────────

    /// Every committed trap, by id — so a column total can be read from the CORPUS rather than
    /// hard-coded a second time beside the numbers it is meant to check.
    ///
    /// ⚠️ **A deliberate twin of `l1_runner`'s helper of the same name, and it may not be
    /// collapsed.** Rust gives no way to share a `#[cfg(test)] mod tests` item across files, which
    /// is the same obstacle `l1_runner`'s own `committed_traps_root()` records ("`trap_gate`'s
    /// lives inside that file's `mod tests` and is unreachable from here"). The house DRY rule
    /// permits redundancy a comment labels as deliberate; this is that label. A DRY pass that
    /// wants one copy must promote it to a shared test-support module, not delete one.
    fn committed_traps() -> BTreeMap<TrapId, Trap> {
        let mut all = BTreeMap::new();
        for file in discover_trap_files(&committed_traps_root()).expect("the corpus walks") {
            for trap in read_traps(&file).expect("a committed trap file reads").trap {
                all.insert(trap.id.clone(), trap);
            }
        }
        all
    }

    fn committed_report() -> Report {
        let answers = l1_answers(&committed_traps_root()).expect("the runner answers the corpus");
        score_corpus(&committed_traps_root(), &answers).expect("the corpus scores")
    }

    /// 🔴 The committed gate is RED, and the eleven are named one by one rather than counted.
    ///
    /// *A residue that can grow in silence is how a gate quietly stops testing* — a count alone
    /// would let a trap move from answered to bucketed without a red, as long as another moved the
    /// other way.
    #[test]
    fn the_committed_corpus_is_red_with_eleven_unanswerable_traps() {
        let report = committed_report();

        assert_eq!(report.unanswered().len(), 11, "{report}");
        assert!(
            !report.passed(),
            "eleven traps were never asked, so the gate does not pass"
        );

        let named: BTreeSet<&str> = report
            .unanswered()
            .iter()
            .map(|u| u.trap.0.as_str())
            .collect();
        assert_eq!(
            named,
            BTreeSet::from([
                "cloned-mac-must-not-merge",
                "docker-veth-must-merge",
                "example-must-abstain",
                "hostname-absence-must-abstain",
                "multi-nic-must-merge",
                "multi-nic-must-not-merge",
                "shared-hardware-vm-must-abstain",
                "shared-hardware-vm-must-merge",
                "shared-hardware-vm-must-not-merge",
                "vrrp-virtual-mac-must-not-merge-bearers",
                "vrrp-virtual-mac-must-not-merge-master",
            ]),
            "the bucket holds exactly these eleven — by NAME, so a swap cannot hide in the count"
        );

        // The three classes, on the REPORT's own record — which, unlike the runner's map, also
        // carries the COLUMN each trap was declined in.
        let mut level = 0;
        let mut no_level = 0;
        let mut no_pair = 0;
        for entry in report.unanswered() {
            match &entry.cause {
                UnanswerableCause::LevelNotImplemented { expected } => {
                    assert!(
                        expected.0.starts_with("l2-"),
                        "a trap is declined for the level ITS AUTHOR named: {} got `{}`",
                        entry.trap.0,
                        expected.0
                    );
                    level += 1;
                }
                UnanswerableCause::NoLevelToRouteOn => {
                    assert_eq!(
                        entry.column,
                        Column::MustAbstain,
                        "only a must-abstain expectation names no rule"
                    );
                    no_level += 1;
                }
                UnanswerableCause::NoPairUnderJudgement => no_pair += 1,
            }
        }
        assert_eq!((level, no_level, no_pair), (8, 2, 1), "the 8 / 2 / 1 split");
    }

    /// 🔴 **The strongest guard this story ships**: for every column, what was scored plus what was
    /// bucketed is what the corpus holds. A trap that vanished from BOTH sets is the only thing it
    /// can catch — and it is the failure `discovered` alone cannot see.
    ///
    /// ⚠️ The arithmetic loop runs BEFORE the three literals, deliberately. With the literals first,
    /// a mutation that breaks `unanswered_in` panics on the first of them and never reaches the
    /// loop — so the loop would be protected by nothing while the test still looked red.
    #[test]
    fn the_per_column_arithmetic_shows_nothing_left_the_denominator() {
        let report = committed_report();
        let traps = committed_traps();

        for column in [Column::MustMerge, Column::MustNotMerge, Column::MustAbstain] {
            let in_corpus = traps
                .values()
                .filter(|t| Column::of(&t.expect) == column)
                .count();
            assert_eq!(
                report.tally().scored_in(column) + report.unanswered_in(column),
                in_corpus,
                "column {}: scored + unanswerable must equal what the corpus holds — a trap \
                 missing from both sets leaves the denominator with nothing to say so",
                column.as_str()
            );
        }

        assert_eq!(
            report.scored() + report.unanswered().len(),
            report.discovered(),
            "and the same equality over the whole corpus: 13 + 11 == 24"
        );
    }

    /// The per-column SPLIT, in its own test.
    ///
    /// 🔴 Split out from the arithmetic above by story 5.8's code review, per story 5.6's own
    /// idiom (*"in one test a missing pair panics before any recall exists"*). One test cannot
    /// carry both: whichever assertion runs first makes the other unreachable under any mutation
    /// that reds it, and the story's original ordering merely MOVED the unreachability rather than
    /// removing it. Two tests, two independent reds.
    ///
    /// The totals are what the loop above guards; these are the DISTRIBUTION inside each total —
    /// the only thing that catches a trap moving from scored to bucketed within one column, which
    /// leaves every total unchanged.
    #[test]
    fn the_per_column_split_of_the_bucket_is_three_five_three() {
        let report = committed_report();
        assert_eq!(report.unanswered_in(Column::MustMerge), 3);
        assert_eq!(report.unanswered_in(Column::MustNotMerge), 5);
        assert_eq!(report.unanswered_in(Column::MustAbstain), 3);
    }

    /// `unaccounted()` reports the traps a producer said NOTHING about — and the arithmetic is
    /// measured on a map where all three terms are non-zero.
    ///
    /// 🔴 Added by story 5.8's code review: the only prior call site passed an EMPTY map, so both
    /// subtractions were `− 0` and replacing the whole body with `self.discovered` left the entire
    /// suite green. Mutation M8 proved the CALL was load-bearing, not the calculation.
    #[test]
    fn unaccounted_counts_what_no_producer_spoke_about() {
        let dir = scratch_dir("unaccounted-partial");
        write_scratch_traps(
            &dir,
            r#"
[[trap]]
id = "spoken-for"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "a trap the producer declined, so it lands in the bucket rather than in the tally."
expect = { must-not-merge = { rule = "l2-different-switch" } }

[[trap]]
id = "answered-one"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "a trap the producer answered, so it is scored and leaves the unaccounted count alone."
expect = { must-not-merge = { rule = "l1-distinct-mac" } }

[[trap]]
id = "silent-about"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "a trap no producer said anything about at all, which is the state this test measures."
expect = { must-not-merge = { rule = "l1-distinct-mac" } }
"#,
        );
        let mut answers = BTreeMap::new();
        answers.insert(
            TrapId("spoken-for".into()),
            Answer::Unanswerable {
                cause: UnanswerableCause::LevelNotImplemented {
                    expected: RuleId("l2-different-switch".into()),
                },
            },
        );
        answers.insert(
            TrapId("answered-one".into()),
            Answer::Answered(Outcome::Refused {
                rule: RuleId("l1-distinct-mac".into()),
            }),
        );

        let report = score_corpus(&dir, &answers).expect("the scratch corpus reads");
        // All three terms non-zero, so neither subtraction can be dropped without a red.
        assert_eq!(report.discovered(), 3);
        assert_eq!(report.scored(), 1);
        assert_eq!(report.unanswered().len(), 1);
        assert_eq!(
            report.unaccounted(),
            1,
            "3 discovered − 1 scored − 1 declined = 1 nobody spoke about: {report}"
        );
        assert!(
            !report.passed(),
            "and the DECLINED one blocks, while the unaccounted one does not — that asymmetry is \
             decision 4, and it is what keeps 4.6b's AC1 true"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// AC2's named case: an `Unanswerable` key for a trap that does not exist is refused exactly as
    /// an `Answered` one is.
    ///
    /// 🔴 Added by story 5.8's code review. `used.insert` sits outside the `match` so BOTH arms
    /// reach the check — but nothing pinned it, and moving that line into the `Answered` arm is the
    /// natural refactor, since the arms already differ.
    #[test]
    fn an_unanswerable_answer_for_an_unknown_trap_is_refused_too() {
        let mut answers = BTreeMap::new();
        answers.insert(
            TrapId("no-such-trap".into()),
            Answer::Unanswerable {
                cause: UnanswerableCause::NoPairUnderJudgement,
            },
        );
        let err = score_corpus(&committed_traps_root(), &answers)
            .expect_err("a declined answer for a trap that does not exist is still a mismatch");
        assert!(
            matches!(err, FixtureError::AnswerForUnknownTrap { ref trap, count: 1 } if trap == "no-such-trap"),
            "a producer that declines a trap the corpus does not carry is a mismatch, not a \
             silent no-op: {err:?}"
        );
    }

    /// The report says how many, why, and that NFR4 is NOT MET (AC6).
    #[test]
    fn the_report_names_the_eleven_and_says_nfr4_is_not_met() {
        let rendered = committed_report().to_string();
        assert!(rendered.contains("24 trap(s) discovered"), "{rendered}");
        assert!(rendered.contains("13 scored"), "{rendered}");
        assert!(rendered.contains(", 11 unanswerable trap(s)"), "{rendered}");
        assert!(
            rendered.contains(
                "unanswerable: trap `multi-nic-must-merge` (must-merge): its author named rule \
                 `l2-uplink-agrees`, at a cascade level this engine does not implement"
            ),
            "each declined trap says WHICH level and WHOSE rule: {rendered}"
        );
        assert!(
            rendered.contains(
                "unanswerable: trap `example-must-abstain` (must-abstain): it does not name a pair"
            ),
            "{rendered}"
        );
        // 🔴 The NFR4 verdict is VENTILATED by cause: one closer does not close all three classes.
        // Story 5.8's code review measured that `NoLevelToRouteOn` and `NoPairUnderJudgement` do
        // not depend on a level at all, so Epic 6 takes the bucket from 11 to 3 and a sentence
        // attributing the whole count to "this cascade level — closed by Epic 6" would go on
        // naming as its closer the epic that had just shipped.
        assert!(
            rendered.contains("NFR4 NOT MET at this epic")
                && rendered.contains("11 trap(s) went unanswered."),
            "{rendered}"
        );
        assert!(
            rendered.contains(
                "8 at a cascade level this engine does not implement (closed by Epic 6)."
            ),
            "the eight `l2-*` traps, and only those, are Epic 6's to close: {rendered}"
        );
        assert!(
            rendered.contains("3 with no level to route on at all (closed by story 5.14 / Epic 6"),
            "the three `must-abstain` traps survive every cascade level and say so: {rendered}"
        );
    }

    /// 🔴 The NFR4 sentence **deletes itself** when the bucket empties — it is not a claim someone
    /// has to remember to remove the day Epic 6 lands.
    #[test]
    fn an_empty_bucket_renders_neither_the_count_nor_the_nfr4_line() {
        let rendered = score_corpus(&committed_traps_root(), &BTreeMap::new())
            .unwrap()
            .to_string();
        // The SUFFIX and the LINE PREFIX, not the bare word: a trap id containing "unanswerable"
        // would have defeated the looser guard, so it would have passed for a reason unrelated to
        // the bucket (story 5.8's code review).
        assert!(
            !rendered.contains("unanswerable trap(s)"),
            "no bucket, no count suffix: {rendered}"
        );
        assert!(
            !rendered.contains("\n  unanswerable: trap `"),
            "and no per-trap line: {rendered}"
        );
        assert!(
            !rendered.contains("NFR4"),
            "and no NFR4 claim — the sentence is tied to the bucket, not written unconditionally: \
             {rendered}"
        );
    }

    /// 🔴 4.6b's AC1, in one line: an ABSENT entry is not a decline. A corpus no producer ran over
    /// is still GREEN — *"it must not require an engine to exist"* — and `unaccounted()` is what
    /// names that state without blocking on it.
    #[test]
    fn an_absent_answer_is_not_a_decline_and_does_not_block() {
        let report = score_corpus(&committed_traps_root(), &BTreeMap::new()).unwrap();
        assert_eq!(report.discovered(), 24);
        assert_eq!(report.scored(), 0);
        assert!(
            report.unanswered().is_empty(),
            "absence never fills the bucket — only an explicit `Answer::Unanswerable` does"
        );
        assert_eq!(
            report.unaccounted(),
            24,
            "and the state is REPORTED: neither scored nor declined"
        );
        assert!(
            report.passed(),
            "a real corpus with no producer at all stays green (4.6b AC1): {report}"
        );
    }

    /// One family's poles live in different epics, so a family does not move as a block — and its
    /// completeness check is not read as a failure of Epic 5 (`epics.md:1555`).
    struct FamilySplit {
        /// The trap ids the L1 engine answers.
        answered: &'static [&'static str],
        /// The trap ids the bucket holds, each with the `l2-*` rule its author named.
        bucketed: &'static [(&'static str, &'static str)],
    }

    #[test]
    fn a_mixed_family_splits_between_the_engine_and_the_bucket() {
        let report = committed_report();
        let bucket: BTreeMap<&str, &UnanswerableCause> = report
            .unanswered()
            .iter()
            .map(|u| (u.trap.0.as_str(), &u.cause))
            .collect();
        // 🔴 Read from the RUNNER's map, not derived as "corpus minus bucket". That derivation
        // folds the report's THIRD state — `unaccounted`, neither scored nor bucketed — into
        // "answered", so a trap the producer dropped entirely would satisfy `is the family's L1
        // pole and is answered`. Measured by story 5.8's code review.
        let answers = l1_answers(&committed_traps_root()).expect("the runner answers the corpus");
        let scored: BTreeSet<String> = answers
            .iter()
            .filter(|(_, a)| matches!(a, opencmdb_core::score::Answer::Answered(_)))
            .map(|(id, _)| id.0.clone())
            .collect();
        assert_eq!(
            report.unaccounted(),
            0,
            "the premise of the derivation below: every discovered trap is either answered or \
             bucketed, so `scored` cannot silently absorb a dropped one"
        );

        let splits = [
            // The three MIXED families: an `l1-*` pole and an `l2-*` pole each.
            FamilySplit {
                answered: &["cloned-mac-must-merge"],
                bucketed: &[("cloned-mac-must-not-merge", "l2-different-hostname")],
            },
            FamilySplit {
                answered: &["docker-veth-must-not-merge"],
                bucketed: &[("docker-veth-must-merge", "l2-uplink-agrees")],
            },
            FamilySplit {
                answered: &["vrrp-virtual-mac-must-merge"],
                bucketed: &[
                    (
                        "vrrp-virtual-mac-must-not-merge-bearers",
                        "l2-different-hostname",
                    ),
                    (
                        "vrrp-virtual-mac-must-not-merge-master",
                        "l2-virtual-mac-prefix",
                    ),
                ],
            },
            // The two PURE-L2 families: nothing is answered at all.
            FamilySplit {
                answered: &[],
                bucketed: &[
                    ("multi-nic-must-merge", "l2-uplink-agrees"),
                    ("multi-nic-must-not-merge", "l2-different-switch"),
                ],
            },
            FamilySplit {
                answered: &[],
                bucketed: &[
                    ("shared-hardware-vm-must-merge", "l2-hostname-agrees"),
                    ("shared-hardware-vm-must-not-merge", "l2-different-hostname"),
                ],
            },
        ];

        for split in &splits {
            for id in split.answered {
                assert!(
                    scored.contains(*id),
                    "{id} is the family's L1 pole and is answered"
                );
            }
            for (id, rule) in split.bucketed {
                assert_eq!(
                    bucket.get(id),
                    Some(&&UnanswerableCause::LevelNotImplemented {
                        expected: RuleId((*rule).to_string()),
                    }),
                    "{id} is bucketed for the level its author named"
                );
            }
        }

        // `shared-hardware-vm` also holds a `must-abstain`, so it is FULLY bucketed at 3 of 3 —
        // and `hostname-absence`, which `epics.md:1527` calls pure-L1, is 2 answered + 1 bucketed.
        assert!(matches!(
            bucket.get("shared-hardware-vm-must-abstain"),
            Some(UnanswerableCause::NoLevelToRouteOn)
        ));
        assert!(scored.contains("hostname-absence-must-merge"));
        assert!(scored.contains("hostname-absence-must-not-merge"));
        assert!(matches!(
            bucket.get("hostname-absence-must-abstain"),
            Some(UnanswerableCause::NoLevelToRouteOn)
        ));

        assert!(
            report.incomplete_families().is_empty(),
            "and NO family is incomplete: completeness is corpus SHAPE, computed over every \
             discovered trap and independent of any answer, so a bucketed pole is not a failure \
             of Epic 5: {:?}",
            report.incomplete_families()
        );
    }

    /// The rendered line stops saying *"0 scored"*.
    #[test]
    fn the_report_line_says_thirteen_scored() {
        let answers = l1_answers(&committed_traps_root()).unwrap();
        let rendered = score_corpus(&committed_traps_root(), &answers)
            .unwrap()
            .to_string();
        assert!(rendered.contains("24 trap(s) discovered"), "{rendered}");
        assert!(rendered.contains("13 scored"), "{rendered}");
        assert!(rendered.contains("0 truth-table failure(s)"), "{rendered}");
    }

    /// The per-column split, INCLUDING the column that is empty (AC4).
    ///
    /// `scored_in` exists so a reader can tell *"the column held"* from *"the column was empty"*.
    /// After this story `must-abstain` is measured by **nothing**: all three committed
    /// `must-abstain` traps are unanswerable at L1 — two name a pair but no rule to route on, the
    /// third names no pair at all. That zero is not a defect: since story 5.8 those same three are
    /// in the blocking bucket ([`Report::unanswered_in`] reports 3 for this column), and story 5.14
    /// / Epic 6 are what make the column non-empty.
    #[test]
    fn the_per_column_tally_names_the_empty_column() {
        let answers = l1_answers(&committed_traps_root()).unwrap();
        let report = score_corpus(&committed_traps_root(), &answers).unwrap();
        let tally = report.tally();

        assert_eq!(tally.scored_in(Column::MustMerge), 7);
        assert_eq!(tally.scored_in(Column::MustNotMerge), 6);
        assert_eq!(
            tally.scored_in(Column::MustAbstain),
            0,
            "the must-abstain column is measured by NOTHING after this story — the vacuity \
             `scored_in` was built to make visible, not a column that held"
        );
        for column in [Column::MustMerge, Column::MustNotMerge, Column::MustAbstain] {
            assert_eq!(
                tally.failures_in(column),
                0,
                "no failure in {}",
                column.as_str()
            );
        }
    }

    /// A right verdict by the WRONG rule fails separately — demonstrated by the REAL engine on the
    /// COMMITTED corpus (AC5).
    ///
    /// This is the measured counter-factual of the level selector, made live. The four traps below
    /// expect an `l2-*` rule and their `must-not-merge` verdict is one L1 happens to reach: the
    /// engine refuses the pair, correctly, and names `l1-distinct-mac` where the trap's author
    /// named a device-level rule. Story 4.7a's separation says that is **not** a truth-table
    /// failure — the verdict passed — and it must be visible anyway. `failures()` stays 0 while
    /// `passed()` is false.
    ///
    /// The four are reached through `l1_runner::answer_trap`, which is level-blind, rather than
    /// through the walk: a test that reimplemented the selector to bypass it would be proving
    /// something about itself.
    #[test]
    fn a_right_verdict_by_an_l2_rule_is_a_wrong_rule_failure_not_a_truth_table_one() {
        let wanted = [
            ("multi-nic-must-not-merge", "l2-different-switch"),
            ("shared-hardware-vm-must-not-merge", "l2-different-hostname"),
            (
                "vrrp-virtual-mac-must-not-merge-bearers",
                "l2-different-hostname",
            ),
            (
                "vrrp-virtual-mac-must-not-merge-master",
                "l2-virtual-mac-prefix",
            ),
        ];
        let mut answers = BTreeMap::new();
        for trap_file in discover_trap_files(&committed_traps_root()).unwrap() {
            for trap in read_traps(&trap_file).unwrap().trap {
                if wanted.iter().any(|(id, _)| *id == trap.id.0) {
                    let outcome = answer_trap(&trap)
                        .expect("its stream reads")
                        .expect("each of the four names a pair");
                    answers.insert(trap.id.clone(), Answer::Answered(outcome));
                }
            }
        }
        assert_eq!(answers.len(), 4, "all four traps were found and answered");

        let report = score_corpus(&committed_traps_root(), &answers).unwrap();
        assert_eq!(
            report.failures(),
            0,
            "the verdict is RIGHT in all four — the truth table passes them"
        );
        assert_eq!(report.rule_mismatches().len(), 4, "{report}");
        for (id, expected_rule) in wanted {
            let mismatch = report
                .rule_mismatches()
                .iter()
                .find(|m| m.trap.0 == id)
                .unwrap_or_else(|| panic!("{id} must be reported as a wrong-rule failure"));
            assert_eq!(mismatch.expected, RuleId(expected_rule.into()));
            assert_eq!(
                mismatch.actual,
                RuleId("l1-distinct-mac".into()),
                "the L1 engine names the rule it actually fired"
            );
            assert_eq!(mismatch.column, Column::MustNotMerge);
        }
        assert!(
            !report.passed(),
            "a wrong rule blocks the gate even with zero truth-table failures"
        );
    }

    /// Replaying the corpus twice yields an identical `Report` (D36).
    ///
    /// The `scored() == 13` assertion on the first run is what stops the equality comparing two
    /// vacuities — the shape `replaying_the_same_corpus_twice_yields_identical_verdicts`
    /// established. Both the structural equality and the rendered string are compared: a `Report`
    /// that compared equal while rendering differently would still be a reproducibility defect.
    #[test]
    fn replaying_the_corpus_twice_yields_an_identical_report() {
        let first = score_corpus(
            &committed_traps_root(),
            &l1_answers(&committed_traps_root()).unwrap(),
        )
        .unwrap();
        let second = score_corpus(
            &committed_traps_root(),
            &l1_answers(&committed_traps_root()).unwrap(),
        )
        .unwrap();

        assert_eq!(
            first.scored(),
            13,
            "two real runs, not two empty ones — otherwise the equality below is vacuous"
        );
        assert_eq!(first, second, "the same corpus, the same engine, twice");
        assert_eq!(first.to_string(), second.to_string());
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
            Answer::Answered(Outcome::Abstained {
                cause: IdentityAbstentionCause::Ambiguous,
            }),
        );
        // must-not-merge, answered with a merge → the false merge.
        answers.insert(
            TrapId("red-must-not-merge".into()),
            Answer::Answered(Outcome::Merged {
                rule: RuleId("l2-uplink-agrees".into()),
            }),
        );
        // must-abstain, answered with a merge → a guess on the ambiguous case.
        answers.insert(
            TrapId("red-must-abstain".into()),
            Answer::Answered(Outcome::Merged {
                rule: RuleId("l2-uplink-agrees".into()),
            }),
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

    /// `passed()` is the D18 gate — `failures == 0` — plus D46b's wrong-rule criterion, the
    /// corpus-completeness criterion, the unanswerable bucket (story 5.8), and a floor of
    /// `discovered > 0`. The floor is what an EMPTY corpus fails; a real corpus with **no producer
    /// at all** still PASSES, because 4.6b's AC1 defines the vacuous-over-a-real-corpus run as green
    /// (the `scored` number is what tells a human it was vacuous, not `passed()`). What does NOT
    /// pass is a producer that RAN and declined — an absent key and an
    /// [`opencmdb_core::score::Answer::Unanswerable`] are different states and only the second
    /// blocks. A run with a failure does not pass.
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
            Answer::Answered(Outcome::Abstained {
                cause: IdentityAbstentionCause::Ambiguous,
            }),
        );
        answers.insert(
            TrapId("mixed-wrong".into()),
            Answer::Answered(Outcome::Abstained {
                cause: IdentityAbstentionCause::Ambiguous,
            }),
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
            Answer::Answered(Outcome::Merged {
                rule: RuleId("l1-exact-mac".into()),
            }),
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

    // ── The (verdict, rule) assertion, at the harness (story 4.7a) ───────────

    /// A right verdict by the WRONG rule turns the gate red — separately from a truth-table failure.
    /// The offending trap is SECOND, behind one answered by the RIGHT rule, so a harness that
    /// stopped after the first would still be caught. `failures()` stays 0 (the verdict passed);
    /// the mismatch is carried on its own, naming both rules, and `passed()` is false (AC1, AC5).
    #[test]
    fn a_right_verdict_by_the_wrong_rule_reddens_the_gate_without_a_truth_table_failure() {
        let dir = scratch_dir("trap-gate-wrong-rule");
        write_scratch_traps(
            &dir,
            r#"
[[trap]]
id = "rule-correct"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "a must-merge trap answered by a merge via the expected rule, so it stays green."
expect = { must-merge = { rule = "l1-exact-mac" } }

[[trap]]
id = "rule-wrong"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "a must-merge trap answered by a merge via a DIFFERENT rule, the right answer wrong reason."
expect = { must-merge = { rule = "l1-exact-mac" } }
"#,
        );
        let mut answers = BTreeMap::new();
        // The right verdict via the RIGHT rule — no mismatch.
        answers.insert(
            TrapId("rule-correct".into()),
            Answer::Answered(Outcome::Merged {
                rule: RuleId("l1-exact-mac".into()),
            }),
        );
        // The right verdict via the WRONG rule — the mismatch.
        answers.insert(
            TrapId("rule-wrong".into()),
            Answer::Answered(Outcome::Merged {
                rule: RuleId("l2-uplink-agrees".into()),
            }),
        );
        let report = score_corpus(&dir, &answers).expect("the corpus reads");

        assert_eq!(report.scored(), 2, "both merges are scored");
        assert_eq!(
            report.failures(),
            0,
            "both verdicts are RIGHT — a wrong rule is not a truth-table failure"
        );
        assert_eq!(
            report.rule_mismatches().len(),
            1,
            "exactly the wrong-rule trap is a mismatch, not the correct one"
        );
        let mismatch = &report.rule_mismatches()[0];
        assert_eq!(mismatch.trap, TrapId("rule-wrong".into()));
        assert_eq!(mismatch.column, Column::MustMerge);
        assert_eq!(mismatch.expected, RuleId("l1-exact-mac".into()));
        assert_eq!(mismatch.actual, RuleId("l2-uplink-agrees".into()));
        assert!(
            !report.passed(),
            "a wrong rule blocks a release exactly as a wrong verdict does (AC5)"
        );
        // And the Display names it, additively — the 4.6b substrings survive.
        let rendered = report.to_string();
        assert!(rendered.contains("2 scored"), "{rendered}");
        // The first line SELF-SIGNALS the wrong-rule red: the count is appended right after the
        // truth-table count, so the line alone can never read as a pass. Pinning them adjacent
        // reds if the suffix is dropped (the review finding this test closes).
        assert!(
            rendered.contains("0 truth-table failure(s), 1 wrong-rule failure(s)"),
            "{rendered}"
        );
        assert!(
            rendered.contains("wrong rule: trap `rule-wrong` (must-merge): expected rule `l1-exact-mac`, got `l2-uplink-agrees`"),
            "{rendered}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    /// The same trap answered by the EXPECTED rule is green (AC2): the rule assertion tightens the
    /// gate, it does not reject a correct answer. This is the discriminating partner of the test
    /// above — same trap, right rule, no mismatch.
    #[test]
    fn a_right_verdict_by_the_right_rule_leaves_the_gate_green() {
        let dir = scratch_dir("trap-gate-right-rule");
        write_scratch_traps(
            &dir,
            r#"
[[trap]]
id = "rule-right"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "a must-merge trap answered by a merge via the very rule the author named, so it passes."
expect = { must-merge = { rule = "l1-exact-mac" } }
"#,
        );
        let mut answers = BTreeMap::new();
        answers.insert(
            TrapId("rule-right".into()),
            Answer::Answered(Outcome::Merged {
                rule: RuleId("l1-exact-mac".into()),
            }),
        );
        let report = score_corpus(&dir, &answers).expect("the corpus reads");
        assert_eq!(report.scored(), 1);
        assert_eq!(report.failures(), 0);
        assert!(
            report.rule_mismatches().is_empty(),
            "the expected rule fired, so there is nothing to report"
        );
        assert!(report.passed(), "right verdict, right rule — green (AC2)");
        std::fs::remove_dir_all(&dir).ok();
    }

    // ── The corpus is incomplete if a family is one-sided (story 4.7b) ───────

    /// A family present in only ONE decision form reddens the gate — separately from any truth-table
    /// or wrong-rule failure, and with NO answers at all (the check is corpus-shape, not scoring). The
    /// one-sided family sits BESIDE a complete family, so a harness that stopped at the first family
    /// would still be caught. `failures()` and `rule_mismatches()` stay empty; `incomplete_families()`
    /// has exactly the one-sided family; `passed()` is false; and `Display` names it, on the first line
    /// AND its own line (AC1, AC5, DR2).
    #[test]
    fn a_one_sided_family_reddens_the_gate_on_its_own() {
        let dir = scratch_dir("trap-gate-one-sided-family");
        write_scratch_traps(
            &dir,
            r#"
[[trap]]
id = "complete-merge"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "the merge side of a family tested BOTH ways, so this family is complete."
family = "complete-fam"
expect = { must-merge = { rule = "l1-exact-mac" } }

[[trap]]
id = "complete-not-merge"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "the not-merge side of the same family, so the complete family is never the one reported."
family = "complete-fam"
expect = { must-not-merge = { rule = "l1-distinct-mac" } }

[[trap]]
id = "lonely-merge"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "a family tested only as a merge, so the corpus was never shown it can refuse the family."
family = "one-sided-fam"
expect = { must-merge = { rule = "l1-exact-mac" } }
"#,
        );
        // No answers at all: the family check does not depend on scoring.
        let report = score_corpus(&dir, &BTreeMap::new()).expect("the corpus reads");

        assert_eq!(report.discovered(), 3);
        assert_eq!(
            report.failures(),
            0,
            "no answer scored, so no truth-table failure"
        );
        assert!(
            report.rule_mismatches().is_empty(),
            "a one-sided family is not a wrong-rule mismatch — distinct third condition"
        );
        assert_eq!(
            report.incomplete_families(),
            &[opencmdb_core::trap::IncompleteFamily {
                family: opencmdb_core::trap::FamilyId("one-sided-fam".into()),
                has_merge: true,
                has_not_merge: false,
            }],
            "exactly the one-sided family, the complete one absent"
        );
        assert!(
            !report.passed(),
            "a one-sided family blocks a release exactly as a wrong verdict does (AC5)"
        );
        // The first line SELF-SIGNALS the red (DR2), adjacent to the truth-table count, and the
        // family gets its own line naming the missing pole (AC1).
        let rendered = report.to_string();
        assert!(
            rendered.contains("0 truth-table failure(s), 1 incomplete-family"),
            "{rendered}"
        );
        assert!(
            rendered.contains(
                "incomplete family `one-sided-fam`: has must-merge, missing must-not-merge"
            ),
            "{rendered}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    /// The discriminating partner (AC2): a family present in BOTH forms leaves the gate green. The
    /// completeness check tightens the gate, it does not reject a corpus that is genuinely two-sided.
    #[test]
    fn a_two_sided_family_leaves_the_gate_green() {
        let dir = scratch_dir("trap-gate-two-sided-family");
        write_scratch_traps(
            &dir,
            r#"
[[trap]]
id = "pair-merge"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "the merge side of the randomized-mac family, committed alongside its negative form."
family = "randomized-mac"
expect = { must-merge = { rule = "l1-exact-mac" } }

[[trap]]
id = "pair-not-merge"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000001", "aaaaaaaa-0000-4000-8000-000000000003"]
reason = "the not-merge side of the randomized-mac family, so the family is tested both ways."
family = "randomized-mac"
expect = { must-not-merge = { rule = "l1-distinct-mac" } }
"#,
        );
        let report = score_corpus(&dir, &BTreeMap::new()).expect("the corpus reads");
        assert_eq!(report.discovered(), 2);
        assert!(
            report.incomplete_families().is_empty(),
            "both poles present, so the family is complete"
        );
        assert!(
            report.passed(),
            "two-sided family, no failures — green (AC2)"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// An abstain-only family has NEITHER decision pole (DR1), so the gate is red and `Display` renders
    /// the "has neither pole" line — pinning the exact DR1 rendering, which the struct-level core test
    /// does not exercise.
    #[test]
    fn an_abstain_only_family_renders_the_neither_pole_line() {
        let dir = scratch_dir("trap-gate-abstain-only-family");
        write_scratch_traps(
            &dir,
            r#"
[[trap]]
id = "lonely-abstain"
replay = "scenario/replay/minimal.jsonl"
observations = ["aaaaaaaa-0000-4000-8000-000000000002"]
reason = "an ambiguous case tagged with a family but carrying no decision pole at all."
family = "ambiguous-cases"
expect = { must-abstain = { cause = "NoObservedValue" } }
"#,
        );
        let report = score_corpus(&dir, &BTreeMap::new()).expect("the corpus reads");
        assert!(
            !report.passed(),
            "a family with no decision pole is incomplete (DR1)"
        );
        let rendered = report.to_string();
        assert!(
            rendered.contains(
                "incomplete family `ambiguous-cases`: has neither pole (needs must-merge and must-not-merge)"
            ),
            "{rendered}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    /// Story 4.13, D36: *"we require REPRODUCIBILITY, not STABILITY. Replay `(data, capability)`
    /// -> the same verdict, always."* The lexical trap D36 names: the verdict may legitimately
    /// change when CAPABILITY changes — that is 4.6c's snapshot comparability — but the same
    /// `(corpus, answers)` may not. No engine exists at v0.2, so the answers map stands in for it
    /// until Epic 5; what this proves is HARNESS determinism across two same-process runs, and it
    /// reds only on nondeterminism visible at that scale — a per-instance-seeded `HashMap` swap,
    /// an ambient-time read. It cannot red on a removal of sorted discovery alone (both walks
    /// would replay the same readdir order); that property is held by `discover_trap_files`' own
    /// sort, not here.
    #[test]
    fn replaying_the_same_corpus_twice_yields_identical_verdicts() {
        let mut answers = BTreeMap::new();
        answers.insert(
            TrapId("dhcp-churn-must-merge".into()),
            Answer::Answered(Outcome::Merged {
                rule: RuleId("l1-exact-mac".into()),
            }),
        );
        answers.insert(
            TrapId("dhcp-churn-must-not-merge".into()),
            Answer::Answered(Outcome::Refused {
                rule: RuleId("l1-distinct-mac".into()),
            }),
        );

        let first = score_corpus(&committed_traps_root(), &answers).expect("the corpus reads");
        let second = score_corpus(&committed_traps_root(), &answers).expect("the corpus reads");

        assert_eq!(
            first, second,
            "the same (corpus, answers) must replay to the same verdicts"
        );
        assert_eq!(
            first.to_string(),
            second.to_string(),
            "and render to the same string"
        );
        // Both new traps answered correctly, so the equality above compares real verdicts,
        // not two vacuities. `first == second` carries all three counts to `second`.
        assert_eq!(first.scored(), 2, "both dhcp-churn traps are scored");
        assert_eq!(first.failures(), 0);
        assert!(first.rule_mismatches().is_empty());
    }
}
