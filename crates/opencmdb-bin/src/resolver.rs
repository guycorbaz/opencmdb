//! The identity resolver: one deterministic pass that turns a set of observations into interfaces
//! and the identity links that place them (story 5.9b).
//!
//! This is the first production caller of [`candidates`] and the first cross-crate caller of
//! [`join`]. Story 5.9 built the schema and wrote no link the engine had derived; this file is what
//! fills it, and it is what story 5.10 re-runs after purging the engine's links.
//!
//! # The mechanism: `join` NAMES the interface, the blocker and `decide_pair` JUSTIFY the placement
//!
//! [`decide_pair`] judges a PAIR of observations and returns no interface — a `Decision` carries a
//! conclusion, a verdict vector and a ruleset version, and no key. So a pair verdict can say *"these
//! two are on the same interface"* and can never say *which*. [`join`] can: it groups observations
//! by the scope-qualified key `(l2_domain, mac)`, and at L1 that map IS the set of interfaces
//! [architecture.md:984-985].
//!
//! The unit of work is therefore `join`'s `(key, group)`. The key names the interface; the blocker
//! proposes the pairs; `decide_pair` supplies the rule and the evidence that justify each placement.
//! D13's order — *candidate generation (blocking) → verdicts → three-way decision*
//! [architecture.md:931] — is the order of the pass, and the blocker is called ONCE over the whole
//! slice before any verdict is asked for.
//!
//! ## Why not build the groups from the `Match` pairs
//!
//! Because `verdict_for_pair` uses an **existential** quantifier — the pair matches when it shares
//! AT LEAST ONE key — so A sharing `k1` with B and B sharing `k2` with C makes A–B and B–C both
//! `Match`, and a connected-component grouping would fuse A with C **although they share no key**:
//! two genuinely distinct interfaces merged. `the_pass_does_not_fuse_a_with_c` measures it, and it
//! must be measured through THIS module rather than through `join` and `decide_pair` directly — a
//! test that never calls the resolver cannot see the resolver's grouping change.
//!
//! ## The singleton, and the abstention
//!
//! A group of ONE has no pair to judge, so [`decide_singleton`] answers it: at L1 the interface IS
//! the key, and an observation carrying that key sits on it by [`join`]'s definition. The self-pair
//! `(o, o)` is deliberately not used to manufacture a pair — `CandidatePair::new` refuses two equal
//! ids, and building it here would re-open in a caller what that constructor closes in the type.
//!
//! **An observation abstains AT MOST ONCE, whatever the number of keys it carries.** Guy's
//! arbitration at this story's code review, on a measurement: an abstention row names no key —
//! `identity_link` holds `observation_id`, a NULL `interface_id` and nothing else — so two
//! abstention rows for one observation would be identical but for their id. They also collide, both
//! landing on `ABSTAINED_SUBJECT` in `identity_link_one_current`, which made the whole pass fail
//! `Constraint("unique")` and roll back. The duplicate was never information; it was one sentence
//! written twice.
//!
//! ## Why the blocker is not decoration
//!
//! [`candidates`] is TOTAL today — every unordered pair of distinct ids — so "the universe" and
//! "all pairs" coincide. That does not make consulting it pointless: it is where the universe is
//! DEFINED, and a pass that read `join`'s keys and never asked would be correct today and silently
//! stop being correct the first time the blocker excludes anything, which F17's `dormant` already
//! plans to make it do [architecture.md:1205]. [`resolve_within`] exists so that the exclusion is
//! reachable from a test rather than only from a future.
//!
//! # No instant COLUMN is read from the clock
//!
//! ⚠️ The heading used to read *"the clock is never read"*, and that was false: this file calls
//! `uuid::Uuid::now_v7()` twice per link, and a v7 UUID embeds a 48-bit wall-clock millisecond —
//! straight into `interface.id` and `identity_link.id`. That is the house idiom (`ObsId` and
//! `ConnectorId` are v7 too), so the sentence is what changes, not the code. **The consequence
//! belongs to story 5.10**: its *"reproduced identically, bit for bit"* can only ever mean *modulo
//! the ids*, because a replayed link is minted afresh.
//!
//! Every instant this pass stores IN A COLUMN is derived from the observations: an interface's window is the
//! `min`/`max` of its group's `observed_at`, and a link's `valid_from` is its own observation's.
//! *"The engine never touches the clock"* [architecture.md:3364], and story 5.10 replays this pass
//! and compares bit for bit. `insert_declared_attribute`'s `NOW(6)` is a DECLARED row authored by a
//! human and is not a precedent.
//!
//! # It IS idempotent, since story 5.11
//!
//! A second pass over the same observations writes nothing at all — not even the same rows again.
//! [`write_link`] reads the current ENGINE version of the slot it is about to fill and takes one of
//! three branches: insert into an empty slot, return without writing when the slot already holds
//! this decision, or close-and-append when it holds a different one. [`Resolution`] reports which,
//! and `links_written = 0` is what a cycle that learned nothing looks like.
//!
//! _(Until story 5.11 the pass appended blindly and a second run was `Err(Constraint("unique"))`
//! with a full rollback, `identity_link_one_current` refusing the second current row.)_
//!
//! **What counts as a CHANGED decision is Guy's arbitration**: the six columns [`same_decision`]
//! compares, **evidence included**. A singleton and a pair both conclude `Match`/`l1-exact-mac`, so
//! without evidence in the set a link whose group grew would keep asserting a justification that is
//! no longer true — and FR16 renders it. The cost is stated rather than discovered: a newcomer that
//! becomes the group's smallest-other witness supersedes every incumbent, which is O(group size)
//! writes and is measured at both ends by
//! `the_write_amplification_is_measured_at_both_ends`.
//!
//! ⚠️ **The engine never supersedes an OPERATOR's row.** [`load_current_engine_link`] filters on
//! `decided_by = 'ENGINE'`, so a human's assertion is invisible to the compare and the pass fails on
//! its insert exactly as it did before — *"may an operator override the engine?"* stays story 5.14's
//! question. Without that filter the engine either ADOPTS a human's row as its own — when it
//! happens to carry the same decision — or SUPERSEDES it when it does not. Both are measured;
//! [`load_current_engine_link`] says which test measures which, because the doc used to claim the
//! second while every test in the workspace exercised the first.
//!
//! # Not wired into `main.rs`
//!
//! By decision, not by omission. The named consumers are stories 5.10 and 5.11; wiring the startup
//! scan would make every deployment write links with no page to display them (story 5.14) and no
//! purge to remove them (story 5.10) — a behaviour change no acceptance criterion asks for. Hence
//! the `allow` below, in the idiom `repo.rs:11` already carries.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use opencmdb_core::identity::blocking::{CandidatePair, candidates};
use opencmdb_core::identity::cascade::{Conclusion, Decision, IdentityAbstentionCause, decide};
use opencmdb_core::identity::l1::{CURRENT_RULESET_VERSION, decide_pair, decide_singleton, join};
use opencmdb_core::observation::{InterfaceId, LinkId, ObsId, Observation, Timestamp};
use opencmdb_core::repo::RepositoryError;
use sqlx::MySqlConnection;

use crate::repo::{
    DecidedBy, PersistedLink, cause_token, classify, close_identity_link, datetime_literal,
    find_interface_by_l1_key, insert_identity_link, insert_interface, load_current_engine_link,
    load_current_engine_slots, open_end, outcome_token, subject_of, widen_interface_seen_window,
};

/// What one pass did, in counts. Rows, not opinions: every field is something a test can also read
/// back out of the database, which is the point — an oracle that restates the pass's own summary
/// measures nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Resolution {
    /// How many pairs the universe this pass was HANDED contains. [`resolve`] supplies
    /// `n(n-1)/2` over distinct observation ids, which is what makes the blocker's size assertable
    /// rather than quotable; [`resolve_within`] reports whatever its caller passed, which is `0` for
    /// the narrowed-universe test. _(This doc promised the formula unconditionally until the code
    /// review, which is false through the seam.)_
    pub candidate_pairs: usize,
    /// Interfaces created by this pass, because their L1 key had never been seen.
    pub interfaces_minted: usize,
    /// Interfaces this pass found by key instead of minting. Story 5.10's replay depends on this
    /// being non-zero on a second pass: a re-minted id would change every reproduced link.
    pub interfaces_found: usize,
    /// Links written, placements and abstentions together. A NEW version, whether it opened an
    /// empty slot or replaced a superseded one — so an idempotent pass reports `0` here.
    pub links_written: usize,
    /// How many of those links are abstentions — a link with no interface and a cause.
    pub abstentions: usize,
    /// Versions CLOSED by this pass because the decision they carried had changed (story 5.11).
    /// Every one of them has a matching entry in [`Self::links_written`]: a supersede is a close
    /// plus an append, never one without the other.
    pub links_superseded: usize,
    /// Slots this pass found already holding the decision it was about to write, and left alone.
    /// Readable back out of the database as *"the row still has the id it had before the pass"* —
    /// which is what distinguishes writing nothing from rewriting the same thing.
    pub links_unchanged: usize,
    /// Current engine links CLOSED because the input no longer supports the slot they held — an
    /// observation that stopped carrying a MAC, or that stopped abstaining.
    ///
    /// Unlike [`Self::links_superseded`] these get **no successor**: there is nothing to write,
    /// which is the whole point. `superseded + vacated` is therefore the number of versions this
    /// pass closed, and only `superseded` has a matching entry in [`Self::links_written`].
    pub links_vacated: usize,
}

impl Resolution {
    /// Fold one slot's [`WriteOutcome`] into the counts.
    ///
    /// `abstentions` stays a SUBSET of `links_written`, which is what its doc promises: an
    /// abstention the pass left untouched wrote no link, so counting it there would make
    /// `abstentions > links_written` reachable and the field would stop meaning *"how many of those
    /// links"*. An idempotent pass over a MAC-less observation therefore reports
    /// `links_written = 0, abstentions = 0, links_unchanged = 1`.
    fn record(&mut self, outcome: WriteOutcome, is_abstention: bool) {
        match outcome {
            WriteOutcome::Written => self.links_written += 1,
            WriteOutcome::Superseded => {
                self.links_written += 1;
                self.links_superseded += 1;
            }
            WriteOutcome::Unchanged => {
                self.links_unchanged += 1;
                return;
            }
        }
        if is_abstention {
            self.abstentions += 1;
        }
    }
}

/// Resolve a slice of observations and write what the engine derives.
///
/// Computes the candidate universe and delegates to [`resolve_within`]. It opens no transaction:
/// **the caller MUST wrap it in `WriteRepository::transact`**, which is what gives the pass
/// read-your-own-writes, so a second observation of one MAC sees the interface the first one caused.
///
/// ⚠️ **That is a precondition, not a structure, and the doc said otherwise until this story's code
/// review.** The parameter is a `&mut MySqlConnection`, which a pooled connection also satisfies —
/// measured: called that way, a pass that then failed left **2 interfaces and 2 links committed**
/// under autocommit. D21's *"an identity decision is NEVER split across two transactions"* holds
/// when the caller cooperates. Taking a unit-of-work type instead would make it structural; that is
/// registered rather than done here, because it would move every call site story 5.9 wrote.
///
/// # Errors
///
/// Any [`RepositoryError`] the writes produce. Two are worth naming because they are this
/// function's own, not the adapter's:
///
/// - [`RepositoryError::InstantRegressed`] when an observation is re-supplied with an
///   `observed_at` EARLIER than the version already stored for that slot;
/// - [`RepositoryError::Constraint`]`("unique")` when an OPERATOR holds a slot this pass needs.
///   That is not the old non-idempotence — a second pass over the same observations now writes
///   nothing at all — it is the frontier described in this module's doc.
pub async fn resolve(
    conn: &mut MySqlConnection,
    observations: &[Observation],
) -> Result<Resolution, RepositoryError> {
    let universe = candidates(observations);
    resolve_within(conn, observations, &universe).await
}

/// Resolve against a universe the caller supplies — the seam [`resolve`] delegates through.
///
/// It exists so that "the blocker is not bypassed" is FALSIFIABLE. With the universe computed
/// internally, deleting the containment check below was measured leaving the entire suite green,
/// because [`candidates`] is total and every pair is in it. Handing this function a narrowed
/// universe is what turns that no-op into a red.
///
/// An observation whose every pair was excluded gets an abstention, and this module's
/// [`nothing_was_evaluated`] is the `Decision` it carries.
///
/// # Errors
///
/// Any [`RepositoryError`] the writes produce.
pub async fn resolve_within(
    conn: &mut MySqlConnection,
    observations: &[Observation],
    universe: &BTreeSet<CandidatePair>,
) -> Result<Resolution, RepositoryError> {
    let groups = join(observations);

    // 🔴 Refuse a self-contradictory slice BEFORE anything is written (story 5.11b).
    //
    // This map used to be a plain `.collect()`, which is LAST-DUPLICATE-WINS. That made the pass
    // read the slice's arrival order twice over: `join` walks the whole slice, so an observation is
    // grouped under the keys of every copy, while `by_id` hands the placement loop only the last —
    // so an observation could be placed on an interface derived from a MAC its winning copy does
    // not carry, and a different arrival order would place it elsewhere. The winning copy's
    // `observed_at` is also what `write_link` stores as `valid_from` and what `seen_window` folds,
    // so a permutation could move a STORED column.
    //
    // The answer is a refusal rather than a documented precondition: an `ObsId` names one immutable
    // observation, and two contents under one id is a caller bug the pass should say out loud.
    let mut by_id: BTreeMap<ObsId, &Observation> = BTreeMap::new();
    for observation in observations {
        if let Some(previous) = by_id.insert(observation.obs_id, observation)
            && contradicts(previous, observation)
        {
            return Err(RepositoryError::ContradictoryObservation);
        }
    }

    let mut summary = Resolution {
        candidate_pairs: universe.len(),
        ..Resolution::default()
    };
    // Observations that got a PLACEMENT. An observation that abstains is deliberately NOT in
    // here, so it falls through to the tail loop and gets exactly one abstention link — see the
    // module doc's "one abstention per observation".
    let mut placed: BTreeSet<ObsId> = BTreeSet::new();
    // Which SUBJECTS this pass wrote or kept, per observation. What is not in here at the end is a
    // slot the input no longer supports, and the tail below closes it.
    let mut visited: BTreeMap<ObsId, BTreeSet<String>> = BTreeMap::new();

    for ((l2_domain, mac_canon), group) in &groups {
        let (first_seen_at, last_seen_at) = seen_window(group, &by_id);

        let interface = match find_interface_by_l1_key(&mut *conn, *l2_domain, mac_canon)
            .await
            .map_err(classify)?
        {
            Some(existing) => {
                summary.interfaces_found += 1;
                widen_interface_seen_window(&mut *conn, existing, first_seen_at, last_seen_at)
                    .await
                    .map_err(classify)?;
                existing
            }
            None => {
                let minted = InterfaceId::from_uuid(uuid::Uuid::now_v7());
                insert_interface(
                    &mut *conn,
                    minted,
                    *l2_domain,
                    mac_canon,
                    first_seen_at,
                    last_seen_at,
                )
                .await
                .map_err(classify)?;
                summary.interfaces_minted += 1;
                minted
            }
        };

        for obs_id in group {
            let observation = by_id[obs_id];
            // No `else` branch: an observation the blocker proposed nothing for is left out of
            // `placed` and abstains ONCE, in the tail loop, however many keys it carries.
            if let Some(decision) = placement_decision(observation, group, universe, &by_id) {
                let outcome =
                    write_link(&mut *conn, observation, Some(interface), &decision).await?;
                placed.insert(*obs_id);
                visited
                    .entry(*obs_id)
                    .or_default()
                    .insert(subject_of(Some(interface)));
                summary.record(outcome, false);
            }
        }
    }

    // Everything that was not placed abstains, and abstains EXACTLY ONCE — an observation carrying
    // no MAC at all, and an observation whose pairs the blocker withheld on every key it carries.
    // The link is written and never omitted: *"the ambiguity is DATA, not a hole"* (D14/FR16).
    let mut abstained: BTreeSet<ObsId> = BTreeSet::new();
    for observation in observations {
        if placed.contains(&observation.obs_id) || !abstained.insert(observation.obs_id) {
            continue;
        }
        let outcome = write_link(&mut *conn, observation, None, &nothing_was_evaluated()).await?;
        visited
            .entry(observation.obs_id)
            .or_default()
            .insert(subject_of(None));
        summary.record(outcome, true);
    }

    // 🔴 Close every current ENGINE slot this pass did NOT visit.
    //
    // `write_link` only ever reads the slot it is about to FILL, so a key that vanished from the
    // input produces no iteration and no visit — and its link would stay current forever, pointing
    // at an interface no fact in the input supports. Measured at this story's code review, on the
    // `multi-nic` shape: an observation that carried two MACs and now carries one left a second
    // current link standing, and `snapshot_links` returned two rows where a replay produces one —
    // a reachable counterexample to story 5.10's purge-and-replay invariant, through pure engine
    // input.
    //
    // ⚠️ **It is this story that made the case silent.** Before the compare, `insert_identity_link`
    // appended blindly and `identity_link_one_current` refused the second write LOUDLY, with a full
    // rollback; the compare routes around the key, so the detection has to be explicit. The
    // uniqueness key was doing this work, and taking its job means taking its duty.
    //
    // Only observations this pass actually SAW are considered: an observation absent from the slice
    // is not evidence that its links are stale, it is evidence of nothing.
    for observation in observations {
        let keep = visited.get(&observation.obs_id);
        for (link_id, subject) in load_current_engine_slots(&mut *conn, observation.obs_id)
            .await
            .map_err(classify)?
        {
            if keep.is_some_and(|subjects| subjects.contains(&subject)) {
                continue;
            }
            close_identity_link(&mut *conn, link_id_of(&link_id)?, observation.observed_at).await?;
            summary.links_vacated += 1;
        }
    }

    Ok(summary)
}

/// Whether two observations supplied under ONE `ObsId` disagree about anything a decision reads.
///
/// # Why not `a != b`
///
/// `Observation` derives `PartialEq`, so the bare comparison would compile and would be one line.
/// It would also refuse a slice where nothing was ever at stake: `raw` is *"opaque provenance …
/// that NO decision ever reads"* (D19) [`observation/mod.rs`], so two copies differing only there
/// contradict nothing. Guy's arbitration at story 5.11b. The comparison is therefore explicit.
///
/// # The destructuring is the guard, and it has no `..`
///
/// A new field on `Observation` breaks this function at COMPILE time and forces whoever adds it to
/// say whether a decision reads it. That is the one thing an explicit comparison otherwise lets a
/// new field escape in silence, and it is why the pattern below names all six fields.
///
/// # `facts` is compared as a SET, not as a sequence
///
/// Same reasoning as `raw`, applied one level down: nothing reads the order of `facts` — `keys_of`
/// collects them into a `BTreeSet` — so two copies differing only in the order their facts were
/// serialised in contradict nothing either, and refusing them would be the over-broad refusal this
/// story's AC5 warns against. ⚠️ Stated honestly: the containment test below is blind to
/// MULTIPLICITY (`[x, x, y]` and `[x, y, y]` compare equal). A repeated fact inside one observation
/// is pathological and reaches no decision — `keys_of` de-duplicates — and the failure direction is
/// the safe one: the guard declines to refuse, leaving the pre-5.11b behaviour rather than
/// inventing a new one.
fn contradicts(a: &Observation, b: &Observation) -> bool {
    let Observation {
        // Equal by construction: this is only ever called on two copies of ONE id.
        obs_id: _,
        connector_id,
        observed_at,
        scope,
        facts,
        // D19 — no decision reads it. See this function's doc.
        raw: _,
    } = a;

    *connector_id != b.connector_id
        || *observed_at != b.observed_at
        || *scope != b.scope
        || facts.len() != b.facts.len()
        || !facts.iter().all(|fact| b.facts.contains(fact))
        || !b.facts.iter().all(|fact| facts.contains(fact))
}

/// The `Decision` that justifies placing `observation` on the interface its group names, or `None`
/// when the blocker proposed no pair this observation could be judged on.
///
/// A group of two or more is judged by [`decide_pair`] against a WITNESS: the smallest `ObsId` in
/// the group other than this one. The group is a `BTreeSet`, so "smallest other" is deterministic
/// and a re-run reproduces it — which is what story 5.10 replays. The rule and the evidence on the
/// link are then the engine's, not a constant this file chose.
///
/// A group of ONE has no pair, and [`decide_singleton`] is the engine's answer for it. The
/// self-pair `(o, o)` is deliberately not used to manufacture one: `CandidatePair::new` refuses two
/// equal ids, and building the pair here would re-open in a caller what that constructor closes in
/// the type.
fn placement_decision(
    observation: &Observation,
    group: &BTreeSet<ObsId>,
    universe: &BTreeSet<CandidatePair>,
    by_id: &BTreeMap<ObsId, &Observation>,
) -> Option<Decision> {
    if group.len() == 1 {
        return Some(decide_singleton(observation));
    }
    // The SMALLEST other id whose pair the blocker actually proposed — the containment test is
    // inside the search, not after it. ⚠️ Measured before this was so: with the filter applied to
    // one candidate witness instead of to all of them, a universe missing the single pair (1,2)
    // made observations 1 AND 2 abstain although (1,3) and (2,3) were both proposed. The `min` is
    // taken over the survivors, so removing a pair can change which witness is chosen but never
    // silences an observation the blocker still speaks about.
    let witness = group
        .iter()
        .filter(|id| **id != observation.obs_id)
        .find(|id| {
            CandidatePair::new(observation.obs_id, **id)
                .is_some_and(|pair| universe.contains(&pair))
        })?;
    Some(decide_pair(observation, by_id[witness]))
}

/// `Abstained { AbsenceOfProof }` with an empty verdict vector — the value of an EMPTY verdict set.
///
/// It comes from [`decide`], not from a struct literal: this file composes no verdict, it hands the
/// algebra the empty vector and takes what the algebra says. `decide(vec![], _)` abstains for
/// absence of proof [`cascade.rs`], and an empty vector is literally true here — nothing was
/// evaluated, because nothing was proposed.
///
/// ⚠️ **`absence_of_proof` is a cause of CONVENIENCE in the excluded-pair case, not a true one.**
/// The engine has two causes and neither means *"the blocker declined to propose"*. Choosing one is
/// deferred rather than decided: `candidates` is TOTAL today, so no caller can reach that branch,
/// and the first story that NARROWS the blocker owns the semantics. For the MAC-less observation
/// the cause IS true — there was no proof.
fn nothing_was_evaluated() -> Decision {
    decide(Vec::new(), CURRENT_RULESET_VERSION)
}

/// The `min`/`max` `observed_at` of a group — an interface's seen-window, derived and never read
/// from the clock.
///
/// # Panics
///
/// Never: `join` only produces non-empty groups, and every id in one came from `observations`.
fn seen_window(
    group: &BTreeSet<ObsId>,
    by_id: &BTreeMap<ObsId, &Observation>,
) -> (Timestamp, Timestamp) {
    let mut instants = group.iter().map(|id| by_id[id].observed_at);
    let first = instants.next().expect("join never produces an empty group");
    instants.fold((first, first), |(lo, hi), at| (lo.min(at), hi.max(at)))
}

/// Write one link for `observation`, guarded, at the sentinel.
///
/// `valid_from` is the observation's own `observed_at` and `valid_to` is `OPEN_END`; the evidence is
/// the decision's own, so a link names what actually argued for it (D19).
///
/// ⚠️ **It keeps the FIRST verdict's evidence only.** Every `Decision` L1 produces carries exactly
/// one verdict, so nothing is lost today — but that is an L1 accident, not a property, and
/// `cascade.rs` says Epic 6's cascade ends it. On that day the evidence of verdicts 2..n would
/// vanish with nothing red. Registered with Epic 6 rather than pre-solved: unioning evidence across
/// a vector is a decision about what a link MEANS, and no producer exists to decide it against.
async fn write_link(
    conn: &mut MySqlConnection,
    observation: &Observation,
    interface: Option<InterfaceId>,
    decision: &Decision,
) -> Result<WriteOutcome, RepositoryError> {
    guard_decision(decision, &[])?;
    let evidence: Vec<ObsId> = decision
        .verdict_vector
        .first()
        .map(|verdict| verdict.evidence.clone())
        .unwrap_or_default();

    let subject = subject_of(interface);
    let held = load_current_engine_link(&mut *conn, observation.obs_id, &subject)
        .await
        .map_err(classify)?;

    if let Some(current) = held {
        if same_decision(&current.link, interface, decision, &evidence) {
            return Ok(WriteOutcome::Unchanged);
        }
        // 🔴 The instant may not run backwards, and this guard exists ABOVE the DDL on purpose.
        // Measured at this story's code review: with only `identity_link_interval` to catch it, one
        // observation whose `observed_at` regressed destroyed the WHOLE cycle — every unrelated
        // observation in the batch rolled back — under an anonymous `Constraint("check")` naming no
        // cause; and the same regression was SILENT one branch over, because `same_decision` does
        // not compare `valid_from`. One condition, two opposite answers. Now it has one.
        //
        // ⚠️ This is the FIRST production caller in this codebase to compare an instant it HOLDS
        // against one the database STORED, which is precisely the `datetime_literal` debt the
        // register has carried since story 5.9. It compares RENDERINGS — `sqlx` is built without
        // its `chrono` feature, so a `DATETIME(6)` has no Rust type to decode into — and
        // `datetime_literal`'s fixed-width `%Y-%m-%d %H:%M:%S%.6f` makes lexicographic order agree
        // with chronological order. The residue is real and named: the rendering TRUNCATES below
        // the microsecond, so two instants less than 1 µs apart compare EQUAL here and the guard
        // does not fire. `repo::tests::datetime_literal_truncates_below_the_microsecond` (story
        // 5.10) is what pins that truncation.
        if datetime_literal(observation.observed_at) < current.valid_from {
            return Err(RepositoryError::InstantRegressed);
        }
        // The close instant is the NEW version's `valid_from`, which is this same observation's
        // `observed_at` — so the chain is exact and half-open, and it is zero-length whenever the
        // caller supplies a stable instant. `0004_supersede_admits_a_zero_length_version.sql` is
        // what admits that; under 0002's strict form this line was `ERROR 4025`.
        close_identity_link(
            &mut *conn,
            link_id_of(&current.link.id)?,
            observation.observed_at,
        )
        .await?;
        insert_identity_link(
            &mut *conn,
            LinkId::from_uuid(uuid::Uuid::now_v7()),
            observation.obs_id,
            interface,
            decision,
            &evidence,
            DecidedBy::Engine,
            observation.observed_at,
            open_end(),
        )
        .await
        .map_err(classify)?;
        return Ok(WriteOutcome::Superseded);
    }

    insert_identity_link(
        conn,
        LinkId::from_uuid(uuid::Uuid::now_v7()),
        observation.obs_id,
        interface,
        decision,
        &evidence,
        DecidedBy::Engine,
        observation.observed_at,
        open_end(),
    )
    .await
    .map_err(classify)?;
    Ok(WriteOutcome::Written)
}

/// Parse a stored link id back into a [`LinkId`].
///
/// The id was minted by this crate as a v7 UUID and stored in a `CHAR(36) ascii_bin` column, so a
/// parse failure means a row written around the adapter — a DECODE problem, not a rejected write.
/// It is therefore [`RepositoryError::Backend`] and not a `Constraint(_)`: naming a constraint that
/// exists in no migration would send a caller looking through the schema for it, and would tell a
/// retry policy the database refused a write when it did not.
///
/// # Errors
///
/// [`RepositoryError::Backend`] when the stored id is not a UUID.
fn link_id_of(stored: &str) -> Result<LinkId, RepositoryError> {
    uuid::Uuid::parse_str(stored)
        .map(LinkId::from_uuid)
        .map_err(|e| RepositoryError::Backend(format!("stored link id is not a uuid: {e}")))
}

/// What [`write_link`] did to one slot. The three branches of *"no new version for an unchanged
/// decision"*, named rather than encoded as a pair of booleans a caller could combine wrongly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WriteOutcome {
    /// The slot was empty and a first version was inserted.
    Written,
    /// The slot held a DIFFERENT decision: that version was closed and a new one appended.
    Superseded,
    /// The slot already held this exact decision. Nothing was written — not even the same row again.
    Unchanged,
}

/// Does the persisted version carry the decision the pass is about to write?
///
/// # It is PURE, and that is not a stylistic choice
///
/// Five of the six columns below cannot be reddened through the database, and this was measured
/// rather than suspected. **`interface_id` is structural and stays so forever**: the lookup key
/// handed to [`load_current_engine_link`] IS `subject_of(interface)`, and
/// `identity_link_current_subject` makes `current_subject = interface_id` on every current
/// placement — so a row found by that key necessarily carries that `interface_id`, at any level.
/// _(An earlier version of this doc blamed L1 for all five, which reads as though Epic 6 would make
/// them reddenable; for this one it never can.)_ The other four ARE L1's doing: within a `join`
/// group every member shares the group's key, `decide_pair` and `decide_singleton` both conclude
/// `Match` with `l1-exact-mac`, the cause is `None` on any placement, and `ruleset_version` is the
/// constant `CURRENT_RULESET_VERSION`.
///
/// **Evidence is the only difference an L1 pass can produce**, so a comparison exercised only
/// through a pass would measure ONE column while claiming six — story 5.9's M3 family, where the
/// adapter cannot emit the incoherent value the guard exists to catch.
///
/// Being pure, it has its own database-free tests, one per column. That is what makes the other
/// five reddenable today rather than on the day Epic 6 supplies a second rule.
///
/// # What is NOT compared, each for a stated reason
///
/// - `observation_id` and `current_subject` are the LOOKUP KEY — a row that did not match them was
///   never a candidate for this comparison;
/// - `valid_to` is the sentinel on every current row, held there by `identity_link_current_subject`;
/// - `decided_by` is `ENGINE` by [`load_current_engine_link`]'s own `WHERE`;
/// - `valid_from` is the observation's own `observed_at`, and both versions of one placement are
///   versions of the SAME observation's placement. Comparing it would be a guard that can never
///   differ — except through a caller that re-supplies an `obs_id` with a different instant, which
///   nothing enforces and which `0004`'s comment records;
/// - `id` is a v7 UUID. Story 5.10 settled that a row identifier is not a decision.
fn same_decision(
    current: &PersistedLink,
    interface: Option<InterfaceId>,
    decision: &Decision,
    evidence: &[ObsId],
) -> bool {
    let (rule_id, abstention_cause) = match &decision.conclusion {
        Conclusion::Match { rule } | Conclusion::NoMatch { rule } => (Some(rule.0.as_str()), None),
        Conclusion::Abstained { cause } => (None, Some(cause_token(cause))),
    };
    current.interface_id.as_deref() == interface.map(|i| i.to_string()).as_deref()
        && current.outcome == outcome_token(&decision.conclusion)
        && current.rule_id.as_deref() == rule_id
        && current.abstention_cause.as_deref() == abstention_cause
        && current.evidence == evidence
        && current.ruleset_version == decision.ruleset_version.0
}

/// Refuse a decision that cannot be honestly persisted, before any row is written.
///
/// Two refusals, and **neither is reachable through [`resolve`]** — L1 emits no `Supports` and no
/// `Opposes`, so it cannot conclude `Ambiguous` at all, and its two rule ids are non-empty
/// constants. They are guards on the WRITER rather than on the pass, and their tests call this
/// function directly, because a test written through the resolver would stay green with the guard
/// deleted. That is story 5.8's measured lesson, applied one story later.
///
/// - **An `Ambiguous` abstention with no candidates** would make *"the ambiguity is DATA, not a
///   hole"* a convention rather than an invariant: FR16 would render an empty candidate list and
///   there would be nothing to display, which is the vapour D14 names. An `AbsenceOfProof`
///   abstention with no candidates is CORRECT and is not refused — nothing was a candidate.
/// - **An empty `rule_id`** satisfies `identity_link_rule_xor_cause`, which only tests
///   `IS NOT NULL`. *"A decision names the rule that settled it"* is not met by an empty name, and
///   D19 wants the id left behind because a rule that fires without one is undebuggable.
///   `0003_resolver_guards.sql` carries the same refusal in DDL, as a second line of defence.
///
/// ⚠️ **Nothing fills `candidates_for_link` yet**: the only call site passes `&[]`, and this pass
/// writes no `link_candidate` row because L1 has no ambiguity to hold candidates for. So the day a
/// producer of `Ambiguous` arrives, this guard would refuse a LEGITIMATE ambiguity rather than let
/// it be written with its candidates — the inverse of FR16. **Whoever produces the first
/// `Ambiguous` owns filling this slice**, and that is Epic 6; the guard is written to take it now
/// so the signature does not have to change under them.
///
/// # Errors
///
/// [`RepositoryError::Constraint`] naming which of the two invariants was violated.
fn guard_decision(
    decision: &Decision,
    candidates_for_link: &[InterfaceId],
) -> Result<(), RepositoryError> {
    match &decision.conclusion {
        Conclusion::Abstained {
            cause: IdentityAbstentionCause::Ambiguous,
        } if candidates_for_link.is_empty() => {
            Err(RepositoryError::Constraint("ambiguity_without_candidates"))
        }
        Conclusion::Match { rule } | Conclusion::NoMatch { rule } if rule.0.is_empty() => {
            Err(RepositoryError::Constraint("rule_id_empty"))
        }
        _ => Ok(()),
    }
}

/// Tests for the resolver.
///
/// # Synthetic inputs, by necessity and not by preference
///
/// **One test here reads `fixtures/`** — story 5.11b's
/// `a_committed_stream_derives_the_same_interfaces_in_every_order`, which replays
/// `hostname-absence.jsonl` through `FixtureConnector` because its AC6 asks for the committed
/// corpus by name. _(This sentence read "Nothing here reads `fixtures/`" until story 5.11b, which
/// falsified it in the same commit that corrected it.)_ Everything else is synthetic, and by
/// necessity: two properties this file rests on cannot be exercised by the
/// committed corpus at all: every committed replay stream carries exactly ONE `l2_domain`
/// [`l1.rs:83-88`], so a resolver keyed on the bare MAC would pass the whole corpus; and **no
/// committed observation carries more than one MAC** — measured over all 13 streams, the maximum is
/// one — so the multi-NIC shape that forced story 5.9's uniqueness key to widen appears nowhere in
/// it. ⚠️ `multi-nic` is NOT that shape: it models a multi-NIC host as two single-MAC observations
/// and both its poles expect `l2-*` rules, so it sits in the eleven-unanswerable bucket and L1 never
/// answers it.
///
/// # Every count is read back from the database
///
/// A test that asserts [`Resolution`]'s fields alone is an oracle restating the pass's own summary.
/// Where a count matters, it is `SELECT`ed. The summary is asserted too, and the two agreeing is
/// itself the assertion.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::permute::{SEED_SWEEP, permutations, shuffled};
    use crate::repo::{MariaRepository, OPEN_END, count_identity_links, datetime_literal};
    use opencmdb_core::observation::{
        ConnectorId, Fact, HostnameSource, L2DomainId, MacAddr, Scope, VantageId,
    };
    use opencmdb_core::repo::WriteRepository;
    use sqlx::MySqlPool;

    /// The rule id the committed corpus writes for a `must-merge` pole, restated as a LITERAL so
    /// these assertions do not read the constant they are checking. Deliberate redundancy, in the
    /// idiom `l1.rs`'s test module established and for the same measured reason.
    const CORPUS_EXACT_MAC: &str = "l1-exact-mac";

    fn l2(n: u128) -> L2DomainId {
        L2DomainId::from_uuid(uuid::Uuid::from_u128(n))
    }

    fn mac(last: u8) -> MacAddr {
        MacAddr([0x00, 0x11, 0x22, 0x33, 0x44, last])
    }

    fn at(secs: i64) -> Timestamp {
        chrono::DateTime::from_timestamp(secs, 0).expect("in range")
    }

    /// An observation carrying the given MACs in the given domain, seen at the given instant.
    fn observation(id: u128, domain: L2DomainId, macs: &[MacAddr], seen: i64) -> Observation {
        Observation {
            obs_id: ObsId::from_uuid(uuid::Uuid::from_u128(id)),
            connector_id: ConnectorId::from_uuid(uuid::Uuid::nil()),
            observed_at: at(seen),
            scope: Scope {
                l2_domain: domain,
                vantage: VantageId::from_uuid(uuid::Uuid::nil()),
            },
            facts: macs
                .iter()
                .map(|addr| Fact::Mac {
                    addr: *addr,
                    locally_administered: false,
                })
                .collect(),
            raw: None,
        }
    }

    /// An observation carrying a hostname and NO MAC — the abstention case.
    fn mac_less(id: u128, domain: L2DomainId, seen: i64) -> Observation {
        let mut o = observation(id, domain, &[], seen);
        o.facts = vec![Fact::Hostname {
            name: "nas".to_string(),
            source: HostnameSource::Dns,
        }];
        o
    }

    /// Connect, migrate, empty every table, and insert the observations the links will name.
    ///
    /// The insert is not decoration: `0003_resolver_guards.sql` gives `identity_link.observation_id`
    /// a foreign key, so a link whose observation does not exist is refused — which is the whole
    /// point of this story being the first writer.
    async fn fixture(observations: &[Observation]) -> Option<MySqlPool> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping resolver test: DATABASE_URL unset");
            return None;
        };
        let pool = MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
        // Children before parents.
        for statement in [
            "DELETE FROM link_candidate",
            "DELETE FROM identity_link",
            "DELETE FROM interface",
            "DELETE FROM observation_record",
        ] {
            sqlx::query(statement).execute(&pool).await.expect("clean");
        }
        for observation in observations {
            crate::repo::insert_observation(&pool, observation)
                .await
                .map_err(classify)
                .expect("insert observation");
        }
        Some(pool)
    }

    /// Run one pass inside one transaction, as D21 requires.
    async fn pass(pool: &MySqlPool, observations: Vec<Observation>) -> Resolution {
        try_pass(pool, observations).await.expect("resolve")
    }

    async fn try_pass(
        pool: &MySqlPool,
        observations: Vec<Observation>,
    ) -> Result<Resolution, RepositoryError> {
        MariaRepository::new(pool.clone())
            .transact(move |unit| {
                let observations = observations.clone();
                Box::pin(async move { resolve(unit.executor(), &observations).await })
            })
            .await
    }

    /// Run one pass inside one transaction against a caller-supplied universe.
    async fn within(
        pool: &MySqlPool,
        observations: Vec<Observation>,
        universe: BTreeSet<CandidatePair>,
    ) -> Result<Resolution, RepositoryError> {
        MariaRepository::new(pool.clone())
            .transact(move |unit| {
                let observations = observations.clone();
                let universe = universe.clone();
                Box::pin(
                    async move { resolve_within(unit.executor(), &observations, &universe).await },
                )
            })
            .await
    }

    async fn interface_count(pool: &MySqlPool) -> i64 {
        let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM interface")
            .fetch_one(pool)
            .await
            .expect("count interfaces");
        n
    }

    async fn current_links(pool: &MySqlPool, obs: ObsId) -> Vec<crate::repo::PersistedLink> {
        crate::repo::load_current_links_for_observation(pool, obs)
            .await
            .map_err(classify)
            .expect("load links")
    }

    /// AC2 — two observations sharing one MAC land on ONE interface, each with its own link.
    #[tokio::test]
    async fn two_observations_of_one_mac_share_one_interface() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
        ];
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        let summary = pass(&pool, observations).await;

        assert_eq!(interface_count(&pool).await, 1, "one key, one interface");
        assert_eq!(
            count_identity_links(&pool).await.expect("count links"),
            2,
            "one link per observation — the link places an OBSERVATION, not a group"
        );
        assert_eq!(summary.interfaces_minted, 1);
        assert_eq!(summary.links_written, 2);
        assert_eq!(summary.abstentions, 0);
        assert_eq!(summary.candidate_pairs, 1, "n(n-1)/2 over two distinct ids");
        for id in [1u128, 2] {
            let links = current_links(&pool, ObsId::from_uuid(uuid::Uuid::from_u128(id))).await;
            assert_eq!(links.len(), 1, "exactly one current link per observation");
            assert_eq!(links[0].outcome, "match");
            assert_eq!(links[0].decided_by, "ENGINE");
        }
    }

    /// AC2 — an observation carrying TWO MACs is on TWO interfaces at once, with one current link
    /// on each. This is the shape story 5.9's uniqueness key was widened for, and the corpus
    /// contains no example of it.
    #[tokio::test]
    async fn one_observation_with_two_macs_lands_on_two_interfaces() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![observation(
            1,
            l2(1),
            &[mac(0x01), mac(0x02)],
            1_700_000_000,
        )];
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        let summary = pass(&pool, observations).await;

        assert_eq!(
            interface_count(&pool).await,
            2,
            "one interface per L1 KEY the observation carries — epics.md's 'exactly ONE' is \
             falsified by join, which loops over keys_of"
        );
        let links = current_links(&pool, ObsId::from_uuid(uuid::Uuid::from_u128(1))).await;
        assert_eq!(links.len(), 2, "one current link per interface");
        assert_eq!(summary.interfaces_minted, 2);
        assert_eq!(
            summary.candidate_pairs, 0,
            "one observation proposes no pair — the blocker refuses the self-pair"
        );
    }

    /// AC3 / decision 5 — an observation alone on its key is placed by the key, with itself as
    /// evidence, and NOT by the self-pair.
    #[tokio::test]
    async fn a_singleton_is_placed_with_itself_as_evidence() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![observation(1, l2(1), &[mac(0x01)], 1_700_000_000)];
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        pass(&pool, observations).await;

        let obs = ObsId::from_uuid(uuid::Uuid::from_u128(1));
        let links = current_links(&pool, obs).await;
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].rule_id.as_deref(), Some(CORPUS_EXACT_MAC));
        assert_eq!(
            links[0].evidence,
            vec![obs],
            "one id, named once — the self-pair would name it twice"
        );
        assert_eq!(links[0].ruleset_version, CURRENT_RULESET_VERSION.0);
    }

    /// AC3 — the rule and the evidence of a placement come from a real `decide_pair` call.
    ///
    /// ⚠️ The RULE is knowable in advance (`l1-exact-mac` is the only rule a placement can carry:
    /// `l1-distinct-mac` rides `Disqualifying`, which `decide` turns into `NoMatch`, which this pass
    /// never writes). So **the EVIDENCE is what carries this assertion** — a test checking only the
    /// rule measures a constant.
    #[tokio::test]
    async fn a_placements_evidence_is_the_pair_the_engine_judged() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
        ];
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        pass(&pool, observations).await;

        let one = ObsId::from_uuid(uuid::Uuid::from_u128(1));
        let two = ObsId::from_uuid(uuid::Uuid::from_u128(2));
        for obs in [one, two] {
            let links = current_links(&pool, obs).await;
            assert_eq!(
                links[0].evidence,
                vec![one, two],
                "the sorted pair the engine judged, not a constant this file chose"
            );
        }
    }

    /// AC4 — a second pass over DIFFERENT observations on the same key FINDS the interface.
    ///
    /// Both halves matter: inside one transaction (read-your-own-writes, D21) and across two
    /// (stability, which story 5.10's replay depends on — a re-minted id would change every
    /// reproduced link).
    #[tokio::test]
    async fn a_second_pass_finds_the_interface_rather_than_minting_one() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let all = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
        ];
        let Some(pool) = fixture(&all).await else {
            return;
        };

        let first = pass(&pool, vec![all[0].clone()]).await;
        assert_eq!(first.interfaces_minted, 1);
        assert_eq!(first.interfaces_found, 0);

        let second = pass(&pool, vec![all[1].clone()]).await;
        assert_eq!(
            second.interfaces_found, 1,
            "the key was already there; finding it is what makes the id stable"
        );
        assert_eq!(second.interfaces_minted, 0);
        assert_eq!(interface_count(&pool).await, 1, "still one interface");

        let a = current_links(&pool, all[0].obs_id).await;
        let b = current_links(&pool, all[1].obs_id).await;
        assert_eq!(
            a[0].interface_id, b[0].interface_id,
            "both links point at the same interface row"
        );
    }

    /// AC4 — read-your-own-writes INSIDE one transaction: the second observation of one MAC sees
    /// the interface the first one caused, without a commit in between.
    #[tokio::test]
    async fn the_second_observation_sees_the_first_ones_interface_in_one_transaction() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let all = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
        ];
        let Some(pool) = fixture(&all).await else {
            return;
        };

        let (first, second) = MariaRepository::new(pool.clone())
            .transact(move |unit| {
                let all = all.clone();
                Box::pin(async move {
                    let first = resolve(unit.executor(), &all[0..1]).await?;
                    let second = resolve(unit.executor(), &all[1..2]).await?;
                    Ok((first, second))
                })
            })
            .await
            .expect("two passes in one transaction");

        assert_eq!(first.interfaces_minted, 1);
        assert_eq!(
            second.interfaces_found, 1,
            "the read pool would not have seen it; the unit's connection does"
        );
        assert_eq!(interface_count(&pool).await, 1);
    }

    /// AC6 / decision 7 — an OLDER second batch widens the window at BOTH ends.
    ///
    /// ⚠️ Both ends are asserted deliberately. Under the plain-assignment mutation `first_seen_at`
    /// lands on the right value anyway, so a test naming only that end passes the mutation.
    #[tokio::test]
    async fn an_older_batch_widens_the_window_at_both_ends() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let all = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_500),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_000),
        ];
        let Some(pool) = fixture(&all).await else {
            return;
        };

        pass(&pool, vec![all[0].clone()]).await;
        pass(&pool, vec![all[1].clone()]).await;

        let (first_seen, last_seen): (String, String) = sqlx::query_as(
            "SELECT CAST(first_seen_at AS CHAR), CAST(last_seen_at AS CHAR) FROM interface",
        )
        .fetch_one(&pool)
        .await
        .expect("read the window");

        assert_eq!(
            first_seen,
            datetime_literal(at(1_700_000_000)),
            "the older batch pushed first_seen_at BACK"
        );
        assert_eq!(
            last_seen,
            datetime_literal(at(1_700_000_500)),
            "and did not narrow last_seen_at — this is the end that carries the mutation"
        );
    }

    /// AC3 / AC6 — every stored instant is the derived one, read back and compared.
    ///
    /// Without this, a clock-derived instant is invisible: every other test asserts COUNTS, which
    /// no clock changes.
    #[tokio::test]
    async fn the_stored_instants_are_the_derived_ones() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
        ];
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        pass(&pool, observations).await;

        let (first_seen, last_seen): (String, String) = sqlx::query_as(
            "SELECT CAST(first_seen_at AS CHAR), CAST(last_seen_at AS CHAR) FROM interface",
        )
        .fetch_one(&pool)
        .await
        .expect("read the window");
        assert_eq!(first_seen, datetime_literal(at(1_700_000_000)));
        assert_eq!(last_seen, datetime_literal(at(1_700_000_100)));

        for (id, secs) in [(1u128, 1_700_000_000i64), (2, 1_700_000_100)] {
            let (valid_from, valid_to): (String, String) = sqlx::query_as(
                "SELECT CAST(valid_from AS CHAR), CAST(valid_to AS CHAR) \
                 FROM identity_link WHERE observation_id = ?",
            )
            .bind(ObsId::from_uuid(uuid::Uuid::from_u128(id)).to_string())
            .fetch_one(&pool)
            .await
            .expect("read the link's instants");
            assert_eq!(
                valid_from,
                datetime_literal(at(secs)),
                "valid_from is the observation's own observed_at, never the clock"
            );
            assert_eq!(valid_to, OPEN_END, "current links sit at the sentinel");
        }
    }

    /// AC5 — an observation carrying no MAC gets a LINK with a cause, never an absence.
    #[tokio::test]
    async fn an_observation_without_a_mac_gets_an_abstention_link() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            mac_less(2, l2(1), 1_700_000_100),
        ];
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        let summary = pass(&pool, observations).await;

        assert_eq!(summary.abstentions, 1);
        let blind = ObsId::from_uuid(uuid::Uuid::from_u128(2));
        let links = current_links(&pool, blind).await;
        assert_eq!(links.len(), 1, "the ambiguity is DATA, not a hole");
        assert_eq!(links[0].outcome, "abstained");
        assert_eq!(links[0].interface_id, None);
        assert_eq!(
            links[0].abstention_cause.as_deref(),
            Some("absence_of_proof"),
            "the PERSISTED token, lowercase with an underscore — not the Rust variant name"
        );
        assert_eq!(links[0].rule_id, None);

        let (candidates_rows,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM link_candidate")
            .fetch_one(&pool)
            .await
            .expect("count candidates");
        assert_eq!(
            candidates_rows, 0,
            "an absence-of-proof abstention has no candidates, and that is CORRECT — \
             nothing was a candidate"
        );
    }

    /// AC5 — the two writer guards, called DIRECTLY.
    ///
    /// Neither is reachable through [`resolve`]: L1 emits no `Supports` and no `Opposes`, so it
    /// cannot conclude `Ambiguous`, and its two rule ids are non-empty constants. A test written
    /// through the resolver would stay green with the guard deleted — story 5.8's measured lesson.
    #[test]
    fn the_writer_guards_refuse_what_the_resolver_cannot_produce() {
        use opencmdb_core::identity::cascade::RuleVerdict;
        use opencmdb_core::trap::RuleId;

        let ambiguous = Decision {
            conclusion: Conclusion::Abstained {
                cause: IdentityAbstentionCause::Ambiguous,
            },
            verdict_vector: vec![],
            ruleset_version: CURRENT_RULESET_VERSION,
        };
        assert_eq!(
            guard_decision(&ambiguous, &[]),
            Err(RepositoryError::Constraint("ambiguity_without_candidates")),
            "an ambiguity with nothing to display is FR16 as vapour"
        );
        assert_eq!(
            guard_decision(
                &ambiguous,
                &[InterfaceId::from_uuid(uuid::Uuid::from_u128(9))]
            ),
            Ok(()),
            "with a candidate it is exactly what the schema exists to hold"
        );

        let nameless = Decision {
            conclusion: Conclusion::Match {
                rule: RuleId(String::new()),
            },
            verdict_vector: vec![RuleVerdict {
                rule: RuleId(String::new()),
                verdict: opencmdb_core::identity::cascade::Verdict::Decisive,
                evidence: vec![],
            }],
            ruleset_version: CURRENT_RULESET_VERSION,
        };
        assert_eq!(
            guard_decision(&nameless, &[]),
            Err(RepositoryError::Constraint("rule_id_empty")),
            "a decision names the rule that settled it, and '' is not a name"
        );

        let honest = decide_singleton(&observation(1, l2(1), &[mac(0x01)], 1_700_000_000));
        assert_eq!(guard_decision(&honest, &[]), Ok(()));
    }

    /// AC8 — the two organs agree: every intra-group pair is in the universe and matches on the
    /// corpus's rule, and a pair sharing no key does not.
    ///
    /// Pure — it states the quantifier. ⚠️ It does NOT carry the connected-components mutation:
    /// it never calls the resolver, so the resolver's grouping is invisible to it. That is
    /// [`the_pass_does_not_fuse_a_with_c`]'s job.
    #[test]
    fn the_blocker_and_the_join_agree_about_every_group() {
        use opencmdb_core::trap::RuleId;

        let observations = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
            observation(3, l2(1), &[mac(0x09)], 1_700_000_200),
        ];
        let universe = candidates(&observations);
        let by_id: BTreeMap<ObsId, &Observation> =
            observations.iter().map(|o| (o.obs_id, o)).collect();

        let mut checked = 0usize;
        for group in join(&observations).values() {
            for a in group {
                for b in group {
                    if a >= b {
                        continue;
                    }
                    let pair = CandidatePair::new(*a, *b).expect("distinct ids");
                    assert!(
                        universe.contains(&pair),
                        "the blocker must propose every pair the join puts together"
                    );
                    assert_eq!(
                        decide_pair(by_id[a], by_id[b]).conclusion,
                        Conclusion::Match {
                            rule: RuleId(CORPUS_EXACT_MAC.to_string())
                        },
                        "sharing the key IS the rule"
                    );
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 1, "one intra-group pair in this fixture");

        assert!(
            matches!(
                decide_pair(&observations[0], &observations[2]).conclusion,
                Conclusion::NoMatch { .. }
            ),
            "and a pair sharing no key is not a placement"
        );
    }

    /// AC8 — the transitivity refutation, THROUGH the resolver and against the database.
    ///
    /// A shares `k1` with B, B shares `k2` with C, A and C share nothing. Both A–B and B–C are
    /// `Match`, so a grouping built from the match pairs as connected components would fuse A with
    /// C — two genuinely distinct interfaces. Grouping by KEY does not.
    #[tokio::test]
    async fn the_pass_does_not_fuse_a_with_c() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01), mac(0x02)], 1_700_000_100),
            observation(3, l2(1), &[mac(0x02)], 1_700_000_200),
        ];
        let Some(pool) = fixture(&observations).await else {
            return;
        };
        let a = observations[0].obs_id;
        let c = observations[2].obs_id;

        assert!(
            matches!(
                decide_pair(&observations[0], &observations[1]).conclusion,
                Conclusion::Match { .. }
            ),
            "A and B match"
        );
        assert!(
            matches!(
                decide_pair(&observations[1], &observations[2]).conclusion,
                Conclusion::Match { .. }
            ),
            "B and C match"
        );

        pass(&pool, observations).await;

        assert_eq!(
            interface_count(&pool).await,
            2,
            "two keys, two interfaces — connected components would give one"
        );
        let a_interface = current_links(&pool, a).await[0].interface_id.clone();
        let c_interface = current_links(&pool, c).await[0].interface_id.clone();
        assert_ne!(
            a_interface, c_interface,
            "A and C share no key and must not share an interface"
        );
    }

    /// AC2 / M1 — the key is SCOPE-qualified: one MAC in two `l2_domain`s is two interfaces.
    ///
    /// Synthetic by necessity: every committed replay stream carries one `l2_domain`, so a
    /// resolver that dropped the scope would pass the whole corpus.
    #[tokio::test]
    async fn one_mac_in_two_scopes_is_two_interfaces() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(2), &[mac(0x01)], 1_700_000_100),
        ];
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        pass(&pool, observations).await;

        assert_eq!(
            interface_count(&pool).await,
            2,
            "the same MAC in two L2 domains is two interfaces — the key is scope-qualified"
        );
    }

    /// M12 / decision 12 — an observation whose every pair the blocker excluded is NOT placed.
    ///
    /// ⚠️ This is the test that makes "the blocker is not bypassed" falsifiable. Against the real
    /// blocker the containment check is unreachable — `candidates` is TOTAL — so deleting it leaves
    /// the whole suite green. Handing [`resolve_within`] an EMPTY universe is what reddens it.
    #[tokio::test]
    async fn a_pair_the_blocker_did_not_propose_is_never_judged() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
        ];
        let Some(pool) = fixture(&observations).await else {
            return;
        };
        let narrowed = BTreeSet::new();

        let summary = MariaRepository::new(pool.clone())
            .transact({
                let observations = observations.clone();
                move |unit| {
                    let observations = observations.clone();
                    let narrowed = narrowed.clone();
                    Box::pin(async move {
                        resolve_within(unit.executor(), &observations, &narrowed).await
                    })
                }
            })
            .await
            .expect("resolve within an empty universe");

        assert_eq!(
            summary.abstentions, 2,
            "with no pair proposed, nothing justifies a placement"
        );
        for observation in &observations {
            let links = current_links(&pool, observation.obs_id).await;
            assert_eq!(
                links[0].outcome, "abstained",
                "the pass must not fall back on the join's key when the blocker said nothing"
            );
        }
    }

    /// 🔴 The blocker withholding ONE pair must not silence an observation it still speaks about.
    ///
    /// Three observations on one MAC, universe missing only `(1,2)`. Observation 1 can still be
    /// judged against 3, and 2 against 3 — so all three are PLACED. Measured before the fix: 1 and
    /// 2 both abstained, because `placement_decision` tested containment on a single candidate
    /// witness instead of searching for one. Found by all three code-review layers.
    #[tokio::test]
    async fn withholding_one_pair_does_not_silence_the_others() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
            observation(3, l2(1), &[mac(0x01)], 1_700_000_200),
        ];
        let Some(pool) = fixture(&observations).await else {
            return;
        };
        let withheld =
            CandidatePair::new(observations[0].obs_id, observations[1].obs_id).expect("distinct");
        let narrowed: BTreeSet<CandidatePair> = candidates(&observations)
            .into_iter()
            .filter(|pair| *pair != withheld)
            .collect();

        let summary = within(&pool, observations.clone(), narrowed)
            .await
            .expect("resolve");

        assert_eq!(
            summary.abstentions, 0,
            "each observation still has a proposed pair to be judged on"
        );
        for observation in &observations {
            let links = current_links(&pool, observation.obs_id).await;
            assert_eq!(
                links[0].outcome, "match",
                "{} was silenced",
                observation.obs_id
            );
        }
    }

    /// 🔴 An observation abstaining on SEVERAL keys writes exactly ONE link (Guy's arbitration).
    ///
    /// Two MACs, empty universe: both groups withhold, and before the arbitration the pass wrote
    /// two abstention rows that collided on `ABSTAINED_SUBJECT` — `Err(Constraint("unique"))` and a
    /// full rollback, **zero links written**. The two rows would have been identical but for their
    /// id: an abstention row names no key.
    #[tokio::test]
    async fn an_observation_abstains_once_however_many_keys_it_carries() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        // X carries both MACs, so it sits in BOTH groups; Y and Z give each group a second member,
        // which is what makes the blocker's silence reach X twice. ⚠️ One observation alone with
        // two MACs would NOT do: each group would be a singleton, and a singleton has no pair for
        // the blocker to withhold — measured, it is placed on both interfaces and abstains nowhere.
        let observations = vec![
            observation(1, l2(1), &[mac(0x01), mac(0x02)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
            observation(3, l2(1), &[mac(0x02)], 1_700_000_200),
        ];
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        // ⚠️ NOT `.expect()`: the pre-arbitration failure is a uniqueness violation that rolls the
        // whole transaction back, so an `expect` panics before any count can be taken and the red
        // says nothing about what was written. Story 5.9's M4/M5 lesson, applied here.
        let outcome = within(&pool, observations.clone(), BTreeSet::new()).await;
        assert_eq!(
            outcome.clone().err(),
            None,
            "a multi-key abstention must not fail the pass"
        );
        let summary = outcome.unwrap_or_default();
        assert_eq!(
            summary.abstentions, 3,
            "one abstention per observation — X abstains ONCE although it sits in two groups"
        );
        let x = current_links(&pool, observations[0].obs_id).await;
        assert_eq!(
            x.len(),
            1,
            "two abstention rows for X would be identical but for their id, and would collide"
        );
        assert_eq!(x[0].outcome, "abstained");
        assert_eq!(x[0].interface_id, None);
        assert_eq!(
            count_identity_links(&pool).await.expect("count"),
            3,
            "three observations, three links"
        );
    }

    /// 🔴 The witness convention, pinned on a group of THREE.
    ///
    /// ⚠️ Every other test uses a group of TWO, where "smallest other" and "largest other" name the
    /// same observation — so the convention was measured by nothing, and swapping the two left all
    /// 402 tests green. Decision 4 calls this determinism *"what story 5.10 replays"*, so it is
    /// load-bearing and this test is what holds it.
    #[tokio::test]
    async fn the_witness_is_the_smallest_other_id_in_the_group() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
            observation(3, l2(1), &[mac(0x01)], 1_700_000_200),
        ];
        let Some(pool) = fixture(&observations).await else {
            return;
        };
        let id = |n: u128| ObsId::from_uuid(uuid::Uuid::from_u128(n));

        pass(&pool, observations).await;

        for (subject, expected) in [
            (1u128, vec![id(1), id(2)]),
            (2, vec![id(1), id(2)]),
            (3, vec![id(1), id(3)]),
        ] {
            let links = current_links(&pool, id(subject)).await;
            assert_eq!(
                links[0].evidence, expected,
                "observation {subject} is judged against the SMALLEST other id in its group"
            );
        }
    }

    /// AC3 — a placement of a group of two names the corpus's merge rule, and only that one.
    ///
    /// `l1-distinct-mac` is unwritable here by three independent steps, so this asserts the single
    /// value rather than a set.
    #[tokio::test]
    async fn a_group_placement_names_the_exact_mac_rule() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
        ];
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        pass(&pool, observations.clone()).await;

        for observation in &observations {
            let links = current_links(&pool, observation.obs_id).await;
            assert_eq!(links[0].rule_id.as_deref(), Some(CORPUS_EXACT_MAC));
        }
    }

    /// The same `obs_id` twice in one slice writes ONE link, with or without a MAC.
    ///
    /// The grouped path was already deduped by `join`'s `BTreeSet`; the tail loop iterated the raw
    /// slice and wrote two abstention rows for a repeated MAC-less observation, which collided.
    #[tokio::test]
    async fn a_repeated_obs_id_writes_one_link() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let twice = mac_less(1, l2(1), 1_700_000_000);
        let Some(pool) = fixture(std::slice::from_ref(&twice)).await else {
            return;
        };

        let summary = pass(&pool, vec![twice.clone(), twice.clone()]).await;

        assert_eq!(summary.links_written, 1, "one observation, one link");
        assert_eq!(count_identity_links(&pool).await.expect("count"), 1);
    }

    /// Sorted, so the comparison across the purge is a SEQUENCE and a divergence names the row.
    async fn interface_ids(pool: &MySqlPool) -> Vec<String> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT id FROM interface ORDER BY id")
            .fetch_all(pool)
            .await
            .expect("read the interface ids");
        rows.into_iter().map(|r| r.0).collect()
    }

    /// A fixture broad enough for the purge to mean something: a group of THREE (so the witness
    /// convention is exercised), a MULTI-MAC observation (two interfaces at once) and a MAC-LESS
    /// one (an abstention). Six links over three interfaces.
    fn purge_fixture() -> Vec<Observation> {
        vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
            observation(3, l2(1), &[mac(0x01)], 1_700_000_200),
            observation(4, l2(1), &[mac(0x02), mac(0x03)], 1_700_000_300),
            mac_less(5, l2(1), 1_700_000_400),
        ]
    }

    /// 🔴 AC3/AC4 — D14's own test: purge the engine's links, re-run, and the decisions come back.
    ///
    /// ⚠️ **TWO columns are excluded, each for its own recorded reason.** `current_subject` is a
    /// FUNCTION of `interface_id` on a current row, held there by `identity_link_current_subject`,
    /// so comparing it would measure nothing new — it is the `ORDER BY` key instead. And `id`: `identity_link.id` is a v7 UUID and
    /// a v7 UUID embeds a wall-clock millisecond, so a replayed link is minted with a different one
    /// — measured 57 ms apart over identical input. D14's *"bit for bit"* means the DECISION, and a
    /// row identifier is not a decision. `LinkSnapshot` has no `id` field at all, so the exclusion
    /// is structural.
    ///
    /// **`interface_id` IS compared**, which is what makes that safe: if the replay re-minted its
    /// interfaces, every reproduced link would point elsewhere and this would red.
    #[tokio::test]
    async fn every_decision_bearing_column_survives_a_purge_and_replay() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = purge_fixture();
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        let first = pass(&pool, observations.clone()).await;
        assert_eq!(first.links_written, 6, "3 + 2 + 1 abstention");
        assert_eq!(first.interfaces_minted, 3);
        let before = crate::repo::snapshot_links(&pool)
            .await
            .map_err(classify)
            .expect("snapshot before");
        let interfaces_before = interface_ids(&pool).await;
        assert_eq!(before.len(), 6, "six current links to reproduce");

        // 🔴 A ONE-SIDED oracle, and it is not optional. `before` and `after` both go through
        // `snapshot_links`, so the comparison below cannot see a column the QUERY gets wrong —
        // measured at the code review: eight of the ten compared columns could be replaced by a
        // constant with the whole suite green, and `rule_id` was asserted nowhere in the workspace.
        // These assert against values this test knows independently of the query.
        let placed = before
            .iter()
            .find(|l| l.outcome == "match")
            .expect("a placement among the six");
        assert_eq!(placed.rule_id.as_deref(), Some("l1-exact-mac"));
        assert_eq!(placed.decided_by, "ENGINE");
        assert_eq!(placed.ruleset_version, 1);
        assert!(
            interfaces_before.contains(placed.interface_id.as_ref().expect("a placement")),
            "a placement names one of the interfaces this pass minted"
        );
        assert_eq!(placed.valid_to, crate::repo::OPEN_END);
        let abstained = before
            .iter()
            .find(|l| l.outcome == "abstained")
            .expect("the MAC-less observation abstains");
        assert_eq!(abstained.interface_id, None);
        assert_eq!(abstained.rule_id, None);
        assert_eq!(
            abstained.abstention_cause.as_deref(),
            Some("absence_of_proof")
        );
        assert_eq!(
            abstained.valid_from,
            crate::repo::datetime_literal(at(1_700_000_400)),
            "valid_from is the observation's own observed_at"
        );

        let purged = MariaRepository::new(pool.clone())
            .transact(|unit| {
                Box::pin(async move {
                    crate::repo::purge_engine_links(unit.executor())
                        .await
                        .map_err(classify)
                })
            })
            .await
            .expect("purge");
        assert_eq!(purged, 6, "every link here is the engine's");
        assert_eq!(
            count_identity_links(&pool).await.expect("count"),
            0,
            "the purge empties the table"
        );

        // AC4 first, so its red is not pre-empted by anything the replay asserts.
        assert_eq!(
            interface_ids(&pool).await,
            interfaces_before,
            "interfaces are NOT purged, and their ids are the same rows"
        );

        let second = pass(&pool, observations).await;
        let after = crate::repo::snapshot_links(&pool)
            .await
            .map_err(classify)
            .expect("snapshot after");

        assert_eq!(
            after, before,
            "every decision-bearing column is reproduced; only the row ids differ, and they are \
             not in the snapshot"
        );
        assert_eq!(
            second.interfaces_found, 3,
            "the replay FINDS its interfaces; minting them would have reddened the line above"
        );
        assert_eq!(second.interfaces_minted, 0);
    }

    /// 🔴 The SECOND sort key: two current links of ONE observation come back by subject.
    ///
    /// ⚠️ **Measured absent by two review layers.** `ORDER BY observation_id, current_subject` →
    /// `ORDER BY observation_id` left the whole suite green, because the tiebreak is only reachable
    /// when one `observation_id` owns two current rows, and the ordering test used three single-MAC
    /// observations. This fixture reaches it, and makes the physical order DISAGREE with the sorted
    /// one:
    ///
    /// - `A` carries `mac03` and is resolved FIRST, so its interface is minted first and gets the
    ///   SMALLER v7 id; `B` carries `mac02` and is minted second, so its id is LARGER;
    /// - `C` carries both. `join`'s key is `(l2_domain, mac)` and `mac02 < mac03`, so C's links are
    ///   WRITTEN iface02 (larger id) then iface03 (smaller). Sorted by `current_subject` they must
    ///   come back the other way round.
    #[tokio::test]
    async fn the_tiebreak_orders_one_observations_links_by_their_subject() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let a = observation(1, l2(1), &[mac(0x03)], 1_700_000_000);
        let b = observation(2, l2(1), &[mac(0x02)], 1_700_000_100);
        let c = observation(3, l2(1), &[mac(0x02), mac(0x03)], 1_700_000_200);
        let Some(pool) = fixture(&[a.clone(), b.clone(), c.clone()]).await else {
            return;
        };

        pass(&pool, vec![a]).await;
        pass(&pool, vec![b]).await;
        pass(&pool, vec![c.clone()]).await;

        let by_mac: Vec<(String, String)> =
            sqlx::query_as("SELECT mac_canon, id FROM interface ORDER BY mac_canon")
                .fetch_all(&pool)
                .await
                .expect("the two interfaces");
        assert_eq!(by_mac.len(), 2);
        let iface02 = by_mac[0].1.clone();
        let iface03 = by_mac[1].1.clone();
        assert!(
            iface03 < iface02,
            "mac03's interface was minted first, so its v7 id is the smaller one"
        );

        let subjects: Vec<String> = crate::repo::snapshot_links(&pool)
            .await
            .map_err(classify)
            .expect("snapshot")
            .into_iter()
            .filter(|l| l.observation_id == c.obs_id.to_string())
            .map(|l| l.interface_id.expect("a placement"))
            .collect();

        assert_eq!(
            subjects,
            vec![iface03.clone(), iface02.clone()],
            "sorted by subject"
        );
        assert_ne!(
            subjects,
            vec![iface02, iface03],
            "and NOT in the order the two links were written — which is what gives the second \
             sort key something to do"
        );
    }

    /// 🔴 A SUPERSEDED engine link is state the purge removes and the replay never restores.
    ///
    /// ⚠️ **This test exists because the story claimed the opposite.** It said a purge-and-replay is
    /// blind to a link-keyed dependency *"by construction"*, on the ground that the purge restores
    /// the state run 1 started from. That is a property of the OTHER fixture, not of the purge:
    /// `purge_engine_links` has **no `current_subject` filter**, so it deletes superseded engine
    /// rows too, while `snapshot_links` excludes them. The Acceptance Auditor built the refuting
    /// fixture and measured a link-keyed mutation reddening the comparison; this is that fixture,
    /// kept as a guard rather than as a caveat.
    ///
    /// Story 5.11 is the one that will start producing superseded links, and it inherits this.
    #[tokio::test]
    async fn a_superseded_engine_link_is_not_restored_by_the_replay() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let mut observations = purge_fixture();
        let extra = observation(9, l2(9), &[mac(0x09)], 1_700_000_900);
        observations.push(extra.clone());
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        // Place the extra observation, then supersede its link: it leaves the snapshot's domain
        // (current_subject becomes NULL) but stays in the table, and stays the ENGINE's.
        pass(&pool, vec![extra.clone()]).await;
        let its_link = crate::repo::load_current_links_for_observation(&pool, extra.obs_id)
            .await
            .map_err(classify)
            .expect("its link");
        crate::repo::close_identity_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::parse_str(&its_link[0].id).expect("a link id")),
            at(1_700_001_000),
        )
        .await
        .expect("supersede it");

        let rows_before = count_identity_links(&pool).await.expect("count");
        let snapshot_before = crate::repo::snapshot_links(&pool)
            .await
            .map_err(classify)
            .expect("snapshot");
        assert_eq!(rows_before, 1, "the superseded row is still in the table");
        assert_eq!(
            snapshot_before.len(),
            0,
            "and the snapshot does not see it — which is exactly why the comparison cannot \
             notice its loss"
        );

        let purged = MariaRepository::new(pool.clone())
            .transact(|unit| {
                Box::pin(async move {
                    crate::repo::purge_engine_links(unit.executor())
                        .await
                        .map_err(classify)
                })
            })
            .await
            .expect("purge");

        assert_eq!(purged, 1, "the purge takes the SUPERSEDED engine row too");
        pass(&pool, observations).await;
        let superseded_after: i64 = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM identity_link WHERE current_subject IS NULL",
        )
        .fetch_one(&pool)
        .await
        .expect("count superseded")
        .0;
        assert_eq!(
            superseded_after, 0,
            "the replay writes only current links, so the superseded row is NOT restored — the \
             store after a purge-and-replay is NOT the store before it"
        );
    }

    /// 🔴 The two natures are mutually exclusive on ONE placement, and here it is measured.
    ///
    /// ⚠️ The claim stood in five documents and in no test until the code review. Both halves:
    /// an operator cannot take the slot the engine holds, and an operator holding a slot the replay
    /// needs makes the whole replay fail.
    #[tokio::test]
    async fn an_operator_cannot_take_a_slot_the_engine_holds() {
        use opencmdb_core::identity::cascade::RulesetVersion;

        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![mac_less(1, l2(1), 1_700_000_000)];
        let Some(pool) = fixture(&observations).await else {
            return;
        };
        pass(&pool, observations.clone()).await;

        let abstention = Decision {
            conclusion: Conclusion::Abstained {
                cause: IdentityAbstentionCause::AbsenceOfProof,
            },
            verdict_vector: vec![],
            ruleset_version: RulesetVersion(1),
        };
        let clash = crate::repo::insert_identity_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::from_u128(0x5_0_1_0)),
            observations[0].obs_id,
            None,
            &abstention,
            &[],
            DecidedBy::Operator,
            at(1_700_000_100),
            open_end(),
        )
        .await
        .map_err(classify);
        assert_eq!(
            clash,
            Err(RepositoryError::Constraint("unique")),
            "the engine already holds (observation, NIL_INTERFACE); the operator cannot have it too"
        );

        // And with the slot taken first, the replay fails and rolls back.
        crate::repo::purge_engine_links(&pool)
            .await
            .expect("clear the engine's link");
        crate::repo::insert_identity_link(
            &pool,
            LinkId::from_uuid(uuid::Uuid::from_u128(0x5_0_1_1)),
            observations[0].obs_id,
            None,
            &abstention,
            &[],
            DecidedBy::Operator,
            at(1_700_000_100),
            open_end(),
        )
        .await
        .map_err(classify)
        .expect("the slot is free now");
        assert_eq!(
            try_pass(&pool, observations).await.err(),
            Some(RepositoryError::Constraint("unique")),
            "the replay needs a slot the operator holds, and the whole pass rolls back"
        );
        assert_eq!(
            count_identity_links(&pool).await.expect("count"),
            1,
            "only the operator's row is left — the pass wrote nothing"
        );
    }

    /// An empty store: the purge removes nothing and says so, and the snapshot is empty.
    #[tokio::test]
    async fn an_empty_store_purges_nothing_and_snapshots_nothing() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let Some(pool) = fixture(&[]).await else {
            return;
        };

        assert_eq!(
            crate::repo::purge_engine_links(&pool)
                .await
                .expect("purge nothing"),
            0
        );
        assert_eq!(
            crate::repo::snapshot_links(&pool)
                .await
                .map_err(classify)
                .expect("snapshot nothing"),
            vec![]
        );
    }

    /// 🔴 AC5 — the operator's rows are INPUTS, and the purge does not touch them.
    ///
    /// ⚠️ **The operator's link names an observation the pass does NOT place, and that is a
    /// constraint, not a convenience.** `identity_link_one_current` is
    /// `(observation_id, current_subject)` and the purge removes only `decided_by='ENGINE'`, so an
    /// operator can never hold **the SAME `(observation, subject)` slot** the engine holds — that
    /// write is refused `Err(Constraint("unique"))` — and an operator row sitting in a slot the
    /// replay needs makes the **whole replay roll back**. ⚠️ Placing the observation on a DIFFERENT
    /// interface is permitted today: an operator link on another subject inserts fine, measured at
    /// the code review. _(An earlier draft said an operator can never "confirm or correct" a
    /// placement, and correcting normally means moving it — which the schema allows.)_ Measured at this story's validation. D14's *"two
    /// natures in one table"* is true of the TABLE and false of one `(observation, subject)`;
    /// whether an operator may ever override the engine is registered with story 5.14.
    ///
    /// ⚠️ **The candidate goes AROUND `guard_decision`, deliberately.**
    /// The constraint being worked around is `resolver::guard_decision`, which refuses
    /// `Abstained { Ambiguous }` with an empty candidate slice — the shape this test needs.
    /// ⚠️ **NOT the schema**: nothing in the DDL restricts candidates to abstentions, and
    /// `deferred-work.md` already records the measurement — *"`link_candidate` rows attach happily
    /// to a MATCH link, measured `Ok(())`"*, from story 5.9's own review. _(An earlier draft of this
    /// comment blamed `identity_link_abstained_has_no_interface`, which constrains `interface_id`
    /// against `outcome` INSIDE `identity_link` and says nothing about candidates. A doc comment
    /// contradicted by a measurement already in the register.)_ So the `Decision` is hand-built and
    /// `insert_identity_link` is called directly, as `repo.rs`'s own `an_abstention` tests do. The
    /// candidate points at an interface the ENGINE minted: `link_candidate_interface_fk` is
    /// RESTRICT.
    #[tokio::test]
    async fn the_operators_rows_and_their_candidates_survive_the_purge() {
        use opencmdb_core::identity::cascade::RulesetVersion;

        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = purge_fixture();
        let Some(pool) = fixture(&observations).await else {
            return;
        };
        pass(&pool, observations).await;
        let engine_interface = interface_ids(&pool)
            .await
            .first()
            .expect("the pass minted an interface")
            .clone();

        // A sixth observation the pass never saw, so the operator's link contends for no slot.
        let untouched = mac_less(6, l2(1), 1_700_000_500);
        crate::repo::insert_observation(&pool, &untouched)
            .await
            .map_err(classify)
            .expect("insert the operator's observation");

        let operator_link = LinkId::from_uuid(uuid::Uuid::from_u128(0x0_9E_5A));
        let ambiguous = Decision {
            conclusion: Conclusion::Abstained {
                cause: IdentityAbstentionCause::Ambiguous,
            },
            verdict_vector: vec![],
            ruleset_version: RulesetVersion(1),
        };
        crate::repo::insert_identity_link(
            &pool,
            operator_link,
            untouched.obs_id,
            None,
            &ambiguous,
            &[untouched.obs_id],
            DecidedBy::Operator,
            at(1_700_000_500),
            open_end(),
        )
        .await
        .map_err(classify)
        .expect("the operator's link");
        crate::repo::insert_link_candidate(
            &pool,
            operator_link,
            InterfaceId::from_uuid(
                uuid::Uuid::parse_str(&engine_interface).expect("a minted interface id"),
            ),
            &[untouched.obs_id],
        )
        .await
        .map_err(classify)
        .expect("the operator's candidate");

        // AC5 says "byte-identical on ALL TEN compared columns", so the operator's whole row is
        // captured before the purge. Asserting two fields would have left eight unchecked.
        let operator_before: Vec<crate::repo::LinkSnapshot> = crate::repo::snapshot_links(&pool)
            .await
            .map_err(classify)
            .expect("snapshot before")
            .into_iter()
            .filter(|l| l.decided_by == "OPERATOR")
            .collect();
        assert_eq!(operator_before.len(), 1);

        let purged = MariaRepository::new(pool.clone())
            .transact(|unit| {
                Box::pin(async move {
                    crate::repo::purge_engine_links(unit.executor())
                        .await
                        .map_err(classify)
                })
            })
            .await
            .expect("purge");

        assert_eq!(purged, 6, "the engine's six, and not the operator's one");
        let survivors = crate::repo::snapshot_links(&pool)
            .await
            .map_err(classify)
            .expect("snapshot");
        assert_eq!(
            survivors.len(),
            1,
            "one row survives, and it is the operator's"
        );
        assert_eq!(
            survivors, operator_before,
            "untouched on every compared column, not merely present"
        );

        // Its own id did not move: an operator row is the SAME row, not a reproduced decision.
        let still_there = crate::repo::load_current_links_for_observation(&pool, untouched.obs_id)
            .await
            .map_err(classify)
            .expect("read it back");
        assert_eq!(still_there.len(), 1);
        assert_eq!(
            still_there[0].id,
            operator_link.to_string(),
            "an INPUT keeps its identity; only derivations are re-minted"
        );

        let candidates = crate::repo::load_link_candidates(&pool, operator_link)
            .await
            .map_err(classify)
            .expect("its candidates");
        assert_eq!(
            candidates.len(),
            1,
            "the candidate went with its link, which stayed"
        );
        assert_eq!(candidates[0].0, engine_interface);
    }

    /// 🔴 AC2 — the snapshot's order comes from its query, not from the order rows were written.
    ///
    /// ⚠️ **This test exists because the purge-and-replay can NEVER carry the `ORDER BY`**, and that
    /// is structural: both snapshots go through the same query, so any order stable within a run
    /// yields two equal sequences whatever the fixture. Measured at the validation — deleting the
    /// `ORDER BY` left the whole suite green. Here the physical order DISAGREES with the prescribed
    /// one, so the clause has something to do.
    #[tokio::test]
    async fn the_snapshot_is_ordered_by_the_query_not_by_insertion() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
            observation(3, l2(1), &[mac(0x01)], 1_700_000_200),
        ];
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        // One pass each, in DESCENDING id order, so the rows are physically written 3, 2, 1.
        for observation in observations.iter().rev() {
            pass(&pool, vec![observation.clone()]).await;
        }

        let snapshot = crate::repo::snapshot_links(&pool)
            .await
            .map_err(classify)
            .expect("snapshot");
        let seen: Vec<String> = snapshot.iter().map(|l| l.observation_id.clone()).collect();
        let expected: Vec<String> = observations.iter().map(|o| o.obs_id.to_string()).collect();
        assert_eq!(
            seen, expected,
            "ascending observation_id, whatever order the rows went in"
        );
    }

    /// Decision 8 / AC9 — ONE FULL PASS at the reference scale, with its wall-clock.
    ///
    /// ⚠️ Added at the code review, which measured that no test called `resolve` at scale: the
    /// quadratic assertion below exercises `candidates` alone, so decision 8's *"the Debug Log
    /// records the wall-clock of one pass"* had nothing behind it.
    ///
    /// 300 hosts is NFR30's reference scale on a Plus-class NAS. **No timing is asserted** — a
    /// wall-clock assertion is a flaky test on shared hardware; the number is printed and the
    /// Debug Log carries it. What IS asserted is that the pass completes and writes what it should.
    #[tokio::test]
    async fn one_full_pass_at_the_reference_scale() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations: Vec<Observation> = (0..300u128)
            .map(|i| {
                let mut o = observation(i + 1, l2(1), &[], 1_700_000_000 + i as i64);
                o.facts = vec![Fact::Mac {
                    addr: MacAddr([0x00, 0x11, 0x22, (i >> 16) as u8, (i >> 8) as u8, i as u8]),
                    locally_administered: false,
                }];
                o
            })
            .collect();
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        let started = std::time::Instant::now();
        let summary = pass(&pool, observations.clone()).await;
        let elapsed = started.elapsed();
        eprintln!(
            "reference scale, cold: n=300, pairs={}, interfaces={}, links={}, pass={:?}",
            summary.candidate_pairs, summary.interfaces_minted, summary.links_written, elapsed
        );

        assert_eq!(
            summary.candidate_pairs, 44_850,
            "n(n-1)/2 at 300 distinct ids"
        );
        assert_eq!(
            summary.interfaces_minted, 300,
            "300 distinct MACs, 300 interfaces"
        );
        assert_eq!(summary.links_written, 300);
        assert_eq!(summary.abstentions, 0);
        assert_eq!(
            interface_count(&pool).await,
            300,
            "read back, not taken from the summary"
        );

        // AC6 — the same pass again, at the same scale. A cycle that learned nothing writes
        // nothing, and this is where that stops being a two-observation claim.
        let started = std::time::Instant::now();
        let idempotent = pass(&pool, observations).await;
        let elapsed = started.elapsed();
        eprintln!(
            "reference scale, idempotent rerun: written={}, superseded={}, unchanged={}, pass={:?}",
            idempotent.links_written,
            idempotent.links_superseded,
            idempotent.links_unchanged,
            elapsed
        );
        assert_eq!(idempotent.links_written, 0);
        assert_eq!(idempotent.links_superseded, 0);
        assert_eq!(idempotent.links_unchanged, 300);
        assert_eq!(
            idempotent.interfaces_found, 300,
            "and it finds every interface rather than minting one"
        );
        assert_eq!(
            count_identity_links(&pool).await.expect("count"),
            300,
            "read back: the second pass appended nothing"
        );
    }

    /// Decision 8 — the universe is `n(n-1)/2` over DISTINCT ids, asserted rather than quoted.
    ///
    /// ⚠️ At the reference scale (300 hosts) that is **44 850**, not the "90k" D13's prose quotes:
    /// the figure there counts pairs the other way. Write the measured number.
    #[test]
    fn the_universe_is_quadratic_and_its_size_is_asserted() {
        for n in [0usize, 1, 2, 5, 20] {
            let observations: Vec<Observation> = (0..n)
                .map(|i| observation(i as u128 + 1, l2(1), &[mac(i as u8)], 1_700_000_000))
                .collect();
            assert_eq!(
                candidates(&observations).len(),
                n * n.saturating_sub(1) / 2,
                "every unordered pair of distinct ids, and nothing else"
            );
        }

        let reference: Vec<Observation> = (0..300u128)
            .map(|i| observation(i + 1, l2(1), &[mac(i as u8)], 1_700_000_000))
            .collect();
        assert_eq!(
            candidates(&reference).len(),
            44_850,
            "300 hosts on the reference NAS — the measured figure, not D13's prose"
        );
    }

    // ── Story 5.11: idempotence ─────────────────────────────────────────────────────────────────

    /// A persisted link carrying exactly what `decide_singleton(o)` places `o` on `iface` with —
    /// the baseline every `same_decision` case below perturbs in ONE column.
    fn persisted_for(observation: &Observation, iface: InterfaceId) -> PersistedLink {
        PersistedLink {
            id: uuid::Uuid::now_v7().to_string(),
            interface_id: Some(iface.to_string()),
            outcome: "match".to_string(),
            rule_id: Some(CORPUS_EXACT_MAC.to_string()),
            abstention_cause: None,
            evidence: vec![observation.obs_id],
            ruleset_version: 1,
            decided_by: "ENGINE".to_string(),
        }
    }

    /// AC2b — the baseline is UNCHANGED. Without this the five perturbation tests below could all
    /// pass on a `same_decision` that always returns `false`.
    #[test]
    fn an_identical_version_is_the_same_decision() {
        let o = observation(1, l2(1), &[mac(0x01)], 1_700_000_000);
        let iface = InterfaceId::from_uuid(uuid::Uuid::now_v7());
        let decision = decide_singleton(&o);
        let evidence = vec![o.obs_id];
        assert!(same_decision(
            &persisted_for(&o, iface),
            Some(iface),
            &decision,
            &evidence
        ));
    }

    /// 🔴 AC2b — one case per column, WITHOUT a database.
    ///
    /// ⚠️ **Five of these six are unreddenable through a pass**, and that was measured at this
    /// story's validation: dropping every column but `evidence` from `same_decision` left the whole
    /// suite green. At L1 the interface is a function of the observation's own key, every group
    /// member shares it, `decide_pair` and `decide_singleton` both conclude `Match`/`l1-exact-mac`,
    /// the cause is `None` on a placement and `ruleset_version` is a constant — so evidence is the
    /// only difference a pass can produce. A comparison tested only through a pass measures ONE
    /// column and claims six. This is the test that makes the other five red.
    #[test]
    fn each_decision_bearing_column_alone_makes_it_a_different_decision() {
        let o = observation(1, l2(1), &[mac(0x01)], 1_700_000_000);
        let other = observation(2, l2(1), &[mac(0x01)], 1_700_000_100);
        let iface = InterfaceId::from_uuid(uuid::Uuid::now_v7());
        let elsewhere = InterfaceId::from_uuid(uuid::Uuid::now_v7());
        let decision = decide_singleton(&o);
        let evidence = vec![o.obs_id];

        // 1. interface_id — the placement itself.
        let mut perturbed = persisted_for(&o, iface);
        perturbed.interface_id = Some(elsewhere.to_string());
        assert!(
            !same_decision(&perturbed, Some(iface), &decision, &evidence),
            "a link on another interface is another decision"
        );

        // 2. outcome.
        let mut perturbed = persisted_for(&o, iface);
        perturbed.outcome = "no_match".to_string();
        assert!(!same_decision(
            &perturbed,
            Some(iface),
            &decision,
            &evidence
        ));

        // 3. rule_id — the rule that settled it.
        let mut perturbed = persisted_for(&o, iface);
        perturbed.rule_id = Some("l1-distinct-mac".to_string());
        assert!(!same_decision(
            &perturbed,
            Some(iface),
            &decision,
            &evidence
        ));

        // 4. abstention_cause. A placement carries none; a stored one is a different decision even
        //    though `identity_link_rule_xor_cause` would have refused that row.
        let mut perturbed = persisted_for(&o, iface);
        perturbed.abstention_cause = Some("absence_of_proof".to_string());
        assert!(!same_decision(
            &perturbed,
            Some(iface),
            &decision,
            &evidence
        ));

        // 5. evidence — the only one a pass can reach on its own (§2, Guy's arbitration).
        let mut perturbed = persisted_for(&o, iface);
        perturbed.evidence = vec![o.obs_id, other.obs_id];
        assert!(!same_decision(
            &perturbed,
            Some(iface),
            &decision,
            &evidence
        ));

        // 6. ruleset_version — D14: a ruleset change is a decision change, even at equal outcome.
        let mut perturbed = persisted_for(&o, iface);
        perturbed.ruleset_version = 2;
        assert!(!same_decision(
            &perturbed,
            Some(iface),
            &decision,
            &evidence
        ));
    }

    /// AC2b — an abstention compares on its CAUSE, and an abstention is never a placement.
    #[test]
    fn an_abstention_and_a_placement_are_never_the_same_decision() {
        let o = mac_less(1, l2(1), 1_700_000_000);
        let iface = InterfaceId::from_uuid(uuid::Uuid::now_v7());
        let abstention = nothing_was_evaluated();

        let mut stored = persisted_for(&o, iface);
        stored.interface_id = None;
        stored.outcome = "abstained".to_string();
        stored.rule_id = None;
        stored.abstention_cause = Some("absence_of_proof".to_string());
        stored.evidence = vec![];
        assert!(
            same_decision(&stored, None, &abstention, &[]),
            "the same abstention, unchanged"
        );

        let mut other_cause = stored.clone();
        other_cause.abstention_cause = Some("ambiguous".to_string());
        assert!(
            !same_decision(&other_cause, None, &abstention, &[]),
            "a different cause is a different decision"
        );

        // 🔴 The CROSS-NATURE comparison the name promises, and which this test did not make until
        // the code review pointed out that both cases above are abstention-versus-abstention.
        // Measured absent: without these two, a `same_decision` that could not tell a placement
        // from an abstention at all would keep this test green.
        let placement = decide_singleton(&o);
        assert!(
            !same_decision(&stored, None, &placement, &[o.obs_id]),
            "a stored ABSTENTION is never a PLACEMENT decision"
        );
        assert!(
            !same_decision(&persisted_for(&o, iface), Some(iface), &abstention, &[]),
            "and a stored PLACEMENT is never an abstention"
        );
    }

    /// Read every version of one observation's links, current and superseded, oldest row first.
    async fn versions(
        pool: &MySqlPool,
        obs: ObsId,
    ) -> Vec<(String, String, String, Option<String>)> {
        let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as(
            "SELECT id, CAST(valid_from AS CHAR), CAST(valid_to AS CHAR), evidence \
             FROM identity_link WHERE observation_id = ? ORDER BY valid_to, id",
        )
        .bind(obs.to_string())
        .fetch_all(pool)
        .await
        .expect("read the versions");
        rows
    }

    /// 🔴 AC1 — a second identical pass writes NOTHING, and the `id`s prove it.
    ///
    /// This is strictly stronger than story 5.10's purge-and-replay comparison, which deliberately
    /// excludes `id` because a replayed link is re-minted. Here nothing is re-minted: these are the
    /// SAME rows. Comparing the ids is what distinguishes *wrote nothing* from *rewrote the same
    /// thing*, and mutation M3a proves the assertion load-bearing.
    #[tokio::test]
    async fn a_second_identical_pass_writes_nothing_at_all() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = purge_fixture();
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        let first = pass(&pool, observations.clone()).await;
        assert_eq!(first.links_written, 6, "3 + 2 + 1 abstention");
        assert_eq!(first.links_unchanged, 0);
        assert_eq!(first.links_superseded, 0);
        let before = crate::repo::snapshot_links(&pool)
            .await
            .map_err(classify)
            .expect("snapshot before");
        let ids_before = all_link_ids(&pool).await;
        assert_eq!(ids_before.len(), 6);

        let second = pass(&pool, observations).await;

        assert_eq!(second.links_written, 0, "a cycle that learned nothing");
        assert_eq!(second.links_superseded, 0);
        assert_eq!(second.abstentions, 0, "no abstention LINK was written");
        assert_eq!(
            second.links_unchanged, 6,
            "every slot already held its decision"
        );
        assert_eq!(
            all_link_ids(&pool).await,
            ids_before,
            "the same rows, not re-minted ones — this is what `id` equality measures"
        );
        assert_eq!(
            crate::repo::snapshot_links(&pool)
                .await
                .map_err(classify)
                .expect("snapshot after"),
            before
        );
        assert_eq!(
            count_identity_links(&pool).await.expect("count"),
            6,
            "no version was appended"
        );
    }

    async fn all_link_ids(pool: &MySqlPool) -> Vec<String> {
        let rows: Vec<(String,)> = sqlx::query_as("SELECT id FROM identity_link ORDER BY id")
            .fetch_all(pool)
            .await
            .expect("read the link ids");
        rows.into_iter().map(|r| r.0).collect()
    }

    /// 🔴 AC3 — a changed witness supersedes, the old version survives, and its interval is
    /// ZERO-LENGTH.
    ///
    /// `o1` is alone on its MAC in run 1, so `decide_singleton` gives it evidence `[o1]`. Run 2 adds
    /// `o2` on the same MAC, `o1`'s witness becomes `o2`, and `decide_pair` gives evidence
    /// **`[o1, o2]` — SORTED ascending by `ObsId`** (`l1.rs:277-278`, deliberate, with its own
    /// committed test). Writing that literal witness-first reds.
    ///
    /// The old version closes at the NEW version's `valid_from`, which is the same observation's
    /// `observed_at` — so `valid_to == valid_from`, which `0002` refused with `ERROR 4025` and
    /// `0004` admits. Mutation M4 (close at `+1 µs`) and M5 (revert `0004`) both red this.
    #[tokio::test]
    async fn a_changed_witness_supersedes_and_the_old_version_survives() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let o1 = observation(1, l2(1), &[mac(0x01)], 1_700_000_000);
        let o2 = observation(2, l2(1), &[mac(0x01)], 1_700_000_100);
        let Some(pool) = fixture(&[o1.clone(), o2.clone()]).await else {
            return;
        };

        let first = pass(&pool, vec![o1.clone()]).await;
        assert_eq!(first.links_written, 1);
        let alone = versions(&pool, o1.obs_id).await;
        assert_eq!(alone.len(), 1);
        assert_eq!(
            alone[0].3.as_deref(),
            Some(serde_json::to_string(&vec![o1.obs_id]).unwrap().as_str()),
            "alone on its key, the singleton is its own evidence"
        );

        let second = pass(&pool, vec![o1.clone(), o2.clone()]).await;
        assert_eq!(second.links_superseded, 1, "o1's justification changed");
        assert_eq!(second.links_written, 2, "o1's new version, and o2's first");
        assert_eq!(second.links_unchanged, 0);

        let after = versions(&pool, o1.obs_id).await;
        assert_eq!(after.len(), 2, "the old version is UNLINKED, never erased");

        let (old_id, old_from, old_to, old_evidence) = after[0].clone();
        assert_eq!(
            old_id, alone[0].0,
            "the superseded row is the SAME row, restamped"
        );
        assert_eq!(
            old_evidence.as_deref(),
            Some(serde_json::to_string(&vec![o1.obs_id]).unwrap().as_str()),
            "history keeps the justification it was written with"
        );
        assert_eq!(
            old_to, old_from,
            "🔴 zero-length: both versions carry the same observation's observed_at, so the close \
             instant IS the open instant. 0002 refused this with ERROR 4025."
        );
        assert_eq!(old_from, datetime_literal(at(1_700_000_000)));

        let (_, new_from, new_to, new_evidence) = after[1].clone();
        assert_eq!(new_to, OPEN_END, "the new version is the current one");
        assert_eq!(new_from, old_from, "same observation, same valid_from");
        assert_eq!(
            new_evidence.as_deref(),
            Some(
                serde_json::to_string(&vec![o1.obs_id, o2.obs_id])
                    .unwrap()
                    .as_str()
            ),
            "sorted ascending by ObsId — NEVER witness-first (l1.rs:277-278)"
        );

        assert_eq!(
            current_links(&pool, o1.obs_id).await.len(),
            1,
            "exactly one current link, which identity_link_one_current also enforces"
        );
    }

    /// 🔴 AC4 — the engine never supersedes an OPERATOR's row.
    ///
    /// The compare-then-supersede read filters on `decided_by = 'ENGINE'`, so an operator's row is
    /// invisible to it: the pass falls through to its insert and `identity_link_one_current` refuses
    /// it, exactly as it did before this story existed. The operator's row comes out untouched —
    /// same `id`, same `valid_to`, still current.
    ///
    /// This PINS today's behaviour rather than changing it. *"May an operator override the engine?"*
    /// is story 5.14's question, and answering it by accident here is what mutation M1 measures.
    #[tokio::test]
    async fn the_engine_never_supersedes_an_operators_link() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let o = mac_less(1, l2(1), 1_700_000_000);
        let Some(pool) = fixture(std::slice::from_ref(&o)).await else {
            return;
        };

        // An operator asserts the abstention slot the pass would want.
        let operator_link = LinkId::from_uuid(uuid::Uuid::now_v7());
        MariaRepository::new(pool.clone())
            .transact(|unit| {
                Box::pin(async move {
                    crate::repo::insert_identity_link(
                        unit.executor(),
                        operator_link,
                        ObsId::from_uuid(uuid::Uuid::from_u128(1)),
                        None,
                        &nothing_was_evaluated(),
                        &[],
                        DecidedBy::Operator,
                        at(1_700_000_000),
                        open_end(),
                    )
                    .await
                    .map_err(classify)
                })
            })
            .await
            .expect("the operator writes");

        let refused = try_pass(&pool, vec![o.clone()]).await;
        assert!(
            matches!(refused, Err(RepositoryError::Constraint("unique"))),
            "the engine does not take a slot a human holds; got {refused:?}"
        );

        let rows = versions(&pool, o.obs_id).await;
        assert_eq!(rows.len(), 1, "the pass rolled back entirely");
        assert_eq!(rows[0].0, operator_link.to_string(), "same id — untouched");
        assert_eq!(rows[0].2, OPEN_END, "still current, never restamped");
        let stored: Vec<(String,)> =
            sqlx::query_as("SELECT decided_by FROM identity_link WHERE id = ?")
                .bind(operator_link.to_string())
                .fetch_all(&pool)
                .await
                .expect("read decided_by");
        assert_eq!(stored[0].0, "OPERATOR");
    }

    /// 🔴 AC5 — a purge-and-replay after a supersede LOSES history, and the snapshots still match.
    ///
    /// Guy's arbitration: the purge is an assumed reset. A link is *"a cache of attention, not of
    /// truth"*, so what the engine believed yesterday is not a truth to preserve — the replay
    /// rebuilds the CURRENT state and owes history nothing. `purge_engine_links` has no
    /// `current_subject` filter and takes superseded rows too; `snapshot_links` never compared them.
    ///
    /// Both numbers are measured, because the equal snapshots alone would hide the loss — which is
    /// exactly why story 5.10's comparison could not see this.
    #[tokio::test]
    async fn a_purge_after_a_supersede_loses_history_and_still_replays() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let o1 = observation(1, l2(1), &[mac(0x01)], 1_700_000_000);
        let o2 = observation(2, l2(1), &[mac(0x01)], 1_700_000_100);
        let Some(pool) = fixture(&[o1.clone(), o2.clone()]).await else {
            return;
        };

        pass(&pool, vec![o1.clone()]).await;
        let grown = pass(&pool, vec![o1.clone(), o2.clone()]).await;
        assert_eq!(grown.links_superseded, 1);

        let rows_before = count_identity_links(&pool).await.expect("count");
        assert_eq!(rows_before, 3, "2 current + 1 superseded");
        let before = crate::repo::snapshot_links(&pool)
            .await
            .map_err(classify)
            .expect("snapshot before");
        assert_eq!(
            before.len(),
            2,
            "the snapshot only ever saw the current two"
        );

        let purged = MariaRepository::new(pool.clone())
            .transact(|unit| {
                Box::pin(async move {
                    crate::repo::purge_engine_links(unit.executor())
                        .await
                        .map_err(classify)
                })
            })
            .await
            .expect("purge");
        assert_eq!(
            purged, 3,
            "the purge takes HISTORY as well as the current rows"
        );

        pass(&pool, vec![o1, o2]).await;

        assert_eq!(
            crate::repo::snapshot_links(&pool)
                .await
                .map_err(classify)
                .expect("snapshot after"),
            before,
            "every decision-bearing column of the CURRENT state comes back"
        );
        let rows_after = count_identity_links(&pool).await.expect("count");
        assert_eq!(
            rows_after, 2,
            "and the store is SMALLER than before the purge: the superseded version is gone, \
             which the equal snapshots above cannot see"
        );
        assert!(rows_after < rows_before);
    }

    /// AC6 — the write amplification, measured at both ends.
    ///
    /// The witness is the SMALLEST OTHER `ObsId`, so *"add one observation"* has two answers: a
    /// newcomer with a LARGER id supersedes nothing, and one with the smallest id supersedes every
    /// other member of its group. Both are recorded rather than one being passed off as the figure.
    #[tokio::test]
    async fn the_write_amplification_is_measured_at_both_ends() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        // A group of six sharing one MAC, minted with ids 2..=7 so id 1 is free below.
        let group: Vec<Observation> = (2..=7u128)
            .map(|i| observation(i, l2(1), &[mac(0x01)], 1_700_000_000 + i as i64))
            .collect();
        let newcomer_smallest = observation(1, l2(1), &[mac(0x01)], 1_700_000_500);
        let newcomer_largest = observation(99, l2(1), &[mac(0x01)], 1_700_000_600);
        let mut all = group.clone();
        all.push(newcomer_smallest.clone());
        all.push(newcomer_largest.clone());
        let Some(pool) = fixture(&all).await else {
            return;
        };

        let cold = pass(&pool, group.clone()).await;
        assert_eq!(cold.links_written, 6);
        assert_eq!(cold.links_superseded, 0);

        // A LARGER id joins: it is nobody's witness, so nothing is superseded.
        let mut with_largest = group.clone();
        with_largest.push(newcomer_largest.clone());
        let larger = pass(&pool, with_largest.clone()).await;
        assert_eq!(
            larger.links_superseded, 0,
            "a larger id is nobody's smallest-other witness"
        );
        assert_eq!(larger.links_written, 1, "only the newcomer's own link");
        assert_eq!(larger.links_unchanged, 6);

        // The SMALLEST id joins: it becomes every other member's witness.
        let mut with_smallest = with_largest.clone();
        with_smallest.push(newcomer_smallest.clone());
        let smallest = pass(&pool, with_smallest).await;
        assert_eq!(
            smallest.links_superseded, 7,
            "O(group size): every incumbent's evidence changed"
        );
        assert_eq!(smallest.links_written, 8, "7 new versions + the newcomer's");
        assert_eq!(smallest.links_unchanged, 0);

        // 🔴 The counts above are `Resolution` fields, and this module's own doc calls a test that
        // asserts them ALONE "an oracle restating the pass's own summary". Measured at the code
        // review: every assertion in this test read a field of the value the code under test
        // returned, and the post-state was asserted nowhere. These `SELECT` it.
        let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM identity_link")
            .fetch_one(&pool)
            .await
            .expect("count every version");
        let (current,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM identity_link WHERE current_subject IS NOT NULL")
                .fetch_one(&pool)
                .await
                .expect("count the current versions");
        assert_eq!(
            current, 8,
            "eight observations on one MAC, one current link each"
        );
        assert_eq!(
            total, 15,
            "8 current + 7 closed — the supersedes left history behind, read back rather than \
             taken from the summary"
        );
        assert_eq!(
            total - current,
            i64::try_from(smallest.links_superseded).expect("a small count"),
            "and the closed rows are exactly what the summary claimed it superseded"
        );
    }

    /// 🔴 AC4 — the operator's row DIFFERS from what the engine would write, and is still untouched.
    ///
    /// ⚠️ **This is the case the `decided_by = 'ENGINE'` doc described while no test exercised it.**
    /// Measured at the code review: in `the_engine_never_supersedes_an_operators_link` and in
    /// story 5.10's sibling, the operator's row carries `nothing_was_evaluated()` — byte-identical
    /// to what the engine would write — so with the filter removed `same_decision` returns `true`,
    /// the pass reports `Unchanged` and returns `Ok`. Those tests measure the engine ADOPTING a
    /// human's row, which is bad enough; the SUPERSEDE the doc claimed was measured by nothing.
    ///
    /// A differing `ruleset_version` is the cheapest way to reach it: it is one of the six compared
    /// columns and needs no `Ambiguous` producer.
    #[tokio::test]
    async fn the_engine_never_adopts_or_supersedes_a_differing_operator_row() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let o = mac_less(1, l2(1), 1_700_000_000);
        let Some(pool) = fixture(std::slice::from_ref(&o)).await else {
            return;
        };

        // Same conclusion, DIFFERENT ruleset — so `same_decision` would say "changed" and the
        // unfiltered path would close a human's row and append its own.
        let operators_view = decide(
            Vec::new(),
            opencmdb_core::identity::cascade::RulesetVersion(2),
        );
        let operator_link = LinkId::from_uuid(uuid::Uuid::now_v7());
        MariaRepository::new(pool.clone())
            .transact(|unit| {
                Box::pin(async move {
                    crate::repo::insert_identity_link(
                        unit.executor(),
                        operator_link,
                        ObsId::from_uuid(uuid::Uuid::from_u128(1)),
                        None,
                        &operators_view,
                        &[],
                        DecidedBy::Operator,
                        at(1_700_000_000),
                        open_end(),
                    )
                    .await
                    .map_err(classify)
                })
            })
            .await
            .expect("the operator writes");

        let refused = try_pass(&pool, vec![o.clone()]).await;
        assert!(
            matches!(refused, Err(RepositoryError::Constraint("unique"))),
            "the engine neither adopts nor supersedes it; got {refused:?}"
        );

        let rows = versions(&pool, o.obs_id).await;
        assert_eq!(rows.len(), 1, "the pass rolled back whole");
        assert_eq!(rows[0].0, operator_link.to_string(), "same id — untouched");
        assert_eq!(rows[0].2, OPEN_END, "still current, never restamped");
        let stored: Vec<(u32,)> =
            sqlx::query_as("SELECT ruleset_version FROM identity_link WHERE id = ?")
                .bind(operator_link.to_string())
                .fetch_all(&pool)
                .await
                .expect("read the ruleset");
        assert_eq!(
            stored[0].0, 2,
            "🔴 the human's own ruleset, not the engine's — this is what a supersede would have lost"
        );
    }

    /// 🔴 AC10 — a slot the input no longer supports is CLOSED, not left standing.
    ///
    /// The `multi-nic` shape: an observation carrying two MACs, re-supplied carrying one. `join`
    /// produces no group for the vanished key, so `write_link` never visits that slot — and before
    /// the code review nothing else did either, leaving a current link pointing at an interface no
    /// fact in the input supports.
    ///
    /// 🔴 **This story is what made the case silent.** The blind append used to fail LOUDLY on
    /// `identity_link_one_current` with a full rollback; the compare routes around the key, so the
    /// detection has to be explicit. Measured before the fix: `Ok(links_unchanged: 1)` and **two**
    /// current links.
    #[tokio::test]
    async fn a_slot_the_input_no_longer_supports_is_closed() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let two_macs = observation(1, l2(1), &[mac(0x01), mac(0x02)], 1_700_000_000);
        let one_mac = observation(1, l2(1), &[mac(0x01)], 1_700_000_100);
        let Some(pool) = fixture(std::slice::from_ref(&two_macs)).await else {
            return;
        };

        let first = pass(&pool, vec![two_macs]).await;
        assert_eq!(first.links_written, 2, "one link per L1 key");
        assert_eq!(first.links_vacated, 0);
        assert_eq!(current_links(&pool, one_mac.obs_id).await.len(), 2);

        let second = pass(&pool, vec![one_mac.clone()]).await;

        assert_eq!(
            second.links_vacated, 1,
            "the mac02 slot has no successor and is closed"
        );
        assert_eq!(second.links_written, 0, "and nothing new was written");
        assert_eq!(second.links_unchanged, 1, "mac01's slot is untouched");
        assert_eq!(
            current_links(&pool, one_mac.obs_id).await.len(),
            1,
            "🔴 ONE current link, read back — two is the orphan this closes"
        );
        assert_eq!(
            count_identity_links(&pool).await.expect("count"),
            2,
            "the closed version is UNLINKED, never erased"
        );
    }

    /// 🔴 AC10 — and the orphan is what falsified story 5.10's replay invariant.
    ///
    /// A link the input does not support cannot be reproduced by a replay, so before the fix
    /// `snapshot_links` returned **2** rows before the purge and **1** after — a counterexample to
    /// *"the engine's output depends only on the observations and on the interfaces"* reachable
    /// through pure engine input, with no operator row and no doctored `obs_id`. Closing the slot is
    /// what restores it, and this test is the one that would have caught it.
    ///
    /// ⚠️ **Both slices carry the SAME `observed_at`, and that is not cosmetic.** Re-supplying one
    /// `obs_id` with a LATER instant makes the replay irreproducible for a second, independent
    /// reason: an unchanged slot keeps the `valid_from` it was first written with, while a replay
    /// from an empty store writes the instant it is handed now. Measured here — the first draft of
    /// this test moved the instant and reddened on `valid_from`, not on the orphan. That belongs to
    /// the unenforced caller-discipline entry in the register, not to this test.
    #[tokio::test]
    async fn the_replay_invariant_survives_an_observation_that_lost_a_key() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let two_macs = observation(1, l2(1), &[mac(0x01), mac(0x02)], 1_700_000_000);
        let one_mac = observation(1, l2(1), &[mac(0x01)], 1_700_000_000);
        let Some(pool) = fixture(std::slice::from_ref(&two_macs)).await else {
            return;
        };

        pass(&pool, vec![two_macs]).await;
        pass(&pool, vec![one_mac.clone()]).await;
        let before = crate::repo::snapshot_links(&pool)
            .await
            .map_err(classify)
            .expect("snapshot before");
        assert_eq!(
            before.len(),
            1,
            "only the slot the input supports is current"
        );

        MariaRepository::new(pool.clone())
            .transact(|unit| {
                Box::pin(async move {
                    crate::repo::purge_engine_links(unit.executor())
                        .await
                        .map_err(classify)
                })
            })
            .await
            .expect("purge");
        pass(&pool, vec![one_mac]).await;

        assert_eq!(
            crate::repo::snapshot_links(&pool)
                .await
                .map_err(classify)
                .expect("snapshot after"),
            before,
            "the replay reproduces the current state exactly — which it could NOT do while the \
             vanished key's link was still standing"
        );
    }

    /// 🔴 AC11 — an `observed_at` that runs BACKWARDS is refused by name, on both branches.
    ///
    /// Measured at the code review, before the guard existed: with only `identity_link_interval` to
    /// catch it, a regressing instant whose decision ALSO changed inverted the interval and killed
    /// the **whole cycle** — every unrelated observation in the batch rolled back — under an
    /// anonymous `Constraint("check")`; and the same regression with an unchanged decision was
    /// entirely SILENT, because `same_decision` does not compare `valid_from`. One condition, two
    /// opposite answers.
    ///
    /// The guard gives it one answer, above the DDL, with a cause a reader can act on.
    #[tokio::test]
    async fn an_instant_that_runs_backwards_is_refused_by_name() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let o1 = observation(1, l2(1), &[mac(0x01)], 1_700_000_000);
        let o2 = observation(2, l2(1), &[mac(0x01)], 1_700_000_100);
        let regressed = observation(1, l2(1), &[mac(0x01)], 1_600_000_000);
        let Some(pool) = fixture(&[o1.clone(), o2.clone()]).await else {
            return;
        };

        pass(&pool, vec![o1]).await;
        let before = crate::repo::snapshot_links(&pool)
            .await
            .map_err(classify)
            .expect("snapshot before");

        // The decision ALSO changes here — o2 joins the group — so this is the branch that used to
        // reach the DDL and lose the batch.
        let refused = try_pass(&pool, vec![regressed, o2.clone()]).await;
        assert!(
            matches!(refused, Err(RepositoryError::InstantRegressed)),
            "the cause is NAMED, not an anonymous check failure; got {refused:?}"
        );
        assert_eq!(
            crate::repo::snapshot_links(&pool)
                .await
                .map_err(classify)
                .expect("snapshot after"),
            before,
            "and the pass rolled back whole — o2's link was never written either"
        );
    }

    /// 🔴 AC2c — story 5.9b's abstention dedup guard, kept and MEASURED.
    ///
    /// `resolve_within`'s tail loop refuses to write a second abstention for one `obs_id`, whatever
    /// the number of keys it carries — Guy's arbitration at story 5.9b's code review, taken after a
    /// measured `ABSTAINED_SUBJECT` collision rolled a whole pass back.
    ///
    /// ⚠️ **This story's write path makes that guard invisible to `a_repeated_obs_id_writes_one_link`**,
    /// which was measured at validation: deleting `!abstained.insert(…)` left all 424 tests green,
    /// because the second write now finds the current row and reports it UNCHANGED rather than
    /// colliding. The guard is kept — an observation still abstains at most once — and this test is
    /// what says so, by counting the writes the pass reports rather than the rows that survive it.
    #[tokio::test]
    async fn a_repeated_obs_id_abstains_once_and_the_pass_says_so() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let o = mac_less(1, l2(1), 1_700_000_000);
        let Some(pool) = fixture(std::slice::from_ref(&o)).await else {
            return;
        };

        let summary = pass(&pool, vec![o.clone(), o.clone(), o.clone()]).await;

        assert_eq!(
            summary.links_written, 1,
            "one abstention WRITTEN, not three — the dedup guard, not the compare"
        );
        assert_eq!(summary.abstentions, 1);
        assert_eq!(
            summary.links_unchanged, 0,
            "🔴 the load-bearing line: without the dedup the 2nd and 3rd copies would reach the \
             compare and report `unchanged`, so counting ROWS cannot tell the two apart"
        );
        assert_eq!(count_identity_links(&pool).await.expect("count"), 1);
    }

    // ------------------------------------------------------------------------------------------
    // Story 5.11b — the arrival order of a scan cannot change what the product believes (NFR6).
    //
    // 🔴 Everything below measures a property that is already true by CONSTRUCTION. Nothing here is
    // expected to red, which is exactly the condition under which a test that measures nothing
    // looks like a success. Every one of them therefore asserts HOW MANY permutations it consumed:
    // measured at validation, a degenerate enumerator reddens ONLY the count assertions, and
    // deleting those lines leaves the behavioural half green.
    // ------------------------------------------------------------------------------------------

    /// The synthetic slice the order measurements run over — every shape the corpus CANNOT produce.
    ///
    /// - observations 1, 2, 3 share one MAC → a group of THREE, so the witness convention is
    ///   exercised where *"smallest other"* and *"first by arrival"* actually differ. On a group of
    ///   two the two formulas coincide and the test would be blind;
    /// - observation 4 carries **two MACs** → the multi-key shape. §6 measured that this is the
    ///   only shape under which a first-key-wins `join` is observable at all: where every
    ///   observation carries exactly one key, that mutation is a no-op and the corpus test stays
    ///   green under it;
    /// - observation 5 carries **no MAC** → an abstention sharing a slice with placements;
    /// - observation 6 carries observation 1's MAC in a **second `l2_domain`** → two scopes, which
    ///   no committed stream has.
    ///
    /// Six observations, so the enumeration is `6! = 720` — measured at ~11.5 ms.
    fn order_fixture() -> Vec<Observation> {
        vec![
            observation(1, l2(1), &[mac(0x01)], 1_700_000_000),
            observation(2, l2(1), &[mac(0x01)], 1_700_000_100),
            observation(3, l2(1), &[mac(0x01)], 1_700_000_200),
            observation(4, l2(1), &[mac(0x02), mac(0x03)], 1_700_000_300),
            mac_less(5, l2(1), 1_700_000_400),
            observation(6, l2(2), &[mac(0x01)], 1_700_000_500),
        ]
    }

    /// 🔴 AC1 (shape A) — the derived interfaces and pairs are identical under EVERY permutation.
    ///
    /// Pure: it calls `join` and `candidates` and never opens a database, which is what lets it be
    /// exhaustive rather than sampled. It is also the ONLY shape in this story that covers the
    /// derived interface SET — shapes B and C both start from a store an in-order pass already
    /// built, so the interfaces exist before they run.
    ///
    /// ⚠️ Because it never enters `resolve_within`, no mutation of the RESOLVER can red it — the
    /// mutation that measures it edits `identity/l1.rs`, which the story's AC7 permits for a
    /// mutation and forbids in the shipped diff. Measured at validation: the originally prescribed
    /// resolver-side mutation left all tests green, and could not have done otherwise.
    #[test]
    fn the_derived_interfaces_and_pairs_are_identical_under_every_permutation() {
        let observations = order_fixture();
        let expected_groups = join(&observations);
        let expected_pairs = candidates(&observations);

        // Independent oracles: the loop below asserts nothing worth having if the slice derives
        // nothing. These two counts are known from the fixture, not read back from the code.
        assert_eq!(
            expected_groups.len(),
            4,
            "mac 01 in l2(1), mac 02, mac 03, and mac 01 again in l2(2)"
        );
        assert_eq!(
            expected_pairs.len(),
            15,
            "6 * 5 / 2 unordered pairs of distinct observation ids"
        );

        let mut consumed = 0usize;
        for (index, permuted) in permutations(&observations).enumerate() {
            assert_eq!(
                join(&permuted),
                expected_groups,
                "permutation {index} changed the derived interfaces"
            );
            assert_eq!(
                candidates(&permuted),
                expected_pairs,
                "permutation {index} changed the proposed pairs"
            );
            consumed += 1;
        }
        assert_eq!(
            consumed, 720,
            "🔴 6! permutations. A degenerate enumerator is caught HERE and by nothing else, \
             because the property above is true by construction and stays green under one"
        );
    }

    /// How many permutations the two DATABASE shapes each run.
    ///
    /// Stated rather than implicit: a permutation sweep multiplies round-trips, so shapes B and C
    /// sample where shape A enumerates. Twelve passes of a seven-link slice is a few hundred
    /// statements, which is affordable; 720 would not be.
    const DB_PERMUTATION_SAMPLE: usize = 12;

    /// A deterministic spread of permutations, **never the identity**.
    ///
    /// 🔴 The `skip(1)` is load-bearing and not hygiene: `permutations` yields lexicographic order,
    /// so element 0 IS the input. Sampling it would turn both database shapes into a comparison of
    /// a run with itself — green under any order-dependence whatsoever. The two assertions below
    /// are what stop a future edit from quietly reintroducing it.
    fn sampled_permutations(observations: &[Observation]) -> Vec<(usize, Vec<Observation>)> {
        let sample: Vec<(usize, Vec<Observation>)> = permutations(observations)
            .enumerate()
            .skip(1)
            .step_by(60)
            .take(DB_PERMUTATION_SAMPLE)
            .collect();
        assert_eq!(
            sample.len(),
            DB_PERMUTATION_SAMPLE,
            "the sample was truncated — the enumerator yielded fewer permutations than it must"
        );
        assert!(
            sample.iter().all(|(_, p)| p.as_slice() != observations),
            "the identity is not a fuzzed order"
        );
        sample
    }

    /// 🔴 AC2 (shape C) — a fuzzed order run into a POPULATED store writes nothing at all.
    ///
    /// The strongest statement in story 5.11b, and it exists only because story 5.11 shipped
    /// idempotence. Any order-dependence anywhere in the pass — in the grouping, in the witness, in
    /// the seen-window, in the tail abstention loop — must surface as a write, a supersede or a
    /// vacate, because those are the only ways this pass can change anything. It needs no snapshot
    /// machinery and no column-by-column comparison to say so.
    ///
    /// ⚠️ It is NOT a duplicate of story 5.11's idempotence test, which re-runs the SAME order.
    /// Measured at validation under M2 (a witness that follows arrival): 5.11's test stayed green
    /// while this one reddened.
    #[tokio::test]
    async fn a_fuzzed_order_into_a_populated_store_writes_nothing() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = order_fixture();
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        let first = pass(&pool, observations.clone()).await;
        assert_eq!(
            first.links_written, 7,
            "3 in the group + 2 for the multi-MAC + 1 abstention + 1 singleton"
        );
        assert_eq!(first.interfaces_minted, 4);
        let ids_before = all_link_ids(&pool).await;
        assert_eq!(ids_before.len(), 7);

        let mut consumed = 0usize;
        for (index, permuted) in sampled_permutations(&observations) {
            let again = pass(&pool, permuted).await;
            assert_eq!(again.links_written, 0, "permutation {index} wrote a link");
            assert_eq!(
                again.links_superseded, 0,
                "permutation {index} superseded a version"
            );
            assert_eq!(again.links_vacated, 0, "permutation {index} vacated a slot");
            assert_eq!(
                again.links_unchanged, 7,
                "permutation {index} failed to recognise a slot it had already filled"
            );
            assert_eq!(
                again.interfaces_minted, 0,
                "permutation {index} minted an interface that already existed"
            );
            consumed += 1;
        }
        assert_eq!(
            consumed, DB_PERMUTATION_SAMPLE,
            "a degenerate enumerator is caught here, not by the no-op assertions above"
        );

        // Read back rather than trusting the summary: a supersede mints a NEW id, so an unchanged
        // id set is the database's own account of "nothing was rewritten".
        assert_eq!(
            all_link_ids(&pool).await,
            ids_before,
            "every link row survived with the id it was first written with"
        );
    }

    /// 🔴 AC3 (shape B) — a purge-and-replay in a fuzzed order reproduces every decision-bearing
    /// column.
    ///
    /// Story 5.10's apparatus with a PERMUTED input, and it needs no new adapter code:
    /// `snapshot_links` is reused unchanged, which is the reason this shape was chosen over a
    /// hand-rolled comparison.
    ///
    /// 🔑 **`interface_id` is literally comparable here**, which shape B alone among the three can
    /// claim. The purge removes the engine's LINKS and leaves the INTERFACES standing, so the
    /// replay FINDS its interfaces by key instead of minting new ids — `interfaces_minted == 0`
    /// below is what says so, and without it the comparison would be measuring a coincidence. This
    /// is D14's statement exactly: the engine's output depends only on the observations and on the
    /// interfaces.
    #[tokio::test]
    async fn a_purge_and_replay_in_a_fuzzed_order_reproduces_every_column() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations = order_fixture();
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        let first = pass(&pool, observations.clone()).await;
        assert_eq!(first.links_written, 7);
        assert_eq!(first.interfaces_minted, 4);
        let before = crate::repo::snapshot_links(&pool)
            .await
            .map_err(classify)
            .expect("snapshot before");
        let interfaces_before = interface_ids(&pool).await;
        assert_eq!(before.len(), 7, "seven current links to reproduce");
        assert_eq!(interfaces_before.len(), 4);

        let mut consumed = 0usize;
        for (index, permuted) in sampled_permutations(&observations) {
            MariaRepository::new(pool.clone())
                .transact(|unit| {
                    Box::pin(async move {
                        crate::repo::purge_engine_links(unit.executor())
                            .await
                            .map_err(classify)
                    })
                })
                .await
                .expect("purge");

            let replay = pass(&pool, permuted).await;
            assert_eq!(
                replay.interfaces_minted, 0,
                "permutation {index} re-minted an interface — `interface_id` would then compare \
                 equal only by accident"
            );
            assert_eq!(
                replay.links_written, 7,
                "permutation {index} rebuilt a different number of links"
            );
            assert_eq!(
                crate::repo::snapshot_links(&pool)
                    .await
                    .map_err(classify)
                    .expect("snapshot after"),
                before,
                "permutation {index} reproduced a different decision"
            );
            consumed += 1;
        }
        assert_eq!(
            consumed, DB_PERMUTATION_SAMPLE,
            "a degenerate enumerator is caught here, not by the comparison above"
        );
        assert_eq!(
            interface_ids(&pool).await,
            interfaces_before,
            "no permutation minted an interface across the whole sweep"
        );
    }

    /// The committed stream this story measures over.
    ///
    /// `hostname-absence.jsonl` carries **six** observations — the largest committed stream — so
    /// its enumeration is `6! = 720`, the same size shape A runs synthetically. The two streams
    /// that could NOT be used are named in the test's own doc rather than left to be rediscovered.
    const CORPUS_STREAM: &str = "scenario/replay/hostname-absence.jsonl";

    /// The corpus's connector identity, **restated** rather than imported.
    ///
    /// `fixture_connector`'s equivalents are private to its own test module. Restating them here is
    /// the deliberate-redundancy idiom this module already uses for `CORPUS_EXACT_MAC`, and it
    /// buys something: if the committed streams were re-issued under another connector id, the
    /// load below fails `ForeignConnectorId` and this test reds LOUDLY, where a shared helper would
    /// have followed the change in silence.
    fn corpus_connector() -> ConnectorId {
        ConnectorId::from_uuid(
            uuid::Uuid::parse_str("33333333-3333-4333-8333-333333333333").expect("a valid uuid"),
        )
    }

    /// The single scope every usable committed stream declares. See [`corpus_connector`].
    fn corpus_scope() -> Scope {
        Scope {
            l2_domain: L2DomainId::from_uuid(
                uuid::Uuid::parse_str("11111111-1111-4111-8111-111111111111")
                    .expect("a valid uuid"),
            ),
            vantage: VantageId::from_uuid(
                uuid::Uuid::parse_str("22222222-2222-4222-8222-222222222222")
                    .expect("a valid uuid"),
            ),
        }
    }

    /// A descriptor wide enough to admit any committed stream — deliberately WIDER than what the
    /// stream emits, because *capable and unseen* must stay legal (D34 §1).
    fn corpus_capabilities() -> opencmdb_core::observation::Capabilities {
        use opencmdb_core::observation::{Capabilities, FactKind};
        Capabilities {
            as_of: at(1_700_000_000),
            kinds: BTreeSet::from([
                FactKind::Mac,
                FactKind::IpV4,
                FactKind::Hostname,
                FactKind::OuiVendor,
                FactKind::Rtt,
                FactKind::Uplink,
                FactKind::DhcpLease,
            ]),
        }
    }

    /// Replay [`CORPUS_STREAM`] through the real connector and collect what it emits.
    async fn corpus_observations() -> Vec<Observation> {
        use opencmdb_core::connector::{Connector, VecSink};

        let mut connector = crate::fixture_connector::FixtureConnector::load(
            corpus_connector(),
            corpus_capabilities(),
            vec![corpus_scope()],
            CORPUS_STREAM,
        )
        .expect("the committed stream loads with the corpus context");
        let mut sink = VecSink::default();
        connector
            .poll(
                at(1_700_000_000),
                &mut sink,
                tokio_util::sync::CancellationToken::new(),
            )
            .await
            .expect("this stream ends cleanly — see the doc on the caller");
        sink.observations
    }

    /// AC6 — the committed corpus is used, and its limits are stated rather than implied.
    ///
    /// # What the corpus can and cannot do here
    ///
    /// It satisfies AC1's letter and reaches none of the interesting shapes. Measured and already
    /// recorded in this module's doc: **no committed observation carries more than one MAC**, and
    /// every stream carries a single `l2_domain`. So this test is reddened by nothing in the
    /// story's permitted mutation set — under a `join` mutated to first-key-wins it stays GREEN,
    /// because first-key-wins is a no-op where every observation has exactly one key. The synthetic
    /// slice of [`order_fixture`] is what carries that measurement; this one proves the property
    /// holds on bytes the project committed rather than on bytes a test invented.
    ///
    /// # Why THIS stream
    ///
    /// Two of the thirteen cannot be used and it is worth naming why, since both look usable:
    /// `capability-downgrade.jsonl` and `partial-then-failed.jsonl` carry their OWN `connector_id`
    /// and `scope`, so loading them with the corpus context is refused `ForeignConnectorId` then
    /// `UncoveredScope`; and `partial-then-failed.jsonl` additionally ends in a `Failure` record,
    /// so `poll` returns `Err(ConnectorError::Unreachable)` with four observations already in the
    /// sink and the obvious `.expect("poll")` PANICS. Both are by design.
    #[tokio::test]
    async fn a_committed_stream_derives_the_same_interfaces_in_every_order() {
        let observations = corpus_observations().await;
        assert_eq!(
            observations.len(),
            6,
            "the largest committed stream, chosen so the enumeration is a full 6!"
        );

        let expected_groups = join(&observations);
        let expected_pairs = candidates(&observations);
        assert!(
            !expected_groups.is_empty(),
            "a stream that derives no interface would make the loop below vacuous"
        );
        assert_eq!(expected_pairs.len(), 15, "6 * 5 / 2 unordered pairs");
        // The stated limit, ASSERTED rather than claimed — and computed here from the facts rather
        // than read out of `keys_of`, which is private to `identity/` and which AC7 forbids this
        // story to touch. Counting DISTINCT addresses is the right measure: one MAC repeated twice
        // is still one key, `keys_of` returning a set.
        for observation in &observations {
            let macs: BTreeSet<MacAddr> = observation
                .facts
                .iter()
                .filter_map(|fact| match fact {
                    Fact::Mac { addr, .. } => Some(*addr),
                    _ => None,
                })
                .collect();
            assert!(
                macs.len() <= 1,
                "no committed observation carries more than one MAC — that is exactly why a \
                 first-key-wins `join` is invisible here and the synthetic slice is required"
            );
        }

        let mut consumed = 0usize;
        for (index, permuted) in permutations(&observations).enumerate() {
            assert_eq!(
                join(&permuted),
                expected_groups,
                "permutation {index} of the committed stream changed the derived interfaces"
            );
            assert_eq!(
                candidates(&permuted),
                expected_pairs,
                "permutation {index} of the committed stream changed the proposed pairs"
            );
            consumed += 1;
        }
        assert_eq!(consumed, 720, "6! permutations of the committed stream");
    }

    /// 🔴 AC5 — a repeated `obs_id` whose DECISION-BEARING content differs is refused BY NAME.
    ///
    /// The census below is exhaustive on purpose: `contradicts` destructures all six fields of
    /// `Observation`, and this list is the reader's copy of that destructuring. Four of the five
    /// variants would previously have been resolved silently in favour of whichever copy arrived
    /// last — and *which* copy that is depends on arrival order, which is the whole subject.
    #[tokio::test]
    async fn a_repeated_obs_id_with_differing_content_is_refused() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let base = observation(1, l2(1), &[mac(0x01)], 1_700_000_000);
        let Some(pool) = fixture(std::slice::from_ref(&base)).await else {
            return;
        };

        let mut other_connector = base.clone();
        other_connector.connector_id = ConnectorId::from_uuid(uuid::Uuid::from_u128(9));
        let mut other_instant = base.clone();
        other_instant.observed_at = at(1_700_000_999);
        let mut other_domain = base.clone();
        other_domain.scope.l2_domain = l2(2);
        let mut other_vantage = base.clone();
        other_vantage.scope.vantage = VantageId::from_uuid(uuid::Uuid::from_u128(9));
        let mut other_facts = base.clone();
        other_facts.facts = vec![Fact::Mac {
            addr: mac(0x02),
            locally_administered: false,
        }];

        for (field, variant) in [
            ("connector_id", other_connector),
            ("observed_at", other_instant),
            ("scope.l2_domain", other_domain),
            ("scope.vantage", other_vantage),
            ("facts", other_facts),
        ] {
            let refused = try_pass(&pool, vec![base.clone(), variant]).await;
            assert!(
                matches!(refused, Err(RepositoryError::ContradictoryObservation)),
                "a slice contradicting itself on {field} must be refused by NAME, not resolved \
                 in favour of the last copy; got {refused:?}"
            );
            assert_eq!(
                count_identity_links(&pool).await.expect("count"),
                0,
                "{field}: the refusal happens before anything is written"
            );
        }
    }

    /// 🔑 AC5 — `raw` is EXCLUDED from the comparison, through the whole pass.
    ///
    /// D19: `raw` is opaque provenance that no decision ever reads, so two copies differing only
    /// there contradict nothing and refusing them would red a case where nothing was at stake.
    /// Guy's arbitration.
    #[tokio::test]
    async fn a_repeated_obs_id_differing_only_in_raw_is_accepted() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let base = observation(1, l2(1), &[mac(0x01)], 1_700_000_000);
        let Some(pool) = fixture(std::slice::from_ref(&base)).await else {
            return;
        };

        let mut with_raw = base.clone();
        with_raw.raw = Some(r#"{"provenance":"a second poll"}"#.to_string());

        let summary = pass(&pool, vec![base, with_raw]).await;
        assert_eq!(
            summary.links_written, 1,
            "one observation, one link — `raw` reaches no decision"
        );
        assert_eq!(count_identity_links(&pool).await.expect("count"), 1);
    }

    /// 🔴 The two exclusions, measured against the one-line comparison they replace.
    ///
    /// Each case asserts BOTH that `contradicts` accepts it and that `a != b` would have refused
    /// it. The second half is what makes the explicit field-by-field comparison load-bearing rather
    /// than decorative: without it, nothing in the suite would notice `contradicts` being replaced
    /// by a bare `!=`.
    #[test]
    fn the_contradiction_test_excludes_what_no_decision_reads() {
        let base = observation(1, l2(1), &[mac(0x01), mac(0x02)], 1_700_000_000);

        let mut reordered = base.clone();
        reordered.facts.reverse();
        assert!(
            !contradicts(&base, &reordered),
            "nothing reads the ORDER of `facts` — `keys_of` collects them into a set"
        );
        assert_ne!(
            base, reordered,
            "🔴 and a bare `a != b` WOULD have refused it"
        );

        let mut with_raw = base.clone();
        with_raw.raw = Some("provenance".to_string());
        assert!(
            !contradicts(&base, &with_raw),
            "D19 — no decision reads `raw`"
        );
        assert_ne!(
            base, with_raw,
            "🔴 and a bare `a != b` WOULD have refused it"
        );
    }

    /// 🔴 AC4 — the REFERENCE-SCALE slice is fuzzed from a FIXED seed sweep, never the clock.
    ///
    /// This is where the seeded shuffle earns its place and the only place it has a consumer: the
    /// shapes above enumerate exhaustively because `n!` is at most 720, and `300!` is not a number
    /// anything enumerates. The seed is printed with every failure, so a red reproduces from the
    /// message alone — which is the entire difference between a fuzz test and an anecdote.
    ///
    /// Shape C at reference scale: each fuzzed order runs into the already-populated store and must
    /// write nothing at all.
    ///
    /// ⚠️ 300 observations over **100** MACs, so the groups are threes. The existing reference-scale
    /// test gives every observation its own MAC, which makes every group a singleton — a shape where
    /// the witness convention is never exercised because there is no other id to choose.
    #[tokio::test]
    async fn the_reference_scale_pass_is_order_independent_across_the_seed_sweep() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let observations: Vec<Observation> = (0..300u128)
            .map(|i| {
                let mac_index = i % 100;
                let mut o = observation(i + 1, l2(1), &[], 1_700_000_000 + i as i64);
                o.facts = vec![Fact::Mac {
                    addr: MacAddr([
                        0x00,
                        0x11,
                        0x22,
                        0x00,
                        (mac_index >> 8) as u8,
                        mac_index as u8,
                    ]),
                    locally_administered: false,
                }];
                o
            })
            .collect();
        let Some(pool) = fixture(&observations).await else {
            return;
        };

        let first = pass(&pool, observations.clone()).await;
        assert_eq!(first.interfaces_minted, 100, "100 distinct MACs");
        assert_eq!(first.links_written, 300);

        let started = std::time::Instant::now();
        let mut seeds = 0usize;
        for seed in SEED_SWEEP {
            let permuted = shuffled(&observations, seed);
            assert_ne!(
                permuted, observations,
                "seed {seed} returned the input — this sweep would then measure nothing"
            );
            let again = pass(&pool, permuted).await;
            assert_eq!(again.links_written, 0, "seed {seed} wrote a link");
            assert_eq!(
                again.links_superseded, 0,
                "seed {seed} superseded a version"
            );
            assert_eq!(again.links_vacated, 0, "seed {seed} vacated a slot");
            assert_eq!(
                again.interfaces_minted, 0,
                "seed {seed} minted an interface"
            );
            assert_eq!(again.links_unchanged, 300, "seed {seed} lost a slot");
            seeds += 1;
        }
        eprintln!(
            "reference scale, fuzzed: n=300, macs=100, seeds={seeds}, elapsed={:?}",
            started.elapsed()
        );
        assert_eq!(
            seeds, 8,
            "the fixed sweep is 0..=7 — a shortened or clock-derived sweep is caught here"
        );
    }

    /// The other half: every field a decision DOES read is caught by `contradicts`.
    ///
    /// A pure companion to the database test above — it is what stays measurable if the refusal
    /// ever moves out of `resolve_within`.
    #[test]
    fn the_contradiction_test_catches_every_field_a_decision_reads() {
        let base = observation(1, l2(1), &[mac(0x01)], 1_700_000_000);
        assert!(
            !contradicts(&base, &base.clone()),
            "a clone contradicts nothing"
        );

        let mut other_connector = base.clone();
        other_connector.connector_id = ConnectorId::from_uuid(uuid::Uuid::from_u128(9));
        assert!(contradicts(&base, &other_connector), "connector_id");

        let mut other_instant = base.clone();
        other_instant.observed_at = at(1_700_000_999);
        assert!(contradicts(&base, &other_instant), "observed_at");

        let mut other_domain = base.clone();
        other_domain.scope.l2_domain = l2(2);
        assert!(contradicts(&base, &other_domain), "scope.l2_domain");

        let mut other_vantage = base.clone();
        other_vantage.scope.vantage = VantageId::from_uuid(uuid::Uuid::from_u128(9));
        assert!(contradicts(&base, &other_vantage), "scope.vantage");

        let mut fewer_facts = base.clone();
        fewer_facts.facts.clear();
        assert!(contradicts(&base, &fewer_facts), "facts, by length");

        let mut other_facts = base.clone();
        other_facts.facts = vec![Fact::Mac {
            addr: mac(0x02),
            locally_administered: false,
        }];
        assert!(contradicts(&base, &other_facts), "facts, by content");
    }
}
