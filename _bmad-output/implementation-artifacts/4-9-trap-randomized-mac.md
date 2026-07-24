# Story 4.9: Trap family — randomized MAC

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author of the trap corpus,
I want the randomized-MAC family committed in positive AND negative form,
so that the engine is proven against the median identity case — not an exotic one — and against
BOTH ways it can fail: a false merge across two randomized presences, and a cowardly abstention on
an exact-MAC match whose MAC merely happens to be locally administered.

## Acceptance Criteria

1. **Given** a locally-administered MAC (the U/L bit set), **when** the positive-form trap is scored,
   **then** at L1 the two observations ARE distinct interfaces and the trap is a `must-not-merge`
   naming the rule that OPPOSES the merge (`l1-distinct-mac`). The positive form asserts **no merge
   across two randomized presences of the same physical interface**: two per-SSID randomized MACs of
   one laptop's Wi-Fi interface differ, and at L1 that difference opposes the merge — regrouping the
   two presences onto one device is L2's topology join, **never** L1's (architecture.md:890-913, 1172;
   D13 L1 = pure A, a deterministic join on `(l2_domain, mac)`).

2. **Given** the negative form, **when** it is scored, **then** it asserts the case where merging
   WOULD be correct and abstaining is cowardice — **D18's middle column, `must-merge`** — naming the
   rule that FIRES (`l1-exact-mac`). The same locally-administered MAC seen twice (only the lease
   moved) MUST merge: the U/L bit is no licence to abstain on an exact-MAC match, and an engine that
   refuses here is the always-abstain engine D18's middle column exists to demolish
   (architecture.md:946-951, 1001; trap.rs module doc).

3. **And** every expectation in the family carries its mandatory **one-sentence `reason`** — the
   oracle, since there is no external truth to appeal to (D19; `Trap::validate`). Each reason states
   plainly WHY the column is correct, in the family's own vocabulary (a randomized/locally-administered
   MAC, an L1 decision), so a later reader can disagree on the record.

4. **And** both traps declare `family = "randomized-mac"`, so the corpus-completeness check (story
   4.7b, `incomplete_families`) sees the family present in **both decision poles** (≥1 `must-merge`
   AND ≥1 `must-not-merge`) and the gate stays green. A one-sided randomized-MAC family would red
   `passed()` on its own — this story is what makes the family complete.

5. **And** the family is committed as **two NEW locked corpus artefacts** —
   `fixtures/scenario/replay/randomized-mac.jsonl` (the presences it judges) and
   `fixtures/scenario/traps/randomized-mac.toml` (the two traps) — **both added to
   `fixtures/MANIFEST.toml` with their sha256** in the same commit. This is a **deliberate corpus
   bump** (the first real trap family; unlike story 4.8's orphan-exempt README, these ARE locked
   artefacts): `cargo xtask ci`'s `fixtures` gate reports **7 artefacts, all sha256 match, no orphan**.
   Omitting either from the manifest is an ORPHAN finding (red); a wrong sha256 is an EDITED finding
   (red).

6. **And** the committed replay stream carries **synthetic values only** — locally-administered MACs
   (the family's whole premise; addr byte 0 has the 0x02 bit set, `locally_administered: true`), RFC
   5737 documentation IPs (`192.0.2.0/24`), invented `doc-`-prefixed hostnames — so the existing
   privacy walk `the_corpus_carries_no_real_network_data` passes over the new stream without a change
   (D19; [[no-private-data-in-artifacts]]).

7. **And** the three committed-corpus assertions that hard-code the trap count are updated from **3
   to 5** in the same change (`example.toml`'s 3 + the family's 2): `report.discovered()` at
   `trap_gate.rs:388` and `:424`, and the `"3 trap(s) discovered"` render string at `:406`. The
   scratch-corpus counts at `trap_gate.rs:497` and `:909` are NOT touched — they assert their own
   scratch dirs, not the committed root. At v0.1 `scored()` and `failures()` stay **0** for the whole
   committed corpus: no answer producer exists (Epic 5), so the family is discovered, parsed and
   validated — obs_ids resolve, reasons present, family complete — but scored by nothing.

## Tasks / Subtasks

- [x] **Task 1 — write the replay stream** `fixtures/scenario/replay/randomized-mac.jsonl` (AC: 1, 2, 6) — pure data, no code
  - [x] Author **three observations** in one stream, JSONL, matching the exact shape of
        `example-traps.jsonl` (fields `obs_id`, `connector_id`, `observed_at`, `scope` {`l2_domain`,
        `vantage`}, `facts`, `raw`). Use a **fresh `obs_id` prefix** (e.g. `cccccccc-…`) — the
        cross-stream rule forbids one `obs_id` in two streams (`fixtures.rs`
        `no_obs_id_is_shared_across_replay_streams`, :1306), and `aaaa`/`bbbb` are taken by
        `minimal.jsonl`/`example-traps.jsonl`. **Pin the three obs_ids as full valid UUIDs** and reuse
        the SAME spelling in the traps' `observations` lists (a non-UUID reds `ObsId`'s parse; a
        stream↔trap spelling mismatch reds `read_traps` with `DanglingObservation`): C1 =
        `cccccccc-0000-0000-0000-000000000001`, C2 = `cccccccc-0000-0000-0000-000000000002`,
        C3 = `cccccccc-0000-0000-0000-000000000003`. Keep `connector_id`,
        `l2_domain` and `vantage` **identical across all three** (the within-stream provenance/scope
        checks, story 4.5a/4.5b): reuse the synthetic UUIDs already in `example-traps.jsonl`
        (`connector_id` `33333333-…`, `l2_domain` `11111111-…`, `vantage` `22222222-…`).
  - [x] **C1 and C2 — the exact-MAC pair (feeds the `must-merge` negative form).** Both carry the
        IDENTICAL locally-administered MAC `02:00:5e:00:53:20` → `{"Mac":{"addr":[2,0,94,0,83,32],"locally_administered":true}}`,
        an hour apart, with DIFFERENT documentation IPs — the IP fact shape is
        `{"IpV4":{"addr":"192.0.2.30"}}` / `{"IpV4":{"addr":"192.0.2.31"}}` — so "only the lease moved"
        is literally true in the bytes.
  - [x] **Hostname is OPTIONAL — but if present it MUST carry `source`.** `Fact::Hostname` is
        `{ name, source }` with `#[serde(deny_unknown_fields)]` and **no default on `source`**
        (`observation/mod.rs:154-158`), and `example-traps.jsonl` carries NO hostname to copy from —
        so `{"Hostname":{"name":"doc-x"}}` fails to parse ("missing field `source`") and reds
        `every_replay_stream_in_the_corpus_is_valid`. The correct literal is
        `{"Hostname":{"name":"doc-laptop","source":"Dhcp"}}` (`HostnameSource` serializes PascalCase:
        `Dhcp`/`Dns`/`Mdns`/`Netbios`/`Other`, `observation/mod.rs:130-136`). **Simplest safe choice:
        omit the hostname entirely** — none of the ACs need it.
  - [x] **C3 — the re-randomized presence (feeds the `must-not-merge` positive form).** A DIFFERENT
        locally-administered MAC `02:00:5e:00:53:21` → `addr [2,0,94,0,83,33]`, `locally_administered:true`,
        IP `192.0.2.32`, later timestamp. C3 is the SAME physical interface as C1 after a per-SSID MAC
        re-randomization — a fact the `reason` states, not one the bytes can carry.
  - [x] **Synthetic-only, non-negotiable (AC6).** Every MAC has the U/L bit set (byte 0 = `0x02`,
        `locally_administered:true`); every IP is `192.0.2.x`; any hostname starts `doc-`. This is the
        exact discipline `assert_facts_are_synthetic` enforces (`fixtures.rs:865-895`) — a real vendor
        MAC or a non-5737 IP reds `the_corpus_carries_no_real_network_data`. Never a real capture (D19).

- [x] **Task 2 — write the family trap file** `fixtures/scenario/traps/randomized-mac.toml` (AC: 1, 2, 3, 4) — pure data, no code
  - [x] Open with a one-line header in the voice of `example.toml`: *the randomized-MAC family — a
        locally-administered MAC is the median case, not the exotic one (architecture.md:901); both
        decision poles, each judging `scenario/replay/randomized-mac.jsonl`.*
  - [x] **The `must-not-merge` (positive form, AC1).** Judges `[C1, C3]`. `family = "randomized-mac"`.
        `expect = { must-not-merge = { rule = "l1-distinct-mac" } }`. `reason` (one sentence, names the
        OPPOSING rule, not the tempting one): *"these two presences are one laptop's Wi-Fi interface
        after a per-SSID MAC re-randomization, yet their locally-administered MACs differ, which opposes
        an L1 merge — regrouping them onto one device is L2's topology join, never L1's."*
  - [x] **The `must-merge` (negative form, AC2).** Judges `[C1, C2]`. `family = "randomized-mac"`.
        `expect = { must-merge = { rule = "l1-exact-mac" } }`. `reason` (one sentence, the anti-cowardice
        point): *"both carry the identical locally-administered MAC 02:00:5e:00:53:20 an hour apart, so
        only the lease moved and the U/L bit is no licence to abstain on an exact-MAC match."*
  - [x] **Keep it to the two poles.** Do NOT add a `must-abstain` third form: the epic asks for
        positive + negative only, and DR1 makes an abstain member OPTIONAL (it satisfies no pole). A
        third column is a later, separate slice if ever wanted — leaving it out keeps this story small
        (story-granularity-small) and the family minimally complete.
  - [x] Reuse the EXACT rule ids already in the corpus: `l1-exact-mac` / `l1-distinct-mac`
        (`example.toml`, `trap_gate.rs` tests). Do not coin new rule names — Epic 5 closes `RuleId`
        into an enum, and an invented id there becomes an `Other(String)` the engine cannot honour
        (trap.rs:31-38).

- [x] **Task 3 — lock the two new artefacts** `fixtures/MANIFEST.toml` (AC: 5) — deliberate corpus bump
  - [x] Append TWO `[[artefact]]` entries (paths `scenario/replay/randomized-mac.jsonl` and
        `scenario/traps/randomized-mac.toml`), each with a one-line comment naming the story and what
        the artefact is, mirroring the 4.5a/4.5b entries' style. Compute each `sha256` from the
        committed bytes — `sha256sum fixtures/scenario/replay/randomized-mac.jsonl` and
        `…/traps/randomized-mac.toml` — and paste the hex. A mismatch is an EDITED finding; an omission
        is an ORPHAN finding. **Author Task 1/Task 2 FIRST, then hash** — any later byte edit (even a
        trailing newline) invalidates the sha256.
  - [x] Do NOT touch the five existing entries or their sha256 (`example.toml` and the four streams
        stay byte-for-byte — any edit there is an unrelated EDITED finding).

- [x] **Task 4 — update the committed-count assertions** `crates/opencmdb-bin/src/trap_gate.rs` (AC: 7) — three edits, tests only
  - [x] `the_committed_corpus_is_discovered_and_scored_by_nothing` (`:388`): `report.discovered()`
        `3` → `5`. Keep `scored()` and `failures()` at `0` (no producer exists).
  - [x] `the_report_says_plainly_that_nothing_was_scored` (`:406`): `"3 trap(s) discovered"` →
        `"5 trap(s) discovered"`. Leave `"0 scored"` and `"0 truth-table failure(s)"` unchanged.
  - [x] `a_trap_with_no_answer_is_discovered_but_not_scored` (`:424`): `report.discovered()` `3` → `5`
        (still answers ONE trap, so `scored()` stays `1`; the other four are unanswered).
  - [x] **Do NOT touch `:497` or `:909`** — `each_column_can_be_driven_red` and
        `a_one_sided_family_reddens_the_gate_on_its_own` build their OWN scratch corpora of 3 traps;
        their counts are unrelated to the committed root. Changing them would be wrong.
  - [x] **Prove the count coupling is real (record it).** Before adding the family to the manifest,
        run `cargo test -p opencmdb-bin the_committed_corpus_is_discovered_and_scored_by_nothing` with
        the assertions still at `3`: it should already RED once the new `.toml` is on disk (the walk
        discovers 5), confirming the family is genuinely discovered. Then update to `5`. Record this in
        the Debug Log — the discovery count IS the guard that the family landed. (No new test needed:
        `discover_trap_files` walks `.toml` under `scenario/traps/` automatically, so the new file is
        picked up with zero harness change beyond the count.)

- [x] **Task 5 — gates and the corpus lock** (AC: 4, 5, 6, 7)
  - [x] `cargo fmt --all` · `cargo clippy --workspace --all-targets --locked -- -D warnings` ·
        `cargo test --workspace --locked` · `cargo run -p xtask -- ci` — all green.
  - [x] `cargo xtask ci`'s `fixtures` gate must report **7 artefacts, all sha256 match, and no orphan
        finding** (5 existing + 2 new). Confirm the new stream and trap file are BOTH listed and BOTH
        match. `file-size` gate: `.toml`/`.jsonl` are not counted (the gate measures `.rs` before the
        first `#[cfg(test)]`); `trap_gate.rs` grows by nothing but three literals inside `#[cfg(test)]`.
  - [x] Confirm the whole committed corpus still `passed()` at v0.1: `discovered() == 5`,
        `scored() == 0`, `failures() == 0`, `rule_mismatches` empty, `incomplete_families` empty (the
        family carries both poles → complete). If any test that renders the report elsewhere hard-codes
        `3`, catch it here (grep `"3 trap"` / `discovered(), 3` across `crates/`).
  - [x] `Cargo.lock` unchanged (no dependency touched — two data files, one manifest bump, three test
        literals). `architecture-views.md` is NOT regenerated in this story (`ℹ views-hash STALE` is by
        design — regenerate at the Epic-4 milestone, not here). `example.toml` and the four existing
        streams are byte-for-byte unchanged (measured via `git status --short`).

## Dev Notes

### The shape of this story in one paragraph

This is the **first real trap family**, and it is almost entirely DATA. Two new committed artefacts —
a three-observation replay stream and a two-trap family file — plus a deliberate `MANIFEST.toml` bump
to lock them, plus three test-literal updates to keep the committed-corpus count assertions honest
(3 → 5). There is **no engine and no new harness code**: at v0.1 the corpus is discovered, parsed and
validated (obs_ids resolve against the named stream, `reason` present, family complete in both poles)
but scored by nothing — the `(verdict, rule)` scoring arrives in Epic 5. The family pins the L1 rule
from **both sides**: the `must-not-merge` (positive form) says two *different* randomized MACs are
distinct L1 interfaces even when they are the same physical device (regrouping is L2's job); the
`must-merge` (negative form) says the *same* randomized MAC seen twice must still merge — the U/L bit
is no excuse to abstain (D18's anti-cowardice middle column). Together they say: **L1 decides on MAC
equality, and "locally administered" changes nothing about that.**

### Which pole is which — do not flip them (AC1/AC2)

The epic's wording is exact and easy to invert; hold it:

| Epic phrase | Column | Rule | Judges | The point |
|---|---|---|---|---|
| **positive form** — "no merge across two randomized presences of the same physical interface" | `must-not-merge` | `l1-distinct-mac` (the OPPOSING rule) | `[C1, C3]` (different MACs) | L1 keeps two randomized MACs distinct; regrouping them onto one device is **L2's** topology join |
| **negative form** — "merging WOULD be correct and abstaining is cowardice (D18's middle column)" | `must-merge` | `l1-exact-mac` (the rule that FIRES) | `[C1, C2]` (same MAC) | the same randomized MAC twice must merge; the U/L bit is no licence to abstain |

D18's three columns (trap.rs:55-76): `must-not-merge` guards the false merge (operator loses trust,
uninstalls), `must-merge` guards **cowardice** (*"an engine that abstains on everything scores
false-merge = 0 and gets demolished by the middle column"*), `must-abstain` guards guessing on the
honestly ambiguous case. This family exercises the first two. The `must-not-merge` names the rule that
OPPOSES the merge, **not** the tempting one (trap.rs:69-73) — `l1-distinct-mac`, because the MACs
differ; a refusal without a named opposing rule is undebuggable.

### Why a randomized MAC is the median, not the exotic (the architecture grounding)

- *"a laptop has Wi-Fi + dock Ethernet + a per-SSID randomized MAC. **These are the median, not the
  exotic.**"* (architecture.md:900-901)
- *"What we observe is not an interface. It is a **presence: (MAC, context, time)**. A per-SSID
  randomized MAC produces two distinct presences that are the same physical interface. **If interface
  == MAC, we gained nothing — we merely renamed.**"* (architecture.md:910-913)
- MAC randomization = 1 device, N ephemeral interfaces = **both** L1 and L2 (architecture.md:895). The
  L1 job (this story) is to keep the presences distinct correctly; the L2 job (Epic 6+) is to regroup
  them. This story only asserts the L1 half, in both poles.
- The **U/L bit** (2nd bit of the 1st octet) = locally administered = randomized/virtual
  (architecture.md:1001); *"a reading, not an inference"* is the register the D13 passage is written in
  (the literal phrase sits at architecture.md:1000 on the sibling IANA-prefix bullet, extended to the
  U/L bit at :1002). Byte 0 = `0x02` sets it; that is why `addr[0]` is `2` in
  every synthetic MAC in the corpus and `locally_administered: true` is carried alongside.

### At v0.1 there is no engine — what "scored" means here (AC7)

`score_corpus` (`trap_gate.rs:223`) discovers `.toml` files, reads and validates each trap through
`read_traps`, and scores each trap **only if `answers` carries an `Outcome` for its `TrapId`**. At
v0.1 the answers map is empty for a real run (no producer exists — Epic 5 builds it), so:

- `discovered()` counts every trap the walk opened → **5** after this story (3 in `example.toml` + 2
  in `randomized-mac.toml`).
- `scored()` and `failures()` stay **0** for the committed corpus.
- `incomplete_families()` runs answer-INDEPENDENTLY over corpus SHAPE (4.7b): the family carries both
  poles, so it is complete and the bucket stays empty; `passed()` stays green.

The three committed-count assertions (`trap_gate.rs:388`, `:406`, `:424`) are the only tests coupled
to the count — they exist precisely so the zeros stay *honest* (a gate that discovered nothing is not
a passing gate). Updating `3 → 5` is mandatory and is itself the evidence the family was discovered.
The scratch-corpus tests (`:497`, `:909`) build their own 3-trap dirs and must NOT be touched.

### How a trap is validated (so the family stays green) — the checks the new files must survive

`read_traps` (`fixtures.rs`) enforces, per trap:
- **obs_id resolution** — every id in `observations` must exist in the trap's named `replay` stream, or
  `read_traps` fails *"trap `X` judges observation `O`, which `replay` does not contain"*
  (`fixtures.rs:313-317`). So `randomized-mac.toml`'s `replay = "scenario/replay/randomized-mac.jsonl"`
  and every `obs_id` it lists must be in that committed stream. **`read_traps` resolves `replay`
  against the BAKED corpus root** (trap_gate.rs:218-222) — the stream must be a committed file, which
  it is once Task 1 lands.
- **`reason` non-empty** — `Trap::validate` refuses an empty/blank reason (the oracle; the principle is
  the module doc trap.rs:12-15, the enforcement is `Trap::validate` at trap.rs:258-268; also bounds the
  reason to 20–300 chars, single line, no control char — both prescribed reasons pass, ~170 and ~230 chars).
- **unique trap ids across files** — `score_corpus` refuses a `TrapId` seen in two files
  (`DuplicateTrapId`, trap_gate.rs:245-251). Pick ids unique in the whole corpus (e.g.
  `randomized-mac-must-merge`, `randomized-mac-must-not-merge` — `example.toml` uses `example-must-*`,
  so no clash).
- **family completeness** — `incomplete_families` (opencmdb-core) groups by case-folded family name and
  requires ≥1 `must-merge` AND ≥1 `must-not-merge` (DR1: a `must-abstain` satisfies neither pole). Both
  traps declaring `family = "randomized-mac"` satisfies this.

And `every_trap_file_in_the_corpus_is_valid` (`fixtures.rs`, the corpus-integrity walk) will parse the
new `.toml` automatically — no change needed there; if the file is malformed it reds that test by name.

### The corpus lock is a DELIBERATE bump this time (AC5) — the opposite of story 4.8

Story 4.8 added an **orphan-exempt README** — invisible to `MANIFEST.toml` on purpose. Story 4.9 adds
**two real artefacts that MUST be locked**:

- The `fixtures` gate is two-directional (MANIFEST.toml header): an artefact under `fixtures/` absent
  from the manifest is an **orphan** (red); a listed artefact whose bytes changed is **edited** (red).
  Adding the two entries with correct sha256 is what turns both green.
- This is a *"I am changing the spec"* commit by design — the review SHOULD see the manifest move. That
  is the mechanism working, not a problem to avoid.
- Order of operations matters: **write the data files, THEN hash them, THEN paste the sha256.** Hashing
  before a final byte edit (a reformatted line, a trailing newline) pins the wrong digest and reds the
  gate with a confusing "edited" finding on a brand-new file.
- READMEs stay exempt: do NOT add `traps/README.md` (the reality-debt register from 4.8) to the
  manifest — it is orphan-exempt by name and locking it would defeat its append-freely purpose.

### Project Structure Notes

- **NEW (locked):** `fixtures/scenario/replay/randomized-mac.jsonl` (3 observations),
  `fixtures/scenario/traps/randomized-mac.toml` (2 traps, `family = "randomized-mac"`). Both listed in
  `fixtures/MANIFEST.toml` with sha256.
- **Updated:** `fixtures/MANIFEST.toml` (two `[[artefact]]` entries — the deliberate bump);
  `crates/opencmdb-bin/src/trap_gate.rs` (three `#[cfg(test)]` count literals, 3 → 5 — tests only, no
  production logic).
- **Unchanged, expected:** `example.toml` and the four existing replay streams (byte-for-byte — any
  edit is an unrelated EDITED finding); `trap.rs` / `score.rs` / `trap_gate.rs` production paths (the
  domain and harness are frozen since 4.6a/4.7a/4.7b — this story adds no rule, no scoring, no engine);
  `traps/README.md` (orphan-exempt, not locked); `Cargo.lock` (no dependency touched); every other
  `.rs`. `discover_trap_files` / `incomplete_families` / `read_traps` are used AS-IS.
- **Out of scope, deliberately:** any engine or rule (Epic 5); a `must-abstain` third form for the
  family (DR1 makes it optional; the epic asks two poles); `docs/project-context.md` and `CLAUDE.md`
  (a new locked family is milestone/push-level under docs-current-before-push — fold in at the Epic-4
  milestone with the `architecture-views.md` regeneration, not per-story); the multi-NIC family (story
  4.10) and every later family.

### Traps (mistakes this story must not make)

1. **Flipping the poles.** Positive form = `must-not-merge` (different MACs, `l1-distinct-mac`);
   negative form = `must-merge` (same MAC, `l1-exact-mac`, the anti-cowardice column). The table above
   is the guard — re-read the epic's two Givens before writing the `.toml`.
2. **Forgetting the manifest bump.** The two new files are LOCKED artefacts (unlike 4.8's README). Both
   need a `[[artefact]]` entry with the correct sha256, or the `fixtures` gate reds as an orphan.
3. **Hashing before the final byte.** Compute sha256 only after the files are final; a later trailing
   newline invalidates it.
4. **Leaving the count assertions at 3.** Three committed-corpus tests hard-code `3`; the family makes
   it `5`. Update `trap_gate.rs:388/:406/:424` — and do NOT touch the scratch counts at `:497`/`:909`.
5. **Recording a real MAC/hostname/IP.** Synthetic-only: locally-administered MACs (U/L bit set), RFC
   5737 IPs, `doc-` hostnames. The privacy walk enforces it; a real capture is disqualifying, not a
   preference (D19; [[no-private-data-in-artifacts]]).
6. **Reusing an existing stream's `obs_id` prefix.** New stream → fresh ids (`cccccccc-…`); a shared
   `obs_id` across two streams reds `no_obs_id_is_shared_across_replay_streams` (`fixtures.rs:1306`).
7. **Coining a new `rule` id.** Use `l1-exact-mac` / `l1-distinct-mac` — the corpus's established ids;
   Epic 5 will close `RuleId` into an enum and an invented id becomes an unhonourable `Other(String)`.
8. **Inconsistent `connector_id`/`scope` within the stream.** All three observations share one
   `connector_id`, `l2_domain` and `vantage` (the within-stream provenance/scope checks, 4.5a/4.5b).
9. **Adding a third `must-abstain` "to be safe".** Not asked, not needed for completeness (DR1), and it
   widens a deliberately small story.
10. **Regenerating `architecture-views.md`.** Its `ℹ views-hash STALE` is by design — Epic-4 milestone,
    not this story.
11. **Claiming more than measured.** Name the command behind every count; record the discovery-count
    coupling proved (5 discovered). Write the weaker true sentence. [[claims-must-match-verification]]

### Latest technical specifics

No new crate, no version bump, no domain or harness code. Rust 1.96+, edition 2024. Two data files
(one `.jsonl`, one `.toml`), one `MANIFEST.toml` bump, three `#[cfg(test)]` literals in `opencmdb-bin`.
**Never invent a version — pin from the committed `Cargo.lock`, which does not move here.**

### References

- [Source: _bmad-output/planning-artifacts/epics.md:1089-1105 — Story 4.9 "Trap family — randomized
  MAC": the two Givens (positive form = no merge across two randomized presences, `must-not-merge` at
  L1; negative form = merging correct / abstaining cowardly, D18's middle column `must-merge`) and the
  mandatory one-sentence reason]
- [Source: _bmad-output/planning-artifacts/architecture.md:888-927 — the L1/L2 split, "median not
  exotic", "a presence is (MAC, context, time)", "if interface == MAC we gained nothing"; the semantic
  spine of both poles]
- [Source: _bmad-output/planning-artifacts/architecture.md:984-1002 — D13: L1 = pure A, a deterministic
  join on `(l2_domain, mac)`; the U/L bit is "a reading, not an inference"]
- [Source: _bmad-output/planning-artifacts/architecture.md:1172 — "At L1 a randomized MAC IS a distinct
  interface"; the `must-not-merge` positive form's grounding]
- [Source: crates/opencmdb-core/src/trap.rs:31-124 — `RuleId`/`TrapId`/`FamilyId`, the `Expectation`
  enum and its three columns (the `must-not-merge` "name the OPPOSING rule" contract, :69-73), and
  `Trap` with `#[serde(default)] family: Option<FamilyId>` — the fields the family file populates]
- [Source: fixtures/scenario/traps/example.toml — the trap-file format and the exact model this family
  follows (`example-must-merge` = identical MAC an hour apart / `l1-exact-mac`; `example-must-not-merge`
  = MACs differ in the final octet / `l1-distinct-mac`); "the families arrive from story 4.9 onward"]
- [Source: fixtures/scenario/replay/example-traps.jsonl — the JSONL observation shape to mirror
  (`obs_id`, `connector_id`, `observed_at`, `scope`, `facts` with `Mac`/`IpV4`, `raw`); locally-
  administered MACs `[2,0,94,0,83,x]`]
- [Source: fixtures/MANIFEST.toml — the two-directional corpus lock (edited + orphan); the 4.5a/4.5b
  entry style to mirror for the two new artefacts]
- [Source: crates/opencmdb-bin/src/trap_gate.rs:223-289 — `score_corpus`: discovery, per-trap
  validation via `read_traps`, answer-independent `incomplete_families`; `:382-427` — the three
  committed-count assertions (3 → 5); `:291-357` — `discover_trap_files` picks up the new `.toml`
  automatically]
- [Source: crates/opencmdb-bin/src/fixtures.rs:305-320 — `read_traps`' obs_id cross-check (a trap may
  only judge observations its stream contains); :820-914 — `the_corpus_carries_no_real_network_data` /
  `assert_facts_are_synthetic` / `assert_synthetic_mac` / `assert_documentation_ip`, the privacy floor
  the new stream must pass]
- [Source: _bmad-output/implementation-artifacts/4-7b-trap-corpus-positive-and-negative.md:235-253 —
  DR1 (a `must-abstain` satisfies neither pole; a family is complete iff it has both `must-merge` AND
  `must-not-merge`) and DR2; the completeness contract this family satisfies]
- [Source: _bmad-output/implementation-artifacts/4-8-reality-debt-register.md — the immediately prior
  story; note the CONTRAST: 4.8 added an orphan-exempt README, 4.9 adds two LOCKED artefacts and bumps
  the manifest deliberately]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Amelia / bmad-dev-story)

### Debug Log References

- **Prove-to-red on the discovery count (Task 4).** With `randomized-mac.toml` on disk but the
  count assertions still at `3`,
  `cargo test -p opencmdb-bin the_committed_corpus_is_discovered_and_scored_by_nothing` RED at
  `trap_gate.rs:388` with `left: 5, right: 3` ("the walk must open the corpus") — the walk opened
  5 traps, which IS the evidence the family landed on the committed root (`score_corpus` resolves
  the corpus from live disk, no rebuild). Assertions then moved 3 → 5 and the test greens.
- **`reason` lengths** measured 230 and 170 chars — both inside `Trap::validate`'s 20–300 single-line
  bound (`trap.rs:173-176`).
- **sha256 computed AFTER the files were final** (Task 3 discipline): stream
  `b271e11f…fc99`, trap file `f0ded78e…912b` (recomputed after the code-review prefix hardening
  `cccccccc` → `eeeeeeee`).

### Completion Notes List

- First real trap family committed as **two locked artefacts** + a deliberate `MANIFEST.toml` bump;
  no engine, no new harness code (v0.1 discovers/parses/validates, scores nothing — Epic 5 scores).
- **Both poles pinned, not flipped:** `must-not-merge`/`l1-distinct-mac` judges `[C1,C3]` (two
  different randomized MACs → distinct L1 interfaces; regrouping is L2's job); `must-merge`/
  `l1-exact-mac` judges `[C1,C2]` (identical locally-administered MAC an hour apart → the U/L bit is
  no licence to abstain, D18's middle column).
- **Synthetic-only stream:** every MAC locally-administered (byte 0 = `0x02`,
  `locally_administered:true`), every IP RFC-5737 `192.0.2.x`, obs_id prefix `eeeeeeee-0000-4000-8000-…`
  — a truly-free leading block (aaaa/bbbb/cccc/dddd are all taken by the four existing streams) in the
  corpus's v4 shape (code review hardened this from the initially-pinned `cccccccc`, which shared a
  leading block with `capability-downgrade.jsonl`); no hostname (the simplest safe choice). Privacy
  walk passes untouched.
- **Verification (commands, not claims):** `cargo fmt --all` clean · `cargo clippy --workspace
  --all-targets --locked -- -D warnings` clean · `cargo test --workspace --locked` all green ·
  `cargo run -p xtask -- ci` → `fixtures  7 fixture(s) match their recorded sha256`, no orphan;
  `views-hash STALE` is by design (ℹ, exit 0), NOT regenerated in this story.
- **Untouched, verified:** `Cargo.lock`, `example.toml`, the four existing replay streams
  (byte-for-byte, `git status --short`); the scratch-corpus counts at
  `each_column_can_be_driven_red` / `a_one_sided_family_reddens_the_gate_on_its_own` (their own
  3-trap dirs, correctly left at `3`); `traps/README.md` (orphan-exempt, not locked).

### Review Findings

- [x] [Review][Decision] **RESOLVED — hardened to `eeeeeeee-0000-4000-8000-…`** (a truly-free
  leading block in the corpus's v4 shape; both artefacts re-hashed in `MANIFEST.toml`, trap
  `observations` updated, gates re-run green). Original finding: obs_id prefix `cccccccc` is not a
  fresh leading block — it collides at the
  leading nibbles with `capability-downgrade.jsonl` (`cccccccc-0000-4000-8000-…`) and breaks the
  corpus's v4-shape convention (every other stream is `-0000-4000-8000-…`). **No collision today:**
  the full UUIDs differ (`-0000-0000-` vs `-0000-4000-8000-`), so `no_obs_id_is_shared_across_replay_streams`
  passes and all gates are green. But the story's Task-1 premise "fresh `cccccccc-…` prefix (aaaa/bbbb
  are taken)" is factually wrong: `cccc` (capability-downgrade) and `dddd` (partial-then-failed) are
  BOTH taken; the first truly-free block is `eeeeeeee`. Fragility: if a future editor normalises these
  obs_ids to proper v4 form, they would collide with `capability-downgrade.jsonl` and red the
  cross-stream guard. **Decision:** keep `cccccccc-0000-0000-0000-…` as committed (passes, ships), or
  harden to `eeeeeeee-0000-4000-8000-…` (distinct free block + matches the v4 convention), which
  requires re-hashing both new artefacts in `MANIFEST.toml` and updating the trap file's `observations`.
  Source: Edge Case Hunter (LOW severity), confirmed by direct prefix audit across all five streams.

### File List

- `fixtures/scenario/replay/randomized-mac.jsonl` (new, locked) — three synthetic observations
- `fixtures/scenario/traps/randomized-mac.toml` (new, locked) — the two-pole family
- `fixtures/MANIFEST.toml` (modified) — two `[[artefact]]` entries with sha256 (deliberate bump)
- `crates/opencmdb-bin/src/trap_gate.rs` (modified) — three `#[cfg(test)]` count literals 3 → 5
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified) — status → review

## Change Log

| Date       | Change                                                                 |
|------------|------------------------------------------------------------------------|
| 2026-07-24 | Story 4.9 implemented: randomized-MAC trap family (two locked artefacts, manifest bump, committed-count 3 → 5). Status → review. |
