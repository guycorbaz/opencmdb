# Story 4.12: Trap family — cloned/spoofed MAC (the inverse trap)

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author of the trap corpus,
I want the cloned-MAC family committed in positive AND negative form,
so that the **catastrophic failure — a false MERGE — has its own dedicated traps**: two distinct hosts
presenting the same MAC must never be fused (the failure that makes an operator lose trust and
uninstall), while an identical MAC that nothing opposes must still merge — vigilance against clones is
no licence for cowardice.

## Acceptance Criteria

1. **Given** two distinct hosts presenting the same MAC (identical `Mac` fact bytes, DIFFERENT synthetic
   hostnames, both alive in one scan), **when** the primary-form trap is scored, **then** it is a
   **`must-not-merge`**: fusing them is the failure that makes an operator lose trust and uninstall
   (epic AC1; D10 architecture.md:511-515 *"two hosts fused → the operator loses trust and uninstalls"*;
   D18's first column, architecture.md:1232). The expectation names the rule that **OPPOSES** the merge —
   the **REUSED** `l2-different-hostname` (coined in 4.11; D13's canonical conflict illustration is
   *"MAC identical AND hostname different"*, architecture.md:935) — never the rule that was tempting
   (`l1-exact-mac`).

2. **And** the inverse form — the SAME cloned MAC judged against a later re-sighting of the SAME host
   (identical MAC, the SAME hostname, only the lease moved) — is a **`must-merge`** naming `l1-exact-mac`
   (the rule that FIRES): vigilance against cloned MACs must not teach the engine to fear every identical
   MAC. This is D18's anti-cowardice middle column (architecture.md:1233) applied to THIS family — an
   engine that reacts to the cloned-MAC trap by abstaining on all exact-MAC matches is demolished here.

3. **And** the corpus **records that no database CHECK can detect a false merge — the schema makes it
   revisable and traceable, not impossible** (epic AC2, D18): the trap-file header carries D18's residual
   fragility verbatim (architecture.md:1263-1265: *"no `CHECK` detects a false merge. The schema makes a
   false merge revisable and traceable; it does not make it impossible."*) together with D21's renunciation
   backing it (architecture.md:1470-1473: **NO unique index on `interface.mac_canon`, deliberate** — *"a
   cloned MAC = two real interfaces, same MAC. A UNIQUE would turn the exact case we must ABSTAIN on into
   a 500."*), and `fixtures/scenario/traps/README.md` (the reality-debt register file) gains a short
   "the schema is not a backstop" passage stating the same two facts — the corpus is the only pre-release
   guard against the false merge, which is exactly why this family exists. The **evidence-free clone**
   (same MAC, no opposing signal at all) is named there as the residual fragility that **cannot be a
   trap** — see Dev Notes — without adding any row to the register table (the register opens empty and
   takes only cases reality produced).

4. **And** every expectation in the family carries its mandatory **one-sentence `reason`** (D19;
   `Trap::validate`, 20–300 chars, single line): the `must-not-merge` reason states that the two presences
   are different physical hosts, one wearing a clone of the other's MAC, and that the DISTINCT HOSTNAMES
   oppose the merge the identical MAC tempts; the `must-merge` reason states that only the lease moved and
   nothing opposes the exact-MAC match. Both name the observed values they rest on, and both claims are
   checkable against the committed bytes (example.toml's lesson: a reason claiming what the bytes
   contradict gets caught in review).

5. **And** both traps declare `family = "cloned-mac"`, so the corpus-completeness check (story 4.7b,
   `incomplete_families`) sees the family present in **BOTH decision poles** (≥1 `must-merge` AND ≥1
   `must-not-merge`) and the gate stays green. Two traps, no `must-abstain` — the honestly-ambiguous
   cloned-MAC edge (same MAC, NO discriminator) is deliberately NOT a trap (Dev Notes); this is a
   two-trap family exactly like 4.9/4.10.

6. **And** the family is committed as **two NEW locked corpus artefacts** —
   `fixtures/scenario/replay/cloned-mac.jsonl` (the three presences it judges) and
   `fixtures/scenario/traps/cloned-mac.toml` (the two traps) — **both added to `fixtures/MANIFEST.toml`
   with their sha256** in the same commit (a **deliberate corpus bump**, exactly like 4.9/4.10/4.11).
   After this story `cargo xtask ci`'s `fixtures` gate reports **13 artefacts, all sha256 match, no
   orphan** (11 existing + 2 new). The README edit needs NO manifest entry (READMEs are orphan-exempt
   by name).

7. **And** the committed replay stream carries **synthetic values only** — every MAC locally-administered
   (byte 0 = `0x02`, `locally_administered: true`), RFC 5737 documentation IPs (`192.0.2.0/24`), hostnames
   beginning `doc-` — so the privacy walk `the_corpus_carries_no_real_network_data`
   (`assert_facts_are_synthetic`, fixtures.rs:867-913, the `doc-` check at :879) passes over the new
   stream unchanged (D19; [[no-private-data-in-artifacts]]).

8. **And** the three committed-corpus assertions that hard-code the trap count are updated from **10 to
   12** in the same change (`example.toml`'s 3 + `randomized-mac.toml`'s 2 + `multi-nic.toml`'s 2 +
   `shared-hardware-vm.toml`'s 3 + this family's **2**): `report.discovered()` at `trap_gate.rs:390` and
   `:426`, the `"10 trap(s) discovered"` render string at `:408`, plus the two adjacent comments (`:389`
   corpus breakdown, `:418` "stays 10"). The scratch-corpus tests are **NOT** touched. At v0.1 `scored()`
   and `failures()` stay **0** for the committed corpus: no answer producer exists (Epic 5), so the family
   is discovered, parsed and validated (obs_ids resolve, reasons present, both poles present) but scored
   by nothing.

## Tasks / Subtasks

> **⚠️ THE RULE-VOCABULARY DECISION IS THE OPPOSITE OF 4.11's — read the "Rule vocabulary" Dev Note
> before writing the `.toml`.** This family COINS NOTHING. The opposing rule is the REUSED
> `l2-different-hostname` (coined by 4.11 as the Epic-5 hostname-conflict contract) and the firing rule is
> the original `l1-exact-mac`. Coining an `l1-different-hostname` or an `l1-cloned-mac` here would create
> two ids for one signal and split the vocabulary. If a reviewer disputes an `l2-` id opposing an
> `l1`-tempted merge, the framing is documented in the Dev Notes — raise it in review, do not silently
> coin.

- [x] **Task 1 — write the replay stream** `fixtures/scenario/replay/cloned-mac.jsonl` (AC: 1, 2, 7) — pure data, no code
  - [x] Author **three observations** in one stream, JSONL (fields in order: `obs_id`, `connector_id`,
        `observed_at`, `scope` {`l2_domain`, `vantage`}, `facts`, `raw`; facts per line: one `Mac`, one
        `IpV4`, one `Hostname` — no `Uplink`: the discriminator is the hostname, topology plays no role
        in this family). **Structural template: a line of `shared-hardware-vm.jsonl` (W1) minus its
        `Uplink` fact** — that stream carries the exact `Hostname` serialization to copy;
        `randomized-mac.jsonl` has the same envelope but NO `Hostname` fact, so do not template from it
        alone. Use the
        **fresh `obs_id` prefix `acacacac`** — verified free in every committed stream (`aaaa…`/`bbbb…`/
        `cccc…`/`dddd…`/`eeee…`/`ffff…`/`abababab` are all taken). **Before committing, `grep -r acacacac
        fixtures/scenario/` and confirm the only hits are the new `.jsonl` and its `.toml`.**
        - K1 = `acacacac-0000-4000-8000-000000000001`
        - K2 = `acacacac-0000-4000-8000-000000000002`
        - K3 = `acacacac-0000-4000-8000-000000000003`
  - [x] Keep `connector_id`, `l2_domain` and `vantage` **identical across all three** (the within-stream
        provenance/scope checks, 4.5a/4.5b): reuse the corpus's synthetic UUIDs — `connector_id`
        `33333333-3333-4333-8333-333333333333`, `l2_domain` `11111111-1111-4111-8111-111111111111`,
        `vantage` `22222222-2222-4222-8222-222222222222`.
  - [x] **The cloned MAC is ONE byte-identical `Mac` fact on ALL THREE lines** —
        `{"Mac":{"addr":[2,0,94,0,83,112],"locally_administered":true}}` (`02:00:5e:00:53:70`, fresh in
        the corpus, locally-administered). This is the point: the MAC alone cannot tell the three
        presences apart.
  - [x] **K1 — the original host** (feeds BOTH traps): the cloned `Mac`, `{"IpV4":{"addr":"192.0.2.112"}}`,
        `{"Hostname":{"name":"doc-host-echo","source":"Dhcp"}}`, `observed_at` `2026-01-05T00:00:00Z`.
  - [x] **K2 — the clone-wearing second host** (feeds the `must-not-merge`, AC1): the SAME cloned `Mac`
        (identical bytes), `{"IpV4":{"addr":"192.0.2.113"}}`,
        `{"Hostname":{"name":"doc-host-foxtrot","source":"Dhcp"}}`, `observed_at` `2026-01-05T00:00:00Z`
        (the same scan as K1 — two hosts answering at once is the natural cloned-MAC sighting; the
        timestamp carries no assertion, the hostname does).
  - [x] **K3 — the original re-seen an hour later** (feeds the `must-merge`, AC2): the SAME cloned `Mac`,
        `{"IpV4":{"addr":"192.0.2.114"}}` (the lease moved),
        `{"Hostname":{"name":"doc-host-echo","source":"Dhcp"}}` (SAME hostname as K1), `observed_at`
        `2026-01-05T01:00:00Z`.
  - [x] **`raw` is `null`** on every line. **Synthetic-only, non-negotiable (AC7):** every MAC
        locally-administered, every IP `192.0.2.x` (`.112`/`.113`/`.114` are fresh), every hostname `doc-*`
        (`doc-host-echo`/`doc-host-foxtrot` are fresh — `doc-host-a/-c/-d` and `doc-vm-alpha/-beta` are
        taken). A real vendor MAC, a non-5737 IP, or a non-`doc-` hostname reds
        `the_corpus_carries_no_real_network_data`.
  - [x] **End the file with a single trailing newline** (every committed artefact does) — settle this
        BEFORE Task 4 hashes the bytes; the same applies to Task 2's `.toml`.

- [x] **Task 2 — write the family trap file** `fixtures/scenario/traps/cloned-mac.toml` (AC: 1, 2, 3, 4, 5) — pure data, no code
  - [x] Open with a header in the voice of the sibling families, carrying **two things**: *(a)* the family
        statement — the cloned/spoofed MAC is **the inverse trap** (architecture.md:521): where every other
        family guards a false split, this one guards the **false MERGE**, the catastrophic direction
        (D10:511-515 — *"false-merge is catastrophic and asymmetric… false-split is benign"*); the
        cloned-MAC false-merge is an **L1** failure (D12, architecture.md:894) and D13's named conflict
        case (architecture.md:971: *"a `Decisive`, ≥1 `Opposes` → `Ambiguous` ← the cloned-MAC case"*) —
        both traps below judge `scenario/replay/cloned-mac.jsonl`, and together they pin: an identical MAC
        is decisive ONLY when nothing opposes it. *(b)* **the AC3 record, verbatim from D18** — *"no
        `CHECK` detects a false merge. The schema makes a false merge revisable and traceable; it does not
        make it impossible."* (architecture.md:1263-1265) — with D21's backing renunciation (NO unique
        index on `interface.mac_canon`: *"A cloned MAC = two real interfaces, same MAC. A UNIQUE would turn
        the exact case we must ABSTAIN on into a 500."*, architecture.md:1470-1473), and the consequence
        stated plainly: **this family is the only pre-release guard against the false merge — the schema
        cannot be one.** The D18 sentence is the ONE string that must be verbatim (AC3 promises it); the
        D21 renunciation may be quoted or tightly paraphrased, but if quoted, spell it as the source does
        (*"…same MAC. A UNIQUE would turn…"* — period then capital, not a semicolon). Keep every id whole
        on its line (4.11's review lesson: a hyphen-broken id greps as absent).
  - [x] Both traps carry `replay = "scenario/replay/cloned-mac.jsonl"` (the key the sibling files use),
        and their `observations` arrays spell the FULL UUIDs from Task 1 — e.g. the `must-not-merge`'s is
        `observations = ["acacacac-0000-4000-8000-000000000001", "acacacac-0000-4000-8000-000000000002"]`
        (K1, K2); a stream↔trap spelling mismatch reds `read_traps` with `DanglingObservation`.
  - [x] **The `must-not-merge` (primary form, AC1).** Judges `[K1, K2]`. `family = "cloned-mac"`.
        `expect = { must-not-merge = { rule = "l2-different-hostname" } }` — REUSED from
        `shared-hardware-vm.toml`, spelled identically. `reason` (one sentence, names the opposing
        values): *"these two presences are different physical hosts, one wearing a clone of the other's
        MAC — the identical MAC tempts the L1 merge but the distinct hostnames doc-host-echo and
        doc-host-foxtrot oppose it, and fusing two real hosts is the false merge an operator never
        forgives."*
  - [x] **The `must-merge` (inverse form, AC2).** Judges `[K1, K3]`. `family = "cloned-mac"`.
        `expect = { must-merge = { rule = "l1-exact-mac" } }` — the original L1 rule, spelled as in
        `example.toml`/`randomized-mac.toml`. `reason`: *"both presences carry the identical MAC and the
        same hostname doc-host-echo an hour apart, so only the lease moved — nothing opposes the
        exact-MAC match, and vigilance against cloned MACs is no licence to fear every identical MAC."*
  - [x] **Measure both reasons** (`wc -m`, 20–300 bound, single line, no control chars) and record the
        counts in the Debug Log. If one exceeds 300, tighten it — never split it into two sentences.
  - [x] **Trap ids unique across the corpus:** `cloned-mac-must-not-merge`, `cloned-mac-must-merge`
        (sibling files use `<family>-must-*`; no clash. Two guards: `TrapFile::validate` reds a
        case-folded duplicate WITHIN one file, trap.rs:316-330; `score_corpus` reds an exact-match
        duplicate ACROSS files, trap_gate.rs:234-251 `DuplicateTrapId`).

- [x] **Task 3 — record the schema-is-no-backstop limit in the register file** `fixtures/scenario/traps/README.md` (AC: 3) — prose, orphan-exempt
  - [x] Add a short section (place it between "The honest limit" and "The register") titled e.g.
        **"The schema is not a backstop"**, stating: *(a)* D18's sentence verbatim (architecture.md:
        1263-1265) — no `CHECK` detects a false merge; the schema makes it revisable and traceable, not
        impossible; *(b)* the D21 renunciation that enforces it (no UNIQUE on `interface.mac_canon` —
        expressing MAC uniqueness in DDL would 500 the exact case the engine must abstain on); *(c)* the
        consequence: the false merge is guarded ONLY by this corpus (the cloned-mac family, story 4.12)
        before release and by an operator's unlink after one (D14, architecture.md:1015-1017: *"a bad
        link is UNLINKED, never erased"*); *(d)* the **evidence-free clone** — same MAC, no opposing
        signal at all — is the residual fragility NO trap can express: a trap needs opposing evidence in
        its bytes, and an evidence-free clone is byte-identical to an honest re-sighting (Dev Notes).
  - [x] Do **NOT** add a row to the register table — the register takes only cases reality produced
        (the README says inventing one "would manufacture false coverage"); this is a *limit statement*
        in prose, like "The honest limit" above it. Do **NOT** add the README to the manifest
        (orphan-exempt by name, no re-hash).

- [x] **Task 4 — lock the two new artefacts** `fixtures/MANIFEST.toml` (AC: 6) — deliberate corpus bump
  - [x] Append TWO `[[artefact]]` entries (paths `scenario/replay/cloned-mac.jsonl` and
        `scenario/traps/cloned-mac.toml`), each with a one-line comment naming the story and what the
        artefact is, mirroring the 4.9/4.10/4.11 entries. Compute each `sha256` from the committed bytes
        (`sha256sum <file>`) and paste the hex. **Author Tasks 1–2 FIRST and finish every byte (including
        the header prose), THEN hash** — a later edit, even a trailing newline, invalidates the sha256.
  - [x] Do NOT touch the eleven existing entries or their sha256 — any edit there is an unrelated EDITED
        finding.

- [x] **Task 5 — update the committed-count assertions** `crates/opencmdb-bin/src/trap_gate.rs` (AC: 8) — three edits, tests only
  - [x] `the_committed_corpus_is_discovered_and_scored_by_nothing` (`:390`): `report.discovered()` `10` →
        `12`; keep `scored()`/`failures()` at `0`. Update the breakdown comment at `:389` ("… ten in the
        committed corpus") to add cloned-mac's **two** and say **twelve**.
  - [x] `the_report_says_plainly_that_nothing_was_scored` (`:408`): `"10 trap(s) discovered"` →
        `"12 trap(s) discovered"`. Leave `"0 scored"` and `"0 truth-table failure(s)"` unchanged.
  - [x] `a_trap_with_no_answer_is_discovered_but_not_scored` (`:426`): `10` → `12`, and the comment at
        `:418` "stays 10" → "stays 12" (`scored()` stays `1` — it still answers only
        `example-must-abstain`).
  - [x] **Do NOT touch the scratch-corpus tests** (`each_column_can_be_driven_red`,
        `a_one_sided_family_reddens_the_gate_on_its_own`, the `discovered(), 3/1/0` assertions at
        `:499/:542/:568`, …) — they build their own temp dirs; their counts are unrelated.
  - [x] **Prove the count coupling is real (record it).** With BOTH `cloned-mac.jsonl` AND
        `cloned-mac.toml` on disk (the stream must be there — obs_id resolution runs before counting, so
        a missing stream fails the test differently, at the "corpus reads" expect) and the assertions
        still at `10`, run `cargo test -p opencmdb-bin --locked
        the_committed_corpus_is_discovered_and_scored_by_nothing`: it must RED with `left: 12, right: 10`
        — proving the two new traps parsed, validated, and resolved their obs_ids against the committed
        stream. Then update to `12` and watch it green. Record both runs in the Debug Log.

- [x] **Task 6 — gates and the corpus lock** (AC: 5, 6, 7, 8)
  - [x] `cargo fmt --all` · `cargo clippy --workspace --all-targets --locked -- -D warnings` ·
        `cargo test --workspace --locked` · `cargo run -p xtask --locked -- ci` — all green.
  - [x] `fixtures` gate reports **13 artefacts, all sha256 match, no orphan** (11 existing + 2 new; the
        README edit is exempt). Confirm the new stream and trap file are BOTH listed and BOTH match.
  - [x] Confirm the committed corpus still `passed()`: `discovered() == 12`, `scored() == 0`,
        `failures() == 0`, `rule_mismatches` empty, `incomplete_families` empty (cloned-mac carries both
        poles). Grep `"10 trap"` / `discovered(), 10` across `crates/` to catch any count assertion this
        story missed.
  - [x] Privacy + validity walks pass over the new files automatically
        (`the_corpus_carries_no_real_network_data`, `every_replay_stream_in_the_corpus_is_valid`,
        `every_trap_file_in_the_corpus_is_valid`, `no_obs_id_is_shared_across_replay_streams`) — no
        harness change.
  - [x] `Cargo.lock` unchanged (two data files, one README passage, one manifest bump, three test
        literals). `architecture-views.md` NOT regenerated (`ℹ views-hash STALE` is by design — Epic-4
        milestone, not here). The eleven existing artefacts byte-for-byte unchanged
        (`git status --short`).

### Review Findings

- [x] [Review][Patch] The register's residual-fragility sentence names only the **evidence-free** clone, but the **perfect** clone (MAC **and** hostname both copied) is equally byte-indistinguishable from an honest re-sighting — and the family's `must-merge` actively MANDATES merging that byte-shape. The honest limit understates the residue it exists to record; widen the sentence to cover both shapes (README is orphan-exempt — no re-hash) [fixtures/scenario/traps/README.md:40-42] — **FIXED**: the sentence now names both shapes ("two shapes stay untrappable by construction") and states plainly that the family's own `must-merge` mandates merging the perfect-clone byte-shape; no manifest change (README orphan-exempt), `fixtures` gate re-run green.
- [x] [Review][Defer] Family replay streams are never exercised through `FixtureConnector::load`'s admissibility checks — only `minimal.jsonl` is; the corpus walks gate parseability/validity, not connector-level admissibility. Pre-existing since 4.9, true of every family stream; cloned-mac's contents would pass those checks anyway (verified by the Edge layer) [crates/opencmdb-bin/src/fixture_connector.rs] — deferred, pre-existing

## Dev Notes

### The shape of this story in one paragraph

The cloned-MAC family — **the inverse trap** (architecture.md:521): every family so far guarded a false
SPLIT (two presences of one thing kept apart); this one guards the **false MERGE**, the direction D10
calls catastrophic and asymmetric (*"two hosts fused → the operator loses trust and uninstalls; false-split
is benign"*, architecture.md:511-515). Like 4.9/4.10/4.11 it is almost entirely DATA: two new committed
artefacts (a **three**-observation replay stream and a **two**-trap family file), a deliberate
`MANIFEST.toml` bump, three test-literal updates (**10 → 12**), plus one README passage (AC3's record).
No engine, no new harness code, no new rule id — this family **reuses** `l2-different-hostname` (4.11's
coinage) and `l1-exact-mac` (the original). What is genuinely NEW: it is the first family whose PRIMARY
form is the `must-not-merge` (the catastrophe has its own dedicated traps, per the epic), and the first
whose header must carry a schema-limit record (AC3).

### Rule vocabulary — this family coins NOTHING (the opposite of 4.11's decision, and why)

**Read this before writing the `.toml`.** 4.11 had to coin because no hostname rule existed. It exists
now, and this family needs exactly it:

- The tempting rule is `l1-exact-mac` — the identical MAC is D13's `Decisive` signal. The expectation must
  name the rule that **OPPOSES**, never the tempting one (trap.rs:66-73; example.toml: *"a refusal without
  a named rule cannot be told apart from an engine that found nothing"*).
- The opposing signal is the **hostname conflict** — D13's own illustration of the cloned-MAC conflict is
  verbatim *"'MAC identical' AND 'hostname different'"* (architecture.md:934-937), and the rule id for
  hostname-conflict already exists: **`l2-different-hostname`**, coined by 4.11 and recorded as the Epic-5
  contract (a hostname-conflict rule in the L2 cascade). One signal, one rule id, reused across families —
  coining `l1-different-hostname` or `l1-cloned-mac` would create a second id for the same signal and
  fracture the vocabulary 4.15 (hostname-collision) will also need.
- **The layer nuance, named so a reviewer sees it was weighed:** D12 classifies the cloned-MAC false-merge
  as **L1** (architecture.md:894 — the attack is on MAC-as-interface-identity), yet the opposing rule
  carries an `l2-` prefix because the hostname is an L2 *signal* (architecture.md:891). That is not a
  contradiction: the rule id names the SIGNAL that opposes, and D13's verdict algebra is exactly the place
  where an L1-decisive MAC meets a hostname `Opposes` (architecture.md:971). If a reviewer prefers an
  L1-flavoured id for this refusal, raise it in review — do NOT silently coin one.

### Why the primary form is `must-not-merge` and not `must-abstain` (D13:971 read carefully)

D13's truth table says the cloned-MAC case — *a `Decisive`, ≥1 `Opposes`* — resolves to **`Ambiguous`**
(architecture.md:971): the engine Epic 5 builds will likely ABSTAIN on [K1, K2] and expose the conflict,
not refuse it. That is fine, and the trap is still a `must-not-merge`, for three reasons:

1. **The epic dictates the column** (epics.md:1143-1145): *"it is a `must-not-merge`: fusing them is the
   failure that makes an operator lose trust and uninstall."* D18's `must-not-merge` column fails on
   exactly one thing — **a merge** (architecture.md:1232).
2. **The scoring already agrees:** `score(must-not-merge, Abstained) = Pass` (score.rs:183, pinned by
   `must_not_merge_fails_only_on_a_merge` at score.rs:646-651 and DEFENDED by
   `an_engine_that_abstains_on_everything_is_demolished_by_the_middle_column` — D18's anti-cowardice
   mechanism is only true because this cell passes). So a D13-correct engine that answers `Ambiguous`
   PASSES this trap; only the catastrophic merge reds it. The trap asserts exactly what the epic wants —
   *never fuse* — without over-constraining Epic 5's design space (refuse vs abstain).
3. **The rule comparison composes:** `WrongRule` fires only when both sides carry a rule (score.rs:224-237)
   — i.e. only if the engine REFUSES, in which case it must cite `l2-different-hostname`. An abstention has
   no rule to be wrong and passes as a plain `Pass` (score.rs:984).

Labelling this trap `must-abstain` instead would FAIL a future engine that legitimately refuses via the
hostname-conflict rule — it would over-commit Epic 5 to one resolution of D13's `Ambiguous`. The
`must-not-merge` column is the minimal true assertion. (If a reviewer wants a third, `must-abstain` trap
pinning D13:971's outcome specifically, that is a corpus EXTENSION to raise in review — not a relabelling
of this one.)

### Why the stream must carry the opposing evidence — and why the evidence-free clone is NOT a trap (AC3)

A trap's oracle is its author, but its bytes must make the expectation *reachable*. Two presences carrying
ONLY the same MAC are **byte-identical to an honest re-sighting** (that is example.toml's and
randomized-mac's `must-merge` shape!) — an engine shown nothing but an identical MAC and told "must not
merge" would be required to detect a clone with zero evidence, which no engine can do and D13 does not ask
(*a `Decisive`, no `Opposes` → `Match`*, architecture.md:970). So:

- **K2 carries the discriminator** (`doc-host-foxtrot` vs K1's `doc-host-echo`) — the trap tests whether
  the engine WEIGHS the counter-evidence instead of letting MAC equality steamroll it.
- **The evidence-free clone is the residual fragility AC3 records in prose**: no trap can express it, no
  `CHECK` can catch it (D18, architecture.md:1263-1265), no UNIQUE may try (D21, architecture.md:1470-1473)
  — the schema's whole answer is revisability (D14: *"a bad link is UNLINKED, never erased"*,
  architecture.md:1015-1017). Writing that down in the corpus (trap-file header + register README) IS the
  second half of this story. It goes in prose, NOT as a register-table row — the register takes only cases
  reality produced.

### The two poles, and which observations feed which (AC1/AC2)

| Pole | Rule | Judges | The point |
|---|---|---|---|
| **`must-not-merge`** (primary) | `l2-different-hostname` (REUSED — the rule that OPPOSES) | `[K1, K2]` — identical MAC, hostnames `doc-host-echo` vs `doc-host-foxtrot`, same scan | the false MERGE: two real hosts, one cloned MAC — fusing them is the catastrophe (D10/D18) |
| **`must-merge`** (inverse) | `l1-exact-mac` (the rule that FIRES) | `[K1, K3]` — identical MAC, SAME hostname, lease moved an hour later | anti-cowardice: an identical MAC that nothing opposes still merges (D13:970) |

The asymmetry mirrors 4.11's, inverted: here the `must-not-merge` is the byte-aligned pole (the distinct
hostnames are IN the bytes) and the `must-merge` rests on the author's "same host, lease moved" oracle
stated in the reason (D19). K1 feeds both traps — the established idiom (4.9's and 4.11's streams share
their first observation across poles).

### How a trap is validated — the checks the new files must survive

Identical to 4.11 (nothing changed since): `read_traps` / `Trap::validate` (trap.rs:250-315) enforce
obs_id resolution against the named `replay` stream (`DanglingObservation`), reason bounds (20–300 chars,
single line, no control char), a rule on every decision, unique trap ids (case-folded), clean family
tokens; `incomplete_families` requires both poles per family. The corpus walks
(`the_corpus_carries_no_real_network_data`, `every_trap_file_in_the_corpus_is_valid`,
`every_replay_stream_in_the_corpus_is_valid`, `no_obs_id_is_shared_across_replay_streams`) pick the new
files up automatically — no harness change; a malformed file reds the walk by name.

### The corpus lock is a DELIBERATE bump (AC6) — same mechanism as 4.9/4.10/4.11

Two real artefacts locked in both directions (EDITED and ORPHAN both red). Order of operations: **finish
every byte of both files (header prose included), THEN hash, THEN paste.** 4.11's review re-hashed once
because a header rewrap came after hashing — do the wrap-check (ids whole on their line, greppable)
BEFORE hashing this time. READMEs stay exempt: the Task-3 edit needs no manifest entry.

### Project Structure Notes

- **NEW (locked):** `fixtures/scenario/replay/cloned-mac.jsonl` (3 observations, one byte-identical
  cloned `Mac` on all three), `fixtures/scenario/traps/cloned-mac.toml` (2 traps,
  `family = "cloned-mac"`, header carrying the AC3 record). Both listed in `fixtures/MANIFEST.toml`
  with sha256.
- **Updated:** `fixtures/MANIFEST.toml` (two `[[artefact]]` entries — the deliberate bump);
  `fixtures/scenario/traps/README.md` (the "schema is not a backstop" passage — orphan-exempt, no
  manifest entry); `crates/opencmdb-bin/src/trap_gate.rs` (three `#[cfg(test)]` count literals 10 → 12,
  plus the `:389`/`:418` comments — tests only, no production logic).
- **Unchanged, expected:** the eleven existing manifest entries (byte-for-byte); `trap.rs` / `score.rs` /
  `gap/mod.rs` / `fixtures.rs` / `trap_gate.rs` production paths; `Cargo.lock`; every other `.rs`.
  `discover_trap_files` / `read_traps` / `incomplete_families` are used AS-IS.
- **Out of scope, deliberately:** any engine or rule producer (Epic 5 owns the real
  `l2-different-hostname` and `l1-exact-mac` rules); a `must-abstain` third trap for D13:971's
  `Ambiguous` outcome (a corpus extension to raise in review if wanted — not this story); the DHCP-churn
  timestamps family (story 4.13 — this family's timestamps deliberately carry NO assertion); the
  hostname-collision family (4.15 — it will REUSE `l2-different-hostname`'s sibling semantics; do not
  pre-build it); `docs/project-context.md` / `CLAUDE.md` (fold in at the Epic-4 milestone with the
  `architecture-views.md` regeneration).

### Traps (mistakes this story must not make)

1. **Coining a rule id.** `l2-different-hostname` and `l1-exact-mac` both exist. A new `l1-*` id for the
   refusal splits one signal across two ids. (Review decision — documented in the Rule-vocabulary note.)
2. **Naming the tempting rule in the `must-not-merge`.** `l1-exact-mac` is what TEMPTS; the expectation
   names what OPPOSES (`l2-different-hostname`) — trap.rs:66-73, example.toml's contract.
3. **Labelling the primary form `must-abstain` because D13:971 says `Ambiguous`.** The epic dictates
   `must-not-merge`; an abstention already PASSES that column (score.rs:183); a `must-abstain` label
   would fail a legitimately-refusing engine. See the dedicated Dev Note.
4. **An evidence-free `must-not-merge`** (dropping K2's hostname): byte-identical to an honest
   re-sighting — the trap would demand clairvoyance. K2 MUST carry `doc-host-foxtrot`.
5. **Giving K3 a different hostname.** K3 is the SAME host re-seen (`doc-host-echo`); a different
   hostname would put an `Opposes` in the must-merge pair and collapse it into the K2 case.
6. **Making the MACs differ anywhere.** The three `Mac` facts are byte-identical — one string,
   copy-pasted. A differing byte silently turns the family into randomized-mac.
7. **Adding a register-table row for the evidence-free clone.** The register takes only cases reality
   produced; this is a prose limit-statement (AC3), not a queue entry.
8. **A hostname not prefixed `doc-`**, a non-5737 IP, or a non-locally-administered MAC — reds the
   privacy walk (fixtures.rs:867-913).
9. **Reusing an existing stream's `obs_id` prefix.** Fresh prefix `acacacac`; grep before committing.
10. **Forgetting the manifest bump, or hashing before the final byte** (including the header wrap-check —
    ids whole on their line, 4.11's review lesson).
11. **Leaving the count assertions at 10.** This family adds TWO, not three: 10 → 12 at
    `trap_gate.rs:390/:408/:426` + comments `:389`/`:418`. Scratch tests untouched.
12. **Touching the README's register table or adding the README to the manifest.** Prose section only;
    orphan-exempt by name.
13. **Regenerating `architecture-views.md`.** Epic-4 milestone, not this story.
14. **Claiming more than measured.** Name the command behind every count; record the proved-red count
    coupling and the measured reason char counts. [[claims-must-match-verification]]
15. **Inconsistent `connector_id`/`l2_domain`/`vantage` within the stream.** All three observations
    share the one `connector_id` and scope prescribed in Task 1 (the within-stream provenance/scope
    checks, 4.5a/4.5b).

### Latest technical specifics

No new crate, no version bump, no domain or harness code. Rust 1.96+, edition 2024. Two data files, one
README passage, one `MANIFEST.toml` bump, three `#[cfg(test)]` literals in `opencmdb-bin`. **Never invent
a version — pin from the committed `Cargo.lock`, which does not move here.**

### References

- [Source: _bmad-output/planning-artifacts/epics.md:1135-1147 — Story 4.12: the story sentence ("the
  catastrophic failure — a false MERGE — has its own dedicated traps") and the two ACs (must-not-merge on
  two distinct hosts presenting the same MAC, D10/D18; the corpus records that no database CHECK can
  detect a false merge — revisable and traceable, not impossible, D18)]
- [Source: _bmad-output/planning-artifacts/architecture.md:509-523 — D10: NFR4 as a triplet, false-merge
  catastrophic and asymmetric ("two hosts fused → the operator loses trust and uninstalls; false-split is
  benign"); the adversarial matrix naming "cloned/spoofed MAC (the inverse trap: false-merge)" at :521]
- [Source: _bmad-output/planning-artifacts/architecture.md:884-895 — D12: "A MAC identifies an INTERFACE,
  not a device"; the L1/L2 table (L1 signal = MAC, attacked by cloned MAC); "Cloned-MAC false-merge =
  L1" at :894]
- [Source: _bmad-output/planning-artifacts/architecture.md:934-937, 967-981 — D13: the conflict
  illustration "'MAC identical' AND 'hostname different'" (:935); the verdict truth table — "a
  `Decisive`, no `Opposes` → `Match`" (:970), "a `Decisive`, ≥1 `Opposes` → `Ambiguous` ← the cloned-MAC
  case" (:971); "the cloned-MAC false-merge falls out on its own — the strict false-merge target is
  obtained by construction, not by calibration" (:981-982)]
- [Source: _bmad-output/planning-artifacts/architecture.md:1224-1234, 1263-1265 — D18: the three-column
  table (must-not-merge fails on a merge — "the operator loses trust and uninstalls"); the AC3 sentence
  VERBATIM: "no `CHECK` detects a false merge. The schema makes a false merge revisable and traceable; it
  does not make it impossible. … That is a release risk, not a test detail."]
- [Source: _bmad-output/planning-artifacts/architecture.md:1470-1473 — D21: "NO unique index on
  `interface.mac_canon`. Deliberate. A cloned MAC = two real interfaces, same MAC. A UNIQUE would turn
  the exact case we must ABSTAIN on into a 500. The uniqueness of a MAC is a DECISION, not a constraint"]
- [Source: _bmad-output/planning-artifacts/architecture.md:1015-1017 — D14: a link is revisable without
  destroying anything; "a bad link is UNLINKED, never erased" (the revisability AC3 points at)]
- [Source: crates/opencmdb-core/src/trap.rs:31-38, 55-76, 250-315 — `RuleId` (a `String` until Epic 5
  closes it), `Expectation` (must-not-merge names the rule that OPPOSES, :66-73), `Trap::validate`
  (reason bounds, unique ids, family token)]
- [Source: crates/opencmdb-core/src/score.rs:175-187, 224-237 — the nine cells: `(MustNotMerge,
  Abstained) = Pass` at :183 (pinned by `must_not_merge_fails_only_on_a_merge` and defended by
  `an_engine_that_abstains_on_everything_is_demolished_by_the_middle_column`); `WrongRule` fires only
  when both sides carry a rule (:224-237), an abstention has no rule to be wrong (:984)]
- [Source: crates/opencmdb-bin/src/trap_gate.rs:389-426 — the three committed-count assertions (10 → 12)
  and their comments; scratch-test counts at :499/:542/:568 NOT touched]
- [Source: crates/opencmdb-bin/src/fixtures.rs:867-913, 1306 — `assert_facts_are_synthetic` (the `doc-`
  hostname check at :879, U/L-bit MACs, RFC 5737 IPs); the cross-stream obs_id guard]
- [Source: fixtures/scenario/traps/shared-hardware-vm.toml — story 4.11: the source of
  `l2-different-hostname` (REUSED here, spelled identically) and the family-header voice]
- [Source: fixtures/scenario/traps/example.toml:14-17 + randomized-mac.toml — the OPPOSING-rule contract
  ("a `must-not-merge` names the rule that OPPOSES the merge, not the one that was merely tempting") and
  the `l1-exact-mac` must-merge shape this family's inverse pole follows]
- [Source: fixtures/scenario/traps/README.md — the reality-debt register (Task 3's home): "The honest
  limit" section, the empty-register rule ("inventing a case … would manufacture false coverage"),
  README orphan-exemption]
- [Source: _bmad-output/implementation-artifacts/4-11-trap-shared-hardware-vm.md — the immediately prior
  family: structural model (fresh obs_id prefix, manifest bump, count update, hash-after-final-byte, the
  hyphen-broken-id review lesson), and the coining of `l2-different-hostname` this story reuses]

## Dev Agent Record

### Agent Model Used

claude-fable-5 (dev-story workflow, 2026-07-24).

### Debug Log References

- **Count-coupling proved red (Task 5).** With BOTH `cloned-mac.jsonl` and `cloned-mac.toml` on disk
  and the assertions still at `10`, `cargo test -p opencmdb-bin --locked
  the_committed_corpus_is_discovered_and_scored_by_nothing` RED with `assertion left == right failed …
  left: 12, right: 10` at `trap_gate.rs:390` — the two new traps were discovered, parsed, validated and
  their obs_ids resolved against the committed stream before the count was checked. Updated `10 → 12`
  at `:390/:408/:426` (+ the breakdown comment above `:390` and the "stays 12" comment at `:418`) and
  the suite went green.
- **Reason char counts measured** (python `len()`, `wc -m` semantics, 20–300 bound): must-not-merge
  **274**, must-merge **229** — both single-line (measured, no `\n`).
- **`acacacac` prefix verified free** before authoring (`grep -r acacacac fixtures/ crates/` → no hit)
  and after (`grep -rl` → only the new `.jsonl` and its `.toml`, as expected — the trap file references
  the stream's obs_ids).
- **Byte-identical cloned Mac verified** (python: the three lines' first fact serialize to ONE distinct
  JSON string). Both ids grep whole in the trap file (`grep -c "l2-different-hostname\|l1-exact-mac"` =
  4: twice in the header prose, twice in the expects). Both files end with a single `\n` (xxd) —
  settled BEFORE hashing.
- **sha256 of the two final artefacts** (`sha256sum`, hashed only after every byte was final): stream
  `659e53771114d4516623567e2d46fcedab75b075799f7a08fc61f28246045a9e`, trap file
  `6b84620933e8bd3da5aadcb1a9961d48292fab58e2a3b96c25d1e175dffdfbcb`; pasted verbatim into
  `MANIFEST.toml` (13 `[[artefact]]` entries counted after the bump).
- **Leftover-count grep clean:** `grep -rn "10 trap|discovered(), 10|stays 10|ten in the committed"
  crates/` → no hit after the three-literal update.

### Completion Notes List

- **The inverse trap shipped — first family whose PRIMARY form guards the false MERGE**:
  `cloned-mac-must-not-merge` / `l2-different-hostname` on [K1, K2] (identical MAC, hostnames
  `doc-host-echo` vs `doc-host-foxtrot`, same scan) + `cloned-mac-must-merge` / `l1-exact-mac` on
  [K1, K3] (identical MAC, same hostname, lease moved an hour later). Both poles present →
  `incomplete_families` empty, `passed()` green (all 107 bin tests pass, including the family walks).
- **This family coins NOTHING (the opposite of 4.11's decision, per the story):**
  `l2-different-hostname` is REUSED from `shared-hardware-vm.toml` (spelled identically,
  grep-verified), `l1-exact-mac` from `example.toml`/`randomized-mac.toml`. No new rule id enters the
  vocabulary.
- **AC3 record landed in BOTH prescribed homes:** the trap-file header carries D18's sentence verbatim
  ("no `CHECK` detects a false merge. The schema makes a false merge revisable and traceable; it does
  not make it impossible.") + D21's renunciation quoted as the source spells it ("A cloned MAC = two
  real interfaces, same MAC. A UNIQUE would turn the exact case we must ABSTAIN on into a 500.");
  `fixtures/scenario/traps/README.md` gained the "The schema is not a backstop" section (between "The
  honest limit" and "The register"), naming the evidence-free clone as the untrappable residual
  fragility — in prose, with NO row added to the register table.
- **Gates all green** (`cargo run -p xtask --locked -- ci`): `fixtures` reports **13 fixture(s) match,
  0 orphan** (11 existing + 2 new); `file-size` largest 884 (unchanged — the edits are data +
  `#[cfg(test)]` literals); `views-hash STALE` by design (ℹ, exits 0 — not regenerated in a story).
  `cargo fmt --all` (no change), `cargo clippy --workspace --all-targets --locked -- -D warnings`, and
  `cargo test --workspace --locked` (**107 bin + 86 core + 42 xtask, all pass**) clean. ⚠️
  `DATABASE_URL` unset locally, so the MariaDB-backed tests skipped as always (CI runs them).
- **Privacy + validity walks pass over the new files automatically** — no harness change:
  `the_corpus_carries_no_real_network_data`, `every_replay_stream_in_the_corpus_is_valid`,
  `every_trap_file_in_the_corpus_is_valid`, `no_obs_id_is_shared_across_replay_streams` all green
  within the 107.
- **`scored()` / `failures()` stay 0** for the committed corpus at v0.1 — no answer producer exists
  (Epic 5). The family is discovered (12), parsed and validated, but scored by nothing.
- **Perimeter exact** (`git status --short`): only the two new fixtures, the `MANIFEST.toml` two-entry
  bump, the README section, the three-literal `trap_gate.rs` test bump, the story file and
  sprint-status. `Cargo.lock` untouched; the eleven existing manifest artefacts byte-for-byte
  unchanged (no diff).

### File List

- **New (locked):** `fixtures/scenario/replay/cloned-mac.jsonl` (3 observations, one byte-identical cloned `Mac`)
- **New (locked):** `fixtures/scenario/traps/cloned-mac.toml` (2 traps, `family = "cloned-mac"`, header carrying the AC3 record)
- **Modified:** `fixtures/MANIFEST.toml` (two `[[artefact]]` entries with sha256 — the deliberate bump, 11 → 13)
- **Modified:** `fixtures/scenario/traps/README.md` ("The schema is not a backstop" section — orphan-exempt, no manifest entry)
- **Modified:** `crates/opencmdb-bin/src/trap_gate.rs` (three `#[cfg(test)]` count literals 10 → 12, plus the two adjacent comments — tests only, no production logic)
- **Modified:** `_bmad-output/implementation-artifacts/sprint-status.yaml` (4.12 → in-progress → review)

## Change Log

| Date       | Change                                                                 |
|------------|------------------------------------------------------------------------|
| 2026-07-24 | Story 4.12 drafted (create-story): cloned/spoofed MAC — the inverse trap family (first family whose primary form guards the false MERGE). Two traps: must-not-merge/`l2-different-hostname` (REUSED from 4.11, coins nothing) judging identical-MAC/distinct-hostname, must-merge/`l1-exact-mac` judging identical-MAC/same-hostname/lease-moved. AC3: the no-CHECK/revisable-not-impossible record lands in the trap-file header (D18:1263-1265 + D21:1470-1473 verbatim) and a "schema is not a backstop" README passage; the evidence-free clone named as the untrappable residual fragility (prose, no register row). Committed count 10 → 12. Status → ready-for-dev. |
| 2026-07-24 | Validated (two fresh-context agents during create: fact-check + gap-hunt). Fact-check: every citation/line number, count, free value, rule spelling and scoring claim verified against sources — 3 defects (1 MED: JSONL shape exemplar wrongly named `randomized-mac.jsonl`, which carries no `Hostname`; 2 LOW: duplicate-id guard misattributed, D13 quote off by one line). Gap-hunt: 6 findings (1 MED: Task-2's D21 quote spelled differently from AC3's, re-hash churn risk; 5 LOW: prove-red precondition missing the stream, shape template, explicit `replay`/`observations` spellings, trailing-newline convention, missing scope-consistency trap). All 9 applied — template is now `shared-hardware-vm.jsonl` (W1) minus `Uplink`, D21 quote matches the source (":…same MAC. A UNIQUE…"), prove-red needs BOTH files on disk, both guards named exactly (trap.rs:316-330 / trap_gate.rs:234-251), citation :981-982, Trap #15 added. Reasons measured 274/229 chars (validators, independently). |
| 2026-07-24 | Implemented (dev-story): wrote `cloned-mac.jsonl` (3 obs, prefix `acacacac`, ONE byte-identical cloned Mac `02:00:5e:00:53:70`, hostnames doc-host-echo/doc-host-foxtrot as the discriminator, no Uplink) and `cloned-mac.toml` (2 traps: must-not-merge/`l2-different-hostname` REUSED, must-merge/`l1-exact-mac`; header carries D18's no-CHECK sentence verbatim + D21's no-UNIQUE renunciation); added the "schema is not a backstop" README section (evidence-free clone named, no register row); locked both in `MANIFEST.toml` (11 → 13); bumped `trap_gate.rs` committed counts 10 → 12 (count-coupling proved red at left:12/right:10 FIRST). Reasons 274/229 chars re-measured on the committed bytes. All gates green: fixtures 13 match / 0 orphan, fmt/clippy/test (107+86+42)/xtask ci clean, `Cargo.lock` + 11 existing artefacts untouched. Status → review. |
| 2026-07-24 | Code review (3 fresh-context layers: Blind Hunter / Edge Case Hunter / Acceptance Auditor). Auditor: clean PASS on all 8 AC + all 15 anti-mistakes, every Dev Agent Record claim reproduced under re-verification (hashes recomputed, reasons re-measured 274/229, tests re-run, D18/D21 quotes byte-compared). Edge: zero defects (every guard traced, both lock directions verified). Blind (diff-only): 11 findings, 10 dismissed by context (pairwise trap scoping is the format's documented semantics; `score(MustNotMerge, Abstained)=Pass` resolves the abstention question; `l1-exact-mac` pre-exists). Result: 1 LOW patch APPLIED (register's residual-fragility widened to name the PERFECT clone beside the evidence-free one — README orphan-exempt, no re-hash; gates re-run green), 1 defer (family streams never run through `FixtureConnector::load` admissibility — pre-existing since 4.9 → deferred-work.md), 12 dismissed. Status → done. |
