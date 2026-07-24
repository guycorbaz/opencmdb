# Story 4.13: Trap family — DHCP churn

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author of the trap corpus,
I want DHCP churn expressed purely as replayed timestamps,
so that **time-dependent behaviour is tested without the engine ever reading a clock** (D19): an
address that moves between two observations must neither split the host that kept its identity nor
fuse the two hosts that successively held it — and replaying the same fixture twice must yield
identical verdicts, because reproducibility is not the same as stability (D36).

## Acceptance Criteria

1. **Given** an address reassigned between two observations — the SAME `IpV4` fact bytes
   (`192.0.2.120`) observed on two hosts with DIFFERENT MACs and DIFFERENT hostnames, two hours
   apart — **when** the primary-form trap is scored, **then** it is a **`must-not-merge`** naming
   **`l1-distinct-mac`** (REUSED from `randomized-mac.toml`/`example.toml` — the rule that
   **OPPOSES**; it is the exemplar `Expectation`'s own doc names, trap.rs:69-71), never the
   IP-continuity temptation: a DHCP lease moves between devices as *normal operation*
   (architecture.md:1317 — real captures rot because *"DHCP moves"*), so a shared address must not
   drag identity with it. Fusing the recycled address's two holders is D10's catastrophic false
   merge (architecture.md:511-515).

2. **And** the inverse form — the SAME host (identical `Mac` bytes, SAME hostname) re-seen one hour
   later on a NEW address (`192.0.2.120` → `192.0.2.121`) — is a **`must-merge`** naming
   **`l1-exact-mac`** (the rule that FIRES, spelled as in `example.toml`/`randomized-mac.toml`):
   address churn must not teach the engine that a moved lease is a new device. This is D18's
   anti-cowardice middle column (architecture.md:1233) applied to THIS family — an engine that
   reacts to churn by splitting (or abstaining) on every address change is demolished here.

3. **And** the churn comes **entirely from `observed_at` values in the file** (epic AC1; D19,
   architecture.md:1293-1295: *"`observed_at` comes from the fixture -> the engine NEVER touches
   the clock (determinism = testability; DHCP churn is tested by replaying timestamps)"*): the
   stream's three instants are strictly increasing authored data, no fact carries a second time
   channel (deliberately **no `DhcpLease`** — its `expires_at` would be a rival clock; see Dev
   Notes), and a new committed-corpus test in `fixtures.rs` pins the shape as a second oracle
   (the house's deliberate-redundancy idiom): the two holders of `192.0.2.120` are separated by
   **nothing but time** — same `IpV4` bytes on N1 and N3, distinct `Mac`/`Hostname`, and the three
   `observed_at` equal to the three authored instants in order.

4. **And** replaying the same fixture twice yields identical verdicts (epic AC2; D36,
   architecture.md:2064-2067: *"we require REPRODUCIBILITY, not STABILITY. Replay
   `(data, capability)` -> the same verdict, always"*): a new test in `trap_gate.rs` scores the
   committed corpus **twice** with one identical `answers` map answering BOTH new traps correctly
   (`Merged { rule: l1-exact-mac }` / `Refused { rule: l1-distinct-mac }`) and asserts the two
   `Report`s are **equal** (`Report` derives `PartialEq, Eq`, trap_gate.rs:89) AND render to the
   same string (`Display` is deterministic by construction, trap_gate.rs:152-160), with
   `scored() == 2`, `failures() == 0` and `rule_mismatches` empty on both — so the equality
   compares real verdicts, not two vacuities. The test's doc comment names D36's lexical trap:
   the verdict may legitimately change when *capability* changes (that is 4.6c's snapshot
   comparability); the same `(corpus, answers)` may not.

5. **And** every expectation carries its mandatory **one-sentence `reason`** (D19;
   `Trap::validate`, 20–300 chars, single line): the `must-not-merge` reason names the recycled
   address and the opposing values (distinct MACs, hostnames `doc-host-golf` vs `doc-host-hotel`);
   the `must-merge` reason names the identical MAC, the same hostname and the address movement.
   Both claims are checkable against the committed bytes (example.toml's lesson: a reason claiming
   what the bytes contradict gets caught in review).

6. **And** both traps declare `family = "dhcp-churn"`, so the corpus-completeness check (story
   4.7b, `incomplete_families`) sees the family present in **BOTH decision poles** (≥1 `must-merge`
   AND ≥1 `must-not-merge`) and the gate stays green. Two traps, no `must-abstain` — a two-trap
   family exactly like 4.9/4.10/4.12.

7. **And** the family is committed as **two NEW locked corpus artefacts** —
   `fixtures/scenario/replay/dhcp-churn.jsonl` (the three presences it judges) and
   `fixtures/scenario/traps/dhcp-churn.toml` (the two traps) — **both added to
   `fixtures/MANIFEST.toml` with their sha256** in the same commit (a **deliberate corpus bump**,
   exactly like 4.9–4.12). After this story `cargo xtask ci`'s `fixtures` gate reports **15
   artefacts, all sha256 match, no orphan** (13 existing + 2 new). No README edit this time — the
   reality-debt register records cases reality produced and limits (4.8/4.12); this family adds
   neither.

8. **And** the committed replay stream carries **synthetic values only** — every MAC
   locally-administered (byte 0 = `0x02`, `locally_administered: true`), RFC 5737 documentation IPs
   (`192.0.2.0/24`), hostnames beginning `doc-` — so the privacy walk
   `the_corpus_carries_no_real_network_data` (`assert_facts_are_synthetic`, fixtures.rs:867-913,
   the `doc-` check at :879) passes over the new stream unchanged (D19;
   [[no-private-data-in-artifacts]]).

9. **And** the three committed-corpus assertions that hard-code the trap count are updated from
   **12 to 14** in the same change (`example.toml`'s 3 + `randomized-mac.toml`'s 2 +
   `multi-nic.toml`'s 2 + `shared-hardware-vm.toml`'s 3 + `cloned-mac.toml`'s 2 + this family's
   **2**): `report.discovered()` at `trap_gate.rs:391` and `:427`, the `"12 trap(s) discovered"`
   render string at `:409`, plus the two adjacent comments (`:387-390` corpus breakdown, `:419`
   "stays 12"). The scratch-corpus tests are **NOT** touched. At v0.2 the committed corpus is
   still scored by nothing (`scored()`/`failures()` stay `0` in the no-answers tests): the family
   is discovered, parsed and validated, but only AC4's test — which supplies its own answers —
   scores it.

## Tasks / Subtasks

> **⚠️ THIS FAMILY COINS NOTHING — read the "Rule vocabulary" Dev Note before writing the
> `.toml`.** The opposing rule is the REUSED `l1-distinct-mac` (the exemplar `Expectation`'s own
> doc comment names, trap.rs:69-71) and the firing rule is `l1-exact-mac`. There is NO IP rule id
> in the vocabulary and none is coined: the IP-continuity temptation is never named by any
> expectation — an expectation names what FIRES or what OPPOSES, never what tempts.

> **⚠️ ATDD ORDER — the two new tests are written FIRST and observed RED before the fixtures
> land** (both red naturally: AC3's test on the missing stream, AC4's on
> `AnswerForUnknownTrap`). Then the fixtures land and both green; then the count coupling is
> proved red at 12 and updated to 14. Mid-story, `cargo xtask ci`'s fixtures gate reds on the two
> not-yet-manifested files (orphans) — expected until Task 5.

- [x] **Task 1 — write the two new tests, observe both RED** (AC: 3, 4) — prove-to-red, house rule
  - [x] **AC3's byte-pin test** in `crates/opencmdb-bin/src/fixtures.rs`'s trailing test module,
        e.g. `the_dhcp_churn_stream_moves_the_address_only_through_observed_at`: read the stream
        via `read_jsonl(&fixture_path("scenario/replay/dhcp-churn.jsonl").unwrap())` (the stream
        carries no control records, so the 4.1 reader is the direct fit; the corpus walks will
        still pick it up through `read_records` unchanged). Assert: exactly **3** observations;
        N1 and N3 contain the **same** `Fact::IpV4` value (`192.0.2.120` — the recycled address,
        compare the parsed facts, not strings); N1 and N2 share identical `Mac` AND `Hostname`
        facts; N3's `Mac` and `Hostname` both differ from N1's; the three `observed_at` are
        **exactly** the three authored instants in strictly increasing order
        (`2026-01-06T00:00:00Z`, `T01:00:00Z`, `T02:00:00Z`) — compare via the module's existing
        `ts()` helper (fixtures.rs:711), do not hand-roll parsing. Doc comment: this restates the
        corpus bytes as a second independent oracle (the `expected()` idiom) and pins D19's
        sentence — the two holders of the recycled address are separated by nothing but time;
        nothing in the harness otherwise validates timestamps.
  - [x] **Placement: append each new test at the END of its module's test section** (below the
        last existing test), NOT beside the committed-corpus tests at trap_gate.rs:380-430 —
        inserting above them would shift every line number Task 4 and AC9 cite. All line numbers
        in this story are as-of-story-creation; in Task 4 the three test NAMES are the anchor,
        the numbers are a courtesy.
  - [x] **AC4's reproducibility test** in `crates/opencmdb-bin/src/trap_gate.rs`'s trailing test
        module, e.g. `replaying_the_same_corpus_twice_yields_identical_verdicts`: build ONE
        `BTreeMap` answering both new traps —
        `TrapId("dhcp-churn-must-merge")` → `Outcome::Merged { rule: RuleId("l1-exact-mac") }`,
        `TrapId("dhcp-churn-must-not-merge")` → `Outcome::Refused { rule: RuleId("l1-distinct-mac") }`
        — call `score_corpus(&committed_traps_root(), &answers)` **twice**, and assert:
        `first == second`; `first.to_string() == second.to_string()`; `first.scored() == 2`;
        `first.failures() == 0`; `first.rule_mismatches().is_empty()` (the `first == second`
        equality carries the three counts to `second`, satisfying AC4's "on both" — asserting
        them on both sides explicitly is equally fine). Doc comment quotes D36's
        reproducibility-not-stability distinction and says plainly that the answers map stands in
        for the engine until Epic 5.
  - [x] **Run both, record both RED in the Debug Log:** the fixtures do not exist yet, so AC3's
        test fails **inside `read_jsonl`** with `FixtureError::Io` naming the missing file —
        `fixture_path` SUCCEEDS on a missing file, it checks path shape, not existence
        (fixtures.rs:62-79) — and AC4's fails at the `score_corpus` expect with
        `AnswerForUnknownTrap` (trap_gate.rs:274-281 — an answer for a trap that does not exist
        is an error, not a silent no-op; this red also proves the two trap-id spellings are
        load-bearing). These reds are TEST-level: the equality legs (`first == second`, the
        `Display` comparison, AC3's fact equalities) are not individually observed red — the
        Debug Log must not claim per-assertion reds.

- [x] **Task 2 — write the replay stream** `fixtures/scenario/replay/dhcp-churn.jsonl` (AC: 1, 2, 3, 8) — pure data, no code
  - [x] Author **three observations** in one stream, JSONL (fields in order: `obs_id`,
        `connector_id`, `observed_at`, `scope` {`l2_domain`, `vantage`}, `facts`, `raw`; facts per
        line, in order: one `Mac`, one `IpV4`, one `Hostname` — no `Uplink`, no `DhcpLease`: time
        lives in `observed_at` alone). **Structural template: a line of `cloned-mac.jsonl`** —
        the exact same envelope and fact shapes. Use the **fresh `obs_id` prefix `adadadad`** —
        `aaaa…`/`bbbb…`/`cccc…`/`dddd…`/`eeee…`/`ffff…`/`abababab`/`acacacac` are all taken.
        **Before committing, `grep -r adadadad fixtures/ crates/` and confirm the only hits are
        the new `.jsonl` and its `.toml`.**
        - N1 = `adadadad-0000-4000-8000-000000000001`
        - N2 = `adadadad-0000-4000-8000-000000000002`
        - N3 = `adadadad-0000-4000-8000-000000000003`
  - [x] Keep `connector_id`, `l2_domain` and `vantage` **identical across all three** (the
        within-stream provenance/scope checks, 4.5a/4.5b): reuse the corpus's synthetic UUIDs —
        `connector_id` `33333333-3333-4333-8333-333333333333`, `l2_domain`
        `11111111-1111-4111-8111-111111111111`, `vantage` `22222222-2222-4222-8222-222222222222`.
  - [x] **N1 — the original holder of the address** (feeds BOTH traps):
        `{"Mac":{"addr":[2,0,94,0,83,120],"locally_administered":true}}` (`02:00:5e:00:53:78`,
        fresh), `{"IpV4":{"addr":"192.0.2.120"}}`,
        `{"Hostname":{"name":"doc-host-golf","source":"Dhcp"}}`, `observed_at`
        `2026-01-06T00:00:00Z`.
  - [x] **N2 — the same host after its lease moved** (feeds the `must-merge`, AC2): the SAME
        `Mac` bytes as N1 (copy-pasted), `{"IpV4":{"addr":"192.0.2.121"}}` (the new lease),
        `{"Hostname":{"name":"doc-host-golf","source":"Dhcp"}}` (SAME hostname), `observed_at`
        `2026-01-06T01:00:00Z`.
  - [x] **N3 — a different host now wearing the recycled address** (feeds the `must-not-merge`,
        AC1): `{"Mac":{"addr":[2,0,94,0,83,121],"locally_administered":true}}`
        (`02:00:5e:00:53:79`, fresh, DIFFERENT from N1's),
        `{"IpV4":{"addr":"192.0.2.120"}}` — **byte-identical to N1's `IpV4`: the recycled
        address is the point** (IP reuse within a stream is legal; only `obs_id`s are
        uniqueness-checked) — `{"Hostname":{"name":"doc-host-hotel","source":"Dhcp"}}`,
        `observed_at` `2026-01-06T02:00:00Z`.
  - [x] **`raw` is `null`** on every line. **Synthetic-only, non-negotiable (AC8):** every MAC
        locally-administered, every IP `192.0.2.x` (`.120`/`.121` are fresh — taken:
        .10/.11/.20/.21/.22/.30/.31/.32/.40/.41/.42/.80–.83/.112–.114), every hostname `doc-*`
        (`doc-host-golf`/`doc-host-hotel` are fresh — `doc-host-a/-c/-d/-echo/-foxtrot` and
        `doc-vm-alpha/-beta` are taken). MAC last bytes 120/121 are fresh in the `83,x` series
        (taken: 1,2,16,17,32,33,64,65,66,80,81,82,83,112).
  - [x] **End the file with a single trailing newline** (every committed artefact does) — settle
        this BEFORE Task 5 hashes the bytes; the same applies to Task 3's `.toml`.
  - [x] Re-run AC3's test: **green**. Record.

- [x] **Task 3 — write the family trap file** `fixtures/scenario/traps/dhcp-churn.toml` (AC: 1, 2, 5, 6) — pure data, no code
  - [x] Open with a header in the voice of the sibling families, carrying: *(a)* the family
        statement — DHCP churn is **time entering as data** (D19, architecture.md:1293-1295
        verbatim in spirit: *"`observed_at` comes from the fixture -> the engine NEVER touches the
        clock (determinism = testability; DHCP churn is tested by replaying timestamps)"*); an IP
        address is the one signal in this corpus that moves between devices as NORMAL operation
        (architecture.md:1317: real captures rot because *"DHCP moves"*; D12's L2 signal row
        names DHCP, architecture.md:891) — both traps below judge
        `scenario/replay/dhcp-churn.jsonl`, and together they pin: **an address is a lease, not
        an identity** — it neither fuses its successive holders nor splits the host that carried
        it. *(b)* the AC4 record — D36's lexical trap, quoted or tightly paraphrased from
        architecture.md:2064-2067 (*"we require REPRODUCIBILITY, not STABILITY. Replay
        `(data, capability)` -> the same verdict, always"*): this family's stream is the corpus's
        first whose **timestamps carry the assertion** (cloned-mac's story said the opposite of
        its own), and the reproducibility test beside it holds the harness to D36. Keep every id
        whole on its line (4.11's review lesson: a hyphen-broken id greps as absent).
  - [x] Both traps carry `replay = "scenario/replay/dhcp-churn.jsonl"` (the key the sibling files
        use), and their `observations` arrays spell the FULL UUIDs from Task 2 — a stream↔trap
        spelling mismatch reds `read_traps` with `DanglingObservation`.
  - [x] **The `must-not-merge` (primary form, AC1).** Judges `[N1, N3]` —
        `observations = ["adadadad-0000-4000-8000-000000000001", "adadadad-0000-4000-8000-000000000003"]`.
        `family = "dhcp-churn"`. `expect = { must-not-merge = { rule = "l1-distinct-mac" } }` —
        REUSED, spelled as in `randomized-mac.toml`. `reason` (one sentence, names the values):
        *"the address 192.0.2.120 that doc-host-golf held at 00:00 was recycled to a different
        box by 02:00 — the distinct MACs and the hostnames doc-host-golf vs doc-host-hotel oppose
        the merge the shared address tempts, because a DHCP lease moves between devices as normal
        operation."*
  - [x] **The `must-merge` (inverse form, AC2).** Judges `[N1, N2]` —
        `observations = ["adadadad-0000-4000-8000-000000000001", "adadadad-0000-4000-8000-000000000002"]`.
        `family = "dhcp-churn"`. `expect = { must-merge = { rule = "l1-exact-mac" } }` — spelled
        as in `example.toml`/`randomized-mac.toml`. `reason`: *"both presences carry the identical
        MAC 02:00:5e:00:53:78 and the hostname doc-host-golf one hour apart, so only the leased
        address moved from 192.0.2.120 to 192.0.2.121 — an address that churns is a lease doing
        its job, not a new device."*
  - [x] **Measure both reasons** (`wc -m`, 20–300 bound, single line, no control chars) and record
        the counts in the Debug Log (as prescribed they measure 275 and 237 — re-measure the
        committed bytes). If one exceeds 300, tighten it — never split it into two sentences.
  - [x] Key order and array layout inside a `[[trap]]` are free (the bytes are hashed fresh, no
        sibling is affected) — but the sibling multi-line `observations` layout
        (randomized-mac.toml) is the house voice; prefer it over the single-line arrays this
        story's prose uses for compactness.
  - [x] **Trap ids unique across the corpus:** `dhcp-churn-must-not-merge`,
        `dhcp-churn-must-merge` (sibling idiom `<family>-must-*`; no clash. Two guards:
        `TrapFile::validate` reds a case-folded duplicate WITHIN one file, trap.rs:315-330;
        `score_corpus` reds an exact-match duplicate ACROSS files, trap_gate.rs:245-251
        `DuplicateTrapId`).
  - [x] Re-run AC4's test: **green** (both answers now land on discovered traps; scored 2,
        failures 0, the two reports equal). Record.

- [x] **Task 4 — prove the count coupling red, then update 12 → 14** `crates/opencmdb-bin/src/trap_gate.rs` (AC: 9) — three edits, tests only
  - [x] **With BOTH new files on disk and the assertions still at `12`**, run
        `cargo test -p opencmdb-bin --locked the_committed_corpus_is_discovered_and_scored_by_nothing`:
        it must RED with `left: 14, right: 12` — proving the two new traps parsed, validated, and
        resolved their obs_ids against the committed stream. Record both runs in the Debug Log.
  - [x] `the_committed_corpus_is_discovered_and_scored_by_nothing` (`:391`): `12` → `14`; keep
        `scored()`/`failures()` at `0`. Update the breakdown comment at `:387-390` to add
        dhcp-churn's **two** and say **fourteen**.
  - [x] `the_report_says_plainly_that_nothing_was_scored` (`:409`): `"12 trap(s) discovered"` →
        `"14 trap(s) discovered"`. Leave `"0 scored"` and `"0 truth-table failure(s)"` unchanged.
  - [x] `a_trap_with_no_answer_is_discovered_but_not_scored` (`:427`): `12` → `14`, and the
        comment at `:419` "stays 12" → "stays 14" (`scored()` stays `1` — it still answers only
        `example-must-abstain`).
  - [x] **Do NOT touch the scratch-corpus tests** (`each_column_can_be_driven_red`,
        `a_one_sided_family_reddens_the_gate_on_its_own`, …) — they build their own temp dirs;
        their counts are unrelated.

- [x] **Task 5 — lock the two new artefacts** `fixtures/MANIFEST.toml` (AC: 7) — deliberate corpus bump
  - [x] Append TWO `[[artefact]]` entries (paths `scenario/replay/dhcp-churn.jsonl` and
        `scenario/traps/dhcp-churn.toml`), each with a one-line comment naming the story and what
        the artefact is, mirroring the 4.9–4.12 entries. Compute each `sha256` from the committed
        bytes (`sha256sum <file>`) and paste the hex. **Finish every byte of both files FIRST
        (header prose included, wrap-check done — ids whole on their line), THEN hash** — a later
        edit, even a trailing newline, invalidates the sha256.
  - [x] Do NOT touch the thirteen existing entries or their sha256 — any edit there is an
        unrelated EDITED finding.

- [x] **Task 6 — gates and the corpus lock** (AC: 6, 7, 8, 9)
  - [x] `cargo fmt --all` · `cargo clippy --workspace --all-targets --locked -- -D warnings` ·
        `cargo test --workspace --locked` · `cargo run -p xtask --locked -- ci` — all green.
  - [x] `fixtures` gate reports the new total — the gate's actual wording is
        `"15 fixture(s) match their recorded sha256 (0 generated, 15 hand-authored)"`
        (xtask/src/main.rs:532; quote the real message in the Debug Log, not a paraphrase) — with
        no orphan (13 existing + 2 new; the new entries need no `generator` flag — absent means
        hand-authored). Confirm the new stream and trap file are BOTH listed and BOTH match.
  - [x] Confirm the committed corpus still `passed()`: `discovered() == 14`, `scored() == 0` in
        the no-answers tests, `failures() == 0`, `rule_mismatches` empty, `incomplete_families`
        empty (dhcp-churn carries both poles). Grep `"12 trap"` / `discovered(), 12` /
        `"stays 12"` / `twelve` across `crates/` to catch any count assertion this story missed.
  - [x] Privacy + validity walks pass over the new files automatically
        (`the_corpus_carries_no_real_network_data`, `every_replay_stream_in_the_corpus_is_valid`,
        `every_trap_file_in_the_corpus_is_valid`, `no_obs_id_is_shared_across_replay_streams`) —
        no harness change.
  - [x] `Cargo.lock` unchanged (two data files, one manifest bump, three test literals, two new
        tests). `architecture-views.md` NOT regenerated (`ℹ views-hash STALE` is by design —
        Epic-4 milestone, not here). The thirteen existing artefacts byte-for-byte unchanged
        (`git status --short`). `fixtures/scenario/traps/README.md` untouched (no register case,
        no limit statement this time).

### Review Findings

- [x] [Review][Patch] The AC4 test's doc comment overclaims: it says the test "reds if a future
      change trades away" sorted discovery — false; within one process the two walks see the same
      readdir order sorted or not, so `first == second` cannot red on a sort removal. Weaken to
      the true sentence (it DOES red on e.g. per-instance `HashMap` iteration order or an
      ambient-time read) [crates/opencmdb-bin/src/trap_gate.rs:1030-1035]
- [x] [Review][Patch] N2's post-move address is pinned nowhere — `ip(1)` is never asserted, yet
      "the lease moved to `.121`" is the value the must-merge premise depends on; a re-authored
      stream where N2 keeps `.120` would stay green. Assert `ip(1)` = `192.0.2.121`
      [crates/opencmdb-bin/src/fixtures.rs:1765-1775]
- [x] [Review][Patch] "shares ONLY the IpV4 bytes" is asserted by comment, not by code: no
      `facts.len()` assertion and `find()` takes the first match, so a duplicated or extra fact
      on any line passes silently. Assert each observation carries exactly 3 facts
      [crates/opencmdb-bin/src/fixtures.rs:1750-1762]
- [x] [Review][Defer] MAC/hostname values are pinned relationally only — the constants the two
      `reason` strings cite (`02:00:5e:00:53:78`, `doc-host-golf`, `doc-host-hotel`) are asserted
      by no test — deferred, pre-existing class: exact-value/round-trip pinning covers only
      `minimal.jsonl` (registered in deferred-work.md since story 4.10; AC3's own text prescribes
      the relational form)

## Dev Notes

### The shape of this story in one paragraph

The DHCP-churn family — **time enters as data** (D19). Every prior family's timestamps were
scenery (4.12 said so explicitly: *"the timestamp carries no assertion, the hostname does"*);
here they ARE the trap: the same address `192.0.2.120` observed at 00:00 and at 02:00 belongs to
two different boxes, and the ONLY thing in the bytes that says "recycled lease" rather than "IP
conflict in one scan" is `observed_at`. Like 4.9–4.12 it is almost entirely DATA: two new
committed artefacts (a **three**-observation stream, a **two**-trap family file), a deliberate
`MANIFEST.toml` bump, three test-literal updates (**12 → 14**) — plus, unlike its siblings, **two
new tests**: AC3's byte-pin (the churn lives in `observed_at` alone) and AC4's reproducibility
test (score the corpus twice, get the identical `Report` — D36). No engine, no new harness code,
no new rule id: `l1-distinct-mac` and `l1-exact-mac` are both REUSED. What is genuinely NEW: the
first family whose timestamps carry the assertion, and the first story that pins D36's
reproducibility-not-stability at the harness level.

### Rule vocabulary — this family coins NOTHING (like 4.12, and why)

**Read this before writing the `.toml`.** The two rules this family needs both exist:

- The `must-not-merge` names the rule that **OPPOSES** (trap.rs:66-73: *"a refusal without a
  named rule cannot be told apart from an engine that simply found nothing"*; example.toml's
  header says the same in its own words). The opposing
  signal here is the **distinct MACs** — and `l1-distinct-mac` is literally the exemplar the
  `Expectation` doc comment names (trap.rs:69-71: *"naming the rule that OPPOSES the merge
  (`l1-distinct-mac`, not the rule that was tempting)"*). Spelled as in `randomized-mac.toml`.
- The `must-merge` names the rule that **FIRES**: `l1-exact-mac`, the original
  (`example.toml`/`randomized-mac.toml`/`cloned-mac.toml`).
- **The temptation is IP-continuity ("same address → same device") and it is never named:** no
  expectation names the tempting rule, so no IP rule id is needed — and none exists. Coining an
  `l2-same-ip`/`l1-ip-continuity` here would enter a rule into the vocabulary that NO expectation
  in this family (or any planned family) will ever cite, purely to describe a mistake. If Epic 5
  builds an IP-history rule, it will be named there, against these traps. Raise in review if
  disputed — do not silently coin.
- **Why `l1-distinct-mac` and not `l2-different-hostname`:** both oppose in the bytes, but the
  distinct MAC is L1's own decisive signal (D12, architecture.md:890) and the precedent is
  randomized-mac's must-not-merge; the hostname difference is corroborating scenery here (it keeps
  the trap's oracle honest — see the next note). If a reviewer prefers the hostname rule, raise it
  in review — the observations carry both signals either way.

### Why N3 carries a different hostname too — the oracle must be reachable

The lesson of 4.12's evidence-free clone applies in mirror: a `must-not-merge` whose two
observations differ ONLY in `observed_at` (same IP, same MAC absent, hostname absent) would demand
the engine detect a recycling with zero discriminating evidence. Here the bytes carry the
discriminators (distinct `Mac`, distinct `Hostname`); what the timestamps add is the *coherence*
of the story — at any instant each address has one holder, and the reassignment happens BETWEEN
observations (epic AC1's exact wording). Drop N3's hostname or give it `doc-host-golf` and the
trap collapses into a different family (a clone variant), not this one.

### The two poles, and which observations feed which (AC1/AC2)

| Pole | Rule | Judges | The point |
|---|---|---|---|
| **`must-not-merge`** (primary) | `l1-distinct-mac` (REUSED — the rule that OPPOSES) | `[N1, N3]` — same `IpV4` `192.0.2.120`, distinct MACs, hostnames `doc-host-golf` vs `doc-host-hotel`, two hours apart | the recycled lease: an address is a lease, not an identity — fusing its successive holders is D10's catastrophic false merge |
| **`must-merge`** (inverse) | `l1-exact-mac` (the rule that FIRES) | `[N1, N2]` — identical MAC, SAME hostname, address moved `.120` → `.121` one hour later | anti-cowardice: a moved lease is not a new device (D18's middle column) |

N1 feeds both traps — the established idiom (4.9/4.11/4.12 all share their first observation
across poles). Note the family's kinship with cloned-mac's `must-merge` (identical MAC, lease
moved): the byte-shape is similar, the FAMILY is different — there the identical MAC was the
tested temptation (fear of clones), here the moving address is (fear of churn). Likewise the
`must-not-merge` is byte-kin to randomized-mac's (two distinct locally-administered MACs, same
rule `l1-distinct-mac`) — there the trap pinned that L1 refuses even when the two MACs are one
re-randomized laptop, here the SHARED ADDRESS is the tested temptation, and what
randomized-mac's stream does not carry is exactly what this one adds: the byte-identical recycled
`IpV4` and the two hostnames (randomized-mac.jsonl has neither a shared IP nor any `Hostname`). The corpus tests temptations, not byte-shapes, and each family's
header says which.

### Reproducibility is not stability (D36) — what AC4's test does and does not claim

D36 (architecture.md:2057-2077): *"Same data, different capability, different verdict: CORRECT."*
The lexical trap it names: requiring **stability** (the verdict never changes) would mean pinning
capability — reintroducing the false merge; what is required is **reproducibility** — replay the
same `(data, capability)` and get the same verdict, always. At v0.2 no engine exists, so the test
renders the claim at the harness level: the same `(corpus, answers)` scored twice yields one
identical `Report` (equality via `PartialEq`, trap_gate.rs:89; rendering via the deliberately
deterministic `Display`, trap_gate.rs:152-160; discovery is sorted, trap_gate.rs:354, and the
answers map is a `BTreeMap` — the determinism is designed, and this test is what reds if a future
change trades it away, e.g. a `HashMap` swap or an ambient-time read). The answers map stands in
for the engine; when Epic 5 lands, the same fixture through the same engine closes the loop.
`polling_twice_replays_the_same_stream` (fixture_connector.rs:1025) already pins the
connector-level half on `minimal.jsonl`; AC4's test adds the verdict-level half over the
committed corpus. **Do not claim more than this in the completion record** — the test proves
harness determinism, not engine determinism (no engine exists to prove).
[[claims-must-match-verification]]

### Why there is deliberately NO `DhcpLease` fact in this stream

`Fact::DhcpLease { ip, expires_at }` exists (observation/mod.rs:159-163) and the privacy walk
already covers it (fixtures.rs:875) — this is a scope decision, not a gap dodge. Three reasons:
*(a)* epic AC1 says the churn comes **entirely from `observed_at`** — an `expires_at` would be a
second, rival time channel inside the facts, diluting exactly what this family pins; *(b)* no rule
consumes leases and no corpus stream has ever carried one — introducing the variant's first use
inside a trap family would make this family also, silently, the DhcpLease format fixture (that is
4.18's wire-format territory, or Epic 11's); *(c)* the three `IpV4` facts already say everything
the traps judge. If a reviewer wants a lease-bearing variant, that is a corpus EXTENSION to raise
in review — not a change to this family.

### How a trap is validated — the checks the new files must survive

Identical to 4.11/4.12 (nothing changed since): `Trap::validate` (trap.rs:249-313) enforces
reason bounds (20–300 chars, single line, no control char), a rule on every decision, non-empty
`replay`/`observations`, clean family tokens; obs_id resolution against the named `replay`
stream (`DanglingObservation`) lives in `read_traps` itself (fixtures.rs:652-690, raised at
:687), NOT in trap.rs; trap ids are unique case-folded within a file (`TrapFile::validate`,
trap.rs:315-333) and exact-match across files (trap_gate.rs:245-251);
`incomplete_families` requires both poles per family.
The corpus walks (`the_corpus_carries_no_real_network_data`,
`every_trap_file_in_the_corpus_is_valid`, `every_replay_stream_in_the_corpus_is_valid`,
`no_obs_id_is_shared_across_replay_streams`) pick the new files up automatically — no harness
change; a malformed file reds the walk by name. Timestamps are NOT validated anywhere in the
harness — their strict increase in this stream is authoring discipline, pinned only by AC3's test.

### The corpus lock is a DELIBERATE bump (AC7) — same mechanism as 4.9–4.12

Two real artefacts locked in both directions (EDITED and ORPHAN both red). Order of operations:
**finish every byte of both files (header prose included, wrap-check done), THEN hash, THEN
paste.** Mid-story, between the files landing and Task 5, `cargo xtask ci`'s fixtures gate reds
with two orphans — expected, resolved by the manifest bump in the same commit.

### Previous story intelligence (4.12)

- **Review outcome:** Auditor PASS 8/8, one LOW patch (a register sentence widened), one defer.
  The defer is inherited context for THIS story: **family replay streams are never exercised
  through `FixtureConnector::load`'s admissibility checks** — only `minimal.jsonl` is; the corpus
  walks gate parseability/validity, not connector-level admissibility. Pre-existing since 4.9,
  recorded in `deferred-work.md`. The dhcp-churn stream inherits the same limit — do NOT fix it
  here (it is a registered defer, not this story's scope).
- **Hash discipline held:** 4.12 hashed only after the final byte (4.11 had re-hashed once after
  a header rewrap). Same order here; do the wrap-check (ids whole on their line) BEFORE hashing.
- **Values verified free before authoring** (grep for the prefix, the hostnames, the MAC bytes) —
  repeated here with `adadadad`/`doc-host-golf`/`doc-host-hotel`/`83,120`/`83,121`/`.120`/`.121`
  (all confirmed free at story-creation time; re-verify at dev time, the corpus may have moved).
- **PR workflow:** branch per story → PR → CI green → squash merge (PRs #21–#27 are the
  pattern; never push straight to master). [[opencmdb-pr-workflow]]

### Project Structure Notes

- **NEW (locked):** `fixtures/scenario/replay/dhcp-churn.jsonl` (3 observations, the recycled
  address `192.0.2.120` on N1 and N3), `fixtures/scenario/traps/dhcp-churn.toml` (2 traps,
  `family = "dhcp-churn"`, header carrying D19's replayed-timestamps sentence and D36's
  reproducibility record). Both listed in `fixtures/MANIFEST.toml` with sha256.
- **Updated:** `fixtures/MANIFEST.toml` (two `[[artefact]]` entries — the deliberate bump,
  13 → 15); `crates/opencmdb-bin/src/trap_gate.rs` (three `#[cfg(test)]` count literals 12 → 14
  + the `:387-390`/`:419` comments, PLUS the new reproducibility test — tests only, no production
  logic); `crates/opencmdb-bin/src/fixtures.rs` (the new byte-pin test in the trailing test
  module — tests only).
- **Unchanged, expected:** the thirteen existing manifest entries (byte-for-byte);
  `fixtures/scenario/traps/README.md` (no register case, no limit statement — first family since
  4.9 that touches neither); `trap.rs` / `score.rs` / `gap/mod.rs` / `fixture_connector.rs`
  production paths; `Cargo.lock`; every other `.rs`. `discover_trap_files` / `read_traps` /
  `incomplete_families` / `score_corpus` are used AS-IS.
- **Out of scope, deliberately:** any engine or rule producer (Epic 5 owns the real
  `l1-distinct-mac`/`l1-exact-mac` rules and any future IP-history rule); a `DhcpLease`-bearing
  stream (4.18 wire-format / Epic 11 territory); the VRRP/HSRP family (4.14 — next); fixing the
  `FixtureConnector::load` admissibility defer (registered in `deferred-work.md`, pre-existing);
  `docs/project-context.md` / `CLAUDE.md` (fold in at the Epic-4 milestone with the
  `architecture-views.md` regeneration).

### Traps (mistakes this story must not make)

1. **Coining a rule id.** `l1-distinct-mac` and `l1-exact-mac` both exist; the IP-continuity
   temptation is never named by any expectation. (Documented in the Rule-vocabulary note.)
2. **Naming the tempting signal in the `must-not-merge`.** The expectation names what OPPOSES
   (`l1-distinct-mac`) — trap.rs:66-73, example.toml's contract.
3. **Giving N1 and N3 the same `observed_at`.** Then the bytes say "one address, two holders, one
   instant" — an IP conflict in a single scan, not a churn; the family's premise (reassigned
   BETWEEN two observations) dies with it. The three instants are strictly increasing.
4. **Giving N3 the hostname `doc-host-golf` or N1's MAC.** Either collapses the `must-not-merge`
   into a clone/re-sighting shape — a different family. N3 differs from N1 in BOTH `Mac` and
   `Hostname`; it shares ONLY the `IpV4` bytes.
5. **Giving N2 a different hostname or MAC.** N2 is the SAME host re-seen; only its `IpV4`
   changed. A differing MAC turns the must-merge pair into randomized-mac.
6. **Adding a `DhcpLease` fact** (or any second time channel). The churn lives in `observed_at`
   alone — see the dedicated Dev Note.
7. **A hostname not prefixed `doc-`**, a non-5737 IP, or a non-locally-administered MAC — reds
   the privacy walk (fixtures.rs:867-913).
8. **Reusing an existing stream's `obs_id` prefix.** Fresh prefix `adadadad`; grep before
   committing.
9. **Forgetting the manifest bump, or hashing before the final byte** (including the header
   wrap-check — ids whole on their line, 4.11's review lesson).
10. **Leaving the count assertions at 12 — or updating them without the red run.** This family
    adds TWO: 12 → 14 at `trap_gate.rs:391/:409/:427` + comments `:387-390`/`:419`. Scratch tests
    untouched. Prove red first, record it.
11. **Answering only ONE trap in the reproducibility test.** With one answer, `scored() == 1` and
    the equality compares a half-vacuity; the test answers BOTH new traps and asserts
    `scored() == 2`.
12. **Wrong `Outcome` shapes in the test.** The `must-not-merge` answer is
    `Refused { rule: l1-distinct-mac }` (a refusal names a rule); `Abstained` also passes the
    column (score.rs:183) but carries a cause instead of a rule and would weaken the
    rule-comparison leg (`run_trap` compares rules only when both sides carry one,
    score.rs:228+). Use `Refused` with the matching rule.
13. **Skipping the natural REDs.** AC3's test reds on the missing stream (inside `read_jsonl`,
    `FixtureError::Io` — NOT in `fixture_path`, which succeeds on a missing file); AC4's reds
    with `AnswerForUnknownTrap` before the `.toml` lands (trap_gate.rs:274-281). Both are the
    house prove-to-red, free of charge — run them BEFORE the fixtures land and record both.
    The reds are TEST-level: do not claim per-assertion reds for the equality legs.
14. **Touching `fixtures/scenario/traps/README.md`.** No register case, no limit statement this
    time — the register takes what reality produced (4.8) and limits (4.12); a routine family
    adds neither.
15. **Regenerating `architecture-views.md`.** Epic-4 milestone, not this story.
16. **Claiming more than measured.** The reproducibility test proves HARNESS determinism, not
    engine determinism; say so. Name the command behind every count.
    [[claims-must-match-verification]]
17. **Inconsistent `connector_id`/`l2_domain`/`vantage` within the stream.** All three
    observations share the one `connector_id` and scope prescribed in Task 2 (the within-stream
    provenance/scope checks, 4.5a/4.5b).

### Latest technical specifics

No new crate, no version bump, no production code. Rust 1.96+, edition 2024. Two data files, one
`MANIFEST.toml` bump, three `#[cfg(test)]` literal updates and two new `#[cfg(test)]` tests in
`opencmdb-bin`. **Never invent a version — pin from the committed `Cargo.lock`, which does not
move here.**

### References

- [Source: _bmad-output/planning-artifacts/epics.md:1149-1161 — Story 4.13: the story sentence
  ("DHCP churn expressed purely as replayed timestamps… without the engine ever reading a clock")
  and the two ACs (churn entirely from `observed_at`, D19; replaying twice yields identical
  verdicts — reproducibility, not stability, D36)]
- [Source: _bmad-output/planning-artifacts/architecture.md:1291-1295 — D19: "`observed_at` comes
  from the fixture -> the engine NEVER touches the clock (determinism = testability; DHCP churn is
  tested by replaying timestamps)"; the engine is a pure function]
- [Source: _bmad-output/planning-artifacts/architecture.md:2057-2077 — D36: reproducibility vs
  stability ("Replay `(data, capability)` -> the same verdict, always. The verdict is allowed to
  change over time. Requiring stability means pinning capability"); the snapshot corollary 4.6c
  already implements]
- [Source: _bmad-output/planning-artifacts/architecture.md:509-523 — D10: false-merge
  catastrophic and asymmetric; the adversarial matrix naming "DHCP churn" at :522]
- [Source: _bmad-output/planning-artifacts/architecture.md:884-895 — D12: the L1/L2 table (L2's
  signal row includes DHCP, :891); "A MAC identifies an INTERFACE, not a device"]
- [Source: _bmad-output/planning-artifacts/architecture.md:1312-1317 — real captures rot: "DHCP
  moves, devices leave" — the churn this family freezes into replayable bytes]
- [Source: _bmad-output/planning-artifacts/architecture.md:244 — composite identity depends on
  IP/DHCP history, which accumulates (the Growth-side reason an IP is history, not identity)]
- [Source: crates/opencmdb-core/src/trap.rs:55-88, 249-333 — `Expectation` (the must-not-merge
  doc naming `l1-distinct-mac` as the opposing exemplar at :69-71), `Trap::validate` (reason
  bounds; obs_id resolution is NOT here — it is `read_traps`', fixtures.rs:652-690),
  `TrapFile::validate` (case-folded within-file id uniqueness, :315-333)]
- [Source: crates/opencmdb-core/src/score.rs:175-187, 228+ — the nine cells
  (`(MustNotMerge, Refused) = Pass` at :182); `run_trap` compares rules only when both sides
  carry one]
- [Source: crates/opencmdb-bin/src/trap_gate.rs:89, 152-160, 245-251, 274-281, 354, 383-430 —
  `Report` derives `PartialEq, Eq`; deterministic `Display`; `DuplicateTrapId`;
  `AnswerForUnknownTrap` (the natural red for AC4's test); sorted discovery; the three
  committed-count assertions (12 → 14) and their comments]
- [Source: crates/opencmdb-bin/src/fixtures.rs:640, 652-690, 867-913, 821, 1306, 1327, 1424 —
  `read_jsonl` (the pure-observation reader AC3's test uses); `read_traps`' obs_id resolution
  (`DanglingObservation` at :687); `assert_facts_are_synthetic` (the `doc-` check at :879,
  `DhcpLease` covered at :875); the four corpus walks that pick the new files up automatically
  (privacy :821, cross-stream obs_id :1306, stream validity :1327, trap-file validity :1424)]
- [Source: crates/opencmdb-bin/src/fixture_connector.rs:282, 1021-1049 —
  "the engine never touches the clock (D19)"; `polling_twice_replays_the_same_stream` (the
  connector-level reproducibility half that already exists, on `minimal.jsonl`)]
- [Source: crates/opencmdb-core/src/observation/mod.rs:20, 140-173, 238-243 — `Timestamp` =
  `chrono::DateTime<Utc>`; `Fact` (`DhcpLease` at :159-163 — deliberately NOT used);
  `Observation` derives `PartialEq` (equality AC3's test rests on); `observed_at` at :243]
- [Source: fixtures/scenario/traps/randomized-mac.toml — the two-trap family voice and BOTH rule
  spellings this family reuses (`l1-distinct-mac` must-not-merge, `l1-exact-mac` must-merge)]
- [Source: fixtures/scenario/replay/cloned-mac.jsonl — the structural template (Mac + IpV4 +
  Hostname envelope, no Uplink) and the sibling whose timestamps carried NO assertion — the
  contrast this family's header names]
- [Source: _bmad-output/implementation-artifacts/4-12-trap-cloned-mac.md — the immediately prior
  family: structural model (fresh obs_id prefix, manifest bump, count update,
  hash-after-final-byte), the review defer this stream inherits
  (`FixtureConnector::load` admissibility — pre-existing, do not fix here), and the
  "timestamp carries no assertion" sentence this family inverts]

## Dev Agent Record

### Agent Model Used

claude-fable-5 (Claude Fable 5)

### Debug Log References

- **AC3 natural RED** (test-level, before the fixtures existed):
  `cargo test -p opencmdb-bin --locked the_dhcp_churn_stream_moves_the_address_only_through_observed_at`
  → panicked at the test's `expect` with `FixtureError::Io { path: ".../fixtures/scenario/replay/dhcp-churn.jsonl",
  source: Os { code: 2, kind: NotFound } }` — the failure came from inside `read_jsonl`, as
  predicted (`fixture_path` succeeded on the missing file). No per-assertion reds are claimed for
  the equality legs.
- **AC4 natural RED** (test-level, before the `.toml` landed): same command filtered on
  `replaying_the_same_corpus_twice_yields_identical_verdicts` → panicked at the first
  `score_corpus` expect with `AnswerForUnknownTrap { trap: "dhcp-churn-must-merge", count: 2 }` —
  proving both trap-id spellings in the answers map are load-bearing.
- **Count-coupling RED**: with BOTH new fixture files on disk and the assertions still at 12,
  `cargo test -p opencmdb-bin --locked the_committed_corpus_is_discovered_and_scored_by_nothing`
  → `assertion 'left == right' failed: the walk must open the corpus / left: 14 / right: 12` —
  the two new traps parsed, validated and resolved their obs_ids. Then updated 12 → 14
  (trap_gate.rs `:391`/`:409`/`:427` + the two comments) and the test greens.
- **Reason lengths** (`wc -m` on the committed bytes, single line each): must-not-merge **275**,
  must-merge **237** — both inside `Trap::validate`'s 20–300 bound, matching the story's
  pre-measured values.
- **`adadadad` grep**: `grep -rln adadadad fixtures/ crates/` → only
  `fixtures/scenario/replay/dhcp-churn.jsonl` and `fixtures/scenario/traps/dhcp-churn.toml`.
  Pre-authoring frees confirmed the same way for `doc-host-golf`/`doc-host-hotel`, MAC bytes
  `83,120`/`83,121` and IPs `.120`/`.121` (no hits anywhere).
- **Hash-after-final-byte order held**: both files finished first (trailing newline confirmed with
  `tail -c 1 | xxd` → `0a`; wrap-check done — every id whole on its line), THEN `sha256sum`:
  `ec4d1d1bf499a8120d661cb3cc42d0f3b2b7fe5575925e7376e84d1a434a8720` (dhcp-churn.jsonl),
  `ce43dfb9abb052d58aad5b52b5f2c1e43e31fb08d1d54a1260e103716e287314` (dhcp-churn.toml). No edit
  after hashing.
- **Gates** (all green): `cargo fmt --all` · `cargo clippy --workspace --all-targets --locked --
  -D warnings` · `cargo test --workspace --locked` → **109 (bin) + 86 (core) + 42 (xtask) passed,
  0 failed** · `cargo run -p xtask --locked -- ci` → fixtures gate reports verbatim
  **"15 fixture(s) match their recorded sha256 (0 generated, 15 hand-authored)"**, no orphan;
  `views-hash` still `ℹ STALE` by design. Residual-count grep
  (`"12 trap"` / `discovered(), 12` / `stays 12` / `twelve` across `crates/`) → no hits.

### Completion Notes List

- The DHCP-churn family landed as pure data + tests, exactly as scoped: two NEW locked corpus
  artefacts (`dhcp-churn.jsonl`, 3 observations around the recycled address `192.0.2.120`;
  `dhcp-churn.toml`, 2 traps), a deliberate `MANIFEST.toml` bump (13 → 15), three test-literal
  updates 12 → 14, and TWO new tests — no production code, no new rule id, no `DhcpLease`.
- **AC1/AC2**: `dhcp-churn-must-not-merge` judges `[N1, N3]` naming the REUSED `l1-distinct-mac`
  (the rule that OPPOSES); `dhcp-churn-must-merge` judges `[N1, N2]` naming `l1-exact-mac` (the
  rule that FIRES). The IP-continuity temptation is never named — nothing was coined.
- **AC3**: the churn comes entirely from `observed_at` — pinned by the new byte-pin test
  `the_dhcp_churn_stream_moves_the_address_only_through_observed_at` (fixtures.rs, end of the test
  module), the second independent oracle in the `expected()` idiom: same `IpV4` bytes on N1/N3,
  identical `Mac`+`Hostname` on N1/N2, distinct on N3, and the three instants exactly
  `2026-01-06T00:00:00Z`/`T01:00:00Z`/`T02:00:00Z` via the module's `ts()` helper.
- **AC4**: `replaying_the_same_corpus_twice_yields_identical_verdicts` (trap_gate.rs, end of the
  test module) scores the committed corpus twice with one `BTreeMap` answering BOTH new traps and
  asserts `first == second`, equal `Display` renderings, `scored() == 2`, `failures() == 0`,
  `rule_mismatches` empty. **This proves HARNESS determinism only** (sorted discovery, `BTreeMap`
  answers, deterministic `Display`) — no engine exists to prove; the answers map stands in until
  Epic 5. The doc comment names D36's lexical trap (capability changes may change the verdict;
  the same `(corpus, answers)` may not).
- **AC5**: both reasons name their checkable values (recycled address, distinct MACs/hostnames;
  identical MAC, same hostname, `.120` → `.121`), measured 275/237 chars.
- **AC6**: `family = "dhcp-churn"` on both traps, both poles present — `incomplete_families`
  stays empty, asserted by the existing `passed_is_the_failures_gate_with_a_discovered_floor`
  over the committed corpus (green in the full suite).
- **AC7**: both artefacts locked in `MANIFEST.toml`; the fixtures gate reports 15/15 matching,
  no orphan. The thirteen existing entries are byte-for-byte untouched (`git status --short`
  shows only the expected files).
- **AC8**: synthetic-only bytes (locally-administered MACs `02:00:5e:00:53:78`/`:79`, RFC 5737
  IPs, `doc-` hostnames); the privacy walk passed over the new stream unchanged in the full suite.
- **AC9**: the three committed-count assertions updated 12 → 14 AFTER the recorded red run;
  scratch-corpus tests untouched; `scored()` stays 0 in the no-answers tests.
- Out of scope honoured: no README edit, no `architecture-views.md` regeneration, no
  `FixtureConnector::load` admissibility fix (registered defer), `Cargo.lock` unchanged.

### File List

- `fixtures/scenario/replay/dhcp-churn.jsonl` — NEW: 3-observation replay stream
- `fixtures/scenario/traps/dhcp-churn.toml` — NEW: 2-trap family file
- `fixtures/MANIFEST.toml` — modified: two `[[artefact]]` entries appended (13 → 15)
- `crates/opencmdb-bin/src/fixtures.rs` — modified: AC3 byte-pin test appended to the trailing
  test module (tests only)
- `crates/opencmdb-bin/src/trap_gate.rs` — modified: AC4 reproducibility test appended to the
  trailing test module; three count literals 12 → 14 and two adjacent comments (tests only)
- `_bmad-output/implementation-artifacts/4-13-trap-dhcp-churn.md` — this story file
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status tracking
- `_bmad-output/implementation-artifacts/deferred-work.md` — modified (code review): one defer
  entry appended under "code review of story-4.13"

## Change Log

| Date       | Change                                                                 |
|------------|------------------------------------------------------------------------|
| 2026-07-24 | Story 4.13 drafted (create-story): DHCP churn — time enters as data (the first family whose timestamps carry the assertion). Two traps judging one recycled address: must-not-merge/`l1-distinct-mac` (REUSED) on [N1, N3] (same `IpV4` 192.0.2.120, distinct MACs/hostnames, two hours apart), must-merge/`l1-exact-mac` on [N1, N2] (identical MAC, same hostname, address moved .120→.121). Coins NOTHING; deliberately no `DhcpLease` (a rival time channel). Two NEW tests, both red-first: AC3's byte-pin in fixtures.rs (churn lives in `observed_at` alone) and AC4's reproducibility test in trap_gate.rs (score the corpus twice with one answers map answering both traps → identical `Report`s, scored 2 — D36: reproducibility, not stability). Committed count 12 → 14; manifest 13 → 15; README untouched. Status → ready-for-dev. |
| 2026-07-24 | Code review (3 fresh-context layers: Blind Hunter / Edge Case Hunter / Acceptance Auditor). **Auditor: PASS 9/9 AC** — every Dev Agent Record claim reproduced by re-measurement (both sha256 recomputed — Blind Hunter did it from the diff bytes alone —, reasons 275/237, tests 109+86+42, fixtures-gate wording verbatim). 0 decision-needed; **3 patches applied** (tests/doc only, no hashed artefact touched): AC4 doc weakened to the true sentence (same-process equality cannot red on a sort removal — that property is `discover_trap_files`' own), N2's moved lease `.121` now value-pinned (the must-merge premise), `facts.len()==3` per line (a duplicated fact can no longer slip past `find()`); **1 defer** → deferred-work.md (MAC/hostname values pinned relationally only — pre-existing class, exact-value pinning covers only `minimal.jsonl` since 4.10); 2 dismissed. Gates re-run green post-patch (109+86+42; "15 fixture(s) match their recorded sha256"). Status → done. |
| 2026-07-24 | Implemented (dev-story): all 6 tasks complete, ATDD order held — both new tests observed RED first (AC3's `FixtureError::Io` on the missing stream, AC4's `AnswerForUnknownTrap`), fixtures landed and both greened, count coupling proved RED at `left: 14, right: 12` then updated. Two new artefacts locked (manifest 13 → 15, sha256 after final byte); reasons measure 275/237 chars. Full gates green: fmt, clippy `-D warnings`, `cargo test --workspace` (109+86+42), `xtask ci` ("15 fixture(s) match their recorded sha256"). No production code, nothing coined, README/`architecture-views.md`/`Cargo.lock` untouched. Status → review. |
| 2026-07-24 | Validated (two fresh-context agents during create: fact-check + gap-hunt). Fact-check: every citation, count, free value, rule spelling and the full scoring trace verified against sources — 5 LOW (obs_id resolution is `read_traps`' in fixtures.rs:652-690, not `Trap::validate`'s; a hybrid quote re-attributed to trap.rs:66-73; walk line refs completed :821/:1306/:1327/:1424; `fixture_path` cannot be the AC3 red — it never touches the filesystem; breakdown comment starts at :387). Gap-hunt: 2 MED + 7 LOW (MED: new tests appended at END of modules so Task-4's cited line numbers stay valid, names are the anchor; MED: AC3's natural red correctly attributed to `read_jsonl`'s `FixtureError::Io`. LOW: `ts()` helper named (fixtures.rs:711); first==second carries AC4's "on both"; test-level-red caveat added to Task 1 + Trap #13; fixtures-gate message quoted verbatim from xtask/src/main.rs:532, no `generator` flag needed; randomized-mac kinship defence added; TOML layout note; reasons pre-measured 275/237). All 14 applied. Imports for both new tests confirmed already in scope; no other hard-coded count in the tree. |
