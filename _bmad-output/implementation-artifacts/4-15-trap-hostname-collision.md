# Story 4.15: Trap family — hostname collision

Status: done

## Story

As the author of the trap corpus,
I want hostname collisions committed as traps,
so that **a signal present on fewer than half the population cannot be over-trusted** (epic 4.15,
F51): two distinct boxes wearing the same factory-default hostname must not fuse on the strength
of one weak signal — and an engine that reacts by distrusting every hostname agreement must still
merge the plain re-sighting the corpus puts beside it.

## Acceptance Criteria

1. **The primary form — the collision (epic AC1).** **Given** two distinct hosts H1 and H2 —
   distinct locally-administered MACs (`[2,0,94,0,83,160]` / `[2,0,94,0,83,161]`), distinct
   addresses (`192.0.2.50` / `.51`) — both reporting the SAME hostname `doc-printer` (two
   printers of one model shipping the same factory default; the realistic collision) **when**
   the trap is scored, **then** it is a **`must-not-merge`** naming **`l1-distinct-mac`**
   (REUSED — the rule that OPPOSES; randomized-mac's precedent): the temptation is
   `l2-hostname-agrees` (what shared-hardware-vm's must-merge legitimately rewards *behind a
   shared uplink*), and the reason says why the hostname alone is insufficient — it is one of
   L2's three named signals AND it is unusable on nearly half of known clients (F51,
   architecture.md:4367), so a signal that weak cannot outvote L1's opposing MACs.

2. **The negative form — anti-cowardice.** **Given** the same host re-seen — H3 carries H1's
   byte-identical `Mac`, byte-identical `IpV4` AND the same hostname, one hour later — **when**
   the trap is scored, **then** it is a **`must-merge`** naming **`l1-exact-mac`** (the rule
   that FIRES): an engine that generalizes the collision into "duplicate hostnames are common,
   distrust every hostname agreement" is demolished here — nothing opposes, and D18's middle
   column forbids the cowardly abstention. (Byte-kinship with cloned-mac's must-merge is
   acknowledged in the header: there the tested fear was clones, here it is collisions — the
   corpus tests temptations, not byte-shapes, 4.13's lesson.)

3. **The F51 record — the corpus records the bound (epic AC2).** The family header carries,
   with the citation: `hostname` is MISSING or empty on nearly half of known clients (never
   null — that non-case is 4.17's territory, architecture.md:4319), it is one of L2's three
   named signals (D12, architecture.md:891), and therefore **"the abstention rate is bounded
   below by this, not by the engine's quality"** (THAT is F51's verbatim sentence,
   architecture.md:4367 — the epic's "by the data, not by engine quality" is a PARAPHRASE;
   quote the architecture in the header, not the epic; likewise F51 says "nearly half", not
   the epic's "fewer than half" — follow F51 for the recorded figure) — F42's favourable
   majority over-estimates by exactly this margin. The record lives in the HEADER, **under the
   corpus lock** (the 4.13 precedent: D36's record), NOT in `traps/README.md` — the register takes cases
   reality produced and untrappable limits (4.8/4.12); F51 is a population bound on how the
   metrics READ, already load-bearing in the architecture. The header also states the
   deliberate scope split: the collision family tests the same-hostname-two-boxes temptation;
   the absent/empty-hostname population F51 counts is 4.17's family.

4. **The stream is committed as 3 observations.**
   `fixtures/scenario/replay/hostname-collision.jsonl`, fresh `obs_id` prefix `afafafaf`
   (first id in full: `afafafaf-0000-4000-8000-000000000001`, then `…0002`/`…0003`), the
   corpus UUIDs (`connector_id` `33333333-3333-4333-8333-333333333333`, `l2_domain`
   `11111111-1111-4111-8111-111111111111`, `vantage` `22222222-2222-4222-8222-222222222222`),
   strictly increasing `observed_at` (`2026-01-09T00:00:00Z`, `T00:05:00Z`, `T01:00:00Z`),
   `raw: null`, 3 facts per line (Mac, IpV4, Hostname — no Uplink: the collision needs no
   topology; hostname alone is the temptation), every `Hostname` with `"source":"Dhcp"` (the
   corpus's uniform choice):
   - **H1** (`…0001`, 00:00): `Mac [2,0,94,0,83,160]` flag `true` · `IpV4 192.0.2.50` ·
     `Hostname doc-printer`.
   - **H2** (`…0002`, 00:05): `Mac [2,0,94,0,83,161]` flag `true` · `IpV4 192.0.2.51` ·
     `Hostname doc-printer` — the collision: same name, different box.
   - **H3** (`…0003`, 01:00): `Mac` byte-identical to H1's · `IpV4 192.0.2.50` byte-identical ·
     `Hostname doc-printer` — the plain re-sighting.
   A byte-pin test in `fixtures.rs` (the second-oracle idiom, with the 4.13/4.14 review
   lessons pre-applied — pin the VALUES the reasons depend on): 3 observations;
   `facts.len() == 3` on every line; the same `Hostname` fact (name AND source) on all three,
   value-pinned to `doc-printer`/`Dhcp`; H1's `Mac` and `IpV4` value-pinned
   (`[2,0,94,0,83,160]`, `192.0.2.50`); H2's `Mac` and `IpV4` value-pinned
   (`[2,0,94,0,83,161]`, `192.0.2.51`) and both `assert_ne` against H1's; H3's `Mac` and
   `IpV4` asserted EQUAL to H1's; flags true on all three (the flag-vs-bytes walk guard from
   4.14 covers them too); the instants VECTOR asserted EQUAL to the three authored values via
   `ts()` (the dhcp-churn shape — vector equality pins the values AND the strict increase; a
   bare windows-increase check would leave the one-hour gap AC2 depends on unpinned).

5. **The two traps are committed with both poles present.**
   `fixtures/scenario/traps/hostname-collision.toml`, `family = "hostname-collision"` on both
   (ids `hostname-collision-must-not-merge`, `hostname-collision-must-merge` — the two-trap
   sibling idiom), so `incomplete_families` stays empty. Multi-line `observations` arrays,
   full UUIDs: the must-not-merge judges `[…0001, …0002]`, the must-merge judges
   `[…0001, …0003]`. No `must-abstain` — the collision is not ambiguous (the MACs decide);
   the hostname-ABSENT abstention cases belong to 4.17. Every id whole on its line.

6. **Every expectation carries its mandatory one-sentence `reason`** (20–300 chars, single
   line), values checkable against the bytes, no raw MAC octets (the 4.14 rule, review-held):
   the must-not-merge reason names the shared default `doc-printer`, the distinct MACs/addresses
   that oppose (descriptively — "distinct MACs", never the octets), and the F51 weakness; the
   must-merge reason names the identical MAC, identical address and same hostname with nothing
   opposing. **Phrasing template: `vrrp-virtual-mac.toml`'s committed reasons** (descriptive,
   octet-free); dhcp-churn's must-merge reason names a raw MAC and is NOT the template.

7. **The corpus lock and count coupling are bumped deliberately, red first.** Both artefacts
   into `fixtures/MANIFEST.toml` (17 → 19; gate message `"19 fixture(s) match their recorded
   sha256 (0 generated, 19 hand-authored)"`), sha256 AFTER the final byte (trailing-newline
   check, wrap-check first). The three committed-count assertions in `trap_gate.rs` move
   **17 → 19** — `the_committed_corpus_is_discovered_and_scored_by_nothing` (:392, the
   breakdown comment's tail "…(story 4.14) three — seventeen in the committed corpus" is
   REPLACED by "…three, `hostname-collision.toml` (story 4.15) two — nineteen in the committed
   corpus"; a pure append would strand "seventeen"),
   `the_report_says_plainly_that_nothing_was_scored` (:410, `"19 trap(s) discovered"`),
   `a_trap_with_no_answer_is_discovered_but_not_scored` (:428 + comment :420 "stays 19") —
   red observed at `left: 19, right: 17` BEFORE the update (line numbers as-of-story-creation;
   test NAMES are the anchor). Reproducibility test and scratch tests untouched.

8. **Synthetic-only, all values fresh and verified**: `afafafaf`, `doc-printer`, MAC last-bytes
   `83,160`/`83,161`, IPs `.50`/`.51` — all grep-free at story creation; re-verify at dev time.
   `grep -rn afafafaf fixtures/ crates/` hits only the two new files at commit time. Privacy +
   validity walks pass unchanged (locally-administered MACs, RFC 5737, `doc-` prefix).

## Tasks / Subtasks

> **⚠️ ATDD ORDER (the 4.13 shape — no harness change this time):** byte-pin test red →
> stream lands, test greens → trap file lands → count red at 17 → 19 → manifest bump → gates.
> Mid-story, between the files landing and Task 5, the fixtures gate reds on two orphans —
> expected, resolved by the manifest bump.

- [x] **Task 1 — byte-pin test, observe RED** (AC: 4): `fixtures.rs` trailing test module,
      appended at the END, e.g. `the_hostname_collision_stream_shares_one_name_across_two_boxes`;
      red = `FixtureError::Io` inside `read_jsonl` on the missing file. Record.
- [x] **Task 2 — the stream** (AC: 4, 8): three lines per AC4 (envelope template: a
      `dhcp-churn.jsonl` line — Mac+IpV4+Hostname, no Uplink). Re-verify frees first. Trailing
      newline. Byte-pin greens.
- [x] **Task 3 — the trap file** (AC: 1, 2, 3, 5, 6): header per AC3 (F51 verbatim with
      citation, the scope split vs 4.17, the cloned-mac kinship note, the no-abstain sentence);
      two `[[trap]]` blocks per AC5; reasons per AC6, measured (`wc -m` or awk), recorded.
      Corpus walks pass unchanged.
- [x] **Task 4 — count coupling red, 17 → 19** (AC: 7): red run recorded, three literals +
      two comments updated; nothing else in trap_gate.rs.
- [x] **Task 5 — manifest bump** (AC: 7): two `[[artefact]]` entries with story-naming
      comments; hash after final byte; existing seventeen entries untouched.
- [x] **Task 6 — gates** (AC: 7, 8): fmt · clippy `--all-targets -D warnings` · test
      `--workspace` · `xtask ci` (quote the real fixtures message, expect 19/19). Residual
      grep `"17 trap"` / `discovered(), 17` / `stays 17` / `seventeen` → no hits.
      `Cargo.lock` unchanged; `architecture-views.md` NOT regenerated; `traps/README.md`
      untouched (AC3's argued choice).

### Review Findings

- [x] [Review][Patch] AC2's prescribed header sentence (the cloned-mac must-merge byte-kinship —
      "the corpus tests temptations, not byte-shapes") was missing from the committed header;
      added, file re-hashed (`557c7e80…940c`), MANIFEST updated (Auditor #1)
- [x] [Review][Patch] The byte-pin read by index while the traps judge by obs_id — a deliberate
      obs_id swap with a re-hashed manifest would invert both traps silently; the three obs_ids
      are now pinned (Edge Case Hunter #1). The same hole in 4.13/4.14's byte-pins → registered
      in deferred-work.md under "code review of story-4.15" (pre-existing pattern)
- [x] [Review][Patch] The `facts.len()==3` justification comment (4.13's lesson) restored
      (Edge Case Hunter #2)
- [x] [Review][Patch] Change Log lacked the "Implemented (dev-story)" row; added (Auditor #2).
      Completion note weakened to the true sentence — the header REFERS to the tempting rule
      descriptively, it does not name the literal id (Auditor #3)
- [x] [Review][Note] Blind Hunter's stale-test-binary observation (Synology mount mtime) is
      environmental, recorded for [[local-gate-must-mirror-ci]] — no repo change

## Dev Notes

### The shape of this story in one paragraph

A routine two-trap family in the 4.13 mold — pure data + three count literals + one byte-pin
test, no harness change, nothing coined (`l1-distinct-mac` and `l1-exact-mac` both REUSED, the
tempting `l2-hostname-agrees` never named — an expectation names what FIRES or OPPOSES, never
what tempts). What is genuinely NEW: the family whose header carries F51's population bound —
the first trap family arguing about the STATISTICS of a signal, not just its bytes.

### Rule vocabulary — nothing is coined

- must-not-merge names `l1-distinct-mac` (the opposer; spelled as in `randomized-mac.toml` /
  `dhcp-churn.toml`). The temptation (`l2-hostname-agrees`, coined by 4.11) is never named
  **by any EXPECTATION** (dhcp-churn's header phrasing) — the header PROSE may and does name
  it, to state the kinship below.
- must-merge names `l1-exact-mac` (the firer; spelled as in `example.toml` and four siblings).
- Do NOT name `l2-different-hostname` anywhere — no pair in this family differs in hostname;
  that is the whole point of the family.

### Kinships, and why this family is distinct (say it in the header)

- **vs cloned-mac**: its must-not-merge is "same MAC, different hostnames"; ours is the exact
  MIRROR — "different MACs, same hostname". Together they pin: neither single signal outvotes
  the other's opposition; the false merge is guarded from both directions.
- **vs shared-hardware-vm**: its must-merge fires `l2-hostname-agrees` behind a SHARED uplink —
  and NOTE (validation caught this): that pair's MACs DIFFER too (its own reason says
  "their locally-administered MACs differ but they share the hostname"), so the distinguisher
  between the two families is the TOPOLOGY CORROBORATION, not the absence of differing MACs.
  Our must-not-merge is the case its rule must not be stretched to: hostname agreement with
  differing MACs and NO shared topology (this stream carries no Uplink at all — the absence
  of corroboration is authored, not accidental).
- **vs 4.17 (next)**: F51's population (absent/empty hostname) is 4.17's family; this family
  tests the present-but-colliding case. The header states the split so neither family is
  blamed for the other's scope.

### Previous story intelligence (4.14)

- Review lesson applied FORWARD in AC4: pin every value a reason cites (4.14's review had to
  patch in the uplink and IP pins; this story's byte-pin prescribes all value pins up front).
- The flag-vs-bytes guard (4.14) now walks every stream — the three `true` flags here are
  covered automatically; nothing to add.
- The no-octets-in-reasons rule is review-held (trap text reaches no scanner — 4.14's defer);
  keep reasons octet-free anyway.
- PR workflow: branch → PR → CI green → squash merge (let gh keep its default "(#N)" subject
  suffix). [[opencmdb-pr-workflow]]
- Line numbers drift: 4.14 appended tests to fixtures.rs — the count-assertion lines are NOW
  :392/:410/:428 (verified at story creation); names anchor.

### Project Structure Notes

- **NEW (locked):** `fixtures/scenario/replay/hostname-collision.jsonl` (3 obs),
  `fixtures/scenario/traps/hostname-collision.toml` (2 traps). Manifest 17 → 19.
- **Updated:** `crates/opencmdb-bin/src/fixtures.rs` (one byte-pin test, appended, tests
  only); `crates/opencmdb-bin/src/trap_gate.rs` (three literals 17 → 19 + two comments, tests
  only); `fixtures/MANIFEST.toml`.
- **Unchanged:** all production paths, `Cargo.lock`, the seventeen existing manifest entries,
  `traps/README.md`, `architecture-views.md`, the privacy helpers (4.14's `is_synthetic_mac`
  and guards are used AS-IS).
- **Out of scope:** the absent/empty-hostname family (4.17); any hostname-normalization rule
  (Epic 5); mDNS/NetBIOS source variants (the corpus's `Dhcp` uniformity holds until a story
  needs otherwise).

### Traps (mistakes this story must not make)

1. **Naming `l2-hostname-agrees` in the must-not-merge.** It is the TEMPTATION; the
   expectation names the opposer `l1-distinct-mac` (trap.rs:66-73).
2. **Giving H2 H1's MAC or IP** — collapses the collision into a re-sighting; or **giving H3 a
   different IP** — turns the must-merge into dhcp-churn's moved-lease shape. H3 is
   byte-identical to H1 except `obs_id`/`observed_at`.
3. **A hostname not prefixed `doc-`**, a non-5737 IP, a universally-administered MAC, or a
   lying flag — the walks red (including 4.14's flag guard).
4. **A `must-abstain` pole or an Uplink fact.** The collision is decided by MACs; topology
   and absence belong to other families.
5. **Putting the F51 record in `traps/README.md`.** The header carries it (AC3's argument);
   the register is for reality-produced cases and untrappable limits.
6. **Forgetting both-pole completeness** — two traps, one family string, both columns.
7. **Hashing before the final byte; skipping the count red; touching scratch tests or the
   reproducibility test.**
8. **Overclaiming in the completion record** — name the command behind every count.
   [[claims-must-match-verification]]

### References

- [Source: _bmad-output/planning-artifacts/epics.md:1177-1189 — Story 4.15: the story
  sentence and the two epic ACs (must-not-merge with the why-insufficient reason; the F51
  record bounding the abstention rate)]
- [Source: _bmad-output/planning-artifacts/architecture.md:4367 — F51 verbatim: "hostname is
  unusable on nearly half of known clients (MISSING/empty, never null)… the abstention rate
  is bounded below by this, not by the engine's quality"]
- [Source: _bmad-output/planning-artifacts/architecture.md:4184-4188 — the measurement's
  reservation 1: "you cannot group by a hostname you do not have"]
- [Source: _bmad-output/planning-artifacts/architecture.md:4319 — absent AND empty occur,
  null NEVER ("must NOT encode null") — 4.17's charter, cited for the scope split]
- [Source: _bmad-output/planning-artifacts/architecture.md:884-895 — D12: hostname one of
  L2's three signals (:891)]
- [Source: fixtures/scenario/traps/cloned-mac.toml — the mirror family (same MAC/different
  hostnames) and the must-merge byte-kinship]
- [Source: fixtures/scenario/traps/shared-hardware-vm.toml — `l2-hostname-agrees`'s
  legitimate firing shape (shared uplink, no opposing MAC)]
- [Source: crates/opencmdb-bin/src/trap_gate.rs:392/:410/:428 + comments :387-391/:420 — the
  three count literals 17 → 19]
- [Source: _bmad-output/implementation-artifacts/4-14-trap-vrrp-hsrp-virtual-mac.md — prior
  story: the value-pinning review lesson this story pre-applies, the flag guard, the
  review-held no-octets rule]

## Dev Agent Record

### Agent Model Used

claude-fable-5 (Claude Fable 5)

### Debug Log References

- **Task 1 natural RED**: `cargo test -p opencmdb-bin --locked the_hostname_collision_stream…`
  → panicked at the expect with `FixtureError::Io { path: ".../fixtures/scenario/replay/hostname-collision.jsonl", source: Os { code: 2, kind: NotFound } }`
  — inside `read_jsonl`, as predicted.
- **Byte-pin GREEN** after the stream landed (1 passed).
- **Reason lengths** (awk length on committed bytes): must-not-merge **255**, must-merge
  **246** — inside 20–300, single line each, octet-free.
- **Count-coupling RED**: with both files on disk and assertions at 17,
  `the_committed_corpus_is_discovered_and_scored_by_nothing` → `left: 19 / right: 17` — both
  new traps parsed, validated, resolved. Then 17 → 19 at :392/:410/:428 + the two comments
  (the breakdown tail REPLACED — "…three and hostname-collision.toml (story 4.15) two —
  nineteen"), green.
- **`afafafaf` grep**: only the two new files. Frees re-verified pre-authoring
  (`doc-printer`, `83,160`/`83,161`, `.50`/`.51` — no hits).
- **Hash-after-final-byte held**: trailing newlines confirmed (`tail -c 1 | xxd` → `0a`),
  then `sha256sum`: `fd15e021d1bf3adc19c92baabcd91f26dc5085d51fc9aae9a7982fb40befc243`
  (jsonl), `e7dfb190f3369f11e581035565377cd00080b7eecd4705c4831fa690f6dd9dcf` (toml). No edit
  after hashing.
- **Gates** (all green): fmt · clippy `--all-targets --locked -- -D warnings` ·
  `cargo test --workspace --locked` → **115 (bin) + 86 (core) + 42 (xtask), 0 failed** ·
  `cargo run -p xtask --locked -- ci` → fixtures verbatim **"19 fixture(s) match their
  recorded sha256 (0 generated, 19 hand-authored)"**, no orphan; views-hash `ℹ STALE` by
  design. Residual grep (`"17 trap"` / `discovered(), 17` / `stays 17` / `seventeen`) → no
  hits. `Cargo.lock` untouched (`git diff --stat` empty).

### Completion Notes List

- The family landed exactly as scoped: two NEW locked artefacts (3-obs stream, 2-trap file),
  manifest 17 → 19, three count literals 17 → 19, ONE new byte-pin test. No harness change,
  nothing coined — `l1-distinct-mac` opposes the collision, `l1-exact-mac` fires the
  re-sighting; the tempting rule is REFERRED TO by the header prose ("the rule 4.11 coined for
  it") and its literal id appears nowhere in the file — no expectation names it.
- **AC1**: `hostname-collision-must-not-merge` judges [H1,H2] — distinct MACs/addresses,
  same `doc-printer`; the reason names the factory-default story, the opposers descriptively,
  and F51's weakness.
- **AC2**: `hostname-collision-must-merge` judges [H1,H3] — byte-identical Mac+IpV4, same
  name, one hour later; anti-cowardice against collision-generalization.
- **AC3**: the header carries F51's TRUE verbatim ("bounded below by this, not by the
  engine's quality"), "nearly half", the 4.17 scope split, the null non-case, the cloned-mac
  mirror and the shared-hardware-vm distinction (topology corroboration — its must-merge
  pair's MACs differ too), and the no-abstain sentence. `traps/README.md` untouched (AC3's
  argued choice).
- **AC4**: byte-pin asserts 3 obs · 3 facts each · hostname value-pinned on H1 and
  equality-carried · both MACs and both IPs value-pinned with inequalities · H3 == H1 on
  Mac+IpV4 · instants as an exact VECTOR (values + order in one assertion).
- **AC5/AC6/AC7/AC8**: both poles present (`incomplete_families` empty in the full run);
  reasons 255/246; deliberate bump red-first; all values synthetic and fresh.

### File List

- `fixtures/scenario/replay/hostname-collision.jsonl` — NEW: 3-observation replay stream
- `fixtures/scenario/traps/hostname-collision.toml` — NEW: 2-trap family file (F51 record)
- `fixtures/MANIFEST.toml` — modified: two `[[artefact]]` entries appended (17 → 19)
- `crates/opencmdb-bin/src/fixtures.rs` — modified: byte-pin test appended (tests only)
- `crates/opencmdb-bin/src/trap_gate.rs` — modified: three count literals 17 → 19 + two
  comments (tests only)
- `_bmad-output/implementation-artifacts/4-15-trap-hostname-collision.md` — this story file
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — status tracking

## Change Log

| Date       | Change                                                                 |
|------------|------------------------------------------------------------------------|
| 2026-07-25 | Story 4.15 drafted (create-story, autonomous run): hostname collision — the mirror of cloned-mac (different MACs, same hostname) and the first family carrying a population bound (F51: hostname unusable on ~half, abstention rate bounded by data). Two traps, nothing coined: must-not-merge [H1,H2] on `l1-distinct-mac` (temptation `l2-hostname-agrees` never named), must-merge [H1,H3] on `l1-exact-mac` (plain re-sighting, anti-cowardice). F51 record in the family HEADER, not the register (argued in AC3). 3-obs stream `afafafaf`, `doc-printer`, no Uplink. Counts 17 → 19, manifest 17 → 19. Byte-pin prescribes all value pins up front (4.14's review lesson pre-applied). Status → ready-for-dev. |
| 2026-07-25 | Validated (two fresh-context agents: fact-check + gap-hunt). 0 HIGH / 4 MED / 6 LOW, all applied: F51's TRUE verbatim distinguished from the epic's paraphrase ("bounded below by this, not by the engine's quality"; "nearly half" not "fewer than half"); shared-hardware-vm kinship corrected (its must-merge pair's MACs DIFFER — the family distinguisher is topology corroboration, not MAC absence); instants pinned as a VECTOR equality (dhcp-churn shape), not a bare increase check; "never named" scoped to expectations (header prose may name the temptation); orphan-red ATDD note restored; breakdown-comment tail REPLACED not appended; reason phrasing template pointed at vrrp-virtual-mac (octet-free), dhcp-churn explicitly NOT the template; "four siblings"; header records F51 "under the corpus lock". |
| 2026-07-25 | Implemented (dev-story): all 6 tasks, ATDD order held — byte-pin RED (`FixtureError::Io`), stream landed and greened, trap file landed, count coupling RED at `left: 19, right: 17` then 17 → 19, manifest 17 → 19 (sha256 after final byte). Reasons 255/246 chars, octet-free. Gates green: fmt, clippy `-D warnings`, `cargo test --workspace` (115+86+42), `xtask ci` ("19 fixture(s) match their recorded sha256"). Nothing coined, no harness change, `Cargo.lock`/README untouched. Status → review. |
| 2026-07-25 | Code review (3 fresh-context layers: Blind Hunter / Edge Case Hunter / Acceptance Auditor). **Auditor: PASS 7/8** (AC2 partial — the header lacked its prescribed cloned-mac kinship sentence). 0 CRITICAL/HIGH; **4 patches applied**: the kinship sentence added to the header (file re-hashed `557c7e80…940c`, MANIFEST updated — the one hashed-artefact patch), the byte-pin now pins the obs_id ↔ line binding (Edge's MED: an obs_id swap would have inverted both traps silently) plus the restored `facts.len()` justification comment, the missing Change Log row, and the completion-note overclaim weakened ("referred to, never named"). **1 defer registered** (the 4.13/4.14 byte-pins share the obs_id-binding hole — pre-existing pattern). Blind Hunter: 0 defects in the change (2 environmental notes, incl. the Synology-mount stale-binary trap for [[local-gate-must-mirror-ci]]). Gates re-run green post-patch (115+86+42; "19 fixture(s) match their recorded sha256"). Status → done. |
