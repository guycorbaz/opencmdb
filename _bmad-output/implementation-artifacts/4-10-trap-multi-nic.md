# Story 4.10: Trap family — multi-NIC

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author of the trap corpus,
I want the multi-NIC family committed in positive AND negative form,
so that a false SPLIT at DEVICE level is caught where it actually lives — at **L2**, the device-grouping
layer — and so the corpus proves the engine against BOTH ways the layer can fail: failing to group two
NICs of one host (a false split), and grouping two genuinely different hosts (a false merge).

## Acceptance Criteria

1. **Given** two interfaces of one host (two distinct MACs seen at the same access-switch uplink), **when**
   the primary-form trap is scored, **then** L1 is CORRECT to keep the two interfaces distinct and **L2
   must GROUP them** — so the trap is a `must-merge` naming the L2 rule that FIRES (`l2-uplink-agrees`).
   **The failure being tested is L2's false split, not L1's** (architecture.md:893: *"multi-NIC false-split
   = L1 correct, L2 failed to group"*). This is the L2 job that story 4.9 explicitly deferred (4.9's
   `must-not-merge` said "regrouping them onto one device is L2's topology join, never L1's"); 4.10
   is where that L2 grouping is finally asserted.

2. **Given** the inverse form — two interfaces of two GENUINELY DIFFERENT hosts (distinct MACs on
   DIFFERENT access switches) — **when** it is scored, **then** it asserts that they are **NOT grouped**:
   a `must-not-merge` naming the L2 rule that OPPOSES the grouping (`l2-different-switch`). Different-switch
   topology opposes the L2 join (it is a decisive negative, not merely an absence of support), which is why
   the refusal names a rule and not a cause (trap.rs:69-73; example.toml's "opposes the merge rather than
   merely failing to support it").

3. **And** every expectation in the family carries its mandatory **one-sentence `reason`** — the oracle,
   since there is no external truth to appeal to (D19; `Trap::validate`). Each reason states plainly WHY
   the column is correct in the family's own vocabulary (two NICs of one host / two hosts on different
   switches; an L2 topology decision, L1 left correct), so a later reader can disagree on the record.

4. **And** both traps declare `family = "multi-nic"`, so the corpus-completeness check (story 4.7b,
   `incomplete_families`) sees the family present in **both decision poles** (≥1 `must-merge` AND ≥1
   `must-not-merge`) and the gate stays green. A one-sided multi-NIC family would red `passed()` on its own.

5. **And** the family is committed as **two NEW locked corpus artefacts** —
   `fixtures/scenario/replay/multi-nic.jsonl` (the three presences it judges) and
   `fixtures/scenario/traps/multi-nic.toml` (the two traps) — **both added to `fixtures/MANIFEST.toml`
   with their sha256** in the same commit (a **deliberate corpus bump**, exactly like story 4.9). After
   this story `cargo xtask ci`'s `fixtures` gate reports **9 artefacts, all sha256 match, no orphan** (7
   existing + 2 new). Omitting either from the manifest is an ORPHAN finding (red); a wrong sha256 is an
   EDITED finding (red).

6. **And** the committed replay stream carries **synthetic values only** — locally-administered MACs (byte
   0 has the 0x02 bit set, `locally_administered: true`, INCLUDING every `Uplink.peer_mac`), RFC 5737
   documentation IPs (`192.0.2.0/24`), and no captured hostnames — so the existing privacy walk
   `the_corpus_carries_no_real_network_data` passes over the new stream unchanged (D19;
   [[no-private-data-in-artifacts]]). The MACs are locally-administered synthetic STAND-INS for real NICs;
   the corpus privacy floor forbids a real vendor address regardless of what a real multi-NIC host would
   present (fixtures.rs:906-913 `assert_synthetic_mac`).

7. **And** the three committed-corpus assertions that hard-code the trap count are updated from **5 to 7**
   in the same change (`example.toml`'s 3 + `randomized-mac.toml`'s 2 + this family's 2):
   `report.discovered()` at `trap_gate.rs:389` and `:425`, and the `"5 trap(s) discovered"` render string
   at `:407`. The scratch-corpus counts elsewhere in `trap_gate.rs` (`each_column_can_be_driven_red`,
   `a_one_sided_family_reddens_the_gate_on_its_own`, and the other scratch tests) are **NOT** touched — they
   assert their own scratch dirs, not the committed root. At v0.1 `scored()` and `failures()` stay **0** for
   the whole committed corpus: no answer producer exists (Epic 5), so the family is discovered, parsed and
   validated (obs_ids resolve, reasons present, family complete in both poles) but scored by nothing.

## Tasks / Subtasks

- [x] **Task 1 — write the replay stream** `fixtures/scenario/replay/multi-nic.jsonl` (AC: 1, 2, 6) — pure data, no code
  - [x] Author **three observations** in one stream, JSONL, matching the exact shape of
        `randomized-mac.jsonl`/`example-traps.jsonl` (fields `obs_id`, `connector_id`, `observed_at`,
        `scope` {`l2_domain`, `vantage`}, `facts`, `raw`). Use a **fresh `obs_id` prefix `ffffffff`** —
        the cross-stream rule forbids one `obs_id` in two committed streams
        (`fixtures.rs` `no_obs_id_is_shared_across_replay_streams`, :1306), and `aaaa`/`bbbb`/`cccc`/`dddd`/`eeee`
        are all taken by the five existing committed streams. **`ffffffff` is free in the committed streams**
        (the `ffffffff-0000-4000-8000-00000000dead` at `fixtures.rs:1649` is a SCRATCH-test literal in a
        tempdir, never walked by the cross-stream guard, and its full UUID differs anyway). **Pin the three
        obs_ids as full valid UUIDs** and reuse the SAME spelling in the traps' `observations` lists (a
        non-UUID reds `ObsId`'s parse; a stream↔trap spelling mismatch reds `read_traps` with
        `DanglingObservation`):
        M1 = `ffffffff-0000-4000-8000-000000000001`, M2 = `ffffffff-0000-4000-8000-000000000002`,
        M3 = `ffffffff-0000-4000-8000-000000000003`. Keep `connector_id`, `l2_domain` and `vantage`
        **identical across all three** (the within-stream provenance/scope checks, story 4.5a/4.5b): reuse
        the synthetic UUIDs already in the corpus (`connector_id` `33333333-3333-4333-8333-333333333333`,
        `l2_domain` `11111111-1111-4111-8111-111111111111`, `vantage` `22222222-2222-4222-8222-222222222222`).
  - [x] **The topology signal is the `Uplink` fact — new to the committed corpus, so write its shape from
        the type** (`observation/mod.rs:164-168`): `Fact::Uplink { peer_mac: MacAddr, peer_port: String }`.
        `MacAddr` serializes as its 6-byte array (same as the `Mac` fact's `addr`), so the literal is
        `{"Uplink":{"peer_mac":[2,0,94,0,96,10],"peer_port":"swport-1"}}`. **`peer_mac` MUST be
        locally-administered** (byte 0 = `0x02`) — `assert_facts_are_synthetic` runs `assert_synthetic_mac`
        on it (`fixtures.rs:877`). `peer_port` is a free `String`, not privacy-scanned; keep it innocuous
        (`swport-1`, does not parse as an IP/MAC).
  - [x] **M1 and M2 — the one-host pair (feeds the `must-merge` primary form).** Two DIFFERENT
        locally-administered MACs on the **SAME access switch** — `Uplink.peer_mac` IDENTICAL
        (`[2,0,94,0,96,10]` = switch A), different `peer_port` (`swport-1` / `swport-2`): one server
        dual-homed into two ports of the same switch. Facts per observation: one `Mac`, one `IpV4`, one
        `Uplink`.
        - M1: `{"Mac":{"addr":[2,0,94,0,83,64],"locally_administered":true}}` (`02:00:5e:00:53:40`),
          `{"IpV4":{"addr":"192.0.2.40"}}`, `{"Uplink":{"peer_mac":[2,0,94,0,96,10],"peer_port":"swport-1"}}`.
        - M2: `{"Mac":{"addr":[2,0,94,0,83,65],"locally_administered":true}}` (`02:00:5e:00:53:41`),
          `{"IpV4":{"addr":"192.0.2.41"}}`, `{"Uplink":{"peer_mac":[2,0,94,0,96,10],"peer_port":"swport-2"}}`.
  - [x] **M3 — the different-host interface (feeds the `must-not-merge` inverse form).** A DIFFERENT
        locally-administered MAC on a **DIFFERENT access switch** — `Uplink.peer_mac` = `[2,0,94,0,96,11]`
        (switch B), so the topology says "different switch":
        `{"Mac":{"addr":[2,0,94,0,83,66],"locally_administered":true}}` (`02:00:5e:00:53:42`),
        `{"IpV4":{"addr":"192.0.2.42"}}`, `{"Uplink":{"peer_mac":[2,0,94,0,96,11],"peer_port":"swport-7"}}`.
  - [x] **`observed_at` — distinct, plausible timestamps.** No ordering guard applies to plain
        observations (`CapabilityOutOfOrder` only governs capability records, of which this stream has
        none), so any valid RFC3339 values parse — but do NOT paste one identical timestamp into all three.
        Give M1 and M2 the SAME scan time (one scan sees both of a host's NICs) and M3 a later one, e.g.
        M1/M2 = `2026-01-03T00:00:00Z`, M3 = `2026-01-03T00:05:00Z`, so the stream reads realistically.
        Timestamps carry no assertion in this family — the topology does.
  - [x] **Synthetic-only, non-negotiable (AC6).** Every MAC — host MAC AND `Uplink.peer_mac` — has the U/L
        bit set (byte 0 = `0x02`, `locally_administered:true`); every IP is `192.0.2.x`; no hostname. This
        is exactly the discipline `assert_facts_are_synthetic` enforces (`fixtures.rs:867-913`) — a real
        vendor MAC (in a `Mac` OR an `Uplink`) or a non-5737 IP reds `the_corpus_carries_no_real_network_data`.
        Never a real capture (D19).

- [x] **Task 2 — write the family trap file** `fixtures/scenario/traps/multi-nic.toml` (AC: 1, 2, 3, 4) — pure data, no code
  - [x] Open with a short header in the voice of `randomized-mac.toml`/`example.toml`: *the multi-NIC
        family — a device with several NICs is the median, not the exotic (architecture.md:898-901); this
        family lives at L2 (device grouping), where a false split actually happens (architecture.md:893).
        Both decision poles judge `scenario/replay/multi-nic.jsonl`.*
  - [x] **The `must-merge` (primary form, AC1).** Judges `[M1, M2]`. `family = "multi-nic"`.
        `expect = { must-merge = { rule = "l2-uplink-agrees" } }`. `reason` (one sentence, names the L2 rule
        that FIRES): *"these two interfaces are one server's NICs dual-homed into the same access switch, so
        L1 is right to keep their distinct MACs apart while L2 groups them by their agreeing uplink — the
        false split this catches lives at L2, never L1."*
  - [x] **The `must-not-merge` (inverse form, AC2).** Judges `[M1, M3]`. `family = "multi-nic"`.
        `expect = { must-not-merge = { rule = "l2-different-switch" } }`. `reason` (one sentence, names the
        OPPOSING rule): *"these two interfaces sit on different access switches, so they are genuinely
        separate hosts and grouping them would be a false merge — different-switch topology opposes the L2
        join rather than merely failing to support it."*
  - [x] **Keep it to the two poles.** Do NOT add a `must-abstain` third form: the epic asks for the
        primary + inverse forms only, and DR1 makes an abstain member OPTIONAL (it satisfies no pole). The
        genuinely-ambiguous topology case (e.g. two MACs behind ONE switch port — host or downstream hub?)
        is a candidate future `must-abstain` member and belongs in the reality-debt register, not this
        small slice (story-granularity-small).
  - [x] **Reuse the EXACT L2 rule ids already in the corpus's vocabulary.** Copy-paste these two literals
        VERBATIM — `l2-uplink-agrees` (the merge rule) and `l2-different-switch` (the opposing rule):
        both are established in `score.rs`'s tests (`score.rs:620,625,705,708,905-906,924,943`) as the L2
        `Outcome::Merged`/`Outcome::Refused` rules. **These ids have never appeared in a committed `.toml`
        (unlike `l1-exact-mac`/`l1-distinct-mac`, which `example.toml` and `randomized-mac.toml` carry), so
        there is no sibling fixture to pattern-copy from — take the spelling from `score.rs`, not from
        memory, and double-check for a typo.** At v0.1 nothing validates rule-id membership, so a
        mis-spelled id ships silently now and becomes an unhonourable `Other(String)` when Epic 5 closes
        `RuleId` into an enum (trap.rs:31-38). Do NOT coin new names, and do NOT reuse the L1 ids — an L1
        id here would falsely claim L1 does the grouping, the exact opposite of the epic's point.
  - [x] **Pick trap ids unique across the whole corpus:** `multi-nic-must-merge`,
        `multi-nic-must-not-merge` (`example.toml` uses `example-must-*`, `randomized-mac.toml` uses
        `randomized-mac-must-*`, so no clash; `score_corpus` reds a `TrapId` seen in two files).

- [x] **Task 3 — lock the two new artefacts** `fixtures/MANIFEST.toml` (AC: 5) — deliberate corpus bump
  - [x] Append TWO `[[artefact]]` entries (paths `scenario/replay/multi-nic.jsonl` and
        `scenario/traps/multi-nic.toml`), each with a one-line comment naming the story and what the
        artefact is, mirroring the 4.9 entries' style. Compute each `sha256` from the committed bytes —
        `sha256sum fixtures/scenario/replay/multi-nic.jsonl` and `…/traps/multi-nic.toml` — and paste the
        hex. **Author Task 1/Task 2 FIRST, then hash** — any later byte edit (even a trailing newline)
        invalidates the sha256, which reds the gate with a confusing "edited" finding on a brand-new file.
  - [x] Do NOT touch the seven existing entries or their sha256 (`example.toml`, `randomized-mac.toml`, and
        the five streams stay byte-for-byte — any edit there is an unrelated EDITED finding).

- [x] **Task 4 — update the committed-count assertions** `crates/opencmdb-bin/src/trap_gate.rs` (AC: 7) — three edits, tests only
  - [x] `the_committed_corpus_is_discovered_and_scored_by_nothing` (`:389`): `report.discovered()` `5` → `7`.
        Keep `scored()` and `failures()` at `0` (no producer exists). Update the comment above it that reads
        "…`randomized-mac.toml` (story 4.9) adds two — five in the committed corpus" to name multi-nic's two
        and say **seven**.
  - [x] `the_report_says_plainly_that_nothing_was_scored` (`:407`): `"5 trap(s) discovered"` →
        `"7 trap(s) discovered"`. Leave `"0 scored"` and `"0 truth-table failure(s)"` unchanged.
  - [x] `a_trap_with_no_answer_is_discovered_but_not_scored` (`:425`): `report.discovered()` `5` → `7`
        (still answers ONE trap, so `scored()` stays `1`; the other six are unanswered). **Also fix the
        stale comment inside the SAME test at `:417`** — "so `scored` is 1 while `discovered` stays 5" →
        `stays 7`; editing only the `:425` assertion would leave `:417` lying.
  - [x] **Do NOT touch the scratch-corpus counts** — `each_column_can_be_driven_red`,
        `a_one_sided_family_reddens_the_gate_on_its_own`, and the other scratch tests build their OWN
        temp-dir corpora; their counts are unrelated to the committed root. Changing them would be wrong.
  - [x] **Prove the count coupling is real (record it).** Before adding the family to the manifest, run
        `cargo test -p opencmdb-bin the_committed_corpus_is_discovered_and_scored_by_nothing` with the
        assertions still at `5` and `multi-nic.toml` on disk: it should RED with `left: 7, right: 5` (the
        walk discovers 7), confirming the family is genuinely discovered on the live corpus root. Then
        update to `7`. Record this in the Debug Log — the discovery count IS the guard that the family
        landed. (No new test needed: `discover_trap_files` walks `.toml` under `scenario/traps/`
        automatically.)

- [x] **Task 5 — gates and the corpus lock** (AC: 4, 5, 6, 7)
  - [x] `cargo fmt --all` · `cargo clippy --workspace --all-targets --locked -- -D warnings` ·
        `cargo test --workspace --locked` · `cargo run -p xtask -- ci` — all green.
  - [x] `cargo xtask ci`'s `fixtures` gate must report **9 artefacts, all sha256 match, and no orphan
        finding** (7 existing + 2 new). Confirm the new stream and trap file are BOTH listed and BOTH match.
        `file-size` gate: `.toml`/`.jsonl` are not counted (the gate measures `.rs` before the first
        `#[cfg(test)]`); `trap_gate.rs` grows only by three literals inside `#[cfg(test)]`.
  - [x] Confirm the whole committed corpus still `passed()` at v0.1: `discovered() == 7`, `scored() == 0`,
        `failures() == 0`, `rule_mismatches` empty, `incomplete_families` empty (both the randomized-mac
        AND the multi-nic families carry both poles → complete). If any test that renders the report
        elsewhere hard-codes `5`, catch it here (grep `"5 trap"` / `discovered(), 5` across `crates/`).
  - [x] `Cargo.lock` unchanged (no dependency touched — two data files, one manifest bump, three test
        literals). `architecture-views.md` is NOT regenerated in this story (`ℹ views-hash STALE` is by
        design — regenerate at the Epic-4 milestone, not here). `example.toml`, `randomized-mac.toml` and
        the five existing streams are byte-for-byte unchanged (measured via `git status --short`).

## Dev Notes

### The shape of this story in one paragraph

The multi-NIC family, and — like 4.9 — it is almost entirely DATA. Two new committed artefacts (a
three-observation replay stream and a two-trap family file), a deliberate `MANIFEST.toml` bump to lock
them, and three test-literal updates to keep the committed-corpus count honest (**5 → 7**). There is **no
engine and no new harness code**: at v0.1 the corpus is discovered, parsed and validated but scored by
nothing — the `(verdict, rule)` scoring arrives in Epic 5. The one genuinely NEW thing versus 4.9 is the
LAYER: **this family lives at L2 (device grouping), not L1 (interface identity).** It pins the L2 grouping
decision from both sides — the `must-merge` (primary form) says two distinct-MAC NICs of one host MUST be
grouped (L1 keeps the MACs distinct correctly; L2 does the grouping); the `must-not-merge` (inverse form)
says two NICs on different switches must NOT be grouped. Together they say: **L2 groups by topology, and a
distinct MAC is no reason to split one host into two devices.**

### Which pole is which — do not flip them, and note they are MIRRORED vs story 4.9 (AC1/AC2)

4.9 was an L1 family whose PRIMARY form was `must-not-merge`. 4.10 is an L2 family whose PRIMARY form is
`must-merge`. The epic's wording is exact; hold it:

| Epic phrase | Column | Rule | Judges | The point |
|---|---|---|---|---|
| **primary form** — "two interfaces of one host … L2 must group them; the failure being tested is L2's" | `must-merge` | `l2-uplink-agrees` (the L2 rule that FIRES) | `[M1, M2]` (same switch, different ports) | L2 groups two distinct-MAC NICs of one host; L1 is correct to keep the MACs distinct — the caught failure is a false SPLIT at L2 |
| **inverse form** — "two genuinely different hosts are not grouped" | `must-not-merge` | `l2-different-switch` (the OPPOSING L2 rule) | `[M1, M3]` (different switches) | different-switch topology opposes the join; grouping them would be a false MERGE of two hosts |

D18's three columns (trap.rs:55-76): `must-not-merge` guards the false merge (operator loses trust,
uninstalls), `must-merge` guards **cowardice / the false split** (an engine that refuses to group scores
false-merge = 0 and is demolished by the middle column), `must-abstain` guards the honestly ambiguous
case. This family exercises the first two, at L2. The `must-not-merge` names the rule that OPPOSES the
grouping (`l2-different-switch`), **not** the one that was tempting (trap.rs:69-73).

### Why multi-NIC is an L2 problem, and why the rule ids are L2 (the architecture grounding)

- The identity model splits into two layers (architecture.md:888-895, D12): **L1 = interface identity**
  ("is this NIC the same as yesterday?", main signal MAC, attacked by MAC randomization / cloned MAC /
  Docker veth) and **L2 = device grouping** ("are these interfaces the same host?", main signals
  hostname / topology / DHCP, attacked by **multi-NIC** / shared-hardware VMs / VRRP/HSRP).
- The epic cites the exact line: *"multi-NIC false-split = L1 correct, **L2 failed to group**"*
  (architecture.md:893). So the trap's expectation is about the DEVICE answer (L2), and the rule that
  fires/opposes is an L2 rule — an L1 rule id would be a false claim about which layer decides.
- **The L2 rule ids are ALREADY the corpus's vocabulary — reuse, do not coin.** `score.rs`'s tests use
  `l2-uplink-agrees` as the L2 `Outcome::Merged` rule and `l2-different-switch` as the L2
  `Outcome::Refused` rule (`score.rs:618-627, 704-708, 905-906, 923-924, 942-943`). They are exactly the
  topology-grouping rules multi-NIC exercises. Using them aligns the committed trap corpus with the rule
  names the scoring already models, and sets the same Epic-5 contract consistently.
- **Why NICs, not exotica:** *"a laptop has Wi-Fi + dock Ethernet + a per-SSID randomized MAC. These are
  the median, not the exotic"* (architecture.md:898-899); the reference NAS is `eth0`+`eth1` bonded +
  `docker0` + N `veth` — one device, ten interfaces (architecture.md:896-897). *"Without the split, a
  Wi-Fi+Ethernet laptop appears as two devices. The operator announces 300 hosts, the tool shows 340. He
  does not trust the inventory, therefore he does not trust the gap. The product is dead"*
  (architecture.md:906-908). The multi-NIC false split is the failure that kills trust — this family
  proves the gate can catch it.

### The topology signal is the `Uplink` fact, and "one host" is the author's oracle (AC1/AC6)

The byte-level distinction between the two poles is the `Uplink { peer_mac, peer_port }` fact — the same
switch (identical `peer_mac`) for the one-host pair, a different switch (different `peer_mac`) for the
two-hosts pair. This is the first use of `Uplink` in the committed corpus, so write its shape from the
type (`observation/mod.rs:164-168`); `peer_mac` is a `MacAddr` and serializes as its 6-byte array, and it
is privacy-checked (`fixtures.rs:877` → `assert_synthetic_mac`), so it MUST be locally-administered.

Per D19 and 4.9's precedent, the ground truth — "these two NICs are ONE host" — is the author's ORACLE,
stated in the `reason`, not a fact the bytes alone prove (4.9 accepted the same: "C3 is the SAME physical
interface as C1 … a fact the reason states, not one the bytes can carry"). Shared topology is the
plausible carrier; the reason makes the judgment. No single topology signal is decisive at L2 by design
(D13: L2 is a rule cascade, "floats may RANK, never DECIDE") — which is exactly why the corpus states the
answer and the reason, and Epic 5/6 build the rule that reaches it.

**The two poles are ASYMMETRIC in how strongly the bytes back them, and that asymmetry is deliberate —
name it so a reviewer sees it was a choice, not an oversight.** The `must-not-merge` pole is byte-aligned:
different `peer_mac` = different switch → "different hosts" is directly carried by the bytes (a decisive
negative, like 4.9's poles). The `must-merge` pole is NOT byte-decisive: "same access switch, two ports"
is *also* the ordinary many-hosts-on-one-switch signature, so the "one host" claim rests on the reason
alone. This is not a weakness to hide — it is the very shape of the L2 problem (D12: L2 is "the same
record-linkage problem" as L1 but on the HARD signals; a false split is born precisely because topology
under-determines grouping). The must-merge trap is therefore committed as an explicit author oracle: the
reason asserts a dual-homed single host, and the corpus never commits a same-switch *must-not-merge*, so
nothing here asserts the catastrophic "same switch → merge everything" behaviour. **If a reviewer prefers
this pole reframed as `must-abstain`** (the honestly-ambiguous reading), that is a legitimate call — but
it is D18's *middle-column* case (an engine that abstains on every multi-NIC scores false-merge = 0 and is
demolished by the merge column), so the family is deliberately committed with the merge pole to keep the
gate able to fail an always-abstain engine. Raise it in review if you disagree; do not silently flip it.

### At v0.1 there is no engine — what "scored" means here (AC7)

`score_corpus` (`trap_gate.rs`) discovers `.toml` files, reads and validates each trap through
`read_traps`, and scores each trap **only if `answers` carries an `Outcome` for its `TrapId`**. At v0.1
the answers map is empty for a real run (no producer — Epic 5 builds it), so:

- `discovered()` counts every trap the walk opened → **7** after this story (3 in `example.toml` + 2 in
  `randomized-mac.toml` + 2 in `multi-nic.toml`).
- `scored()` and `failures()` stay **0** for the committed corpus.
- `incomplete_families()` runs answer-INDEPENDENTLY over corpus SHAPE (4.7b): multi-nic carries both
  poles, so it is complete and the bucket stays empty; `passed()` stays green.

The three committed-count assertions (`trap_gate.rs:389`, `:407`, `:425`) are the only tests coupled to
the count — they exist precisely so the zeros stay HONEST. Updating `5 → 7` is mandatory and is itself the
evidence the family was discovered. The scratch-corpus tests build their own temp dirs and must NOT be
touched.

### How a trap is validated (so the family stays green) — the checks the new files must survive

`read_traps` / `Trap::validate` (see 4.9's notes) enforce, per trap:
- **obs_id resolution** — every id in `observations` must exist in the trap's named `replay` stream, or
  `read_traps` fails naming the trap and the missing observation. So `multi-nic.toml`'s
  `replay = "scenario/replay/multi-nic.jsonl"` and every `obs_id` it lists (M1/M2 for must-merge; M1/M3
  for must-not-merge) must be in that committed stream. `read_traps` resolves `replay` against the BAKED
  corpus root, so the stream must be a committed file (it is, once Task 1 lands).
- **`reason` non-empty, 20–300 chars, single line, no control char** (`trap.rs:258-312`). Both prescribed
  reasons are single-line and inside the bound (~200-240 chars); measure and record the exact counts.
- **unique trap ids across files** (`DuplicateTrapId`) — `multi-nic-must-merge` /
  `multi-nic-must-not-merge` are unique in the whole corpus.
- **family present + clean token** (`FamilyEmpty`/`FamilyMalformed`) — `"multi-nic"` is a clean token (an
  internal hyphen is fine; the guard refuses surrounding whitespace and control chars, trap.rs:300-310).
- **family completeness** — `incomplete_families` groups by case-folded family name and requires ≥1
  `must-merge` AND ≥1 `must-not-merge` (DR1: a `must-abstain` satisfies neither pole). Both traps
  declaring `family = "multi-nic"`, one per pole, satisfies this.

And `the_corpus_carries_no_real_network_data` / `every_trap_file_in_the_corpus_is_valid` /
`every_replay_stream_in_the_corpus_is_valid` walk the new files automatically — no harness change; a
malformed file reds the walk by name.

### The corpus lock is a DELIBERATE bump (AC5) — same mechanism as story 4.9

Two real artefacts that MUST be locked. The `fixtures` gate is two-directional (MANIFEST.toml header): an
artefact under `fixtures/` absent from the manifest is an ORPHAN (red); a listed artefact whose bytes
changed is EDITED (red). Adding the two entries with correct sha256 turns both green. This is an "I am
changing the spec" commit by design — the review SHOULD see the manifest move. Order of operations:
**write the data files, THEN hash, THEN paste the sha256.** READMEs stay exempt (do NOT add any README to
the manifest — orphan-exempt by name).

### Project Structure Notes

- **NEW (locked):** `fixtures/scenario/replay/multi-nic.jsonl` (3 observations, first corpus use of the
  `Uplink` fact), `fixtures/scenario/traps/multi-nic.toml` (2 traps, `family = "multi-nic"`). Both listed
  in `fixtures/MANIFEST.toml` with sha256.
- **Updated:** `fixtures/MANIFEST.toml` (two `[[artefact]]` entries — the deliberate bump);
  `crates/opencmdb-bin/src/trap_gate.rs` (three `#[cfg(test)]` count literals, 5 → 7, plus the adjacent
  comments — tests only, no production logic).
- **Unchanged, expected:** `example.toml`, `randomized-mac.toml` and the five existing replay streams
  (byte-for-byte — any edit is an unrelated EDITED finding); `trap.rs` / `score.rs` / `trap_gate.rs`
  production paths (domain and harness frozen since 4.6a/4.7a/4.7b — this story adds no rule, no scoring,
  no engine); all READMEs (orphan-exempt); `Cargo.lock`; every other `.rs`. `discover_trap_files` /
  `incomplete_families` / `read_traps` are used AS-IS.
- **Out of scope, deliberately:** any engine or rule producer (Epic 5); a `must-abstain` third form for
  the family (DR1 optional; the epic asks two poles); the actual L2 grouping algorithm and its blocking
  key (Epic 6+); `docs/project-context.md` and `CLAUDE.md` (a new locked family is milestone/push-level
  under docs-current-before-push — fold in at the Epic-4 milestone with the `architecture-views.md`
  regeneration, not per-story); the shared-hardware-VM family (story 4.11) and every later family.

### Traps (mistakes this story must not make)

1. **Using an L1 rule id.** multi-NIC is an L2 family (architecture.md:893). The rules are
   `l2-uplink-agrees` (must-merge) and `l2-different-switch` (must-not-merge) — established in `score.rs`.
   An `l1-*` id would falsely assert L1 does the grouping, inverting the epic's point.
2. **Flipping the poles.** Primary form = `must-merge` (one host, `l2-uplink-agrees`, `[M1,M2]`); inverse
   form = `must-not-merge` (different hosts, `l2-different-switch`, `[M1,M3]`). This is the MIRROR of 4.9 —
   re-read the epic's two Givens and the table above before writing the `.toml`.
2b. **Silently downgrading the `must-merge` to `must-abstain`.** The `must-merge` pole's "one host" rests
   on the reason, not on byte-decisive topology (unlike the byte-aligned `must-not-merge`) — this is
   deliberate (see the topology Dev Note). The epic asks for a `must-merge`, and D18's middle column needs
   it to keep the gate able to fail an always-abstain engine. If you think it should be `must-abstain`,
   raise it in review — do NOT flip it in the `.toml` on your own.
3. **A real MAC in an `Uplink`.** `Uplink.peer_mac` is privacy-checked too (`fixtures.rs:877`) — it MUST
   be locally-administered, exactly like a `Mac` fact. Byte 0 = `0x02`.
4. **Forgetting the manifest bump.** The two new files are LOCKED artefacts. Both need a `[[artefact]]`
   entry with the correct sha256, or the `fixtures` gate reds as an orphan.
5. **Hashing before the final byte.** Compute sha256 only after the files are final; a later trailing
   newline invalidates it.
6. **Leaving the count assertions at 5.** Three committed-corpus tests hard-code `5`; the family makes it
   `7`. Update `trap_gate.rs:389/:407/:425` (and their comments) — and do NOT touch the scratch counts.
7. **Recording a real hostname/IP.** Synthetic-only: locally-administered MACs, RFC 5737 IPs, no captured
   hostname. The privacy walk enforces it; a real capture is disqualifying (D19;
   [[no-private-data-in-artifacts]]).
8. **Reusing an existing stream's `obs_id` prefix.** New stream → fresh ids (`ffffffff-…`); aaaa–eeee are
   taken by the five committed streams. (`ffffffff` at fixtures.rs:1649 is a scratch-test literal, not a
   committed stream — no collision.)
9. **Coining a new `rule` id.** Use `l2-uplink-agrees` / `l2-different-switch` — the corpus's established
   L2 ids; Epic 5 will close `RuleId` into an enum and an invented id becomes an unhonourable
   `Other(String)`.
10. **Inconsistent `connector_id`/`scope` within the stream.** All three observations share one
    `connector_id`, `l2_domain` and `vantage` (the within-stream provenance/scope checks, 4.5a/4.5b).
11. **Adding a third `must-abstain` "to be safe".** Not asked, not needed for completeness (DR1), and it
    widens a deliberately small story. The ambiguous topology case goes to the reality-debt register.
12. **Regenerating `architecture-views.md`.** Its `ℹ views-hash STALE` is by design — Epic-4 milestone,
    not this story.
13. **Claiming more than measured.** Name the command behind every count; record the discovery-count
    coupling proved (7 discovered). Write the weaker true sentence. [[claims-must-match-verification]]

### Latest technical specifics

No new crate, no version bump, no domain or harness code. Rust 1.96+, edition 2024. Two data files (one
`.jsonl` — first corpus use of the `Uplink` fact — one `.toml`), one `MANIFEST.toml` bump, three
`#[cfg(test)]` literals in `opencmdb-bin`. **Never invent a version — pin from the committed `Cargo.lock`,
which does not move here.**

### References

- [Source: _bmad-output/planning-artifacts/epics.md:1107-1119 — Story 4.10 "Trap family — multi-NIC": the
  two Givens (two interfaces of one host → L1 correct, L2 must group, the failure is L2's; inverse → two
  genuinely different hosts not grouped) and the architecture.md:893 citation]
- [Source: _bmad-output/planning-artifacts/architecture.md:879-927 — D12 the three-level model
  `observation → interface → device`; the L1/L2 table (:888-891), "multi-NIC false-split = L1 correct, L2
  failed to group" (:893), "median not exotic" (:898-901), "the product is dead" economic test (:906-908)]
- [Source: _bmad-output/planning-artifacts/architecture.md:929-1002 — D13: L1 = pure A deterministic join;
  L2 = the FORM of C with A's decision function, "named rules"; "floats may RANK, never DECIDE"; structural
  facts never scored]
- [Source: crates/opencmdb-core/src/score.rs:618-627, 704-710, 905-906, 923-943 — the L2 rule vocabulary
  this family reuses: `l2-uplink-agrees` (the merge rule) and `l2-different-switch` (the opposing rule)]
- [Source: crates/opencmdb-core/src/observation/mod.rs:146-173 — the `Fact` enum; `Uplink { peer_mac:
  MacAddr, peer_port: String }` (:164-168), the topology signal this family carries; `MacAddr` serializes
  as a 6-byte array]
- [Source: crates/opencmdb-core/src/trap.rs:31-124, 258-390 — `RuleId`/`TrapId`/`FamilyId`, the
  `Expectation` enum and its columns (the `must-not-merge` "name the OPPOSING rule" contract, :69-73),
  `Trap`/`Trap::validate`, and `incomplete_families` (the family-completeness check)]
- [Source: crates/opencmdb-bin/src/fixtures.rs:820-913 — `the_corpus_carries_no_real_network_data` /
  `assert_facts_are_synthetic` (the `Uplink.peer_mac` privacy check at :877) / `assert_synthetic_mac`
  (U/L-bit requirement, :906-913) / `assert_documentation_ip`; :1306 the cross-stream obs_id guard]
- [Source: crates/opencmdb-bin/src/trap_gate.rs:382-428 — the three committed-count assertions (5 → 7);
  :223-289 `score_corpus`: discovery, per-trap validation, answer-independent `incomplete_families`]
- [Source: fixtures/scenario/traps/randomized-mac.toml + fixtures/scenario/replay/randomized-mac.jsonl —
  story 4.9, the immediately prior family and the exact model this story follows (three observations, two
  poles, fresh obs_id block, manifest bump); note the CONTRAST: 4.9 is L1 with primary = must-not-merge,
  4.10 is L2 with primary = must-merge]
- [Source: _bmad-output/implementation-artifacts/4-7b-trap-corpus-positive-and-negative.md:235-253 — DR1
  (a `must-abstain` satisfies neither pole; a family is complete iff it has both `must-merge` AND
  `must-not-merge`) and DR2; the completeness contract this family satisfies]

### Review Findings

Code review 2026-07-24 (three parallel layers: Blind Hunter / Edge Case Hunter / Acceptance Auditor).
Outcome: **0 decision-needed, 0 patch, 1 defer, 2 dismissed.** Acceptance Auditor: all 7 AC satisfied,
no defect. Both context-aware layers (edge, auditor) recomputed the two sha256 and confirmed they match.

- [x] [Review][Defer] New replay stream's serde byte-shape is not pinned by a round-trip test [crates/opencmdb-bin/src/fixtures.rs] — deferred, pre-existing. `re_serializing_reproduces_the_committed_bytes` covers only `minimal.jsonl`, so no committed stream but `minimal.jsonl` (incl. `randomized-mac.jsonl`, `example-traps.jsonl`) has its serde byte-shape pinned by round-trip; the new stream is still gated for *parseability* by `every_replay_stream_in_the_corpus_is_valid`. Not caused by this change; corpus-wide. Recorded in `deferred-work.md`.
- Dismissed — `Uplink.peer_mac` is locally-administered (Blind Hunter, no-context realism nit): this is MANDATORY, not a defect. `assert_facts_are_synthetic` runs `assert_synthetic_mac` on `Fact::Uplink { peer_mac, .. }` (fixtures.rs:877), so the corpus privacy floor forbids a real vendor OUI there regardless of real-world plausibility (AC6; story Trap #3). Both context-aware layers confirmed it correct.
- Dismissed — "double trailing newline in `MANIFEST.toml`" (Blind Hunter): false positive. The last bytes are `…a7be"\n\n`, byte-for-byte the same trailing-blank convention as the committed HEAD file (each `[[artefact]]` block separated by one blank line, one blank line at EOF). No change introduced.

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (Claude Opus 4.8, 1M context) — bmad-dev-story workflow.

### Debug Log References

- **Prove-to-red (the discovery-count coupling is real).** With `multi-nic.jsonl` + `multi-nic.toml` on
  disk and the three committed-count assertions still at `5`, `cargo test -p opencmdb-bin --locked
  the_committed_corpus_is_discovered_and_scored_by_nothing` RED with `assertion left == right failed …
  left: 7, right: 5` at `trap_gate.rs:389`. The walk discovered 7 traps on the live corpus root, and
  `score_corpus` returned `Ok` (no `.expect` panic) — proving the two new traps parse, validate,
  resolve their obs_ids against the committed stream, and survive `Trap::validate` before the count is
  even checked. Assertions then updated `5 → 7`; test green.
- **Measured `reason` char counts** (bound 20–300, single line, no control char): `multi-nic-must-merge`
  = **231**, `multi-nic-must-not-merge` = **222**. Both inside the bound.
- **sha256, computed AFTER the files were final** (via `sha256sum`, then pasted into `MANIFEST.toml`):
  - `scenario/replay/multi-nic.jsonl` = `c5fc7dd5d41c1491e6c8d44ee11652b853f997eca3c5b1099ecff34b9bffdeeb`
  - `scenario/traps/multi-nic.toml`   = `b7f5f03f31ba2d607e0d15487babaf55c2fd0bc7f0a3350ee7f47e961515a7be`
- **Gate suite (all green).** `cargo fmt --all` (no changes); `cargo clippy --workspace --all-targets
  --locked -- -D warnings` (clean); `cargo test --workspace --locked` (all pass, incl. the corpus walks
  `every_trap_file_in_the_corpus_is_valid`, `every_replay_stream_in_the_corpus_is_valid`,
  `the_corpus_carries_no_real_network_data`); `cargo run -p xtask --locked -- ci` — `fixtures` reports
  **9 fixture(s) match their recorded sha256 (0 generated, 9 hand-authored)**, no orphan/edited finding;
  `file-size` largest 884 (well under 2000); `views-hash STALE` is the by-design ℹ (exit 0). `Cargo.lock`
  untouched; `example.toml`, `randomized-mac.toml` and the five existing streams byte-for-byte unchanged
  (`git status --short`).

### Completion Notes List

- Implemented the multi-NIC trap family — the first **L2** (device-grouping) family, mirroring 4.9's L1
  shape but with the poles flipped (primary = `must-merge`).
- **AC1** — `multi-nic-must-merge` judges `[M1, M2]` (two distinct-MAC NICs of one host, same access
  switch = identical `Uplink.peer_mac`, different ports), naming the L2 rule that FIRES,
  `l2-uplink-agrees`. The caught failure is L2's false split; L1 is correct to keep the MACs distinct.
- **AC2** — `multi-nic-must-not-merge` judges `[M1, M3]` (different access switch = different
  `Uplink.peer_mac`), naming the OPPOSING L2 rule `l2-different-switch` (a decisive negative, not an
  absence of support).
- **AC3** — both traps carry the mandatory one-sentence `reason` (231 / 222 chars) stating the oracle in
  the family's own vocabulary (two NICs of one host / two hosts on different switches; an L2 decision,
  L1 left correct).
- **AC4** — both declare `family = "multi-nic"`, one per pole → `incomplete_families` sees both decision
  poles present; the gate stays green.
- **AC5** — two NEW locked artefacts (`multi-nic.jsonl`, `multi-nic.toml`) added to `MANIFEST.toml` with
  sha256 in the same change; `fixtures` gate now 9 artefacts, all match, no orphan.
- **AC6** — synthetic-only: every MAC (host AND `Uplink.peer_mac`) locally-administered (byte 0 = `0x02`,
  `locally_administered: true`), every IP in RFC 5737 `192.0.2.0/24`, no hostname; the privacy walk
  passes over the new stream unchanged. First corpus use of the `Uplink` fact.
- **AC7** — the three committed-count assertions updated `5 → 7` (`trap_gate.rs:389`, the render string
  at `:407`, `:425`) plus the adjacent/inner comments (`:387-389`, `:417`); scratch-corpus counts left
  untouched; `scored()`/`failures()` stay 0 (no answer producer exists at v0.1 — Epic 5).
- Rule ids `l2-uplink-agrees` / `l2-different-switch` were REUSED verbatim from `score.rs`'s L2
  vocabulary (grep-verified), not coined; no L1 id used. The poles were NOT flipped and the `must-merge`
  was NOT silently downgraded to `must-abstain` (the deliberate byte-asymmetry is documented in the
  story's Dev Notes and left as an explicit author oracle, per the epic).
- No engine, no harness code, no new dependency: two data files, one manifest bump, three `#[cfg(test)]`
  literals. `architecture-views.md` deliberately NOT regenerated (Epic-4 milestone task).

### File List

- `fixtures/scenario/replay/multi-nic.jsonl` (new, locked) — three synthetic observations, first corpus use of `Uplink`
- `fixtures/scenario/traps/multi-nic.toml` (new, locked) — the two-pole multi-NIC family
- `fixtures/MANIFEST.toml` (modified) — two `[[artefact]]` entries with sha256 (deliberate bump)
- `crates/opencmdb-bin/src/trap_gate.rs` (modified) — three `#[cfg(test)]` count literals 5 → 7 (+ comments)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (modified) — status transitions

## Change Log

| Date       | Change                                                                 |
|------------|------------------------------------------------------------------------|
| 2026-07-24 | Story 4.10 drafted (create-story): multi-NIC trap family — L2 device grouping, poles must-merge/`l2-uplink-agrees` + must-not-merge/`l2-different-switch`, first corpus `Uplink` fact, committed-count 5 → 7. Status → ready-for-dev. |
| 2026-07-24 | Validated (two fresh-context agents: fact-check + gap-hunt). Fact-check clean on all load-bearing claims; fixed 3 LOW citation slips (4.9 quote "them"; architecture.md:896-897/898-899). Gap-hunt: made the `must-merge` pole's non-byte-decisive oracle explicit (asymmetry named, must-abstain reframing acknowledged, D18 middle-column rationale recorded); added `observed_at` guidance, an L2-rule-id copy-paste caution (no `.toml` exemplar), and named the stale comment at trap_gate.rs:417. |
| 2026-07-24 | Implemented (dev-story): wrote `multi-nic.jsonl` (3 obs, first corpus `Uplink` fact) + `multi-nic.toml` (2 poles, `l2-uplink-agrees`/`l2-different-switch`); prove-to-red confirmed `left: 7, right: 5`; updated the 3 committed-count assertions 5 → 7; locked both artefacts in `MANIFEST.toml` (sha256). All gates green (`fixtures` 9 match, no orphan; clippy clean; full test pass). Status → review. |
| 2026-07-24 | Code review (3-layer: Blind Hunter / Edge Case Hunter / Acceptance Auditor). Auditor: all 7 AC satisfied, no defect; both context-aware layers recomputed the two sha256 and confirmed match. 0 decision-needed, 0 patch, 1 defer (new stream's serde byte-shape not round-trip-pinned — pre-existing, corpus-wide → `deferred-work.md`), 2 dismissed (locally-administered `Uplink.peer_mac` is mandatory not a nit; "double trailing newline" false positive — bytes match HEAD). Status → done. |
