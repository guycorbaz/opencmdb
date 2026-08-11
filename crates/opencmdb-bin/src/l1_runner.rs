//! The L1 answer producer — where the identity engine finally meets the trap corpus (Story 5.7).
//!
//! Since story 4.6b the harness in [`crate::trap_gate`] has walked the committed corpus, read its
//! traps, validated them — and scored **zero**, because the `answers` map it takes has always been
//! empty. Story 5.5 built the producer ([`opencmdb_core::identity::l1::decide_pair`]) and story 5.6
//! the blocker; neither had a production caller. This module is that caller: it walks a trap
//! corpus, asks the real engine about the pair each trap names, and returns the
//! `BTreeMap<TrapId, Answer>` the harness scores.
//!
//! Since **story 5.8** that map is TOTAL over the corpus: a trap the engine cannot be asked about
//! gets an [`Answer::Unanswerable`] carrying WHY, rather than being left out. See [`l1_answers`].
//!
//! # Why it is NOT in `trap_gate.rs`
//!
//! `trap_gate`'s module doc states a **structural** guarantee: *"It scores answers; it never runs a
//! producer"*. That is a FILE-level property today — nothing in that file can reach an engine —
//! and putting the producer beside `score_corpus` would weaken it to a per-function promise on the
//! very day it first has something to promise about. The seam between the two is a
//! `BTreeMap<TrapId, Answer>`, which is **data**: no trait, no callback, no engine parameter.
//! _(Story 5.8 widened that map's VALUE from `Outcome` to `Answer`, and with it `score_corpus`'s
//! signature — its arity, and the guarantee above, are unchanged. This paragraph said "the seam
//! STAYS a `BTreeMap<TrapId, Outcome>`… `score_corpus`'s signature and body are unchanged by this
//! story", which was true of story 5.7 and is false of 5.8.)_
//!
//! # The selector is the EXPECTED RULE's LEVEL, never the outcome
//!
//! [`l1_answers`] asks the engine about a trap when two conditions hold, and both are named
//! predicates so a mutation can hit either alone. **Story 5.8 made the ORDER significant** — it
//! decides which cause a trap that fails both is filed under — so they are numbered as they are
//! consulted:
//!
//! 1. the trap names exactly two distinct observations ([`named_pair`]);
//! 2. its expectation names a rule whose id starts with `l1-` ([`expects_an_l1_rule`]).
//!
//! _(Until story 5.8 this list gave the level first and the pair second. The answered set is the
//! same either way — a trap failing both is unanswerable regardless — but the CAUSE recorded for
//! `example-must-abstain` differs, and pair-first is the decision: *cannot be asked* outranks
//! *cannot be routed*.)_
//!
//! The obvious objection to the level condition is that it looks like *"answer only the traps we
//! already pass"* — scoring theatre. Four things answer it:
//!
//! - **it selects by LEVEL, not by outcome.** `Expectation::rule()` is what the trap's AUTHOR said
//!   answers the case, frozen in Epic 4 **before any engine existed**, precisely so the metric
//!   could not be bent to the engine (D19: *"a metric written after the engine is bent to fit the
//!   engine"*). The predicate reads the id's prefix and nothing else — not the column, not the
//!   family, not the reason, and never the answer;
//! - **the unanswered traps do not leave the denominator.** `Report::discovered()` stays 26 while
//!   `scored()` is 15, and `Tally::scored_in` splits it per column. Since story 5.8 the residue is
//!   a bucket that BLOCKS (`Report::unanswered`), so the committed gate does not pass at all while
//!   eleven traps go unasked; the exclusion is visible here and blocking there, silent in neither;
//! - **a PREFIX, not a whitelist of the two implemented ids.** A trap expecting an `l1-*` rule this
//!   engine does not implement is answered anyway, and reds as a wrong-rule failure. A whitelist
//!   would let a future L1 rule slip out of the denominator in silence. The committed corpus writes
//!   exactly two `l1-*` ids today, so the two selectors agree on the committed bytes — only the
//!   prefix keeps agreeing tomorrow, and a scratch-corpus test pins that;
//! - **the counter-factual is measured, not argued.** Answering every trap this runner CAN answer —
//!   25 of the 26, the twenty-sixth naming one observation and getting no answer at all — plus the
//!   one-line shortcut [`answer_trap`] refuses for that last one, makes the gate red: **6
//!   truth-table failures and 4 wrong-rule failures**. It decomposes as 4 + 4 over the eight `l2-*`
//!   traps and 2 more over the two paired `must-abstain` ones. ⚠️ Measured at contexting by a probe
//!   since deleted, and pinned by **no live test**: the two mutations that reproduce its halves are
//!   not in the tree. `answer_trap` is level-BLIND for exactly that
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
use std::path::{Path, PathBuf};

use opencmdb_core::identity::l1::decide_pair;
use opencmdb_core::observation::{ObsId, Observation};
use opencmdb_core::score::{Answer, Outcome, UnanswerableCause, outcome_of};
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
/// level to route on.
///
/// ⚠️ Since story 5.8 this is consulted **second**, after [`named_pair`], so it sees **two** of the
/// three committed `must-abstain` traps: `example-must-abstain` names one observation and the pair
/// condition removes it first. _(This doc said "the three committed `must-abstain` traps are
/// excluded here, before the pair condition is ever consulted" — true under story 5.7's order and
/// inverted by 5.8's.)_
fn expects_an_l1_rule(trap: &Trap) -> bool {
    trap.expect
        .rule()
        .is_some_and(|rule| rule.0.starts_with(L1_PREFIX))
}

/// The two DISTINCT observations a trap puts under judgement, or `None` when it names no such pair.
///
/// The identity question L1 answers is about a PAIR. A trap naming one observation, or three, is
/// not a pair and there is nothing for `decide_pair` to be asked about — see [`answer_trap`] for
/// why the tempting one-line alternative is refused.
///
/// ⚠️ **The self-pair is refused HERE, not merely absent from the corpus.** `[x, x]` is two ids and
/// one observation: `decide_pair(o, o)` intersects `keys_of(o)` with itself, so `l1-exact-mac` fires
/// and the trap MERGES — a pass no rule reasoned about, the same *right answer for the wrong reason*
/// [`answer_trap`] refuses for the pairless case. Story 5.6 closed the self-pair in the TYPE
/// ([`opencmdb_core::identity::blocking::CandidatePair::new`] is `None` on `(a, a)`); this is that
/// same refusal on the path that does not go through that type. It is not redundant with
/// `Trap::validate`'s `DuplicateObservation`: [`answer_trap`] reads its stream with `read_jsonl` and
/// **never calls `read_traps`**, so nothing validates a [`Trap`] a caller built by hand.
fn named_pair(trap: &Trap) -> Option<(ObsId, ObsId)> {
    match trap.observations.as_slice() {
        [a, b] if a != b => Some((*a, *b)),
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
/// `None` — no answer at all — when the trap does not name exactly two DISTINCT observations
/// ([`named_pair`]: one id, three ids, or the same id twice).
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
/// output: of the 15 committed traps whose expected rule is `l1-*`, **zero** name other than two
/// observations, so through the walk this guard is unreachable and any assertion on the walk's
/// output would be vacuous. Story 5.6's idiom: a guard the committed corpus cannot exercise through
/// the production path needs a test that reaches it directly.
///
/// # Errors
///
/// [`FixtureError`] if the trap's replay stream cannot be resolved or read.
///
/// # Panics
///
/// If an id the trap names is absent from the stream it names. That is a **broken invariant rather
/// than an input error** — [`read_traps`] cross-checks every `obs_id` against the replay stream and
/// refuses the file with `DanglingObservation` — but this function reads the stream with
/// [`read_jsonl`] and **never calls `read_traps`**, so on THIS path the precondition is the
/// caller's: pass a [`Trap`] that came from that reader. [`Trap`] is fully public with public
/// fields, so a hand-built one can reach the panic; [`resolve`]'s message names the trap. The
/// walk in [`l1_answers`] holds the precondition itself, because it reads every trap through
/// [`read_traps`].
pub fn answer_trap(trap: &Trap) -> Result<Option<Outcome>, FixtureError> {
    let Some((a, b)) = named_pair(trap) else {
        return Ok(None);
    };
    let stream = read_jsonl(&fixture_path(&trap.replay)?)?;
    Ok(Some(answer_pair(&stream, trap, a, b)))
}

/// Walk the trap corpus rooted at `traps_root` and say something about **every** trap it holds.
///
/// The result is exactly the `answers` map [`crate::trap_gate::score_corpus`] takes, and since
/// story 5.8 it is **TOTAL over the corpus**: one entry per discovered trap — 26 over the committed
/// one, **15 [`Answer::Answered`] and 11 [`Answer::Unanswerable`]**. Totality is the point. While a
/// declined trap was simply ABSENT from this map, a producer could drop a trap out of the
/// denominator with no reason attached and the gate stayed green; `epics.md`'s story 5.8 forbids
/// that — the unanswerable traps *"never silently leave the denominator"*. Absence still MEANS
/// something ([`crate::trap_gate::Report::unaccounted`], 4.6b's vacuous state) — this function just
/// never produces it.
///
/// # The classification is PAIR-FIRST, and the order decides one trap's cause
///
/// `example-must-abstain` is in two classes at once: it names a cause and no rule, AND it names one
/// observation. Whichever condition is consulted first wins, so the order is a decision, not a
/// style:
///
/// 1. **no pair** → [`UnanswerableCause::NoPairUnderJudgement`] — *cannot be asked*;
/// 2. a pair, but the expectation names no `l1-*` rule → [`UnanswerableCause::NoLevelToRouteOn`]
///    when it names no rule at all, else [`UnanswerableCause::LevelNotImplemented`] — *cannot be
///    routed*;
/// 3. otherwise the engine is asked.
///
/// Pair-first gives **8 / 2 / 1**; level-first would give 8 / 3 / 0. Pair-first is taken because a
/// trap with no pair is unanswerable at every level, present and future, so filing it under a level
/// it does not have would be wrong the day Epic 6 implements `l2-*` — and because 8 / 2 / 1 is what
/// story 5.7 measured and registered.
///
/// ⚠️ **The ANSWERED SET is invariant under that order.** A trap failing both conditions is
/// unanswerable either way, so the reorder moves a CAUSE and never a key. ⚠️ **The keys are no
/// longer the thirteen story 5.7 produced** — story 5.13b added two, so what survives 5.8's reorder
/// is the RELATION (a cause moves, a key does not) and not the list. A test asserts the set. Do not read the reorder as a behaviour
/// change.
///
/// Discovery goes through [`discover_trap_files`], the harness's own walk, so the two see the same
/// tree. Streams are read once per distinct `replay` and reused; trap files are read from
/// `traps_root` while streams are always resolved against the baked corpus root (module doc).
///
/// # Errors
///
/// [`FixtureError`] if the walk, a trap file or a replay stream cannot be read or validated, or
/// [`FixtureError::DuplicateTrapId`] if one `TrapId` appears in two files. `TrapFile::validate`
/// enforces uniqueness only WITHIN a file, and a duplicate here would silently overwrite an entry
/// and shorten a map whose LENGTH is now load-bearing — the residue arithmetic reads it. The
/// harness raises the same error for the same corpus, but a caller of this function alone would
/// otherwise get a short count with no diagnostic.
///
/// Ids are compared **folded** (`trim().to_lowercase()`), the same normalization
/// `TrapFile::validate` uses within a file — so `"Shared-Id"` and `"shared-id"` in two files are
/// one duplicate here as they would be in one file, rather than two traps a report cannot tell
/// apart.
pub fn l1_answers(traps_root: &Path) -> Result<BTreeMap<TrapId, Answer>, FixtureError> {
    let mut answers: BTreeMap<TrapId, Answer> = BTreeMap::new();
    // One read per distinct replay stream. `contains_key` + `insert` rather than
    // `entry().or_insert_with(..)`: `?` is not allowed in that closure, and this is the shape
    // `read_traps` itself uses for the same reason.
    let mut streams: BTreeMap<String, Vec<Observation>> = BTreeMap::new();
    // Every trap id seen so far, and the file it came from — the same guard `score_corpus` carries,
    // stated here because this map's LENGTH is read, not only its contents.
    //
    // Keyed on the id FOLDED the way `TrapFile::validate` folds it (`trim().to_lowercase()`), not
    // on the raw string. Story 5.8's code review measured the asymmetry: `"Shared-Id"` in one file
    // and `"shared-id"` in another passed the raw guard, inflating `discovered` — the denominator
    // this story made load-bearing — and rendering two bucket lines a reader cannot tell apart,
    // which is verbatim the harm `TrapError::DuplicateId` exists to prevent.
    let mut seen: BTreeMap<String, PathBuf> = BTreeMap::new();

    for trap_file in discover_trap_files(traps_root)? {
        let traps = read_traps(&trap_file)?;
        for trap in &traps.trap {
            let folded = trap.id.0.trim().to_lowercase();
            if let Some(first) = seen.insert(folded, trap_file.clone()) {
                return Err(FixtureError::DuplicateTrapId {
                    trap: trap.id.0.clone(),
                    first,
                    second: trap_file.clone(),
                });
            }
            let answer = match named_pair(trap) {
                // (1) The pair condition, FIRST and alone — see the doc above.
                None => Answer::Unanswerable {
                    cause: UnanswerableCause::NoPairUnderJudgement,
                },
                // (2) The LEVEL selector, second and alone. Its false arm re-consults the
                // expectation to tell "names no rule" from "names a rule at another level": two
                // predicates cannot yield three causes, and a third predicate would either
                // duplicate the `l1-` test or leave `expects_an_l1_rule` dead.
                Some(_) if !expects_an_l1_rule(trap) => Answer::Unanswerable {
                    cause: match trap.expect.rule() {
                        None => UnanswerableCause::NoLevelToRouteOn,
                        Some(expected) => UnanswerableCause::LevelNotImplemented {
                            expected: expected.clone(),
                        },
                    },
                },
                // (3) Both conditions hold: ask the real engine.
                Some((a, b)) => {
                    if !streams.contains_key(&trap.replay) {
                        let stream = read_jsonl(&fixture_path(&trap.replay)?)?;
                        streams.insert(trap.replay.clone(), stream);
                    }
                    let stream = &streams[&trap.replay];
                    Answer::Answered(answer_pair(stream, trap, a, b))
                }
            };
            answers.insert(trap.id.clone(), answer);
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
    use opencmdb_core::trap::{Expectation, RuleId};
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

    /// The fifteen traps whose expected rule is `l1-*` — eight `must-merge`, seven `must-not-merge`.
    ///
    /// Written out as literals rather than derived from the corpus by the same predicate the
    /// runner uses: an expectation computed by calling the code under test proves nothing. That is
    /// the whole reason, and it is enough — a DRY pass may not replace this list with a call to
    /// [`expects_an_l1_rule`].
    ///
    /// ⚠️ This is **not** the *"third independent statement"* `l1.rs:326-329` points at. That one is
    /// [`the_producers_rule_ids_are_the_corpus_spelling`] below, and it restates two RULE ids
    /// against the TOML. These are fifteen TRAP ids: no DRY pass could collapse them into a rule-id
    /// constant, so borrowing that argument here would protect nothing.
    fn expected_answered() -> BTreeSet<TrapId> {
        ids(&[
            "blinded-source-must-merge",
            "blinded-source-must-not-merge",
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
    /// ⚠️ `epics.md:1545` gave story 5.8 the premise that there are **8** — the `l2-*` class alone.
    /// The three `must-abstain` traps are invisible to an `l2-*` selector because
    /// `Expectation::MustAbstain` carries a CAUSE and no rule, so `Expectation::rule()` returns
    /// `None` for them. **Story 5.8 corrected `epics.md` itself**; the register entry that carried
    /// the correction forward is closed, and the eleven are now a blocking bucket rather than a
    /// residue named only here.
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

    /// The traps the map says were ANSWERED — the `Answer::Answered` half, not the whole map.
    ///
    /// Since story 5.8 the map is TOTAL, so `answers.keys()` is all 26 ids. Deriving the answered
    /// set from the keys would make this test — and the residue test below — assert nothing.
    fn answered_ids(answers: &BTreeMap<TrapId, Answer>) -> BTreeSet<TrapId> {
        answers
            .iter()
            .filter(|(_, a)| matches!(a, Answer::Answered(_)))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// The traps the map DECLINED, with the cause each was declined for.
    fn unanswered_causes(
        answers: &BTreeMap<TrapId, Answer>,
    ) -> BTreeMap<TrapId, UnanswerableCause> {
        answers
            .iter()
            .filter_map(|(id, a)| match a {
                Answer::Unanswerable { cause } => Some((id.clone(), cause.clone())),
                Answer::Answered(_) => None,
            })
            .collect()
    }

    /// The map is TOTAL over the corpus, and the ANSWERED ids are unchanged by story 5.8's
    /// reordering — the pair-first classification moves a cause, never a key.
    #[test]
    fn the_committed_corpus_yields_fifteen_l1_answers_and_an_entry_for_every_trap() {
        let answers = l1_answers(&committed_traps_root()).expect("the committed corpus answers");
        assert_eq!(
            answers.len(),
            26,
            "TOTAL over the corpus: every discovered trap has an entry, so none can leave the \
             denominator without a reason attached"
        );
        assert_eq!(
            answered_ids(&answers),
            expected_answered(),
            "the answered set is the fifteen traps whose expected rule is `l1-*`"
        );
        // A statement about the MAP, not about `expected_answered()`'s length — the set equality
        // above already pins the membership, so counting the literal would assert nothing.
        assert_eq!(
            answers
                .values()
                .filter(|a| matches!(a, Answer::Answered(_)))
                .count(),
            15,
            "fifteen of the twenty-six entries are answers"
        );
    }

    /// The residue, by NAME rather than by count. *A residue that can grow in silence is how a gate
    /// quietly stops testing* — a count alone would let a trap move from answered to unanswered
    /// without a red, as long as another moved the other way.
    #[test]
    fn the_eleven_unanswered_traps_are_named_one_by_one() {
        let answers = l1_answers(&committed_traps_root()).unwrap();
        let all: BTreeSet<TrapId> = committed_traps().keys().cloned().collect();
        assert_eq!(all.len(), 26, "the corpus holds twenty-six traps");

        // 🔴 Derived by FILTERING the map, never by `all.difference(answers.keys())`. The map is
        // total since story 5.8, so a key difference is empty and that derivation would assert
        // nothing while still compiling — the shape this test exists to prevent.
        let unanswered: BTreeSet<TrapId> = unanswered_causes(&answers).into_keys().collect();
        assert_eq!(
            unanswered,
            expected_unanswered(),
            "the traps L1 leaves unanswered are exactly these eleven"
        );
        assert_eq!(
            unanswered.len(),
            11,
            "eleven, where epics.md:1545 said eight until story 5.8 corrected it"
        );
        // Both halves read from the MAP, never from the literal oracle. With
        // `expected_answered().len()` on the left this was `11 + 13 == 24` — a restatement of the
        // two assertions above that stayed green under M5b, the very mutation its message claims to
        // catch.
        assert_eq!(
            unanswered.len() + answered_ids(&answers).len(),
            all.len(),
            "answered and unanswered PARTITION the corpus — neither set may lose a trap silently"
        );
    }

    /// The three causes, by class: **8 / 2 / 1**. The split story 5.7 measured and registered, and
    /// the one story 5.8's pair-first ordering decides — level-first would give 8 / 3 / 0 with the
    /// same eleven ids, so a test on the ids alone cannot see the ordering at all.
    #[test]
    fn the_residue_decomposes_into_eight_two_and_one() {
        let answers = l1_answers(&committed_traps_root()).unwrap();
        let causes = unanswered_causes(&answers);

        // Exhaustive, with NO `_` arm: `UnanswerableCause`'s own doc promises that a fourth variant
        // must break the build "wherever it is matched", and a wildcard here would have let a new
        // class compile straight into the "not LevelNotImplemented" bucket while the two counts
        // below silently excluded it.
        let level_not_implemented: BTreeMap<&TrapId, &RuleId> = causes
            .iter()
            .filter_map(|(id, c)| match c {
                UnanswerableCause::LevelNotImplemented { expected } => Some((id, expected)),
                UnanswerableCause::NoLevelToRouteOn | UnanswerableCause::NoPairUnderJudgement => {
                    None
                }
            })
            .collect();
        let no_level = causes
            .values()
            .filter(|c| **c == UnanswerableCause::NoLevelToRouteOn)
            .count();
        let no_pair = causes
            .values()
            .filter(|c| **c == UnanswerableCause::NoPairUnderJudgement)
            .count();

        assert_eq!(
            level_not_implemented.len(),
            8,
            "eight traps name a rule at an unimplemented level"
        );
        assert_eq!(
            no_level, 2,
            "two `must-abstain` traps name a pair but no rule to route on"
        );
        assert_eq!(
            no_pair, 1,
            "one trap names no pair at all — and PAIR-FIRST is what files it here rather than \
             under `NoLevelToRouteOn`, which is where level-first would put it"
        );

        // Each `LevelNotImplemented` carries the rule its AUTHOR named, never one the engine chose.
        let by_name: BTreeMap<&str, &str> = level_not_implemented
            .iter()
            .map(|(id, rule)| (id.0.as_str(), rule.0.as_str()))
            .collect();
        assert_eq!(
            by_name,
            BTreeMap::from([
                ("cloned-mac-must-not-merge", "l2-different-hostname"),
                ("docker-veth-must-merge", "l2-uplink-agrees"),
                ("multi-nic-must-merge", "l2-uplink-agrees"),
                ("multi-nic-must-not-merge", "l2-different-switch"),
                ("shared-hardware-vm-must-merge", "l2-hostname-agrees"),
                ("shared-hardware-vm-must-not-merge", "l2-different-hostname"),
                (
                    "vrrp-virtual-mac-must-not-merge-bearers",
                    "l2-different-hostname"
                ),
                (
                    "vrrp-virtual-mac-must-not-merge-master",
                    "l2-virtual-mac-prefix"
                ),
            ]),
            "the level a trap was declined for is the one its author wrote in the corpus"
        );

        // And the pairless one is named, so a corpus change that adds a second reds here rather
        // than shifting a count nobody reads.
        assert_eq!(
            causes.get(&TrapId("example-must-abstain".into())),
            Some(&UnanswerableCause::NoPairUnderJudgement)
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

    // ── The two guards the COMMITTED corpus cannot exercise (code review, 2026-08-02) ──────────
    //
    // `named_pair` refuses three things: one id, three ids, and the same id twice. The committed
    // corpus reaches only the first — every trap names one or two, and `Trap::validate` rejects a
    // duplicated id outright, so no trap FILE can express the other two. Both were measured
    // invisible before these tests existed: relaxing the arm to `[a, b, ..]` left all 350 tests
    // green. Story 5.6's M2 idiom — a guard the production path cannot reach needs a test that
    // reaches it directly — which is why these go through `answer_trap` and not the walk.

    /// A [`Trap`] over `minimal.jsonl` with the observation list a test chooses.
    ///
    /// Built by CLONING a committed trap so the replay path stays the corpus's own rather than a
    /// literal that could drift, then overriding what the test is about. It is not read through
    /// [`read_traps`] and could not be: the two lists below are exactly what a trap FILE cannot
    /// express.
    fn over_minimal(observations: Vec<ObsId>) -> Trap {
        let mut trap = committed_traps()[&TrapId("example-must-abstain".into())].clone();
        assert_eq!(
            trap.replay, "scenario/replay/minimal.jsonl",
            "the premise: the ids below are `minimal.jsonl`'s"
        );
        trap.id = TrapId("hand-built-over-minimal".into());
        trap.observations = observations;
        trap.expect = Expectation::MustMerge {
            rule: RuleId(L1_EXACT_MAC.into()),
        };
        trap
    }

    fn obs(literal: &str) -> ObsId {
        ObsId::from_uuid(uuid::Uuid::parse_str(literal).expect("an obs_id literal"))
    }

    /// Three observations are not a pair, and the third is not silently ignored.
    ///
    /// The guard's upper half. `Trap::validate` refuses an EMPTY observation list and a DUPLICATED
    /// one, but says nothing about a third id [`trap.rs:311-317`], so an `l1-*` trap naming three
    /// would validate — and under a `[a, b, ..]` arm it would be answered on its first two ids with
    /// the third dropped in silence. That is the same disappearance from the denominator the module
    /// doc argues a whitelist selector would cause, reached through a different door.
    #[test]
    fn a_trap_that_names_three_observations_gets_no_answer() {
        let trap = over_minimal(vec![
            obs("aaaaaaaa-0000-4000-8000-000000000001"),
            obs("aaaaaaaa-0000-4000-8000-000000000002"),
            obs("aaaaaaaa-0000-4000-8000-000000000003"),
        ]);
        assert_eq!(
            answer_trap(&trap).expect("its stream reads"),
            None,
            "three ids are not a pair — and the runner does not answer on the first two"
        );
    }

    /// The same observation twice is not a pair, and the merge it would have manufactured is real.
    ///
    /// Story 5.6 closed the self-pair in the TYPE (`CandidatePair::new(a, a)` is `None`). That type
    /// is not on this path: [`answer_trap`] never calls [`read_traps`], so `Trap::validate`'s
    /// `DuplicateObservation` does not hold the precondition for a [`Trap`] built by hand. The
    /// second assertion is what makes the first load-bearing rather than tidy — it measures the pass
    /// the missing guard WOULD have produced, in the idiom
    /// `the_pass_the_pairless_shortcut_would_have_manufactured_is_real_and_refused` established.
    #[test]
    fn a_trap_that_names_the_same_observation_twice_gets_no_answer() {
        let trap = over_minimal(vec![
            obs("aaaaaaaa-0000-4000-8000-000000000001"),
            obs("aaaaaaaa-0000-4000-8000-000000000001"),
        ]);
        assert_eq!(
            answer_trap(&trap).expect("its stream reads"),
            None,
            "two ids, one observation — the engine is not asked whether a thing is itself"
        );

        let stream = read_jsonl(&fixture_path(&trap.replay).unwrap()).unwrap();
        let one = resolve(&stream, &trap, obs("aaaaaaaa-0000-4000-8000-000000000001"));
        assert_eq!(
            outcome_of(&decide_pair(one, one)),
            Outcome::Merged {
                rule: RuleId(L1_EXACT_MAC.into())
            },
            "an observation shares every key with itself, so without the guard `l1-exact-mac` \
             fires and the trap MERGES — a pass no rule reasoned about"
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
                let Some(Answer::Answered(outcome)) = answers.get(&id) else {
                    panic!("{} is a pure-L1 trap and must be ANSWERED", id.0);
                };
                assert_eq!(
                    run_trap(&traps[&id].expect, outcome),
                    TrapVerdict::Pass,
                    "{} must pass on both the verdict and the rule",
                    id.0
                );
            }
        }
        // 🔴 `contains_key` no longer says anything: the map is TOTAL since story 5.8, so every
        // trap has an entry and the old `!contains_key` guard would have been silently false while
        // still compiling. What must be asserted is the VARIANT.
        assert!(
            matches!(
                answers.get(&TrapId("hostname-absence-must-abstain".into())),
                Some(Answer::Unanswerable { .. })
            ),
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
    ///
    /// # Returns
    ///
    /// `(rule ids checked, distinct rule ids seen)` — so a CALLER can pin the walk's completeness
    /// over a corpus it knows. Those two numbers are deliberately **not** asserted here: this helper
    /// runs over scratch roots too, and a count assertion inside it would red mutation M5c (a
    /// scratch corpus of `multi-nic.toml` alone) on the count instead of on the both-occur guard,
    /// masking the guard M5c was measured to prove load-bearing.
    fn assert_rule_ids_are_canonical(root: &Path) -> (usize, usize) {
        let mut seen_exact = false;
        let mut seen_distinct = false;
        let mut checked = 0usize;
        let mut distinct: BTreeSet<String> = BTreeSet::new();

        for file in discover_trap_files(root).expect("the corpus walks") {
            for trap in read_traps(&file).expect("a trap file reads").trap {
                let Some(rule) = trap.expect.rule() else {
                    continue;
                };
                let id: &str = &rule.0;
                checked += 1;
                distinct.insert(id.to_string());

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

        (checked, distinct.len())
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
        let (checked, distinct) = assert_rule_ids_are_canonical(&committed_traps_root());
        // ASSERTED here rather than quoted in prose: without these two, the canonicality assertions
        // above hold over whatever the walk happens to reach, and a truncated walk or a new eighth
        // rule id would pass in silence. They live in THIS test and not in the helper — see its
        // `# Returns`.
        assert_eq!(
            checked, 23,
            "twenty-three of the twenty-six committed traps name a rule; the other three are \
             `must-abstain` and name a cause"
        );
        assert_eq!(
            distinct, 7,
            "the corpus writes seven distinct rule ids — the figure `l1.rs`'s doc and this story \
             both quote, asserted so an eighth reds instead of passing"
        );
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
    /// of the denominator in silence — which is exactly what story 5.8's blocking bucket prevents.
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
        // The VARIANT, not the length: the map is total since story 5.8, so `len() == 1` would hold
        // just as well if the prefix selector had declined this trap — which is precisely the
        // behaviour this test exists to refuse.
        assert!(
            matches!(
                answers.get(&TrapId("scratch-unimplemented-l1".into())),
                Some(Answer::Answered(_))
            ),
            "the prefix selector ANSWERS an `l1-*` rule this engine does not implement, rather \
             than bucketing it — a whitelist of the two implemented ids would let a future L1 rule \
             leave the denominator in silence"
        );
        assert_eq!(answers.len(), 1);

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

        // The DIRECTORY, not just the file, and the same call every sibling scratch test in
        // `trap_gate.rs` makes: `remove_file` alone leaked one empty directory per run into
        // `/tmp` (measured: three of them before this was fixed).
        std::fs::remove_dir_all(&dir).ok();
    }

    // ── The cross-file id guard (story 5.8, inherited from 5.7's review) ──────

    /// 🔴 One `TrapId` in two files is REFUSED by the runner, and the test calls the runner
    /// **directly**.
    ///
    /// `TrapFile::validate` enforces uniqueness only WITHIN a file. Before story 5.8 a duplicate
    /// here merely overwrote an entry; now the map is total and its LENGTH is read by the residue
    /// arithmetic, so a duplicate shortens a denominator with no diagnostic at all.
    ///
    /// ⚠️ **`score_corpus` refuses this same corpus for its own reasons** — the assertion at the
    /// end measures that, and it is why this test may not go through the harness: composed with
    /// `score_corpus` the guard is invisible, and a test written that way stays GREEN with the
    /// runner's guard deleted. It would be measuring the harness, which already worked.
    #[test]
    fn one_trap_id_in_two_files_is_refused_by_the_runner_itself() {
        let dir = scratch_dir("duplicate-id");
        let body = |id: &str| {
            format!(
                r#"
[[trap]]
id = "{id}"
replay = "scenario/replay/minimal.jsonl"
observations = [
  "aaaaaaaa-0000-4000-8000-000000000001",
  "aaaaaaaa-0000-4000-8000-000000000003",
]
reason = "one id declared in two separate files, which no per-file validation can ever catch."
expect = {{ must-not-merge = {{ rule = "l1-distinct-mac" }} }}
"#
            )
        };
        std::fs::write(dir.join("first.toml"), body("shared-id")).expect("first file writes");
        std::fs::write(dir.join("second.toml"), body("shared-id")).expect("second file writes");

        let err =
            l1_answers(&dir).expect_err("a duplicate id is refused, not silently overwritten");
        match &err {
            FixtureError::DuplicateTrapId {
                trap,
                first,
                second,
            } => {
                assert_eq!(trap, "shared-id");
                // The two PATHS, named and distinct. `assert_ne!(first, second)` alone held by
                // construction — `seen` can only collide across files, since `TrapFile::validate`
                // refuses a within-file duplicate before this loop ever sees it.
                assert_eq!(first, &dir.join("first.toml"), "{err}");
                assert_eq!(second, &dir.join("second.toml"), "{err}");
            }
            other => panic!("expected DuplicateTrapId, got {other:?}"),
        }

        // The harness refuses it too — stated here as the measured reason this test calls
        // `l1_answers` rather than `score_corpus`.
        assert!(
            matches!(
                crate::trap_gate::score_corpus(&dir, &BTreeMap::new()),
                Err(FixtureError::DuplicateTrapId { .. })
            ),
            "score_corpus has its own guard, which is exactly why it cannot be the one under test"
        );

        // 🔴 And the ids are compared FOLDED, as `TrapFile::validate` folds them within a file.
        // Measured by story 5.8's code review against the raw-keyed first version: these two passed
        // it, `discovered` read 2, and the report rendered two bucket lines a reader cannot tell
        // apart — the exact harm `TrapError::DuplicateId`'s own doc exists to prevent.
        std::fs::write(dir.join("first.toml"), body("Shared-Id")).expect("first file rewrites");
        assert!(
            matches!(l1_answers(&dir), Err(FixtureError::DuplicateTrapId { .. })),
            "`Shared-Id` and `shared-id` are one id across files, as they are within one"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    /// The committed corpus has no duplicate — so the guard above is recorded as
    /// unreachable-today by MEASUREMENT rather than assumed to be.
    #[test]
    fn the_committed_corpus_has_twenty_six_distinct_ids_across_eleven_files() {
        let files = discover_trap_files(&committed_traps_root()).expect("the corpus walks");
        assert_eq!(files.len(), 11, "eleven committed trap files");

        let mut ids: Vec<TrapId> = Vec::new();
        for file in &files {
            for trap in read_traps(file).expect("a committed trap file reads").trap {
                ids.push(trap.id);
            }
        }
        let distinct: BTreeSet<&TrapId> = ids.iter().collect();
        assert_eq!(ids.len(), 26, "twenty-six traps");
        assert_eq!(
            distinct.len(),
            ids.len(),
            "and every id is distinct ACROSS files, not merely within one"
        );
    }
}
