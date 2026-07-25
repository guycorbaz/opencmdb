# Story 4.18: Wire-format traps written from measurement

Status: done

## Story

As the author of the trap corpus,
I want the wire-format traps derived from the measured payload rather than from belief,
so that **their red can actually arrive** (epic 4.18, D45): the corpus commits a synthetic body
whose every field behaviour is a MEASUREMENT, plus the Observations the future UniFi parser MUST
produce from it — the oracle written before the code it judges — and a shape test that guards
both TODAY, while the harness that runs them under the real parser lands with Epic 11.

## Decision record (party mode, 2026-07-25 — Winston, Murat, John; Amelia not convened)

The roundtable Guy's mandate required for the Epic-11-gated pair. Unanimous on the shape;
one point arbitrated. Full positions in the epic report.

1. **4.18 is authored NOW.** All of its content is measurement, not belief; the private captured
   body is uncommittable, so a SYNTHETIC body carries the measured behaviours ("shapes and
   conclusions, never the values" — the standing scrub rule). The "expected variant" is expressed
   as **expected Observations** — D19's frozen schema is the parser's output contract, so the
   spec constrains Epic 11 rather than the parser certifying itself ("the worst oracle: tests
   green from birth" — Murat). **Murat's red line, adopted as the story's stop criterion:** every
   expectation must be DERIVABLE from the frozen schema, the measurement, or committed corpus
   doctrine — where it would require an undecided parser choice, the hole is NAMED in the
   charter, never guessed ("the moment you write 'the parser should probably…', stop" — John).
2. **4.19 is SPLIT.** Delivered WITH this story (4.19a): the drift-surface record (127 payload
   keys vs 7 `Fact` variants — a measurement) and the LAYER CHARTER as binding constraints on
   Epic 11 (a renamed field must produce an explicit error, never a silently empty collection;
   `#[serde(default)]` forbidden on presence-feeding collections; layer-A drift injection is
   theatre — D35 verbatim). Deferred to Epic 11 (4.19b): the mutation GENERATOR, the ~30
   generated fixtures, and their expected outcomes — expected outcomes for an undefined error
   taxonomy are "written from belief" (D45), and a generator has no test that reds without the
   parser it attacks (Winston's argument, arbitrated over Murat's generator-now — the house
   rule "no guard without a red" decides).
3. **Epic 4 closes DONE with the re-scope recorded, not silent** (John: "an epic in-progress
   that doesn't progress is noise that erodes the whole status file"): a correct-course
   sprint-change record, a GitHub issue for Epic 11, dated notes in epics.md, and
   sprint-status entries that say what was delivered and what moved — never a bare "done".

## Acceptance Criteria

1. **The synthetic wire body encodes every measured behaviour and nothing the source cannot
   produce.** `fixtures/scenario/wire/unifi-clients.json` — a UniFi-style clients response
   (`{"meta":{"rc":"ok"},"data":[…]}`) with FOUR client entries, every MEASURED field behaviour closed by
   the measurement table (architecture.md:4311-4326) — and the parts the table never covered
   (the `meta`/`rc`/`data` ENVELOPE and the `ip` key that carries the IpV4 expectation) marked
   as vendor-convention HOLES in the charter, confirmed or bumped at Epic 11's first real
   capture (validation's HIGH: the measurement lists field behaviours, not the envelope): `mac` lowercase colon-separated on 100%
   of entries; `last_seen` a 10-digit SECONDS epoch; `oui` present on all and EMPTY on three of
   four (the "large share"); `vlan` key absent everywhere; `network_id` a fixed 24-char string
   with exactly ONE distinct value across entries; `hostname` present, EMPTY, and MISSING each
   occurring (4.17's shapes) and null NEVER; `is_wired` bool with both values occurring (the bool-100% is measured; "both values occur"
   is the one inference the charter admits as such — the split being total implies it);
   `sw_port` an integer of 1–2 digits on the wired entries — and on wired ONLY, because the
   presence rate on wireless was NOT measured: the certain case is encoded, the hole is named
   in the charter (D45 both ways). Placement note: `scenario/wire/`, NOT `capture/` — the
   capture charter says real payloads that rot; this body is a SPEC (right or wrong, never
   rots), a declared deviation from the architecture tree's capture/ placement which assumed a
   committable real body.
2. **All values synthetic**: MACs `02:00:5e:00:53:{dc,dd,de,df}` (bytes 220–223, fresh,
   locally-administered so the free-text scan admits them); IPs `192.0.2.90`–`.93` (fresh);
   hostnames `doc-host-juliett` / `""` / (absent) / `doc-host-kilo`; `oui` `"doc-vendor"` on c1
   and `""` on c2/c3/c4; `is_wired` true on c1/c3 (with `sw_port` 7 and 3) and false on c2/c4
   (no `sw_port` — the named hole); `network_id` `"doc-network-000000000001"` (24 chars); `last_seen`
   `1768176000/+300/+600/+900` (= `2026-01-12T00:00:00Z` + 5-minute steps).
3. **The expected Observations are committed beside it** —
   `fixtures/scenario/wire/unifi-clients.expected.jsonl`, 4 lines, prefix `bdbdbdbd`
   (`bdbdbdbd-0000-4000-8000-000000000001`…`0004`), the corpus UUIDs for
   connector/l2_domain/vantage: per client, the facts DERIVABLE without guessing —
   `Mac` (bytes from the wire mac, flag from the U/L bit), `IpV4`, `Hostname` mapped by 4.17's
   committed doctrine (wire `""` → fact with `""`; wire key MISSING → no fact; c1
   `doc-host-juliett`, c2 `""`, c3 none, c4 `doc-host-kilo`; `source: Dhcp` marked provisional
   in the charter), `OuiVendor` mapped the same way EXCEPT the measurement's own words ("a
   named Fact that is usually absent") make ""→fact-with-"" a 4.17-doctrine derivation, not a
   certainty — named in the charter as a recorded-bump candidate; `observed_at` = the epoch
   seconds, exactly (`2026-01-12T00:00:00Z` / `T00:05` / `T00:10` / `T00:15`). **NO `Uplink`**
   (the measurement never covered `sw_mac` — hole named). **The identity/scope columns are
   HARNESS CONTEXT, not expectations**: the `bdbdbdbd` obs_ids and the standard corpus scope
   UUIDs are placeholders marking the hole (a real parser generates ids and maps scope at
   runtime); the charter states Epic 11's runner compares FACTS + `observed_at`, injecting its
   own context.
4. **The wire charter (and 4.19a's deliverables) live in `fixtures/scenario/wire/README.md`**
   (orphan-exempt like every corpus README, appendable): *(a)* what this area is — D35 layer B's
   spec half, written before the parser, run by Epic 11's harness; *(b)* the measurement table's
   behaviours restated with the source citation; *(c)* **the named holes**: hostname `source`
   attribution provisional; `OuiVendor` empty-vs-absent mapping derived from 4.17 doctrine,
   revisable by deliberate bump; no `Uplink` (sw_mac unmeasured); `sw_port`-on-wireless
   unmeasured; ids/scope = harness context; *(d)* **4.19a — the drift-surface record**: the
   payload carries 127 distinct keys where `Fact` names 7 — that ratio IS the drift surface
   D35's mutations exist to cover; *(e)* **4.19a — the binding layer charter for Epic 11**: a
   renamed field must yield an explicit error, never a silently empty collection;
   `#[serde(default)]` is FORBIDDEN on any collection feeding presence; injecting a drift error
   at layer A "tests nothing — it asserts the engine handles an error you handed it, without
   proving the parser produces one. That is the most insidious theatre of all" (epics.md's
   4.19 AC restating D35 — architecture.md:2032-2034 carries D35's own wording; cite the one
   you quote); *(f)* the `bdbdbdbd` obs_id prefix is RESERVED here — the cross-stream uniqueness walk
   covers `scenario/replay/` only and cannot see this directory (M4); *(g)* the
   4.19b re-scope: generator + ~30 mutations + expected outcomes land WITH the parser (Epic
   11), because outcomes for an undefined error taxonomy would be belief (D45) — with the
   pointer to the correct-course record and the GitHub issue.
5. **A shape test guards both artefacts TODAY** (the fixture must survive months un-run —
   Murat). In `fixtures.rs`'s test module, e.g.
   `the_wire_spec_encodes_the_measured_field_behaviours`, red-first (`FixtureError::Io`-style
   natural red is not available for a JSON body — the test reads via `std::fs` +
   `serde_json::Value`; its red is the missing file's io error, recorded): asserts on the BODY —
   4 entries; every `mac` matches lowercase-colon form; every `last_seen` is a 10-digit
   integer; NO `vlan` key on any entry; every `network_id` is 24 chars and the set of distinct
   values has size 1; `hostname` present/empty/missing each occurring and `null` occurring
   NEVER (walk every entry: if the key exists its value is a string, never `Value::Null`);
   `oui` present on all, empty on exactly 3; `is_wired` bool on all, both values present;
   `sw_port` present iff wired, integer, 1–2 digits. Asserts on the EXPECTED stream (read via
   `read_jsonl`) — 4 observations, obs_ids pinned, the three CONTEXT UUIDs (connector/l2_domain/vantage)
   pinned per line (they are placeholders — all the more reason they cannot drift, M2), fact
   multisets closed per line (4/4/3/4 with per-kind extraction), every value pinned, instants
   = the epochs converted.
   **Asserts CONNECTING the two** (the derivation as executable spec): per index, the expected
   `Mac` bytes re-derived from the body's `mac` string; the expected hostname
   presence/emptiness mirrors the body's key state; the expected `IpV4` equals the body's
   `ip` string parsed, and the expected `OuiVendor` equals the body's `oui` string, per index
   (M3 — without these two the pair can drift while both halves stay green); the expected
   `observed_at` equals the body's `last_seen` read as epoch seconds. **Privacy, mechanically** (Winston's condition):
   the RAW body text routed through `assert_text_is_synthetic` (every MAC/IP token held to the
   corpus rule) and the expected stream's facts through `assert_facts_are_synthetic` (the
   walks don't reach `scenario/wire/` — this test IS the privacy coverage, say so in its doc).
6. **The corpus lock covers the pair**: both files enter `MANIFEST.toml` (23 → 25; gate
   message `"25 fixture(s) match…"`), sha256 after final byte, each entry's comment carrying
   **"consumer pending: Epic 11"** (Murat's visible-promise hook). Trap counts are UNTOUCHED
   (24 — these are wire artefacts, not `[[trap]]`s; `trap_gate.rs` does not move). The
   `capture/` README gains one dated line pointing to `scenario/wire/` for the synthetic spec
   half (so its "empty until Epic 11" sentence stays true for CAPTURES only).
7. **Gates**: fmt · clippy · test --workspace · xtask ci (quote the 25/25 message);
   `Cargo.lock` unchanged; `architecture-views.md` and `traps/README.md` untouched — but `fixtures/README.md` and
   `fixtures/scenario/README.md` ARE touched (H2: their charters must stay true); residual
   greps clean; the "Implemented" Change Log row written at dev time (the 4.15/4.16 lesson).

## Tasks / Subtasks

- [x] **Task 0 — open the Epic 11 GitHub issue FIRST** (AC: 4f): "Epic 11: run the 4.18 wire
      spec under the real parser + implement 4.19b (mutation generator, ~30 fixtures, expected
      outcomes)" — so the wire README can cite the real issue number (validation's M1); the
      correct-course record is cited by title and completed at epic closure.
- [x] **Task 1 — shape test skeleton, observe RED** (AC: 5): the io-error red on the missing
      body, recorded.
- [x] **Task 2 — the wire body** (AC: 1, 2): four entries per AC1/AC2; trailing newline.
- [x] **Task 3 — the expected stream** (AC: 3): four Observation lines; test greens as its
      assertions land.
- [x] **Task 4 — the wire README + the two parent READMEs** (AC: 4): the wire charter (holes
      + 4.19a record + re-scope pointer); AND amend `fixtures/README.md` (its Layout table says
      scenario/ "proves the engine" and "both halves of scenario/ are discovered by walking" —
      both become false with wire/: add the wire row — proves the PARSER, read by the shape
      test, does not rot, structurally out of the re-capture job's reach) and
      `fixtures/scenario/README.md` (gains the wire/ paragraph) — validation's H2, the
      docs-current-before-push rule applied inside the corpus.
- [x] **Task 5 — the manifest bump 23 → 25 + capture README line** (AC: 6): hash after final
      byte; "consumer pending: Epic 11" on both entries.
- [x] **Task 6 — gates** (AC: 7).

## Dev Notes

### The shape of this story in one paragraph

The epic's last authored artefacts and its first WIRE-level ones: a synthetic measured body, the
Observations the parser must produce from it, a charter naming every hole, and a shape test that
makes the pair self-guarding for the months before Epic 11 runs it for real. Nothing is a
`[[trap]]`; trap counts and `trap_gate.rs` do not move. The derivability red line is the whole
discipline: measurement and committed doctrine may be encoded; parser design may only be HOLED.

### What is firm vs holed (the charter's table — keep it straight)

| Expectation | Basis | Status |
|---|---|---|
| Mac bytes + U/L flag | the wire string, mechanically | FIRM |
| IpV4 | the wire string | FIRM |
| observed_at = last_seen seconds | the measurement (10-digit SECONDS) | FIRM |
| hostname ""→fact-empty, missing→no fact | 4.17's committed doctrine | FIRM (doctrine) |
| hostname `source: Dhcp` | none — parser choice | HOLE (provisional value) |
| OuiVendor ""→fact-empty | 4.17 doctrine vs the measurement's "usually absent" wording | HOLE (doctrine-derived, bump-revisable) |
| Uplink | sw_mac never measured | HOLE (omitted) |
| obs_id / connector / scope | runtime context | HOLE (placeholders, PINNED by the test so they cannot drift) |
| the meta/rc/data envelope | vendor API convention, unmeasured | HOLE (confirm/bump at first Epic 11 capture) |
| the `ip` key (name/presence/format) | unmeasured — required to carry IpV4 | HOLE (same status) |
| sw_port on wireless | unmeasured | HOLE (body encodes wired only) |

### Previous story intelligence

- All standing byte-pin rules apply to the expected stream (obs_id binding, value pins, vector
  instants, `facts.len()` closure, "value-identical" wording).
- The Synology-Drive non-determinism (4.17's review): `touch` before concluding on any
  impossible test result.
- Reasons/traps machinery: NOT involved — no `[[trap]]`, no reason bounds, no family
  completeness. Do not touch `trap_gate.rs`.
- PR workflow unchanged.

### Traps (mistakes this story must not make)

1. **Encoding an unmeasured behaviour as fact** (a vlan key, a null hostname, sw_port on
   wireless, an Uplink) — D45 in both directions.
2. **Guessing parser design in the expected stream** — the moment "probably" appears, the item
   moves to the charter's hole table.
3. **Committing anything under `capture/`** — the spec half lives in `scenario/wire/`; the
   capture charter stays true.
4. **Touching trap counts or `trap_gate.rs`** — 24 stays 24; only the MANIFEST count moves.
5. **A real vendor OUI string or any non-`doc-` name** — the body is scanned by the test's
   privacy routing; keep every token synthetic.
6. **Skipping the body↔expected CONNECTING assertions** — without them the pair can drift
   apart while both stay internally consistent.
7. **Forgetting the "consumer pending: Epic 11" markers or the capture-README pointer line.**
8. **Claiming the wire spec is TESTED against a parser.** It is guarded in shape and privacy;
   its first real red arrives with Epic 11 — the weaker true sentence.
   [[claims-must-match-verification]]

### References

- [Source: _bmad-output/planning-artifacts/epics.md:1219-1256 — Stories 4.18 (:1219-1236)
  and 4.19 (:1237-1256, whose AC restates D35's theatre sentence at :1252-1253 — the wording
  AC4(e) quotes); the epic front-matter's "authored here, executable in Epic 11" clause at
  :836-838]
- [Source: _bmad-output/planning-artifacts/architecture.md:4311-4326 — the measurement table
  (every encoded behaviour's citation); :4313-4314 — D45's sentence]
- [Source: _bmad-output/planning-artifacts/architecture.md:2036-2038 — D35 layer B: raw bytes
  under the real parser; ~30 mutation fixtures; :3349 — the capture/mutations tree this story
  deviates from (declared)]
- [Source: fixtures/capture/README.md — the capture charter (real, rots, scrubbed) the
  synthetic spec must NOT violate — hence scenario/wire/]
- [Source: fixtures/scenario/traps/hostname-absence.toml — 4.17's committed doctrine the
  hostname/oui mappings derive from]
- [Source: crates/opencmdb-bin/src/fixtures.rs — `read_jsonl`, `fixture_path`,
  `assert_text_is_synthetic`, `assert_facts_are_synthetic` (the privacy routing the shape
  test reuses)]

## Dev Agent Record

### Agent Model Used

claude-fable-5 (Claude Fable 5)

### Debug Log References

- **Task 0**: GitHub issue **#34** opened FIRST ("Epic 11: run the 4.18 wire spec under the
  real parser + implement 4.19b"), so the wire README and both MANIFEST comments cite the real
  number.
- **Task 1 natural RED**: `the_wire_spec_encodes_the_measured_field_behaviours` → panicked at
  the read expect with `Os { code: 2, kind: NotFound }` on the missing body (the io red the
  story predicted — `fixture_path` checks shape, not existence).
- **GREEN** after Tasks 2–3: the body's four entries pass every measured-behaviour assertion,
  the expected stream's values pin, and the DERIVATION assertions (mac→bytes, ip→IpV4,
  oui→OuiVendor, hostname key-state mirror, last_seen→observed_at) connect the pair.
- **Privacy routing**: the raw body text through `assert_text_is_synthetic` and each expected
  line's facts through `assert_facts_are_synthetic` — both inside the shape test (the walks do
  not reach `scenario/wire/`; the test IS the coverage).
- **Hash-after-final-byte held**: trailing `0a` on both, then `sha256sum`:
  `a5239e155be8d69a0d96a6cb5afc5a4648b25dec7041e9e600d0534192b9024c` (body),
  `707f4443768733b152977a937edf266ad8a48698d8bb1d9125246806db385d2f` (expected). No edit after.
- **Gates** (all green): fmt · clippy `--all-targets --locked -- -D warnings` ·
  `cargo test --workspace --locked` → **119 (bin) + 86 (core) + 42 (xtask), 0 failed** ·
  `cargo run -p xtask --locked -- ci` → fixtures verbatim **"25 fixture(s) match their
  recorded sha256 (0 generated, 25 hand-authored)"**; `trap_gate.rs` untouched (24 stays 24);
  `Cargo.lock` untouched.

### Completion Notes List

- The wire pair landed as the party decision prescribed: a synthetic measured body
  (`scenario/wire/unifi-clients.json` — the declared deviation from the capture/ tree), the
  expected Observations (`unifi-clients.expected.jsonl`), the wire README carrying the
  charter, the NAMED-HOLES table (envelope + `ip` key included — validation's H1), 4.19a's
  drift-surface record (127 keys vs 7 variants) and binding layer charter, the `bdbdbdbd`
  prefix reservation, and the pointers to issue #34 and the correct-course record.
- **AC1/AC2**: every measured behaviour encoded, positions assigned (c1 wired+named+vendor,
  c2 wireless+empty-name+empty-oui, c3 wired+missing-name, c4 wireless+named); nothing
  unmeasured encoded (no vlan, no sw_port on wireless, no Uplink).
- **AC3**: expected facts all FIRM-or-holed per the table; context UUIDs and obs_ids are
  pinned placeholders.
- **AC4**: the wire README delivered with all seven sections (a–g).
- **AC5**: the shape test guards body, expected, derivation, privacy — and the two parent
  READMEs were amended so their charters stay true (H2): `fixtures/README.md`'s layout row
  and both-halves sentence, `fixtures/scenario/README.md`'s wire paragraph, plus the dated
  pointer line in `fixtures/capture/README.md`.
- **AC6/AC7**: MANIFEST 23 → 25 with "CONSUMER PENDING: Epic 11 (issue #34)" on both entries;
  trap counts untouched; gates green.
- The weaker true sentence, on the record: the wire spec is guarded in SHAPE and PRIVACY
  today; nothing runs it under a parser — its first real red arrives with Epic 11.

### File List

- `fixtures/scenario/wire/unifi-clients.json` — NEW: the synthetic measured wire body
- `fixtures/scenario/wire/unifi-clients.expected.jsonl` — NEW: the expected Observations
- `fixtures/scenario/wire/README.md` — NEW: the wire charter + named holes + 4.19a record
- `fixtures/README.md` — modified: layout row + both-halves sentence (H2)
- `fixtures/scenario/README.md` — modified: the wire paragraph (H2)
- `fixtures/capture/README.md` — modified: one dated pointer line
- `fixtures/MANIFEST.toml` — modified: two entries appended (23 → 25)
- `crates/opencmdb-bin/src/fixtures.rs` — modified: the shape test appended (tests only)
- `_bmad-output/implementation-artifacts/4-18-wire-format-traps.md` — this story
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status tracking

## Change Log

| Date       | Change                                                                 |
|------------|------------------------------------------------------------------------|
| 2026-07-25 | Story 4.18 drafted (create-story, autonomous run) after the party-mode decision (Winston/Murat/John): 4.18 authored NOW as spec-before-parser (synthetic measured body + expected Observations + shape test + charter with the named-holes table), 4.19 SPLIT (4.19a — drift-surface record + binding layer charter — delivered here; 4.19b — generator + mutations + expected outcomes — re-scoped to Epic 11 per D45/no-guard-without-a-red), Epic 4 to close DONE with the re-scope recorded (correct-course + GitHub issue + dated notes). Wire pair at `scenario/wire/` (declared deviation from the capture/ tree — the capture charter stays true), MANIFEST 23 → 25 with "consumer pending: Epic 11" markers, trap counts untouched. Status → ready-for-dev. |
| 2026-07-25 | Validated (two fresh-context agents). 2 HIGH / 5 MED / 4 LOW, all applied. The HIGHs: the meta/rc/data ENVELOPE and the `ip` key were beliefs presented as measurement — both are now named holes (the story had violated its own red line on its outermost bytes); and the two parent READMEs (`fixtures/README.md` "both halves… prove the engine", `fixtures/scenario/README.md`) would have become false — their amendment is now Task 4's second half. MEDs: issue opened FIRST so the README cites the real number; context UUIDs pinned by the test; oui/ip added to the connecting assertions; the `bdbdbdbd` reservation note (the uniqueness walk cannot see wire/). Plus citation-range and attribution fixes (epics.md:836-838 + :1219-1256; the theatre quote is epics.md's 4.19 AC, not architecture's D35 wording). |
| 2026-07-25 | Implemented (dev-story): issue #34 opened first; shape test RED (io NotFound) then GREEN over body + expected + derivation + privacy routing; wire README with charter/holes/4.19a; parent READMEs amended; MANIFEST 23 → 25 with consumer-pending markers (sha256 after final byte: `a5239e15…`, `707f4443…`). Gates green: fmt, clippy `-D warnings`, `cargo test --workspace` (119+86+42), `xtask ci` ("25 fixture(s) match their recorded sha256"); trap_gate untouched (24). Status → review. |
| 2026-07-25 | Code review — **PARTIAL COVERAGE, said plainly**: the Acceptance Auditor ran to completion (**AUDIT: PASS 7/7**, every Dev Record claim reproduced — both sha256 recomputed, suite 119+86+42, xtask message verbatim, the natural io red REPLAYED and the tree restored, issue #34 checked against what the story claims, the diff confirmed to be exactly the 10 declared files, trap_gate untouched at 24, all seven README sections a–g verified one by one, the five connecting assertions present). **The combined Blind/Edge-Case layer was cut off mid-pass by an API spend limit and produced no findings** — this story therefore carries ONE independent review layer, not the house's three. Recorded rather than glossed: the missing layer's usual catch (byte-level adversarial reading of the new fixtures) is partly covered by the Auditor's own recomputation, but not equivalently. **1 LOW from the Auditor, accepted as-is**: on the expected stream only the instants are restated literally as a second oracle — the fact VALUES are pinned by derivation from the sha256-locked body, so nothing can drift without a deliberate bump, but the house's deliberate-redundancy idiom is applied thinner here than in the trap families; worth revisiting if the body is ever bumped. Status → done. |
