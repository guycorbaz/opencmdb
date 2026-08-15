# Story 6.3: NFR5's two remaining assertions are measured, not asserted

Status: review

<!-- ⚠️ CONTEXTED 2026-08-15, the day after story 6.2 merged (PR #91 → `ead7738`, docs flipped by
     PR #92 → `72dfe1f`). The tree this story extends is master at `72dfe1f`.
     🔴 THE STORY'S WHOLE DIFFICULTY WAS MEASURED AT CONTEXTING, NOT ASSUMED: both assertions are
     TRUE BY CONSTRUCTION on the committed tree — there is NO `UPDATE observation_record` anywhere
     in `crates/` (one INSERT, `repo.rs:277`, and nothing else), and the documenting transaction
     issues exactly one SELECT on `observation_record` plus N INSERTs on `declared_attribute`
     (`document.rs:100-140`). A before/after test therefore CANNOT FAIL on this tree. That is
     Epic 5's dominant defect class, and `epics.md:1792` names it inside the AC itself — see §1.
     ✅ VALIDATED 2026-08-15, TWO fresh-context layers (fact-check + gap-hunt), the gap-hunt in
     an isolated worktree against its own mariadb:10.11.11. 🔑 BOTH BUILT A PROTOTYPE OF THE
     EIGHTH GATE rather than reasoning about one, and they agree on every central measurement:
     the EMPTY allowlist holds (36 files, docker/ included), M1b is GREEN behaviourally and RED
     on the gate, and §4's (0,2) boundary is exact. All corrections APPLIED — see the discharged
     obligations. 🔴 TWO ARBITRATIONS WERE RAISED BY THE VALIDATION AND BOTH ARE TAKEN
     (Guy, 2026-08-15): (i) AC2's "a divergence opens" is UNSATISFIABLE as epics.md words it —
     the test DELETES the documented sighting and the limit is registered (§4b); (ii) the eighth
     gate + its prescribed rustdoc breaks the file-size ceiling — the gate lands in its own
     xtask module (§3b). Each is recorded WITH the alternatives refused.
     ⚠️ THE STORY IS READY FOR `dev-story`; nothing is blocked. -->

## Story

As the next developer,
I want the never-overwrite invariant proven on the gesture that could break it,
So that the two assertions parked since Epic 5 stop waiting for a precondition that now exists.

**And this is the story that closes NFR5 — the last of the three assertions, and the two that
have been registered as NOT COVERED since story 5.12 on 2026-08-07.** The register row is
explicit (`deferred-work.md:2894-2900`): 5.12 covered assertion **3** (*no code path writes a
declared field with a non-human author*) and FR13's blindness corollary, and left assertions
**1** and **2** owned by *"the triage epic"* because *"both need the `document` gesture … neither
of which exists"*. Story 6.2 shipped that gesture yesterday. **This story writes no feature: its
deliverable is that the feature could not have overwritten anything, and that a future change
which tried would be caught.**

---

## What this story does NOT do

- it does **not** put a button on the page — story 6.4, after Epic 6b. A diff touching
  `crates/opencmdb-bin/templates/` or `page.rs` is a **FINDING**: stories 6.1–6.3 are
  backend-only and touch no screen (`epics.md:1728`, `sprint-change-proposal-2026-08-13.md:50`);
- it does **not** add, change or remove a refusal, a status code, a route or a column. **No
  migration.** The product's behaviour is byte-identical before and after this story; what
  changes is what a future change can no longer do silently;
- it does **not** implement `document-field` (FR13(b), Epic 7's), an `entity`/`device` table or a
  `state` column (story 6.5's schema story);
- it does **not** re-open the CSRF mechanism, `auth.rs`, `is_public` or the Basic surface —
  stories 6.1/6.2 settled those, and their residuals are Epic 19's;
- it does **not** hold D15's sibling rule (*`declared_attribute.entity_id` is NEVER updated —
  `architecture.md:1064`*). That is a different invariant on a different table; §3 says so and
  registers it to story 6.5 rather than absorbing it;
- it does **not** claim to make a violation IMPOSSIBLE. §3's gate is a **TRIPWIRE on story
  5.12's precedent**, stated at that width and no wider;
- it does **not** edit `epics.md` — and it must NOT, because §1 and §4 both register divergences
  from the epic's own text;
- it does **not** add a crate. The lockfile is untouched.

---

## 1. 🔴 Both assertions are TRUE BY CONSTRUCTION today — and that is the whole difficulty

**Measured at contexting, on master `72dfe1f`:**

| what NFR5 forbids | what the tree does today | measurement |
|---|---|---|
| the act of documenting touches the observed side | the documenting transaction runs **one SELECT** on `observation_record` and **N INSERTs** on `declared_attribute` — it never names `identity_link`, `interface` or `link_candidate` | `document.rs:100-140` |
| any code updates an observation | there is **no `UPDATE observation_record` anywhere in `crates/`** — one INSERT (`repo.rs:277`) and **no other write** (three SELECTs also name it: `:233`, `:298`, `:330`); every `DELETE` is test cleanup, plus `docker/seed-example.sql:24` live and `:49` commented out — the latter a free GREEN probe | `rg observation_record crates/ -g '*.rs'` — 19 matches, zero UPDATE |
| ingestion alters a declared field | the ingest path is `scan_pass::poll_ingest_resolve` → `repo::insert_observation` → `resolver::resolve`; **not one of them names `declared_attribute`** | `scan_pass.rs:75-101` (ingest) and `:130` (the `resolve` call); `resolver::resolve` is at `resolver.rs:207`, `resolve_within` at `:228` |
| the divergence computation consults provenance | `gap::reconcile` takes `declared: &[(String, String)]` — provenance is **not in its signature**, and the store read `load_declared_attributes` selects three columns | `gap/mod.rs:104`, `repo.rs:312-314` |

🔴 **Read the consequence plainly: a test that documents an observation and then compares the
row before and after passes on this tree, and would pass just as well with its assertions
deleted.** Nothing can move the row, so nothing the test asserts is at stake.

**That is exactly the class Epic 5's retrospective named at its end, after finding it in at
least nine of twenty stories** (`epic-5-retro-2026-08-12.md:103-104`):

> **A guard placed where the defect cannot occur reads as coverage and is none.**

and its remedy is retro **action item 2** (`epic-5-retro-2026-08-12.md:200-203`), verbatim:

> *"**Concretely: for each new guard, name the code path where the defect could be written, and
> check the guard is on THAT path.**"* — Owner: whoever writes a story. Effective immediately.

⚠️ `epics.md:1792` is that action item applied literally, and it is this story's third AC:
*"each guard names the code path where the violation could be WRITTEN, not merely a path where
it would be visible."* **The AC is not decoration; it is the reason the story is hard.**

🔑 **And the architecture had already prescribed both the guard and its shape.** D10
(`architecture.md:524-525`): *"**NFR5 extended:** the anti-regression test must also verify that
after an adoption the observed record is bit-for-bit unchanged and the link holds."* And D10's
companion rule for how such a guard is carried (`architecture.md:557-559`):

> *"**AC-8b is enforced by the compiler:** the effective-value module simply does not receive the
> `FieldDecision` store handle in its signature. **An architecture test you can bypass by
> carelessness is not a test, it's a post-it.**"*

**So the story's central prescription is a PAIR of carriers, and they are different in kind:**

- **(a) a BEHAVIOURAL guard** — a bit-for-bit snapshot comparison across the documenting gesture
  and across an ingestion — carried by a mutation that plants the write **at the site where a
  future story would plant it**, not at the route;
- **(b) a SOURCE guard** — an eighth `cargo xtask ci` gate (§3) — because **the AC asks for the
  absence of a code path, and you cannot measure the absence of code by running code.** That
  sentence is story 5.12's, and 5.12 is the precedent this story follows rather than re-derives.

**Neither carrier subsumes the other, and §7's M1b proves it — ✅ MEASURED AT VALIDATION, by
BOTH layers independently, each having BUILT a prototype gate rather than argued from one.**
A `UPDATE observation_record` planted in a **new, uncalled** `repo.rs` fn leaves the full
workspace suite GREEN against a live MariaDB (361/161/62, `DATABASE_URL` set, 0 failed) and
REDS the gate at `repo.rs:1182` with an exact located verdict. M1b re-measures it at dev; it is
no longer a prediction.

---

## 2. The two instruments — and neither exists today

### 2a. The observation snapshot: it must be written, and `snapshot_links` is its shape

**Measured: there is no bit-for-bit snapshot helper for `observation_record`.** The only
**production** per-table snapshot helper in the workspace is `repo::snapshot_links`
(`repo.rs:1242`, story 5.10) — `resolver.rs:847`'s `interface_windows` is a second one, but
`#[cfg(test)]`, and it is the `CAST(… AS CHAR)` precedent cited below. The closest existing read is `repo::load_observation_by_id` (`repo.rs:205`), which names
all seven columns — but it is **semantic, not byte-level**: `observed_at` goes through
`DATE_FORMAT` + `parse_from_rfc3339` (`repo.rs:231`, `:242-244`) and `facts` through
`serde_json::from_str` (`:245-246`). A round-trip through Rust types cannot see a byte that
changed and re-serialised the same.

**Prescribed:** a `#[cfg(test)]` helper in `repo.rs`'s test module

```
SELECT id, connector_id, CAST(observed_at AS CHAR), l2_domain, vantage, facts, raw
  FROM observation_record ORDER BY id
```

returning all **seven** columns as text. The `CAST(… AS CHAR)` idiom is the codebase's own for
reading instants back raw — `sqlx` is built without its `chrono` feature (`repo.rs:1231-1237`),
and `resolver.rs:847`'s `interface_windows` is the precedent.

⚠️ **`raw` is `LONGTEXT NULL`** (`0001_initial.sql:32`) — the helper's tuple must carry
`Option<String>` for it, and the test data must exercise a **non-NULL** `raw`, or the column is
in the comparison and carries nothing. §7's M6 is the control that measures this.

🔑 **Why all seven and not "the decision-bearing ones"**: story 5.10 deliberately EXCLUDED the
row id from `LinkSnapshot` because a v7 UUID embeds a wall clock and a replayed link cannot
carry the same one (`repo.rs:1175-1184`). **That reasoning does not transfer here** — nothing is
replayed, the row must be the SAME row, and `id` is precisely what a mis-targeted write would
change. *An exclusion justified elsewhere is not justified here; the honest default is every
column, and every exclusion is a decision that needs its own sentence.*

### 2b. "The link is intact" — the instrument exists, and it is not `snapshot_links`

`repo::snapshot_links` (`repo.rs:1242`) deliberately drops the row `id`. For *"the link is
intact"* the better instrument is **`repo::load_current_links_for_observation`** (`repo.rs:785`),
whose `PersistedLink` (`repo.rs:742`) **does carry `id`** — and the id is what tells "the same
link" from "an equal link written afresh". ⚠️ The two real differentiators are that `id` and the
per-observation filter; **the `current_subject IS NOT NULL` restriction is NOT one of them**, both
readers carry it — the first draft listed it as if it were, and validation corrected that.

⚠️ **`identity_link` is legitimately UPDATEd** — `repo::close_identity_link` (`repo.rs:684`,
`UPDATE identity_link SET valid_to = ?, current_subject = NULL`) is how the engine supersedes.
**So the link half gets a behavioural guard and NO gate**, and §3 says why in one sentence: a
gate over a table the product legitimately mutates would need an allowlist as long as its
callers, which is a gate that means nothing.

---

## 3. 🔴 The eighth gate — `observed-immutable`

**The decision, and the alternative refused.** The alternative was *behavioural tests only*,
argued as *"a gate is scope creep in a story that ships no feature"*. It is refused because the
AC asks for the absence of a code path and §1's table shows a behavioural test cannot see one:
the violating write does not exist yet, and the test only runs what exists. 5.12 met the same
wall and answered it the same way — *"you cannot measure the absence of code by running code"* —
and that gate has since caught what reading did not.

**The gate, precisely:**

- **name** `observed-immutable`, the project's **eighth** `cargo xtask ci` gate (`main.rs:162-186`
  today wires seven: dependency-frontier, ddl-collation, vocabulary, fixture-manifest, file-size,
  float-free, declared-authorship);
- **perimeter** `.rs` **and** `.sql` under `AUTHORSHIP_ROOTS` — `["crates", "docker"]`
  (`xtask/src/main.rs:1124`). It must **reuse that constant**, not declare a second one;
- **verbs: every OVERWRITING form, not `UPDATE` alone** — table `observation_record`. 🔴 **The
  first draft said *"verb `UPDATE` only"* and validation MEASURED two holes it opens**, both of
  which modify an existing observation row and both of which came back **GREEN**:
  `INSERT INTO observation_record … ON DUPLICATE KEY UPDATE raw = VALUES(raw)` and
  `REPLACE INTO observation_record …`. The governing keyword nearest the reference is
  `insert into` / `replace into`, so an `== "update"` filter drops them. ⚠️ **Story 5.12's own
  probe corpus already carries this shape (`e12_on_dup_key.rs`) — it reds THERE only because the
  authorship gate flags every non-`select` verb, so restricting to `update` would re-open, for
  this table, the hole 5.12 had closed for its own.** And `ON DUPLICATE KEY UPDATE` is the
  ordinary gesture (*"make the ingest idempotent"*), not an adversary's.
  **Measured resolution, which PRESERVES the empty allowlist**: red when the governing keyword is
  `update`, **or** `replace`/`replace into`, **or** the statement after the reference contains
  `on duplicate key update`. Re-measured after the widening: probes red, and the **committed tree
  stays GREEN across 36 files with no allowlist** — the two committed `ON DUPLICATE KEY`
  occurrences (`0005_document_guards.sql:16`, `repo.rs:1551`) are both inside comments, which the
  comment stripper removes;
- **allowlist: EMPTY.** 🔑 That is the gate's strongest property and it is MEASURED: zero
  occurrences on the committed tree, so it needs no sanctioned site at all — where the authorship
  gate needed four. *An empty allowlist is the one form of allowlist nobody can quietly widen.*

⚠️ **`DELETE` stays out of the verb list, on story 5.12's own recorded reasoning**
(`deferred-work.md:2872-2877`): a bulk delete is *"data loss, not authorship"*, a different
invariant. Here the reasoning is even more concrete — **measured, `docker/seed-example.sql:24`
carries a `DELETE FROM observation_record`** (the demo's idempotent cleanup), so a DELETE verb
would red the committed tree and buy an exemption for a shipped file that has no test. Registered
with the same owner 5.12 gave it: *the story that first needs a data-retention guarantee*.

🔑 **It must REUSE story 5.12's normalisation apparatus, not write a second matcher.** The
authorship gate spent a whole day acquiring `normalise_sql_text` (`:1353`), `is_invisible`
(`:1333`), `statement_before`/`statement_after` (`:1413`/`:1423`), `enclosing_fn` (`:1438`) and
`governing_keyword` (`:1558`), each closing a measured hole: the `--` comment the first draft
missed, the offset map counting characters where the caller indexed bytes, the `\"` inside a Rust
literal that truncated the statement, the zero-width space inside a keyword. **A fresh matcher
re-acquires every one of those holes.** If the shared helpers need a parameter to serve two
tables, that is a refactor with both gates' tests behind it — not a copy.

**And it must have an END-TO-END test from the first commit.** 🔴 Story 5.12's structural finding
was that *the whole body of `gate_declared_authorship` was deletable with the xtask suite green
(56/56)*, because every test attacked the helper and none ran the gate. §7's M2 is that mutation,
and it must red on day one.

**Probes.** The gate gets its own entries under `xtask/probes/`, pinned with **located** verdicts
(file **and line**) on the authorship corpus's precedent — 5.12 learned that *a pinned boolean
proves THAT a gate fires and never WHERE*, and paid for it with a line map that was wrong for
every multibyte file. Minimum: the plain write, the write behind a `--` comment, the write behind
a block comment, a write in a `.sql` file, a write with an invisible character inside the verb,
and a **negative** (a legitimate `UPDATE interface` / `UPDATE identity_link`, which must stay
GREEN — a gate that reds on the engine's own supersede is a gate that will be deleted).

⚠️ 🔴 **But two of those prescribed negatives CANNOT FAIL, and validation caught it.** Measured:
`UPDATE interface SET …` and `UPDATE identity_link SET …` contain **no `observation_record` token
at all**, so a table-anchored matcher never enters its loop body — they are green under *any*
implementation, a deliberately broken one included. **That is a guard placed where the defect
cannot occur, in the story whose AC3 is that very rule.** Keep them if you like, but label them
**vacuity markers**, and make the load-bearing negatives the ones where the table name really
appears in a legitimate non-overwriting context:
`UPDATE identity_link … WHERE observation_id IN (SELECT id FROM observation_record)` (green
because the governing keyword is `select`), a plain `SELECT … FROM observation_record`, and a
**commented-out** write — the shape `docker/seed-example.sql:49` already carries for `DELETE`.

### 3b. 🔴 The gate collides with the `file-size` gate, and it must be decided BEFORE dev

**Measured at validation.** `xtask/src/main.rs` is the largest file in the tree, and the
`file-size` ceiling is **2000 lines of CODE** (everything before the first `#[cfg(test)]`; doc
comments COUNT, only tests are excluded):

| state | code lines | headroom |
|---|---|---|
| committed `72dfe1f` | **1829** | 171 |
| + a **minimal** `gate_observed_immutable` (one-line doc, no probe constants) | **1959** | **41** |
| + the rustdoc §3/AC4 prescribes, modelled on `gate_declared_authorship`'s (**54 lines**, `:1677-1730`) + AC6's module-doc entry (~5) | **≈ 2018** | 🔴 **OVER** |

So the story cannot ship both the gate and the documentation it prescribes for it inside
`main.rs`.

✅ **ARBITRATED (Guy, 2026-08-15): the gate lands in its own module, `xtask/src/observed_immutable.rs`**,
with the shared normalisation helpers raised to `pub(crate)`. ⚠️ **`CLAUDE.md`'s own rule already
prescribed it** — *"A file approaching the ceiling is split into modules or a sub-crate, not
grown"* — so this is applying a standing rule, not inventing an exception. It keeps `main.rs`
under the ceiling, keeps the prescribed rustdoc intact, and touches no matcher logic.
**Refused, and recorded with the reason**: *trimming the rustdoc to fit the 41 remaining lines*,
which would pare back exactly the written promise AC4 exists to pin (the tripwire wording, 5.12's
residual classes, the `DELETE`/`identity_link` exclusions) and would leave the next story 12 lines
of headroom — paying with the documentation to avoid a refactor the rule already required; and
*splitting `main.rs` wholesale, one gate per file*, cleaner long-term but a large refactor inside
a story that ships no feature, and a much wider review surface.
⚠️ This story therefore performs a **bounded refactor of `xtask`**, which the "no PRODUCT
behaviour change" fence permits (xtask is a dependency of nobody and is not shipped) — stated
here rather than discovered mid-dev.

🔑 **And validation established the exact cost of serving a second table**: only **two** helpers
are table-bound — `is_table_reference` (`:1399`) and `statement_after` (`:1423`) both hard-code
`DECLARED_TABLE.len()`. Everything else (`normalise_sql_text`, `statement_before`,
`enclosing_fn`, `governing_keyword`, `is_invisible`, the comment stripper) is table-agnostic and
reusable as-is. Parameterise those two; copy nothing.

**The promise, stated at its width.** This is a **TRIPWIRE against a good-faith change, never a
barrier against a determined one.** Read it as *"a future story will not add such a write by
accident"*, never as *"such a write cannot exist"*. Story 5.12's residual classes are inherited
verbatim and not re-litigated: a table name assembled at runtime (`format!("observation_{}", …)`)
is invisible to any text matcher, and guard neutralisation is closed by a database privilege, not
by a gate. **The `GRANT` is registered as the real closure** — and ⚠️ note that
`deferred-work.md` carries no standalone `GRANT` row today (it is referenced at `:3161` only),
so this story writes one.

---

## 4. 🔴 AC2's letter is not reachable with two co-existing sightings — measured, and the boundary IS the deliverable

**The epic's AC2 reads** (`epics.md:1788-1790`): *"**Given** a declared field and an ingestion
that contradicts it **When** the scan runs **Then** the declared field is unchanged and a
divergence opens."*

**Read `gap::reconcile` (`gap/mod.rs:104-158`) and the second half does not follow.** The
function collects the observed value per field across **all in-perimeter observations**, and
`gap/mod.rs:124-140`:

- two observations disagreeing on a field put it in `conflicting`;
- the field is then **removed** from `observed` and abstains as `ConflictingObservations`;
- the declared field, finding no observed value, abstains again as `NoObservedValue`.

**So: document observation A (hostname `nas`), then ingest B contradicting it (hostname
`intruder`), and with both sightings in the store the page shows TWO ABSTENTIONS AND ZERO GAPS.**
No divergence opens. This is not a defect — it is FR16 working (*never picked, never merged*) —
and story 6.2's code review already registered its neighbour (`deferred-work.md:3414-3428`, a
same-key multi-value observation).

⚠️ **A test written to the AC's letter would therefore either fail, or — far worse — be quietly
weakened until it passed.** That is the trap this section exists to remove before dev.

**The resolution: the assertion has two halves, they are reachable at different widths, and each
is measured where it IS reachable.**

| half | reachable? | how it is measured |
|---|---|---|
| *the declared field is **unchanged*** | ✅ **always** — this is NFR5's actual invariant | snapshot `declared_attribute` (all 7 columns, `updated_at` included) before and after a real ingestion through `poll_ingest_resolve`; assert byte-equality |
| *and **a divergence opens*** | ✅ only when exactly one in-perimeter sighting carries the field | a second test where the contradicting sighting is the only one carrying `hostname` — the shape `build_view_surfaces_a_drift_gap` (`page.rs:553`) already uses, lifted to the store |
| *two sightings disagreeing* | ✅ and it yields **abstentions, not a gap** | asserted explicitly, with `gaps.is_empty()` **and** `abstention_count() == 2`, so the boundary is pinned rather than discovered later |

🔑 **The third row is not a consolation prize — it is the more valuable measurement**, because it
is the case a real network produces (a host re-scanned under a new hostname, its old sighting
still in the store) and because nothing pins it today. **Register the divergence from the epic's
letter; do not edit `epics.md`.**

### 4b. ✅ VALIDATED — and the boundary is HARDER than this section first said

Both layers ran `reconcile` for real, pure and store-backed. **§4's `(0, 2)` prediction is
CONFIRMED exactly** — `{ConflictingObservations: 1, NoObservedValue: 1}`, pure and through
`page::reconcile_view` after a real `document_all` + a real `poll_ingest_resolve`. Three further
measurements the section did NOT anticipate, each of which changes what the dev must write:

- 🔴 **The single-sighting shape is NOT REACHABLE through the documenting gesture at all.** The
  declared `hostname` can only come from observation A, so A is in the store by construction, so
  every contradicting ingestion conflicts. Producing a gap required **deleting A**. So
  `epics.md:1790`'s *"a divergence opens"* is not merely *"reachable at a different width"* — it
  is **unreachable end-to-end through the new write path**. The only three options are: delete
  the documented sighting (explicit, artificial, and honest); seed the declared row through the
  manual path (which loses *"through the new write path"*, the AC's own phrase); or restrict AC2
  to the halves that ARE reachable and register the rest. ⚠️ **This is an arbitration, not a
  detail — see AC2.**
- 🔴 **The store-backed single-sighting shape measures `(1, 1)`, not `(1, 0)`** — the extra
  abstention is `mac`, a declared field with no observed value once B carries no MAC. **A dev
  mirroring story 6.2's J3 oracle (`gaps.is_empty() && abstention_count == 0`) would red for the
  wrong reason.**
  🔴 ⚠️ **CORRECTED AT DEV, and the correction is the more useful fact: `(1, 1)` is NOT a property
  of the shape — it is a property of the FIXTURE.** Implementing it produced `(1, 0)` and reddened
  the prescribed assertion, because the contradicting sighting written here carries **the same
  MAC** (a re-scan of one NIC under a new hostname — the realistic case), so `mac` is still
  observed and nothing abstains. The validation figure had been taken with a **MAC-less** B.
  **Both are now pinned in one test**, same-MAC → `(1, 0)` and MAC-less → `(1, 1)` with its cause,
  so the dependency is measured instead of asserted. *A figure quoted without the fixture that
  produced it is not a measurement* — and story 6.2's oracle turns out to be right for the first
  case and wrong for the second, which is exactly the distinction the original warning blurred.
- 🔴 **On the SHIPPED connector, AC2's *"a divergence opens"* half can NEVER fire.** `arp_ping`
  emits `ipv4` + `rtt`; `rtt` is not a declared field, so `ipv4` is the only declarable one —
  **and `ipv4` is also the perimeter key**, so an in-perimeter observation agrees on it by
  definition. Swept all four shapes: `gaps = 0` in every one. *The divergence half of NFR5's
  first assertion is a property of fixtures today, not of the product on a real network* — which
  is a true and useful sentence to have measured, and belongs in the register.

Also measured, and worth pinning because it is counter-intuitive: **the conflict abstention is
counted once per FIELD, not per sighting** (three disagreeing sightings still give `(0, 2)`), and
**two sightings that AGREE with each other but differ from the declared value DO open a gap**
`(1, 0)` — conflict requires disagreement *between sightings*, never with the declared side.

⚠️ **And the oracle must pin BOTH numbers.** Story 6.2's M11 measured that a wrong attribute key
yields an ABSTENTION rather than a gap, so `gaps.len() == 1` alone is satisfiable by an accident
and `!gaps.is_empty()` is satisfiable by almost anything. Every reconcile oracle in this story
asserts the gap's **field, declared and observed values** AND `abstention_count()`.

---

## 5. The ingestion must go through the real pass, and the connector cannot supply the contradiction

**Measured:** `arp_ping::emitted_facts` (`arp_ping.rs:216`) emits `IpV4` + `Rtt` and
`declared_kinds` (`arp_ping.rs:207`) declares those two only — **no `Hostname`, no `Mac`, ever**
(story 5.14 pinned this as a structural zero). A *contradicting* ingestion needs a `Hostname` or
a `Mac`, so the shipped connector cannot produce one.

**Prescribed:** drive `scan_pass::poll_ingest_resolve` — which is generic over `Connector`
(`scan_pass.rs:75`) — with `FixtureConnector::from_observations`. `page.rs:1083-1120` is the
worked example to copy.

🔴 **This is what makes the test a re-assertion "through the new write path" rather than a repeat
of an existing one.** The nearest existing carrier, `index_renders_the_real_gap`
(`main.rs:1173`), inserts its observation with `repo::insert_observation` **directly**, and
asserts on **rendered HTML**. ⚠️ **This story's first draft justified the new carrier by claiming
that test "would stay green if the declared value were rewritten to match the observation" —
REFUTED at validation by measurement**: rewriting the seeded declared hostname reds it
(`panicked at main.rs:1271: renders the declared hostname`), because `"nas"` appears nowhere else
in the rendered output. *The story about checking claims had shipped a claim with no check; it is
recorded rather than quietly deleted.* **The true justification is narrower and sufficient**: that
test asserts on rendered VALUES, so it cannot see `facts`, `raw`, `observed_at` or the row's
bytes; it never crosses the document route; and it inserts directly rather than through the pass.
*An assertion on what a page shows is not an assertion on what a row holds.*

⚠️ **Cleanup order is children-before-parents** — `identity_link.observation_id` gained a foreign
key in `0003_resolver_guards.sql:43-45`, so it is `link_candidate` → `identity_link` →
`interface` → `observation_record` (± `declared_attribute`). Canonical arrays at
`main.rs:1297-1305` and `page.rs:1069-1076`; the warning is spelled out at `main.rs:1193-1197`.

---

## 6. What must be pinned

- the observation row, all seven columns including a **non-NULL `raw`**, byte-identical across a
  successful `POST /document-all` (AC1);
- the observation's `identity_link` rows, **`id` included**, unchanged across the same gesture
  (AC1) — the link is intact, not merely present;
- ⚠️ the same two comparisons across a **REFUSED** document (409 and 422-domain), because a
  rolled-back write and an absent write are indistinguishable to a naive after-only check and the
  refusal paths are where a future "mark it attempted" write would land;
- the declared rows, all seven columns including `updated_at`, byte-identical across an ingestion
  that contradicts them (AC2);
- a divergence opening on the single-sighting shape, with field/declared/observed AND
  `abstention_count()` (AC2);
- the two-sighting shape yielding **abstentions and no gap**, both numbers (AC2, §4);
- the gate: green on the committed tree with an **empty** allowlist; red on a planted `UPDATE
  observation_record` in `.rs` **and** in `.sql`; green on `UPDATE interface` and `UPDATE
  identity_link` (AC3);
- the gate's END-TO-END path, so its body is not deletable with the xtask suite green (AC3);
- `cargo xtask ci` now enumerating **eight** gates — 🔑 including in `xtask/src/main.rs`'s own
  **module doc**, which 5.12's review caught enumerating six while the file implemented seven.

⚠️ **Anything red beyond these is a FINDING**: this story changes no product behaviour, so a
pre-existing test that goes red is either a real regression or a test that was passing for the
wrong reason. Do not "fix" it without recording which.

---

## Acceptance Criteria

**AC1 — documenting leaves the observed side bit-for-bit unchanged, and the link intact.**
**Given** a seeded observation carrying `ipv4` + `hostname` + `mac` **and a non-NULL `raw`**, a
resolved identity link over it, and a same-origin authenticated `POST /document-all` naming it
**When** the gesture completes (201)
**Then** the `observation_record` row is byte-identical across the gesture on **all seven**
columns, and `load_current_links_for_observation` returns the same rows **including their `id`s**
— and the same two comparisons hold across a **refused** gesture (409 and 422-domain).
_Reddened by: M1 (planted write in the documenting transaction), M3 (planted link close), M4
(refusal-path write); M6 is its `raw`-column CONTROL._

**AC2 — an ingestion that contradicts a declared field changes nothing on the declared side, and
the divergence behaviour is pinned at its real boundary.**
**Given** an observation documented through story 6.2's route, and a contradicting sighting
ingested through `scan_pass::poll_ingest_resolve` driven by `FixtureConnector`
**When** the pass runs
**Then** every `declared_attribute` row is byte-identical on all seven columns, `updated_at`
included — **this half is unconditional, it is NFR5's actual invariant, and it is what AC2 is
really for**; and the divergence behaviour is pinned in **three** named shapes, each with BOTH
numbers: the two-sighting shape yielding `(gaps, abstentions) = (0, 2)` with
`{ConflictingObservations: 1, NoObservedValue: 1}`; the three-sighting shape also `(0, 2)`
(the conflict counts once per FIELD); and the single-in-perimeter-sighting shape yielding
**`(1, 1)` — not `(1, 0)`** (the spare abstention is `mac`, a declared field with no observed
value), with the gap asserted on field, declared value and observed value.
🔴 ✅ **ARBITRATED (Guy, 2026-08-15) — option (a): the test DELETES the documented sighting, and
the limit is REGISTERED.** The single-sighting shape is produced by removing observation A before
ingesting the contradicting one, which models *the old sighting aged out*; both halves of NFR5's
first assertion are then measured through the real reconcile.
⚠️ **The artificiality is declared, not hidden**: no production code path purges an observation
today, so the DELETE is the test's own gesture and the test must say so in a comment. FK order
applies (`link_candidate` → `identity_link` → `interface` → `observation_record`).
**Refused, and recorded with the reason**: *(b)* seeding the declared row through the manual
path, which would lose *"through the new write path"* — the AC's own phrase and the whole point
of re-asserting here rather than trusting 5.12; *(c)* restricting AC2 to the reachable halves,
which is more honest about today but stops measuring drift detection at all, and D22 makes
drift-can-reopen the property that keeps NFR5 alive.
**And the registered limit stands beside it** (AC7 row (12)): the divergence half is unreachable
end-to-end through the gesture, and on the shipped connector it can never fire at all. `epics.md`
is NOT edited.
_Reddened by: M5 (planted declared write in the ingest path — two carriers, the test and the
`declared-authorship` gate), M7 (the reconcile oracle)._

**AC3 — each guard sits on the path where the violation would be WRITTEN, and the source guard
is the eighth gate.**
**Given** `epics.md:1792` and retro action item 2
**When** `cargo xtask ci` runs
**Then** an eighth gate `observed-immutable` reds on **every overwriting form** of a write to
`observation_record` — `UPDATE`, `REPLACE INTO`, and `INSERT … ON DUPLICATE KEY UPDATE` (§3's
measured widening; an `UPDATE`-only verb list was MEASURED to let the last two through) — in
`.rs` and in `.sql` under `crates/` and `docker/`, with an **EMPTY** allowlist (measured green
across 36 files), stays green on `UPDATE interface` / `UPDATE identity_link`, reuses story
5.12's normalisation helpers rather than a second matcher, carries located probe verdicts (file
**and** line), and has an **end-to-end** test so its body is not deletable with the xtask suite
green.
_Reddened by: M1b (🔑 the headline — a planted write in an UNCALLED fn is GREEN on AC1's
behavioural test and RED here, which is what proves the two carriers are not redundant), M2 (the
gate body), M8 (the located probes)._

**AC4 — the promise is stated at its width, and no wider.** The gate's rustdoc and the register
name it a **TRIPWIRE**, inherit 5.12's residual classes verbatim (runtime-assembled table names;
guard neutralisation, closed only by a database privilege), state why `DELETE` is out with its
measurement (`docker/seed-example.sql:24`), and state why `identity_link` gets no gate
(`close_identity_link` legitimately updates it). **NFR5 is now covered on all three assertions —
and the register says at what width**, so *"NFR5 is covered by anti-regression tests"* is never
read as more than the tests measure.

**AC5 — no PRODUCT behaviour change.** `crates/opencmdb-bin/templates/` at **zero diff**, and
**`page.rs`'s non-test code at zero diff** — ⚠️ **narrowed at validation, which measured the
collision**: `build_view`, `build_identity_view` and `reconcile_view` are all **private** to
`page.rs`, and every existing store-backed reconcile test lives in that file's trailing test
module (D56b). A blanket *"`page.rs` at zero diff"* would force AC2's oracle either through
rendered HTML — which §5 condemns — or into a re-derivation of `build_view`'s perimeter
selection in `main.rs`, a second oracle free to drift. **A test added to `page.rs`'s trailing
module is therefore EXPECTED, not a fence breach.** ⚠️ `xtask` is refactored per §3b and is not
the product;
no migration; no route, status, refusal or column added or altered; `opencmdb-core` unchanged in
behaviour (a doc correction is permitted and must be stated at that width — 5.13b's precedent,
where a promise of non-modification sheltered a false sentence).

**AC6 — gates and tree.** `cargo xtask ci` **eight** gates green (the module doc enumerating
eight — 5.12's caught defect), plus `views-hash ℹ STALE exit 0`; 28 fixtures; trap gate still RED
at 26/15/11 by design; `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D
warnings` clean; no new crate; `Cargo.lock` untouched.
🔴 **`file-size` is the gate at risk and it is named here for that reason**: `xtask/src/main.rs`
sits at **1829** code lines against a **2000** ceiling, and a minimal eighth gate takes it to
**1959** — 41 lines of headroom against a prescribed rustdoc of ~59 (§3b, measured). **AC6 is not
met by "eight gates green" alone; it is met when `file-size` is green WITH the prescribed
documentation present**, which is what forces §3b's module split.
⚠️ Also measured: `report()` pads its label at `{:<14}` and `observed-immutable` is 18 characters,
which misaligns every line of the gate summary — a one-line fix, and a visible one.

**AC7 — the register, each row WITH ITS OWNER, re-read against THIS list.** ⚠️ 🔑 *A re-read that
reads only what you wrote cannot find what you did not write* (Epic 5's retro, takeaway 2): count
the rows written against **this** enumeration, not against themselves.
**(1)** 🔴 **`epics.md:1724` says NFR5 *"(assertions 2 and 3)"* where the register
(`deferred-work.md:2896-2899`) and this story's own ACs say **(1) and (2)*** — assertion 3 is what
5.12 covered. `sprint-status.yaml`'s comment repeated the same wrong numbering and was
**corrected at contexting, with the correction recorded in place**; `epics.md` is NOT edited —
owner: **Epic 6's retrospective**.
**(2)** 🔴 **§4's boundary**: an ingestion contradicting a declared field yields ABSTENTIONS, not
a divergence, while the older sighting co-exists — a divergence from `epics.md:1790`'s letter,
measured — owner: **Epic 6's retrospective**.
**(3)** 🔴 **A story-6.2 review patch marked `[x]` applied is NOT in the shipped tree** (measured
at contexting): `same_origin` guards `Origin` multiplicity with `get_all` (`document.rs:253`) but
reads `Host` with `.get()` (`document.rs:271`), the asymmetry the patch existed to remove. Either
the patch was lost to the `git checkout -- <file>` class 6.2's own Dev Notes warn about, or the
checkbox is wrong. **Not fixed here** (out of subject) — owner: **Epic 19**, and the *lost-patch*
question owner: **Epic 6's retrospective**.
**(4)** ⚠️ **`6-1-write-route-writes-nothing.md:3` still reads `Status: review`** while
`sprint-status.yaml` and PR #89 say `done` — a doc-twin drift of exactly the class these stories
keep catching. **Corrected by this story** (a one-word artifact fix), recorded rather than done
silently.
**(5)** ⚠️ **`DELETE` stays outside the new gate's verb list**, with its measurement
(`docker/seed-example.sql:24`) — owner: **the story that first needs a data-retention guarantee**
(5.12's own owner for the same call).
**(6)** ⚠️ **D15's sibling rule is NOT held by this gate**: *"`UPDATE declared_attribute SET
entity_id = ?` … the most dangerous line of SQL in this project — and it looks like a routine
refactor"* (`architecture.md:1064-1069`) — owner: **story 6.5** (the entity/device schema story).
**(7)** 🔴 **The `GRANT` (5.12's *voie B*) has no standalone register row** — it is referenced
only at `deferred-work.md:3161`. This story WRITES one, as the real closure of the
guard-neutralisation class for both gates — owner: **unassigned** (a deployment/privilege
decision).
**(8)** ⚠️ **Retro action item 4 — *fix the mutation driver once, in `xtask`* — is still
UNASSIGNED and NOT DONE** (`epic-5-retro-2026-08-12.md:208-211`), so this story's prove-to-red
runs on the driver that produced four recurrences in one epic. Re-registered — owner: **needs
Guy's go-ahead**.
**(9)** ⚠️ **6.1's `lazy_pool()` row** (`deferred-work.md:3347`, owner *"the next story that
touches the test helpers"*): this story adds test helpers, so the row comes due. Either close it
or re-register it **with the reason** — owner: **this story**, and if it is re-registered the new
owner must be named rather than left as *"the next story"* a third time. ⚠️ This row had **no
owner** in the first draft, in the AC whose own header demands one — caught at validation, and it
is exactly the *"a re-read that reads only what you wrote"* failure this AC opens with.
**(10)** ⚠️ **NFR5's residual width after this story** — the two gates are tripwires; 5.12's
stated classes still stand (`'engine'` passes the DDL CHECK; a runtime-assembled name is
invisible; `docker/seed-example.sql` is a whole-file site with no test) — owner: **Epic 19** for
the privilege half, **the seed-file story** for the last.
**(11)** 🔴 **A stale `SANCTIONED_READS` entry is caught by NOTHING** — measured at validation by
planting a non-existent path/fn: 62/62 xtask tests green, gate green.
`the_allowlist_sanctions_a_place_and_not_a_name` walks `SANCTIONED_SITES` only. Owner: **this
story if it takes resolution (2)**, otherwise **Epic 6's retrospective** (it is a hole in 5.12's
apparatus that 6.2 widened without extending its guard).
**(12)** 🔴 **NFR5's first assertion has a divergence half that the SHIPPED product cannot
exhibit** — `arp_ping` emits only `ipv4`+`rtt`, and `ipv4` is the perimeter key, so no in-perimeter
observation can disagree with a declared `ipv4`. *The divergence half is a property of fixtures
today, not of the product on a real network.* Owner: **the connector story that emits a MAC or a
hostname** — the same story that already inherits story 5.14's two shielded races.
**(13)** ⚠️ **`deferred-work.md:2891` still instructs the implementer to add the writer to
`SANCTIONED_FNS`**, a name retired at 5.12's repair in favour of `SANCTIONED_SITES` — a stale
instruction in the very row story 6.2 discharged. Owner: **Epic 6's retrospective**.
**(14)** ⚠️ **`CHAR(36)` columns strip trailing spaces on retrieval**, so a padding-only
difference is invisible to §2a's snapshot. Irrelevant to the comparison this story makes, but it
bounds the phrase *"byte-identical"* and is stated rather than left implied. No owner — a limit.

**AC8 — documents in the same commit, ONE live count in ONE place.** This story file carries the
final test count; `CLAUDE.md`, `docs/project-context.md` and `sprint-status.yaml`'s comments cite
it **by reference and carry no number** (6.1's AC8 / 6.2's AC9, including F2 — whose violation
was a 🔴 review patch on 6.2).

---

## Tasks / Subtasks

- [x] **T1 — the observation snapshot helper** (AC1): `#[cfg(test)]` in `repo.rs`, all seven
      columns, `CAST(observed_at AS CHAR)`, `Option<String>` for `raw`, `ORDER BY id`
- [x] **T2 — AC1's behavioural guards** (AC1): document-success and both refusal paths; the
      observation compare and the `load_current_links_for_observation` compare (ids included);
      a **non-NULL `raw`** in the fixture
- [x] **T3 — AC2's ingest guards** (AC2): `FixtureConnector` → `poll_ingest_resolve`; the
      declared seven-column compare (through the **widened** sanctioned reader, resolution (3));
      the two-sighting boundary `(0, 2)` with its cause breakdown; the three-sighting `(0, 2)`;
      and the single-sighting divergence — **which deletes the documented observation first**
      (Guy's arbitration §4b, FK order, with the comment saying why) — pinned at `(1, 1)`, the
      spare abstention being `mac`. ⚠️ Do NOT copy 6.2's J3 oracle (`abstention_count == 0`):
      it reds here for the wrong reason. The store-backed reconcile assertions live in
      `page.rs`'s trailing test module (AC5)
- [x] **T4 — the eighth gate** (AC3, AC4): `gate_observed_immutable` in `xtask`, reusing
      `AUTHORSHIP_ROOTS` and 5.12's normalisation helpers; **empty allowlist**; wired into
      `cargo xtask ci`; the module doc enumerating **eight**; the rustdoc stating the tripwire
      promise and the DELETE/`identity_link` exclusions with their measurements
- [x] **T4b — the gate's end-to-end test and its located probes** (AC3): the end-to-end path
      from day one (5.12's deletable-body finding), probes with `(file, Option<line>)` verdicts,
      including the **load-bearing** GREEN negatives of §3 (a `SELECT … FROM observation_record`,
      an `UPDATE identity_link … WHERE observation_id IN (SELECT …)`, a commented-out write) —
      not only the two vacuity markers validation measured unfailable
- [x] **T4c — the `xtask` module split** (AC6, §3b): move the new gate to
      `xtask/src/observed_immutable.rs`, raise the shared normalisation helpers to `pub(crate)`,
      parameterise the **two** table-bound ones (`is_table_reference`, `statement_after`), and
      re-measure `file-size` with the prescribed rustdoc present (Guy's arbitration, §3b)
- [x] **T5 — prove-to-red** (AC1–AC4): **nine ids** — M1, M1b, M2, M3, M4′, M5, M6, M7, M8 —
      predictions FIRST, each carrier read from its own panic or gate message; M1/M5 dual-carrier,
      M6 a green control, M1b green on one carrier and red on the other; ⚠️ **commit the green
      state BEFORE the pass** (Dev Notes)
- [x] **T6 — the register and the documents** (AC7, AC8), including row (4)'s artifact
      correction (`6-1-…md:3`'s stale `Status: review`); row (1)'s `sprint-status.yaml` comment
      was already corrected at contexting

---

## 7. Prove-to-red

| id | mutation | predicted |
|---|---|---|
| **M1** | add `UPDATE observation_record SET raw = 'documented' WHERE id = ?` inside `StoreDocument::document_all` before `tx.commit()` | RED — AC1's observation compare (`raw` differs). ⚠️ **Also reds the new gate**; read BOTH messages and record both carriers, do not attribute one red to the other |
| **M1b** | 🔑 **the headline**: put the same `UPDATE observation_record` in a **new, UNCALLED** `repo.rs` fn | ✅ **MEASURED AT VALIDATION, both layers: GREEN on every behavioural test (361/161/62, live DB, 0 failed), RED on the gate alone** (`repo.rs:1182`, exact located verdict). This is what proves §1's two carriers are not redundant. ⚠️ If it reds a behavioural test, something calls the fn — a FINDING about the mutation, not a bonus. ⚠️ An uncalled `fn` triggers `dead_code`, which **fails `cargo clippy -D warnings`** (AC6): add `#[allow(dead_code)]` to the mutation, or do not run clippy on the mutated tree — otherwise the red is compiler-carried and misattributed |
| **M2** | replace `gate_observed_immutable`'s body with `Ok((true, String::new()))` | RED — the gate's END-TO-END test. 🔴 Story 5.12 measured this exact mutation GREEN (56/56) because every test attacked the helper; if it is green here, the end-to-end test is missing, not passing |
| **M3** | call `repo::close_identity_link` on the subject's link inside the documenting transaction | RED — AC1's link compare, **on the row COUNT, never on a field**: `load_current_links_for_observation` filters `WHERE observation_id = ? AND current_subject IS NOT NULL` (`repo.rs:795`), and a closed link sets `current_subject = NULL`, so the row leaves the result set entirely. Determined by reading at validation rather than left as *"predict which half"* — but still read the panic message, because a vector-length red and a field red look nothing alike |
| **M4′** | write the observation `UPDATE` on a REFUSAL path, **on its OWN connection** (`.execute(&self.pool)`) | RED — AC1's refused-gesture compare (measured: `raw_before="original raw"` → `raw_after="attempted"`). 🔴 **The naive form of this mutation was REFUTED at validation and is kept here as the lesson**: written on `&mut *tx`, `document_all` returns without `tx.commit()`, the transaction ROLLS BACK, and the observation is byte-identical mutation or no mutation — the behavioural guard **cannot** see it (only the gate reds, at `document.rs:122`). AC1's own bullet said *"a rolled-back write and an absent write are indistinguishable"* two pages earlier, and the first draft then prescribed a guard that could not see one. **This is Epic 5's dominant class inside the story about Epic 5's dominant class, and 5.11b's finding verbatim.** An own-connection write is also the shape a real "mark it attempted" feature would take |
| **M5** | add `UPDATE declared_attribute SET attr_value = ?` inside `resolver::resolve_within` | RED — AC2's declared compare **and** the `declared-authorship` gate (an unsanctioned write site). Two carriers, both to be named |
| **M6** | **CONTROL** — keep M1, but narrow the snapshot to exclude `raw` | **GREEN by design**, and that is the point: it measures that `raw` is load-bearing in the comparison. ⚠️ A snapshot whose columns are never shown load-bearing is a snapshot that compares nothing (story 5.10's M6 idiom) |
| **M7** | make `gap::project` emit `"ip"` for `"ipv4"` | RED **on both halves** — measured at validation: `gaps=0, abst=3 {OutOfPerimeter: 1, NoObservedValue: 2}`, so the gap VANISHES and `gaps.len() == 1` reds on its own. ⚠️ **The first draft predicted the opposite** (*"a wrong key yields an abstention, so only `abstention_count()` catches it"*) by importing 6.2's M11 with its polarity reversed: 6.2's oracle asserted a gap was CLOSED, where an abstention is invisible; **this story's oracle asserts a gap OPENS, where it is fatal**. The mechanism is `OutOfPerimeter` — the identity key itself stops matching — not a plain key mismatch. Keep the `abstention_count()` pin: it is justified by §4's boundary, not by this row |
| **M8** | shift one probe's expected line by one | RED — the located probe table. 🔴 5.12 shipped a broken byte/char line map that **no boolean probe could see**; a pinned boolean proves THAT a gate fires and never WHERE |

⚠️ **Predict first, then measure; a divergence is a FINDING.** Read each red's message one by one
— *a mutation named for one thing and applied to another measures the other thing* (four
recurrences across 5.13/5.13b/5.14/6.1). **NINE ids** — M1, M1b, M2, M3, M4′, M5, M6, M7, M8;
a `b` suffix counts as its own id (6.1's *"fifteen runtime mutations … sixteen mutation ids"*).
⚠️ **The first draft said "eight" while its own table carried nine — the headline-that-does-not-
describe-its-own-table defect, caught at validation, in the story whose method section forbids
it.**

**Carriers, per row rather than per bin** — the summary must not flatten what the rows say:
**M1** and **M5** are **DUAL-carrier** (each reds a behavioural assertion *and* a gate, and both
messages must be read and recorded); **M2** and **M8** are gate-message-carried; **M3**, **M4′**
and **M7** are assertion-carried; **M6** is a GREEN control by design; **M1b** is GREEN on the
behavioural carrier and RED on the gate, which is its whole point. **So the headline is NOT
"nine reds"**, and *"every red assertion-carried"* must not be written — that sentence has been
refuted by review in four consecutive stories, and this table contains three gate reds by
construction.

---

## Dev Notes

### Traps, each measured on this project

- 🔴 **Commit the green state BEFORE the mutation pass, and revert the MUTATION, never the FILE
  — a file-level `git checkout -- <file>` equals a mutation revert ONLY on a COMMITTED
  baseline.** 6.1's review lost every uncommitted review patch in two files exactly that way,
  and ⚠️ **register row (3) suggests it may have happened a second time on 6.2** — a patch marked
  applied is absent from the tree;
- ⚠️ **`cargo test --workspace A B` passes TWO filters and silently runs nothing**;
- ⚠️ **Never read a measurement through a truncation** — `head -8` once turned an 18-red
  measurement into a false "unreachable" claim;
- ⚠️ **`DATABASE_URL` is unset locally** — every DB test passes by `return`ing, so **a green
  local suite says NOTHING about this story**, whose entire deliverable is DB-backed. 🔴 **The
  6.1/6.2 container on port 13316 is GONE** (measured at contexting: no opencmdb container
  existed; port 3306 is held by an unrelated `kesh-mariadb-dev` — do not assume an inherited
  container, story 6.2's own Dev Notes said *"the 6.1 container is still up"* and it is not).
  ✅ **A fresh one was started at contexting and the baseline verified against it**:
  `opencmdb-story63`, `mariadb:10.11.11`, `DATABASE_URL='mysql://root:story63@127.0.0.1:13318/opencmdb_test'`.
  ⚠️ Check it is still running before trusting a green run — and check the ELAPSED TIME, since a
  skipped DB suite and a passing one report the identical count;
- ⚠️ **`DB_TEST_LOCK`** (`main.rs:43`) must be taken before connecting in every DB test —
  concurrent `migrate!` duplicates `_sqlx_migrations` version 1;
- ⚠️ **Cleanup order is children-before-parents** (`0003_resolver_guards.sql:43-45`'s FK):
  `link_candidate` → `identity_link` → `interface` → `observation_record` (± `declared_attribute`);
- ⚠️ **`cargo fmt --all --check` runs in CI before the tests** (`ci.yml:56`); rustfmt has
  invalidated a mutation driver's anchor mid-story once — make every driver print `MUTATED` and
  assert its anchor;
- ✅ **SETTLED AT VALIDATION — AC2's seven-column declared snapshot DOES red the
  `declared-authorship` read half**, measured by both layers: `🔴 authorship 1 unsanctioned
  access(es): repo.rs: a read of declared_attribute names 'origin_obs_id' — FR13` (one finding,
  naming only the first provenance column found). The same snapshot minus the three provenance
  columns is green. **Three resolutions were measured; take the third**: (1) *reuse
  `read_declared_provenance_for_test` as-is* — ❌ **insufficient**, it returns only
  `(attr_key, origin, origin_obs_id, actor_id)` and is keyed on one `entity_id`; (2) a second
  `SANCTIONED_READS` entry — ✅ works; (3) ✅ **WIDEN `read_declared_provenance_for_test` to the
  seven columns, same name, same file** — the gate keys on `(path, fn)`, so the projection is
  invisible to it, and this is **story 6.2's own precedent** (`raw_declared_write_for_ddl_test`
  was widened exactly this way, `repo.rs:1345-1348`). It adds no allowlist entry;
- 🔴 ⚠️ **A sentence in this story's first draft was FALSE and both layers refuted it by
  planting a fake entry**: `the_allowlist_sanctions_a_place_and_not_a_name`
  (`xtask/src/main.rs:2645`) walks **`SANCTIONED_SITES` ONLY**. A bogus
  `("crates/does/not/exist.rs", Some("no_such_function"))` in `SANCTIONED_READS` leaves 62/62
  xtask tests green and the gate green. **A stale `SANCTIONED_READS` entry is caught by
  nothing** — which is precisely why resolution (3) above is preferred over (2), and if (2) is
  ever taken, the existence test must be extended to the read half in the same commit;
- ⚠️ **A raw declared INSERT in a test has exactly one legal home**:
  `raw_declared_write_for_ddl_test` (`repo.rs:1351`), already carrying `origin` /
  `origin_obs_id` / `actor_id` as parameters since 6.2.

### The tree this story extends (measured 2026-08-15, master `72dfe1f`)

Seven gates green + `views-hash ℹ STALE exit 0`; 28 fixtures; trap gate RED at 26 discovered /
15 scored / 11 unanswerable, by design until Epic 6's L2 stories (6.15 last). Story 6.2's
surface: `POST /document-all` writing adopted rows through `repo::adopt_declared_attribute`
(`SANCTIONED_SITES`' fourth entry), `DocumentPort`/`StoreDocument` with the pool inside the impl,
`gap::project` `pub`, `SANCTIONED_READS` (one entry), `0005_document_guards.sql`'s unique index.
**Baseline test count: 580 — 357 bin + 161 core + 62 xtask, RE-MEASURED on this tree at
contexting (2026-08-15), not inherited from 6.2's file.** Measured **twice**: with `DATABASE_URL`
unset, and against a live `mariadb:10.11.11` — **all green both ways, 0 failed**.

🔑 ⚠️ **And the pair is itself the measurement worth keeping: the two runs report the SAME
COUNT.** With `DATABASE_URL` unset every DB-backed test passes by `return`ing, and nothing in the
output says so — the only tell is the clock, `0.06 s` against `5.79 s` for the `opencmdb` binary's
357. *A count is not a verification, and this suite cannot tell you which one you just got.*
Since this story's entire deliverable is DB-backed, **run it against the container and check the
elapsed time, not just the tally.** The container used at contexting:
`DATABASE_URL='mysql://root:story63@127.0.0.1:13318/opencmdb_test'` (`opencmdb-story63`).

This story's own count goes in Completion Notes and nowhere else (AC8).

### Stack and dependencies — stated, not researched, and here is why

**No new dependency, no version decision, no external API.** The story adds test helpers, an
`xtask` gate and probe files; it touches no `Cargo.toml` and `Cargo.lock` stays byte-identical
(AC6). The versions it works against are the committed ones — Rust 1.96+ / edition 2024, `sqlx`
`=0.9.0` (verified in `crates/opencmdb-bin/Cargo.toml:51`: `default-features = false`, features
`runtime-tokio`/`tls-rustls-ring`/`mysql`/`migrate`/`macros` — **no `chrono`**, which is why §2a
casts instants to `CHAR`), `axum` 0.8.9, MariaDB **10.11.11** (the exact DSM 7 package, so
dev = CI = prod).
⚠️ **Never invent a version — pin from the real `Cargo.lock`.** The BMad checklist asks for
latest-library research at this step; it is **N/A here by scope**, recorded rather than skipped
in silence.

### Validation obligations — ✅ ALL DISCHARGED 2026-08-15 (results folded into the sections above)

Two fresh-context layers ran: a fact-check layer and a gap-hunt layer, the latter in an isolated
git worktree against its own `mariadb:10.11.11` database. 🔑 **Both BUILT a prototype of the
eighth gate rather than reasoning about one**, and they agree on every central measurement.

1. **§4's boundary, re-derived independently** — ✅ `(0, 2)` **CONFIRMED**, pure and store-backed
   (through a real `document_all` then a real `poll_ingest_resolve`), with the cause breakdown.
   ⚠️ And it went further than the obligation asked: see **§4b** for the three findings that
   change AC2 — the single-sighting shape is unreachable through the gesture, it measures
   `(1, 1)` not `(1, 0)`, and the shipped connector can never produce a gap at all.
2. **§3's gate, built rather than argued** — ✅ **BUILT TWICE, INDEPENDENTLY.** The **EMPTY
   allowlist holds**: `✅ observed-immutable … across 36 file(s)`, `docker/` included. Every
   evasion shape 5.12 had to close reds with an **exact located line** (`--` comment, block
   comment, zero-width space inside the verb, `UpDaTe` split across a newline, backtick-quoted
   and schema-qualified names, a MariaDB executable comment `/*!50000 … */`, a trigger body, and
   a `JOIN` that updates the observed row). 🔴 **And it found two holes the story had not seen**
   (§3's overwriting verbs) **and one collision it had not foreseen** (§3b, `file-size`).
3. **M1b** — ✅ **MEASURED, no longer a prediction**: suite GREEN (361/161/62, `DATABASE_URL`
   set), gate RED at `repo.rs:1182`. Both layers, independently.
4. **The authorship-gate collision** — ✅ **SETTLED**: it reds; all three resolutions were run;
   resolution (3) (widen the sanctioned reader) is prescribed. See the Dev Notes bullet — and
   the FALSE sentence the first draft carried about the allowlist-existence test, refuted by
   planting a fake entry.
5. **Register rows (3) and (4)** — ✅ both **CONFIRMED** verbatim by both layers.

⚠️ **What validation cost the story, stated plainly, because it is the argument for doing it:**
**two claims refuted by measurement** (§5's justification and M7's polarity), **two headline
numbers that contradicted their own table** (nine ids, not eight; the carrier bins), **one
sentence asserting as fact what the same file called a prediction**, **one FALSE claim about an
existing test**, **two prescribed guards that could not fail**, **one mutation (M4) that the
behavioural guard structurally could not see**, and **two design collisions** (`page.rs`'s
private seam, the `file-size` ceiling). None of these were findable by reading the story.

### References

- [Source: `epics.md:1776-1792` — story 6.3] — ⚠️ divergences registered §1 (numbering) and §4 (AC2's letter)
- [Source: `prd.md:1208-1223` — NFR5's three assertions and FR13's corollary, verbatim]
- [Source: `deferred-work.md:2894-2900`] — the row this story discharges; `:2872-2877` the DELETE reasoning; `:3161` the `GRANT` reference; `:3414-3428` the neighbouring multi-value finding
- [Source: `epic-5-retro-2026-08-12.md:86-110`] — the defect class; `:200-203` action item 2, this story's AC3; `:112-117` *the mutation driver lies*; `:208-211` action item 4, still unassigned
- [Source: `architecture.md:524-525` (D10, *NFR5 extended*), `:557-559` (D10, the compiler as carrier), `:296-297` (D2's six-condition test), `:1064-1069` (D15), `:1491-1502` (D22)]
- [Source: `crates/opencmdb-bin/src/document.rs:97-146`] — the documenting transaction, statement by statement
- [Source: `crates/opencmdb-bin/src/repo.rs:263` `insert_observation`, `:1242` `snapshot_links`, `:785` `load_current_links_for_observation`, `:684` `close_identity_link`, `:1351` the sanctioned raw writer]
- [Source: `crates/opencmdb-core/src/gap/mod.rs:104-158`] — `reconcile`, and §4's conflict branch at `:124-140`
- [Source: `xtask/src/main.rs:1124` `AUTHORSHIP_ROOTS`, `:1151` `SANCTIONED_SITES`, `:1181` `SANCTIONED_READS`, `:1333-1558` the normalisation helpers, `:1731` the authorship gate, `:3154` the located probe table]
- [Source: `_bmad-output/implementation-artifacts/6-2-route-writes-a-declared-value.md:41-45`] — the sentences that hand this story its deliverable

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context) — `claude-opus-5[1m]`.

### Implementation Plan (as executed, 2026-08-15)

T1 → T6 in order, against a live `mariadb:10.11.11` on port **13318** (`opencmdb-story63`,
`mysql://root:story63@127.0.0.1:13318/opencmdb_test`). ⚠️ The 6.1/6.2 container on 13316 was
GONE at contexting; do not inherit a container on faith. Commit `3ded1db` froze the green state
**before** the mutation pass, so every `git checkout -- <file>` in T5 reverted a mutation and not
a day's work.

- **T1** `repo.rs`: `ObservationRowSnapshot` + `snapshot_observation_records` (seven columns,
  `CAST(observed_at AS CHAR)`, `Option<String>` for `raw`); and
  `read_declared_provenance_for_test` **widened** from four columns to seven, returning
  `DeclaredRowSnapshot` — resolution (3), same name, same file. Its one caller (6.2's J3 test)
  updated. ✅ Measured immediately: `authorship` stays GREEN, which is the premise resolution (3)
  rests on.
- **T2** three store-backed guards in `main.rs`: the successful gesture, the 409 refusal and the
  422-domain refusal, each comparing the observation (seven columns) and the link (`id`
  included). Both populations asserted NON-EMPTY first.
- **T3** four guards: the declared side byte-identical across a real ingestion through
  `poll_ingest_resolve` + `FixtureConnector`; the two-sighting boundary; the three-sighting
  boundary; and the divergence after Guy's arbitrated DELETE.
- **T4/T4c** `xtask/src/observed_immutable.rs` — the eighth gate in its own module (Guy's
  arbitration), `is_table_reference_of` and `statement_after_of` parameterised by table, the other
  five helpers raised to `pub(crate)` unchanged. Module doc updated to enumerate eight; `report()`
  padding widened `{:<14}` → `{:<18}`.
- **T4b** 18 located probes under `xtask/probes/observed/` + README, driven END TO END, plus the
  real-tree and fail-closed tests.
- **T5** the mutation pass below. **T6** register and documents.

### Debug Log — prove-to-red (T5), predictions FIRST, each carrier read from its own message

| id | mutation | result | carrier, read from its own message |
|---|---|---|---|
| **M1** | `UPDATE observation_record` inside the documenting transaction | **RED ×2** | assertion `documenting moved a byte of the observed record` (`raw` "…blob, é" → "documented") **and** the gate, `1 overwriting access(es)`. **Dual-carrier, both messages read** |
| **M1b** | the same write in a **new, UNCALLED** `repo.rs` fn | **product suite GREEN (363 + 161, 0 failed); xtask RED; gate RED** | assertion `the_observed_gate_is_green_on_the_real_tree` + the gate message. 🔴 **The validation's "the whole workspace suite stays green" is FALSE on this tree** — it was measured on a prototype gate that had no real-tree test. The PRODUCT suite stays green; the gate's own test reds. The claim is narrowed accordingly |
| **M2** | gate body replaced by `Ok((true, …))` | **RED ×2** | assertions `11 of 18 probes got the wrong verdict` and `a missing root must not green`. 🔑 Story 5.12's structural finding — *the whole gate body deletable with the xtask suite green* — **does not recur**: the end-to-end carrier shipped with the gate rather than with its review |
| **M3** | close the subject's link inside the documenting transaction | **RED ×2** | assertion `documenting disturbed the identity link`, **on the row COUNT** (`right: []`) exactly as reading predicted, plus a precondition in the refusal test |
| **M4′** | refusal-path write **on its own connection** | **RED** | assertion `a nothing-to-document refusal moved a byte of the observed record` (`raw` → "attempted") |
| **M4-naive** | 🔑 **CONTROL** — the SAME write on the handler's transaction | **product suite GREEN; gate RED** | none behaviourally. **The pair is the finding**: the refusal returns without committing, the transaction rolls back, and *the transaction — not the guard's placement — is what makes the write invisible*. 5.11b's finding, reproduced deliberately |
| **M5** | `UPDATE declared_attribute` inside `resolver::resolve_within` | **RED ×3 + gate** | assertion `the scanner altered a declared field — NFR5's first assertion is broken`, plus `authorship`, `1 unsanctioned access(es) to declared_attribute`. **Dual-carrier** |
| **M6** | narrow the snapshot to exclude `raw` (with M1 applied) | 🔴 **RED, where GREEN was predicted** | my own precondition guard `a NULL raw would put a column in the comparison that carries nothing` fires **before** the comparison. **Not executable as a control** — story 5.13's assertion-order family, a fourth occurrence. It does prove that guard load-bearing |
| **M6-bis** | 🔑 **CONTROL, redesigned** — exclude `raw` from the EQUALITY only | **GREEN, as intended** | measures what M6 meant to: `raw` is the column carrying M1's detection. Narrow the comparison and the mutation goes invisible |
| **M7** | `gap::project` emits `"ip"` for `"ipv4"` | **RED ×4** | assertion on `gaps.len()` — the gap **vanishes** (`left: 0`). The corrected prediction holds; the first draft had imported 6.2's M11 with its polarity reversed |
| **M8** | one probe's pinned line shifted by one | **RED ×1** | assertion naming the location: `reds, but not at the line it must name (planted.rs:3:)` while the gate says `planted.rs:2:` |

**Eleven ids run — nine mutations and two CONTROLS.** ⚠️ **The headline is NOT "nine reds".**
M4-naive and M6-bis are GREEN **by design** and each is the point of its pair; M1b is GREEN on
the product suite and RED on the gate, which is what it exists to show; and **M6 diverged from
its prediction** and was replaced rather than quietly re-labelled.

**Carriers are MIXED and named per row.** M1 and M5 are dual-carrier (assertion + gate message);
M2, M3, M4′, M6, M7, M8 and M1b's xtask half are assertion-carried; the gate halves are
gate-message-carried. **Zero compiler-carried and zero `.expect()`-carried.**
*"Every red assertion-carried" is NOT claimed* — three reds here are a gate's own output.

🔴 **Three findings the pass produced, each recorded rather than smoothed over**: M6's
non-executability (and M6-bis, which measures what it meant to); M1b's suite-green claim narrowed
from *workspace* to *product*; and the M4′/M4-naive pair, which turns *"a rolled-back write and an
absent write are indistinguishable"* from a sentence into a measurement.

### Completion Notes

- **All 8 ACs MET.** AC1 ← M1, M3, M4′ (+ M6-bis as the `raw` control); AC2 ← M5, M7; AC3 ← M1b,
  M2, M8; AC4 ← the gate's rustdoc and the register; AC5–AC8 ← the tree checks below.
- ⚠️ **ONE LIVE COUNT, HERE (AC8): 580 → 590 tests — 363 bin + 161 core + 66 xtask** — verified
  against the live `mariadb:10.11.11`, 0 failed. The twins cite this file and carry no number.
- **Eight gates green** + `views-hash ℹ STALE exit 0`. `file-size`: 32 files, largest **1849**
  (against 1829 before) — Guy's module-split arbitration kept the prescribed rustdoc AND the
  ceiling, where growing `main.rs` would have reached ≈ 2018.
- **`observed-immutable` runs with an EMPTY allowlist across 36 files**, `docker/` included.
- Verified unchanged: **no migration**; `page.rs` and `crates/opencmdb-bin/templates/` at zero
  diff; `Cargo.lock` untouched; no new crate; `epics.md` not edited; 28 fixtures; trap gate still
  RED at 26/15/11 by design.
- 🔑 **What the story could NOT close, stated plainly**: the divergence half of NFR5's first
  assertion is unreachable through the documenting gesture (the test performs its own DELETE) and
  **cannot fire at all on the shipped connector**. Registered, owner named. *"NFR5 is covered by
  anti-regression tests" is now true — at the width the register states, and no wider.*

### File List

- `crates/opencmdb-bin/src/repo.rs` — `ObservationRowSnapshot` + `snapshot_observation_records`;
  `DeclaredRowSnapshot`; `read_declared_provenance_for_test` widened to seven columns
- `crates/opencmdb-bin/src/main.rs` — seven new store-backed guards and their fixtures; the 6.2
  J3 caller updated for the widened reader
- `xtask/src/observed_immutable.rs` — **NEW**: the eighth gate and its rustdoc
- `xtask/src/main.rs` — `mod observed_immutable`; the gate wired into `run_ci`; module doc
  enumerating eight; `is_table_reference_of` / `statement_after_of` parameterised; five helpers
  raised to `pub(crate)`; `report()` padding; `OBSERVED_PROBES` and four tests
- `xtask/probes/observed/` — **NEW**: 18 probes + `README.md`
- `_bmad-output/implementation-artifacts/6-3-nfr5-remaining-assertions.md` — this file
- `_bmad-output/implementation-artifacts/deferred-work.md` — the register rows
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status
- `_bmad-output/implementation-artifacts/6-1-write-route-writes-nothing.md` — register row (4)'s
  stale `Status: review` corrected
- `CLAUDE.md`, `docs/project-context.md` — the twins, citing this file and carrying no count

### Change Log

- **2026-08-15 — contexted**, then VALIDATED by two fresh-context layers the same day; two
  arbitrations raised and taken by Guy. Status → `ready-for-dev`.
- **2026-08-15 — developed.** T1–T6, eleven mutation ids, 580 → 590 tests, eight gates green.
  Status → `review`.
