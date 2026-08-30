//! Fault injection over a replay stream, and the oracle that compares two runs (story 5.13).
//!
//! NFR8(a), D35(a): **for any injected fault, `device_facts(faulted) ⊆ device_facts(clean)` on the
//! same fixture — a fault may only REMOVE knowledge, never ADD an assertion.** D35 reduces the whole
//! of NFR8 to it: *"if I had to reduce it all to one assertion: the faulted run cannot invent a
//! single fact. Everything else is observability."*
//!
//! # The faulted run is DERIVED IN MEMORY, and that is the only place it can live
//!
//! `epics.md`'s story 4.5a demands a fixture that *"places observations AFTER the failure record, or
//! the assertion cannot fail"* — and, two lines below, that *"nothing may follow a terminal failure
//! record"*. Both are true, of different things: `read_records` enforces the second on the FILE path,
//! and [`crate::fixture_connector::FixtureConnector::from_records`] deliberately declines to, in its
//! own words — *"a caller needs to build exactly that shape to prove a faulted replay emits a strict
//! PREFIX of the clean one (D35(a)). Enforcing it here would forbid the test that proves the story's
//! own criterion."* This module is that caller.
//!
//! So **nothing here reads or writes `fixtures/`**: a mutilation takes the records a committed
//! stream already yielded and returns new ones.
//!
//! ⚠️ **The two mutilations differ in how literally they keep "the same fixture".** [`cut_at`] adds
//! one control line and changes nothing else, so the clean and faulted runs really are the same
//! records — that is what tail-keeping buys, and it is why its strictness measures the CONNECTOR
//! rather than arithmetic. [`blind_after`] adds a control line **and rewrites every following
//! observation**, because a descriptor denying a kind an observation still carries makes the stream
//! refuse to load. So under M-B strictness is guaranteed before the connector is invoked, and what
//! M-B really measures is the INCLUSION half. Stated here because an earlier version of this
//! paragraph claimed "the same records" for both.
//!
//! # 🔴 `⊆` is satisfied by `=`, so the property alone measures NOTHING
//!
//! The inclusion is green for a connector that ignores every fault, for a mutilation that removes
//! nothing, and for a test whose faulted run *is* the clean run. Every caller therefore asserts a
//! PAIR, in two assertions with two messages, because the two failures mean opposite things:
//!
//! * inclusion fails → the run INVENTED something. A **product** defect;
//! * strictness fails → the fault did not BITE. A **test** defect.
//!
//! # 🔴 The oracle is a MULTISET, and that is a decision rather than a workaround
//!
//! [`Fact`] derives neither `Ord` nor `Hash`, so the obvious `BTreeSet<(ObsId, Fact)>` does not
//! compile and the obvious repair is two derives in `opencmdb-core`. **Refused, because a SET is the
//! wrong structure**: it collapses `[x, x]` onto `[x]`, so a run that emitted a fact twice and a run
//! that emitted it once would compare equal. Story 5.11b measured exactly that hole — the
//! `facts.len()` term in `resolver`'s `contradicts` *"was droppable with the suite green, being the
//! only term that catches `[x]` against `[x, x]`"*. [`multiset_included`] needs `PartialEq` alone,
//! `n` is at most a few dozen facts, and **`opencmdb-core` is not touched**.
//!
//! Wired into no runtime path — fault injection is not `/healthz` — hence the `dead_code` allow, for
//! the same reason `fixtures.rs`, `l1_runner.rs` and `trap_gate.rs` carry it.
#![allow(dead_code)]

use std::collections::{BTreeSet, HashSet};

use opencmdb_core::connector::{Connector, ConnectorError, VecSink};
use opencmdb_core::observation::{
    Capabilities, ConnectorId, Fact, FactKind, ObsId, Observation, Scope, Timestamp,
};
use tokio_util::sync::CancellationToken;

use crate::fixture_connector::FixtureConnector;
use crate::fixtures::Record;

/// The error a cut injects. `Unreachable` and not another variant because it is the one D35's
/// layer-A list opens with (*"401, timeout, partial"*) and the one a mid-sweep loss really presents
/// as; the detail names the injection so a failure message is not mistaken for a committed one.
const CUT_DETAIL: &str = "fault injected by story 5.13: the poll stopped here";

/// **M-A — the poll fails after `k` records, and the tail is KEPT.**
///
/// Inserts a terminal [`Record::Failure`] at index `k` and leaves every following record in place.
/// `poll` returns `Err` when it reaches the failure, so the faulted run emits a strict PREFIX of
/// what the clean one emitted.
///
/// # 🔑 Keeping the tail is the whole point
///
/// Truncating instead would make the two runs differ in their INPUT, and the claim would degenerate
/// to *"a shorter input produces fewer facts"*, which is arithmetic rather than a property of the
/// connector. With the tail kept, **the two runs are the same records** and the only difference is
/// one control line — which is `epics.md:1012`'s own clause, and a shape no committed FILE may
/// carry (module doc).
///
/// That argument is load-bearing and is therefore carried by an assertion rather than by this
/// paragraph: the returned length is always `records.len() + 1`, and
/// `cut_at_keeps_the_tail_so_the_two_runs_hold_the_same_records` reds if a truncating version is
/// substituted.
///
/// `k >= records.len()` appends the failure at the end, which removes nothing — the degenerate case
/// the sweep excludes by name and which [`AC3`-style callers](crate::fault_injection) must test
/// separately (story 5.13, mutation M4).
pub(crate) fn cut_at(records: &[Record], k: usize) -> Vec<Record> {
    let at = k.min(records.len());
    let mut out = Vec::with_capacity(records.len() + 1);
    out.extend_from_slice(&records[..at]);
    out.push(Record::Failure(ConnectorError::Unreachable {
        detail: CUT_DETAIL.to_string(),
    }));
    out.extend_from_slice(&records[at..]);
    out
}

/// The earliest instant a capability record inserted at `k` may legally carry.
///
/// `from_records` refuses a capability record dated before an observation it postdates
/// ([`crate::fixtures::FixtureError::CapabilityPredatesObservation`]) or before a preceding
/// capability record ([`crate::fixtures::FixtureError::CapabilityOutOfOrder`]). The first is tracked
/// as a **MAX over every preceding observation**, not as the previous line, because a stream is not
/// required to be sorted by `observed_at`.
///
/// Returns `None` when the prefix carries no instant at all — an empty prefix, or one holding only
/// failures — in which case any instant is legal.
pub(crate) fn earliest_legal_as_of(records: &[Record], k: usize) -> Option<Timestamp> {
    records[..k.min(records.len())]
        .iter()
        .filter_map(|record| match record {
            Record::Observation(observation) => Some(observation.observed_at),
            Record::Capability(capabilities) => Some(capabilities.as_of),
            Record::Failure(_) => None,
        })
        .max()
}

/// **M-B — the source goes half-blind after `k` records.**
///
/// Inserts a [`Record::Capability`] restricted to `kinds` at index `k`, **and strips from every
/// following observation any fact whose kind is not in `kinds`**.
///
/// # ⚠️ The strip is mandatory, and it is also the honest shape
///
/// `from_records` validates each observation against the descriptor in force AT ITS OWN POSITION,
/// so a record denying `Mac` in front of an observation carrying one makes the whole stream refuse
/// to load with `UndeclaredFactKind`. Beyond making it legal, the strip is what a half-blind source
/// really does: it stops reporting what it can no longer see, rather than reporting it under a
/// descriptor that denies it.
///
/// # ⚠️ `kinds` is a degeneracy axis, and the caller owns it
///
/// If `kinds` denies nothing the tail actually carries, the mutilation removes nothing and the
/// strictness half is unsatisfiable — green for the wrong reason. Callers must choose `kinds` so it
/// denies at least one kind present after `k`, and **assert it** rather than assume it;
/// [`denied_kinds_present_after`] is what makes that assertion cheap.
///
/// `as_of` is a parameter and not derived here on purpose: deriving it would hide the two ordering
/// rules behind a helper that always satisfies them, and a caller passing a deliberately illegal
/// instant is how those rules get a test. Use [`earliest_legal_as_of`] to obtain a legal one.
pub(crate) fn blind_after(
    records: &[Record],
    k: usize,
    kinds: &BTreeSet<FactKind>,
    as_of: Timestamp,
) -> Vec<Record> {
    let at = k.min(records.len());
    let mut out = Vec::with_capacity(records.len() + 1);
    out.extend_from_slice(&records[..at]);
    out.push(Record::Capability(Capabilities {
        as_of,
        kinds: kinds.clone(),
    }));
    for record in &records[at..] {
        match record {
            Record::Observation(observation) => {
                let mut reduced = observation.clone();
                reduced.facts.retain(|fact| kinds.contains(&fact.kind()));
                out.push(Record::Observation(reduced));
            }
            other => out.push(other.clone()),
        }
    }
    out
}

/// Whether the records after `k` carry at least one fact kind `kinds` denies.
///
/// The non-degeneracy predicate for [`blind_after`]: false means the mutilation would strip nothing
/// and the strictness half could not hold. It reads the ORIGINAL records, so a caller checks it
/// before mutilating rather than inferring it from a result that has already lost the evidence.
pub(crate) fn denied_kinds_present_after(
    records: &[Record],
    k: usize,
    kinds: &BTreeSet<FactKind>,
) -> bool {
    records[k.min(records.len())..]
        .iter()
        .filter_map(Record::as_observation)
        .flat_map(|observation| observation.facts.iter())
        .any(|fact| !kinds.contains(&fact.kind()))
}

/// A kind set that is guaranteed to deny something the records after `k` carry.
///
/// Every kind the stream uses, MINUS the kind of the first fact present after `k`. Returning `None`
/// means the tail carries no fact at all, so no blinding could bite there — the position is
/// degenerate for M-B and the sweep must say so rather than pass vacuously.
pub(crate) fn kinds_denying_something_after(
    records: &[Record],
    k: usize,
) -> Option<BTreeSet<FactKind>> {
    let victim = records[k.min(records.len())..]
        .iter()
        .filter_map(Record::as_observation)
        .flat_map(|observation| observation.facts.iter())
        .map(|fact| fact.kind())
        .next()?;
    let mut kinds: BTreeSet<FactKind> = records
        .iter()
        .filter_map(Record::as_observation)
        .flat_map(|observation| observation.facts.iter())
        .map(|fact| fact.kind())
        .collect();
    kinds.remove(&victim);
    Some(kinds)
}

/// Everything needed to replay a set of records as the connector the stream belongs to.
///
/// Derived from the stream rather than supplied: two committed streams carry their own
/// `connector_id` and scope, and a constant here would load one of them and refuse the other.
#[derive(Debug, Clone)]
pub(crate) struct StreamContext {
    /// The connector every observation in the stream is attributed to.
    pub(crate) id: ConnectorId,
    /// Every distinct scope the stream touches — `from_records` refuses an uncovered one.
    pub(crate) scopes: Vec<Scope>,
    /// The descriptor in force BEFORE any record: dated at the earliest **observation** instant and
    /// admitting every kind the stream carries, so the clean run loads unchanged.
    ///
    /// ⚠️ **Observation instants only — a capability record's `as_of` is read for its `kinds` and
    /// deliberately not for its date.** `from_records` compares a capability record's `as_of`
    /// against preceding RECORDS, never against the constructor's descriptor, so this cannot make a
    /// stream unloadable today. It could if that comparison ever widened, and 5.13b is the story
    /// that feeds control-carrying streams here.
    pub(crate) initial: Capabilities,
}

/// Derive the replay context of a stream from the stream itself.
///
/// Returns `None` for a stream carrying no observation, which has no connector to be attributed to.
///
/// The initial descriptor admits **every kind the stream carries**, which is the only choice that
/// leaves the clean run loadable: containment is positional and the constructor's descriptor governs
/// every observation before the first capability record. It deliberately does NOT derive capability
/// from what was seen in any narrower sense — `fixture_connector`'s module doc explains why that
/// would be worse than any gap — it simply refuses to deny what the file already states.
pub(crate) fn stream_context(records: &[Record]) -> Option<StreamContext> {
    let mut observations = records.iter().filter_map(Record::as_observation).peekable();
    let first = *observations.peek()?;
    let id = first.connector_id;

    let mut scopes: Vec<Scope> = Vec::new();
    let mut kinds: BTreeSet<FactKind> = BTreeSet::new();
    let mut earliest: Option<Timestamp> = None;
    for observation in records.iter().filter_map(Record::as_observation) {
        if !scopes.contains(&observation.scope) {
            scopes.push(observation.scope);
        }
        for fact in &observation.facts {
            kinds.insert(fact.kind());
        }
        earliest = Some(match earliest {
            Some(at) if at <= observation.observed_at => at,
            _ => observation.observed_at,
        });
    }
    // A capability record already in the stream may widen the alphabet beyond what any observation
    // carries. Admitting it costs nothing and refusing it would make such a stream unloadable.
    for record in records {
        if let Record::Capability(capabilities) = record {
            kinds.extend(capabilities.kinds.iter().copied());
        }
    }

    Some(StreamContext {
        id,
        scopes,
        initial: Capabilities {
            as_of: earliest.expect("a stream with an observation has an instant"),
            kinds,
        },
    })
}

/// What one run of these records emitted, and how the poll ended.
///
/// The facts are returned as a `Vec` and **not** as a set: see the module doc. `raw` never enters
/// the comparison — it is opaque provenance no decision reads (D19), so including it would make the
/// oracle sensitive to a field the product is defined to ignore.
///
/// # Panics
///
/// If the records do not load — a mutilation that produces an inadmissible stream is a defect in the
/// mutilation, not an outcome to be handled, and the message names the refusal.
pub(crate) async fn run(records: Vec<Record>, context: &StreamContext) -> RunOutcome {
    let mut connector = FixtureConnector::from_records(
        context.id,
        context.initial.clone(),
        context.scopes.clone(),
        "story 5.13 in-memory replay",
        records,
    )
    .unwrap_or_else(|e| panic!("the mutilated stream must still load: {e}"));

    let mut sink = VecSink::default();
    let outcome = connector
        .poll(context.initial.as_of, &mut sink, CancellationToken::new())
        .await;

    RunOutcome {
        claims: sink.observations.iter().flat_map(claims_of).collect(),
        observations: sink.observations.iter().map(|o| o.obs_id).collect(),
        error: outcome.err(),
    }
}

/// Everything one observation ASSERTS — its facts, and what it says about itself.
///
/// 🔴 The provenance entry is why this is not simply a list of facts. Story 5.13's code review
/// measured the hole: an oracle reading only `Fact` values is blind to a blinded source that starts
/// **back-dating everything it reports**, and `observed_at` is what the engine writes into
/// `valid_from`, `first_seen_at` and `last_seen_at`. D35(a) forbids adding *an assertion*, not
/// adding *a `Fact`* — and an observation asserts its own instant, scope and origin as surely as it
/// asserts a MAC.
///
/// `raw` is the one field deliberately left out: D19 says no decision reads it, and including it
/// would make the oracle sensitive to a field the product is defined to ignore. That exclusion has
/// its own test.
fn claims_of(observation: &Observation) -> Vec<Claim> {
    let mut out: Vec<Claim> = observation
        .facts
        .iter()
        .map(|fact| Claim::Fact(observation.obs_id, fact.clone()))
        .collect();
    out.push(Claim::Provenance {
        obs: observation.obs_id,
        observed_at: observation.observed_at,
        scope: observation.scope,
        connector: observation.connector_id,
    });
    out
}

/// One thing a run asserted. The unit the monotone-honesty comparison counts.
///
/// Exhaustive by construction: [`claims_of`] is the only producer, and a field added to
/// [`Observation`] that carries an assertion must be added HERE or the oracle silently stops
/// seeing it. That is the failure this enum exists to make visible rather than possible.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Claim {
    /// A fact the observation carried, attached to the observation that carried it.
    Fact(ObsId, Fact),
    /// What the observation asserts about ITSELF: when it was seen, where, and by whom.
    Provenance {
        /// The observation making the claim.
        obs: ObsId,
        /// The instant it claims to have been seen at — what the engine stores as `valid_from`.
        observed_at: Timestamp,
        /// The scope it claims to have been seen in.
        scope: Scope,
        /// The connector it claims to come from.
        connector: ConnectorId,
    },
}

/// Everything one run asserted, the observations that carried it, and how the poll ended.
#[derive(Debug, Clone)]
pub(crate) struct RunOutcome {
    /// Every [`Claim`] the run emitted, in emission order, **duplicates preserved**.
    pub(crate) claims: Vec<Claim>,
    /// The observations the run emitted, in order.
    pub(crate) observations: Vec<ObsId>,
    /// The error the poll ended with, or `None` for a clean run.
    pub(crate) error: Option<ConnectorError>,
}

impl RunOutcome {
    /// The FACT claims alone — a view, never a second stored field, so the two cannot drift.
    ///
    /// Used only where a test needs to say *"the facts are identical and something else is not"*.
    pub(crate) fn facts_only(&self) -> Vec<Claim> {
        self.claims
            .iter()
            .filter(|c| matches!(c, Claim::Fact(..)))
            .cloned()
            .collect()
    }
}

/// Multiset inclusion: is every element of `sub` present in `sup`, **counting duplicates**?
///
/// Successive removal, not a set difference, which is the whole point (see the module doc):
/// `[x, x] ⊆ [x]` is **false** here and would be true of any set-based comparison. Needs
/// [`PartialEq`] alone, so no trait is added to `opencmdb-core`.
///
/// ⚠️ **A thin wrapper over [`unaccounted`], deliberately.** The two were written as separate copies
/// of one removal loop, and story 5.13's code review measured the consequence: every acceptance
/// criterion calls `unaccounted(..).is_empty()`, so replacing this function's body with `true` red
/// exactly ONE test while the module doc advertised it as the oracle. One implementation now, so the
/// two spellings cannot drift and the doc names what the criteria actually run on.
pub(crate) fn multiset_included<T: PartialEq + Clone>(sub: &[T], sup: &[T]) -> bool {
    unaccounted(sub, sup).is_empty()
}

/// The elements of `sub` that `sup` cannot account for — what an inclusion failure should REPORT.
///
/// An assertion that says only *"not included"* sends the reader back to the data; this names the
/// facts the faulted run invented. Same removal semantics as [`multiset_included`], which returns
/// `true` exactly when this returns an empty vector.
pub(crate) fn unaccounted<T: PartialEq + Clone>(sub: &[T], sup: &[T]) -> Vec<T> {
    let mut remaining: Vec<T> = sup.to_vec();
    let mut extra = Vec::new();
    for item in sub {
        match remaining.iter().position(|candidate| candidate == item) {
            Some(at) => {
                remaining.swap_remove(at);
            }
            None => extra.push(item.clone()),
        }
    }
    extra
}

/// The distinct `obs_id`s a set of records names.
///
/// Used by AC1's tail-keeping guard to state that a cut removes no observation from the STREAM,
/// only from what the poll reaches. _(An earlier doc named AC2 as the caller; AC2 compares the
/// emitted counts on [`RunOutcome`] and never calls this.)_
pub(crate) fn distinct_observations(records: &[Record]) -> HashSet<ObsId> {
    records
        .iter()
        .filter_map(Record::as_observation)
        .map(|observation| observation.obs_id)
        .collect()
}

/// Tests for fault injection and its oracle — story 5.13.
///
/// What is pinned here is this module's own claim: that a mutilation removes what it says it
/// removes, that the oracle counts duplicates, and that a faulted run's facts are **included in and
/// strictly smaller than** a clean run's. The two are always separate assertions: an inclusion
/// failure is a product defect and a strictness failure is a test defect, and one combined
/// assertion cannot tell a reader which happened.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{fixture_path, read_records, walk_replay_streams};

    const RANDOMIZED: &str = "scenario/replay/randomized-mac.jsonl";
    const PARTIAL: &str = "scenario/replay/partial-then-failed.jsonl";
    const DOWNGRADE: &str = "scenario/replay/capability-downgrade.jsonl";

    fn stream(relative: &str) -> Vec<Record> {
        read_records(&fixture_path(relative).expect("a corpus-relative path"))
            .unwrap_or_else(|e| panic!("{relative} must read: {e}"))
    }

    fn context(records: &[Record]) -> StreamContext {
        stream_context(records).expect("a committed stream carries an observation")
    }

    /// Every committed stream carrying NO control record — the clean side of the sweep.
    ///
    /// Discovered through `walk_replay_streams`, the corpus's own walk, so a stream added tomorrow
    /// enters the sweep without anyone remembering to list it here.
    fn control_free_streams() -> Vec<(String, Vec<Record>)> {
        let mut out = Vec::new();
        walk_replay_streams(&mut |path| {
            let records = read_records(path).unwrap_or_else(|e| panic!("{path:?} must read: {e}"));
            let has_control = records.iter().any(|r| !matches!(r, Record::Observation(_)));
            if !has_control {
                let name = path
                    .file_name()
                    .expect("a walked file has a name")
                    .to_string_lossy()
                    .to_string();
                out.push((name, records));
            }
        });
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    fn only(kind: FactKind) -> BTreeSet<FactKind> {
        let mut set = BTreeSet::new();
        set.insert(kind);
        set
    }

    // ── T1: the mutilations ────────────────────────────────────────────────────

    /// 🔑 AC1's design argument, as an assertion rather than a paragraph.
    ///
    /// The cut ADDS a record and removes none, so the clean and faulted streams hold the same
    /// observations and differ by one control line — which is what makes *"on the same fixture"*
    /// literal. **Mutation M3** (a `cut_at` that truncates) reds exactly here: it returns `k + 1`
    /// records where this demands `len + 1`.
    #[test]
    fn cut_at_keeps_the_tail_so_the_two_runs_hold_the_same_records() {
        let records = stream(RANDOMIZED);
        let faulted = cut_at(&records, 1);

        assert_eq!(
            faulted.len(),
            records.len() + 1,
            "the cut ADDS a failure and removes nothing — a truncating cut_at reds here"
        );
        assert!(
            matches!(faulted[1], Record::Failure(_)),
            "the failure sits AT k, not at the end"
        );
        assert_eq!(
            distinct_observations(&faulted),
            distinct_observations(&records),
            "the same observations are present in both streams; only their emission differs"
        );
    }

    /// The blinding also ADDS one record, and strips ONLY what follows it.
    ///
    /// The prefix is untouched — evaluating it against the new descriptor would be D34 §1's own bug,
    /// *"the past would change status"*.
    #[test]
    fn blind_after_adds_one_record_and_strips_only_what_follows() {
        let records = stream(RANDOMIZED);
        let as_of = earliest_legal_as_of(&records, 1).expect("the prefix carries an instant");
        let faulted = blind_after(&records, 1, &only(FactKind::IpV4), as_of);

        assert_eq!(faulted.len(), records.len() + 1);
        assert!(matches!(faulted[1], Record::Capability(_)));

        let before = faulted[0]
            .as_observation()
            .expect("record 0 is an observation");
        let original = records[0]
            .as_observation()
            .expect("record 0 is an observation");
        assert_eq!(
            before.facts, original.facts,
            "the PREFIX keeps every fact — the past does not change status (D34 §1)"
        );

        for record in &faulted[2..] {
            let observation = record.as_observation().expect("the tail is observations");
            assert!(
                observation.facts.iter().all(|f| f.kind() == FactKind::IpV4),
                "every denied kind is stripped after the record, or the stream cannot load"
            );
        }
    }

    /// ⚠️ The `as_of` rule is a MAX over the prefix, not the previous line — a stream is not
    /// required to be sorted by `observed_at`, and `from_records` tracks it as a max for that
    /// reason. Measured on the committed stream, whose instants ascend, plus a hand-built prefix
    /// whose instants do NOT.
    #[test]
    fn earliest_legal_as_of_is_the_max_of_the_prefix_not_the_last_record() {
        let records = stream(RANDOMIZED);
        let second = records[1]
            .as_observation()
            .expect("record 1 is an observation")
            .observed_at;
        assert_eq!(earliest_legal_as_of(&records, 2), Some(second));

        // The same prefix in descending order: the answer is still the MAX, which is now first.
        let descending = vec![records[1].clone(), records[0].clone()];
        assert_eq!(
            earliest_legal_as_of(&descending, 2),
            Some(second),
            "a max, not a last — this is what a descending prefix distinguishes"
        );
        assert_eq!(
            earliest_legal_as_of(&records, 0),
            None,
            "an empty prefix binds nothing"
        );
    }

    /// 🔴 The rule BITES: a capability record dated before an observation it postdates makes the
    /// whole stream refuse to load. Without this the `as_of` parameter would be decoration.
    #[test]
    fn a_capability_dated_before_the_prefix_is_refused_at_load() {
        let records = stream(RANDOMIZED);
        let legal = earliest_legal_as_of(&records, 2).expect("the prefix carries an instant");
        let context = context(&records);

        let too_early = legal - chrono::Duration::seconds(1);
        let refused = blind_after(&records, 2, &only(FactKind::IpV4), too_early);
        let err = FixtureConnector::from_records(
            context.id,
            context.initial.clone(),
            context.scopes.clone(),
            "as_of probe",
            refused,
        )
        .expect_err("a capability predating its prefix must be refused");
        assert!(
            format!("{err}").contains("predates") || format!("{err:?}").contains("Predates"),
            "the refusal must name the ordering rule, got: {err}"
        );

        // The control: the same mutilation at a legal instant loads.
        let accepted = blind_after(&records, 2, &only(FactKind::IpV4), legal);
        FixtureConnector::from_records(
            context.id,
            context.initial.clone(),
            context.scopes.clone(),
            "as_of control",
            accepted,
        )
        .expect("the same shape at a legal instant must load — else the probe proves nothing");
    }

    /// 🔴 **Mutation M5**: `blind_after` without the strip. The stream refuses to load, and the
    /// carrier is an `Err` rather than a panic — recorded as such.
    #[test]
    fn a_capability_denying_a_kind_the_tail_still_carries_is_refused() {
        let records = stream(RANDOMIZED);
        let context = context(&records);
        let as_of = earliest_legal_as_of(&records, 1).expect("the prefix carries an instant");

        // The un-stripped shape, built by hand: the record is inserted and nothing else changes.
        let mut unstripped = Vec::new();
        unstripped.extend_from_slice(&records[..1]);
        unstripped.push(Record::Capability(Capabilities {
            as_of,
            kinds: only(FactKind::IpV4),
        }));
        unstripped.extend_from_slice(&records[1..]);

        let err = FixtureConnector::from_records(
            context.id,
            context.initial.clone(),
            context.scopes.clone(),
            "unstripped probe",
            unstripped,
        )
        .expect_err("an observation emitting a denied kind must be refused");
        assert!(
            format!("{err}").contains("Mac") || format!("{err:?}").contains("Mac"),
            "the refusal names the denied kind, got: {err}"
        );
    }

    // ── T2: the oracle ─────────────────────────────────────────────────────────

    /// 🔴 **The reason the oracle is a multiset and not a set**, pinned rather than asserted in a
    /// comment. Story 5.11b measured this exact hole on `contradicts`' `facts.len()` term: it was
    /// *"droppable with the suite green, being the only term that catches `[x]` against `[x, x]`"*.
    /// Any set-based comparison passes the second assertion below.
    #[test]
    fn multiset_inclusion_counts_duplicates_where_a_set_would_not() {
        let one = vec!["x"];
        let two = vec!["x", "x"];

        assert!(multiset_included(&one, &two), "[x] ⊆ [x, x]");
        assert!(
            !multiset_included(&two, &one),
            "[x, x] ⊄ [x] — a SET comparison returns true here, and that is the whole point"
        );
        assert_eq!(
            unaccounted(&two, &one),
            vec!["x"],
            "the surplus is NAMED, not merely counted"
        );
        assert!(
            multiset_included(&one, &one),
            "inclusion is reflexive — hence the strictness half"
        );
    }

    /// `raw` is excluded from the oracle (D19: no decision reads it), and the exclusion is
    /// load-bearing rather than incidental — so the bare comparison that WOULD have differed is
    /// asserted beside it, story 5.11b's idiom.
    #[tokio::test]
    async fn raw_is_excluded_and_the_bare_comparison_would_have_differed() {
        let records = stream(RANDOMIZED);
        let context = context(&records);

        let mut with_raw = records.clone();
        let first = match &mut with_raw[0] {
            Record::Observation(observation) => observation,
            _ => panic!("record 0 is an observation"),
        };
        first.raw = Some("a payload no decision reads".to_string());

        let bare_differs = with_raw[0] != records[0];
        assert!(
            bare_differs,
            "the bare `!=` WOULD have refused these two — that is what is excluded"
        );

        let clean = run(records.clone(), &context).await;
        let raw_carrying = run(with_raw, &context).await;
        assert_eq!(
            clean.claims, raw_carrying.claims,
            "the oracle reads facts only: `raw` moves nothing"
        );
    }

    // ── AC1 / AC2: the connector layer ─────────────────────────────────────────

    /// **AC1** — a cut run's facts are included in the clean run's, and strictly.
    ///
    /// Two assertions, two messages. **M2** (a fault-conditional invention) reds the first;
    /// **M1** (a `poll` that continues past the failure) and **M4** (`k = len`) red the second.
    #[tokio::test]
    async fn ac1_a_cut_run_includes_strictly() {
        let records = stream(RANDOMIZED);
        let context = context(&records);

        let clean = run(records.clone(), &context).await;
        let faulted = run(cut_at(&records, 1), &context).await;

        let invented = unaccounted(&faulted.claims, &clean.claims);
        assert!(
            invented.is_empty(),
            "AC1(i) INCLUSION: the faulted run INVENTED {} claim(s): {invented:?}",
            invented.len()
        );
        assert!(
            faulted.claims.len() < clean.claims.len(),
            "AC1(ii) STRICTNESS: the fault did not bite — clean={} faulted={}",
            clean.claims.len(),
            faulted.claims.len()
        );
        assert!(
            faulted.error.is_some(),
            "a cut run ends in Err — if it does not, the failure record was never reached"
        );
        assert!(
            clean.error.is_none(),
            "the clean run of a control-free stream ends Ok"
        );
    }

    /// **AC2** — a blinded run loses FACTS, not observations, and that is what distinguishes it
    /// from AC1. The observation count is asserted equal on purpose.
    #[tokio::test]
    async fn ac2_a_blinded_run_loses_facts_not_observations() {
        let records = stream(RANDOMIZED);
        let context = context(&records);
        let kinds = only(FactKind::IpV4);
        let as_of = earliest_legal_as_of(&records, 1).expect("the prefix carries an instant");

        assert!(
            denied_kinds_present_after(&records, 1, &kinds),
            "the mutilation must deny something the tail CARRIES, or it strips nothing"
        );

        let clean = run(records.clone(), &context).await;
        let faulted = run(blind_after(&records, 1, &kinds, as_of), &context).await;

        let invented = unaccounted(&faulted.claims, &clean.claims);
        assert!(
            invented.is_empty(),
            "AC2(i) INCLUSION: the blinded run INVENTED {} claim(s): {invented:?}",
            invented.len()
        );
        assert!(
            faulted.claims.len() < clean.claims.len(),
            "AC2(ii) STRICTNESS: the blinding did not bite — clean={} faulted={}",
            clean.claims.len(),
            faulted.claims.len()
        );
        assert_eq!(
            faulted.observations.len(),
            clean.observations.len(),
            "AC2: the loss is measured in FACTS — a blinded source still answers"
        );
        assert!(
            faulted.error.is_none(),
            "a capability change leaves the poll Ok (D33)"
        );
    }

    /// 🔴 **Mutation M4, which the sweep deliberately cannot carry**: a failure at `k = len` removes
    /// nothing, so the strictness half is FALSE while the inclusion half stays true. This is the
    /// degenerate case, isolated so the sweep's bound has a witness rather than a convention.
    #[tokio::test]
    async fn m4_a_failure_at_the_end_removes_nothing_and_only_strictness_notices() {
        let records = stream(RANDOMIZED);
        let context = context(&records);

        let clean = run(records.clone(), &context).await;
        let degenerate = run(cut_at(&records, records.len()), &context).await;

        assert!(
            multiset_included(&degenerate.claims, &clean.claims),
            "the inclusion half is GREEN here — which is exactly why it cannot stand alone"
        );
        assert_eq!(
            degenerate.claims.len(),
            clean.claims.len(),
            "nothing was removed: k = len is the degenerate position the sweep excludes"
        );
    }

    /// The committed witness that the synthesised shape is the real one: `partial-then-failed.jsonl`
    /// carries a terminal failure, and its poll ends in `Err` after emitting its prefix.
    #[tokio::test]
    async fn the_committed_faulted_stream_ends_in_err_after_emitting_its_prefix() {
        let records = stream(PARTIAL);
        let context = context(&records);
        let outcome = run(records.clone(), &context).await;

        assert_eq!(
            outcome.observations.len(),
            4,
            "the committed faulted stream emits its four-observation prefix before failing"
        );
        assert!(matches!(
            outcome.error,
            Some(ConnectorError::Unreachable { .. })
        ));
    }

    /// 🔴 **Three paths in this module were reachable by NO test** — story 5.13's code review
    /// measured all three droppable with the suite green: `stream_context`'s capability-widening
    /// loop, `earliest_legal_as_of`'s `Capability` arm, and `blind_after`'s pass-through arm. One
    /// cause: **no test handed a control-carrying stream to any of them.** `RANDOMIZED` is
    /// control-free and `PARTIAL` carries a `Failure`, not a `Capability`.
    #[test]
    fn a_control_carrying_stream_reaches_the_paths_the_control_free_ones_cannot() {
        let records = stream(DOWNGRADE);
        let record_at = records
            .iter()
            .position(|r| matches!(r, Record::Capability(_)))
            .expect("this stream carries a capability record");
        let as_of = match &records[record_at] {
            Record::Capability(c) => c.as_of,
            _ => unreachable!(),
        };

        // (a) the widening. ⚠️ The COMMITTED record declares only kinds its observations already
        // carry, so it cannot show this — measured below, not assumed. The exercising stream is
        // therefore built here, which is what this module is for.
        let carried: BTreeSet<FactKind> = records
            .iter()
            .filter_map(Record::as_observation)
            .flat_map(|o| o.facts.iter())
            .map(|f| f.kind())
            .collect();
        let declared: BTreeSet<FactKind> = match &records[record_at] {
            Record::Capability(c) => c.kinds.clone(),
            _ => unreachable!(),
        };
        assert!(
            declared.is_subset(&carried),
            "the committed record declares nothing unseen — if that changes, the synthetic stream \
             below stops being necessary"
        );
        let unseen = FactKind::DhcpLease;
        assert!(!carried.contains(&unseen));
        let mut widened = declared.clone();
        widened.insert(unseen);
        let synthetic: Vec<Record> = records
            .iter()
            .map(|r| match r {
                Record::Capability(c) => Record::Capability(Capabilities {
                    as_of: c.as_of,
                    kinds: widened.clone(),
                }),
                other => other.clone(),
            })
            .collect();
        assert!(
            context(&synthetic).initial.kinds.contains(&unseen),
            "the initial descriptor admits what a capability record DECLARES even when no \
             observation carries it — dropping the widening loop reds HERE"
        );

        // (b) `earliest_legal_as_of` reads a capability record's own instant.
        let observations_only = records[..=record_at]
            .iter()
            .filter_map(Record::as_observation)
            .map(|o| o.observed_at)
            .max()
            .expect("the prefix carries observations");
        assert!(
            as_of > observations_only,
            "the record must postdate its prefix, or reading it changes no answer"
        );
        assert_eq!(
            earliest_legal_as_of(&records, record_at + 1),
            Some(as_of),
            "the capability record's instant wins — its arm returning None reds HERE"
        );

        // (c) `blind_after` passes a non-observation record through rather than dropping it.
        let kinds = kinds_denying_something_after(&records, record_at + 1)
            .expect("the tail carries a fact");
        let mutilated = blind_after(&records, 0, &kinds, as_of);
        assert_eq!(
            mutilated
                .iter()
                .filter(|r| matches!(r, Record::Capability(_)))
                .count(),
            2,
            "the stream's own capability record survives beside the injected one — a pass-through \
             arm that drops it reds HERE"
        );
    }

    /// `stream_context`'s two remaining claims, measured rather than asserted in prose: the
    /// descriptor is dated at the EARLIEST observation instant, and a scope seen twice is listed
    /// once. Both were droppable with the suite green before this test.
    #[test]
    fn the_context_is_dated_earliest_and_lists_each_scope_once() {
        let records = stream(RANDOMIZED);
        let context = context(&records);
        let instants: Vec<_> = records
            .iter()
            .filter_map(Record::as_observation)
            .map(|o| o.observed_at)
            .collect();
        let earliest = *instants.iter().min().expect("observations");
        let latest = *instants.iter().max().expect("observations");
        assert_ne!(
            earliest, latest,
            "the stream must span time, or the two coincide"
        );
        assert_eq!(
            context.initial.as_of, earliest,
            "dated at the EARLIEST — a `latest` implementation reds here"
        );
        assert_eq!(records.iter().filter_map(Record::as_observation).count(), 3);
        assert_eq!(
            context.scopes.len(),
            1,
            "three observations in one scope, listed once — dropping the `contains` guard reds here"
        );
    }

    /// 🔴 `denied_kinds_present_after` must be able to say **NO**, and nothing measured that.
    ///
    /// Story 5.13's code review replaced its body with `true` and the suite stayed green while
    /// AC3's doc called it the non-degeneracy guard. A guard that cannot refuse is not a guard.
    #[test]
    fn denied_kinds_present_after_can_say_no() {
        let records = stream(RANDOMIZED);
        let everything: BTreeSet<FactKind> = records
            .iter()
            .filter_map(Record::as_observation)
            .flat_map(|o| o.facts.iter())
            .map(|f| f.kind())
            .collect();
        assert!(
            !denied_kinds_present_after(&records, 0, &everything),
            "a kind set denying NOTHING must be refused — a body returning `true` reds here"
        );
        assert!(denied_kinds_present_after(
            &records,
            0,
            &only(FactKind::IpV4)
        ));
        assert!(!denied_kinds_present_after(
            &records,
            records.len(),
            &only(FactKind::IpV4)
        ));
    }

    /// 🔑 The oracle sees PROVENANCE, not only facts — the hole story 5.13's code review measured.
    ///
    /// Two runs whose emitted facts are identical and whose observations differ only in
    /// `observed_at`. Before the widening this compared equal; `observed_at` is what the engine
    /// stores as `valid_from` and folds into the interface's seen-window.
    #[tokio::test]
    async fn the_oracle_sees_provenance_not_only_facts() {
        let records = stream(RANDOMIZED);
        let context = context(&records);

        let mut back_dated = records.clone();
        if let Record::Observation(o) = &mut back_dated[2] {
            o.observed_at -= chrono::Duration::hours(1);
        }

        let clean = run(records, &context).await;
        let shifted = run(back_dated, &context).await;

        assert_eq!(
            clean.facts_only().len(),
            shifted.facts_only().len(),
            "the FACTS are untouched — a fact-only oracle sees nothing here"
        );
        assert!(
            multiset_included(&clean.facts_only(), &shifted.facts_only()),
            "and they are the same facts, in both directions"
        );
        assert!(
            !unaccounted(&shifted.claims, &clean.claims).is_empty(),
            "but the CLAIM set differs — a back-dated observation asserts something the clean run \
             did not, and that is what D35(a) forbids"
        );
    }

    // ── Story 5.13b: the committed twin pair ───────────────────────────────────

    const BLINDED_CLEAN: &str = "scenario/replay/blinded-source.jsonl";
    const BLINDED_FAULTED: &str = "scenario/replay/blinded-source-blinded.jsonl";

    /// Where the committed blinding sits, and the instant it carries.
    ///
    /// ⚠️ `BLINDED_AS_OF` is a LITERAL and deliberately not `earliest_legal_as_of(clean, 2)`, which
    /// returns `00:00:05Z` — the earliest LEGAL instant, i.e. a bound, not the value the file
    /// states. Rebuilding the derivation from the bound would red the twin guard on a difference
    /// the committed pair does not have.
    const BLINDED_K: usize = 2;
    const BLINDED_AS_OF: &str = "2026-04-01T00:00:07Z";

    /// The `obs_id` correspondence between the two committed twins, **as a table**.
    ///
    /// The twins cannot share their ids: `no_obs_id_is_shared_across_replay_streams` refuses any
    /// `obs_id` present in two committed streams, and `blind_after` preserves them. So the guard
    /// below compares modulo THIS mapping and nothing else — every other field must match.
    const TWIN_OBS_IDS: [(&str, &str); 4] = [
        (
            "dbdbdbdb-0000-4000-8000-000000000001",
            "bebebebe-0000-4000-8000-000000000001",
        ),
        (
            "dbdbdbdb-0000-4000-8000-000000000002",
            "bebebebe-0000-4000-8000-000000000002",
        ),
        (
            "dbdbdbdb-0000-4000-8000-000000000003",
            "bebebebe-0000-4000-8000-000000000003",
        ),
        (
            "dbdbdbdb-0000-4000-8000-000000000004",
            "bebebebe-0000-4000-8000-000000000004",
        ),
    ];

    fn blinded_kinds() -> BTreeSet<FactKind> {
        BTreeSet::from([FactKind::Mac, FactKind::IpV4])
    }

    fn twin_instant() -> Timestamp {
        chrono::DateTime::parse_from_rfc3339(BLINDED_AS_OF)
            .expect("a literal RFC 3339 instant")
            .with_timezone(&chrono::Utc)
    }

    fn obs_id(s: &str) -> ObsId {
        ObsId::from_uuid(uuid::Uuid::parse_str(s).expect("a literal UUID"))
    }

    /// 🔴 **The twin guard — the committed blinded twin IS the committed clean twin, blinded.**
    ///
    /// Without it the corpus would hold two files trusted to stay twins, and an edit to one would
    /// silently break a relation that story 5.13 held BY CONSTRUCTION. That is the difference
    /// between a deliberate redundancy and an accidental one.
    ///
    /// # 🔴 Why a whole-`Vec` equality and not a keyed lookup
    ///
    /// The first draft of story 5.13b required only that the mapping be *"total and injective —
    /// never a positional zip"*. **A guard satisfying that letter was MEASURED GREEN** on the very
    /// mutation it was written to catch: swap the faulted twin's third and fourth observations —
    /// both sit AFTER the capability record, so the stream still loads — and a per-`obs_id` lookup
    /// finds every observation exactly where it expects it. Injectivity is not the missing
    /// property; a claim about the SEQUENCE is. So the mapping is applied as a REWRITE of the
    /// derivation and the two whole record vectors are compared in one assertion.
    #[test]
    fn the_committed_blinded_twin_is_the_clean_twin_blinded() {
        let clean = stream(BLINDED_CLEAN);
        let faulted = stream(BLINDED_FAULTED);

        let clean_ids: Vec<ObsId> = clean
            .iter()
            .filter_map(Record::as_observation)
            .map(|o| o.obs_id)
            .collect();
        assert_eq!(
            clean_ids.len(),
            TWIN_OBS_IDS.len(),
            "the mapping must cover the clean twin exactly — a table that stops short of the file \
             would let the observations it omits go uncompared"
        );

        let mapping: std::collections::BTreeMap<ObsId, ObsId> = TWIN_OBS_IDS
            .iter()
            .map(|(from, to)| (obs_id(from), obs_id(to)))
            .collect();
        assert_eq!(
            mapping.len(),
            TWIN_OBS_IDS.len(),
            "TOTAL as a function: no clean id appears twice on the left"
        );
        let images: BTreeSet<ObsId> = mapping.values().copied().collect();
        assert_eq!(
            images.len(),
            TWIN_OBS_IDS.len(),
            "INJECTIVE: no two clean ids share an image, or two observations would collapse onto one"
        );
        for id in &clean_ids {
            assert!(
                mapping.contains_key(id),
                "TOTAL over the file: {id:?} is in the clean twin and has no image"
            );
        }

        let derived = blind_after(&clean, BLINDED_K, &blinded_kinds(), twin_instant());
        let rekeyed: Vec<Record> = derived
            .into_iter()
            .map(|record| match record {
                Record::Observation(mut observation) => {
                    observation.obs_id = mapping[&observation.obs_id];
                    Record::Observation(observation)
                }
                other => other,
            })
            .collect();

        assert_eq!(
            rekeyed, faulted,
            "the committed blinded twin is NOT the committed clean twin blinded — the pair has \
             drifted, and every claim resting on it is void"
        );
    }

    /// **Story 5.13b AC3(i)** — the committed clean twin, blinded in memory, INVENTS nothing and
    /// loses facts rather than observations.
    ///
    /// # 🔴 Why the strictness half is NOT asserted here
    ///
    /// Under a blinding, strictness is guaranteed **before the connector is invoked** — the strip
    /// is what makes the stream loadable at all, so a blinding that removed nothing could not be
    /// built (this module's own doc says so, and the mutation that targets strictness directly dies
    /// at the load rather than at an assertion). Asserting it here would be an assertion carried by
    /// nothing. It is asserted on the CUT instead, by
    /// [`the_clean_twin_carries_the_strictness_half_through_the_cut`], and the routing is itself
    /// asserted there rather than left to a reader.
    ///
    /// What DOES carry *"the fault bit"* here is `denied_kinds_present_after`, and the fact counts
    /// below say by how much.
    #[tokio::test]
    async fn ac3_the_committed_clean_twin_blinded_invents_nothing() {
        let records = stream(BLINDED_CLEAN);
        let context = context(&records);
        let kinds = blinded_kinds();

        assert!(
            denied_kinds_present_after(&records, BLINDED_K, &kinds),
            "the blinding must deny something the tail CARRIES, or it strips nothing and this test \
             passes on a mutilation that did not happen"
        );

        let clean = run(records.clone(), &context).await;
        let faulted = run(
            blind_after(&records, BLINDED_K, &kinds, twin_instant()),
            &context,
        )
        .await;

        let invented = unaccounted(&faulted.claims, &clean.claims);
        assert!(
            invented.is_empty(),
            "INCLUSION: the blinded run INVENTED {} claim(s): {invented:?}",
            invented.len()
        );
        assert_eq!(
            faulted.observations.len(),
            clean.observations.len(),
            "the loss is measured in FACTS — a half-blind source still answers"
        );
        assert_eq!(
            (clean.facts_only().len(), faulted.facts_only().len()),
            (12, 10),
            "the twins' fact counts, pinned: four observations of three facts, of which the two \
             after the record lose their Rtt"
        );
    }

    /// **Story 5.13b AC3(ii)** — the strictness half, on the CUT, plus the ROUTING that puts it
    /// there.
    ///
    /// The routing is asserted rather than described: the clean twin carries no control record, so
    /// it is discovered by `control_free_streams` and swept by
    /// [`ac3_the_sweep_holds_at_every_bounded_position_and_none_is_degenerate`] — where `cut_at`
    /// keeps the tail and strictness is reddenable. Its faulted twin is deliberately NOT in that
    /// set. Without these two assertions *"the strictness half is routed to the cut"* would be a
    /// sentence rather than a fact, and the day the clean twin gained a control record it would
    /// leave the sweep in silence.
    #[tokio::test]
    async fn the_clean_twin_carries_the_strictness_half_through_the_cut() {
        let names: Vec<String> = control_free_streams()
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        assert!(
            names.iter().any(|n| n == "blinded-source.jsonl"),
            "the clean twin must be IN the control-free sweep, which is where its strictness half \
             is measured: {names:?}"
        );
        assert!(
            !names.iter().any(|n| n == "blinded-source-blinded.jsonl"),
            "and the faulted twin must NOT be — it carries a control record, and a stream that \
             already carries one is not a clean side"
        );

        let records = stream(BLINDED_CLEAN);
        let context = context(&records);
        let clean = run(records.clone(), &context).await;
        let cut = run(cut_at(&records, BLINDED_K), &context).await;

        assert!(
            multiset_included(&cut.claims, &clean.claims),
            "INCLUSION on the cut: {:?}",
            unaccounted(&cut.claims, &clean.claims)
        );
        assert!(
            cut.claims.len() < clean.claims.len(),
            "STRICTNESS: the cut did not bite — clean={} cut={}",
            clean.claims.len(),
            cut.claims.len()
        );
    }

    // ── AC3: the sweep, bounded and non-degenerate ─────────────────────────────

    /// **AC3** — every control-free committed stream, every position `0 ≤ k < len`, both
    /// mutilations. Inclusion and strictness at each, as always in two assertions.
    ///
    /// # ⚠️ Two things are asserted that a naive sweep leaves implicit
    ///
    /// **The COUNT.** A degenerate enumerator yielding zero positions leaves this test green while
    /// measuring nothing — story 5.11b's measured defect, reproduced here by construction if the
    /// count were not read. It is pinned at the corpus's own figures, so a stream added tomorrow
    /// reds this and someone looks.
    ///
    /// **The NON-DEGENERACY.** A count of positions says nothing about whether each one removed
    /// anything. `k = len` is excluded by the bound (mutation M4 has its own test), and for M-B the
    /// kind set is CHOSEN to deny a kind the tail carries — `denied_kinds_present_after` asserts it
    /// at every position rather than trusting the choice.
    #[tokio::test]
    async fn ac3_the_sweep_holds_at_every_bounded_position_and_none_is_degenerate() {
        let streams = control_free_streams();
        assert_eq!(
            streams.len(),
            12,
            "the corpus carries 12 control-free replay streams; a new one belongs in this sweep"
        );

        let mut cut_positions = 0usize;
        let mut blind_positions = 0usize;

        for (name, records) in &streams {
            let context = context(records);
            let clean = run(records.clone(), &context).await;
            assert!(
                clean.error.is_none(),
                "{name}: a control-free stream polls clean, or it is not the clean side"
            );

            for k in 0..records.len() {
                // ── M-A, the cut ──
                let faulted = run(cut_at(records, k), &context).await;
                let invented = unaccounted(&faulted.claims, &clean.claims);
                assert!(
                    invented.is_empty(),
                    "{name} k={k}: cut INCLUSION — invented {invented:?}"
                );
                assert!(
                    faulted.claims.len() < clean.claims.len(),
                    "{name} k={k}: cut STRICTNESS — clean={} faulted={} (a bounded k must remove \
                     at least one observation's facts)",
                    clean.claims.len(),
                    faulted.claims.len()
                );
                cut_positions += 1;

                // ── M-B, the blinding ──
                let kinds = kinds_denying_something_after(records, k)
                    .unwrap_or_else(|| panic!("{name} k={k}: the tail carries no fact at all"));
                assert!(
                    denied_kinds_present_after(records, k, &kinds),
                    "{name} k={k}: the kind set must deny something the tail CARRIES"
                );
                let as_of = earliest_legal_as_of(records, k).unwrap_or(context.initial.as_of);
                let blinded = run(blind_after(records, k, &kinds, as_of), &context).await;

                let invented = unaccounted(&blinded.claims, &clean.claims);
                assert!(
                    invented.is_empty(),
                    "{name} k={k}: blind INCLUSION — invented {invented:?}"
                );
                assert!(
                    blinded.claims.len() < clean.claims.len(),
                    "{name} k={k}: blind STRICTNESS — clean={} faulted={}",
                    clean.claims.len(),
                    blinded.claims.len()
                );
                assert_eq!(
                    blinded.observations.len(),
                    clean.observations.len(),
                    "{name} k={k}: blinding loses FACTS, never observations"
                );
                blind_positions += 1;
            }
        }

        assert_eq!(
            cut_positions, 43,
            "43 bounded positions over 12 streams — the count is the guard against an enumerator \
             that yields nothing"
        );
        assert_eq!(
            blind_positions, 43,
            "both mutilations sweep the same positions"
        );
    }

    /// The bound is a DECISION, and this is the measurement behind it: `0 ≤ k ≤ len` would add
    /// exactly one degenerate position per stream — 12 of 55 — every one of them at `k = len`, and
    /// every one of them failing the strictness half. That is mutation M4, twelve times.
    #[tokio::test]
    async fn the_excluded_position_is_exactly_one_per_stream_and_it_is_m4() {
        let streams = control_free_streams();
        let mut degenerate = 0usize;
        let mut total_unbounded = 0usize;

        for (_, records) in &streams {
            let context = context(records);
            let clean = run(records.clone(), &context).await;
            for k in 0..=records.len() {
                total_unbounded += 1;
                let faulted = run(cut_at(records, k), &context).await;
                if faulted.claims.len() == clean.claims.len() {
                    degenerate += 1;
                    assert_eq!(k, records.len(), "the only degenerate position is k = len");
                }
            }
        }

        assert_eq!(
            total_unbounded, 55,
            "0..=len over 12 streams gives 55 positions"
        );
        assert_eq!(
            degenerate, 12,
            "exactly one per stream — 12 of 55, which is why the sweep is bounded at len"
        );
    }

    // ── AC4 / AC5: the engine layer ────────────────────────────────────────────

    /// Connect, migrate, empty every table, and insert the observations the links will name.
    ///
    /// `None` when `DATABASE_URL` is unset — the whole of this layer then passes by RETURNING, and
    /// the suite's counts are identical either way. A green suite says NOTHING about layer 2.
    ///
    /// ⚠️ The observations inserted are the CLEAN run's, which is a superset of any faulted run's,
    /// so the foreign key holds for both passes. Under the blinding that leaves
    /// `observation_record` holding the clean FACTS while the pass resolves the faulted ones —
    /// harmless, because the engine reads the slice it is handed and never the table, but it is a
    /// state no real run produces and it is recorded rather than left to be rediscovered.
    async fn db_fixture(observations: &[Observation]) -> Option<sqlx::MySqlPool> {
        let Ok(url) = std::env::var("DATABASE_URL") else {
            eprintln!("skipping layer-2 test: DATABASE_URL unset");
            return None;
        };
        let pool = sqlx::MySqlPool::connect(&url).await.expect("connect");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrate");
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
                .expect("insert observation");
        }
        Some(pool)
    }

    /// One resolver pass inside one transaction, as D21 requires.
    async fn engine_pass(
        pool: &sqlx::MySqlPool,
        observations: Vec<Observation>,
    ) -> crate::resolver::Resolution {
        use opencmdb_core::repo::WriteRepository;
        crate::repo::MariaRepository::new(pool.clone())
            .transact(move |unit| {
                let observations = observations.clone();
                Box::pin(
                    async move { crate::resolver::resolve(unit.executor(), &observations).await },
                )
            })
            .await
            .expect("the pass must resolve")
    }

    /// The observations a run emitted, in emission order — what the engine is handed.
    async fn emitted(records: Vec<Record>, context: &StreamContext) -> Vec<Observation> {
        let mut connector = FixtureConnector::from_records(
            context.id,
            context.initial.clone(),
            context.scopes.clone(),
            "story 5.13 layer-2 replay",
            records,
        )
        .expect("the mutilated stream must load");
        let mut sink = VecSink::default();
        let _ = connector
            .poll(context.initial.as_of, &mut sink, CancellationToken::new())
            .await;
        sink.observations
    }

    /// A link's identity FOR THIS COMPARISON: the placement it asserts, and nothing else.
    ///
    /// 🔴 The comparison excludes **eight** of [`crate::repo::LinkSnapshot`]'s ten fields —
    /// `rule_id`, `evidence`, `outcome`, `abstention_cause`, `ruleset_version`, `decided_by`,
    /// `valid_from` and `valid_to`. The exclusion is right, and the reason is D35(a)'s own shape: a
    /// fault legitimately WEAKENS a justification. Under the cut, observation 1 keeps its placement
    /// but is settled as a singleton rather than against a partner, so a row-level subset would red
    /// on a run that did exactly the right thing. What the claim is about is the PLACEMENT, and the
    /// placement is what is compared — which is what made story 5.10's `id` exclusion safe too.
    ///
    /// ⚠️ **What this comparison does NOT do, stated because an earlier version of this doc had it
    /// backwards.** It reassured the reader that `evidence` was *"the only one of the three excluded
    /// columns that could have varied at all"*, which reads as *"and therefore nothing can hide
    /// there"*. Story 5.13's code review measured the opposite: an engine persisting an `evidence`
    /// list naming an observation that was never observed passes **every test in this module**, and
    /// only four pre-existing `resolver` tests catch it. `evidence` is a JSON column with no foreign
    /// key, so a fabricated id persists.
    ///
    /// 🔑 **What keeps an invented justification out is the ENGINE'S STRUCTURE, not this oracle**: a
    /// decision's evidence is drawn from the verdict vector, which names only observations in the
    /// slice the pass was handed. That is a real guarantee and a stronger one than a comparison —
    /// but it belongs to `identity::l1`, not to this module, and a story that leans on it must SAY
    /// so and PIN it. [`tests::the_engine_cites_no_evidence_it_was_not_handed`] is that pin: it reds
    /// the day the structure changes.
    ///
    /// ⚠️ Narrower than it looks in the other direction too: at L1 today `rule_id` and `outcome` are
    /// CONSTANT over every row carrying an interface — every member of a `join` group shares the
    /// key — and `decide_singleton` names that SAME rule and DOES carry evidence. So of the three
    /// columns §6 discusses, `evidence` is the only one that could have varied at all.
    fn placement_key(row: &crate::repo::LinkSnapshot) -> (String, Option<String>) {
        (row.observation_id.clone(), row.interface_id.clone())
    }

    fn placements(rows: &[crate::repo::LinkSnapshot]) -> HashSet<(String, Option<String>)> {
        rows.iter()
            .filter(|row| row.interface_id.is_some())
            .map(placement_key)
            .collect()
    }

    /// The shared body of AC4/AC5: run clean, snapshot, purge, run faulted, snapshot, compare.
    ///
    /// Returns `(clean_rows, faulted_rows, faulted_summary)` so each caller asserts for itself —
    /// a helper that asserted would make the two tests one test wearing two names.
    async fn two_passes(
        pool: &sqlx::MySqlPool,
        clean: Vec<Observation>,
        faulted: Vec<Observation>,
    ) -> (
        Vec<crate::repo::LinkSnapshot>,
        Vec<crate::repo::LinkSnapshot>,
        crate::resolver::Resolution,
    ) {
        engine_pass(pool, clean).await;
        let clean_rows = crate::repo::snapshot_links(pool).await.expect("snapshot 1");

        crate::repo::purge_engine_links(pool)
            .await
            .expect("purge the engine's links — interfaces are NOT purged");

        let summary = engine_pass(pool, faulted).await;
        let faulted_rows = crate::repo::snapshot_links(pool).await.expect("snapshot 2");
        (clean_rows, faulted_rows, summary)
    }

    /// **AC4** — a cut pass places no observation the clean pass did not, and strictly fewer.
    ///
    /// `interface_id` is literally comparable because the second pass FINDS its interfaces rather
    /// than minting them: `purge_engine_links` deletes links only (story 5.10's apparatus).
    #[tokio::test]
    async fn ac4_a_cut_pass_places_a_strict_subset() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let records = stream(RANDOMIZED);
        let context = context(&records);

        let clean = emitted(records.clone(), &context).await;
        let faulted = emitted(cut_at(&records, 1), &context).await;
        assert!(
            faulted.len() < clean.len(),
            "the cut must remove an observation, or layer 2 measures nothing"
        );

        let Some(pool) = db_fixture(&clean).await else {
            return;
        };
        let (clean_rows, faulted_rows, summary) = two_passes(&pool, clean, faulted).await;

        let pc = placements(&clean_rows);
        let pf = placements(&faulted_rows);

        let invented: Vec<_> = pf.difference(&pc).collect();
        assert!(
            invented.is_empty(),
            "AC4(a) INCLUSION: the faulted pass placed what the clean pass did not: {invented:?}"
        );
        assert!(
            pf.len() < pc.len(),
            "AC4(c) STRICTNESS: clean placed {} faulted placed {}",
            pc.len(),
            pf.len()
        );
        assert_eq!(
            summary.interfaces_minted, 0,
            "AC4: a faulted pass carries a SUBSET of the keys, so it can mint nothing"
        );
        assert!(
            summary.interfaces_found > 0,
            "it FOUND the clean pass's interfaces — that is what makes interface_id comparable"
        );
    }

    /// 🔑 **The structural property AC4's exclusion leans on, pinned so it reds if it changes.**
    ///
    /// `placement_key` excludes `evidence`, and story 5.13's code review measured what that costs:
    /// an engine persisting an `evidence` list naming an observation that was never observed passes
    /// every other test here. The exclusion is still right — a fault weakens a justification — but
    /// what keeps a fabricated citation out is not this module's oracle, it is `identity::l1`'s
    /// shape: a decision's evidence comes from the verdict vector, which names only observations in
    /// the slice the pass was handed.
    ///
    /// That guarantee is worth having and worth NOT taking on trust. This states it as an assertion
    /// over a real faulted pass: **every `obs_id` any link cites is one the faulted run actually
    /// emitted.** It does not close the hole by widening the comparison — it names the wall that
    /// closes it, and it falls down with the wall.
    #[tokio::test]
    async fn the_engine_cites_no_evidence_it_was_not_handed() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let records = stream(RANDOMIZED);
        let context = context(&records);

        let clean = emitted(records.clone(), &context).await;
        let faulted = emitted(cut_at(&records, 1), &context).await;
        let seen: HashSet<ObsId> = faulted.iter().map(|o| o.obs_id).collect();
        assert!(
            !seen.is_empty(),
            "the faulted run must emit something, or this is vacuous"
        );

        let Some(pool) = db_fixture(&clean).await else {
            return;
        };
        engine_pass(&pool, clean).await;
        crate::repo::purge_engine_links(&pool).await.expect("purge");
        engine_pass(&pool, faulted).await;

        let rows = crate::repo::snapshot_links(&pool).await.expect("snapshot");
        assert!(
            !rows.is_empty(),
            "the faulted pass must write something, or this is vacuous"
        );
        let mut cited = 0usize;
        for row in &rows {
            for id in &row.evidence {
                cited += 1;
                assert!(
                    seen.contains(id),
                    "the faulted pass cited {id} as evidence and the faulted run never emitted it \
                     — an invented justification is an added assertion (D35(a))"
                );
            }
        }
        assert!(
            cited > 0,
            "no link cited any evidence, so the loop asserted nothing"
        );
    }

    /// **AC5** — the blinded pass adds **nothing at all**, which is a STRONGER form of the
    /// property than the one this test used to assert.
    ///
    /// 🔴 **v0.3.0 removed the exception this criterion used to need.** It formerly read *"the
    /// blinded pass adds rows the clean pass has not, and every one is an abstention"*, and its doc
    /// had to argue that an abstention is not an invented fact — true, but an argument, and D35(a)
    /// is the flat statement that **a fault may only REMOVE knowledge, never ADD an assertion**.
    ///
    /// An observation stripped of its MAC has no L1 key, and since v0.3.0 the engine writes no link
    /// for a sighting it can never place. So the blinded pass now adds **no row of any kind**: the
    /// property holds without the clause that had to be defended. *A criterion that no longer needs
    /// its exception is a criterion that got simpler, not one that got weaker.*
    #[tokio::test]
    async fn ac5_a_blinded_pass_adds_nothing_at_all() {
        let _guard = crate::DB_TEST_LOCK.lock().await;
        let records = stream(RANDOMIZED);
        let context = context(&records);
        let kinds = kinds_denying_something_after(&records, 1).expect("the tail carries a fact");
        let as_of = earliest_legal_as_of(&records, 1).expect("the prefix carries an instant");

        let clean = emitted(records.clone(), &context).await;
        let faulted = emitted(blind_after(&records, 1, &kinds, as_of), &context).await;
        assert_eq!(
            faulted.len(),
            clean.len(),
            "the blinding loses facts, not observations — the shape AC5 needs"
        );

        let Some(pool) = db_fixture(&clean).await else {
            return;
        };
        let (clean_rows, faulted_rows, summary) = two_passes(&pool, clean, faulted).await;

        let pc = placements(&clean_rows);
        let pf = placements(&faulted_rows);
        let invented: Vec<_> = pf.difference(&pc).collect();
        assert!(
            invented.is_empty(),
            "AC5(a) INCLUSION: the blinded pass placed what the clean pass did not: {invented:?}"
        );
        assert!(
            pf.len() < pc.len(),
            "AC5(c) STRICTNESS: clean placed {} faulted placed {}",
            pc.len(),
            pf.len()
        );
        assert_eq!(
            summary.interfaces_minted, 0,
            "AC5: a faulted pass carries a SUBSET of the keys, so it can mint nothing"
        );
        assert!(
            summary.interfaces_found > 0,
            "it FOUND the clean pass's interfaces — the property AC4 asserts, and what makes \
             interface_id comparable here too"
        );

        // (b): the faulted pass has NO row the clean pass has not — not a placement, and since
        // v0.3.0 not an abstention either.
        //
        // ⚠️ The guard this replaces asserted `!extra.is_empty()` — *"at least one faulted-only row
        // must EXIST, or the claim is vacuously true"* — because the interesting claim was then
        // about the KIND of the extra rows. There are no extra rows now, so the vacuity guard would
        // itself be asserting that a thing exists which the product deliberately no longer creates.
        // The claim below is not vacuous: it can fail, and the strictness assertion above is what
        // proves the faulted pass really did less.
        let clean_keys: HashSet<_> = clean_rows.iter().map(placement_key).collect();
        let extra: Vec<_> = faulted_rows
            .iter()
            .filter(|row| !clean_keys.contains(&placement_key(row)))
            .collect();
        assert!(
            extra.is_empty(),
            "AC5: a fault may only REMOVE knowledge — the blinded pass wrote rows the clean pass \
             did not: {extra:?}"
        );
    }
}
