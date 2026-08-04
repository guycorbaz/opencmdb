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
//! ## Why the blocker is not decoration
//!
//! [`candidates`] is TOTAL today — every unordered pair of distinct ids — so "the universe" and
//! "all pairs" coincide. That does not make consulting it pointless: it is where the universe is
//! DEFINED, and a pass that read `join`'s keys and never asked would be correct today and silently
//! stop being correct the first time the blocker excludes anything, which F17's `dormant` already
//! plans to make it do [architecture.md:1205]. [`resolve_within`] exists so that the exclusion is
//! reachable from a test rather than only from a future.
//!
//! # The clock is never read
//!
//! Every instant this pass stores is derived from the observations: an interface's window is the
//! `min`/`max` of its group's `observed_at`, and a link's `valid_from` is its own observation's.
//! *"The engine never touches the clock"* [architecture.md:3364], and story 5.10 replays this pass
//! and compares bit for bit. `insert_declared_attribute`'s `NOW(6)` is a DECLARED row authored by a
//! human and is not a precedent.
//!
//! # It is not idempotent, and that is story 5.11's
//!
//! Running the pass twice over the SAME observations is `Err(Constraint("unique"))` and a full
//! rollback: `insert_identity_link` appends, and `identity_link_one_current` refuses a second
//! current row for one `(observation_id, current_subject)`. Superseding an unchanged decision
//! instead of appending is *"no new version for an unchanged decision"*, which
//! `0002_interface_and_identity_link.sql`'s own header already names as story 5.11's. A second pass
//! over DIFFERENT observations on the same key is the supported shape, and it finds the interface
//! rather than minting one.
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
use opencmdb_core::identity::cascade::{
    Conclusion, Decision, IdentityAbstentionCause, RulesetVersion, decide,
};
use opencmdb_core::identity::l1::{CURRENT_RULESET_VERSION, decide_pair, decide_singleton, join};
use opencmdb_core::observation::{InterfaceId, LinkId, ObsId, Observation, Timestamp};
use opencmdb_core::repo::RepositoryError;
use sqlx::MySqlConnection;

use crate::repo::{
    DecidedBy, classify, find_interface_by_l1_key, insert_identity_link, insert_interface,
    open_end, widen_interface_seen_window,
};

/// What one pass did, in counts. Rows, not opinions: every field is something a test can also read
/// back out of the database, which is the point — an oracle that restates the pass's own summary
/// measures nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Resolution {
    /// How many candidate pairs the blocker proposed over this slice — `n(n-1)/2` over DISTINCT
    /// observation ids, which is what makes the universe's size assertable rather than quotable.
    pub candidate_pairs: usize,
    /// Interfaces created by this pass, because their L1 key had never been seen.
    pub interfaces_minted: usize,
    /// Interfaces this pass found by key instead of minting. Story 5.10's replay depends on this
    /// being non-zero on a second pass: a re-minted id would change every reproduced link.
    pub interfaces_found: usize,
    /// Links written, placements and abstentions together.
    pub links_written: usize,
    /// How many of those links are abstentions — a link with no interface and a cause.
    pub abstentions: usize,
}

/// Resolve a slice of observations and write what the engine derives.
///
/// Computes the candidate universe and delegates to [`resolve_within`]. It opens no transaction:
/// the caller wraps it in `WriteRepository::transact`, which is what makes D21's *"an identity
/// decision is NEVER split across two transactions"* structural rather than a promise — and what
/// gives the pass read-your-own-writes, so a second observation of one MAC sees the interface the
/// first one caused.
///
/// # Errors
///
/// Any [`RepositoryError`] the writes produce. `Constraint("unique")` on a second pass over the
/// same observations is the non-idempotence described in this module's doc, not a defect.
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
    let by_id: BTreeMap<ObsId, &Observation> = observations.iter().map(|o| (o.obs_id, o)).collect();

    let mut summary = Resolution {
        candidate_pairs: universe.len(),
        ..Resolution::default()
    };
    let mut placed: BTreeSet<ObsId> = BTreeSet::new();

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
            placed.insert(*obs_id);
            match placement_decision(observation, group, universe, &by_id) {
                Some(decision) => {
                    write_link(&mut *conn, observation, Some(interface), &decision).await?;
                    summary.links_written += 1;
                }
                None => {
                    // Every pair this observation could have been judged on was excluded from the
                    // universe, so nothing argued for the placement the key would otherwise give.
                    write_link(&mut *conn, observation, None, &nothing_was_evaluated()).await?;
                    summary.links_written += 1;
                    summary.abstentions += 1;
                }
            }
        }
    }

    // An observation carrying no MAC has no L1 key, so it is in no group — and it still gets a
    // LINK, never an absence (D14/FR16): the ambiguity is DATA, not a hole.
    for observation in observations {
        if placed.contains(&observation.obs_id) {
            continue;
        }
        let decision = abstention_for(observation, observations, universe);
        write_link(&mut *conn, observation, None, &decision).await?;
        summary.links_written += 1;
        summary.abstentions += 1;
    }

    Ok(summary)
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
    let Some(witness) = group.iter().find(|id| **id != observation.obs_id) else {
        return Some(decide_singleton(observation));
    };
    let pair = CandidatePair::new(observation.obs_id, *witness)?;
    if !universe.contains(&pair) {
        return None;
    }
    Some(decide_pair(observation, by_id[witness]))
}

/// The abstention for an observation that carries no L1 key at all.
///
/// Taken from the ENGINE wherever a pair exists: `verdict_for_pair` is `Neutral` whenever either
/// side carries no MAC, so [`decide_pair`] against any other observation returns
/// `Abstained { AbsenceOfProof }` with the rule that tried recorded in the vector — an abstention
/// with an explanation. Only a slice holding this observation alone has no pair to ask about, and
/// then [`nothing_was_evaluated`] is the honest answer.
///
/// ⚠️ The witness is the smallest `ObsId` other than this one, over the whole slice rather than over
/// a group, for the same determinism reason as [`placement_decision`].
fn abstention_for(
    observation: &Observation,
    observations: &[Observation],
    universe: &BTreeSet<CandidatePair>,
) -> Decision {
    let witness = observations
        .iter()
        .filter(|o| o.obs_id != observation.obs_id)
        .filter(|o| {
            CandidatePair::new(observation.obs_id, o.obs_id)
                .is_some_and(|pair| universe.contains(&pair))
        })
        .min_by_key(|o| o.obs_id);
    match witness {
        Some(other) => decide_pair(observation, other),
        None => nothing_was_evaluated(),
    }
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
async fn write_link(
    conn: &mut MySqlConnection,
    observation: &Observation,
    interface: Option<InterfaceId>,
    decision: &Decision,
) -> Result<(), RepositoryError> {
    guard_decision(decision, &[])?;
    let evidence: Vec<ObsId> = decision
        .verdict_vector
        .first()
        .map(|verdict| verdict.evidence.clone())
        .unwrap_or_default();
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
    .map_err(classify)
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

/// The ruleset every link this pass writes is stamped with — the engine's own, never a local
/// constant. Re-exported for the tests that assert a persisted `ruleset_version`.
pub(crate) const RESOLVER_RULESET_VERSION: RulesetVersion = CURRENT_RULESET_VERSION;

/// Tests for the resolver.
///
/// # Synthetic inputs, by necessity and not by preference
///
/// Nothing here reads `fixtures/`. Two properties this file rests on cannot be exercised by the
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
}
