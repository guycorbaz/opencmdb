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
//! stream already yielded and returns new ones. The clean run and the faulted run are then *the same
//! records* differing by one control line, which is what makes *"on the same fixture"* literal
//! rather than approximate.
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
    Capabilities, ConnectorId, Fact, FactKind, ObsId, Scope, Timestamp,
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
    /// The descriptor in force BEFORE any record: dated at the stream's earliest instant and
    /// admitting every kind the stream carries, so the clean run loads unchanged.
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
        facts: sink
            .observations
            .iter()
            .flat_map(|observation| {
                observation
                    .facts
                    .iter()
                    .map(|fact| (observation.obs_id, fact.clone()))
            })
            .collect(),
        observations: sink.observations.iter().map(|o| o.obs_id).collect(),
        error: outcome.err(),
    }
}

/// One run's emitted facts, the observations that carried them, and how the poll ended.
#[derive(Debug, Clone)]
pub(crate) struct RunOutcome {
    /// Every `(obs_id, fact)` the run emitted, in emission order, duplicates preserved.
    pub(crate) facts: Vec<(ObsId, Fact)>,
    /// The observations the run emitted, in order.
    pub(crate) observations: Vec<ObsId>,
    /// The error the poll ended with, or `None` for a clean run.
    pub(crate) error: Option<ConnectorError>,
}

/// Multiset inclusion: is every element of `sub` present in `sup`, **counting duplicates**?
///
/// Implemented by successive removal rather than by a set difference, which is the whole point (see
/// the module doc): `[x, x] ⊆ [x]` is **false** here and would be true of any set-based comparison.
/// Needs [`PartialEq`] alone, so no trait is added to `opencmdb-core`.
///
/// Quadratic, deliberately. The corpus's largest stream carries 6 observations and the fact counts
/// are in the tens; an index would buy nothing and would need the very `Ord`/`Hash` this refuses.
pub(crate) fn multiset_included<T: PartialEq + Clone>(sub: &[T], sup: &[T]) -> bool {
    let mut remaining: Vec<T> = sup.to_vec();
    for item in sub {
        match remaining.iter().position(|candidate| candidate == item) {
            Some(at) => {
                remaining.swap_remove(at);
            }
            None => return false,
        }
    }
    true
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

/// The distinct `obs_id`s a set of records names — the denominator AC2 compares against.
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
    use opencmdb_core::observation::{HostnameSource, Observation};

    const RANDOMIZED: &str = "scenario/replay/randomized-mac.jsonl";
    const PARTIAL: &str = "scenario/replay/partial-then-failed.jsonl";

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
        let mut descending = vec![records[1].clone(), records[0].clone()];
        descending.truncate(2);
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
            clean.facts, raw_carrying.facts,
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

        let invented = unaccounted(&faulted.facts, &clean.facts);
        assert!(
            invented.is_empty(),
            "AC1(i) INCLUSION: the faulted run INVENTED {} fact(s): {invented:?}",
            invented.len()
        );
        assert!(
            faulted.facts.len() < clean.facts.len(),
            "AC1(ii) STRICTNESS: the fault did not bite — clean={} faulted={}",
            clean.facts.len(),
            faulted.facts.len()
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

        let invented = unaccounted(&faulted.facts, &clean.facts);
        assert!(
            invented.is_empty(),
            "AC2(i) INCLUSION: the blinded run INVENTED {} fact(s): {invented:?}",
            invented.len()
        );
        assert!(
            faulted.facts.len() < clean.facts.len(),
            "AC2(ii) STRICTNESS: the blinding did not bite — clean={} faulted={}",
            clean.facts.len(),
            faulted.facts.len()
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
            multiset_included(&degenerate.facts, &clean.facts),
            "the inclusion half is GREEN here — which is exactly why it cannot stand alone"
        );
        assert_eq!(
            degenerate.facts.len(),
            clean.facts.len(),
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

        assert_eq!(outcome.observations.len(), 4);
        assert!(matches!(
            outcome.error,
            Some(ConnectorError::Unreachable { .. })
        ));
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
            11,
            "the corpus carries 11 control-free replay streams; a new one belongs in this sweep"
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
                let invented = unaccounted(&faulted.facts, &clean.facts);
                assert!(
                    invented.is_empty(),
                    "{name} k={k}: cut INCLUSION — invented {invented:?}"
                );
                assert!(
                    faulted.facts.len() < clean.facts.len(),
                    "{name} k={k}: cut STRICTNESS — clean={} faulted={} (a bounded k must remove \
                     at least one observation's facts)",
                    clean.facts.len(),
                    faulted.facts.len()
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

                let invented = unaccounted(&blinded.facts, &clean.facts);
                assert!(
                    invented.is_empty(),
                    "{name} k={k}: blind INCLUSION — invented {invented:?}"
                );
                assert!(
                    blinded.facts.len() < clean.facts.len(),
                    "{name} k={k}: blind STRICTNESS — clean={} faulted={}",
                    clean.facts.len(),
                    blinded.facts.len()
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
            cut_positions, 39,
            "39 bounded positions over 11 streams — the count is the guard against an enumerator \
             that yields nothing"
        );
        assert_eq!(
            blind_positions, 39,
            "both mutilations sweep the same positions"
        );
    }

    /// The bound is a DECISION, and this is the measurement behind it: `0 ≤ k ≤ len` would add
    /// exactly one degenerate position per stream — 11 of 50 — every one of them at `k = len`, and
    /// every one of them failing the strictness half. That is mutation M4, eleven times.
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
                if faulted.facts.len() == clean.facts.len() {
                    degenerate += 1;
                    assert_eq!(k, records.len(), "the only degenerate position is k = len");
                }
            }
        }

        assert_eq!(
            total_unbounded, 50,
            "0..=len over 11 streams gives 50 positions"
        );
        assert_eq!(
            degenerate, 11,
            "exactly one per stream — 11 of 50, which is why the sweep is bounded at len"
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
    /// 🔴 `rule_id`, `evidence` and `outcome` are deliberately EXCLUDED. A fault legitimately
    /// WEAKENS a justification — under the cut, observation 1 keeps its placement but is settled as
    /// a singleton rather than against a partner — and a row-level subset would red on a run that
    /// did exactly the right thing. What the claim is about is the PLACEMENT, and the placement is
    /// what is compared, which is what made story 5.10's `id` exclusion safe too.
    ///
    /// ⚠️ Narrower than it looks, and measured: at L1 today `rule_id` and `outcome` are CONSTANT
    /// over every row carrying an interface — every member of a `join` group shares the key, so the
    /// rule is always `l1-exact-mac` and the conclusion always a match. `decide_singleton` names
    /// that SAME rule and DOES carry evidence (the observation itself). So `evidence` is the only
    /// one of the three excluded columns that could have varied at all.
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

    /// **AC5** — the blinded pass adds rows the clean pass has not, and **every one is an
    /// abstention**.
    ///
    /// 🔑 An abstention is not an invented fact. D35(a) forbids ADDING an assertion; an abstention
    /// asserts nothing — it is the recorded absence of one, and FR16 makes it a first-class outcome.
    /// An observation stripped of its MAC has no L1 key, so the engine says so in a row rather than
    /// staying silent, and that row is the honest answer.
    ///
    /// ⚠️ (b) is largely IMPLIED by (a): a faulted-only row carrying an interface would already be
    /// in `pf \ pc`. Its one independent clause is `outcome == "abstained"` — it is what refuses a
    /// faulted-only row that carries no interface and is not an abstention.
    #[tokio::test]
    async fn ac5_a_blinded_pass_adds_only_abstentions() {
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
            "AC4(a) INCLUSION: the blinded pass placed what the clean pass did not: {invented:?}"
        );
        assert!(
            pf.len() < pc.len(),
            "AC4(c) STRICTNESS: clean placed {} faulted placed {}",
            pc.len(),
            pf.len()
        );
        assert_eq!(summary.interfaces_minted, 0);

        // (b): every row the faulted pass has and the clean pass has not is an ABSTENTION.
        let clean_keys: HashSet<_> = clean_rows.iter().map(placement_key).collect();
        let extra: Vec<_> = faulted_rows
            .iter()
            .filter(|row| !clean_keys.contains(&placement_key(row)))
            .collect();
        assert!(
            !extra.is_empty(),
            "AC5: at least one faulted-only row must EXIST, or the claim is vacuously true"
        );
        for row in &extra {
            assert_eq!(
                row.outcome, "abstained",
                "AC4(b): a faulted-only row is a PLACEMENT, not an abstention: {row:?}"
            );
            assert!(
                row.interface_id.is_none(),
                "an abstention names no interface"
            );
        }
    }

    /// A hand-built `HostnameSource` fact, used by no other test here, exists to keep the
    /// `HostnameSource` import honest if the oracle ever grows a kind-specific branch.
    #[test]
    fn a_hostname_fact_compares_by_value() {
        let a = Fact::Hostname {
            name: "doc-host".to_string(),
            source: HostnameSource::Dhcp,
        };
        let b = a.clone();
        assert!(multiset_included(&[a], &[b]));
    }
}
