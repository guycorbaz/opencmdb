# Story 4.16: Trap family — ephemeral Docker veth

Status: done

## Story

As the author of the trap corpus,
I want ephemeral container interfaces committed as traps,
so that **short-lived interfaces do not inflate the inventory or the gap** (epic 4.16): a veth is
a real interface of its docker host while it lives — grouping it is correct, not optional — and
when it vanishes it is **gone** (dormant-bound, still queryable), never "never a device"; a
successor veth wearing the recycled container address is a NEW interface, not a re-sighting.

## Acceptance Criteria

1. **The grouping form — a living veth belongs to its host.** **Given** the docker host E1
   (stable MAC `[2,0,94,0,83,180]`, `192.0.2.60`, `doc-dockerhost`, uplink switch
   `[2,0,94,0,96,10]` port `swport-21`) and its container's veth E2 (fresh MAC
   `[2,0,94,0,83,181]`, container address `192.0.2.61`, NO hostname, the SAME uplink —
   containers exit through the host's port) **when** the trap is scored, **then** it is a
   **`must-merge`** naming **`l2-uplink-agrees`** (REUSED — multi-nic's firing rule: distinct
   MACs, agreeing uplink; NOTE that multi-nic's committed pair agrees across DIFFERENT ports of
   one switch, while here even the PORT agrees — the header must defend this as the honest veth
   form, bridged traffic exiting through the host's own port: a STRONGER agreement, not a
   different kind): ephemerality is no licence to refuse the grouping
   while the interface lives (the reference NAS *is* `eth0`+`eth1` + `docker0` + N `veth`,
   architecture.md:898). **Byte-kinship, held in mutual check and said in the header:** this
   pair is byte-kin to vrrp-virtual-mac's PRIMARY (`[V1, A]` — shared uplink, must-NOT-merge);
   the distinguisher is structural — the VRRP MAC carries the IANA virtual-router prefix
   (disqualified as anchor, with its entailed `locally_administered: false` flag — the only
   byte-difference in the pair shape), the veth MAC is an ordinary locally-administered address with
   nothing opposing. An engine that generalizes 4.14's refusal to every shared-uplink pair is
   demolished here.

2. **The succession form — "gone", not "never a device" (epic AC1).** **Given** the first veth
   E2 and its successor E4 — container restarted: NEW MAC `[2,0,94,0,83,182]`, the RECYCLED
   container address `192.0.2.61` (byte-identical `IpV4`), the same uplink, one hour later —
   **when** the trap is scored, **then** it is a **`must-not-merge`** naming
   **`l1-distinct-mac`** (REUSED — the rule that OPPOSES): the recycled address and the shared
   uplink tempt continuity, the distinct MACs oppose it. **The reason draws the epic's exact
   distinction in its one sentence:** the first veth is **gone** — a real interface,
   dormant-bound, still queryable — not "never a device"; and the successor is a new
   interface, not its re-sighting. (Byte-kinship with dhcp-churn's must-not-merge — a recycled
   address across distinct MACs — acknowledged in the header: there the temptation was
   IP-continuity across two boxes, here it is container-slot continuity across one host's
   ephemeral interfaces.) **And the header MUST scope this refusal to L1 (validation's HIGH):**
   both veths legitimately GROUP into the host's device at L2 — the must-merge itself mandates
   it — so this must-not-merge speaks at INTERFACE level only (E2 and E4 are not one interface
   re-seen); the L2 grouping of both veths into `doc-dockerhost` remains correct and is
   deliberately NOT asserted by any trap. This is multi-nic's own established duality ("L1 is
   right to keep their distinct MACs apart while L2 groups them"); without the sentence, the
   family reads as self-contradictory the day Epic 5 groups both veths into the host.

3. **The F17 record — consistency with the dormant lifecycle (epic AC2).** The family header
   records, with citations: a locally-administered MAC unobserved for the configured window
   becomes **`dormant`** — excluded from gap metrics and automatic candidate generation, still
   queryable, `first_seen`/`last_seen` and IP history retained, **"returning to `active` if the
   MAC is re-observed"** (F17 verbatim, architecture.md:1485) — with **"same entity"** added by
   D17's property 1 (:1186-1187), NOT by F17; attribute each phrase to its own source (D17's
   rule at :1177-1178); only
   `mac_kind='local'` goes dormant (a universal MAC's absence is information — D17 property 2);
   left `active`, dead veths make the gap drift "monotonically upward and never comes back
   down. Within a year, the product's central indicator is noise" (D17 verbatim, :1164-1167). The header states plainly that the
   TRAPS assert identity columns and the LIFECYCLE is the engine's (Epic 5+) — the family is
   *consistent with* dormant, it does not test the sweep (no trap can: the corpus has no clock
   and no engine, D19).

4. **The stream is committed as 4 observations** —
   `fixtures/scenario/replay/docker-veth.jsonl`, fresh prefix `babababa` (first full id
   `babababa-0000-4000-8000-000000000001`, then `…0002`/`…0003`/`…0004`), the corpus UUIDs
   (`connector_id` `33333333-3333-4333-8333-333333333333`, `l2_domain`
   `11111111-1111-4111-8111-111111111111`, `vantage` `22222222-2222-4222-8222-222222222222` —
   ONE l2_domain, the premise L1's determinism and the must-merge both rest on), `raw:
   null`, instants `2026-01-10T00:00:00Z` / `T00:05:00Z` / `T01:00:00Z` / `T01:05:00Z`:
   - **E1** (`…0001`, 00:00): host — `Mac [2,0,94,0,83,180]` flag `true` · `IpV4 192.0.2.60` ·
     `Hostname doc-dockerhost` (`"source":"Dhcp"`) · `Uplink` peer `[2,0,94,0,96,10]` port
     `swport-21` — 4 facts.
   - **E2** (`…0002`, 00:05): veth-1 — `Mac [2,0,94,0,83,181]` flag `true` · `IpV4 192.0.2.61`
     · `Uplink` same peer/port — 3 facts, no hostname (a veth answers ARP, it resolves no
     name).
   - **E3** (`…0003`, 01:00): the host RE-SEEN — byte-identical facts to E1 (Mac, IpV4,
     Hostname, Uplink), only `obs_id`/`observed_at` differ — 4 facts. **E3 feeds no trap,
     deliberately, and the header says why:** it is the authored evidence that the window
     stayed open and veth-1 FAILED to reappear — without it, "disappeared within the
     observation window" would be an assertion about bytes the corpus does not carry (the
     stream would be equally consistent with the scan simply stopping at 00:05). NFR7 forbids
     an absence fact; the absence is expressed the only honest way — a later sweep that sees
     the host and not the veth.
   - **E4** (`…0004`, 01:05): veth-2 — `Mac [2,0,94,0,83,182]` flag `true` · `IpV4
     192.0.2.61` byte-identical to E2's · `Uplink` same peer/port — 3 facts.
   A byte-pin test in `fixtures.rs` (all review lessons pre-applied): 4 obs; fact counts
   4/3/4/3; **the four obs_ids pinned** (the 4.15 review's binding rule); E1's four facts
   value-pinned; E3 asserted byte-identical to E1 fact-by-fact (Mac, IpV4, Hostname, Uplink);
   E2's Mac/IpV4 value-pinned; E4's Mac value-pinned + `assert_ne` vs E2's, E4's IpV4 asserted
   EQUAL to E2's; the three trap-judged uplinks (E1, E2, E4) asserted equal and E1's
   value-pinned (`swport-21`) — E3's uplink is covered by the E3 == E1 fact-by-fact assertion,
   so all four are locked; flags true on all; the instants VECTOR asserted equal to the four
   authored values via `ts()`.

5. **The two traps are committed with both poles present.**
   `fixtures/scenario/traps/docker-veth.toml`, `family = "docker-veth"` on both (ids
   `docker-veth-must-merge` judging `[…0001, …0002]`, `docker-veth-must-not-merge` judging
   `[…0002, …0004]`), multi-line `observations`, full UUIDs, every id whole on its line. No
   `must-abstain` — nothing here is ambiguous: the living veth's grouping is a topology fact,
   the succession's split is L1's determinism (state it in the header; the D16 semantic-dustbin
   citation from 4.14 does not need repeating — one sentence suffices).

6. **Reasons**: mandatory one-sentence, 20–300 chars. Precision (validation): the review-held
   rule is **no raw MAC octets**; the recycled address is named DESCRIPTIVELY ("the recycled
   container address" — 4.15's practice), spec ids (F17/D17) are HEADER material and never
   appear in a reason — the reason says at most "dormant, still queryable". The must-merge
   reason names the shared uplink and the absence of any opposing signal; the must-not-merge
   reason names the recycled address, the distinct MACs, AND the epic's
   gone-not-never-a-device distinction (this is the sentence epic AC1 demands). A validated
   draft measuring 269 chars exists; keep margin by that shape.

7. **Corpus lock + count coupling, red first.** Manifest 19 → 21 (gate message `"21 fixture(s)
   match their recorded sha256 (0 generated, 21 hand-authored)"`), sha256 after final byte.
   `trap_gate.rs` count literals **19 → 21** at the three sites (:392, with the breakdown
   comment ABOVE it at :387-391 — its tail "…two — nineteen…" REPLACED by "…two,
   `docker-veth.toml` (story 4.16) two — twenty-one…";
   :410 `"21 trap(s) discovered"`; :428 + comment :420 "stays 21"); red observed at
   `left: 21, right: 19` first. Reproducibility and scratch tests untouched. Mid-story orphan
   red on the fixtures gate expected until the manifest bump.

8. **Synthetic-only, fresh, verified**: `babababa`, `doc-dockerhost`, MAC last-bytes
   `83,180/181/182`, IPs `.60`/`.61`, port `swport-21` — all grep-free at story creation;
   re-verify at dev. The established switch `[2,0,94,0,96,10]` is reused. Privacy walks
   (including 4.14's flag guard) pass unchanged.

## Tasks / Subtasks

- [x] **Task 1 — byte-pin test, RED** (AC: 4): e.g.
      `the_docker_veth_stream_replaces_its_veth_without_replacing_its_host`, end of
      fixtures.rs test module; red = `FixtureError::Io`. Record.
- [x] **Task 2 — the stream** (AC: 4, 8): four lines per AC4. **Template for E1/E3 (4 facts,
      Mac+IpV4+Hostname+Uplink in that order): the `aeaeaeae-…-0002` line of
      `vrrp-virtual-mac.jsonl` — the exact combination already exists there** (validation's
      pointer; no multi-nic+dhcp-churn stitching needed). E2/E4 are Mac+IpV4+Uplink (drop the
      Hostname from the same template). Re-verify frees. Trailing newline. Byte-pin greens.
- [x] **Task 3 — the trap file** (AC: 1, 2, 3, 5, 6): header per AC3 + the two kinship records
      (vrrp mutual check, dhcp-churn recycled-address kin) + E3's why + the one-sentence
      no-abstain; two `[[trap]]` blocks; reasons measured and recorded.
- [x] **Task 4 — count red, 19 → 21** (AC: 7): red recorded, three literals + two comments.
- [x] **Task 5 — manifest bump** (AC: 7): two entries, hash after final byte, nineteen
      existing entries untouched.
- [x] **Task 6 — gates** (AC: 7, 8): fmt · clippy · test --workspace · xtask ci (quote the
      21/21 message). Residual grep `"19 trap"` / `discovered(), 19` / `stays 19` / `nineteen`
      → no hits. `Cargo.lock` unchanged; `architecture-views.md` and `traps/README.md`
      untouched.

### Review Findings

- [x] [Review][Patch] `Expectation::MustNotMerge`'s doc comment (opencmdb-core, trap.rs) said
      "These observations describe different devices" — now contradicted by this family's
      committed L1-scoped refusal (both members share ONE device). Doc widened to the true
      sentence: the refusal is scoped by the NAMED RULE's level (`l1-*` interface, `l2-*`
      device), with docker-veth cited as the committed counter-example (Edge Case Hunter #1 —
      a DOC-ONLY change to a production file; no logic touched; a false doc is a defect)
- [x] [Review][Patch] Change Log lacked the "Implemented (dev-story)" row — the same finding as
      4.15's review; added, and the lesson is now written into the dev checklist of the NEXT
      story to break the recidive (Auditor #1)
- [x] [Review][Patch] MANIFEST comment mis-attributed "gone-not-never-a-device" to the
      SUCCESSOR veth (it is the predecessor's predicate); reworded — MANIFEST comments are not
      hashed, no re-hash (Blind Hunter #1)
- [x] [Review][Patch] The byte-pin's doc claimed "byte-identical fact-by-fact" — the test pins
      parsed VALUES (order and raw bytes are the corpus lock's business); weakened to
      "value-identical" in both doc and inner comment (Blind Hunter #2)
- [x] [Review][Defer] `Observation.raw` is scanned by no privacy rule (minimal.jsonl's third
      observation already carries uninspected prose) — pre-existing since 4.1 → registered in
      deferred-work.md under "code review of story-4.16" (Edge Case Hunter #2)

## Dev Notes

### The shape of this story in one paragraph

A routine two-trap family (pure data + count literals + one byte-pin test, nothing coined, no
harness change) whose content is anything but routine: it is the corpus's third
mutual-check pairing — its must-merge is the shared-uplink shape 4.14 REFUSES (the IANA prefix
is the only difference), its must-not-merge is the recycled-address shape 4.13 refused
(container-slot continuity instead of DHCP-lease continuity) — and the first family whose
header speaks for the dormant lifecycle (F17/D17) while honestly stating no trap can test a
sweep the engine does not yet run.

### Why E3 exists and feeds no trap

The epic's premise is "appears and disappears within the observation window". NFR7/D35 forbid
an absence observation, so disappearance can only be authored as: the window demonstrably
stayed open (the host re-seen at 01:00) while the veth failed to reappear. Without E3 the
stream is equally consistent with "the scan stopped at 00:05" and the family's premise rests
on prose. An unreferenced observation is legal (`read_traps` resolves trap→obs references,
nothing requires the converse; `minimal.jsonl` already carries two unreferenced observations
and two whole streams carry no trap at all — legality has precedent) — this is the first
deliberate use of an unreferenced observation AS AUTHORED EVIDENCE; defend it in the header,
cite the minimal.jsonl precedent there, and the byte-pin pins E3 == E1 so it cannot drift
into a third device.

### Rule vocabulary — nothing is coined

`l2-uplink-agrees` fires the must-merge (multi-nic's own firing shape — distinct MACs,
agreeing uplink; here even the port agrees). `l1-distinct-mac` opposes the must-not-merge
(randomized-mac/dhcp-churn's opposer). The dormant lifecycle needs NO rule id here: dormancy
is a state transition the ENGINE applies on a clock the corpus does not have — a trap
asserting it would be a trap asserting the untestable.

### The three-way mutual check (say it in the header)

| Family | Pair shape | Column | Why |
|---|---|---|---|
| vrrp-virtual-mac (4.14) | shared uplink, IANA virtual MAC | must-NOT-merge | structural prefix disqualifies |
| **docker-veth (this)** | shared uplink, ordinary local MAC | **must-merge** | nothing opposes; the NAS IS its veths |
| dhcp-churn (4.13) | recycled address, distinct MACs, two boxes | must-not-merge | a lease moves between devices |
| **docker-veth (this)** | recycled address, distinct MACs, one host's slots | **must-not-merge** | same verdict, different temptation: slot continuity |

An engine passing 4.14 by "shared uplink + weird MAC → refuse" fails here; an engine passing
this family's must-merge by "shared uplink → group" fails 4.14. That is the corpus doing its
job.

### Previous story intelligence (4.15)

- The obs_id ↔ line binding is pinned in the byte-pin from the start (4.15's review patch,
  now standing practice; the 4.13/4.14 back-fill is a registered defer — do NOT fix it here).
- Value-pin everything a reason cites; instants as a VECTOR; `facts.len()` with its
  justification comment.
- Reasons: vrrp template (octet-free); margins healthier than 300-ε (4.15 landed 255/246).
- Blind Hunter's environment note: on this Synology mount, `cargo test` can run a STALE
  binary — if a count assertion greens when it should red (or vice versa), `touch` the edited
  files or `cargo clean -p opencmdb-bin` before concluding. [[local-gate-must-mirror-ci]]
- PR workflow: branch → PR → CI → squash merge, gh's default subject suffix.

### Project Structure Notes

- **NEW (locked):** `fixtures/scenario/replay/docker-veth.jsonl` (4 obs),
  `fixtures/scenario/traps/docker-veth.toml` (2 traps). Manifest 19 → 21.
- **Updated:** `fixtures.rs` (one byte-pin test), `trap_gate.rs` (three literals + two
  comments), `MANIFEST.toml`.
- **Unchanged:** production paths, `Cargo.lock`, nineteen existing manifest entries,
  `traps/README.md`, `architecture-views.md`, privacy helpers.
- **Out of scope:** the dormant sweep and its config constraint (engine/Epic 5+; FR38b);
  issue #1's Docker connector (Growth); any docker0/bridge modelling; the hostname-absent
  family (4.17 — E2/E4's hostname-lessness here is scenery, not the tested signal — say so
  in the header to keep the families' scopes clean).

### Traps (mistakes this story must not make)

1. **Giving E2 or E4 a hostname** — it would blur into 4.17's territory and weaken "a veth
   answers ARP, it resolves no name".
2. **Giving E4 a different IP or uplink** — the recycled address and stable uplink ARE the
   temptation; without them the must-not-merge is trivial.
3. **Coining a dormancy/lifecycle rule id, or a must-abstain pole.**
4. **Dropping E3, or letting it drift from E1** (byte-pin pins the identity).
5. **A reason that AFFIRMS deletion or inexistence** ("deleted", "never existed" as claims) —
   DENYING them is the required gesture: "gone, not never-a-device" is exactly the sentence
   epic AC1 demands; F17's whole point is the row survives. (Validation: this trap and AC6 are
   complementary, not contradictory — forbid the affirmation, require the negation.)
6. **Naming `l2-uplink-agrees` in the must-not-merge or `l1-distinct-mac` in the must-merge**
   — each pole names its own rule (FIRES / OPPOSES).
7. **Forgetting the obs_id pins, the E3==E1 pins, or hashing before the final byte.**
8. **Claiming the family tests dormancy.** It is CONSISTENT with F17; the sweep is untestable
   here. Write the weaker true sentence. [[claims-must-match-verification]]

### References

- [Source: _bmad-output/planning-artifacts/epics.md:1191-1203 — Story 4.16 and its two ACs
  (gone vs never-a-device in one sentence; consistency with F17)]
- [Source: _bmad-output/planning-artifacts/architecture.md:1485 — F17 verbatim: the dormant
  FR (window, exclusion from gap metrics, still queryable, return-to-active on re-observation,
  human-affirmed exemption, config constraint — "same entity" is D17 property 1's phrase at
  :1186-1187, not F17's)]
- [Source: _bmad-output/planning-artifacts/architecture.md:1152-1206 — D17 in full: the
  monotonic-drift correctness argument (:1164-1167), the rule (:1177-1178), the three
  properties (reversible; only local; human affirmation never auto-dormant), the
  window<retention startup failure]
- [Source: _bmad-output/planning-artifacts/architecture.md:890, :895-898 — L1's Docker-veth
  row; "MAC randomization = 1 device, N ephemeral interfaces = both"; the reference NAS =
  eth0+eth1+docker0+N veth]
- [Source: fixtures/scenario/traps/multi-nic.toml — `l2-uplink-agrees`'s firing shape this
  must-merge reuses]
- [Source: fixtures/scenario/traps/vrrp-virtual-mac.toml — the shared-uplink must-NOT-merge
  this family holds in mutual check (the IANA prefix is the only byte-difference in shape)]
- [Source: fixtures/scenario/traps/dhcp-churn.toml — the recycled-address must-not-merge kin]
- [Source: crates/opencmdb-bin/src/trap_gate.rs:392/:410/:428 + comments :387-391/:420 — the
  count literals 19 → 21]
- [Source: _bmad-output/implementation-artifacts/4-15-trap-hostname-collision.md — prior
  story: the obs_id-binding rule, the stale-binary environment note]

## Dev Agent Record

### Agent Model Used

claude-fable-5 (Claude Fable 5)

### Debug Log References

- **Task 1 natural RED**: `cargo test -p opencmdb-bin --locked the_docker_veth_stream…` →
  panicked with `FixtureError::Io { path: ".../fixtures/scenario/replay/docker-veth.jsonl", source: Os { code: 2, kind: NotFound } }`
  inside `read_jsonl`, as predicted.
- **Byte-pin GREEN** after the stream landed (1 passed).
- **Reason lengths** (awk length on committed bytes): must-merge **269**, must-not-merge
  **269** — inside 20–300, single line, no MAC octets, no spec ids (the validated draft shape).
- **Count-coupling RED**: with both files on disk and assertions at 19,
  `the_committed_corpus_is_discovered_and_scored_by_nothing` → `left: 21 / right: 19`. Then
  19 → 21 at the three sites + the two comments (breakdown tail REPLACED — "…two and
  `docker-veth.toml` (story 4.16) two — twenty-one"), green.
- **`babababa` grep**: only the two new files under fixtures/. Frees re-verified
  pre-authoring (`doc-dockerhost`, `83,180/181/182`, `.60`/`.61`, `swport-21` — no hits).
- **Hash-after-final-byte held**: trailing newlines `0a` confirmed, then `sha256sum`:
  `9cf1569e0711948ad0da324303091994ba250031a5d978a3075c9126c57cb09a` (jsonl),
  `2c2a5407d1a7aeb98dbd0b9340fd8c403f46f9b43f9b9d523015bb153b3782b8` (toml). No edit after
  hashing.
- **Gates** (all green): fmt · clippy `--all-targets --locked -- -D warnings` ·
  `cargo test --workspace --locked` → **116 (bin) + 86 (core) + 42 (xtask), 0 failed** ·
  `cargo run -p xtask --locked -- ci` → fixtures verbatim **"21 fixture(s) match their
  recorded sha256 (0 generated, 21 hand-authored)"**, no orphan; views-hash `ℹ STALE` by
  design. Residual grep (`"19 trap"` / `discovered(), 19` / `stays 19` / `nineteen`) → no
  hits. `Cargo.lock` untouched.

### Completion Notes List

- The family landed exactly as scoped: two NEW locked artefacts (4-obs stream, 2-trap file),
  manifest 19 → 21, three count literals 19 → 21, ONE byte-pin test. Nothing coined, no
  harness change.
- **AC1**: `docker-veth-must-merge` [E1,E2] fires `l2-uplink-agrees`; the header defends the
  same-port agreement as the honest veth form (stronger than multi-nic's different-port
  exemplar, not different in kind) and carries the vrrp mutual check (the IANA prefix and its
  entailed flag = the only pair-shape difference).
- **AC2**: `docker-veth-must-not-merge` [E2,E4] opposes on `l1-distinct-mac`; the reason
  carries the epic's exact sentence (gone — dormant-bound, still queryable — not
  never-a-device; a new interface, not a re-sighting); **the header scopes the refusal to L1
  explicitly** (validation's HIGH): both veths group into the host at L2, correct and
  asserted by no trap — multi-nic's duality.
- **AC3**: the F17/D17 record with honest attribution (F17's "returning to active" verbatim;
  "same entity" = D17 property 1; the D17 drift quote verbatim) and the plain statement that
  no trap can test the sweep.
- **AC4**: byte-pin pins the four obs_ids, fact counts 4/3/4/3, E1's four facts by value,
  E3 == E1 fact-by-fact, E2/E4 values with the recycled-address equality and the MAC
  inequality, the three trap-judged uplinks equal with E1's value-pinned, instants as an
  exact vector.
- **AC5–AC8**: both poles, no abstain (header sentence); E3's why + the minimal.jsonl
  legality precedent in the header; deliberate bump red-first; all values synthetic and
  fresh.

### File List

- `fixtures/scenario/replay/docker-veth.jsonl` — NEW: 4-observation replay stream
- `fixtures/scenario/traps/docker-veth.toml` — NEW: 2-trap family file (F17/D17 record)
- `fixtures/MANIFEST.toml` — modified: two entries appended (19 → 21)
- `crates/opencmdb-bin/src/fixtures.rs` — modified: byte-pin test appended (tests only)
- `crates/opencmdb-bin/src/trap_gate.rs` — modified: three literals 19 → 21 + two comments
  (tests only)
- `_bmad-output/implementation-artifacts/4-16-trap-ephemeral-docker-veth.md` — this story
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status tracking

## Change Log

| Date       | Change                                                                 |
|------------|------------------------------------------------------------------------|
| 2026-07-25 | Story 4.16 drafted (create-story, autonomous run): ephemeral Docker veth — the third mutual-check family. must-merge [host, veth] on `l2-uplink-agrees` (the shape 4.14 refuses — the IANA prefix is the only difference); must-not-merge [veth1, veth2] on `l1-distinct-mac` across the recycled container address (4.13's kin, slot-continuity temptation), its reason carrying the epic's gone-not-never-a-device sentence. Header records F17/D17 consistency while stating no trap can test the sweep. E3 (host re-seen, feeds no trap) is the authored evidence of disappearance — NFR7 forbids an absence fact. 4-obs stream `babababa`, `doc-dockerhost`, counts 19 → 21. Status → ready-for-dev. |
| 2026-07-25 | Validated (two fresh-context agents: fact-check + gap-hunt). 1 HIGH / 5 MED / 8 LOW, all applied. The HIGH: the must-not-merge [E2,E4] is the corpus's first must-not-merge whose two members belong to ONE L2 device — the header must scope the refusal to L1 explicitly (multi-nic's duality), else the family reads self-contradictory when Epic 5 groups both veths into the host. Also: "exact firing shape" weakened (multi-nic's committed pair agrees across DIFFERENT ports; same-port is a stronger agreement the header defends); "same entity" re-attributed to D17 property 1 (not F17); D17 quote made verbatim; reasons precision (no MAC octets review-held, descriptive address, no spec ids in reasons, 269-char draft validated); E3 legality precedent (minimal.jsonl) cited; E1/E3 template = vrrp's own 4-fact line; corpus UUIDs written out; ":392 comment tail" → ":387-391"; Trap 5 clarified (forbid the affirmation, require the negation). |
| 2026-07-25 | Implemented (dev-story): all 6 tasks, ATDD held — byte-pin RED (`FixtureError::Io`), stream landed and greened, trap file landed (header carrying the L1-scoping sentence, the two mutual checks, the F17/D17 record with honest attribution, E3's why + minimal.jsonl precedent), count RED `left: 21, right: 19` then 19 → 21, manifest 19 → 21 (sha256 after final byte: `9cf1569e…`, `2c2a5407…`). Reasons 269/269, octet-free, no spec ids. Gates green: fmt, clippy `-D warnings`, `cargo test --workspace` (116+86+42), `xtask ci` ("21 fixture(s) match their recorded sha256"). Status → review. |
| 2026-07-25 | Code review (3 fresh-context layers). **Auditor: PASS 8/8** — every Dev Record claim reproduced (suite 116+86+42, xtask message verbatim, both sha256, reasons 269/269, count-RED replayed with the stale-binary `touch` guard). 0 CRITICAL/HIGH; **4 patches applied**: `Expectation::MustNotMerge`'s core doc widened to the rule-scoped truth (the committed corpus now contains an L1-scoped refusal whose members share one device — doc-only production change), the missing "Implemented" Change Log row (4.15's recidive), the MANIFEST comment's predicate mis-attribution, the byte-pin doc weakened to "value-identical". **1 defer registered** (`Observation.raw` unscanned by the privacy walk — pre-existing since 4.1). Gates re-run green post-patch. Status → done. |
