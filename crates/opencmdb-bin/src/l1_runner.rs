//! The L1 answer producer — where the identity engine finally meets the trap corpus (Story 5.7).
//!
//! Since story 4.6b the harness in [`crate::trap_gate`] has walked the committed corpus, read its
//! traps, validated them — and scored **zero**, because the `answers` map it takes has always been
//! empty. Story 5.5 built the producer ([`opencmdb_core::identity::l1::decide_pair`]) and story 5.6
//! the blocker; neither had a production caller. This module is that caller: it walks a trap
//! corpus, asks the real engine about the pair each trap names, and returns a
//! `BTreeMap<TrapId, Outcome>` the harness can score.
//!
//! # Why it is NOT in `trap_gate.rs`
//!
//! `trap_gate`'s module doc states a **structural** guarantee: *"It scores answers; it never runs a
//! producer"*. That is a FILE-level property today — nothing in that file can reach an engine —
//! and putting the producer beside `score_corpus` would weaken it to a per-function promise on the
//! very day it first has something to promise about. The seam between the two stays a
//! `BTreeMap<TrapId, Outcome>`, which is **data**: no trait, no callback, no engine parameter.
//! `score_corpus`'s signature and body are unchanged by this story.
//!
//! # The selector is the EXPECTED RULE's LEVEL, never the outcome
//!
//! [`l1_answers`] answers a trap when two conditions hold, and both are named predicates so a
//! mutation can hit either alone:
//!
//! 1. the trap's expectation names a rule whose id starts with `l1-` ([`expects_an_l1_rule`]);
//! 2. the trap names exactly two observations ([`named_pair`]).
//!
//! The obvious objection to (1) is that it looks like *"answer only the traps we already pass"* —
//! scoring theatre. Four things answer it:
//!
//! - **it selects by LEVEL, not by outcome.** `Expectation::rule()` is what the trap's AUTHOR said
//!   answers the case, frozen in Epic 4 **before any engine existed**, precisely so the metric
//!   could not be bent to the engine (D19: *"a metric written after the engine is bent to fit the
//!   engine"*). The predicate reads the id's prefix and nothing else — not the column, not the
//!   family, not the reason, and never the answer;
//! - **the unanswered traps do not leave the denominator.** `Report::discovered()` stays 24 while
//!   `scored()` is 13, and `Tally::scored_in` splits it per column. Story 5.8 turns the residue
//!   into a bucket that BLOCKS; the exclusion is visible here and blocking there, silent in
//!   neither;
//! - **a PREFIX, not a whitelist of the two implemented ids.** A trap expecting an `l1-*` rule this
//!   engine does not implement is answered anyway, and reds as a wrong-rule failure. A whitelist
//!   would let a future L1 rule slip out of the denominator in silence. The committed corpus writes
//!   exactly two `l1-*` ids today, so the two selectors agree on the committed bytes — only the
//!   prefix keeps agreeing tomorrow, and a scratch-corpus test pins that;
//! - **the counter-factual is measured, not argued.** Answering all 24 traps makes the gate red: 6
//!   truth-table failures and 4 wrong-rule failures. `answer_trap` is level-BLIND for exactly that
//!   reason — it is what lets a test ask the engine about an `l2-*` trap without reimplementing
//!   this module.
//!
//! # Two functions, one shared core
//!
//! [`answer_trap`] answers ONE trap whatever level its expectation names; [`l1_answers`] walks a
//! corpus and applies the level selector. They are not one function with a flag because
//! [`answer_trap`] has to be reachable on its own: with the selector buried inside the walk, the
//! wrong-rule demonstration would have to duplicate this module inside a test, and a test that
//! reimplements its subject proves nothing.
//!
//! What they share is [`answer_pair`] — **not** the pair-to-`Outcome` step, which is the one-liner
//! `outcome_of(&decide_pair(a, b))` and factors nothing. What is genuinely duplicated is resolving
//! an `ObsId` to an `&Observation` **plus the panic that must name the trap**.
//!
//! ⚠️ [`answer_trap`] does its own `read_jsonl`; [`l1_answers`] caches **one read per distinct
//! replay stream** and therefore cannot simply loop over it. That saving is the walk's own —
//! `read_traps` already reads each stream once per trap FILE for its `obs_id` cross-check, so the
//! corpus is touched more than once either way.
//!
//! # The stream root is the BAKED corpus root, always
//!
//! A trap's `replay` field is resolved through [`crate::fixtures::fixture_path`], against the
//! committed corpus — never against the `traps_root` handed to [`l1_answers`]. So
//! `l1_answers(scratch)` reads trap FILES from the scratch root and STREAMS from `fixtures/`. That
//! is the same limit `read_traps` carries (recorded at `trap_gate.rs`'s `score_corpus`), it is
//! stated here about the runner too, and it is load-bearing rather than incidental: it is what lets
//! a scratch trap file vary an expectation while judging real committed observations.
//!
//! Wired into no runtime path — the release gate is not `/healthz` — hence the `dead_code` allow,
//! for the same reason `fixtures.rs`, `arp_ping.rs` and `trap_gate.rs` carry it.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::Path;

use opencmdb_core::identity::l1::decide_pair;
use opencmdb_core::observation::{ObsId, Observation};
use opencmdb_core::score::{Outcome, outcome_of};
use opencmdb_core::trap::{Trap, TrapId};

use crate::fixtures::{FixtureError, fixture_path, read_jsonl, read_traps};
use crate::trap_gate::discover_trap_files;

/// The prefix that names the identity cascade's first level in a rule id.
///
/// A prefix and not a list of the two implemented ids — see the module doc. It is spelled once
/// here so the selector and its doc cannot drift apart.
const L1_PREFIX: &str = "l1-";

/// Whether this trap's EXPECTATION names a rule at the cascade's first level.
///
/// `None` — an abstaining expectation, which names a cause and no rule — is false: there is no
/// level to route on. The three committed `must-abstain` traps are excluded here, before the pair
/// condition is ever consulted.
fn expects_an_l1_rule(trap: &Trap) -> bool {
    trap.expect
        .rule()
        .is_some_and(|rule| rule.0.starts_with(L1_PREFIX))
}

/// The two observations a trap puts under judgement, or `None` when it does not name exactly two.
///
/// The identity question L1 answers is about a PAIR. A trap naming one observation, or three, is
/// not a pair and there is nothing for `decide_pair` to be asked about — see [`answer_trap`] for
/// why the tempting one-line alternative is refused.
fn named_pair(trap: &Trap) -> Option<(ObsId, ObsId)> {
    match trap.observations.as_slice() {
        [a, b] => Some((*a, *b)),
        _ => None,
    }
}

/// Resolve one of a trap's `obs_id`s inside the stream it names.
///
/// # Panics
///
/// If the id is absent from the stream. That is a **broken invariant, not an ordinary outcome**:
/// [`read_traps`] cross-checks every trap's `obs_id`s against its replay stream and refuses the
/// file with `DanglingObservation` otherwise, so a miss here means the trap never went through that
/// reader. The message names the trap, in `corpus_pairs()`'s idiom.
fn resolve<'a>(stream: &'a [Observation], trap: &Trap, id: ObsId) -> &'a Observation {
    stream.iter().find(|o| o.obs_id == id).unwrap_or_else(|| {
        panic!(
            "trap `{}` names observation {id}, absent from `{}` — read_traps' cross-check should \
             have refused the file first",
            trap.id.0, trap.replay
        )
    })
}

/// Ask the real engine about one pair, and record the answer as the release gate records answers.
///
/// The shared core of [`answer_trap`] and [`l1_answers`]: the `ObsId` → `&Observation` resolution
/// (twice) and the trap-naming panic behind it. The engine call itself is one line —
/// `outcome_of(&decide_pair(a, b))` — and factoring only that would share nothing.
fn answer_pair(stream: &[Observation], trap: &Trap, a: ObsId, b: ObsId) -> Outcome {
    let left = resolve(stream, trap, a);
    let right = resolve(stream, trap, b);
    outcome_of(&decide_pair(left, right))
}

/// Answer ONE trap with the real L1 engine, whatever level its expectation names.
///
/// **Level-blind on purpose.** It applies no `l1-` selector, which is what makes the wrong-rule
/// demonstration possible: a test can ask the engine about a trap whose expected rule is `l2-*`
/// and observe the answer land in `Report::rule_mismatches`, without duplicating [`l1_answers`].
///
/// `None` — no answer at all — when the trap does not name exactly two observations.
///
/// # Why a pairless trap gets NO answer, rather than the answer that would pass
///
/// The tempting implementation is one line: `decide(vec![], CURRENT_RULESET_VERSION)` returns
/// `Abstained { AbsenceOfProof }`, which maps to `Outcome::Abstained` and **PASSES** the
/// `must-abstain` column. Measured on the committed corpus: `example-must-abstain` names one
/// observation and would pass exactly that way.
///
/// **Refused.** The pass would come from calling the algebra with an empty verdict vector — the
/// engine evaluating *nothing* — not from L1 reasoning about the observation. A trap that passes
/// because no rule was asked is the *right answer for the wrong reason* D19 and D46b exist to
/// catch, and it would put a **1** in the `must-abstain` column of a gate that never asked the
/// question. So the condition is named, and it is asserted HERE rather than on [`l1_answers`]'
/// output: of the 13 committed traps whose expected rule is `l1-*`, **zero** name other than two
/// observations, so through the walk this guard is unreachable and any assertion on the walk's
/// output would be vacuous. Story 5.6's idiom: a guard the committed corpus cannot exercise through
/// the production path needs a test that reaches it directly.
///
/// # Errors
///
/// [`FixtureError`] if the trap's replay stream cannot be resolved or read.
pub fn answer_trap(trap: &Trap) -> Result<Option<Outcome>, FixtureError> {
    let Some((a, b)) = named_pair(trap) else {
        return Ok(None);
    };
    let stream = read_jsonl(&fixture_path(&trap.replay)?)?;
    Ok(Some(answer_pair(&stream, trap, a, b)))
}

/// Walk the trap corpus rooted at `traps_root` and answer every trap the L1 engine is asked about.
///
/// The result is exactly the `answers` map [`crate::trap_gate::score_corpus`] takes. A trap is in
/// it when BOTH selector conditions hold — the expectation names an `l1-*` rule, and the trap names
/// exactly two observations. Every other discovered trap is simply absent, which the harness counts
/// as *discovered and not scored*.
///
/// Discovery goes through [`discover_trap_files`], the harness's own walk, so the two see the same
/// tree. Streams are read once per distinct `replay` and reused; trap files are read from
/// `traps_root` while streams are always resolved against the baked corpus root (module doc).
///
/// # Errors
///
/// [`FixtureError`] if the walk, a trap file or a replay stream cannot be read or validated.
pub fn l1_answers(traps_root: &Path) -> Result<BTreeMap<TrapId, Outcome>, FixtureError> {
    let mut answers: BTreeMap<TrapId, Outcome> = BTreeMap::new();
    // One read per distinct replay stream. `contains_key` + `insert` rather than
    // `entry().or_insert_with(..)`: `?` is not allowed in that closure, and this is the shape
    // `read_traps` itself uses for the same reason.
    let mut streams: BTreeMap<String, Vec<Observation>> = BTreeMap::new();

    for trap_file in discover_trap_files(traps_root)? {
        let traps = read_traps(&trap_file)?;
        for trap in &traps.trap {
            // The LEVEL selector, first and alone.
            if !expects_an_l1_rule(trap) {
                continue;
            }
            // The pair condition, second and alone. On the committed corpus it excludes nothing —
            // the level selector already removed the only pairless trap — and that invisibility is
            // exactly why `answer_trap` carries the assertion.
            let Some((a, b)) = named_pair(trap) else {
                continue;
            };
            if !streams.contains_key(&trap.replay) {
                let stream = read_jsonl(&fixture_path(&trap.replay)?)?;
                streams.insert(trap.replay.clone(), stream);
            }
            let stream = &streams[&trap.replay];
            answers.insert(trap.id.clone(), answer_pair(stream, trap, a, b));
        }
    }
    Ok(answers)
}

/// Tests for the answer producer — the map it builds, and the residue it deliberately leaves out.
///
/// The REPORT these answers produce is `trap_gate.rs`'s subject and its tests live there:
/// `cascade.rs`'s convention is that *a test lives with the item whose CLAIM it pins*. What is
/// pinned here is the runner's own claim — WHICH traps it answers and which it does not.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::fixtures_dir;
    // The two implemented rule ids, read here so the corpus can be compared against them. Imported
    // inside `mod tests` because production code in this file needs only the prefix — an import
    // kept alive solely by a test module is an `unused_imports` error in the lib build CI compiles.
    use opencmdb_core::identity::l1::{L1_DISTINCT_MAC, L1_EXACT_MAC};
    use opencmdb_core::score::{Score, TrapVerdict, run_trap, score};
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn committed_traps_root() -> PathBuf {
        // Written locally rather than imported: `trap_gate::committed_traps_root` lives inside that
        // file's `mod tests` and is unreachable from here. A relative path is refused for the same
        // reason `fixtures.rs`'s `the_fixtures_path_is_expressed_once` refuses one — the corpus
        // root is expressed by `fixtures_dir()`, once.
        fixtures_dir().join("scenario/traps")
    }

    /// Every committed trap, by id. The expectations the assertions below score against.
    fn committed_traps() -> BTreeMap<TrapId, Trap> {
        let mut all = BTreeMap::new();
        for file in discover_trap_files(&committed_traps_root()).expect("the corpus walks") {
            for trap in read_traps(&file).expect("a committed trap file reads").trap {
                all.insert(trap.id.clone(), trap);
            }
        }
        all
    }

    fn ids(names: &[&str]) -> BTreeSet<TrapId> {
        names.iter().map(|n| TrapId((*n).to_string())).collect()
    }

    /// The thirteen traps whose expected rule is `l1-*` — seven `must-merge`, six `must-not-merge`.
    ///
    /// Written out as literals rather than derived from the corpus by the same predicate the
    /// runner uses: an expectation computed by calling the code under test proves nothing. This is
    /// the third independent statement of the corpus's L1 surface, beside `l1.rs`'s two constants
    /// and its test-side restatement, and a DRY pass may not collapse it into any of them.
    fn expected_answered() -> BTreeSet<TrapId> {
        ids(&[
            "cloned-mac-must-merge",
            "dhcp-churn-must-merge",
            "dhcp-churn-must-not-merge",
            "docker-veth-must-not-merge",
            "example-must-merge",
            "example-must-not-merge",
            "hostname-absence-must-merge",
            "hostname-absence-must-not-merge",
            "hostname-collision-must-merge",
            "hostname-collision-must-not-merge",
            "randomized-mac-must-merge",
            "randomized-mac-must-not-merge",
            "vrrp-virtual-mac-must-merge",
        ])
    }

    /// The eleven traps L1 cannot answer, in three classes: eight whose expected rule is `l2-*`,
    /// two `must-abstain` traps naming a pair, and one `must-abstain` trap naming a single
    /// observation.
    ///
    /// ⚠️ `epics.md:1545` hands story 5.8 the premise that there are **8** — the `l2-*` class alone.
    /// The three `must-abstain` traps are invisible to an `l2-*` selector because
    /// `Expectation::MustAbstain` carries a CAUSE and no rule, so `Expectation::rule()` returns
    /// `None` for them. The correction is registered in `deferred-work.md` with story 5.8 as owner.
    fn expected_unanswered() -> BTreeSet<TrapId> {
        ids(&[
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
        ])
    }

    // ── Which traps the runner answers (AC2, AC8) ─────────────────────────────

    #[test]
    fn the_committed_corpus_yields_thirteen_l1_answers() {
        let answers = l1_answers(&committed_traps_root()).expect("the committed corpus answers");
        let answered: BTreeSet<TrapId> = answers.keys().cloned().collect();
        assert_eq!(
            answered,
            expected_answered(),
            "the answered set is the thirteen traps whose expected rule is `l1-*`"
        );
        assert_eq!(answers.len(), 13);
    }

    /// The residue, by NAME rather than by count. *A residue that can grow in silence is how a gate
    /// quietly stops testing* — a count alone would let a trap move from answered to unanswered
    /// without a red, as long as another moved the other way.
    #[test]
    fn the_eleven_unanswered_traps_are_named_one_by_one() {
        let answers = l1_answers(&committed_traps_root()).unwrap();
        let all: BTreeSet<TrapId> = committed_traps().keys().cloned().collect();
        assert_eq!(all.len(), 24, "the corpus holds twenty-four traps");

        let unanswered: BTreeSet<TrapId> = all
            .difference(&answers.keys().cloned().collect())
            .cloned()
            .collect();
        assert_eq!(
            unanswered,
            expected_unanswered(),
            "the traps L1 leaves unanswered are exactly these eleven"
        );
        assert_eq!(
            unanswered.len(),
            11,
            "eleven, where epics.md:1545 says eight"
        );
    }

    /// AC2's named condition, asserted where it is REACHABLE.
    ///
    /// `example-must-abstain` names one observation, so there is no pair. Through [`l1_answers`]
    /// this guard is unreachable — the level selector excludes the trap first, since a
    /// `must-abstain` expectation names no rule — so an assertion on the walk's output would be
    /// vacuous. It is asserted directly on [`answer_trap`], which is level-blind and where dropping
    /// the guard panics on an out-of-bounds index.
    #[test]
    fn a_trap_that_names_no_pair_gets_no_answer() {
        let traps = committed_traps();
        let pairless = &traps[&TrapId("example-must-abstain".into())];
        assert_eq!(
            pairless.observations.len(),
            1,
            "the premise: this trap names ONE observation"
        );
        assert_eq!(
            answer_trap(pairless).expect("its stream reads"),
            None,
            "no pair, no answer — the engine is not asked a question it cannot form"
        );
    }

    /// What the refused one-liner WOULD have scored, measured rather than asserted in prose.
    ///
    /// `decide(vec![], _)` yields `Abstained { AbsenceOfProof }`, which maps to
    /// `Outcome::Abstained` and PASSES the `must-abstain` column. This test states that the pass is
    /// real and that it is refused anyway: it would come from the engine evaluating *nothing*, and
    /// it would put a 1 in a column the gate never actually asked about.
    #[test]
    fn the_pass_the_pairless_shortcut_would_have_manufactured_is_real_and_refused() {
        use opencmdb_core::identity::cascade::decide;
        use opencmdb_core::identity::l1::CURRENT_RULESET_VERSION;

        let traps = committed_traps();
        let pairless = &traps[&TrapId("example-must-abstain".into())];
        let manufactured = outcome_of(&decide(Vec::new(), CURRENT_RULESET_VERSION));
        assert_eq!(
            score(&pairless.expect, &manufactured),
            Score::Pass,
            "the shortcut really would pass — which is why it is refused, not because it fails"
        );
        assert_eq!(
            answer_trap(pairless).unwrap(),
            None,
            "and the runner still declines to answer"
        );
    }

    // ── The four families `epics.md` calls pure-L1 (AC4) ──────────────────────

    /// Both DECISION poles of the four pure-L1 families are answered, and each PASSES its trap —
    /// verdict and rule.
    ///
    /// ⚠️ The narrower claim, written down rather than assumed. `epics.md:1529` asks that *"their
    /// traps pass in BOTH poles"*; measured, `hostname-absence` holds a **third** trap — a
    /// `must-abstain` — which the L1 engine gets wrong and which the runner therefore does not
    /// answer. The sentence is true of the two decision poles and false of *"all their traps"*, so
    /// this test asserts the first reading and asserts the third trap's ABSENCE explicitly.
    #[test]
    fn the_four_pure_l1_families_pass_in_both_decision_poles() {
        let answers = l1_answers(&committed_traps_root()).unwrap();
        let traps = committed_traps();
        for family in [
            "randomized-mac",
            "dhcp-churn",
            "hostname-collision",
            "hostname-absence",
        ] {
            for pole in ["must-merge", "must-not-merge"] {
                let id = TrapId(format!("{family}-{pole}"));
                let outcome = answers
                    .get(&id)
                    .unwrap_or_else(|| panic!("{} is a pure-L1 trap and must be answered", id.0));
                assert_eq!(
                    run_trap(&traps[&id].expect, outcome),
                    TrapVerdict::Pass,
                    "{} must pass on both the verdict and the rule",
                    id.0
                );
            }
        }
        assert!(
            !answers.contains_key(&TrapId("hostname-absence-must-abstain".into())),
            "the family's THIRD trap is a must-abstain the L1 engine cannot answer — \
             `both poles` means both DECISION poles, not `all its traps`"
        );
    }

    // ── The producer's ids, against the corpus BYTES (AC7) ────────────────────
    //
    // This comparison lives here rather than in `fixtures.rs` because the claim it pins is the
    // RUNNER's: `l1.rs:94-96` says the two constants are spelled exactly as the corpus spells them
    // *"because story 5.7 compares this producer's id against those bytes"*, and it is this module
    // that would emit a rule id the gate then reports as a wrong-rule failure. The corpus files are
    // a dependency it reads. It is in `opencmdb-bin` because D47 forbids the domain crate to touch
    // the filesystem.

    /// Assert the rule-id claims over a trap corpus rooted at `root`.
    ///
    /// **Root-parameterised on purpose.** The alternative walk, `fixtures::walk_trap_files`, takes
    /// no root — it hardcodes the committed corpus — so a test built on it cannot be reddened by a
    /// rename in a scratch copy: it would keep reading the committed bytes and stay green. Through
    /// `discover_trap_files(root)` the mutation is a real mutation.
    fn assert_rule_ids_are_canonical(root: &Path) {
        let mut seen_exact = false;
        let mut seen_distinct = false;
        let mut checked = 0usize;

        for file in discover_trap_files(root).expect("the corpus walks") {
            for trap in read_traps(&file).expect("a trap file reads").trap {
                let Some(rule) = trap.expect.rule() else {
                    continue;
                };
                let id: &str = &rule.0;
                checked += 1;

                // Every id the corpus writes — `l2-*` included — must be its own trimmed,
                // lowercased self. `run_trap` compares raw `RuleId` strings with no
                // normalization, so a trailing space or a casing difference on either side would
                // be a false-positive wrong-rule failure: *"a red gate on a correct answer"*.
                assert_eq!(
                    id,
                    id.trim(),
                    "rule id `{id}` in {} is not trimmed",
                    file.display()
                );
                assert_eq!(
                    id,
                    id.to_lowercase(),
                    "rule id `{id}` in {} is not lowercase",
                    file.display()
                );

                if id.starts_with(L1_PREFIX) {
                    // The producer implements exactly two L1 rules. A THIRD `l1-*` id in the
                    // corpus is a trap this engine would answer with the wrong rule, and it must
                    // be a red here rather than a surprise in the gate.
                    assert!(
                        id == L1_EXACT_MAC || id == L1_DISTINCT_MAC,
                        "the corpus writes `{id}` in {}, which `identity::l1` does not implement",
                        file.display()
                    );
                    seen_exact |= id == L1_EXACT_MAC;
                    seen_distinct |= id == L1_DISTINCT_MAC;
                }
            }
        }

        assert!(
            checked > 0,
            "the walk found no rule id at all in {}",
            root.display()
        );
        // Both constants must OCCUR, or the assertion above passes by finding none — the vacuity
        // that would let either constant be renamed to anything at all and stay green.
        assert!(
            seen_exact,
            "no trap in {} names `{L1_EXACT_MAC}`",
            root.display()
        );
        assert!(
            seen_distinct,
            "no trap in {} names `{L1_DISTINCT_MAC}`",
            root.display()
        );
    }

    /// The producer's two rule ids are the corpus's own spelling, and every id it writes is
    /// canonical.
    ///
    /// `l1.rs` already restates the two ids as independent literals in its own test module
    /// (`CORPUS_EXACT_MAC` / `CORPUS_DISTINCT_MAC`), which catches a rename of one constant but
    /// cannot catch **both literals being wrong relative to the TOML**. This is the third
    /// independent statement, from the TOML side, and none of the three may be collapsed into the
    /// others by a DRY pass.
    #[test]
    fn the_producers_rule_ids_are_the_corpus_spelling() {
        assert_rule_ids_are_canonical(&committed_traps_root());
    }

    /// A private scratch directory per test — a shared constant path races between concurrent
    /// `cargo test` runs. The same reasoning `fixtures.rs` and `trap_gate.rs` both state.
    fn scratch_dir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("opencmdb-l1-runner-{}-{tag}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    /// The selector is a PREFIX, not a whitelist of the two implemented ids.
    ///
    /// A trap expecting an `l1-*` rule this engine does **not** implement is answered anyway, and
    /// its answer lands in `rule_mismatches` — visibly wrong. A whitelist would have dropped it out
    /// of the denominator in silence, which is exactly what story 5.8 exists to prevent.
    ///
    /// ⚠️ The scratch trap is **`must-not-merge`**, and the column is load-bearing: `run_trap`
    /// raises `WrongRule` only on a verdict PASS, and `minimal.jsonl` contains **no pair the L1
    /// engine merges** — its three observations carry MAC `…:53:01`, no MAC, and MAC `…:53:02`, so
    /// no two share a key. A `must-merge` scratch trap over that stream is a truth-table failure
    /// and could never reach `rule_mismatches` at all.
    ///
    /// The stream is a COMMITTED one: `read_traps` resolves `replay` against the baked corpus root,
    /// never against the scratch root (module doc).
    #[test]
    fn an_unimplemented_l1_rule_is_answered_and_reported_as_a_wrong_rule() {
        let dir = scratch_dir("unimplemented-l1-rule");
        let path = dir.join("scratch-traps.toml");
        std::fs::write(
            &path,
            r#"
[[trap]]
id = "scratch-unimplemented-l1"
replay = "scenario/replay/minimal.jsonl"
observations = [
  "aaaaaaaa-0000-4000-8000-000000000001",
  "aaaaaaaa-0000-4000-8000-000000000003",
]
reason = "a rule at L1 that this engine does not implement must still be asked, and answered wrongly."
expect = { must-not-merge = { rule = "l1-not-yet-implemented" } }
"#,
        )
        .expect("the scratch trap file writes");

        let answers = l1_answers(&dir).expect("the scratch corpus answers");
        assert_eq!(
            answers.len(),
            1,
            "the prefix selector answers it although the id is not one of the two implemented"
        );

        let report =
            crate::trap_gate::score_corpus(&dir, &answers).expect("the scratch corpus scores");
        assert_eq!(report.scored(), 1);
        assert_eq!(
            report.failures(),
            0,
            "the verdict is right: the pair is refused"
        );
        assert_eq!(report.rule_mismatches().len(), 1, "{report}");
        let mismatch = &report.rule_mismatches()[0];
        assert_eq!(mismatch.expected.0, "l1-not-yet-implemented");
        assert_eq!(mismatch.actual.0, L1_DISTINCT_MAC);
        assert!(
            !report.passed(),
            "and the gate is red, not silently smaller"
        );

        std::fs::remove_file(&path).ok();
    }
}
