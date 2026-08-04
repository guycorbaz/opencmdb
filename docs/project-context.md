# opencmdb — Project Context

_Auto-loaded by BMad workflows. Concise grounding + resume point. Full detail lives in
`_bmad-output/planning-artifacts/`._

## Published (2026-07-17)

- **Repo (PUBLIC):** https://github.com/guycorbaz/opencmdb — branch `master`, first commit is planning artifacts + the Rust workspace skeleton. GitHub handle is `guycorbaz` (NOT `gcorbaz`, the system user).
- **Landing page (GitHub Pages):** https://guycorbaz.github.io/opencmdb/ — self-contained modern site on the `gh-pages` branch (single `index.html`, no CDN). Edit it there; `master` does not carry the site.
- **README.md** on `master` — English, refreshed 2026-07-22 to match reality: it had still claimed
  *"the code has just begun… a skeleton"* and *"there is no runnable product to deploy"* while
  `v0.1.0`/`v0.1.1` were tagged and on Docker Hub. It now states what runs (one page, one connector,
  a real gap on a small perimeter), what does NOT (triage inbox, IPAM, UniFi, alerts, admin UI), and
  that ~1/5 of the planned work is done. **The `LICENSE` file EXISTS** — AGPL-3.0-or-later, verbatim,
  committed 2026-07-22 (`135ff46`); the README references it and the same identifier is declared in
  the `Cargo.toml`s. _(This line read "Still outstanding: there is no LICENSE file" until 2026-07-26 —
  false for four days, in the file BMad auto-loads first. Found in Epic 4's retrospective.)_
- **Docker Hub image is `gcorbaz/opencmdb`** — the SYSTEM-user handle, *not* the GitHub handle
  `guycorbaz`. The two differ; do not "correct" either one into the other.
- **`docker/docker-compose.yml` and `docker/.env.example` DO exist** (under `docker/`, not at the
  repo root — an `ls` at the root misses them, which produced a false "the compose file is missing"
  claim on 2026-07-22).
- **Docs/site are English-only** (locked rule); private network data is scrubbed from all artifacts (see [[no-private-data-in-artifacts]]).

## What it is

A self-hosted, single-binary **Rust** network reconciliation engine (lightweight IPAM + a light
application CMDB + network topology) for advanced home-labs and SMBs without a dedicated IT team.
**Core thesis:** continuously compare the **observed** state (auto-discovered) with the **declared**
state (documented by the operator); the *gap* between them is the product. Open-source, self-hosted
(Docker on Synology priority), distributed via Docker Hub.

## Where the code is (2026-07-22) — READ THIS FIRST

**Planning is done and the code is well past the skeleton.** `v0.1.1` is tagged and released to
Docker Hub; the binary runs and scans on Guy's NAS (frontend on macvlan, no Traefik).
⚠️ The NAS is still on the `0.1.0` image.

| Epic | State |
|---|---|
| **E1** Les gates tiennent | ✅ done (6 stories) |
| **E2** Le contrat de connecteur | ✅ done (5 stories) |
| **E3** Mon premier écart réel — **v0.1** | ✅ done (10 stories), retrospective held |
| **E4** Infra fixtures & corpus de pièges — v0.2 | ✅ **done 2026-07-25** (19/19 authored; 4.19 split — see below) |
| **E5** Identité d'interface fiable — v0.3 | 🔵 **in-progress** since 2026-07-27 — **17 stories, 12 done** (5.9b INSERTED 2026-08-03 at story 5.9's contexting: the schema and the resolver that fills it are two stories) (5.4b INSERTED 2026-07-29 at 5.4's contexting: the verdict algebra as a TOTAL function, plus the `xtask` gate that refuses a float under `identity/`); **5.1, 5.2 and 5.2b done** (PR #41, #44, #46, all merged 2026-07-28). Epic 5's **three inherited-debt stories are closed**; the engine proper starts at **5.3, `done`** (PR #48, merged 2026-07-29) — it ships the identity engine's own abstention vocabulary and NO engine. **5.4 is `done`** (PR #52, merged 2026-07-29 as `da87b62`) — the engine's return type (`Verdict`, `RuleVerdict`, `RulesetVersion`, `Conclusion`, `Decision`) and still NO algebra. **5.4b is `done`** (PR #55 squash-merged 2026-07-30 as `4f4e774`, code-reviewed first) — `decide` as a TOTAL pure function over D13's table (the uncovered class abstains; the D13 correction is issue #54, a milestone act) plus the sixth `xtask` gate, `float-free`. Still no rule and no producer. The review's 22 patches are applied and 5 deferrals registered; what it caught was mostly **claims, not behaviour** — a shipped doc quoting 47 offenders where the tree it described gave 45, a register count of eight where ten were annotated, this very file's test count left stale under a checked-off task, and a premise assertion that had gone inert. On the code: `decide` now matches on `Option<RuleId>` so the two unreachable arms are gone, and the gate's matcher is a numeric-literal tokeniser — `1e-3` and `1.` were green, `"192.168.0.1"` and `t.0.1` were red, all measured. **5.5 is `done`** (PR #57 squash-merged 2026-07-31 as `0ebd50f`, code-reviewed first) — the L1 join, the corpus's two rules and the first caller of `decide` outside its own tests, in the new `identity/l1.rs`; **281 → 309 tests**; AC3 discharged as a NEGATIVE requirement (no IANA predicate: D13's `Disqualifying` label is *"as GROUPING anchors"*, and grouping is L2, so implementing it at L1 would have reddened two committed traps). Six mutations, **every red assertion-carried**; the one the VALIDATION added (a non-canonical rule id) reds ten tests where the validation agent's self-referential version red zero. Of the register's eight requirements, **four closed and four open, each with the measurement that says why** — R7 was reported closed and the code review downgraded it. Its three-layer review found **6 HIGH, and three of them were about CLAIMS rather than code, all three the implementer's** — the fourth consecutive story with that defect. It also found that a GROUP address (broadcast/multicast) is treated as an interface identity, a false merge the corpus can never catch; Guy chose to document and register it rather than filter, with the behaviour pinned by a test. **5.6 is `done`** (PR #59 squash-merged 2026-08-01 as `020c706`, code-reviewed first) — the blocker, in the new `identity/blocking.rs`: `CandidatePair` (private fields ordered by its constructor; `new(a, a)` is `None`, which gives the self-pair precondition its first holder) and `candidates()`, **TOTAL by decision** — every unordered pair of distinct `obs_id`s, calling neither `join` nor `decide_pair`, because proposing is not judging and a blocker that consults a rule is that rule's echo. **The float was TYPED, not avoided**: `BLOCKING_RECALL_FLOOR_PER_MILLE: u32 = 999` on D13's own milli-units corollary, and the `float-free` gate walks **4** files where it walked 3 — unweakened. The corpus assertion lives in `fixtures.rs`'s test module (D47: core may not read files) and **asserts rather than quotes** its counts: 24 traps, 23 pairs, 10 `must-merge`, recall 1000‰. **309 → 332 tests.** Two mutations carry the story: **M1 (an exact-MAC blocker) scores 700‰ exactly**, and **M2 (a same-`l2_domain` blocker) leaves the WHOLE CORPUS GREEN** — only one synthetic cross-domain test stands between that wrong blocker and a pass, which is why the story required it to be written first. Zero compiler-carried reds across all six mutations, and three divergences from the predictions were stated rather than left implicit. The blocker has **no production caller**. Its three-layer code review applied **9 patches**, registered 2 deferrals and dismissed 11 findings, taking the workspace to **333**; **6 of the 9 were sentences, not behaviour** — the fifth story running with that shape. The HIGH was a doc **refuted by a measurement inside the same commit**: it claimed blocking on the MAC, the hostname *or* the `l2_domain` would each pass the whole corpus, where measurement over the ten `must-merge` pairs gives MAC **700‰** (the story's own M1, twelve lines below the sentence), hostname **400‰**, `l2_domain` **1000‰** — only the third passes, and it is the one the corpus is blind to. The one behavioural gap: nothing fed `candidates` an observation with **empty `facts`**, so a fact-presence narrowing was measured GREEN across all 332 tests and no committed observation (0 of 51) could have caught it. **5.7 is `done`** (PR #61 squash-merged 2026-08-02 as `d47631b`, code-reviewed first) — the seam is crossed: `score::outcome_of(&Decision) -> Outcome` (exhaustive, no `_` arm, and **still no `From` impl** — the refusal was kept because a `From` makes the conversion free at every call site) plus a new `crates/opencmdb-bin/src/l1_runner.rs` that walks the corpus, asks the real engine about the pair each trap names, and fills `score_corpus`'s map. **`score_corpus`'s signature and body were unchanged BY 5.7**; the seam was a `BTreeMap<TrapId, Outcome>`, which is data _(5.8 widened that value to `Answer`)_. The committed corpus then reported **24 discovered, 13 scored, 0 failures, 0 wrong-rule, passed** _(story 5.8 turned that last word to FALSE — see below)_ — `scored` had read **0** since story 4.6b, nine stories. `scored_in` = 7 / 6 / **0**, and that zero is asserted with its reason: the `must-abstain` column is measured by nothing. **333 → 350 tests.** Six mutations, every red assertion- or panic-carried; **M4 was run on both sides and the difference is the finding** — dropped from `answer_trap` it panics, dropped from `l1_answers` the **entire suite stays green**, because the level selector removes the corpus's only pairless trap first. 🔴 It also corrects a premise story 5.8 was given: **eleven** committed traps are unanswerable at L1, not eight — the three `must-abstain` ones are invisible to an `l2-*` selector because `Expectation::MustAbstain` carries a cause and no rule. `epics.md` was deliberately not edited; the correction is registered with 5.8 as owner. **5.7 has now been code-reviewed** (three layers, 2026-08-02): 11 patches applied, 2 deferrals registered, 7 findings dismissed, **350 → 352 tests**, six gates green — and it stays at `review`, because `done` is the MERGE's business here. AC1–AC10 were re-verified MET by independent measurement, and **6 of the 11 patches were sentences rather than behaviour — the sixth consecutive story with that shape**. The two behavioural ones were both measured INVISIBLE first: `answer_trap` had **re-opened the self-pair** story 5.6 closed in the type — a hand-built `Trap { observations: vec![x, x] }` merges on `l1-exact-mac`, since an observation shares every key with itself, and `answer_trap` never calls `read_traps`, so `Trap::validate`'s `DuplicateObservation` did not hold the precondition there (Guy's call: close it in `named_pair`, not in a doc); and nothing covered a trap naming THREE observations, where relaxing the pattern to `[a, b, ..]` left all 350 tests green. The HIGH claim was again a doc **falsified inside its own file**: `CLAUDE.md` still said *"Nothing feeds the corpus harness (5.7)"* in the present tense 400 words above *"the seam is crossed"* — this file had corrected the identical sentence in 5.7's own commit and its twin was missed. Also caught: a Debug Log figure of **311** code lines for `trap_gate.rs` where it is **405**, *below* the story's own 384 baseline and so impossible; and *"all seven ids"*, quoted in three documents and guarded by `checked > 0` alone, now asserted as `checked == 21` / `distinct == 7` — placed in the committed-corpus test and deliberately **not** in the shared helper, where it would have masked the guard M5c proves load-bearing. **5.8 is `done`** (PR #63 squash-merged 2026-08-03 as `bbd3b3a` after a green CI run, code-reviewed first) — the bucket for the traps L1 cannot answer, and 🔴 **it turns the committed gate RED on purpose**: `24 discovered, 13 scored, 0 failures, 0 wrong-rule, **11 unanswerable**, passed = FALSE`, and it stays false until Epic 6 implements `l2-*` — which is what `epics.md:416` always said (*"NFR4 stays RED and is closed by Epic 6"*) and what D18 demands (*"a gate that cannot fall is decoration"*). `score::Answer` (`Answered(Outcome)` | `Unanswerable { cause }`) and `UnanswerableCause` (the three measured classes, **8 / 2 / 1**) land in `opencmdb-core`; `Report` gains a fourth bucket that BLOCKS, with `unanswered_in(column)`; `l1_answers` becomes **TOTAL over the corpus** (24 entries, not 13). **The seam widened in VALUE, not arity** — `score_corpus` takes `&BTreeMap<TrapId, Answer>`, still no engine, no callback, no closure, so 4.6b's AC1 survives literally and an EMPTY map is still green, asserted in one line. 🔴 **`Unanswerable` is NOT an abstention**: map it to `Outcome::Abstained` and `example-must-abstain` PASSES because nothing was asked — D18's cowardice moved from the engine up to the harness, and mutation M3 measures it. The classification is **PAIR-FIRST**, which is what yields 8 / 2 / 1 rather than 8 / 3 / 0, and the ANSWERED set is invariant under that order. The `NFR4 NOT MET` line renders **only while the bucket is non-empty**, so it deletes itself when Epic 6 empties it. **352 → 364 tests**, and **367** after the review. It **edits `epics.md:1545`** (8 → eleven) — the one lifting of the verify-only rule, taken because 5.7 registered that correction with 5.8 as owner — and closes the second inherited entry: `l1_answers` refuses a cross-file duplicate `TrapId`, **tested on `l1_answers` directly**, because `score_corpus` refuses the same corpus for its own reasons and a harness-level test stays GREEN with the runner's guard deleted. **5.8 has now been code-reviewed** (three layers, 2026-08-03): 19 patches applied, 2 decisions taken by Guy, **364 → 367 tests**, six gates green. 🔑 **Four of the defects violated the story's OWN AC8** — the doc twins that AC exists to keep in step — and two guards asserted NOTHING before their patch: `unaccounted()`'s arithmetic, and the partition assertion, which now READS THE MAP and reds at `23 != 24` under M5b where the literal-oracle form stays green — the M5 family again, an oracle restating the expectation instead of measuring the code. The decisions: the `NFR4 NOT MET` line is **ventilated by cause** rather than naming one closer (Epic 6 takes the bucket 11 → 3, so a single-closer sentence would have kept naming as its closer the epic that had just shipped), and `TrapError::RuleMalformed` mirrors `FamilyMalformed`, because a padded rule id was silently mis-routed into the bucket and explained there with a false sentence. **5.9 is `done`** (PR #65 squash-merged 2026-08-03 as `57db541` after a green CI run, code-reviewed first) — the persistence schema, and the first DDL since story 3.2. Its deliverable is **invisible without a running MariaDB**: `DATABASE_URL` is unset locally, every DB-backed test passes by `return`ing, and the suite reports 176/156/46 either way — so it was built and mutated against a real `mariadb:10.11.11` on host port **13306** (3306 is held by an unrelated container; the validation caught that before it could migrate another project's database). It ships `0002_interface_and_identity_link.sql` — `interface`, `identity_link`, `link_candidate`, and **no `device`, no `entity` supertype, no `state`**, all registered with their owners — plus `InterfaceId`/`LinkId` and the adapter. **367 → 383 tests.** 🔴 SPLIT at contexting: **5.9b INSERTED** (Epic 5 → 17 stories), so **the blocker still has no production caller** and `join` still has no cross-crate caller. 🔑 The link's uniqueness key is `(observation_id, link_subject, valid_to)`: the narrow `(observation_id, valid_to)` was **measured refusing a legitimate multi-NIC write** (`join` puts one observation on every key it carries; `multi-nic` is a committed trap family), and its abstention half is closed by a **second sentinel**, the nil UUID — D21's NULL trap closed twice by one idiom. ⚠️ The sentinel is a WRITTEN column plus a CHECK, not a generated one: MariaDB 10.11 refuses to INDEX a generated column coalescing to a string literal (**error 1901**, measured `STORED` and `VIRTUAL`) — unknown to both the story and its validation. Seven mutations, every red assertion-carried, zero compiler-carried; **M3 first came back GREEN** — dropping the rule-XOR-cause CHECK left all 378 tests passing, because the adapter derives rule and cause from one `match` and cannot emit an incoherent pair, so two raw-SQL inserts now measure it. The validation found **6 HIGH, 4 of them from the agent that COMPILED the story** (fifth consecutive story with that split): two of six prescribed mutations were no-ops or not executable (M4, M5), and AC3's *"exactly one current"* assertion existed in no prescribed test **5.9 has now been code-reviewed** (three layers, 2026-08-03; two ran against their own live `mariadb:10.11.11` and the Auditor re-executed the whole mutation pass): **1 decision, 22 patches, 13 deferrals, 2 dismissed**, taking the workspace to **383 tests** with five NEW mutations (M8–M12), all assertion-carried. 🔴 **Guy's uniqueness key was arbitrated a SECOND time, on a measurement that falsified the first**: `(observation_id, link_subject, valid_to)` contains `valid_to`, which is `NOT NULL` on CLOSED rows too, so the key constrained HISTORY — two versions of one placement closed at the same DERIVED instant collided (`ERROR 1062`) and the second close was refused **while the link silently stayed current**. Since every instant here is data-derived and never the clock, a replay reproduces instants, so that is story 5.10's purge-and-replay, not an exotic path. The key is now `(observation_id, current_subject)`, where `current_subject` is the interface (or D21's `NIL_INTERFACE`) **while current and NULL once superseded** — NULL-distinctness used DELIBERATELY to drop closed rows out of the key, where D21 warns against suffering it accidentally. 🔑 **`architecture.md:1468` had already prescribed that sentinel** — *"Same reasoning for `NIL_INTERFACE`/`NIL_DEVICE`"* — which decision 9 re-derived over thirty lines instead of citing. Also caught: `ON DELETE CASCADE` was missing, so **story 5.10's purge failed `ERROR 1451`** the moment an engine link carried a candidate — the ambiguity case the table exists for; `close_identity_link` had no guard, so closing an unknown id returned `Ok(())`, a re-close rewrote history and closing AT the sentinel **resurrected a superseded link as current**; an `interface` minted with the nil UUID stole an abstention's slot; and **four more DDL guards plus three persisted tokens (`OPERATOR`, `no_match`, `absence_of_proof`) were measured droppable with the whole suite green** — the M3 family again, four more times. 🔑 **The guard the second arbitration exists to install had no recorded red at all**: the multi-NIC test was reddened by none of M1–M7, and M8 is that missing mutation. And **four of the review's patches were false numbers or sentences in the story's own documents**, including *"11 done"* in both twins while the same files said 5.9 was at `review` — a **direct violation of the story's own AC10**, committed in the very commit that claims to enforce it. **5.9b is `review`** (not `done` — that is the MERGE's business) — the resolver, and the first production caller of `identity::blocking::candidates` as well as the first cross-crate caller of `identity::l1::join`, in the new `crates/opencmdb-bin/src/resolver.rs`. **`join` NAMES the interface** (at L1 its map IS the set of interfaces), **the blocker and `decide_pair` JUSTIFY each placement**, and D13's order is the order of the pass. `resolve` delegates to `resolve_within(conn, observations, &universe)`, and that seam is not decoration: without it the universe check is unreachable and dropping it was measured leaving the **entire suite green**, because `candidates` is TOTAL. **383 → 408 tests.** 🔴 `epics.md`'s AC1 — *"each observation carrying a MAC lands on exactly ONE interface"* — is **falsified by the code it describes**: `join` loops `for key in keys_of(observation)`, so a two-MAC observation lands on two; Guy widened it at contexting to *one interface per L1 KEY*, `epics.md` was NOT edited, and the correction is registered with Epic 5's retrospective. 🔑 **`identity::l1::decide_singleton` is the one change to the engine**: an observation alone on its key has no pair, `insert_identity_link` requires a `Decision`, and both alternatives were worse — a struct literal with an empty `verdict_vector` is the *"merged, with no explanation"* shape D13 forbids, and composing an L1 verdict in `opencmdb-bin` is what `verdict_for_pair`'s `pub(crate)` exists to prevent. **No struct-literal `Decision` was needed anywhere**, so the register's *"first story that reconstructs a `Decision` outside `decide`"* clause is ANSWERED and still UNMET. `0003_resolver_guards.sql` installs the three guards the first writer owes; its foreign key reds **in two waves — 12 tests in `repo.rs`, then 2 more (one in `main.rs`) that are invisible until the twelve are fixed**, because a failing test rolls back and leaves no link for the cleanup to trip over. **All 14 mutations ran against a live `mariadb:10.11.11` and every red assertion- or panic-carried, zero compiler-carried**; M2 turns the connected-components refutation into a measurement (`left: 1, right: 2`), M7 reds on `last_seen_at` rather than `first_seen_at`, and M4 on the evidence rather than the rule **5.9b is `done`** (PR #67 squash-merged 2026-08-04 as `94314e9` after a green CI run, code-reviewed first) (three layers, 2026-08-04; two against their own live `mariadb:10.11.11`, the Auditor re-executing the whole mutation pass): **2 arbitrations by Guy, 23 patches, 5 deferrals**, taking the workspace to **408 tests** and the mutation set to **18**. 🔴 **It refuted a headline in five documents, the commit subject included** — *"every red is ASSERTION-carried"* is FALSE for M6b (1 assertion, 3 `.expect()` panics), and the story's own T7 bullet had recorded the truth one section above; the driver read the carrier off the whole output, so a MIXED set collapsed to one label. 🔴 **Four behavioural defects, three found separately by two layers**: `placement_decision` tested containment on ONE witness while its sibling filtered correctly twelve lines below — universe missing only `(1,2)` and observations 1 AND 2 abstained; a multi-key abstention collided on `ABSTAINED_SUBJECT` and rolled the whole pass back; `abstention_for` had **no observable effect** (its body replaced by `nothing_was_evaluated()` left every test green, a `Neutral` verdict's evidence being empty and the vector unpersisted); and a repeated MAC-less `obs_id` wrote two colliding links. 🔑 **Guy's arbitrations**: an observation abstains **at most once** whatever the key count — an abstention row names no key, so the two rows would be identical but for their id — and the smallest-other-`ObsId` witness convention is kept and now **measured on a group of three**, every earlier test having used a group of two where the two formulas coincide. Also caught: the reference-scale pass and its wall-clock existed in no test (now **44 850 pairs, 300 interfaces, ~73 ms**), AC7's *"shown to bite"* had no record (the gate now names `0003…sql:48`), §9 did not apply its own *"a divergence is a FINDING"* rule to its own three divergences, and a doc cited a test that had been invented.. ⚠️ The pass is **NOT idempotent** — twice over one slice is `Err(Constraint("unique"))` and a full rollback, which is story 5.11's — and it is deliberately **not wired into `main.rs`**. |
| **E6–E23** | backlog |

Live status is `_bmad-output/implementation-artifacts/sprint-status.yaml`, not this file.

**What exists today:** a three-crate workspace that builds and ships. `cargo xtask ci` runs **six**
real gates — dependency frontier (D47), DDL binary collation (D64), retired vocabulary (D65), the
fixture corpus lock (both directions: edited AND orphan), the file-size ceiling (D56b) and
`float-free` (D13, story 5.4b) — plus the informational `views-hash` staleness check, which reports
`ℹ STALE` and exits 0 by design. _(This sentence said "four" until 2026-07-30; `file-size` and
`float-free` had shipped without it being updated.)_ `opencmdb-core` holds the domain
(`Observation`, `Fact`, `Capabilities`, the closed `ConnectorError` taxonomy, the `Connector` trait
and its consumer-driven contract test). `opencmdb-bin` holds everything touching the outside world:
MariaDB pool + migrations, axum/askama/HTMX pages, an ARP/ping connector, the fixture reader,
`FixtureConnector` and — since story 5.7 — `l1_runner`, the release gate's answer producer. Test
counts on `master` after **5.9b** merged: **203 (bin) + 159 (core) + 46 (xtask) = 408**, measured
with `cargo test --workspace`; the implementation reached 402 and the code review took it to 408.
`master` after 5.9 merged carried **383** (181 + 156 + 46). `master` after 5.8 merged carried **367** (165 + 156 + 46); after 5.7, **352**
(153 + 153 + 46). _(This line still said 367 «after 5.8 merged» on 2026-08-04, two stories later —
the same staleness its own parenthesis below was written to stop.)_ _(This line read "86 + 46 + 38" until
2026-08-01 — a snapshot of Epic 3's close, left standing through all of Epic 4 and six Epic-5
stories, in the file BMad auto-loads first. The paragraph below carries the per-story history; this
one carries the CURRENT number and nothing else.)_

**Epic 4 closed on 2026-07-25.** The corpus is committed and locked: **25 artefacts, 24 traps
across nine families** (randomized-mac, multi-nic, shared-hardware-vm, cloned-mac, dhcp-churn,
vrrp-virtual-mac, hostname-collision, docker-veth, hostname-absence — each in positive AND
negative form, each naming the RULE rather than the outcome), the scoring algebra, the metrics
harness, the trap runner, the reality-debt register, and the wire-format spec. Test counts on
master at Epic 4's close: **119 (bin) + 86 (core) + 42 (xtask)**; **281** on the story-5.4b branch
on master after 5.4b's merge (**135 bin + 100 core + 46 xtask**) — story 5.1 added the two
corpus-wide walks, story 5.2 the trap-text scan, the `raw` scan and the scanner's six closed
evasions, and story 5.2b five byte-pin tests over six of the eight streams that had none (the four
unpinned families plus `example-traps.jsonl`, `dhcp-churn`'s existing pin extended) together with
the trap→`obs_id` binding pin; story **5.3** the four tests of the identity engine's own abstention
vocabulary (`IdentityAbstentionCause`, two variants, in the new `opencmdb-core/src/identity/`) — a
TYPE and no engine; story **5.4** four more, for the engine's RETURN type — `Verdict` (D13's five),
`RuleVerdict { rule, verdict, evidence }`, `RulesetVersion`, `Conclusion` and `Decision` — still with
no algebra, no rule and no producer; and story **5.4b** the ALGEBRA itself — `decide` as a TOTAL pure
function over D13's table plus the class it leaves uncovered — with six core tests, plus four `xtask`
tests for the sixth gate, `float-free`. Story **5.5** then added **25** core tests for the L1 join and
the first firing rules, taking the workspace to **309 (135 bin + 128 core + 46 xtask)** after its code review: the join on
`(l2_domain, mac)` returning a `BTreeMap<_, BTreeSet<ObsId>>`, the corpus's two rule ids
(`l1-exact-mac` -> `Decisive` on at least one shared key, `l1-distinct-mac` -> `Disqualifying` when
they share none, `Neutral` when either side has no MAC), and `decide_pair` — **the first caller of
`decide` outside its own tests**. Story **5.6** then added **19 core + 4 bin** tests for the blocker
— `candidates()`, the `CandidatePair` that refuses the self-pair, and D13's recall floor as an
integer in per-mille, with the corpus half of the assertion in `fixtures.rs`'s test module because
D47 forbids core to read files — taking the workspace to **333 (139 bin + 148 core + 46 xtask)**
after its code review, whose nine patches added one test (an observation with NO fact at all: a
`facts.is_empty()` narrowing inside `candidates` was measured GREEN across all 332).
Story **5.7** then added **12** tests (7 in the new `l1_runner`, 5 in `trap_gate`) for the producer
and the report it makes possible, plus **5** in `score.rs` for the `Decision → Outcome` mapping,
taking the workspace to **350 (151 bin + 153 core + 46 xtask)** — and **352 (153 + 153 + 46)** after
its code review, whose eleven patches added two `l1_runner` tests for the two guards the committed
corpus cannot exercise: a trap naming the SAME observation twice (which merged on `l1-exact-mac`,
an observation sharing every key with itself — the self-pair story 5.6 had closed in the TYPE and
`answer_trap` re-opened by never calling `read_traps`), and a trap naming THREE, where relaxing the
pattern to `[a, b, ..]` had left all 350 green.
Story **5.8** then added **12** tests (**9** in `opencmdb-bin` — 6 in `trap_gate` for the fourth
bucket, the per-column arithmetic, the two render halves and the empty-map guarantee, 3 in
`l1_runner` for the 8/2/1 split and the cross-file id guard — and **3** in `score.rs` for the new
vocabulary), taking the workspace to **364 (162 bin + 156 core + 46 xtask)** — and **367 (165 + 156
+ 46)** after its code review. Its nineteen patches added **five** test functions and removed two,
and the net **+3 is entirely in `trap_gate`**: `unaccounted_counts_what_no_producer_spoke_about`
(the arithmetic asserted nothing before the patch),
`the_per_column_split_of_the_bucket_is_three_five_three` (the partition assertion now READS THE MAP —
run on both sides, the map-reading form reds at `23 != 24` under M5b where the literal-oracle form
stays green), and `an_unanswerable_answer_for_an_unknown_trap_is_refused_too`. In `opencmdb-core` the
count is UNCHANGED at 156: two near-vacuous assertions were replaced rather than added to — one in
`score.rs`, one in `trap.rs`, the latter for `TrapError::RuleMalformed`.
**The corpus harness is now fed AND the gate now falls** — `l1_runner` fills `score_corpus`'s map
with 13 real engine answers and 11 named refusals, and `passed()` is **false** until Epic 6. But
**the blocker still has no production caller** (it is reached from its own tests and
from `fixtures.rs`'s test module only), and that is a DECISION rather than an omission: a trap NAMES
the pair it puts under judgement, so the runner has nothing to generate. `Verdict::Supports`/`Opposes`
still have **no producer** (Epic 6), and `ScoredRecord::verdict_vector` is still provably empty —
**11 of the 11 replay streams a trap names carry no `capability` control record**, so producing one
would mean inventing a capability snapshot for all 24 traps (D36 in reverse).
**5.2b's trap→`obs_id` binding pin** closed a hole measured TWICE: exchanging two `observations`
vectors in `cloned-mac.toml`, and later in `hostname-collision.toml`, each made the corpus DEMAND a
false merge while the whole suite stayed green. **All 24 committed traps across all ten trap files
now have their `observations` vector, `Expectation` and `family` pinned** — the scope grew from
five families to ten on the story's own code review, which counted 14 of 24 and measured the
residue. ⚠️ `partial-then-failed.jsonl` and `capability-downgrade.jsonl` still have no value pin
(registered, owner named). **Three** deliberate privacy-walk amendments have
landed: two ADMITTING a byte shape with its family (the IANA VRRP MAC range in 4.14, the
honestly-empty hostname in 4.17) and one REFUSING one with no family behind it (multicast MACs, 5.2
— measured against all 39 committed MACs first, none of which has the I/G bit set, so it reddened
nothing). Each was proven red at its boundary. **Story 4.19 was SPLIT at closure**: 4.19a (the drift-surface
record and the binding layer charter) shipped in `fixtures/scenario/wire/README.md`; **4.19b
(the mutation generator, ~30 generated fixtures, expected parse outcomes) moved to Epic 11** —
expected outcomes for an error taxonomy that does not exist would be written from belief (D45).
The promise is carried by **GitHub issue #34**, the wire charter, `CONSUMER PENDING` markers in
the MANIFEST, dated notes in `epics.md`, and
`_bmad-output/implementation-artifacts/epic-4-correct-course-2026-07-25.md`. ⚠️ **The one
milestone task now DUE and not done: regenerating `architecture-views.md`** (its `sourceSha256`
has not matched since commit `da23f9f`, which predates Epic 4 — the staleness is inherited, not
caused by this epic; the end of Epic 4 is the milestone the project chose for it, and it passed).
**Now tracked as GitHub issue #50** rather than in these notes alone. The drift was MEASURED on
2026-07-29 and is smaller than the hash suggests: **two commits, 59 insertions / 11 deletions**, and
neither touches a decision body, a dissent or a renunciation — one corrects the source-tree diagram
(D64 fallout), the other adds a D65 post-completion section. ⚠️ **There is no generator** — `cargo
xtask` exposes only `ci`, whose `views-hash` check merely VERIFIES the frontmatter hash. The bulk of
the real work is re-checking that the 883 view file's line citations still resolve, since the source
grew by 236 lines.

**The fixture corpus was the epic's centre of gravity.** `fixtures/` at the workspace root is a
SPEC, not test data, locked by `MANIFEST.toml` in both directions. A replay stream carries
observation lines (`obs_id`) and control records (`record`) — `failure` ends a poll with a
`ConnectorError`, `capability` changes the source's descriptor mid-stream and the poll continues.
Epic 4 builds the metrics harness and the trap corpus **before** the identity engine, on purpose:
*"a metric written after the engine is bent to fit the engine."*

**Two standing lessons, learned the hard way and worth carrying:**
- **Name the test or command behind every claim.** Four consecutive completion records over-claimed;
  reviews caught each. Write the weaker true sentence instead.
- **A comment asserting a checkable property gets checked.** Story 2.2 shipped a comment saying a
  guardrail existed; it did not, and the claim survived until story 4.5a's review added a variant and
  watched the build succeed. `deferred-work.md` is the register — **append to it, never rewrite a
  bullet**.

**Issue tracking:** GitHub Issues on `guycorbaz/opencmdb` is the single source of truth for bugs and
change requests outside the BMad story flow. Open at this date: #1 (Docker connector), #2 (frontier
gate blind spots), #3/#4 (scan CIDRs from the UI), #11 (scan tuning in the UI), #12 (no admin
surface), #13 (QR-code equipment labels, Brother QL-820NWBc — post-v1.0).

## Planning status (2026-07-17)

Complete and saved in `_bmad-output/planning-artifacts/`:
- **Product Brief** (`product-brief-opencmdb.md` + `-distillate.md`), **competitive analysis**.
- **PRD** (`prd.md`) — 53 functional requirements (FR52 struck, number retained), 31 NFRs (NFR16 struck
  by D64, number retained), 7 user journeys, phased scope.
- **UX Design Specification** (`ux-design-specification.md`) — full 14-step spec.
- **Architecture** (`architecture.md`, **5123 l.** as of 2026-07-22) — **complete, 8/8. Decision register D1–D66, feedback
  F1–F59.** Readiness: **NOT READY, and deliberately so** — the open gaps are named and countable.
  **Start at its Decision Index (§ near the top, 74 entries, line numbers verified): scan it BEFORE opening
  a question, not after.** Three times in 24h this register "discovered" a hole where its own rules were
  already standing (F56) — D57 sat on the critical path for a day while D25/D21 already answered it.
- **`architecture-views.md`** (~880 l.) — **CROSS-CUTTING VIEWS, derived, never edited by hand.** Gathers
  what the source scatters and nobody can reconstruct by scanning: **every named renunciation, every
  measured number, every recorded dissent, every author amendment, every piece of named theatre, everything
  still open.** It POINTS, it does not restate. **It carries `sourceSha256` — if it no longer matches
  `architecture.md`, the file is STALE and must not be trusted.** Never apply a decision from it.
- Theme visualizer: https://claude.ai/code/artifact/b598a17b-5303-4c32-bb58-f7a79fbb8182

> **⚠️ Read `prd.md`'s `editHistory` frontmatter for the true state of the requirements — NOT the F-tables
> inside `architecture.md`. Start any question at the Decision Index near the top of `architecture.md`
> (75 entries, line numbers verified) — scan it BEFORE opening a question, not after (F56).**

**Planning is DONE. All architecture decisions are closed (D1–D66).**

_The five-step implementation sequencing this section used to carry (workspace skeleton → the D65
gates → D64's DDL grep → story 1 → epics) is **all complete**, through Epic 3 and the v0.1 release.
It was removed on 2026-07-22 rather than left standing: a resume point that describes work finished
weeks ago sends the next reader to the wrong place. See "Where the code is" at the top for the real
one. The two sqlx traps it warned about were both real and are both handled in the shipped code:
`Reads` is two traits, and `query*()` takes `impl SqlSafeStr` (dynamic SQL needs `AssertSqlSafe`)._

> **Note on `architecture-views.md`:** it is STILL stale, deliberately — `cargo xtask ci` reports
> `ℹ views-hash STALE` and exits 0, because the mismatch IS the staleness signal. Regenerate it at a
> MILESTONE, not per-decision or per-story. **Do not regenerate it inside a story**; several story
> files say so explicitly. **Tracked as GitHub issue #50**, which carries the measured scope.
> _(This note named "the end of Epic 4" as the natural milestone. Epic 4 closed on 2026-07-25 and
> the regeneration did not happen, so that sentence had become a plan rather than a fact — which is
> exactly why it is now an issue and not only a note.)_

## Locked, non-negotiable decisions (do not reopen)

- **Language/runtime:** Rust, single binary.
- **Database:** **MariaDB 10.11+ is the ONLY supported engine** (Synology-native, included in DSM's
  automatic backups). CI is pinned to `mariadb:10.11.11` — the exact DSM 7 package — so **dev = CI =
  prod**. **SQLite: NOT supported, and it is a REFUSAL, not a deferral ("SQLite later" is banned in
  writing). MySQL: NOT supported** — a different product from MariaDB, and we do not claim what has no
  CI. **PostgreSQL: not supported at MVP**; re-opened as a *possible* future addition, and it is not
  free — the repository trait gets audited BEFORE any such port. **Comparison and normalization never
  descend into the engine** (a correctness rule, not a portability one: identity is the product), held
  by binary collation on every text column + a CI grep over the DDL.
  _(D64, 2026-07-17. This line previously read "SQLite (small) and MySQL/MariaDB (larger) … SQLite-first,
  MariaDB activated later within MVP" — **stale twice over**: D1 had already made MariaDB day-1.)_
- **Deployment:** Docker (Synology Container Manager) priority; native binary also supported.
- **Discovery:** first-class **zero-privilege UniFi connector** + generic ARP/ping scanner
  (NET_RAW → ping-only fallback). Connectors isolated behind a Rust `Connector` trait, contract-tested.
- **Web stack:** HTMX + Askama templates + Tailwind (standalone CLI, no Node; assets via `rust-embed`,
  generated CSS committed). **Polling, not SSE, at MVP.** Internal tokio scheduler (no cron/Redis/workers).
- **UI:** bilingual EN/FR; **docs English-only**. Dark theme default. WCAG 2.1 AA on key views.
- **Audience:** unified (advanced home-labber = SMB-without-IT). Single admin at MVP;
  **multi-user-ready schema** from day 1 (multi-user role = Growth).
- **Delivery:** phased MVP / Growth / Vision. Reference scale: ~300 hosts / 36 subnets on an
  x86 Synology Plus-class NAS.

## Key design pillars for the Architecture phase

- **Composite device identity** (MAC + hostname + IP/DHCP history + connection topology; service
  fingerprint = Growth) — NOT raw MAC. This is the highest-leverage/riskiest problem; precision/recall
  on labeled fixtures gates release.
- **Linked-never-merged** observed/declared model — reconciliation *links*, never overwrites declared.
- **Commit state machine:** `in_queue → pending_commit(server deadline) → committed | failed`;
  **Undo returns to `in_queue` — it is not a failure branch** (*an undo is not a failure: the operator
  changed his mind*). A scan touching a `pending_commit` item is quarantined (`superseded_by_pending`);
  the **server timer is authoritative**; transitions serialized per `item_id`; idempotency via 409/ETag.
  _(Vocabulary corrected 2026-07-17 — F59. This line carried `Accept-as-declared`, `pending_accept` and
  `reverting`: all three RETIRED on 2026-07-16, and `accept-as-declared` is on the denylist. The state was
  named after ONE gesture when the protocol covers document / accept-gap / exclude alike.)_
- **Triage gestures (canonical, binding):** **document** (EN) / « Merger » (FR UI) — closes the gap, field
  by field, and carries the amber accent · **accept-gap** / « Accepter l'écart » — *seen, not yet decided*;
  the gap stays OPEN and it is deliberately NEUTRAL, never amber · **exclude** / « exclure » · attach ·
  create · snooze. **Retired, denylisted: `accept-as-declared`, `revert`, `ignore`, and `merge` in ENGLISH
  only** (the pillar *linked, never merged* is intact; the FR UI verb stays « Merger »). *You document a
  VALUE; you accept a GAP.*
- **Orthogonal `source_state ∈ {live, blind}`:** when a source is blind, freeze last-known state,
  suppress observation-derived alerts, and **never fabricate divergences**.
- **Optimistic UI** (client-instant, server-reconciled) with **explicit focus management on every
  HTMX swap** (accessibility requirement #1).
- **Security/threat model:** protect against a stolen DB/backup + unauthenticated network access
  (not local root); encryption key kept separate from the DB volume; API-key rotation + encrypted
  secret backup at MVP; TLS via reverse proxy (deployment concern).

## Working conventions

Guy works in **French**; decisive; uses BMad **Party Mode** heavily at each step; guiding mantra
**"on affinera à l'usage"** — freeze durable *principles*, calibrate *details/values* in V1.

**Story flow, as actually practised:** `create-story` → **`validate` (MANDATORY since 2026-07-26 —
Guy's decision at Epic 4's retrospective; two fresh-context agents, fact-check + gap-hunt)** →
`dev-story` → `code-review` → next story. It is not optional and the story template's
"Validation is optional" banner is wrong: measured over the 9 Epic-4 stories that had it, validation
produced **6 HIGH findings on 4 stories**, two of which would have shipped a trap that passed for the
wrong reason. Stories are sliced FINE: prefer many small ones over few big ones, and split a story
when its halves turn out not to be variants of one idea (4.5 → 4.5a/4.5b).

**Commits: one per story, on a BRANCH → PR → green CI → squash merge. Never straight to `master`**
(adopted mid-Epic 4 at PR #15 and held since: 22 PRs, zero direct pushes, 47/47 green CI runs).
`enforce_admins` is false, so honouring this is on the author, not on GitHub. Message names what
changed and what was measured. Run the full local gate before pushing — `cargo fmt --all`,
`cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo xtask ci`
— because Epic 3's retrospective recorded four CI-only failures from skipping exactly that.
_(This paragraph said "directly on `master` (no PR flow)" until 2026-07-26 — true of Epic 3, false
for all of Epic 4. Found in Epic 4's retrospective.)_
⚠️ `DATABASE_URL` is usually unset locally, and the MariaDB-backed tests `return` early and pass
either way: a green suite says **nothing** about the database.

⚠️ **The local test suite has shown unexplained non-determinism** (8 failures across 5 runs,
identical sha256, clean `git status`, then 15+ green — stories 4.15/4.17). The recorded cause
("Synology Drive replaced the corpus with a stale server state") is **REFUTED by measurement**: Drive
has never synced this tree — the repo was created five days after Guy deliberately stopped syncing
`devel`, and Drive's history holds zero rows touching `opencmdb`. **The cause is OPEN**; CI (clean
checkout) has never reproduced it. Do not re-adopt the refuted explanation.
**It RECURRED on 2026-08-02** on `master` at `d47631b`, and the measurement is on **issue #38**:
2 red runs out of ~11, **on two DIFFERENT tests** (one of them
`fixtures::tests::a_decision_carrying_an_abstention_cause_is_refused`), a green run between them,
then **8 consecutive green**. Clean tree; `cargo xtask ci` re-verified all 25 fixture sha256s green
*during* the red runs; CI passed on the same commit in 55 s. A second hypothesis — a scratch-path
collision between concurrently-running tests — was raised and **also refuted**: the six
`write_traps` tags in `fixtures.rs` are distinct and `scratch_dir` embeds `std::process::id()`. What
the recurrence adds is that the failure **moves between tests**, which argues against any single
test being the defective one.
