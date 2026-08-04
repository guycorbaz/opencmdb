# Story 5.9b: The engine resolves a set of observations and writes the links it derives

Status: done

<!-- ⏳ VALIDATION HALF DONE (2026-08-04). Two fresh-context agents are required before `dev-story`
     — Guy's decision at the Epic 4 retrospective (2026-07-26). The template's "Validation is
     optional" banner does not apply here.

     ✅ FACT-CHECK DONE: 78 claims measured, 69 true, **6 false and 6 gaps**, all applied below and
     each re-measured independently before being accepted. 🔑 **All 44 line citations were correct
     and every count was exact** — the defects were entirely in QUOTED TOKENS, a CORPUS claim, and
     the AC↔task↔mutation seam:
       • the persisted abstention token is `absence_of_proof`, not `AbsenceOfProof` (the variant
         name, which appears in no committed byte — an assertion against it reds, and the tempting
         fix breaks a pinned test);
       • AC3 allowed `rule_id ∈ {l1-exact-mac, l1-distinct-mac}` where **`l1-distinct-mac` is
         unwritable by three independent steps** — an assertion over the set would have passed
         while measuring nothing;
       • 🔴 **`multi-nic` is NOT the multi-MAC shape**, and no committed observation carries more
         than one MAC (max 1 over all 13 streams). Three documents use that family as evidence for
         a case it does not contain. The arbitration stands on its own measurement; the supporting
         sentence was decoration;
       • three mutations (M1, M6, M8's target) were aimed at assertions **no prescribed test
         contained** — the same shape the last three reviews caught — and one guard had no mutation
         at all (now M13);
       • two register entries are owned by CONDITION rather than by name, and decisions 5 and 6
         meet their clause: nine entries became **eleven**.

     ✅ GAP-HUNT DONE: the story was BUILT end to end in an isolated worktree against a live
     `mariadb:10.11.11`, reaching **398 tests** with six green gates and `fixtures/` untouched, and
     every mutation was executed. **7 HIGH, 5 MEDIUM, 7 LOW**, all applied. Two of its findings were
     arbitrated by Guy and became **decisions 5 (`decide_singleton`) and 10 (the pass is not
     idempotent)**; decisions 11 and 12 follow from two more. §9 carries the measured end state and
     the eight things building it found that reading it could not.
     🔴 The four that would have cost the most: **`Utc::now()` does not compile** in this workspace,
     so mutation M6 was not executable and half of AC6 was enforced by `Cargo.toml` rather than by
     this story · **running the pass twice is `Err(Constraint("unique"))` and a full rollback**, so
     AC4 was unbuildable as written · **M2 and M12 were measured NO-OPS** against their named targets
     (M12 left all 397 tests green) · and AC7's foreign key reds **14 tests across two files**, not
     the eight the register claims.

     ✅ **Both layers are in and applied. The story is ready for `dev-story`.**

     🔑 THE GAP-HUNT MUST RUN WITH A LIVE DATABASE. Measured at story 5.9's validation and again
     at this contexting: `DATABASE_URL` is UNSET on this machine and the suite reports
     181 / 156 / 46 green **identically** with and without it. A prove-to-red pass run without a
     database records green mutations over guards that were never executed. §7 has the exact
     `docker run` — host port **13306**, never 3306.

     🔑 ASK THE GAP-HUNT EXPLICITLY: *"does each prescribed mutation actually red, and is the red
     carried by the assertion I named?"* On stories 5.5, 5.6, 5.7 and 5.9, **every HIGH came from
     the agent that COMPILED the story and none from the fact-check**, and on 5.7 and 5.9 the
     HIGHs were **no-ops** — mutations that reddened nothing, and assertions that existed in no
     prescribed test. -->

## Story

As the operator,
I want a scan's observations turned into interfaces and identity links by the engine, in one
deterministic pass,
so that what the engine decides is written down rather than recomputed, and story 5.10 has
something to purge.

**This story is the first production caller of `identity::blocking::candidates` and the first
cross-crate caller of `identity::l1::join`.** Story 5.9 shipped the schema and the adapter and
deliberately called neither; the register has owned that residue as *"story 5.9 or Epic 6, whichever
first hands the blocker a set of observations"* since story 5.6. This story is it.

**What this story does NOT do**, so the boundary is explicit and not discovered at review:

- it does **not** wire the resolver into `main.rs`'s startup scan (decision 3). The resolver is
  production code with named consumers in stories 5.10 and 5.11; the shipped binary's behaviour is
  unchanged, and no deployment starts writing links it has no page to display (5.14) and no purge
  to remove (5.10);
- it does **not** create `device`, the `entity` supertype, or a `state` column — Epic 6 and the
  lifecycle epic, all three already registered by story 5.9;
- it does **not** implement an `l2-*` rule, and therefore **does not empty the trap corpus's
  unanswerable bucket**. `passed()` stays **false** with **11 unanswerable** — that is story 5.8's
  deliverable and `epics.md:1560`'s *"closed by Epic 6"*. **If the committed gate turns green,
  something is wrong; report it as a finding rather than celebrating it**;
- it does **not** display anything. FR16's rendering is **story 5.14**;
- it does **not** touch `identity::blocking` or `identity::cascade`, and it changes **one** thing in
  `identity::l1`: a new `decide_singleton` (decision 5), added because the validation measured that a
  singleton group has no `Decision` and every alternative route was worse. `join`, `decide_pair` and
  `verdict_for_pair` are untouched. **Any FURTHER change to the engine is a FINDING;**
- it does **not** re-run the corpus harness, `score_corpus` or `l1_runner`. A trap NAMES its pair;
  this pass generates its own. They are different callers of the same engine and neither becomes
  the other.

**Nothing under `fixtures/` moves.** No artefact bytes, no `MANIFEST.toml`, no re-hash. If a step
appears to require re-authoring a committed artefact, **STOP** — that is a finding.

**`architecture.md` is NOT edited** (issue #54; a correction to a decision body is a milestone act).
**`architecture-views.md` is NOT regenerated** (issue #50).
**`epics.md` is NOT edited** — verify-only. §2 records a correction to its AC1 with the measurement;
story 5.8's precedent for editing it does not apply, because 5.8 was *given* its correction by 5.7
with a named owner, and this one is found at this contexting.

⚠️ **Branch from `master`.** Measured at contexting: `master` is at **`47bdca2`**, the tree is
clean, no branch of any earlier story survives locally or on `origin`, and
`cargo test --workspace --locked` reports **383 tests** — **181 bin + 156 core + 46 xtask** (plus
one ignored doc-test). `cargo xtask ci` reports **six green gates**, `float-free` walking **4**
files, `file-size` largest **1136**, and `views-hash ℹ STALE` exiting 0 by design.

---

## What this story inherits, measured rather than assumed

Everything below was measured at contexting on `47bdca2`, by reading the tree and running the
gates. **The dev re-derives none of it; a surprise reads as a FINDING.**

### 1. The three organs that exist, and what each one will and will not answer

| organ | signature | what it answers | what it does NOT |
|---|---|---|---|
| `identity::blocking::candidates` | `(&[Observation]) -> BTreeSet<CandidatePair>` [`blocking.rs:171`] | every unordered pair of **distinct** `obs_id`s — TOTAL by decision | reads no `Fact`, calls no rule, proposes no self-pair (`CandidatePair::new(a, a)` is `None`) |
| `identity::l1::join` | `(&[Observation]) -> BTreeMap<(L2DomainId, MacAddr), BTreeSet<ObsId>>` [`l1.rs:172`] | the **set of interfaces at L1** — one key, one interface, the set being the observations on it | mints no `InterfaceId`, reads no clock, returns no rule and no evidence |
| `identity::l1::decide_pair` | `(&Observation, &Observation) -> Decision` [`l1.rs:305`] | the rule, the verdict and the evidence for **one pair** | returns **no interface**, and does not refuse the self-pair — that exclusion lives in `CandidatePair::new` |

🔴 **`decide_pair` cannot name an interface, and that single fact shapes the whole pass.** A
`Decision` carries `conclusion`, `verdict_vector` and `ruleset_version` [`cascade.rs:349-357`] and no
key. So the pair verdict can say *"these two are on the same interface"* and can never say
*which*. Decision 2 is the consequence.

**What `decide_pair` can conclude at L1, measured over `decide`'s table:**

| the pair | rule | verdict | conclusion |
|---|---|---|---|
| shares at least one L1 key | `l1-exact-mac` | `Decisive` | `Match` |
| both carry a MAC, shares none | `l1-distinct-mac` | `Disqualifying` | `NoMatch` |
| either side carries no MAC | `l1-exact-mac` | `Neutral` | `Abstained { AbsenceOfProof }` |

🔑 **`Conclusion::Ambiguous` is NOT reachable at L1**, and that is derivable rather than hoped:
`decide` reaches `Ambiguous` only through `Supports` or `Opposes` [`cascade.rs`, D13's table], L1
emits neither, and three documents already record that those two verdicts *"have no producer until
Epic 6"*. **Therefore the only abstention cause this pass can write is `AbsenceOfProof`.** Decision 6
turns that into a guard rather than leaving it a happy accident.

### 2. 🔴 `epics.md`'s AC1 is falsified by the code it describes — the correction, and Guy's arbitration

`epics.md:1600` says: *"each observation carrying a MAC lands on exactly ONE `interface` — the one
its `(l2_domain, mac)` key names"*.

**Measured:** `join` is `for observation { for key in keys_of(observation) { … } }` [`l1.rs:174-178`]
and `keys_of` returns a `BTreeSet<L1Key>` with **one entry per `Fact::Mac`** [`l1.rs:128-152`]. An
observation carrying two MACs therefore lands on **two** interfaces. `l1.rs:186` says so in prose
(*"An observation may carry several MACs"*), and story 5.9's uniqueness key was widened for exactly
this reason — the FIRST of its two widenings [`0002…sql:74-77`]; the second had an unrelated cause
(`valid_to` in the key constrained HISTORY, [`0002…sql:48-54`]).

🔴 **But the corpus does NOT contain this shape, and the story must not pretend otherwise.**
Measured at the validation over all 13 committed replay streams: the maximum number of `Mac` facts
on one observation is **1** (43 observations carry one, 6 carry none). **`multi-nic` is not that
shape**: it models a multi-NIC host as two SINGLE-MAC observations, and both its poles expect
`l2-uplink-agrees` / `l2-different-switch` [`fixtures/scenario/traps/multi-nic.toml:18,29`] — L2
rules, so it sits in the eleven-unanswerable bucket and L1 never answers it. The multi-MAC
observation is real **in the type** (`keys_of` returns a set; `l1.rs:471`
`an_observation_carrying_two_macs_joins_two_groups` exercises it synthetically) and story 5.9
measured a synthetic one being refused its second link. **Every test of it in this story is
therefore synthetic by necessity, not by preference** — and the sentence *"`multi-nic` is a
committed trap family"*, which three documents use as evidence for the multi-MAC case, is
decoration on the arbitration rather than support for it. The arbitration stands on its own
measurement.

**Guy's arbitration at this contexting: AC1 widens to *one interface per L1 key the observation
carries*.** The word "exactly" survives and moves: exactly one interface per key, exactly one
current link per `(observation, subject)`. That is AC2 below, and it is the same sentence story
5.9's `UNIQUE (observation_id, current_subject)` already enforces in the schema.

⚠️ **`epics.md` is NOT edited.** Story 5.8 lifted the verify-only rule once, because story 5.7 had
handed it a correction with 5.8 named as owner; here the correction is found at contexting by the
same story that will apply it, so there is nobody to hand it to and no precedent to invoke. The
correction lives in this file and in `deferred-work.md`, with **Epic 5's retrospective** as the owner
of the `epics.md` edit.

### 3. 🔴 The mechanism — who NAMES the interface, who JUSTIFIES the placement

Arbitrated by Guy at this contexting, after two alternatives were stated and one was refuted.

**The unit of work is `join`'s `(key, group)` pair. The key NAMES the interface; the blocker and
`decide_pair` JUSTIFY each placement.** Concretely, per group:

1. `candidates(observations)` is called **once**, over the whole slice, and its result is the
   universe — nothing is judged that it did not propose.
2. For each observation `o` in the group, a **witness** `w` is chosen deterministically (§decision
   4), the pair `(o, w)` is confirmed present in the universe, and `decide_pair(o, w)` supplies the
   link's **rule** and **evidence**.
3. The interface is found by its key or minted; the link is written with that rule and evidence.

**Why not the obvious alternative** — build the groups from the `Match` pairs alone, as connected
components:

> ⚠️ **Derived, and the story requires it to be MEASURED before it is quoted as fact** (task T7).
> `verdict_for_pair` uses an **existential** quantifier — *"the pair shares an interface when it
> shares AT LEAST ONE key"* [`l1.rs:186-190`, and the same doc says the universal reading *"would
> make a multi-NIC host oppose itself"*; the code is `keys_a.intersection(&keys_b).next().is_some()`
> at `l1.rs:265`]. So A shares `k1` with B and B shares `k2` with C makes A–B and
> B–C both `Match`, and a connected-component grouping merges A with C **although they share no
> key** — two genuinely distinct interfaces fused. ⚠️ **The shape is real in the TYPE and absent
> from the CORPUS** (§2): B must carry both `k1` and `k2`, and no committed observation carries two
> MACs. So the refutation can only be measured synthetically — which is what AC8 and mutation M2
> require, and why neither may be softened into prose.

**Why the blocker is not bypassed.** `candidates` is TOTAL — every unordered pair of distinct ids —
so "the universe" and "all pairs" coincide today. That does not make the call decorative: it is
where the universe is DEFINED, and `epics.md:1604` requires the pass to be *"candidate generation
(blocking) -> verdicts -> three-way decision"* in that order, D13's own [architecture.md:931]. A
pass that read `join`'s keys and never asked the blocker would be correct today and would silently
stop being correct the first time the blocker excludes anything — which F17's `dormant` already
plans to make it do [architecture.md:1205].

### 4. The tree this story extends, measured

| what | where | size |
|---|---|---|
| the MariaDB adapter — the only place SQL against the domain tables is written | `crates/opencmdb-bin/src/repo.rs` | 1886 lines (657 code + tests from `:658`) |
| the migrations | `crates/opencmdb-bin/migrations/` | `0001_initial.sql` (34 l., `declared_attribute` + `observation_record`), `0002_interface_and_identity_link.sql` (118 l., `interface` + `identity_link` + `link_candidate`) |
| the L1 engine | `crates/opencmdb-core/src/identity/l1.rs` | 897 l. — `join`, `decide_pair`, `L1_EXACT_MAC`, `L1_DISTINCT_MAC`, `CURRENT_RULESET_VERSION` |
| the blocker | `crates/opencmdb-core/src/identity/blocking.rs` | 661 l. — `candidates`, `CandidatePair`, the recall floor |
| the abstract persistence contract, sqlx-free | `crates/opencmdb-core/src/repo/mod.rs` | 66 l. — `WriteRepository::transact`, `WriteUnit`, `RepositoryError` |
| the ingestion path a future story would wire this into | `crates/opencmdb-bin/src/main.rs:215-252` | holds `sink.observations` in memory, then inserts them **one per transaction** |

**The adapter idiom, which this story follows and does not reinvent** [`repo.rs`]:

- **query bodies are free functions generic over `sqlx::Executor`** (D49), written **once**, called
  by the read side with the pool and by a unit of work with the transaction connection;
- **static SQL, bound values** (D48). `sqlx` 0.9 rejects `sqlx::query(&format!(…))` **at compile
  time** — *"dynamic SQL strings should be audited for possible injections"*;
- **`classify(sqlx::Error) -> RepositoryError`** is the ONE translation in the crate. It already maps
  `is_unique_violation` → `Constraint("unique")` and `is_check_violation` → `Constraint("check")`;
- **ids are bound as `String`** (D48: `CHAR(36) ascii_bin`), instants through `datetime_literal`
  [`repo.rs:330-333`] — **the single formatting site; do not invent a second**;
- **DB tests are gated and serialized**: `let Ok(url) = std::env::var("DATABASE_URL") else { return }`,
  then `let _guard = crate::DB_TEST_LOCK.lock().await;`, then `sqlx::migrate!("./migrations")`, then
  a `DELETE FROM …` per table to isolate the run.

**What the adapter does NOT have, and this story adds** — both are register entries this story owns:
there is **no `find_interface_by_l1_key`** and **no way to widen an interface's seen-window**.
`0002`'s own header says *"the re-run finds an interface by its key"* and no lookup exists.

### 5. The ELEVEN registered entries this story owns — nine by name, two by condition

`grep "Owner: story 5.9b\|RE-OWNED to story 5.9b" deferred-work.md` returns **10 lines, of which one
is a section preamble** — so **nine entries by name**. Counted, not quoted. 🔴 **Two more are owned
by CONDITION rather than by name** (#10 and #11 below), and the validation found them: their owner
clause is *"the first story that reconstructs a `Decision` from somewhere other than `decide`"*, and
decisions 5 and 6 meet it. **Eleven in total.**

| # | entry | where | what this story must do |
|---|---|---|---|
| 1 | `L1Key` is a bare tuple alias *(owner clause: "the first story to persist a key")* | `:2199` | **The condition is now MET** — this story holds `L1Key` values and persists them. Close it or refuse the newtype **with the measurement**, not silently. |
| 2 | the blocker still has no production caller | `:2203` | **CLOSE.** `candidates` is called by `resolver.rs`. |
| 3 | the universe is quadratic in the caller's slice, and nothing bounds it | `:2207` | **First story that can measure `n`.** Measure it; do not install a bound without a decision (decision 8). |
| 4 | no `find_interface_by_l1_key`, no window widening | `:2273` | **CLOSE** — both are needed by AC4 and AC6. |
| 5 | `identity_link.observation_id` carries no foreign key; adding it *"reds 8 tests"* | `:2278` | **CLOSE** via `0003` — and FIX the eight tests rather than dropping the FK (T5). |
| 6 | an `Ambiguous` abstention with ZERO candidates is storable | `:2291` | **CLOSE** with a guard tested DIRECTLY (decision 6). |
| 7 | `mac_canon`'s canonical form is asserted by a comment only | `:2304` | **CLOSE** via `0003`'s `CHECK (mac_canon = LOWER(mac_canon))`. |
| 8 | `rule_id = ''` satisfies the rule-XOR-cause CHECK | `:2308` | **CLOSE** — a guard in the writer AND the DDL echo in `0003`. |
| 9 | `count_identity_links` is `pub` with no caller and no test | `:2316` | **CLOSE** by giving it a PRODUCTION caller, or delete it. ⚠️ **The entry's premise is half stale**: it has had a test since story 5.9's own code review (`repo.rs:1810`, inside the CASCADE test). *"No production caller"* is still true; *"no test"* is not. Correct the entry rather than repeating it. |
| 10 | an incoherent `Decision` is buildable by struct literal *(owner clause: "the first story that reconstructs a `Decision` from somewhere other than `decide`")* | `:2187` | 🔴 **The condition is now MET, and this story is what meets it.** Decisions 5 and 6 both need a `Decision` the engine did not produce — see decision 5's ⚠️ below. Dispose of it with the measurement; do not carry it. |
| 11 | nothing enforces that a `RuleVerdict` built by struct literal leaves non-empty evidence | `:2195` | Same: its twin, met by the same construction. |

⚠️ **An entry whose condition is not met is ANSWERED, not CLOSED.** Six consecutive code reviews
have caught the opposite, and story 5.9's own review caught it four more times inside the commit
that claimed to enforce it. If #1's newtype is refused, say *"refused, and here is why"* — that is a
closure by decision; if the resolver turns out not to hold an `L1Key`, say *"the condition is not
met"* and re-own it.

### 6. 🔴 The purge-stability constraint story 5.10 inherits from THIS pass

Story 5.9 decided it in the schema; **this story is where it is either honoured or broken**, because
this is the first code that computes the values.

1. **`interface` rows are NOT purged and their `id` is stable.** 5.10 deletes *links*
   (`DELETE … WHERE decided_by='ENGINE'`). If a re-run re-minted an interface's UUID, every
   reproduced link would carry a different `interface_id` and 5.10 could never pass. **This is why
   `find_interface_by_l1_key` is an AC and not a convenience.**
2. **Every instant this pass stores is DERIVED from the observations, never from the clock.**
   `interface.first_seen_at` / `last_seen_at` are `min`/`max` of the group's `observed_at`;
   `identity_link.valid_from` is its own observation's `observed_at`; `valid_to` is `OPEN_END`.
   *"The engine never touches the clock"* [architecture.md:3364].
3. ⚠️ **`insert_declared_attribute` uses `NOW(6)`** [`repo.rs:125`]. That is a DECLARED row authored
   by a human and **is not a precedent**. Do not copy it.
4. ⚠️ `datetime_literal` truncates below the microsecond in silence, and the register already names
   story 5.10 as the owner of that [`:2301`]. Do not fix it here; do not depend on sub-microsecond
   precision either.

### 7. 🔴 A green suite says NOTHING about the database, and most of this story is database

`DATABASE_URL` is **unset** on this machine — measured at this contexting, and
`cargo test --workspace --locked` gives **181 / 156 / 46** either way. Every DB-backed test
`return`s. CI provides the service (`mariadb:10.11.11`, D64: dev = CI = prod).

```sh
docker run --rm -d --name opencmdb-dev-db -p 13306:3306 \
  -e MARIADB_ROOT_PASSWORD=opencmdb -e MARIADB_DATABASE=opencmdb_test \
  mariadb:10.11.11
docker ps --filter name=opencmdb-dev-db      # ⚠️ CONFIRM IT IS UP BEFORE TRUSTING ANY GREEN RUN
export DATABASE_URL='mysql://root:opencmdb@127.0.0.1:13306/opencmdb_test'
cargo test --workspace --locked
```

🔴 **Host port 13306, NOT 3306.** Re-measured at this contexting: `0.0.0.0:3306` is **still held**
by an unrelated container from another project. The dangerous path is not the `docker run` failing —
it is the dev exporting the DSN anyway, reaching a **MariaDB 11** instance (the wrong engine version
under D64) and having `sqlx::migrate!` apply `0001`, `0002` and `0003` to **someone else's
database**. Never touch that container. If 13306 is taken too, pick another free port.

🔴 **Drop and recreate `opencmdb_test` before the first run with `0003`, and between DDL
mutations.** Measured at the validation: on a database that still holds a story-5.9-era link whose
`observation_id` names no observation, `0003`'s foreign key fails **ERROR 1452**, `sqlx::migrate!`
refuses the whole set, and every DB test dies at `.expect("migrate")`. The container above is the
one story 5.9 used; **its leftover data is the trap, not the container**.

⚠️ `docker/docker-compose.yml` deliberately has **no database service** — it points at an external
MariaDB. It is not the tool for this.

**The Debug Log must state, for every mutation, whether it ran with a database and which assertion
carried the red.** "Green" without that qualification is not a measurement.

### 8. Gates, and the traps that cost an hour if they are not read

- **The six gates must stay green.** `cargo xtask ci`: frontier (D47), `ddl-collation` (D64),
  vocabulary (D65), `fixtures`, `file-size` (D56b), `float-free` (D13). `views-hash` reports
  `ℹ STALE` and exits 0 — **by design, do not regenerate**.
- ⚠️ **`ddl-collation` is a per-line reflex heuristic, not a parser** [`xtask/src/main.rs:307-367`]:
  it uppercases each line and flags it when it contains `VARCHAR` / `TEXT` / `" CHAR"` /
  `CHAR…`(at line start) / `CLOB` **and** does not contain `_BIN` or `COLLATE BINARY`. It skips a
  line only when the line **starts** with `--`. **Consequences for `0003`:** the word *text* or
  *character* in a TRAILING comment reds the gate; a column split over two lines reds it. **One
  column per line, prose on its own `--` line**, as `0001` and `0002` are already written.
- ⚠️ **D47 is a gate.** `opencmdb-core` must not gain `sqlx`, `anyhow`, `axum` or `askama`. Every
  line of SQL in this story lives in `opencmdb-bin`. If decision 1's newtype lands in core, it
  carries no persistence.
- ⚠️ **`float-free` walks `crates/opencmdb-core/src/identity/` only** (4 files today). This story
  adds no float anywhere; `ruleset_version` is a `u32` and there is no confidence, no score, no
  ranking.
- ⚠️ **`file-size` ceiling is 2000 CODE lines** (largest today: 1136). `repo.rs` is at **657 code
  lines** and this story adds query bodies to it. Watch it; a new module is the answer, not growth.
- ⚠️ **Run clippy TWICE** before pushing: `cargo clippy --workspace --all-targets -- -D warnings`
  **and** `cargo clippy --workspace --locked -- -D warnings` (the second is what CI runs; an import
  kept alive only by a test passes the first and fails the second).
- ⚠️ **`sqlx::migrate!` embeds the migration directory at COMPILE time.** After adding `0003`, a
  stale build runs the old set — if a test fails with "table doesn't exist" or "unknown constraint",
  `touch` the crate before believing it.
- ⚠️ **Never edit `0002`.** `sqlx` checksums applied migrations; changing one that has run makes
  every existing database refuse to migrate. New guards go in `0003`.
- ⚠️ **Issue #38 — unexplained local test non-determinism.** If a test reds once and then passes
  eight times on a clean tree, that is #38, not a finding about this story. Record it on the issue,
  and **do not adopt a cause without naming the check that would have failed if it were wrong.**

### 9. The END STATE, measured by an agent that BUILT this story before you did

The validation's gap-hunt implemented AC1–AC10 in an isolated worktree against a live
`mariadb:10.11.11` and ran every mutation. **These are targets, not predictions — a divergence is a
FINDING, not a variation.**

| | before | after |
|---|---|---|
| tests | **383** (181 bin + 156 core + 46 xtask) | **398** (196 + 156 + 46) |
| `repo.rs` code lines | 657 | **763** |
| `resolver.rs` code lines | — | **281** |
| `file-size` largest | 1136 (ceiling 2000) | **1136** — unchanged |
| `float-free` | 4 files under `identity/` | **4** — unchanged |
| `cargo xtask ci` | six green + `views-hash ℹ STALE` | identical |
| `git status fixtures/` | clean | **clean — nothing moved** |
| trap corpus | 11 unanswerable, `passed() == false` | **unchanged**, and its naming test stays green |

`0002`, `epics.md` and `architecture.md` untouched. Both clippy forms green — **the `--locked` one
only with T3's `#![allow(dead_code)]`**.

⚠️ **What building it found that reading it could not** — eight things, every one of them now folded
into an AC or a task above, and worth reading as a list because it is the shape the next story will
repeat:

1. **The clock is not in the build.** `Utc::now()` does not compile; half of AC6 was guaranteed by
   `Cargo.toml`, not by this story.
2. **`decide_pair` returns a `Decision`, `insert_identity_link` requires one, and a singleton has
   neither.** The organs compose for pairs and leave a hole that only appears under a type-checker.
3. **Running the pass twice explodes** — `Err(Constraint("unique"))`, full rollback. AC4's central
   sentence was unbuildable as worded.
4. **M2's mutation and AC8's test lived on opposite sides of the seam.** Each sensible alone; wired
   together they measured nothing.
5. **The foreign key reds 14 tests across TWO files, not 8 in one** — and two of them fail for an
   entirely different reason, order-dependently.
6. **A green test silently stopped measuring its guard**: MariaDB reports the observation FK before
   the interface FK, so `every_ddl_guard_refuses_what_it_names` is now satisfied by the wrong
   constraint — and because it never reds, nothing routes anyone to it.
7. **The widening test's red comes from the end AC6 did not name.** `first_seen_at` alone passes M7.
8. **Decision 8's own numbers disagreed with D13's quote by 2×** — 44 850 measured, "90k" quoted, in
   the same paragraph.

🔴 **§9's own rule applied to §9 — three divergences, reported here as findings rather than left as
variations.** The table above is the gap-hunt's worktree, not this implementation:

| §9 target | measured at implementation | at the code review |
|---|---|---|
| 398 tests (196 + 156 + 46) | **402** (197 + 159 + 46) | **408** (203 + 159 + 46) |
| `resolver.rs` **281** code lines | **390** | **431** |
| `repo.rs` **763** code lines | **773** | **773** |

The `file-size` gate is unaffected (largest still 1136) and the direction is upward in every row —
the gap-hunt built a narrower version of the same story. The point is not the drift but that §9
said *"a divergence is a FINDING"* and the implementer did not apply it to §9. Caught by the
Acceptance Auditor.

⚠️ **Issue #38 recurred once during that run** (`fixtures::tests::a_decision_carrying_an_abstention_cause_is_refused`,
one red in ~25 runs, clean tree, all 25 fixture sha256s green, not reproduced). No cause adopted.

---

## Decisions taken at contexting

Nine. They are decisions, not suggestions — a dev who disagrees reports a FINDING rather than
choosing differently. Decisions 1, 2 and 3 were taken by Guy at this contexting; the rest are
measured against the tree.

**1. AC1 widens: one interface per L1 KEY the observation carries.** §2. `epics.md` is not edited.

**2. `join` names the interface; the blocker and `decide_pair` justify the placement.** §3. The
connected-component alternative is refuted by the existential quantifier, and T7 requires that
refutation to be MEASURED before this sentence is quoted as fact.

**3. The resolver is NOT wired into `main.rs`.** It is production code in a new
`crates/opencmdb-bin/src/resolver.rs`, called by its own tests today and by stories 5.10 and 5.11
next. Wiring the startup scan would make every real deployment write links with no page to display
them (5.14) and no purge to remove them (5.10) — a behaviour change no acceptance criterion asks
for. **Register the residue with story 5.14 as owner**, so the promise is not silently carried.

**4. The witness is the smallest `ObsId` in the group other than `o`, and the link's evidence is the
pair verdict's evidence.** The group is a `BTreeSet<ObsId>`, so "smallest other" is deterministic and
therefore reproducible, which is what story 5.10 replays. D19 wants *"the rule_id and the evidence"*
of the rule that fired; the verdict's `evidence` is the sorted pair [`l1.rs:277-278`], so a link
names the pair that justified it — **not the whole group**. If a richer evidence is ever wanted for
display, that is **story 5.14**'s call; register it. _(The sort is `evidence.sort()` at
`l1.rs:278`, and its comment records that the asymmetry was measured before it was fixed.)_

**5. 🔴 A SINGLETON group is placed by the key, not by a pair — and the self-pair is NOT used to
manufacture one.** When an observation is alone on its key there is no pair to judge. The placement
is still true: at L1 the interface **is** the key, so the observation carrying that key is on that
interface by `join`'s definition. The link therefore carries `rule = l1-exact-mac` and
`evidence = [o.obs_id]`, and **the story states plainly that this is the one placement in the pass
whose rule does not come from a `decide_pair` call.**

🔴 **The `Decision` for a singleton comes from a NEW `identity::l1::decide_singleton(&Observation)`,
and that is the ONE change this story makes inside the engine.** Guy's arbitration at the
validation, on a hole the gap-hunt found **under the compiler**: `insert_identity_link` takes
`decision: &Decision` [`repo.rs:353-358`], a singleton produces no `decide_pair` call, so the
resolver must manufacture one — and exactly two other routes exist, both refused:

- a **struct literal** `Decision { conclusion: Match { .. }, verdict_vector: vec![], .. }` — the
  shape `cascade.rs:322-330` documents as *"merged, with no explanation"*, which D13's *"the list IS
  the explanation"* exists to prevent;
- `decide(vec![RuleVerdict { rule: RuleId(L1_EXACT_MAC), verdict: Decisive, evidence: vec![o] }], …)`
  **from the resolver** — it compiles and keeps the vector non-empty, but it makes `opencmdb-bin`
  the first producer of an L1 rule verdict outside `identity/l1.rs`, in the teeth of the reason
  `verdict_for_pair` is `pub(crate)` at all.

`decide_singleton` keeps verdict composition where it belongs, builds the one-element `Decisive`
vector itself and returns `decide(…)`'s value — so nothing bypasses `decide` and the residue
`cascade.rs:345-347` registers is closed rather than met.

⚠️ **This AMENDS the story's own "does not touch `identity::l1`" boundary**, deliberately and once.
The boundary sentence also says *"if a change there appears necessary, that is a FINDING"* — this is
that finding, raised at the validation and arbitrated rather than absorbed. Nothing else in `l1.rs`
moves, and `join`, `decide_pair` and `verdict_for_pair` are untouched.

🔑 **The residue is stated in THREE places and all three must move together**:
`deferred-work.md:2187`, `:2195`, and the doc comment at `cascade.rs:345-347` — which names
**story 5.9** as its owner, an attribution 5.9 answered in tests only. Updating two of three is the
doc-twin defect four of story 5.9's review patches were.

⚠️ **The MAC-less abstention is NOT in the same position.** `verdict_for_pair` is `Neutral` whenever
either side carries no MAC, so `decide_pair(o, w)` on any other observation returns
`Abstained { AbsenceOfProof }` **from the engine** — no struct literal needed. The only case that
still needs one is a slice holding a single MAC-less observation, where the blocker proposes no pair
at all. **Prefer the engine's value wherever a pair exists**, and say in the code which of the two
paths produced each `Decision`.

⚠️ **Calling `decide_pair(o, o)` instead is REFUSED.** It would return `Match` — an observation
trivially shares every key with itself — and it would re-open in the resolver exactly the self-pair
that story 5.6 closed in the TYPE (`CandidatePair::new(a, a) -> None`) and story 5.7's code review
found re-opened in `answer_trap`. **Twice closed; not re-opened a third time.**

**6. The abstention at L1 is `AbsenceOfProof` with ZERO candidates, and the incoherent shape is
refused by a guard tested DIRECTLY.** §1 derives that `Ambiguous` is unreachable at L1. So:
- an observation carrying **no MAC** is in no `join` group, and the pass writes it a **link** with
  `interface_id = NULL`, `outcome = 'abstained'`, `abstention_cause = 'absence_of_proof'` and **no**
  `link_candidate` rows — *"never an absence"* (D14/FR16), and correctly no candidates, because
  nothing was a candidate;
- the writer refuses `Conclusion::Abstained { cause: Ambiguous }` with an empty candidate list, and
  refuses an empty `rule_id`. **Both guards are unreachable through the resolver** (the engine
  cannot produce either), so **both are tested by calling the writer directly** — story 5.8's lesson,
  where a guard tested through its harness stayed green with the guard deleted.

**7. `find_interface_by_l1_key` reads inside the transaction, and the seen-window is widened with
`LEAST`/`GREATEST` in the UPDATE.** The lookup is what makes AC4's read-your-own-writes real rather
than a convention, and what makes an interface id stable across runs (§6.1). For the window: reading
a `DATETIME(6)` back as a *value* is not possible here — `sqlx` is built without its `chrono`
feature, which is why `load_link_valid_to` renders with `CAST(… AS CHAR)` [`repo.rs`] — and
comparing MariaDB's rendered strings in Rust would be an instant comparison wearing a string
costume. `LEAST`/`GREATEST` over two `DATETIME(6)`s is **bookkeeping, not the identity comparison
D10 forbids**: no domain value is under judgement and MariaDB is the only engine (D64).
**Register the alternative** — enabling `sqlx`'s `chrono` feature would collapse the second
rendering site *and* let the window be computed in Rust; the register already owns that entry with
*"the first story that needs to read an instant back as a value"* as owner clause [`:2260`], and
this story does **not** meet it.

**8. The quadratic universe is MEASURED, not bounded.** This is the first story that can measure
`n`. D13 says *"at 300 hosts 90k pairs is not a performance concern on the reference NAS"*
[architecture.md, quoted by `epics.md:1510`]. The pass asserts `candidates(obs).len() == n*(n-1)/2`
over distinct ids — **asserted, not quoted** — and the Debug Log records the wall-clock of one pass
at the reference scale. **No refusal threshold is installed**: a bound with no measured need is the
speculation the *"create only what the story needs"* rule refuses. The register entry stays OPEN
with its number.

⚠️ **D13's "90k" and this story's own formula disagree by a factor of two, and the measured number
is the smaller one.** At n = 300, `n(n-1)/2` = **44 850** — measured at the validation, with blocking
at ~30 ms and a full ingest-plus-resolve pass at ~106 ms. D13's figure counts pairs the other way.
**Write 44 850 in the Debug Log, not 90k**, and treat the two timings as PREDICTIONS to confirm or
refute rather than as targets.

**9. The resolver writes `match` and `abstained` links only — never `no_match`.** A `no_match` is a
fact about a PAIR; the link is `observation -> interface`, so a non-match produces no placement and
therefore no row. `no_match` stays in the schema's vocabulary for the operator's own decisions and
for Epic 6. **No CHECK is added to forbid it** — that would be a constraint asserting a rule this
epic has not decided.

**10. 🔴 The pass is NOT idempotent over the same observations, and that is story 5.11's.** Guy's
arbitration at the validation, on a measurement: running `resolve` twice over the **same** slice
inside one `transact` returns `Err(Constraint("unique"))` and rolls the whole transaction back —
**0 interfaces, 0 links**. `insert_identity_link` appends, and `identity_link_one_current` refuses a
second current row for one `(observation_id, current_subject)`. There is no supersede and no upsert
here: `0002`'s own header already names the owner — *"story 5.11's 'no new version for an unchanged
decision', i.e. the normal path"*. **AC4 was written as if a re-run just worked; it does not, and
its two tests use DIFFERENT observations on the SAME key instead** (measured green that way:
1 interface found, 0 minted). Register the non-idempotence with **story 5.11**.

**11. What an observation gets when the blocker did NOT propose its pair is left undecided, on
purpose — and the seam that would decide it exists anyway.** `candidates` is TOTAL today, so the
case is **unreachable**; the enum has two causes and neither means *"the blocker declined to
propose"*, so choosing one now would be inventing a semantics no caller can produce. The resolver
therefore abstains, and the story says plainly that `absence_of_proof` is then a **cause of
convenience** rather than a true one. **Register it with the first story that NARROWS the blocker**
— F17's `dormant` exclusion [architecture.md:1205] is the named candidate. The seam
(`resolve_within`, decision 12 below) exists so that mutation M12 has something to red, not because
the case is live.

**12. The entry point splits in two: `resolve(conn, observations)` delegates to
`resolve_within(conn, observations, &universe)`.** Measured at the validation: with the universe
computed internally there is **no seam to narrow**, and M12 — *"judge a pair the blocker did not
propose"* — left the **entire suite green (397/397)**. The seam is what turns a confirmed no-op into
an assertion-carried red. It costs one function.

---

## Acceptance Criteria

**AC1 — the pass exists, is production code, and calls the two organs in D13's order.**
**Given** `epics.md:1602-1604` — *"both are called by production code, and D13's order —
candidate generation (blocking) -> verdicts -> three-way decision — is the order of the pass; the
blocker is not bypassed by reading the join's key directly"*
**When** the resolver is written
**Then** `crates/opencmdb-bin/src/resolver.rs` exists, is **not** `#[cfg(test)]`, and calls
`identity::blocking::candidates`, `identity::l1::join` and `identity::l1::decide_pair` — the first
being called **once, over the whole slice, before any verdict is asked for**.
**And** the entry point is TWO functions — `resolve(conn, observations)` delegating to
`resolve_within(conn, observations, &universe)` (decision 12). ⚠️ **Without that seam mutation M12
was measured leaving the ENTIRE SUITE GREEN (397/397)**: with the universe computed internally there
is nothing to narrow, so "the blocker is not bypassed" is unfalsifiable.
**And** every pair it judges is confirmed present in the universe `resolve_within` was handed; a pair
that is not there is never judged, and an observation whose every pair was excluded **abstains** —
with `absence_of_proof` as a **cause of convenience**, which decision 11 registers rather than
pretends is true.
**And** every `pub` item, field and variant carries a `///` doc (house rule), and the module doc
states §3's mechanism and decision 5's singleton case.

**AC2 — one interface per L1 key, and a multi-MAC observation gets one link per interface.**
**Given** decision 1 and `join`'s `for key in keys_of(observation)` [`l1.rs:174-178`]
**When** the pass runs over observations of which at least one carries two MACs
**Then** each `(l2_domain, mac)` key present in the slice has **exactly one** `interface` row, and
the two-MAC observation holds **two** current `identity_link` rows, one per interface.
**And** two observations sharing one key produce **one** interface and **two** links.
**And** the counts are asserted by reading the tables, never by trusting the summary the pass
returns — story 5.8's M5 lesson: an oracle that restates the expectation measures nothing.

**AC3 — every placement is a link carrying the rule that settled it, its evidence and the ruleset.**
**Given** `epics.md:1580`'s restatement of D14 — *"carrying the rule applied, the evidence, when, by
whom, and `ruleset_version`"* — which deliberately drops D14's own **`confidence`**
[architecture.md:1015] and which `deferred-work.md:2238-2248` records as the later and correct
wording _(the citation read "D14" here until the validation, hiding the divergence inside an
ellipsis — the exact defect this project's reviews exist to catch)_
**When** a placement is written
**Then** the row carries `outcome='match'`, `rule_id = 'l1-exact-mac'`,
`decided_by='ENGINE'`, `ruleset_version = CURRENT_RULESET_VERSION.0` (**1**), `evidence` = the pair
verdict's evidence (decision 4), `valid_from` = the observation's own `observed_at`, and
`valid_to = OPEN_END`.
🔴 **`l1-exact-mac` is the ONLY rule a placement can ever carry, and an assertion over the set
{`l1-exact-mac`, `l1-distinct-mac`} would pass while measuring nothing.** Chain, measured at the
validation: `verdict_for_pair` returns `L1_DISTINCT_MAC` only with `Verdict::Disqualifying`
[`l1.rs:269`], `decide`'s first arm turns any `Disqualifying` into `Conclusion::NoMatch`
[`cascade.rs:491`], `outcome_token` renders that `"no_match"` [`repo.rs:255`], and decision 9 says
this pass never writes a `no_match` row. So `l1-distinct-mac` is unwritable here **by three
independent steps**, and a test must assert the single value.
**And** for a group of size ≥ 2 the rule and the evidence come from an actual `decide_pair` call —
**not** from a constant the resolver chose. Since the rule is knowable in advance, **the evidence is
what carries this assertion**; a test that checks only the rule measures a constant.
**And** for a singleton group (decision 5) the rule is `l1-exact-mac` and the evidence is the single
`obs_id`, and a test asserts that shape explicitly so it cannot drift into a self-pair.

**AC4 — read-your-own-writes inside ONE transaction, and the interface id is stable across runs.**
**Given** D21 — *"identity resolution runs INSIDE the writer actor, against the write connection"*
and *"an identity decision is NEVER split across two transactions"* [architecture.md:1434-1446]
**When** two observations of the same MAC arrive in one pass
**Then** the second sees the first's interface: **one** `interface` row, found by
`find_interface_by_l1_key` rather than re-minted.
**And** a second pass over **DIFFERENT observations on the same key**, in the same transaction and
then in a fresh one, **finds** that interface and does not create a second one — asserted by
`SELECT COUNT(*) FROM interface`, which is what story 5.10's replay depends on (§6.1).
🔴 ⚠️ **Not over the SAME observations: that is measured to explode.** `resolve` run twice over one
slice returns `Err(Constraint("unique"))` and rolls the whole transaction back — 0 interfaces,
0 links. The pass is not idempotent, by decision 10, and that is story 5.11's deliverable. An AC
written as "run it twice" is unbuildable.
**And** the resolver's signature takes the write connection and returns a `Result`; it never opens a
transaction of its own and never reaches the read pool.

**AC5 — an abstention is a LINK, never an absence — and the incoherent shapes are refused.**
**Given** D14/FR16 — *"the ambiguity is DATA, not a hole; otherwise there is nothing to display and
FR16 is vapour"* — and §1's derivation that L1 can only abstain with `AbsenceOfProof`
**When** the pass meets an observation carrying no MAC
**Then** it writes a link with `interface_id IS NULL`, `outcome='abstained'`,
`abstention_cause='absence_of_proof'`, and **no** `link_candidate` rows.
⚠️ **The persisted token is `absence_of_proof`, lowercase with an underscore** — `cause_token`'s
exhaustive match [`repo.rs:266`], pinned by `every_persisted_token_is_pinned` [`repo.rs:1852`].
`'AbsenceOfProof'` is the Rust variant name and appears in no committed byte; an assertion written
against it reds, and the tempting "fix" — editing `cause_token` — breaks a pinned test. _(This AC
carried the wrong spelling until the validation; `CLAUDE.md` carries it too, and this story's AC10
twin-update is where that gets corrected.)_
**And** a guard in `resolver.rs` — **not** in `insert_identity_link` — **refuses** an
`Abstained { Ambiguous }` with an empty candidate list and **refuses** an empty `rule_id`, returning
`RepositoryError::Constraint`, and **both are tested by calling that guard directly** with no
database at all, because the resolver cannot reach either shape (decision 6).
⚠️ **It cannot live in `insert_identity_link`**: that function returns `Result<(), sqlx::Error>` and
`RepositoryError::Constraint` is not constructible as an `sqlx::Error` — measured at the validation.
(`close_identity_link` returns `RepositoryError`, so the adapter is already inconsistent here; do
not "fix" that inconsistency in this story, and do not change `insert_identity_link`'s signature,
which would move every one of story 5.9's call sites.)
**And** the doc says in one sentence why an `AbsenceOfProof` with zero candidates is correct while an
`Ambiguous` with zero candidates is not.

**AC6 — the pass reads no clock, and a re-run reproduces every instant it stores.**
**Given** `epics.md:1614` — *"the pass reads no clock for anything it stores as an interface's
`first_seen_at`/`last_seen_at`: those are derived from the observations, so a re-run reproduces
them"*
**When** the pass writes
**Then** `interface.first_seen_at` / `last_seen_at` are the `min` / `max` of the group's
`observed_at`, `identity_link.valid_from` is the observation's own `observed_at`, and **no `NOW(6)`
and no `Clock` appears anywhere in `resolver.rs`** — **all three read back from the database and
compared**, not merely written.
🔑 ⚠️ **`Utc::now()` does not compile in this workspace, so it is not the risk.** `chrono` is pinned
`default-features = false` in BOTH crates precisely to keep its `clock` feature off — and
`crates/opencmdb-bin/Cargo.toml:22-23` says why: *"otherwise feature unification would re-enable it
in `opencmdb-core` and the domain could call `Utc::now()` (D19)"*. Measured: `error[E0599]: no
associated function named 'now' found for struct 'Utc'`. **The only clock reachable from here is
SQL's `NOW(6)`**, and that is what the AC and mutation M6 must name. _(This AC forbade `Utc::now()`
until the validation — a prohibition `Cargo.toml` already enforced, which is not a guard this story
earns.)_
**And** a second pass over observations OLDER than an existing interface's window **widens** it
rather than narrowing it (decision 7), **asserted at BOTH ends of the window**. ⚠️ Measured: under
mutation M7 `first_seen_at` lands on the right value anyway and the red comes entirely from
`last_seen_at` — **a test asserting only `first_seen_at` passes M7**.
**And** the mutation that replaces a derived instant with `NOW(6)` is run on **both** instants and
reds each on its own assertion (T7/M6a, M6b).

**AC7 — `0003` installs the three guards the first writer owes, and the FOURTEEN affected tests are
FIXED, not the guard dropped.**
**Given** register entries #5, #7 and #8 (§5)
**When** `crates/opencmdb-bin/migrations/0003_resolver_guards.sql` lands
**Then** it adds `identity_link.observation_id → observation_record(id)` as a FOREIGN KEY, a
`CHECK (mac_canon = LOWER(mac_canon))` on `interface`, and a `CHECK (rule_id IS NULL OR rule_id <> '')`
on `identity_link`.
**And** the tests that mint an `ObsId` without inserting an observation are **fixed to insert one**.
🔴 **The number is FOURTEEN, not eight, and it spans TWO files** — measured at the validation on a
freshly created database with only `0003` added:
- **12 in `repo.rs`**, all `Constraint("foreign_key")` panic-carried at `.expect()`;
- **2 more with a different failure mode entirely** — `repo::tests::ingest_observation_round_trip`
  and **`main::tests::index_renders_the_real_gap`** — which `DELETE FROM observation_record` without
  deleting links first and die on **ERROR 1451** as soon as an earlier test leaves a link behind.
  These need a **cleanup-ordering** fix, not an inserted observation, and they are order-dependent,
  so a single-test run hides them.
The register's *"reds 8 tests"* is corrected here rather than repeated. **`main.rs` is in scope for
this task**; the register named only `repo.rs`.
🔴 **And `every_ddl_guard_refuses_what_it_names` must be sent an observation that EXISTS.** Measured:
after `0003`, its `identity_link_interface_fk` case (a link naming a ghost interface, on an
observation that was never inserted) is satisfied by **ERROR 1452 on `identity_link_observation_fk`**
— MariaDB reports the observation FK first, both classify as `foreign_key`, and the assertion can no
longer tell them apart. **The test stays GREEN**, so an instruction that says "fix each test the FK
reds" never routes anyone to it, and one of story 5.9's guards goes quietly unmeasured. One line
fixes it; the story has to be the thing that says so.
**And** `0002` is **not edited** (§8), the `ddl-collation` gate stays green, and it is **shown to
bite**: strip one `COLLATE` temporarily, watch the gate name the line, restore.
**And** each new CHECK is tested by a **raw SQL insert** that goes around the adapter — story 5.9's
M3 lesson, where dropping a CHECK left all 378 tests green because the adapter could not emit the
incoherent pair.

**AC8 — the two organs are shown to AGREE, and the alternative grouping is shown to be wrong.**
**Given** decision 2 and §3's refutation
**When** the agreement is asserted
**Then** a test over synthetic observations asserts that for every `join` group, every intra-group
pair is in the blocker's universe and `decide_pair` concludes `Match` on `l1-exact-mac`; and that a
pair sharing no key concludes `NoMatch` on `l1-distinct-mac`.
**And** the transitivity case is built explicitly — A shares `k1` with B, B shares `k2` with C, A and
C share nothing — **through `resolve`, against the database**, asserting **2 interfaces** and that
A's and C's are different, while both A–B and B–C are `Match`. **This is §3's refutation, measured
rather than asserted in prose.**
🔴 ⚠️ **The pure version of this test does NOT carry it.** Measured at the validation: with the
grouping mutated to connected components (M2), a transitivity test written against `join` and
`decide_pair` directly **stays green** — it never calls `resolve`, so a mutation of the resolver's
grouping is invisible to it. Keep the pure test, which states the quantifier; but **M2's named
target is the DB one**, where the red is assertion-carried (`two interfaces, left: 1, right: 2`).
**And** the inputs are synthetic: every committed replay stream carries one `l2_domain`
[`l1.rs:83-88`], so the corpus cannot exercise the scope half of the key.

**AC9 — the eleven registered entries are disposed of, each with its measurement.**
**Given** §5's table
**When** `deferred-work.md` gains this story's section
**Then** each of the eleven is CLOSED, ANSWERED or RE-OWNED **with the measurement that justifies the
verb**, and an entry whose condition is unmet is never reported as closed.
**And** the quadratic entry (#3) carries the **measured** `n` and pair count of a reference-scale
pass, and stays OPEN by decision 8.
**And** the new residues are registered with named owners: decision 3 (no `main.rs` wiring →
story 5.14), decision 4 (evidence names the pair, not the group → story 5.14), decision 7 (the
`sqlx` `chrono` feature → the first story that reads an instant back as a value), §2's `epics.md`
AC1 correction → **Epic 5's retrospective**, and the four the validation added:
- **decision 10** — the pass is not idempotent over the same observations → **story 5.11**;
- **decision 11** — what an observation gets when the blocker excluded all its pairs, and the fact
  that `absence_of_proof` is then a cause of convenience → **the first story that narrows the
  blocker** (F17's `dormant` is the named candidate);
- 🔴 **`epics.md:1616` is departed from too**: it requires the abstention link to carry *"its
  `link_candidate` rows"*, and decision 6 writes **zero** for an `AbsenceOfProof`. Justified, but it
  is a second unregistered divergence from the epic → **Epic 5's retrospective**, beside §2's;
- the **`main.rs` cleanup-ordering** fragility AC7 uncovered, if T5's fix leaves any residue.

**AC10 — prove-to-red, and the documents say the same thing as the code.**
**Given** the house rule (story 1.3) and the AC8-family defect that story 5.9's own review caught
**four times inside the commit meant to enforce it**
**When** the story closes
**Then** every guard has a recorded mutation, with **what carried the red — assertion,
`expect`/`expect_err` panic, or compiler** — and whether the run had a database.
**And** `docs/project-context.md` **and** `CLAUDE.md` carry the same test counts, the same story
status and the same Epic 5 tally, verified by grepping both for the phrases they duplicate. **Do not
write "12 done" while `sprint-status.yaml` says `review`** — `done` is the MERGE's business here.
**And** the trap corpus's report is re-checked and still reads **11 unanswerable, `passed() == false`**.

---

## Tasks / Subtasks

- [x] **T1 — branch, live database, committed baseline (AC10)**
  - [x] Branch from `master` at `47bdca2`: `story-5.9b-engine-resolves-and-writes-links`.
  - [x] Start `mariadb:10.11.11` on **13306** and export `DATABASE_URL` (§7). Confirm `docker ps`
        before trusting any green run. Record the baseline **with** the DB set, and note which tests
        run now that were silently returning.
  - [x] **Commit the clean baseline before the mutation pass** — `git checkout <file>` restores to
        HEAD, not to uncommitted work, and that has destroyed new tests mid-pass twice.

- [x] **T2 — the adapter's new query bodies (AC4, AC5, AC7)**
  - [x] `find_interface_by_l1_key(executor, l2_domain, &MacAddr) -> Result<Option<InterfaceId>, sqlx::Error>`
        — static SQL, bound values, `ORDER BY id LIMIT 1` **with a doc saying why an ordered
        first-match is correct**: the L1 index is deliberately non-unique (a cloned MAC is two real
        interfaces), so this returns *an* interface for the key and the cloned-MAC case is Epic 6's.
  - [x] `widen_interface_seen_window(executor, id, first_seen, last_seen)` — `LEAST`/`GREATEST`
        (decision 7), with the D10 sentence in its doc.
  - [x] The link writer (see T3) may live in `resolver.rs`; the raw query bodies stay in `repo.rs`.
        ⚠️ **Watch `file-size`**: `repo.rs` is at 657 code lines of a 2000 ceiling.
  - [x] Give `count_identity_links` a caller and a test, or delete it (register #9).

- [x] **T3 — the resolver (AC1, AC2, AC3, AC5, AC6)**
  - [x] New `crates/opencmdb-bin/src/resolver.rs`; declare `mod resolver;` in `main.rs`.
  - [x] 🔴 **`#![allow(dead_code)]` at the top of the file, with the one-line justification
        `repo.rs:11` already carries** — it is the price of decision 3. ⚠️ Measured at the
        validation: without it, `cargo clippy --workspace --locked -- -D warnings` (**the CI form**)
        fails with **8 `error: … is never used`** on the whole deliverable, while `--all-targets`
        passes because the tests keep it alive. That is §8's two-clippy warning landing on this
        story's own central module, and it bites at T9 after everything is written.
  - [x] **TWO entry points** (decision 12): `resolve(conn, observations)` computes the universe and
        delegates to `resolve_within(conn, observations, &universe)`. Both return a summary of
        **counts** (interfaces minted, interfaces found, links, abstentions, candidate pairs).
        Neither opens a transaction: the caller wraps it in `repo.transact(…)`, which is what makes
        *"never split across two transactions"* structural rather than a promise. ⚠️ **The seam is
        not decoration** — without it M12 was measured leaving all 397 tests green.
  - [x] ⚠️ **Do not prescribe an idempotent pass** (decision 10). Running `resolve` twice over one
        slice is `Err(Constraint("unique"))` and a full rollback; that is story 5.11's.
  - [x] The pass, in D13's order: `candidates` once → `join` → per group, per observation, confirm
        the pair is in the universe, `decide_pair`, find-or-mint the interface, widen the window,
        write the link.
  - [x] The abstention branch for observations with no L1 key (decision 6), taking its `Decision`
        from `decide_pair(o, w)` **whenever any pair exists** — the engine returns
        `Abstained { AbsenceOfProof }` for free — and only manufacturing one for a slice of a single
        MAC-less observation.
  - [x] The two guards, in `resolver.rs` and returning `RepositoryError::Constraint`: `Ambiguous`
        with no candidates, and an empty `rule_id`. ⚠️ **Not in `insert_identity_link`** — it
        returns `sqlx::Error`, in which `RepositoryError::Constraint` is not constructible (AC5).
  - [x] ⚠️ No `Utc::now()`, no `NOW(6)`, no `Clock`. Every instant derived (§6.2).

- [x] **T3b — `identity::l1::decide_singleton` (decision 5, AC3)**
  - [x] `pub fn decide_singleton(o: &Observation) -> Decision` in
        `crates/opencmdb-core/src/identity/l1.rs`: build the one-element `Decisive` `RuleVerdict`
        on `L1_EXACT_MAC` with `evidence = vec![o.obs_id]`, return
        `decide(vec![…], CURRENT_RULESET_VERSION)`. **Nothing bypasses `decide`.**
  - [x] Its doc says what it is for and why it exists **here** rather than in the resolver: verdict
        composition stays inside `l1.rs`, which is the whole reason `verdict_for_pair` is
        `pub(crate)`.
  - [x] ⚠️ **This is the ONLY change to the engine in this story.** `join`, `decide_pair` and
        `verdict_for_pair` are untouched, and a second engine edit is a FINDING.
  - [x] Update the residue statement in **all three** places that carry it —
        `deferred-work.md:2187`, `:2195`, and the doc comment at `cascade.rs:345-347`, which names
        story 5.9 as owner. Two of three is the doc-twin defect.

- [x] **T4 — `0003_resolver_guards.sql` (AC7)**
  - [x] The three guards. One statement per line, prose on its own `--` lines (§8).
  - [x] ⚠️ **Do not edit `0002`.** ⚠️ `touch` the crate after adding the file — `sqlx::migrate!`
        embeds the directory at compile time.
  - [x] 🔴 **DROP AND RECREATE `opencmdb_test` before the first `0003` run, and between DDL
        mutations.** Measured: on a database that already holds a story-5.9-era link whose
        `observation_id` names no observation, the `ALTER TABLE … ADD CONSTRAINT
        identity_link_observation_fk` fails **ERROR 1452**, `sqlx::migrate!` refuses the set, and
        every DB test dies at `.expect("migrate")`. §7 tells you to reuse the container story 5.9
        used; its data is what bites.
  - [x] Show the `ddl-collation` gate biting, then restore.

- [x] **T5 — fix the tests the FK reds (AC7)**
  - [x] Run the suite with `0003` applied and **count** the reds. 🔴 The measured number is
        **14 across two files** — 12 in `repo.rs` (`Constraint("foreign_key")`, panic-carried) plus
        `repo::tests::ingest_observation_round_trip` and **`main::tests::index_renders_the_real_gap`**,
        which fail **ERROR 1451** on a `DELETE FROM observation_record` that does not delete links
        first. The register's "eight" is corrected, not repeated.
  - [x] Fix the twelve by inserting the observation the link refers to; fix the two others by
        **ordering the cleanup** (links before observations). **Never by dropping the FK** and never
        by relaxing a test.
  - [x] ⚠️ **The two ERROR 1451 ones are ORDER-DEPENDENT** — they pass when run alone. Trust the
        full-suite run, not a filtered one.
  - [x] 🔴 **Then go to `every_ddl_guard_refuses_what_it_names`, which does NOT red** and now passes
        for the wrong reason (AC7). Give its interface-FK case a real observation.

- [x] **T6 — the tests (AC2–AC8), all `DATABASE_URL`-gated and under `DB_TEST_LOCK`**
  - [x] two observations, one shared MAC → **1** interface, **2** links, both current (AC2).
  - [x] one observation with two MACs → **2** interfaces, **2** current links (AC2). ⚠️ **Synthetic
        by necessity**: no committed observation carries two MACs (§2), so nothing under `fixtures/`
        can stand in for this test.
  - [x] a singleton group → one link, `l1-exact-mac`, evidence `[o]` (AC3, decision 5).
  - [x] a placement's rule and evidence come from `decide_pair` — assert the evidence equals the
        sorted pair, not a constant (AC3).
  - [x] two passes over **DIFFERENT observations on the same key**, first in ONE transaction (AC4,
        read-your-own-writes), then in TWO → **1** interface either way, found and not minted
        (measured green: `interfaces_found = 1`, `interfaces_minted = 0`). ⚠️ **Not the same
        observations twice** — that is `Err(Constraint("unique"))` and a full rollback (decision 10).
  - [x] an older second batch **widens the window at BOTH ends** (AC6, decision 7). ⚠️ Measured:
        under M7 `first_seen_at` lands on the right value anyway and only the `last_seen_at`
        assertion reds — a test asserting one end passes the mutation.
  - [x] an observation with no MAC → an abstained link, `AbsenceOfProof`, **0** candidates (AC5).
  - [x] the two writer guards, **called directly** (AC5) — not through the resolver, which cannot
        reach them.
  - [x] the **three new guards** — the observation FK and the two CHECKs — each by a **raw SQL
        insert** going around the adapter (AC7): a link naming an absent `observation_id` (M8), an
        uppercase `mac_canon` (M9), an empty `rule_id` (M10). ⚠️ **The FK is not a CHECK**; do not
        let a "three CHECKs" heading absorb a constraint of another kind.
  - [x] the agreement test and the **transitivity** case, synthetic, no DB needed (AC8).
  - [x] `candidates(obs).len() == n*(n-1)/2` over distinct ids, asserted (decision 8).
  - [x] 🔴 **two scopes, one MAC** — two observations sharing a MAC in **two** `l2_domain`s →
        **2** interfaces, **2** links. Synthetic by necessity: every committed stream carries one
        `l2_domain` [`l1.rs:83-88`]. **Added at the validation**, which measured that M1 had no
        target test.
  - [x] 🔴 **the stored instants are the derived ones** — read `interface.first_seen_at` /
        `last_seen_at` and `identity_link.valid_from` back and compare against the group's
        `min`/`max` `observed_at` and the observation's own. **Added at the validation**: every
        other prescribed test asserts a COUNT, which a clock-derived instant does not change, so
        M6 had nothing to red. ⚠️ Read them with `CAST(… AS CHAR)` and compare against
        `datetime_literal(expected)` — `sqlx` is built without `chrono` here (decision 7), and that
        is the same transport idiom `load_link_valid_to` already uses [`repo.rs`].
  - [x] 🔴 **the reference scale** — build **300** synthetic observations and assert
        `candidates(&obs).len() == 44_850`, recording the pass's wall-clock in the Debug Log
        (decision 8, AC9). **Added at the validation**, which measured that no task produced the
        reference-scale number both decision 8 and AC9 demand.

- [x] **T7 — prove-to-red (AC10). Every mutation run WITH `DATABASE_URL` set unless marked.**
      Record for each: DB yes/no, tests red, **and what CARRIED the red**.
  - [x] **M1** — make `find_interface_by_l1_key` ignore `l2_domain` and match on `mac_canon` alone
        → the **two-scope test** must red on its interface COUNT. ⚠️ **Re-aimed at the validation.**
        As first written it said *"group by the bare MAC"*, which is `join`'s key and therefore an
        edit to `identity/l1.rs` — a file this story forbids itself to touch, and a guard story 5.5
        already reds (`l1.rs:452`, `l1.rs:461`). The mutation must live in THIS story's code.
        *Predicted to red only a synthetic test: every committed stream carries one `l2_domain`.*
        ⚠️ Measured at the validation in its ORIGINAL form (mutating `keys_of`): **10 tests red, 7 of
        them `opencmdb-core`'s own** — it re-measured stories 5.5 and 5.6's committed guards and
        added no coverage here, with only one assertion-carried red among the three in the resolver.
  - [x] **M2** — build the groups as connected components of the `Match` pairs instead of by key →
        **the DB-level transitivity test** must red (§3's refutation, MEASURED). ⚠️ **Re-aimed at
        the validation**: against the PURE transitivity test it was measured a **NO-OP** — that test
        calls `join` and `decide_pair` directly and never calls `resolve`, so the whole suite stays
        green except one incidental multi-NIC assertion. §3 is right; the wiring was wrong. Against
        the DB test the red is assertion-carried (`two interfaces, left: 1, right: 2`).
  - [x] **M3** — skip `find_interface_by_l1_key` and always mint → the "1 interface after two
        passes" test must red on its COUNT (AC4).
  - [x] **M4** — take the link's rule from a constant instead of from the `Decision` → the
        rule/evidence test must red. ⚠️ *Predict the shape: a constant `l1-exact-mac` is the RIGHT
        answer for every group of size ≥ 2, so the red must come from the **evidence**, not from the
        rule. If it only reds on the rule, the test is measuring the wrong half.*
  - [x] **M5** — write the abstention as an absence (no link row) → the abstention test must red on
        its COUNT, never on an `.expect()` of the write (story 5.9's M4 lesson: the `.expect()` form
        lets a foreign key carry the red and the assertion is never evaluated).
  - [x] 🔴 **M6 — `NOW(6)`, in TWO variants, and NOT `Utc::now()`.** ⚠️ **Rewritten at the
        validation, twice over.** `Utc::now()` **does not compile** here (`error[E0599]`; `chrono` is
        `default-features = false` in both crates to keep the `clock` feature off), so M6 as first
        written was **not executable**. And its named target was "the re-run reproducibility test",
        which asserts `COUNT(*) FROM interface` — a count no clock changes.
        - **M6a** — interface window ← `NOW(6)` → the widening test reds, assertion-carried.
        - **M6b** — link `valid_from` ← `NOW(6)` → 🔴 measured to red **nothing in the resolver**:
          the only three reds were pre-existing `repo.rs` tests, panic-carried, for the interval
          CHECK rather than for the instant. **No test read a link's `valid_from` back, and
          `PersistedLink` does not even carry the column.** The stored-instants test of T6 is what
          gives M6b a target — this is story 5.9's AC3 defect repeating one story later.
  - [x] **M7** — `LEAST`/`GREATEST` → plain assignment → the widening test must red (AC6).
  - [x] **M8** — drop the observation FK from `0003` → the test that a link cannot name a
        non-existent observation must red (AC7).
  - [x] **M9** — drop `CHECK (mac_canon = LOWER(mac_canon))` → its raw-insert test must red (AC7).
  - [x] **M10** — drop `CHECK (rule_id <> '')` → its raw-insert test must red (AC7).
  - [x] **M11** — delete the `Ambiguous`-without-candidates guard → its direct test must red (AC5).
  - [x] 🔴 **M13** — delete the **empty-`rule_id` guard in the WRITER** (not the DDL CHECK, which is
        M10) → its direct test must red (AC5). **Added at the validation**: decision 6 names two
        writer guards and only one had a mutation. Story 5.9's M3 is the lesson — an adapter guard
        and its DDL echo must each be measured, because dropping one leaves the other carrying the
        test and the story records a red it did not earn.
  - [x] 🔴 **M12** — delete the universe check → a test handing `resolve_within` an **EMPTY**
        universe must red (`"match"` vs `"abstained"`, assertion-carried). ⚠️ **The suspicion was
        confirmed by measurement**: against the real blocker, deleting the check leaves the **entire
        suite green (397/397)**, because `candidates` is TOTAL. The seam of decision 12 is what
        makes this mutation mean anything, and the narrowed-universe test is not optional.

- [x] **T8 — register and docs (AC9, AC10)**
  - [x] Append this story's section to `deferred-work.md`: the **eleven** entries of §5 with their
        verbs and measurements, plus the four new residues of AC9.
  - [x] Update `docs/project-context.md` **and** `CLAUDE.md` with the same numbers, then grep both
        for every phrase they duplicate. **Both twins, in the same commit.**
  - [x] Update `sprint-status.yaml`.

- [x] **T9 — the full local gate, then the PR**
  - [x] `cargo fmt --all` · clippy **twice** (§8) · `cargo test --workspace --locked` **with the DB
        running** · `cargo xtask ci`.
  - [x] Re-check the corpus report: **11 unanswerable, `passed() == false`** (AC10).
  - [x] Then `code-review`, then push → PR → green CI → **squash merge**. Never push to `master`.
        `done` is the MERGE's business here, not the review's.

---

## Dev Notes

### Existing shapes to follow, not reinvent

- **The adapter idiom** is §4. Read `repo.rs` before writing a line; it answers every structural
  question this story raises, and it is 657 code lines.
- **`datetime_literal`** [`repo.rs:330-333`] is the single instant-formatting site. Do not write a
  second format string.
- **`MacAddr`'s `Display`** is lowercase colon-separated and **is** `mac_canon`. There is no second
  canonicalisation, and `0003`'s CHECK is what stops a future writer inventing one.
- **`insert_identity_link` derives `outcome`/`rule_id`/`abstention_cause` from ONE `match` over the
  `Decision`** [`repo.rs:353-396`]. The resolver hands it a `Decision`; it does not assemble those
  columns itself. That is what makes the DDL CHECKs a second line of defence rather than the only
  one — and it is also why story 5.9's M3 first came back GREEN.
- **`current_subject_of`** [`repo.rs:401-406`] is the single derivation site for the sentinel. Do not
  compute it in the resolver.

### Compile-level facts — each costs an hour if it is discovered under `rustc`

- `identity::l1::join` and `identity::blocking::candidates` are **deliberately not flat-re-exported**
  from `opencmdb_core` [`lib.rs:46-60`]. Reach them as `opencmdb_core::identity::l1::join` and
  `opencmdb_core::identity::blocking::{candidates, CandidatePair}`. `verdict_for_pair` is
  `pub(crate)` in core and is **not reachable at all** — `decide_pair` is the entry point.
- `L1Key` is `pub type L1Key = (L2DomainId, MacAddr)` [`l1.rs:89`] — a tuple, so it destructures.
- `join` returns `BTreeMap<L1Key, BTreeSet<ObsId>>`; the set gives you the witness by
  `group.iter().find(|id| **id != o.obs_id)`.
- `CandidatePair::new(a, b)` returns `Option` and orders its fields; `low()`/`high()` read them.
  **The ordering carries no meaning** — it is not chronology.
- `Decision { conclusion, verdict_vector, ruleset_version }`; `Conclusion::{Match{rule}, NoMatch{rule},
  Abstained{cause}}`; `RuleId(pub String)`; `RulesetVersion(pub u32)`; `CURRENT_RULESET_VERSION` is
  `RulesetVersion(1)` and lives in `identity::l1`.
- `RuleVerdict.evidence` is `Vec<ObsId>`, **sorted** by `verdict_for_pair` so the pair is unordered.
- `Observation { obs_id, connector_id, observed_at, scope: Scope { l2_domain, vantage }, facts, raw }`.
  `Timestamp = chrono::DateTime<chrono::Utc>`.
- `MariaUnit::executor()` is `pub(crate)` and returns `&mut MySqlConnection`; the query bodies take
  `E: Executor<'e, Database = MySql>`, which `&mut MySqlConnection` satisfies. **Re-borrow with
  `&mut *conn` between calls** or the first call moves it.
- 🔴 **`chrono::Utc::now()` DOES NOT COMPILE here** — `error[E0599]`. `chrono` is pinned
  `default-features = false` in **both** crates to keep its `clock` feature off, and
  `crates/opencmdb-bin/Cargo.toml:22-23` says why: feature unification would otherwise re-enable it
  in `opencmdb-core` and let the domain read the clock (D19). The only clock in reach is SQL's
  `NOW(6)`.
- 🔴 **`datetime_literal` is a PRIVATE `fn`** [`repo.rs:331`], so the resolver cannot call it and a
  second format string is the natural reflex. **Make it `pub(crate)`** — that is the change that
  keeps the "single formatting site" sentence true.
- 🔴 **The id newtypes have no `FromStr`.** `find_interface_by_l1_key` must `uuid::Uuid::parse_str`
  the `CHAR(36)` it reads back and launder the failure through `sqlx::Error::Decode`. There is no
  shorter route today.
- ⚠️ **sqlx 0.9 rejects `sqlx::query(&format!(…))` at compile time.** Write one static statement per
  table for the test cleanups; do not loop over table names.
- `sqlx::migrate!("./migrations")` embeds the directory at COMPILE time (§8).

### What a reviewer will challenge, and the answer that is already measured or decided

| challenge | answer |
|---|---|
| *"`epics.md` says exactly ONE interface and you write N."* | §2. `join` loops over `keys_of`, `l1.rs:186` says an observation may carry several MACs, and story 5.9 measured a synthetic multi-MAC observation being refused its second link. ⚠️ **Do not answer "`multi-nic` is a committed family"** — measured false as support: no committed observation carries two MACs, and `multi-nic` expects `l2-*` rules. Guy's arbitration at this contexting; `epics.md` deliberately not edited. |
| *"The blocker is TOTAL, so calling it is decoration."* | §3. It is where the universe is DEFINED, `epics.md:1604` requires the order, and F17's `dormant` already plans to make it exclude. A pass that read the key directly would be correct today and silently wrong later. |
| *"Why not build the groups from the `Match` pairs?"* | §3, and **M2 measures it**: the quantifier is existential, so A–B on `k1` and B–C on `k2` fuse A with C although they share no key. |
| *"A singleton link's rule came from no rule firing."* | Decision 5, stated plainly rather than hidden. The alternative — `decide_pair(o, o)` — re-opens the self-pair that story 5.6 closed in the type and 5.7's review found re-opened once already. |
| *"An abstention with zero candidates makes *the ambiguity is DATA* a convention."* | Decision 6: at L1 the only reachable cause is `AbsenceOfProof`, which correctly has none. `Ambiguous` with none is REFUSED by a guard, tested directly because the engine cannot reach it. |
| *"`LEAST`/`GREATEST` is comparison in SQL and D10 forbids that."* | Decision 7: D10 forbids SQL descending into a **domain value** comparison because identity is the product; a seen-window is bookkeeping, no value is under judgement, and MariaDB is the only engine. The alternative is registered with its owner clause. |
| *"You added no bound on the quadratic universe."* | Decision 8: D13 says 90k pairs at 300 hosts is not a concern, the count is **asserted** by formula, the wall-clock is recorded, and the entry stays open. A threshold with no measured need is speculation. |
| *"The resolver has no production caller — same defect as the blocker had."* | Decision 3. Its named consumers are stories 5.10 and 5.11, in this epic; the residue is registered with story 5.14 as owner. Wiring `main.rs` is a behaviour change no AC asks for. |
| *"Adding the observation FK reds eight tests — that is a regression."* | Register entry #5, which names this story as owner precisely because it is the first that writes links from observations that exist. The tests are FIXED (T5), not the guard dropped. |

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 5.9b`] — the five criteria, and AC1's
  *"exactly ONE interface"* which §2 corrects with the measurement.
- [Source: `_bmad-output/planning-artifacts/epics.md#Story 5.10`, `#Story 5.11`] — the two named
  consumers of this pass.
- [Source: `_bmad-output/planning-artifacts/architecture.md:931`] — **D13**: candidate generation
  (blocking) → verdicts → three-way decision.
- [Source: `_bmad-output/planning-artifacts/architecture.md:984-1011`] — **D13**: L1 is a
  deterministic join on `(l2_domain, mac) -> interface`; the blocking-recall assertion; *"without
  blocking, abstention has no denominator"*.
- [Source: `_bmad-output/planning-artifacts/architecture.md:1013-1049`] — **D14**: the link as an
  SCD2 entity, the ambiguity as a link, the purge test, `ruleset_version` mandatory.
- [Source: `_bmad-output/planning-artifacts/architecture.md:1434-1446`] — **D21**:
  read-your-own-writes inside the writer actor, the transaction unit, *"an identity decision is
  NEVER split across two transactions"*.
- [Source: `_bmad-output/planning-artifacts/architecture.md:1462-1479`] — **D21**: the NULL trap,
  `OPEN_END`, `NIL_INTERFACE`, and no unique index on the L1 key.
- [Source: `_bmad-output/planning-artifacts/architecture.md:3364`] — *"the engine never touches the
  clock"*.
- [Source: `crates/opencmdb-core/src/identity/l1.rs`] — `join`, `keys_of`, `decide_pair`,
  `verdict_for_pair`'s existential quantifier, the one-`l2_domain` warning.
- [Source: `crates/opencmdb-core/src/identity/blocking.rs`] — `candidates`, `CandidatePair`, totality
  by decision.
- [Source: `crates/opencmdb-bin/src/repo.rs`] — the adapter idiom, the sentinels, the eight query
  bodies story 5.9 shipped.
- [Source: `crates/opencmdb-bin/migrations/0002_interface_and_identity_link.sql`] — the schema this
  pass fills, and the header sentence *"the re-run finds an interface by its key"*.
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md:2199-2318`] — the **nine** entries
  naming story 5.9b as owner.
- [Source: `_bmad-output/implementation-artifacts/5-9-persist-interface-and-identity-link.md`] — the
  split, the `observation -> interface` arbitration, and the two uniqueness-key arbitrations.

---

## Dev Agent Record

### Agent Model Used

_(to be filled by the dev agent)_

### Debug Log References

#### The database run (AC10)

Everything below ran against a live **`mariadb:10.11.11`** in `opencmdb-dev-db`, host port **13306**
(3306 is held by an unrelated `mariadb:11-jammy` from another project, untouched). `docker ps` was
confirmed before every trusted run, and `SELECT VERSION()` reported `10.11.11-MariaDB-ubu2204`.

🔑 **The baseline is the same with and without a database, and that is the point.** With
`DATABASE_URL` set, `cargo test --workspace --locked` reports **181 / 156 / 46** — the same counts
as without it. What changes is that the DB-backed tests EXECUTE instead of `return`ing: the bin
suite takes **0.36 s** with the database and **0.02 s** without. That timing gap is the only local
evidence they ran at all.

#### 🔴 A mutation that changed BIND ARITY hung the suite for three hours

Recorded because it cost real time and because the next reader will otherwise repeat it. The first
form of M6b replaced `valid_from`'s `?` with `NOW(6)` in the SQL **and left the matching
`.bind(datetime_literal(valid_from))` in place** — twelve binds for eleven placeholders. The suite
did not fail: it **hung**, at 0 % CPU, for ~2 h 48 min, holding `DB_TEST_LOCK` so nothing else could
run. A prepared-statement parameter mismatch desynchronises the MySQL protocol and the connection
waits for a packet that never comes.

Two consequences, both applied: **a mutation must preserve arity** — the corrected M6b drops the
placeholder AND its bind — and **the mutation driver now runs each `cargo test` under a 420 s
timeout**, so a hang is recorded as `<the suite HUNG>` instead of eating an afternoon.

#### The foreign key reds in TWO WAVES, and the second is invisible until the first is fixed

The story predicted *"14 tests across two files"*. Measured, the shape is sharper:

1. `0003` applied to a freshly created database → **12 red in `repo.rs`**, every one
   `Constraint("foreign_key")` panic-carried at `.expect()`. That is the register's "8", corrected.
2. Those twelve fixed (an `an_observation` helper inserts the row the link names) →
   **2 more appear**: `repo::tests::ingest_observation_round_trip` and
   **`main::tests::index_renders_the_real_gap`**, both **ERROR 1451**, from a
   `DELETE FROM observation_record` that does not delete links first.

🔑 **The second wave cannot be seen before the first is fixed**, because a failing test rolls back
its transaction and leaves no link behind for the cleanup to trip over. Fixed by ordering the
cleanup (children before parents) in both files.

#### The mutation table — 18 mutations, every red assertion- or panic-carried, ZERO compiler-carried

🔴 **The heading read *"every red ASSERTION-carried"* until this story's code review, and that was
FALSE.** The Acceptance Auditor re-executed the pass and measured M6b: 4 reds, of which **1 is an
assertion and 3 are `.expect()` panics** in pre-existing `repo.rs` interval-CHECK tests. Two things
are worth recording rather than quietly fixing:

- **the story's own T7 bullet already said so**, written at the validation — *"the only three reds
  were pre-existing `repo.rs` tests, panic-carried"* — and the summary written on top of it
  contradicted it. A claim falsified by its own document, one section away;
- **the cause is methodological**: the mutation driver classified the carrier by scanning the WHOLE
  test output for an assertion failure, so a MIXED set collapsed to a single label. It now reports a
  carrier **per test**. That is the same family as an oracle that restates the expectation instead of
  measuring the code.

The wording below is the one stories 5.7 and 5.9 used, and it is the true one.

| # | mutation | DB | tests red | what carried the red |
|---|---|---|---|---|
| M1 | `find_interface_by_l1_key` ignores `l2_domain` | ✅ | 1 — `one_mac_in_two_scopes_is_two_interfaces` | assertion, `left: 1, right: 2` |
| M2 | groups as connected components of the `Match` pairs | ✅ | 2 — `the_pass_does_not_fuse_a_with_c`, `one_observation_with_two_macs_lands_on_two_interfaces` | assertion, `left: 1, right: 2` |
| M3 | always mint, never look up | ✅ | 3 | assertion, `left: 0, right: 1` on the found-count |
| M4 | evidence from a constant instead of the verdict | ✅ | 1 — `a_placements_evidence_is_the_pair_the_engine_judged` | assertion, **on the EVIDENCE** |
| M5 | the abstention written as an absence | ✅ | 1 | assertion, `left: 0, right: 1` on the COUNT |
| M6a | interface window ← `NOW(6)` | ✅ | 3 | assertion, `left: "2026-08-04 11:47:56.907039"` |
| M6b | link `valid_from` ← `NOW(6)` | ✅ | 4 | 🔴 **1 assertion** (`the_stored_instants_are_the_derived_ones`) **+ 3 `.expect()` panics** (`closing_a_link_refuses_what_it_must`, `superseding_a_link_leaves_the_old_row_readable`, `two_versions_may_be_closed_at_the_same_instant` — pre-existing interval-CHECK tests) |
| M7 | `LEAST`/`GREATEST` → plain assignment | ✅ | 1 | assertion, **on `last_seen_at`** |
| M8 | drop `identity_link_observation_fk` | ✅ recreated | 1 | assertion, `left: None, right: Some(Constraint("foreign_key"))` |
| M9 | drop `interface_mac_canon_lower` | ✅ recreated | 1 | assertion |
| M10 | drop `identity_link_rule_id_not_empty` | ✅ recreated | 1 | assertion |
| M11 | drop the `Ambiguous`-without-candidates guard | ✅ | 1 | assertion, `Ok(())` vs `Err(Constraint("ambiguity_without_candidates"))` |
| M12 | drop the universe containment check | ✅ | 1 | assertion, `left: 0, right: 2` abstentions |
| M13 | drop the empty-`rule_id` guard in the WRITER | ✅ | 1 | assertion |

**Four mutations added AT the code review, for guards it measured as unheld:**

| # | mutation | DB | tests red | what carried the red |
|---|---|---|---|---|
| M14 | witness: smallest → **largest** other `ObsId` | ✅ | 1 — `the_witness_is_the_smallest_other_id_in_the_group` | assertion |
| M15 | containment tested on ONE witness instead of searched | ✅ | 1 — `withholding_one_pair_does_not_silence_the_others` | assertion |
| M16 | drop the tail loop's `abstained` dedup | ✅ | 1 — `a_repeated_obs_id_writes_one_link` | panic (the collision aborts before any count) |
| M16b | write the abstention INSIDE the group loop, i.e. per key | ✅ | 1 — `an_observation_abstains_once_however_many_keys_it_carries` | assertion |

🔑 **M16 reddened the wrong test, and that is the finding.** It was written for *"one abstention per
observation"* and reds the DUPLICATE-`obs_id` guard instead, because the property is enforced by the
tail loop iterating **observations** rather than `(group, observation)` pairs — the `abstained` set
only guards a repeated entry in the slice. **M16b** is the mutation that actually reaches the
arbitration's guard. Two mutations, two distinct properties, and neither stands in for the other.

🔑 **M16b's first run was PANIC-carried**, because the pre-arbitration failure is a uniqueness
violation that rolls the transaction back and `.expect()` fires before any count exists — story
5.9's M4/M5 lesson, met again. The test now asserts `outcome.err() == None` and the red is
assertion-carried.

**Three predictions the validation made, all three confirmed by measurement:**

- **M4's red comes from the EVIDENCE, not the rule.** `left: [obs1]` against
  `right: [obs1, obs2]`. The rule is knowable in advance, so a test checking only the rule would
  have measured a constant.
- **M7's red comes from `last_seen_at`, the end AC6 did not originally name.**
  `left: "2023-11-14 22:13:20"` against `right: "2023-11-14 22:21:40"` — the window was NARROWED to
  the older batch. A test asserting `first_seen_at` alone passes this mutation.
- **M12 needs the `resolve_within` seam.** With the universe computed internally the check is
  unreachable, `candidates` being total; handed an EMPTY universe the test reds `0` abstentions
  against `2`.

**And M2 turns §3's refutation from a derivation into a measurement.** Grouping by connected
components fuses A with C although they share no key: `left: 1, right: 2` interfaces. The story
said this had to be measured rather than quoted, and it is.

#### The reference scale, measured at last (decision 8, AC9)

⚠️ **Added at the code review, which measured that no test called `resolve` at scale** — the
quadratic assertion exercised `candidates` alone, so decision 8's *"the Debug Log records the
wall-clock of one pass"* had nothing behind it in three documents.

`one_full_pass_at_the_reference_scale`, 300 synthetic observations on 300 distinct MACs, against the
live database:

```
reference scale: n=300, pairs=44850, interfaces=300, links=300, pass=73.493341ms
```

**44 850 pairs, not D13's prose "90k"** — the figure there counts pairs the other way. The full pass
(blocking, join, 300 `decide_pair` calls, 600 SQL statements inside one transaction) takes **~73 ms**
on this machine. **No timing is asserted**: a wall-clock assertion is a flaky test on shared
hardware. What the test asserts is the pair count, the interface count read back from the database,
and that the pass completes.

#### The `ddl-collation` gate, shown to bite (AC7)

⚠️ Also added at the code review: AC7's clause was ticked with nothing recorded. `0003` declares no
column, so a temporary text column was appended to it, without a collation:

```
🔴 ddl-collation  1 text column(s) without a binary collation:
    …/migrations/0003_resolver_guards.sql:48: ALTER TABLE interface ADD COLUMN probe VARCHAR(8) …
```

It names the file and the line. Removed, the gate returns to `✅ every text column carries an
explicit binary collation`. **Green was never the evidence; this is.**

#### One collateral red, explained rather than left standing

**M6a also reds `repo::tests::every_ddl_guard_refuses_what_it_names**, with
`left: None, right: Some(Constraint("check"))`. That is not noise: the test's `interface_seen_window`
case hands `insert_interface` an INVERTED window (`first = 1_700_000_100`, `last = 1_700_000_000`)
and expects the CHECK to refuse it. M6a hardcodes `NOW(6), NOW(6)`, so the parameters never reach
the columns, both ends are equal, and there is nothing left to violate. The mutation is faithful and
story 5.9's guard is load-bearing — both facts, from one red.

### Completion Notes List

- **AC1–AC10 met.** 383 → **402 tests** (197 bin + 159 core + 46 xtask), six `xtask ci` gates green,
  `cargo fmt --check` clean, **both** clippy forms clean, `git status fixtures/` empty, and the
  committed trap corpus still reports **11 unanswerable with `passed() == false`** — verified after
  the fact, because a green gate here would have been a regression, not a win.
- 🔴 **`identity::l1::decide_singleton` is the one change to the engine**, and it closes the hole the
  validation found under a compiler: a singleton group has no pair, `insert_identity_link` requires a
  `Decision`, and both alternatives were worse — a struct literal with an empty `verdict_vector` is
  the *"merged, with no explanation"* shape D13 forbids, and composing the verdict in `opencmdb-bin`
  is what `verdict_for_pair`'s `pub(crate)` exists to prevent. It builds the one-element `Decisive`
  vector and returns `decide`'s value.
- 🔑 **No struct-literal `Decision` was needed ANYWHERE, which is better than the story predicted.**
  The excluded-pair and lone-MAC-less-observation cases both use
  `decide(Vec::new(), CURRENT_RULESET_VERSION)` — the algebra's own answer for an empty verdict set,
  and literally true here: nothing was evaluated because nothing was proposed. So the register entry
  *"the first story that reconstructs a `Decision` from somewhere other than `decide`"* is ANSWERED
  with its clause **still unmet**, not closed.
- **The register: 11 entries disposed — 6 closed, 4 answered-not-closed, 1 measured and left open.**
  ⚠️ Two of the four are dispositions the story expected to be closures: `L1Key`'s newtype is
  **refused** with its reason (the key is destructured at its one use site and never travels as a
  value), and **`count_identity_links` still has no PRODUCTION caller** — it gains two test callers
  here and nothing more, so reporting it closed would be the over-claim seven reviews have caught.
  The story's §5 said "CLOSE"; the measurement says otherwise, and the measurement wins.
- ⚠️ **A task line and its own acceptance criterion disagree, and the disagreement is reported rather
  than silently reconciled.** T6 says the transitivity case is *"synthetic, no DB needed"*; AC8, as
  corrected at the validation, requires it **through `resolve` against the database**, because a
  pure test never calls the resolver and cannot see its grouping change. **Both were written** — the
  pure one states the quantifier, the DB one carries M2 — and this note exists because `dev-story`
  may not edit a task line.
- **Decision 8's number is measured, not quoted**: `candidates(&obs).len() == 44_850` at the
  reference scale of 300 hosts, asserted in a test. D13's prose says "90k" for the same scale; the
  figure counts pairs the other way, and the register carries the correction.
- **`datetime_literal` and `open_end` were promoted out of privacy** (`pub(crate)`), the second out
  of `repo.rs`'s test module entirely: the resolver writes every current link at the sentinel, and a
  private helper is what makes a second spelling of an instant the natural reflex.
- 🔴 **CODE-REVIEWED (three layers, 2026-08-04), and it cost the story four behavioural defects.**
  Two layers ran against their own live `mariadb:10.11.11`; the Auditor re-executed all fourteen
  mutations and **reproduced thirteen exactly, including all three named predictions**. The code was
  right about the engine and wrong about four edges; the documents were wrong more often than that,
  which is the seventh consecutive story with that shape. **402 → 408 tests**, mutations **14 → 18**.
  🔑 **The review refuted a headline of mine that sat in five places**, including the commit subject:
  *"every red is assertion-carried"* is false for M6b (1 assertion, 3 `.expect()` panics), and my own
  T7 bullet had said so one section above. The driver collapsed a MIXED carrier set to one label; it
  now reports per test.
  🔴 **Guy arbitrated twice.** An observation abstains **at most once**, whatever the number of keys
  — two abstention rows collided on `ABSTAINED_SUBJECT` and failed the whole pass, and they would
  have been identical but for their id, because an abstention row names no key. And the
  smallest-other-`ObsId` witness convention is **kept and now measured** on a group of three: every
  earlier test used a group of two, where the two formulas name the same observation.
  🔑 **The worst behavioural defect was internal inconsistency**: `placement_decision` tested
  containment on ONE candidate witness while its sibling `abstention_for` filtered correctly twelve
  lines below. Measured — universe missing only `(1,2)`, and observations 1 AND 2 abstained.
  🔑 **`abstention_for` was deleted**: replacing its body with `nothing_was_evaluated()` left every
  test green, because a `Neutral` verdict carries empty evidence and the vector is not persisted.
  Its doc promised *"an abstention with an explanation"* that never reached a column.
- ⏸️ **T9's push/PR is deliberately NOT done.** The house order is `dev-story` → `code-review` → PR
  → green CI → squash merge, and `done` is the MERGE's business here.

### File List

- `crates/opencmdb-bin/src/resolver.rs` — NEW (the pass, its two entry points, the guards, **21 tests** — 15 at implementation, 6 added at the code review; the File List said 20 and the tree said 15)
- `crates/opencmdb-bin/migrations/0003_resolver_guards.sql` — NEW (three guards)
- `crates/opencmdb-bin/src/repo.rs` — MODIFIED (`find_interface_by_l1_key`,
  `widen_interface_seen_window`, `open_end` promoted out of the test module, `datetime_literal` made
  `pub(crate)`, the `an_observation` fixture helper, the ordered cleanup, the three new guard tests,
  15 link tests pointed at real observations)
- `crates/opencmdb-bin/src/main.rs` — MODIFIED (`mod resolver;`, the ordered cleanup in
  `index_renders_the_real_gap`)
- `crates/opencmdb-core/src/identity/l1.rs` — MODIFIED (`decide_singleton` + 3 tests)
- `crates/opencmdb-core/src/identity/cascade.rs` — MODIFIED (the struct-literal residue's doc: the
  clause stands unmet, and why)
- `_bmad-output/implementation-artifacts/deferred-work.md` — MODIFIED (this story's section)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — MODIFIED
- `docs/project-context.md`, `CLAUDE.md` — MODIFIED (AC10, including the `absence_of_proof` token
  both twins spelled as the Rust variant name)

---

## Change Log

| date | note |
|---|---|
| 2026-08-04 | **DONE.** PR **#67** squash-merged as **`94314e9`** after a green CI run. `master` carries **408 tests** (203 bin + 159 core + 46 xtask) and six green gates. **Epic 5 is now 12/17.** The branch is deleted locally and on `origin`. |
| 2026-08-04 | **CODE-REVIEWED** (three layers; two against their own live `mariadb:10.11.11`, and the Acceptance Auditor re-executed the whole mutation pass). **2 arbitrations by Guy, 23 patches, 5 deferrals, 0 dismissed. 402 → 408 tests**, mutations **14 → 18**. 🔴 **The review refuted a headline of mine in five places, the commit subject included**: *"every red is ASSERTION-carried"* is false for M6b — 1 assertion, 3 `.expect()` panics — and my own T7 bullet said so one section above. Root cause: the driver read the carrier off the WHOLE output, so a mixed set became one label; it now reports per test. 🔴 **Four behavioural defects, three of them found separately by two layers**: `placement_decision` tested containment on ONE witness while its sibling filtered correctly twelve lines below (universe missing only `(1,2)` → observations 1 AND 2 abstained); a multi-key abstention collided on `ABSTAINED_SUBJECT` and rolled the whole pass back; `abstention_for` had **no observable effect** (its body replaced by `nothing_was_evaluated()` left every test green — a `Neutral` verdict's evidence is empty and the vector is not persisted); and a repeated MAC-less `obs_id` wrote two colliding links. 🔑 **Guy's two arbitrations**: one abstention per observation whatever the key count — the deciding measurement being that an abstention row names no key, so the two rows would be identical but for their id — and the witness convention kept but MEASURED on a group of three, since every earlier test used a group of two where "smallest other" and "largest other" coincide. Also: the reference-scale pass and its wall-clock existed in no test (now **44 850 pairs, 300 interfaces, ~73 ms**), AC7's *"shown to bite"* had no record (the gate now names `0003…sql:48`), §9 did not apply its own *"a divergence is a FINDING"* rule to its own three divergences, `sprint-status.yaml` contradicted itself, a doc cited a test I had invented, and `RESOLVER_RULESET_VERSION` was dead code with a false doc. |
| 2026-08-04 | **IMPLEMENTED → `review`.** 383 → **402 tests** (197 bin + 159 core + 46 xtask), six gates green, `fmt --check` clean, both clippy forms clean, `fixtures/` untouched, and the trap corpus still **11 unanswerable / `passed() == false`** (re-checked, because a green gate here would have been a regression). All **14 mutations run WITH a live `mariadb:10.11.11`**, and **every red is assertion-carried — zero compiler-carried**. 🔴 Three of the validation's predictions confirmed by measurement: M4 reds on the EVIDENCE and not the rule; M7 reds on `last_seen_at`, the window end AC6 had not named; and M12 is a no-op without the `resolve_within` seam, reddening `0` abstentions against `2` only when handed an empty universe. **M2 turns §3's refutation into a measurement** — connected components fuse A with C, `left: 1, right: 2`. 🔑 **No struct-literal `Decision` was needed anywhere**: `decide(vec![], _)` is the algebra's own answer for an empty verdict set, so the register's *"first story that reconstructs a `Decision` outside `decide`"* clause is ANSWERED and still UNMET rather than closed — as is `count_identity_links`, which gains two test callers and still no production one. 🔴 The foreign key reds in **two waves**: 12 in `repo.rs`, then 2 more (one in `main.rs`) that are **invisible until the first twelve are fixed**, because a failing test rolls back and leaves no link for the cleanup to trip over. ⚠️ And a mutation that changed BIND ARITY **hung the suite for 2 h 48 min at 0 % CPU** holding `DB_TEST_LOCK` — a prepared-statement parameter mismatch desynchronises the protocol; the driver now runs each suite under a timeout. |
| 2026-08-04 | **GAP-HUNT layer done** — the story BUILT end to end in an isolated worktree against a live `mariadb:10.11.11`, **398 tests** (196 bin + 156 core + 46 xtask), six gates green, `fixtures/` untouched, every mutation executed. **7 HIGH, 5 MEDIUM, 7 LOW, all applied.** 🔴 Two arbitrations by Guy: a singleton's `Decision` comes from a **new `identity::l1::decide_singleton`** rather than from a struct literal or from the resolver composing a verdict — the one change this story makes to the engine, and the honest closure of the residue `cascade.rs:345-347` registers; and **the pass is NOT idempotent** (twice over one slice is `Err(Constraint("unique"))` and a full rollback), which is story 5.11's, so AC4's two tests move to different observations on the same key. 🔑 **Four prescriptions were not executable or measured nothing**: `Utc::now()` does not compile (`chrono` is `default-features = false` in both crates to keep `clock` off, deliberately), so M6 became `NOW(6)` in two variants; **M12 left the entire suite green** and needed the `resolve_within` seam to mean anything; **M2 was a no-op** against a transitivity test that never calls `resolve`; and **M6b had no target at all** — no test read a link's `valid_from` back, story 5.9's AC3 defect one story later. Also: the foreign key reds **14 tests across `repo.rs` AND `main.rs`**, not eight; `every_ddl_guard_refuses_what_it_names` now passes for the wrong reason and **never reds**, so nothing routes the dev to it; the widening test's red comes from `last_seen_at`, the end AC6 did not name; the reference scale is **44 850** pairs, not D13's "90k"; and `resolver.rs` needs `#![allow(dead_code)]` or the **CI** clippy form fails with 8 errors on the story's own deliverable. |
| 2026-08-04 | **FACT-CHECK layer done** (fresh context, read-only): **78 claims measured, 69 true, 6 false, 6 gaps.** All applied, each re-measured independently first. 🔑 **44 / 44 line citations correct and every count exact** — the defects were entirely in quoted tokens, one corpus claim, and the AC↔task↔mutation seam. The three HIGH: the persisted token is `absence_of_proof` and not the variant name `AbsenceOfProof` (which appears in no committed byte); AC3's rule SET is unmeasurable because `l1-distinct-mac` is unwritable by three independent steps (`Disqualifying` → `NoMatch` → a row decision 9 never writes); and 🔴 **`multi-nic` is not the multi-MAC shape — no committed observation carries more than one MAC** (max 1 across 13 streams), so a sentence three documents use as evidence supports nothing. Also: M1 required editing a file the story forbids itself, M6 and M8 named tests that did not exist, one guard had no mutation (now M13), and two register entries owned by CONDITION are met by decisions 5 and 6 — **nine entries became eleven**. ⏳ The gap-hunt layer is still running. |
| 2026-08-04 | Story contexted on `master` at `47bdca2` (383 tests, six green gates, clean tree). **Three arbitrations taken with Guy**: (1) `epics.md`'s AC1 *"exactly ONE interface"* is **falsified by `join`** and widens to *one interface per L1 KEY* — `epics.md` deliberately not edited, correction owned by Epic 5's retrospective; (2) the mechanism is **`join` NAMES, the blocker and `decide_pair` JUSTIFY**, with the connected-component alternative refuted by the existential quantifier and required to be MEASURED (M2); (3) the resolver is **not wired into `main.rs`** — production code with named consumers in 5.10 and 5.11, residue registered with 5.14. Six further decisions measured against the tree, of which decision 5 (a singleton is placed by the key, and `decide_pair(o, o)` is REFUSED because it re-opens the self-pair story 5.6 closed in the type) and decision 6 (`Ambiguous` is unreachable at L1, so the only abstention cause this pass writes is `AbsenceOfProof`) are the two a reviewer will challenge first. **Nine registered entries** counted and dispositioned. ⏳ **Validation by two fresh-context agents is still owed, and the gap-hunt MUST run a live `mariadb:10.11.11` on port 13306.** |

---

### Review Findings

Three-layer review (Blind Hunter · Edge Case Hunter · Acceptance Auditor), 2026-08-04, on
`master...18844ea`. **Two layers ran against their own live `mariadb:10.11.11`** (ports 13309 and
13310, both stopped, `kesh-mariadb` never touched) and the Auditor **re-executed all fourteen
mutations independently**. Every finding below that names a measurement was re-measured by the
implementer before being written down.

🔑 **Thirteen of the fourteen mutations reproduced exactly, including all three named predictions.**
The defects are overwhelmingly in the story's ACCOUNT of itself — the seventh consecutive story with
that shape — with four exceptions in behaviour, three of which two independent layers found
separately.

- [x] [Review][Patch] 🔴 **RESOLVED BY GUY (2026-08-04): at most ONE abstention link per observation, whatever the number of keys.** The deciding measurement is that an abstention row names no key — `identity_link` has `observation_id`, a NULL `interface_id` and nothing else — so the two rows in conflict would be **identical but for their id**: the duplicate is not lost information, it is the same sentence twice. No schema change, the nil sentinel stands, and an observation may still be PLACED on N interfaces while abstaining elsewhere. _(Original finding: a multi-key abstention is unwritable and aborts the entire pass)_ — an observation abstaining on two of its L1 keys writes two links with `interface_id = NULL`, both landing on `ABSTAINED_SUBJECT`, and `identity_link_one_current` refuses the second: measured `Err(Constraint("unique"))` with **0 links left behind**. This is the mirror of the bug story 5.9's second arbitration fixed, in the one column that arbitration left alone. Unreachable through `resolve` today (`candidates` is TOTAL) but reachable through `resolve_within`, which exists precisely so narrowing is exercisable. **Found by all three layers.** Options: a key-scoped abstention sentinel, or one abstention link per observation regardless of key count [`resolver.rs:190`, `repo.rs:517`, `0002…sql:77`]
- [x] [Review][Patch] 🔴 **RESOLVED BY GUY (2026-08-04): keep the smallest-other-`ObsId` convention and MEASURE it on a group of three.** A test, not a redesign, and it keeps D19's property that a link carries the evidence the rule actually used. The min→max swap becomes **mutation M14**, which must red. _(Original finding: the witness policy is measured by nothing)_ — mutation MUT-B replaced "smallest other `ObsId`" with "largest" and **all 402 tests stayed green**, although the persisted evidence changes completely on a group of three. Every committed test uses a group of TWO, where the two formulas name the same observation. Decision 4 calls this determinism *"what story 5.10 replays"*, so it is an unmeasured load-bearing claim. Options: keep the convention and measure it on a group of ≥3, or make a placement's evidence the whole group [`resolver.rs:231`]

- [x] [Review][Patch] `placement_decision` consults ONE witness where it should search for any proposed pair — with the universe missing only `(1,2)`, observations 1 and 2 both abstain although `(1,3)` and `(2,3)` are proposed; **re-measured by the implementer**, `abstentions=2 links=3`. Its sibling `abstention_for` already filters correctly. Three sentences are false as a result, AC1's included [`resolver.rs:231-237`]
- [x] [Review][Patch] 🔴 **"every red is ASSERTION-carried" is FALSE for M6b** — re-measured: 4 reds, **1 assertion + 3 `.expect()` panics** in pre-existing `repo.rs` interval-CHECK tests. The headline appears in the Debug Log, the Change Log, both doc twins and the commit subject; the story's own T7 bullet already recorded the truth underneath it. The driver's carrier analysis collapsed a MIXED set to one label — a methodological defect worth its own note [story Debug Log, `CLAUDE.md`, `docs/project-context.md`]
- [x] [Review][Patch] `abstention_for` has no observable effect — replacing its body with `nothing_was_evaluated()` leaves all 402 tests green, and the two persisted rows are byte-identical, because a `Neutral` verdict carries EMPTY evidence and the vector is not stored. Its doc claims *"an abstention with an explanation"* [`resolver.rs:251-268`]
- [x] [Review][Patch] Decision 8's reference-scale **pass** and its wall-clock exist nowhere — `the_universe_is_quadratic_and_its_size_is_asserted` calls `candidates` only, never `resolve`, and no timing is recorded. Three documents prescribe it [`resolver.rs:1049`, story §498/868/1004]
- [x] [Review][Patch] `RESOLVER_RULESET_VERSION` is dead and its doc is false — `grep` over the workspace returns only its own definition line [`resolver.rs:371`]
- [x] [Review][Patch] `open_end`'s doc cites `the_sentinel_instant_renders_as_the_sentinel_literal`, **a test that does not exist**; the assertion lives in `the_two_sentinels_are_pinned` [`repo.rs:248`]
- [x] [Review][Patch] `sprint-status.yaml` says `review` in its status line and *"NEXT = `dev-story` 5.9b"* with **398 tests** in its narrative — the same self-contradiction four of story 5.9's review patches were, one file over
- [x] [Review][Patch] §9's own rule — *"a divergence is a FINDING, not a variation"* — went unapplied on three divergences: **398 → 402** tests, `resolver.rs` **281 → 390** code lines, `repo.rs` **763 → 773**
- [x] [Review][Patch] `#! The clock is never read` heads a file that calls `uuid::Uuid::now_v7()` twice per link — a v7 UUID embeds a wall-clock millisecond and goes into the primary key of every interface and link, so story 5.10's *"bit for bit"* can only mean *modulo ids* [`resolver.rs:40, :163, :319`]
- [x] [Review][Patch] `resolve`'s *"structural rather than a promise"* is false — it takes `&mut MySqlConnection`, so a pooled connection compiles and runs the pass in autocommit; measured: a failing pass left **2 interfaces and 2 links committed** [`resolver.rs:103-107`]
- [x] [Review][Patch] A repeated MAC-less `obs_id` writes two links and fails `Constraint("unique")`, while a repeated MAC-carrying one is deduped by `join` — the tail loop iterates the raw slice [`resolver.rs:200-208`]
- [x] [Review][Patch] `Resolution::candidate_pairs`'s doc promises `n(n-1)/2` *"over this slice"*; it is `universe.len()`, whatever the caller passed [`resolver.rs:87-89`]
- [x] [Review][Patch] AC7's *"the `ddl-collation` gate is shown to bite"* has no recorded measurement, though T4 is ticked
- [x] [Review][Patch] The File List says `resolver.rs` carries **20 tests**; it carries **15** (the arithmetic elsewhere is right: 402 − 383 = 19 = 15 + 1 + 3)
- [x] [Review][Patch] AC1's *"the module doc states decision 5's singleton case"* is unmet — the singleton is documented on `placement_decision` and `decide_singleton`, not in the module doc
- [x] [Review][Patch] AC3 demands a test asserting `rule_id = 'l1-exact-mac'`; it is asserted on the singleton path only, never for a group of ≥ 2
- [x] [Review][Patch] `raw_link` lost its doc comment to the newly inserted test, which now carries a sentence false of itself [`repo.rs`]
- [x] [Review][Patch] `0003` names no precondition and no recovery — a database carrying story-5.9-era links fails ERROR 1452, and the failure sticks as `Dirty(3)` even after the offending rows are deleted, because `_sqlx_migrations` records the failure and MySQL DDL is not transactional
- [x] [Review][Patch] `write_link` keeps only `verdict_vector.first()`'s evidence while its doc says the evidence is *"the decision's own"* — inert at L1, silent the day Epic 6 emits a second verdict [`resolver.rs:312-316`]
- [x] [Review][Patch] `guard_decision(decision, &[])` is hard-coded at the only call site, so the `Ambiguous` branch would abort a legitimate ambiguity rather than write it — the inverse of FR16 — and nothing says who fills that slice [`resolver.rs:311`]
- [x] [Review][Patch] A group that abstains wholesale still mints an interface nothing points at, and interfaces are never purged [`resolver.rs:151-177`]

- [x] [Review][Defer] Two concurrent passes can mint two interfaces for one L1 key — plain `SELECT` under REPEATABLE READ against a deliberately non-unique index. D21's single writer actor makes it unreachable today; the precondition is unstated
- [x] [Review][Defer] `widen_interface_seen_window` ignores `rows_affected()`, so a non-existent id returns `Ok(())` — the silent-success shape story 5.9's review closed in `close_identity_link`
- [x] [Review][Defer] An `observed_at` at or past `OPEN_END` fails the whole batch with an opaque `check`, naming no column
- [x] [Review][Defer] Sub-microsecond `observed_at` is truncated by `datetime_literal`, so two distinct instants store as one — already registered by story 5.9 with story 5.10 as owner
- [x] [Review][Defer] The eleven register entries still stand unmarked at their original lines, closed 120 lines below — the house pattern, but the next story's grep inherits a misleading count
