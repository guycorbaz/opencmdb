# Story 4.14: Trap family — VRRP/HSRP shared virtual MAC

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author of the trap corpus,
I want the shared virtual-MAC family committed,
so that **a redundancy protocol does not read as one device** (epic 4.14): the IANA VRRP virtual
MAC is a *protocol* address borne in turn by two real routers — it must neither fold the VIP into
its master of the minute (D16's rejected option B), nor fuse the two bearers into one box, nor
dissolve the one virtual gateway into per-sighting ghosts when a failover moves its uplink.

## Decision record (party mode, 2026-07-25)

This story required two design decisions, taken in a party-mode roundtable (Winston, Amelia,
Murat) run under Guy's autonomous-run mandate. Recorded here so the dev agent implements a
decision, not a debate; the epic-end report to Guy carries the full positions.

1. **The privacy walk is AMENDED, not worked around (unanimous).** An authentic VRRP virtual MAC
   (`00:00:5e:00:01:xx`) is universally administered, so today's walk reds on it — but a
   locally-administered stand-in would commit bytes that do NOT carry the structural fact this
   family exists to test (architecture.md:999-1002; D16 creates `virtual_device` on that very
   prefix). D19 says the corpus is a SPEC — "a spec that fakes its load-bearing bytes is a false
   spec" (Winston). The walk therefore learns the invariant it always meant: *no byte may identify
   a real network* — a protocol-defined MAC identifies a protocol, the same bytes on every VRRP
   deployment on earth, the MAC analog of an RFC 5737 address (architecture.md itself prints
   `00:00:5e:00:01:0a` in prose at :1139-1140). **Scope is Amelia/Murat's narrow version:** admit ONLY
   the 5-octet VRRP IPv4 prefix `00:00:5e:00:01` — the range this corpus actually commits. HSRP
   (`00:00:0c:07:ac` — a **Cisco** OUI, not IANA; Murat's correction) and HSRPv2 stay OUT until a
   fixture commits those octets, each with its own prove-to-red ("no allowance a fixture does not
   exercise" — the corpus-lock principle applied to the allowlist). ONE shared helper feeds both
   the fact walk and the free-text scanner.
2. **Three traps; one rule id is coined; `l1-exact-mac` fires the must-merge (arbitrated).** The
   roundtable agreed on the stream (4 observations) and on three traps with no `must-abstain`
   pole. Two points were disputed and arbitrated: *(a)* Murat is right that the primary
   must-not-merge `[V1, A]` cannot cite an existing rule — `l1-distinct-mac` is not the opposer,
   because multi-nic's own must-merge (`l2-uplink-agrees`) already mandates grouping distinct-MAC
   interfaces whose uplink agrees; the ONLY thing that opposes folding the VIP into its master is
   the structural prefix disqualification, so this story coins **`l2-virtual-mac-prefix`** (cited
   by that expectation — the anti-coining doctrine forbids an id NO expectation cites, not one an
   expectation needs; D16's "no new rule, no score" bans a new *scored* rule, and this id names an
   ingestion READING). *(b)* Winston/Amelia are right that the must-merge `[V1, V2]` fires
   **`l1-exact-mac`**: L1 is `(l2_domain, mac) -> interface` and "is not a probabilistic problem"
   (architecture.md:985) — deterministic for a virtual MAC too; the virtual-device recognition is
   a separate act layered on top, and coining a second id there would deny L1's own determinism.

## Acceptance Criteria

1. **The primary form — the VIP must not fold into its master (D16's rejected option B).**
   **Given** the VIP sighting V1 and the master's physical presence A — V1 carrying the authentic
   IANA VRRP MAC (`00:00:5e:00:01:0a`, bytes `[0,0,94,0,1,10]`, `locally_administered: false`),
   the VIP `192.0.2.1` and the SAME uplink as A (peer `[2,0,94,0,96,10]`, port `swport-11`) —
   **when** the trap is scored, **then** it is a **`must-not-merge`** naming the COINED rule
   **`l2-virtual-mac-prefix`** (the IANA virtual-router prefix disqualifies this MAC as a grouping
   anchor — "a reading, not an inference", architecture.md:999-1002): the agreeing uplink is the
   temptation (`l2-uplink-agrees` is what multi-nic's must-merge rewards), and attaching the VIP
   to its master of the minute is "choosing a winner between two legitimate owners is merging"
   (D16). The expectation names what OPPOSES, never what tempts (house doctrine, trap.rs:66-73).

2. **The transitive form — the two bearers must not fuse.** **Given** the two physical routers A
   and B (distinct locally-administered MACs, distinct hostnames `doc-rtr-alpha` /
   `doc-rtr-bravo`, distinct addresses `.2` / `.3`) **when** the trap is scored, **then** it is a
   **`must-not-merge`** naming **`l2-different-hostname`** (REUSED — the cloned-mac precedent for
   two real boxes at device level): the temptation is the two-hop transitive fusion A ≡ VIP ≡ B
   through the shared virtual address, D10's catastrophic false merge arriving via redundancy.

3. **The negative form — the same evidence legitimately belongs to ONE device (epic AC2).**
   **Given** the two VIP sightings V1 and V2 — byte-identical virtual `Mac`, byte-identical
   `IpV4` `192.0.2.1`, same `l2_domain`, one hour apart, with the uplink MOVED from switch
   `[2,0,94,0,96,10]` port `swport-11` (A's attachment) to a DIFFERENT switch
   `[2,0,94,0,96,11]` port `swport-12` (B's attachment — redundancy pairs typically span two
   switches, and the second switch is already in the corpus's furniture from multi-nic) —
   **when** the trap is scored, **then** it is a **`must-merge`** naming **`l1-exact-mac`**
   (L1 is deterministic on `(l2_domain, mac)`, architecture.md:985; D16: the virtual gateway is
   ONE device, `kind='virtual'`): the moved uplink is a committed L2 contradiction in the
   corpus's OWN vocabulary — a different `peer_mac` is exactly the committed shape of
   `l2-different-switch` (multi-nic's must-not-merge, M1/M3) — that the merge must overcome; a
   failover is the redundancy doing its job, not a second device. This is the corpus's first
   must-merge where a committed L2 signal actively opposes L1's verdict. (Validation note: same
   switch + different port would NOT have been a contradiction — multi-nic's must-merge treats
   that as an uplink that AGREES; the second switch is what makes the claim true.) An engine that answers the virtual
   prefix with a blanket refusal-to-merge dissolves the default gateway — the most-consulted
   object on the network (D16) — into per-sighting ghosts; this is D18's anti-cowardice column
   holding the line.

4. **The privacy walk admits exactly the VRRP range, through one helper, proven red at the
   boundaries.** A new private helper in `fixtures.rs`'s test module — e.g.
   `is_synthetic_mac(addr: MacAddr) -> bool`, true iff `addr.is_locally_administered()` OR the
   first FIVE octets equal `[0, 0, 94, 0, 1]` (`00:00:5e:00:01`, the IANA VRRP IPv4
   virtual-router block) — becomes the single source of truth for BOTH `assert_synthetic_mac`
   (:906) and the free-text scanner `assert_text_is_synthetic`'s MAC leg (:854-860). Both
   panic messages are updated to name the closed rule — e.g. *"… is neither locally
   administered nor in the IANA VRRP virtual-router range 00:00:5e:00:01:xx — a real vendor
   address must never be committed"*. The doc comment states the invariant (protocol identity,
   not network identity; the MAC analog of RFC 5737), that the list is closed and 5-octet
   exact, and that the invariant is **byte-level, not position-level**: the admitted range is
   admitted wherever a MAC is checked (`Fact::Mac`, `Uplink::peer_mac`, free text) — an
   allowance on the bytes cannot be a hole in one position and not another. Boundary tests pin
   the carve-out's edges on the helper: `00:00:5e:00:01:0a` → true; `00:00:5e:00:00:0a`,
   `00:00:5e:00:02:0a` (VRRP IPv6 — NOT admitted until a fixture commits it),
   `00:00:5f:00:01:0a` and the plain vendor-style `00:11:22:33:44:55` → all false. **The
   scanner's wiring is proven directly, not assumed** (validation caught that NO committed
   trap text ever reaches the scanner — its only call site is the `Record::Failure` walk at
   :831): a test calls `assert_text_is_synthetic` on a string containing `00:00:5e:00:01:0a`
   (red before the amendment, green after) and a `#[should_panic]` sibling on a string
   containing `00:00:5e:00:00:0a` (still red — the scanner kept its teeth). **Prove-to-red:**
   *(i)* the natural red — with the new stream on disk and the walk unamended,
   `the_corpus_carries_no_real_network_data` reds naming `00:00:5e:00:01:0a`; *(ii)* the
   mutation — widening the helper's match to the 3-octet `00:00:5e` prefix must red the
   boundary test (recorded, then reverted); *(iii)* the scanner-wiring red above.

5. **The flag-vs-bytes cross-check closes the corpus's open flank (Amelia's guard).** V1/V2 are
   the corpus's FIRST `locally_administered: false` facts, and nothing today cross-checks the
   authored serde flag against the U/L bit of the bytes. `assert_facts_are_synthetic`'s
   `Fact::Mac` arm binds BOTH fields and asserts
   `*locally_administered == addr.is_locally_administered()` (the doc comment on
   `MacAddr::is_locally_administered`, observation/mod.rs:74-77, names exactly this
   cross-check as its purpose), with a failure message naming the path and both values. Proven
   red with an in-memory mis-paired fact (a locally-administered byte pattern carrying
   `locally_administered: false`) — constructed in a test, never committed. **The test form is
   `#[should_panic(expected = "…")]` calling `assert_facts_are_synthetic` directly** (the
   function returns `()` and panics; a bool-helper form does not exist for the flag — the flag
   never reaches `is_synthetic_mac`).

6. **The stream is committed as 4 observations whose bytes carry the whole story.**
   `fixtures/scenario/replay/vrrp-virtual-mac.jsonl`, fresh `obs_id` prefix `aeaeaeae`, one
   `connector_id` / `l2_domain` / `vantage` (the corpus's synthetic UUIDs), strictly increasing
   `observed_at` (`2026-01-08T00:00:00Z`, `T00:05:00Z`, `T00:10:00Z`, `T01:00:00Z`), `raw: null`:
   - **V1** (`…0001`, 00:00): virtual `Mac` `[0,0,94,0,1,10]` flag `false` · `IpV4 192.0.2.1` ·
     `Uplink` peer `[2,0,94,0,96,10]` port `swport-11` — 3 facts, no `Hostname` (an ARP-style
     sighting; the VIP answers, nobody resolves it).
   - **A** (`…0002`, 00:05): `Mac` `[2,0,94,0,83,140]` flag `true` · `IpV4 192.0.2.2` ·
     `Hostname doc-rtr-alpha` · `Uplink` peer `[2,0,94,0,96,10]` port `swport-11` (SAME as V1 —
     the temptation) — 4 facts.
   - **B** (`…0003`, 00:10): `Mac` `[2,0,94,0,83,141]` flag `true` · `IpV4 192.0.2.3` ·
     `Hostname doc-rtr-bravo` · `Uplink` peer `[2,0,94,0,96,11]` (the SECOND switch —
     multi-nic's `M3` furniture) port `swport-12` — 4 facts.
   - **V2** (`…0004`, 01:00): virtual `Mac` byte-identical to V1's, flag `false` ·
     `IpV4 192.0.2.1` byte-identical · `Uplink` peer `[2,0,94,0,96,11]` port `swport-12`
     (B's switch and port — the failover crossed switches) — 3 facts.
   The first obs_id in full is `aeaeaeae-0000-4000-8000-000000000001` (neighbours follow the
   corpus convention, `…0002`/`…0003`/`…0004`). `Hostname` facts carry
   `"source":"Dhcp"` — the corpus's uniform choice, and `Fact::Hostname` REQUIRES `source`
   (`deny_unknown_fields`; the multi-nic template has no Hostname line to copy). Verbatim
   templates for the two novel line shapes:
   - V1: `{"obs_id":"aeaeaeae-0000-4000-8000-000000000001","connector_id":"33333333-3333-4333-8333-333333333333","observed_at":"2026-01-08T00:00:00Z","scope":{"l2_domain":"11111111-1111-4111-8111-111111111111","vantage":"22222222-2222-4222-8222-222222222222"},"facts":[{"Mac":{"addr":[0,0,94,0,1,10],"locally_administered":false}},{"IpV4":{"addr":"192.0.2.1"}},{"Uplink":{"peer_mac":[2,0,94,0,96,10],"peer_port":"swport-11"}}],"raw":null}`
   - A: `{"obs_id":"aeaeaeae-0000-4000-8000-000000000002","connector_id":"33333333-3333-4333-8333-333333333333","observed_at":"2026-01-08T00:05:00Z","scope":{"l2_domain":"11111111-1111-4111-8111-111111111111","vantage":"22222222-2222-4222-8222-222222222222"},"facts":[{"Mac":{"addr":[2,0,94,0,83,140],"locally_administered":true}},{"IpV4":{"addr":"192.0.2.2"}},{"Hostname":{"name":"doc-rtr-alpha","source":"Dhcp"}},{"Uplink":{"peer_mac":[2,0,94,0,96,10],"peer_port":"swport-11"}}],"raw":null}`
   A new byte-pin test in `fixtures.rs` (the `expected()` second-oracle idiom) asserts the
   fact-counts per line (3/4/4/3), the two byte-identities (V1.Mac==V2.Mac, V1.IpV4==V2.IpV4),
   the flag truthfulness on all four MACs, the uplink equalities (V1.Uplink==A.Uplink,
   V2.Uplink==B.Uplink), the switch move (V1.peer_mac `[2,0,94,0,96,10]` ≠ V2.peer_mac
   `[2,0,94,0,96,11]`, V2's port pinned to `swport-12`), the two hostnames, and the four
   authored instants in strict increase (module `ts()` helper).

7. **The three traps are committed with both poles present and no `must-abstain` — deliberately.**
   `fixtures/scenario/traps/vrrp-virtual-mac.toml`, `family = "vrrp-virtual-mac"` on all three
   (ids `vrrp-virtual-mac-must-not-merge-master`, `vrrp-virtual-mac-must-not-merge-bearers`,
   `vrrp-virtual-mac-must-merge`), so `incomplete_families` sees ≥1 `must-merge` AND ≥1
   `must-not-merge`. The header records, with citations: *(a)* the family statement — 1
   interface, 2 devices (architecture.md:894); the VIP gets `kind='virtual'` created on the
   structural prefix, "one line of truth doing two jobs" (D16); *(b)* **the absent third column
   is a spec assertion** (Winston): D16 rejected abstention here in terms — "a SEMANTIC DUSTBIN…
   not an ambiguity, a topology fact" — so a `must-abstain` in this family would be the drift
   D16 forbids; *(c)* the coined-rule record — `l2-virtual-mac-prefix` is coined HERE, cited by
   the primary expectation, names an ingestion reading not a scored rule (the corpus's first
   *structural* id — the earlier L2 ids were coined by 4.10/4.11, so do NOT write "first coin";
   validation caught that), and the multi-nic argument (why no existing id can oppose
   `[V1, A]`); *(d)* the privacy record — the VRRP range
   entered the walk with this family (protocol identity, not network identity); HSRP stays out
   until committed, and its OUI is Cisco's, not IANA's. Every id whole on its line (4.11's
   wrap lesson).

8. **Every expectation carries its mandatory one-sentence `reason`** (20–300 chars, single line,
   `Trap::validate`), each claim checkable against the committed bytes, and — per Amelia —
   **no raw MAC octets in the reasons**. Two honesty notes from validation: *(a)* this DEVIATES
   from the dhcp-churn precedent, whose committed must-merge reason names a raw MAC — nothing
   scans trap-file text (the scanner's only call site is the `Record::Failure` walk), so the
   no-octets rule is held by review alone; say so, claim no gate. *(b)* keep the roles straight
   in the primary's reason: the uplink shared with `doc-rtr-alpha` is the TEMPTATION, the IANA
   virtual-router range is what OPPOSES — a reason that lists the shared uplink among the
   opposing values inverts the doctrine (trap.rs:66-73). The bearers' reason names the two
   hostnames; the must-merge reason names the byte-identities and the uplink move from
   `swport-11` to the second switch's `swport-12`.

9. **The corpus lock and the count coupling are bumped deliberately, red first.** Both new
   artefacts enter `fixtures/MANIFEST.toml` with sha256 computed AFTER the final byte (15 → 17
   artefacts; the gate's message becomes `"17 fixture(s) match their recorded sha256 (0
   generated, 17 hand-authored)"`). The three committed-count assertions in `trap_gate.rs` move
   **14 → 17** — `the_committed_corpus_is_discovered_and_scored_by_nothing` (:391, breakdown
   comment :387-390 gains "vrrp-virtual-mac.toml (story 4.14) three — seventeen"),
   `the_report_says_plainly_that_nothing_was_scored` (:409, `"17 trap(s) discovered"`),
   `a_trap_with_no_answer_is_discovered_but_not_scored` (:427 + comment :419 "stays 17") — with
   the red at `left: 17, right: 14` observed and recorded BEFORE the update. The 4.13
   reproducibility test (`replaying_the_same_corpus_twice_yields_identical_verdicts`, :1033) is
   untouched and must stay green — its two answers score 2 regardless of three new unanswered
   traps; verified by the run, not asserted on faith. Scratch-corpus tests untouched.

10. **Synthetic-only, with the one sanctioned exception.** Every OTHER value obeys the walk
    unchanged: locally-administered physical MACs (fresh last-bytes `83,140` / `83,141`),
    RFC 5737 addresses (fresh `.1` / `.2` / `.3`), `doc-` hostnames (fresh `doc-rtr-alpha` /
    `doc-rtr-bravo`), the two established switch peers `[2,0,94,0,96,10]` / `[2,0,94,0,96,11]`
    (multi-nic's furniture) with fresh ports `swport-11` / `swport-12`. The virtual MAC is the
    ONLY universally-administered byte pattern
    in the corpus, and it is admitted by name. `grep -rn aeaeaeae fixtures/ crates/` hits only
    the two new files at commit time.

## Tasks / Subtasks

> **⚠️ ATDD ORDER.** The privacy amendment's natural red NEEDS the stream on disk first, so this
> story's order differs from 4.13's: byte-pin test red → stream lands → privacy walk reds on the
> virtual MAC (the natural red, recorded) → helper + amendment + boundary/cross-check tests →
> walk greens → trap file lands → count red at 14 → 17. Mid-story the fixtures gate reds on two
> orphans — expected until the manifest bump.

- [x] **Task 1 — write the byte-pin test, observe it RED** (AC: 6) — prove-to-red
  - [x] In `fixtures.rs`'s trailing test module, e.g.
        `the_vrrp_stream_shares_one_virtual_mac_and_moves_its_uplink`, appended at the END of
        the module (4.13's placement rule: cited line numbers stay valid; names are the anchor).
        Read via `read_jsonl(&fixture_path("scenario/replay/vrrp-virtual-mac.jsonl").unwrap())`.
        Assert everything AC6 lists. Doc comment: the second-oracle idiom, and that THIS test is
        where the stream's shape is pinned (the harness validates no uplink geometry anywhere
        else).
  - [x] Run it: RED inside `read_jsonl` with `FixtureError::Io` (missing file — `fixture_path`
        checks shape, not existence). Record the red.

- [x] **Task 2 — write the replay stream** `fixtures/scenario/replay/vrrp-virtual-mac.jsonl`
      (AC: 6, 10) — pure data
  - [x] Four lines exactly as AC6 prescribes (envelope template: a `multi-nic.jsonl` line — it
        already carries `Uplink`; fields in order `obs_id`, `connector_id`, `observed_at`,
        `scope`, `facts`, `raw`). Corpus UUIDs: `connector_id`
        `33333333-3333-4333-8333-333333333333`, `l2_domain` `11111111-1111-4111-8111-111111111111`,
        `vantage` `22222222-2222-4222-8222-222222222222`. Trailing newline. **Before authoring,
        re-verify the frees**: `aeaeaeae`, `doc-rtr-alpha`, `doc-rtr-bravo`, bytes `83,140` /
        `83,141`, IPs `.1/.2/.3`, ports `swport-11/12` (all free at story-creation).
  - [x] Byte-pin test: still red, but now on CONTENT if any byte diverges — drive to green.
  - [x] `the_corpus_carries_no_real_network_data`: **RED naming `00:00:5e:00:01:0a`** — the
        natural red of AC4. Record the exact panic message.

- [x] **Task 3 — amend the privacy walk** `crates/opencmdb-bin/src/fixtures.rs` (AC: 4, 5) —
      test-module code only
  - [x] The `is_synthetic_mac` helper (AC4's contract, doc comment included); rewire
        `assert_synthetic_mac` and `assert_text_is_synthetic`'s MAC leg onto it — ONE list.
  - [x] The `Fact::Mac` arm binds `locally_administered` and cross-checks it against the bytes
        (AC5) — failure message names the path and both values.
  - [x] Boundary tests on the helper (AC4's five cases, `00:11:22:33:44:55` as the vendor-style
        case) + the two DIRECT scanner-wiring tests (AC4: `assert_text_is_synthetic` green on a
        string carrying `00:00:5e:00:01:0a`, `#[should_panic]` on one carrying
        `00:00:5e:00:00:0a`) + the mis-paired-flag red (AC5:
        `#[should_panic(expected = "…")]` calling `assert_facts_are_synthetic` directly — the
        function panics, there is no bool form for the flag); record the reds.
  - [x] Mutation: widen the helper to `00:00:5e` 3-octet match → boundary test reds → revert.
        Record. `the_corpus_carries_no_real_network_data` now GREEN over the new stream.

- [x] **Task 4 — write the trap file** `fixtures/scenario/traps/vrrp-virtual-mac.toml`
      (AC: 1, 2, 3, 7, 8) — pure data
  - [x] Header per AC7 (a–d), every id whole on its line. Three `[[trap]]` blocks, multi-line
        `observations` arrays (house voice), full UUIDs:
        - `vrrp-virtual-mac-must-not-merge-master`: `[…0001, …0002]`,
          `expect = { must-not-merge = { rule = "l2-virtual-mac-prefix" } }`
        - `vrrp-virtual-mac-must-not-merge-bearers`: `[…0002, …0003]`,
          `expect = { must-not-merge = { rule = "l2-different-hostname" } }`
        - `vrrp-virtual-mac-must-merge`: `[…0001, …0004]`,
          `expect = { must-merge = { rule = "l1-exact-mac" } }`
  - [x] Reasons per AC8 (no MAC octets; measure with `wc -m`, record counts; 20–300, one
        sentence each).
  - [x] Trailing newline; then the whole-corpus walks
        (`every_trap_file_in_the_corpus_is_valid`, `no_obs_id_is_shared_across_replay_streams`,
        `every_replay_stream_in_the_corpus_is_valid`) pass unchanged.

- [x] **Task 5 — prove the count coupling red, update 14 → 17** `trap_gate.rs` (AC: 9)
  - [x] With both files on disk and assertions at 14:
        `cargo test -p opencmdb-bin --locked the_committed_corpus_is_discovered_and_scored_by_nothing`
        → RED `left: 17, right: 14`. Record. Then the three updates + two comments; scratch
        tests untouched; reproducibility test untouched and green.

- [x] **Task 6 — lock the artefacts** `fixtures/MANIFEST.toml` (AC: 9) — deliberate bump
  - [x] Two `[[artefact]]` entries with story-naming comments (mirror the 4.13 entries), sha256
        of the FINAL bytes (`sha256sum` after the last edit, trailing-newline check
        `tail -c 1 | xxd` → `0a`). Existing fifteen entries byte-for-byte untouched.

- [x] **Task 7 — gates** (AC: 9, 10)
  - [x] `cargo fmt --all` · `cargo clippy --workspace --all-targets --locked -- -D warnings` ·
        `cargo test --workspace --locked` · `cargo run -p xtask --locked -- ci`. Quote the
        fixtures gate's real message (expect 17/17). Residual grep: `"14 trap"` /
        `discovered(), 14` / `stays 14` / `fourteen` → no hits outside history. `Cargo.lock`
        unchanged; `architecture-views.md` NOT regenerated (Epic-4 milestone);
        `fixtures/scenario/traps/README.md` untouched (no reality-produced case; the
        conflated-VIP-and-master sibling family Murat sketched goes to the epic report, not the
        register).

### Review Findings

- [x] [Review][Patch] The byte-pin test pinned only V2's uplink octet-exact — V1/A's end was
      asserted relationally (`uplink(0)==uplink(1)`, `!=uplink(3)`), so a re-authored stream
      putting V1/A on the second switch with a different port would stay green while dissolving
      the "different switch, not merely port" premise, and the test's doc overclaimed "pinned
      here or nowhere". Both ends now pinned octet-exact (Edge Case Hunter #1 / Auditor #1 —
      the 4.13 `.121` lesson, mirrored) [crates/opencmdb-bin/src/fixtures.rs]
- [x] [Review][Patch] A's and B's own addresses (`192.0.2.2`/`.3`) were unasserted scenery —
      the bearers' reason claims "distinct addresses" and the primary's premise needs the VIP to
      be nobody's own address; both now value-pinned (Edge Case Hunter #2 / Blind Hunter #2)
      [crates/opencmdb-bin/src/fixtures.rs]
- [x] [Review][Patch] "the whole last octet is the VRID — any value is in-range" asserted a
      protocol falsehood (VRID 0 is not deployable, RFC 9568 says 1–255); weakened to the true
      sentence (Blind Hunter #1 / Edge Case Hunter #3) [crates/opencmdb-bin/src/fixtures.rs]
- [x] [Review][Patch] sprint-status.yaml's `last_updated` comment still said "→ ready-for-dev"
      while the binding status field said review (Auditor #2)
- [x] [Review][Defer] Trap-file text (headers, reasons) is scanned by no privacy rule — its only
      call site is the `Record::Failure` walk; pre-existing since 4.2, honestly stated by this
      story's own scanner-wiring test doc → registered in deferred-work.md under
      "code review of story-4.14"
- [x] [Review][Defer] The free-text scanner's named evasions (trailing-punctuation tokens,
      dash-form MACs, the U/L-bit rule admitting IPv6-multicast-shaped strings) — pre-existing
      "floor, not a proof", not aggravated here → same register entry

## Dev Notes

### The shape of this story in one paragraph

The corpus's first VIRTUAL identity family, and its first story that touches harness code beyond
count literals: two committed artefacts (a 4-observation stream, a 3-trap file), the privacy
walk's first sanctioned exception (the 5-octet VRRP range, one helper, boundaries proven red),
one NEW guard (flag-vs-bytes cross-check), one COINED rule id (`l2-virtual-mac-prefix` — the
first coin since 4.11's L2 pair, and the corpus's first STRUCTURAL id; 4.10/4.11 coined the
four l2-* ids, so "first coin ever" is false — validation caught it), counts 14 → 17, manifest
15 → 17. The
family's three traps hold D16's whole geometry: don't fold the VIP into its master (primary),
don't fuse the bearers (transitive), DO track the one virtual gateway across failover
(anti-cowardice, against a committed L2 contradiction — a corpus first).

### Why `l2-virtual-mac-prefix` is coined and what it must NOT become

The anti-coining doctrine (4.13) forbids an id **no expectation cites** — this one is cited by
the primary trap. It names the structural ingestion reading (architecture.md:999-1002:
IANA prefix → "Disqualifying as grouping anchor", "a reading, not an inference"), NOT a scored
rule — D16's "no new rule, no score" stands. The decisive argument (Murat's, arbitrated in):
`l1-distinct-mac` cannot oppose `[V1, A]` because multi-nic's must-merge already REWARDS
grouping distinct-MAC interfaces whose uplink agrees (`l2-uplink-agrees`) — an engine following
the existing vocabulary to the letter WOULD fold the VIP into its master; only the prefix
reading stops it. If Epic 5 implements the reading under a different name, the trap reds with a
`rule_mismatch` and the rename is a deliberate corpus bump — that friction is the spec working,
not a defect. Do NOT also cite the coined rule from the must-merge (`l1-exact-mac` is correct
there — L1's determinism is its own law, architecture.md:985) and do NOT retrofit it into 4.9's
U/L case (randomized-mac's expectations are settled).

### The temptation map — which signal pulls which way (keep it straight while authoring)

| Pair | Tempts toward merge | Opposes | Column | Named rule |
|---|---|---|---|---|
| `[V1, A]` | shared uplink `swport-11` (`l2-uplink-agrees`), shared subnet, gateway role | IANA virtual prefix (structural) | must-not-merge | `l2-virtual-mac-prefix` (COINED) |
| `[A, B]` | two-hop transitivity through the VIP | distinct hostnames (and MACs, IPs) | must-not-merge | `l2-different-hostname` (REUSED) |
| `[V1, V2]` | byte-identical MAC + VIP (L1) | uplink moved to a DIFFERENT switch `[…96,10]` → `[…96,11]` — the committed shape of `l2-different-switch` (multi-nic M1/M3; same-switch-different-port would AGREE, not oppose) | **must-merge** | `l1-exact-mac` (REUSED) |

The third row is the family's teeth: cloned-mac taught "same MAC, different hostnames → refuse";
an engine that generalizes that to "suspicious MAC → refuse" is demolished HERE, by the corpus's
first must-merge that overcomes a committed opposing L2 signal. The two families hold each other
in check (Murat) — say so in the header.

### Why there is deliberately NO `must-abstain` (and it is a spec assertion, not an omission)

D16, verbatim: abstention here would be "a SEMANTIC DUSTBIN — the catch-all for what we failed
to model… if `Ambiguous` means both 'real conflict' and 'unmodelled case', it means nothing".
There is no evidence conflict in these bytes — both sightings agree on everything; the sharing
IS the protocol. A reviewer proposing a third pole re-opens D16, not this story. The header
carries the sentence (Winston's "documented-as-false third column is itself a spec assertion").

### The privacy amendment is a NARROWING of error, not a widening of licence

The walk's invariant was never "U/L bit set" — that was the approximation. The invariant is "no
committed byte can identify a real network" (the register's own language). `00:00:5e:00:01:0a`
carries zero bits about any particular network — it is VRID 10 anywhere on earth, and the
architecture already published it in prose (:1139-1140). The helper's doc comment must state: the
list is CLOSED, 5-octet-exact, extended only alongside a fixture that commits the new range
(HSRP's `00:00:0c:07:ac` is a Cisco OUI — entering it would admit a vendor OUI's neighborhood
and needs its own argued story). The boundary reds are the proof the gate kept its teeth:
`00:00:5e:00:00:0a` / `00:00:5e:00:02:0a` / `00:00:5f:00:01:0a` all still panic.

### V1/V2 carry `locally_administered: false` — the first honest `false` in the corpus

The serde flag is the CONNECTOR's reading (observation/mod.rs:148-151); the bytes are the ground
truth (`MacAddr::is_locally_administered`, :74-79, whose doc names the cross-check as its
purpose). Until now every corpus MAC was `true`/U-L-set, so a lying flag was unobservable —
AC5's guard makes flag-vs-bytes a corpus invariant the day the first `false` enters. The red is
in-memory (a mis-paired fact built in the test), never a committed artefact.

### Previous story intelligence (4.13)

- Review: Auditor PASS 9/9, 3 patches — all three about **asserting values the prose claimed**
  (pin `.121`; `facts.len()==3`; weaken an overclaiming doc comment). Applied forward here:
  AC6's byte-pin asserts fact-counts AND V2's `swport-12` port explicitly; every claim in the
  reasons is present in the bytes; doc comments claim only what the test proves.
- The standing defer (since 4.9, deferred-work.md): family streams are not driven through
  `FixtureConnector::load` admissibility — inherited by this stream, NOT fixed here.
- Hash after the final byte; wrap-check before hashing; fresh-value grep before authoring —
  all held in 4.13, same ritual here.
- PR workflow: branch per story → PR → CI green → squash merge (PRs #21–#28) — never straight
  to master. [[opencmdb-pr-workflow]]

### Project Structure Notes

- **NEW (locked):** `fixtures/scenario/replay/vrrp-virtual-mac.jsonl` (4 obs),
  `fixtures/scenario/traps/vrrp-virtual-mac.toml` (3 traps). Both in `MANIFEST.toml` (15 → 17).
- **Updated:** `crates/opencmdb-bin/src/fixtures.rs` — test module ONLY (`is_synthetic_mac`
  helper + rewired `assert_synthetic_mac` / `assert_text_is_synthetic`, the flag-vs-bytes arm,
  boundary tests, byte-pin test); `crates/opencmdb-bin/src/trap_gate.rs` — three count literals
  14 → 17 + two comments (tests only); `fixtures/MANIFEST.toml` (two entries).
- **Unchanged, expected:** all production paths (`trap.rs`, `score.rs`, `fixture_connector.rs`,
  `gap/`), `Cargo.lock`, the fifteen existing manifest entries, `traps/README.md`,
  `architecture-views.md` (STALE by design until the Epic-4 milestone).
- **Out of scope, deliberately:** HSRP/HSRPv2 octets (future fixture, own story); the
  conflated-VIP-and-master ambiguous sibling family (Murat's sketch — epic report material);
  any engine work (Epic 5 implements the prefix reading `l2-virtual-mac-prefix` names); the
  `FixtureConnector::load` admissibility defer; VRID/master tracking (D16 defers to Growth).

### Traps (mistakes this story must not make)

1. **Substituting `02:00:5e:00:01:0a`.** The U/L variant is the OTHER family's signal
   (randomized/local) — the spec would test nothing. The whole point is the authentic bytes.
2. **Widening the walk beyond 5 octets, or admitting HSRP "while we're at it".** Closed list,
   one range, boundaries proven red. HSRP is a Cisco OUI — its own story.
3. **Citing `l1-distinct-mac` on the primary.** Multi-nic's must-merge defeats it (the
   arbitrated point) — the opposer is the coined structural rule.
4. **Citing the coined rule on the must-merge.** `l1-exact-mac` fires there; L1 is
   deterministic (architecture.md:985). Two ids on one pair = the vocabulary drifting.
5. **A `must-abstain` pole.** D16 rejected it in terms; the absence is the assertion.
6. **Hostname on V1/V2, or a `DhcpLease` anywhere.** The VIP sighting is ARP-shaped (3 facts);
   a hostname would collapse the primary into Murat's sibling family; a lease is a rival time
   channel (4.13's lesson).
7. **Different `l2_domain`s.** VRRP is L2-local; a second domain would break L1's
   `(l2_domain, mac)` premise AND the must-merge.
8. **MAC octets in a reason string.** The header exercises the text scanner; reasons stay
   value-checkable without octets (Amelia). A reason claiming what the bytes contradict gets
   caught in review (example.toml's lesson).
9. **Forgetting V2's port pin or the fact-counts in the byte-pin test** (4.13's review lesson —
   assert the values the prose depends on, `.121`'s mirror is `swport-12`).
10. **Mis-pairing a committed flag.** The committed V1/V2 flags are `false` and TRUE to the
    bytes; the mis-paired case lives only in-memory (AC5). A committed mis-pair would red the
    new guard — that red is the guard working.
11. **Touching the reproducibility test.** Its `scored() == 2` premise survives three unanswered
    traps by construction — verify by run, change nothing.
12. **Coining anything else** (`l2-same-ip`, an HSRP id, a VRID rule). One coin, one citation,
    argued in the header. [[claims-must-match-verification]]
13. **Regenerating `architecture-views.md`** (milestone, not story) or editing `traps/README.md`
    (no reality-produced case here).
14. **Hashing before the final byte** (wrap-check first — ids whole on their line, 4.11's
    lesson), or leaving the count grep unrun (`fourteen`/`14 trap`/`stays 14`).

### Latest technical specifics

No new crate, no version bump, no production-path code. Rust 1.96+, edition 2024. All Rust
changes live in `#[cfg(test)]` modules of `opencmdb-bin` (fixtures.rs, trap_gate.rs). **Never
invent a version — pin from the committed `Cargo.lock`, which does not move here.**

### References

- [Source: _bmad-output/planning-artifacts/epics.md:1163-1175 — Story 4.14: the story sentence
  and the two epic ACs (expectation states not-merge or abstain with reason; negative form
  covers the same-evidence-one-device case)]
- [Source: _bmad-output/planning-artifacts/architecture.md:995-1002 — structural facts never
  scored; the IANA VRRP/HSRP prefixes verbatim; U/L bit; "Disqualifying as grouping anchors,
  known at ingestion"]
- [Source: _bmad-output/planning-artifacts/architecture.md:884-895 — D12/D13's L1/L2 table;
  :894 "VRRP/HSRP = 1 interface, 2 devices = L2"; :985 L1 deterministic, "not a probabilistic
  problem"]
- [Source: _bmad-output/planning-artifacts/architecture.md:1101-1144 — D16 in full: options A
  (abstention, REJECTED — "semantic dustbin") and B (attribute to master, REJECTED — "choosing
  a winner between two legitimate owners is merging") and C (virtual_device, DECIDED, "no new
  rule, no score"); the printed MAC `00:00:5e:00:01:0a` at :1139-1140]
- [Source: _bmad-output/planning-artifacts/architecture.md:509-523 — D10: false merge
  catastrophic; the adversarial matrix naming "VRRP/HSRP shared virtual MAC" at :522]
- [Source: crates/opencmdb-bin/src/fixtures.rs:848-913 — `assert_text_is_synthetic` (MAC leg
  :854-860), `assert_facts_are_synthetic` (the `Fact::Mac` arm :876), `assert_synthetic_mac`
  (:906-913) — the three sites the amendment touches; the corpus walks at :821/:1306/:1327/:1424]
- [Source: crates/opencmdb-core/src/observation/mod.rs:70-90 — `MacAddr` (pub `[u8; 6]`),
  `is_locally_administered` (:78, U/L = bit 1 of octet 0) whose doc comment names the
  flag-vs-bytes cross-check; `Fact::Mac { addr, locally_administered }` at :148-151]
- [Source: crates/opencmdb-bin/src/trap_gate.rs:383-430, :1033 — the three count assertions
  (14 → 17) and their comments; `replaying_the_same_corpus_twice_yields_identical_verdicts`
  (untouched)]
- [Source: fixtures/scenario/replay/multi-nic.jsonl — the envelope template WITH `Uplink`
  facts; the switch peer `[2,0,94,0,96,10]` and the taken ports swport-1/2/7]
- [Source: fixtures/scenario/traps/cloned-mac.toml — `l2-different-hostname`'s device-level
  precedent, and the family this one must hold in mutual check]
- [Source: _bmad-output/implementation-artifacts/4-13-trap-dhcp-churn.md — the immediately
  prior family: ATDD order, hash ritual, count-coupling procedure, review lessons applied
  forward (value pinning, fact-counts)]

## Dev Agent Record

### Agent Model Used

claude-fable-5 (Claude Fable 5)

### Debug Log References

- **Task 1 natural RED** (byte-pin, before the stream existed):
  `cargo test -p opencmdb-bin --locked the_vrrp_stream_shares_one_virtual_mac_and_moves_its_uplink`
  → panicked at the test's expect with `FixtureError::Io { path: ".../fixtures/scenario/replay/vrrp-virtual-mac.jsonl", source: Os { code: 2, kind: NotFound } }`
  — inside `read_jsonl`, as predicted (`fixture_path` succeeded on the missing file).
- **AC4 natural RED** (privacy walk, stream on disk, walk unamended):
  `the_corpus_carries_no_real_network_data` → panicked with the exact message
  `".../fixtures/scenario/replay/vrrp-virtual-mac.jsonl: 00:00:5e:00:01:0a is not locally administered — a real vendor address must never be committed"`.
- **AC4 mutation RED**: helper widened to the 3-octet `00:00:5e` match →
  `the_vrrp_allowance_is_five_octets_exact` panicked at
  `"00:00:5e:00:00:0a sits below the block and stays out"` — then reverted, test green again.
- **AC4 scanner-wiring**: `the_text_scanner_admits_the_vrrp_range` (green only AFTER the
  rewiring — the scanner's MAC leg calls `is_synthetic_mac`) and the `#[should_panic]` sibling
  `the_text_scanner_still_refuses_a_mac_outside_the_block` (message
  `"neither locally administered nor in the IANA VRRP"`) both pass.
- **AC5 flag red**: `a_mac_whose_flag_contradicts_its_bytes_is_refused` —
  `#[should_panic(expected = "authored locally_administered flag contradicts its own U/L bit")]`
  on an in-memory `Fact::Mac { addr: [2,0,94,0,83,99], locally_administered: false }` — passes
  (i.e. the guard panics as required). The `83,99` byte pattern is in-memory only, never
  committed.
- **Count-coupling RED**: with both fixture files on disk and assertions at 14,
  `the_committed_corpus_is_discovered_and_scored_by_nothing` →
  `assertion 'left == right' failed: the walk must open the corpus / left: 17 / right: 14` —
  the three new traps parsed, validated and resolved their obs_ids. Then 14 → 17 at the three
  sites + two comments, green.
- **Reason lengths** (awk length on the committed bytes, single line each): must-not-merge-master
  **286**, must-not-merge-bearers **280**, must-merge **275** — all inside 20–300.
- **`aeaeaeae` grep**: `grep -rln aeaeaeae fixtures/ crates/` → only the new `.jsonl` and (after
  Task 4) the new `.toml`. Frees re-verified the same way for `doc-rtr-alpha`/`doc-rtr-bravo`,
  MAC bytes `83,140`/`83,141`, IPs `.1/.2/.3`, ports `swport-11`/`swport-12` before authoring.
- **Hash-after-final-byte held**: trailing newline confirmed on both files
  (`tail -c 1 | xxd` → `0a`), wrap-check done (the virtual MAC whole on its own header line),
  THEN `sha256sum`:
  `cb560e12ea9e2ea9c0b65acd6739a8a34ec25ec4af2e21ec113c08b1f28738c1` (vrrp-virtual-mac.jsonl),
  `e454665e24733e78afcc621e2242b1c7d25fd6e3f584254be56b002ca1052488` (vrrp-virtual-mac.toml).
  No edit after hashing.
- **Gates** (all green): `cargo fmt --all` · `cargo clippy --workspace --all-targets --locked
  -- -D warnings` · `cargo test --workspace --locked` → **114 (bin) + 86 (core) + 42 (xtask)
  passed, 0 failed** · `cargo run -p xtask --locked -- ci` → fixtures gate verbatim
  **"17 fixture(s) match their recorded sha256 (0 generated, 17 hand-authored)"**, no orphan;
  `views-hash` still `ℹ STALE` by design. Residual grep
  (`"14 trap"` / `discovered(), 14` / `stays 14` / `fourteen` across `crates/` and `xtask/`) →
  no hits. `Cargo.lock` untouched (`git diff --stat Cargo.lock` empty). The reproducibility
  test (`replaying_the_same_corpus_twice_yields_identical_verdicts`) untouched and green in the
  full run — its `scored() == 2` premise survives three unanswered traps, verified by the run.

### Completion Notes List

- The VRRP family landed exactly as scoped: two NEW locked artefacts
  (`vrrp-virtual-mac.jsonl`, 4 observations; `vrrp-virtual-mac.toml`, 3 traps), a deliberate
  `MANIFEST.toml` bump (15 → 17), three count literals 14 → 17, and the story's harness work —
  the privacy walk's first sanctioned exception (`is_synthetic_mac`, one helper feeding both
  the fact walk and the free-text scanner, 5-octet exact, boundaries proven red) plus the
  flag-vs-bytes cross-check guard. No production-path code.
- **AC1**: `vrrp-virtual-mac-must-not-merge-master` judges `[V1, A]` naming the COINED
  `l2-virtual-mac-prefix`; the header carries the coining argument (multi-nic's must-merge
  defeats every existing opposer).
- **AC2**: `vrrp-virtual-mac-must-not-merge-bearers` judges `[A, B]` naming the REUSED
  `l2-different-hostname`.
- **AC3**: `vrrp-virtual-mac-must-merge` judges `[V1, V2]` naming `l1-exact-mac`, across an
  uplink that moved to the SECOND switch (`[2,0,94,0,96,11]`) — a true committed
  `l2-different-switch`-shaped contradiction (the validation fix), overcome by L1's
  determinism.
- **AC4**: admitted range proven at all six boundary cases; scanner wiring proven directly
  (both directions); natural red + mutation red recorded above.
- **AC5**: flag-vs-bytes is now a corpus invariant (walked over every stream); red proven
  in-memory via `#[should_panic]`.
- **AC6**: byte-pin test asserts fact-counts (3/4/4/3), both byte-identities, flag
  truthfulness on all four MACs, both uplink equalities, the switch move with V2's port pinned
  to `swport-12`, both hostnames (`source: Dhcp`), and the four authored instants.
- **AC7**: three traps, both poles present, NO must-abstain — the header states D16's
  rejection as a spec assertion; `incomplete_families` empty (green in the full suite).
- **AC8**: reasons 286/280/275 chars, one sentence each, no raw MAC octets (the deviation from
  dhcp-churn's precedent is review-held and noted in the story), the primary's reason keeps
  temptation (shared uplink) and opposition (IANA range) in their doctrinal roles.
- **AC9**: counts and manifest bumped red-first; reproducibility test untouched and green;
  scratch tests untouched.
- **AC10**: the virtual MAC is the corpus's only universally-administered byte pattern; every
  other value obeys the unamended rules; `aeaeaeae` grep clean.
- Out of scope honoured: no HSRP octets, no engine work, no README/register edit, no
  `architecture-views.md` regeneration, `Cargo.lock` unchanged, `FixtureConnector::load`
  admissibility defer untouched.

### File List

- `fixtures/scenario/replay/vrrp-virtual-mac.jsonl` — NEW: 4-observation replay stream
- `fixtures/scenario/traps/vrrp-virtual-mac.toml` — NEW: 3-trap family file (one coined rule)
- `fixtures/MANIFEST.toml` — modified: two `[[artefact]]` entries appended (15 → 17)
- `crates/opencmdb-bin/src/fixtures.rs` — modified, test module only: `is_synthetic_mac`
  helper; `assert_synthetic_mac` and `assert_text_is_synthetic` rewired onto it (messages
  updated); `Fact::Mac` arm cross-checks flag vs bytes; byte-pin test + boundary test + two
  scanner-wiring tests + flag `#[should_panic]` test appended
- `crates/opencmdb-bin/src/trap_gate.rs` — modified: three count literals 14 → 17 and two
  comments (tests only)
- `_bmad-output/implementation-artifacts/4-14-trap-vrrp-hsrp-virtual-mac.md` — this story file
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status tracking

## Change Log

| Date       | Change                                                                 |
|------------|------------------------------------------------------------------------|
| 2026-07-25 | Story 4.14 drafted (create-story, autonomous run): the VRRP shared-virtual-MAC family. Two party-mode decisions recorded (Winston/Amelia/Murat): (1) the privacy walk admits exactly the 5-octet IANA VRRP range `00:00:5e:00:01` through one shared helper, boundaries proven red — a stand-in MAC would be a spec faking its load-bearing bytes; HSRP stays out (Cisco OUI) until a fixture commits it; plus Amelia's flag-vs-bytes cross-check guard. (2) Three traps, no must-abstain (D16's rejection is the assertion): primary must-not-merge [V1,A] naming the COINED `l2-virtual-mac-prefix` (multi-nic's must-merge defeats every existing opposer — the arbitrated point), transitive must-not-merge [A,B] on `l2-different-hostname`, and must-merge [V1,V2] on `l1-exact-mac` across a failover-moved uplink — the corpus's first must-merge overcoming a committed opposing L2 signal. Counts 14 → 17, manifest 15 → 17. Status → ready-for-dev. |
| 2026-07-25 | Validated (two fresh-context agents: fact-check + gap-hunt). Fact-check 1 HIGH / 1 MED / 2 LOW; gap-hunt 2 HIGH / 3 MED / 5 LOW; all applied. The two structural HIGHs: (1) NO committed trap text ever reaches `assert_text_is_synthetic` (sole call site = the `Record::Failure` walk, fixtures.rs:831) — the "header exercises the scanner" claim was deleted and replaced by two DIRECT scanner-wiring tests, and the no-octets-in-reasons rule is now honestly labeled review-held (dhcp-churn's committed reason names a raw MAC and nothing reds); (2) V1/V2 on the same switch with different ports was the AGREEING shape per multi-nic's own must-merge — B/V2 moved to the second switch `[2,0,94,0,96,11]` so the must-merge overcomes a TRUE committed `l2-different-switch` contradiction. Also: "first coin" corrected (4.10/4.11 coined the l2-* ids), `#[should_panic]` prescribed for the flag red (no bool form exists), verbatim V1/A JSON lines + `"source":"Dhcp"` pinned, `00:11:22:33:44:55` named as the vendor boundary case, amended panic-message text prescribed, byte-level-not-position-level invariant stated (covers `Uplink::peer_mac`), architecture.md:1139 → :1139-1140. |
| 2026-07-25 | Implemented (dev-story): all 7 tasks complete, ATDD order held — byte-pin RED (`FixtureError::Io`), stream landed, privacy walk's natural RED recorded naming `00:00:5e:00:01:0a`, helper + rewiring + boundary tests (mutation to 3-octet proven red, reverted) + scanner-wiring tests + flag-vs-bytes `#[should_panic]`, trap file landed, count coupling proven RED at `left: 17, right: 14` then updated. Two artefacts locked (manifest 15 → 17, sha256 after final byte); reasons 286/280/275 chars. Full gates green: fmt, clippy `-D warnings`, `cargo test --workspace` (114+86+42), `xtask ci` ("17 fixture(s) match their recorded sha256"). No production-path code; `Cargo.lock`/README/`architecture-views.md` untouched. Status → review. |
| 2026-07-25 | Code review (3 fresh-context layers: Blind Hunter / Edge Case Hunter / Acceptance Auditor). **Auditor: PASS 9/10 AC** (AC6 partial — see patch 1) — every Dev Agent Record claim reproduced by re-measurement (both sha256 recomputed, reasons 286/280/275, tests 114+86+42, fixtures-gate wording verbatim, the 3-octet mutation and the natural red both REPLAYED by the auditor, tree left clean). 0 CRITICAL/HIGH surviving triage; **4 patches applied** (tests/comment/status only, no hashed artefact touched): V1's uplink now pinned octet-exact alongside V2's (the "different switch" premise was half-pinned), A/B's own addresses value-pinned, the VRID-0 comment weakened to the true sentence, the sprint-status comment refreshed; **2 defers registered** in deferred-work.md (trap-file text unscanned — pre-existing since 4.2; the scanner's named evasions — pre-existing floor). Gates re-run green post-patch (114+86+42; "17 fixture(s) match their recorded sha256"). Status → done. |
