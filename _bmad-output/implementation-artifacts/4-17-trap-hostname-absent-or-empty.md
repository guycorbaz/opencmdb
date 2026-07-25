# Story 4.17: Trap family — absent and empty hostname (and never null)

Status: done

## Story

As the author of the trap corpus,
I want the hostname-absence family encoded from measurement,
so that **the corpus tests what the source can actually produce and nothing it cannot** (epic
4.17, D45): the source produces hostname MISSING and hostname EMPTY — never `null` — and an
engine must read BOTH as the absence of a signal: an empty string is not a matchable value, a
name that stops resolving is not a second device, and a trap on `null` would be a gate on a
false truth whose red can never arrive.

## Acceptance Criteria

1. **The false-agreement form — `"" == ""` is not hostname agreement.** **Given** two distinct
   hosts G1 and G2 — distinct locally-administered MACs (`[2,0,94,0,83,200]` /
   `[2,0,94,0,83,201]`), distinct addresses (`192.0.2.70` / `.71`) — BOTH carrying
   `Hostname { name: "", source: Dhcp }` (the EMPTY form the measurement records,
   architecture.md:4319) **when** the trap is scored, **then** it is a **`must-not-merge`**
   naming **`l1-distinct-mac`** (REUSED — the opposer): the temptation is
   `l2-hostname-agrees` firing on the equality of two empty strings — an empty name is the
   absence of a signal wearing a value's clothes, and the distinct MACs oppose. (The tempting
   rule is named by no expectation — dhcp-churn's doctrine.)

2. **The equivalence form — EMPTY must count as NO OBSERVED VALUE (the family's core).**
   **Given** G3 and G4 behind an IDENTICAL uplink (peer `[2,0,94,0,96,10]`, port `swport-31` —
   the shared-hardware-vm shape, where the hostname is THE needed discriminator behind one
   port) — G3 carrying the EMPTY hostname, G4 carrying NO `Hostname` fact at all (the MISSING
   form) — **when** the trap is scored, **then** it is a **`must-abstain`** with cause
   **`NoObservedValue`** (the established spelling, example.toml / shared-hardware-vm): the
   two shapes the source actually produces for "no name" are equivalent, and neither side of
   the invited grouping has an observed name — an engine that reads `""` as a present value
   (and abstains only on MISSING) is demolished here, because the pair would then look
   half-named. Column precedent: shared-hardware-vm's `[W1, W4]` (identical uplink, hostname
   undiscriminating) is must-abstain, NOT must-not-merge — follow it.

3. **The silence form — a name that stops resolving opposes nothing.** **Given** the same box
   re-seen — G6 carries G5's value-identical `Mac` AND `IpV4` (parsed equality — the 4.16 review's wording rule), but NO `Hostname` fact (G5 had
   `doc-host-india`) — **when** the trap is scored, **then** it is a **`must-merge`** naming
   **`l1-exact-mac`** (the firer): a hostname that went missing is a signal that fell silent,
   not a contradiction; an engine that treats disappearance as evidence of a second device
   (or abstains on it) is demolished by D18's middle column. NFR7's echo: absence is derived,
   never observed — so absence can never OPPOSE.

4. **The D45 record — and NO null, structurally.** The family header records, with citations:
   the measured source behaviour is "MISSING and empty both occur · null NEVER — the trap set
   must encode absent and empty — and must NOT encode null, or it tests a case the source
   cannot produce" (architecture.md:4319, the wire-format measurement table); D45's sentence
   — a trap written from belief "is a gate on a false truth; their red has no repair, because
   it will never arrive" (architecture.md:4313-4314); and the STRUCTURAL note that
   `Fact::Hostname`'s `name: String` cannot even represent `null` — the format refuses the
   non-case at the type level, which is the strongest form of "must NOT encode". Also state
   the scope seam: F51's population bound lives in hostname-collision's header (4.15); THIS
   family encodes the absence shapes themselves; the WIRE-level hostname behaviours (JSON
   field missing vs `""`) are 4.18's layer — here they are already normalized into
   fact-missing vs fact-empty.

5. **The privacy walk learns that an empty name is synthetic — proven red at the boundary.**
   The `Fact::Hostname` arm (`assert_facts_are_synthetic`, fixtures.rs) currently requires
   `name.starts_with("doc-")` — an EMPTY name reds it, yet the epic mandates encoding EMPTY.
   The rule becomes `name.is_empty() || name.starts_with("doc-")` with the message updated
   (e.g. *"hostnames must be invented (doc-…) or honestly empty, not captured: {name}"*) and
   the doc stating why an empty string is trivially synthetic (it identifies nothing — the
   measurement's EMPTY form must be committable, story 4.17). **Prove-to-red:** *(i)* the
   natural red — with the new stream on disk and the walk unamended,
   `the_corpus_carries_no_real_network_data` reds naming the empty hostname; *(ii)* the
   boundary — an in-memory `#[should_panic(expected = "hostnames must be invented")]` test pins that a
   non-empty, non-`doc-` name (e.g. `"printer-salon"`) still reds — the expected substring
   guards against a pass-for-the-wrong-panic (4.14's idiom). (This is the corpus's second sanctioned privacy
   amendment; 4.14's VRRP range was the first — same ritual, same honesty.)

6. **The stream is committed as 6 observations** —
   `fixtures/scenario/replay/hostname-absence.jsonl`, fresh prefix `bcbcbcbc` (first full id
   `bcbcbcbc-0000-4000-8000-000000000001`, then `…0002`–`…0006`), the corpus UUIDs
   (`connector_id` `33333333-3333-4333-8333-333333333333`, `l2_domain`
   `11111111-1111-4111-8111-111111111111`, `vantage` `22222222-2222-4222-8222-222222222222`),
   `raw: null`, instants `2026-01-11T00:00:00Z` / `T00:05:00Z` / `T00:10:00Z` / `T00:15:00Z`
   / `T00:20:00Z` / `T01:00:00Z`:
   - **G1** (`…0001`, 00:00): `Mac [2,0,94,0,83,200]` flag `true` · `IpV4 192.0.2.70` ·
     `Hostname { "", Dhcp }` — 3 facts (EMPTY).
   - **G2** (`…0002`, 00:05): `Mac [2,0,94,0,83,201]` flag `true` · `IpV4 192.0.2.71` ·
     `Hostname { "", Dhcp }` — 3 facts (EMPTY — the false-agreement bait).
   - **G3** (`…0003`, 00:10): `Mac [2,0,94,0,83,202]` flag `true` · `IpV4 192.0.2.72` ·
     `Hostname { "", Dhcp }` · `Uplink` peer `[2,0,94,0,96,10]` port `swport-31` — 4 facts
     (EMPTY, behind the shared port).
   - **G4** (`…0004`, 00:15): `Mac [2,0,94,0,83,203]` flag `true` · `IpV4 192.0.2.73` ·
     `Uplink` peer `[2,0,94,0,96,10]` port `swport-31` — 3 facts, NO Hostname (MISSING,
     behind the same port — identical `Uplink` bytes to G3's).
   - **G5** (`…0005`, 00:20): `Mac [2,0,94,0,83,204]` flag `true` · `IpV4 192.0.2.74` ·
     `Hostname { "doc-host-india", Dhcp }` — 3 facts (the named box).
   - **G6** (`…0006`, 01:00): `Mac` value-identical to G5's · `IpV4 192.0.2.74` value-identical
     · — 2 facts, NO Hostname (the name fell silent).
   A byte-pin test in `fixtures.rs` (all standing review lessons): 6 obs; fact counts
   3/3/4/3/3/2; the six obs_ids pinned; the three empty hostnames value-pinned
   (`Fact::Hostname { name: "".into(), source: Dhcp }` on G1, and G2's asserted EQUAL to
   G1's; G3's likewise EQUAL); G4 and G6 asserted to carry NO Hostname fact
   (`.iter().all(|f| !matches!(f, Fact::Hostname { .. }))` — the MISSING form is an
   assertion, not an accident); G1/G2 MACs and IPs value-pinned and distinct; **G3/G4 MACs and IPs value-pinned and
   distinct too** (validation's HIGH — without them an accidental shared MAC would collapse
   the abstain pair into an `l1-exact-mac` pair while every assertion stayed green); G3/G4
   uplinks asserted EQUAL and value-pinned as the WHOLE `Uplink` fact (peer `[2,0,94,0,96,10]`
   AND port `swport-31`); G5's three facts value-pinned; G6 == G5 on
   `Mac` and `IpV4`; flags true everywhere; the instants VECTOR via `ts()`.

7. **The three traps are committed — both poles present, the abstain counting for neither
   (DR1).** `fixtures/scenario/traps/hostname-absence.toml`, `family = "hostname-absence"` on
   all three: `hostname-absence-must-not-merge` judging `[…0001, …0002]`,
   `hostname-absence-must-abstain` judging `[…0003, …0004]` (cause `NoObservedValue`),
   `hostname-absence-must-merge` judging `[…0005, …0006]`. `incomplete_families` stays green
   (≥1 merge + ≥1 not-merge; the abstain is neutral per DR1 — the second three-column family
   after shared-hardware-vm). Multi-line observations, full UUIDs, ids whole on their lines.

8. **Reasons**: one sentence each, 20–300 chars, no MAC octets, no spec ids, descriptive
   values (the standing template): the must-not-merge names the two empty names and the
   opposing MACs; the must-abstain names the empty-vs-missing equivalence and the
   undiscriminated shared port; the must-merge names the identical MAC and address and the
   name that fell silent — G5→G6 is FORTY MINUTES (00:20 → 01:00): do not copy
   hostname-collision's "one hour apart" phrasing, it would be false here.

9. **Corpus lock + count coupling, red first.** Manifest 21 → 23 (`"23 fixture(s) match
   their recorded sha256 (0 generated, 23 hand-authored)"`); count literals **21 → 24** (this
   family adds THREE — like 4.11 and 4.14) at the three sites (:393/:411/:429 as of story creation —
   names anchor; breakdown comment tail REPLACED "…two — twenty-one…" → "…two and
   `hostname-absence.toml` (story 4.17) three — twenty-four…"; `"24 trap(s) discovered"`;
   "stays 24"); red observed at `left: 24, right: 21` first. Reproducibility and scratch
   tests untouched. Mid-story orphan red expected until the manifest bump.

10. **Synthetic-only, fresh, verified**: `bcbcbcbc`, `doc-host-india`, MAC last-bytes
    `83,200`–`83,204`, IPs `.70`–`.74`, `swport-31` — all grep-free at story creation;
    re-verify at dev. The empty hostname is the ONLY non-`doc-` name in the corpus and it is
    admitted by the amended rule.

## Tasks / Subtasks

- [x] **Task 1 — byte-pin test, RED** (AC: 6): e.g.
      `the_hostname_absence_stream_encodes_empty_and_missing_and_never_null`, end of the
      fixtures.rs test module; red = `FixtureError::Io`. Record.
- [x] **Task 2 — the stream** (AC: 6, 10): six lines per AC6 (templates: hostname-collision
      lines for the 3-fact shape, vrrp/docker-veth for the 4-fact Uplink shape; G6 is
      Mac+IpV4 only). Re-verify frees. Trailing newline. Byte-pin still red on content until
      exact; drive green. Then `the_corpus_carries_no_real_network_data`: **RED naming the
      empty hostname** — the natural red of AC5. Record the exact message.
- [x] **Task 3 — amend the privacy walk** (AC: 5): the one-line rule change + message + doc;
      the `#[should_panic]` boundary test (`"printer-salon"`); walk greens over the new
      stream. Record both.
- [x] **Task 4 — the trap file** (AC: 1, 2, 3, 4, 7, 8): header per AC4 (D45 + the
      measurement quote + the structural-null note + the two scope seams); three `[[trap]]`
      blocks; reasons measured and recorded.
- [x] **Task 5 — count red, 21 → 24** (AC: 9): red recorded, three literals + two comments.
- [x] **Task 6 — manifest bump 21 → 23** (AC: 9): two entries, hash after final byte,
      twenty-one existing entries untouched.
- [x] **Task 7 — gates** (AC: 9, 10): fmt · clippy · test --workspace · xtask ci (quote the
      23/23 message). Residual grep `"21 trap"` / `discovered(), 21` / `stays 21` /
      `twenty-one` → no hits. `Cargo.lock` unchanged; `architecture-views.md` and
      `traps/README.md` untouched. **Add the "Implemented (dev-story)" Change Log row — the
      4.15 AND 4.16 reviews both had to demand it; do not make it three.**

## Dev Notes

### The shape of this story in one paragraph

The last measurement-grounded family of the epic and the second three-column family: three
traps pinning one equivalence — MISSING ≡ EMPTY ≡ "no signal", never `null` — plus the
corpus's second sanctioned privacy amendment (one line: an empty name is trivially synthetic).
Nothing coined (`l1-distinct-mac`, `l1-exact-mac`, cause `NoObservedValue` all established);
the tempting `l2-hostname-agrees` named by no expectation. The family closes the hostname
triptych: 4.15 (present-but-colliding), 4.17 (absent-and-empty), F51's bound recorded between
them.

### The equivalence is the family (keep it straight while authoring)

| Pair | Shapes | Column | The pin |
|---|---|---|---|
| `[G1, G2]` | EMPTY vs EMPTY | must-not-merge / `l1-distinct-mac` | `"" == ""` is not agreement |
| `[G3, G4]` | EMPTY vs MISSING | must-abstain / `NoObservedValue` | empty COUNTS AS no-observed-value |
| `[G5, G6]` | NAMED vs MISSING | must-merge / `l1-exact-mac` | absence opposes nothing |

The abstain's cause spelling is the load-bearing subtlety: `NoObservedValue` on a pair where
one side carries a byte-present `Hostname` fact — that IS the assertion that empty counts as
absent. An engine (or reviewer) proposing `ConflictingObservations` there has missed the
family's point: `""` vs missing is not a conflict, it is the same silence twice.

And the honest mechanics (validation): **the cause is RECORDED truth, not scored** — `score()`
ignores causes by design (score.rs's own doc and test: an abstention with the WRONG cause still
passes the column; only rules are compared). So the gate demolishes an engine that DECIDES on
[G3, G4]; an engine abstaining with `ConflictingObservations` passes the gate and is wrong only
against the recorded truth — Epic 5's cascade owns that comparison. Do not claim more in the
completion record. [[claims-must-match-verification]]

### Why G3/G4 abstain while G1/G2 refuse (and multi-nic does neither)

Follow the committed precedents, they are consistent: behind an IDENTICAL uplink (one port —
the hypervisor/shared-port shape) the hostname is the needed discriminator, and its absence
abstains (shared-hardware-vm `[W1, W4]`); with NO topology at all and distinct MACs, L1's
opposition decides (randomized-mac / hostname-collision); multi-nic's uplink-agreement merge
is the different-ports-same-switch shape, not this one. G1/G2 carry no Uplink; G3/G4 carry
identical Uplinks. The header states which precedent each trap follows.

### Previous story intelligence (4.16)

- The review's doc patch widened `Expectation::MustNotMerge` to rule-scoped truth — no
  scoping question arises here (all three pairs are device-level questions).
- Standing byte-pin rules: obs_id ↔ line binding pinned; value-pin everything a reason
  cites; instants as VECTOR; `facts.len()` with justification comment; "value-identical"
  wording (not "byte-identical") for parsed-equality claims.
- The Change Log "Implemented" row: demanded by BOTH 4.15's and 4.16's reviews — Task 7
  makes it explicit this time.
- Stale-binary trap on this Synology mount: `touch` edited files if a count test behaves
  impossibly. [[local-gate-must-mirror-ci]]
- PR workflow: branch → PR → CI → squash merge.

### Project Structure Notes

- **NEW (locked):** `fixtures/scenario/replay/hostname-absence.jsonl` (6 obs),
  `fixtures/scenario/traps/hostname-absence.toml` (3 traps). Manifest 21 → 23.
- **Updated:** `fixtures.rs` — test module only (the Hostname-arm rule change + message +
  doc, the boundary test, the byte-pin test); `trap_gate.rs` (three literals 21 → 24 + two
  comments); `MANIFEST.toml`.
- **Unchanged:** all production paths, `Cargo.lock`, twenty-one existing manifest entries,
  `traps/README.md`, `architecture-views.md`, `is_synthetic_mac` (4.14's amendment is
  untouched — this one is the Hostname arm); `fixture_connector.rs`'s sibling `doc-` rule
  (`every_fact_this_module_authors_is_synthetic`) untouched — it walks module-AUTHORED facts
  only, never the corpus, and none of them is empty (the grep for `starts_with("doc-")` hits
  TWO sites; only the fixtures.rs one is amended).
- **Out of scope:** the WIRE-level hostname behaviours (JSON missing vs `""` under the real
  parser — 4.18/Epic 11); `Fact::OuiVendor`-empty traps (the measurement's `oui` row — 4.18's
  wire territory); any hostname normalization rule (Epic 5); F51's bound (4.15 carries it).

### Traps (mistakes this story must not make)

1. **Encoding a `null` hostname anywhere** — the family's own point; `Fact::Hostname` cannot
   even represent it, and the header says so as a structural guarantee.
2. **`ConflictingObservations` on the abstain.** Empty-vs-missing is the same silence twice;
   `NoObservedValue` IS the assertion (the Dev Note table).
3. **Giving G4 an empty hostname or G3 none** — the pair must hold ONE of each shape, or the
   equivalence is untested.
4. **Giving G1/G2 uplinks (they would drift toward the abstain shape) or G3/G4 different
   ports (they would drift toward multi-nic's merge shape).**
5. **A `doc-` name on G1/G2/G3** — the EMPTY form is the point; and a non-empty non-`doc-`
   name anywhere reds the amended walk (that red is the boundary test's job, in memory).
6. **Widening the privacy amendment beyond `is_empty()`** — one predicate, proven at the
   boundary; whitespace-only names (`" "`) still red, deliberately: the measurement records
   `""`, not padding.
7. **Naming `l2-hostname-agrees` in any expectation** — it is the temptation of AC1, prose
   only.
8. **Forgetting this family adds THREE** (21 → 24, like 4.11) while the manifest adds two
   (21 → 23) — the two counters diverge for the first time since 4.11; grep both.
9. **Hashing before the final byte; skipping any red; touching the reproducibility test.**
10. **Omitting the "Implemented" Change Log row** (the 4.15/4.16 recidive — Task 7).

### References

- [Source: _bmad-output/planning-artifacts/epics.md:1205-1218 — Story 4.17: encode MISSING
  and EMPTY, never null (D45); each form carries its expected outcome and reason]
- [Source: _bmad-output/planning-artifacts/architecture.md:4310-4325 — the wire-format
  measurement table: hostname "MISSING and empty both occur · null NEVER… must NOT encode
  null, or it tests a case the source cannot produce"; D45's "gate on a false truth" at
  :4313-4314]
- [Source: _bmad-output/planning-artifacts/architecture.md:4184-4188 — the measurement's
  reservation 1 (the population F51 counts — recorded in 4.15's header, cross-referenced
  here)]
- [Source: fixtures/scenario/traps/shared-hardware-vm.toml — the must-abstain precedent
  (`[W1, W4]`, identical uplink, `NoObservedValue`) and the three-column family shape]
- [Source: fixtures/scenario/traps/hostname-collision.toml — the sibling family (present-but-
  colliding) whose header carries F51; the scope seam stated in both]
- [Source: crates/opencmdb-bin/src/fixtures.rs — the `Fact::Hostname` arm this story amends
  (`starts_with("doc-")`), `assert_facts_are_synthetic`, the corpus walks]
- [Source: crates/opencmdb-core/src/observation/mod.rs:154-158 — `Fact::Hostname { name:
  String, source }` — `String`, not `Option<String>`: null is unrepresentable, the
  structural note of AC4]
- [Source: crates/opencmdb-bin/src/trap_gate.rs — count literals 21 → 24 (three sites + two
  comments)]
- [Source: _bmad-output/implementation-artifacts/4-16-trap-ephemeral-docker-veth.md — prior
  story: the rule-scoped MustNotMerge doc, the standing byte-pin rules, the Change Log
  lesson]

## Dev Agent Record

### Agent Model Used

claude-fable-5 (Claude Fable 5)

### Debug Log References

- **Task 1 natural RED**: `the_hostname_absence_stream_encodes_empty_and_missing_and_never_null`
  → `FixtureError::Io { path: ".../hostname-absence.jsonl", NotFound }` inside `read_jsonl`.
- **Byte-pin GREEN** after the stream landed.
- **AC5 natural RED** (walk unamended, stream on disk):
  `the_corpus_carries_no_real_network_data` panicked at the Hostname arm with the exact
  message `".../hostname-absence.jsonl: hostnames must be invented, not captured: "` — the
  empty name after the colon IS the red's evidence.
- **AC5 boundary**: `a_captured_looking_hostname_is_still_refused` —
  `#[should_panic(expected = "hostnames must be invented")]` on the in-memory
  `"printer-salon"` — passes (the amended rule still reds non-empty non-`doc-` names).
- **Reason lengths** (awk length): must-not-merge **226**, must-abstain **248**, must-merge
  **237** — inside 20–300, one line, no MAC octets, no spec ids.
- **Count-coupling RED**: assertions at 21 with both files on disk →
  `left: 24 / right: 21`. Then 21 → 24 at the three sites + two comments (breakdown tail
  REPLACED — "…two and `hostname-absence.toml` (story 4.17) three — twenty-four").
- **`bcbcbcbc` grep**: only the two new files under fixtures/. Frees re-verified
  pre-authoring (`doc-host-india`, `83,200`–`204`, `.70`–`.74`, `swport-31` — no hits).
- **Hash-after-final-byte held**: trailing `0a` on both, then `sha256sum`:
  `454b2d90254a71e9f852a7fc028fa3e616c82880d7df49945da2ad9e387ddbb6` (jsonl),
  `234f95fd789b6b19a20ca64578c221e09c1deb61c6e424930ea68c9b96429a30` (toml). No edit after.
- **Gates** (all green): fmt · clippy `--all-targets --locked -- -D warnings` ·
  `cargo test --workspace --locked` → **118 (bin) + 86 (core) + 42 (xtask), 0 failed** ·
  `cargo run -p xtask --locked -- ci` → fixtures verbatim **"23 fixture(s) match their
  recorded sha256 (0 generated, 23 hand-authored)"**; views-hash `ℹ STALE` by design.
  Residual grep (`"21 trap"` / `discovered(), 21` / `stays 21` / `twenty-one`) → no hits.
  `Cargo.lock` untouched.

### Completion Notes List

- The family landed exactly as scoped: two NEW locked artefacts (6-obs stream — the corpus's
  largest — and a 3-trap file, the second three-column family), manifest 21 → 23, count
  literals 21 → 24, the second sanctioned privacy amendment (one predicate:
  `name.is_empty() || starts_with("doc-")`, natural red + boundary `should_panic`), and ONE
  byte-pin test. Nothing coined; `NoObservedValue` reused with its established spelling.
- **AC1**: `[G1,G2]` refuse on `l1-distinct-mac` — `"" == ""` is not agreement; the tempting
  rule named by no expectation.
- **AC2**: `[G3,G4]` abstain with `NoObservedValue` behind the identical pinned Uplink —
  the byte-present empty name COUNTS AS absent (shared-hardware-vm's column precedent).
- **AC3**: `[G5,G6]` merge on `l1-exact-mac` — the name fell silent forty minutes later
  (the hostname-collision "one hour" phrasing deliberately NOT copied).
- **AC4**: the header carries the measurement charter verbatim, D45's sentence, the
  structural-null note (`String`, not `Option<String>`), both scope seams (4.15's F51 /
  4.18's wire layer), AND the cause-is-recorded-truth honesty note (score() ignores causes —
  the gate demolishes deciders, not wrong-cause abstainers).
- **AC5**: both privacy reds recorded above; the amendment is one predicate, the message
  names the closed rule, whitespace-only names still red.
- **AC6**: byte-pin pins the six obs_ids, fact counts 3/3/4/3/3/2, the three empty hostnames
  (value + carried equalities), the two MISSING assertions, all four G1–G4 MACs/IPs pinned
  and pairwise distinct, the whole Uplink fact pinned on G3 and equal on G4, G5's three
  facts, G6 == G5, the instants vector.
- **AC7–AC10**: three traps, both poles + neutral abstain (DR1); reasons 226/248/237;
  deliberate bump red-first; every value synthetic and fresh.

### File List

- `fixtures/scenario/replay/hostname-absence.jsonl` — NEW: 6-observation replay stream
- `fixtures/scenario/traps/hostname-absence.toml` — NEW: 3-trap family file (D45 record)
- `fixtures/MANIFEST.toml` — modified: two entries appended (21 → 23)
- `crates/opencmdb-bin/src/fixtures.rs` — modified, test module only: the Hostname-arm
  amendment (+ doc + message), the boundary `should_panic` test, the byte-pin test
- `crates/opencmdb-bin/src/trap_gate.rs` — modified: three literals 21 → 24 + two comments
- `_bmad-output/implementation-artifacts/4-17-trap-hostname-absent-or-empty.md` — this story
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status tracking

## Change Log

| Date       | Change                                                                 |
|------------|------------------------------------------------------------------------|
| 2026-07-25 | Story 4.17 drafted (create-story, autonomous run): the hostname-absence family — three traps pinning MISSING ≡ EMPTY ≡ "no signal", never null (D45; `Fact::Hostname`'s `String` makes null unrepresentable — the structural note). must-not-merge [G1,G2] on `l1-distinct-mac` (`"" == ""` is not agreement); must-abstain [G3,G4] with `NoObservedValue` — the load-bearing subtlety: the cause asserts that a byte-present empty name COUNTS AS absent (shared-hardware-vm's column precedent); must-merge [G5,G6] on `l1-exact-mac` (a name that fell silent opposes nothing). Second sanctioned privacy amendment: `name.is_empty() || starts_with("doc-")`, natural red + boundary `should_panic`. 6-obs stream `bcbcbcbc`; traps 21 → 24 (the counters diverge from the manifest's 21 → 23, as in 4.11 and 4.14). Status → ready-for-dev. |
| 2026-07-25 | Validated (two fresh-context agents: fact-check + gap-hunt). 1 HIGH / 4 MED / 8 LOW, all applied. The HIGH: G3/G4's MACs and IPs were unpinned — an accidental shared MAC would collapse the abstain pair into an exact-MAC pair silently; now value-pinned and distinct. MEDs: score() IGNORES abstention causes (the cause is recorded truth, not gate mechanics — an engine abstaining with the wrong cause passes the gate; said honestly in the Dev Notes); "first divergence since 4.11" was false (4.14 also added +3/+2); "byte-identical" re-banned (the 4.16 wording rule); D45 anchor :4313-4314. LOWs: three (not two) empty hostnames; the SECOND `starts_with("doc-")` site named as untouched (fixture_connector's module-authored walk); `should_panic` expected-substring; the whole-Uplink pin; the forty-minutes phrasing warning (do not copy "one hour apart"). |
| 2026-07-25 | Implemented (dev-story): all 7 tasks, ATDD held — byte-pin RED (`FixtureError::Io`), stream landed and greened, privacy walk's natural RED recorded (its message ends with the empty name — the evidence), one-predicate amendment + boundary `should_panic` ("printer-salon"), trap file landed (header: measurement charter, D45, structural null, both scope seams, cause-is-recorded-truth), count RED `left: 24, right: 21` then 21 → 24, manifest 21 → 23 (sha256 after final byte: `454b2d90…`, `234f95fd…`). Reasons 226/248/237. Gates green: fmt, clippy `-D warnings`, `cargo test --workspace` (118+86+42), `xtask ci` ("23 fixture(s) match their recorded sha256"). Status → review. |
| 2026-07-25 | Code review (3 fresh-context layers). **Auditor: PASS 10/10** — every Dev Record claim reproduced, and ALL THREE reds REPLAYED by the auditor (the byte-pin's `Io`, the privacy walk's natural red on the reverted predicate, the count red), tree restored clean each time. 0 CRITICAL/HIGH/MED in the diff itself; **1 patch applied**: the must-abstain reason's "shared port" tightened to "shared uplink" (the pin is the whole Uplink fact — Edge F1; reason now 250 chars, toml re-hashed `294257bb…4ed7`, MANIFEST updated). 2 no-action LOWs (boundary-test substring matches the old message too — 4.14's idiom held; header cites sources by name without line anchors — the tolerant precedent). **Blind Hunter's MED is environmental and recorded for the epic report:** the local suite is non-deterministic under Synology Drive sync — the corpus was transiently replaced by a stale server state mid-run (8 failures across 5 runs with identical sha256 and clean git status, then 15+ green runs, `cloud-drive-daemon` active on the tree) — CI (clean checkout) unaffected; joins [[local-gate-must-mirror-ci]]. Status → done. |
