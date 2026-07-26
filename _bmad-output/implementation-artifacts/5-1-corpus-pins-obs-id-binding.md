# Story 5.1: The corpus pins the obs_id-to-line binding, and every stream goes through the connector

Status: ready-for-dev

<!-- Validation is MANDATORY here (Guy's decision, Epic 4 retrospective 2026-07-26): two
     fresh-context agents (fact-check + gap-hunt) before dev-story. The template banner saying
     validation is optional does not apply to this project. -->

> **Validation pass held 2026-07-26** — two fresh-context agents (fact-check + gap-hunt). Three
> HIGH and six MEDIUM findings, all applied above. The three HIGH: the file-size gate claim was
> backwards (the hoist RAISES the count, and the count is 698 not 699); a FOURTH byte-fidelity
> register entry (story-4.13's) was about to be silently orphaned; and *"the one place
> `UndeclaredFactKind` bites"* was false twice over — the dev had been told to put it in a doc
> comment. Both agents independently re-measured the corpus byte-shape and the `Serialize` derive;
> both came back clean, so AC3 carries a measurement rather than a hope.

## Story

As the owner of the corpus,
I want each byte-pin test to assert the `obs_id` of the line it reads, and every committed replay
stream to be loaded through `FixtureConnector::load` and round-tripped through serde,
so that a re-authored stream cannot invert what its traps judge — or drift out of the canonical
byte-shape — while every assertion stays green.

**This is inherited debt, placed at the HEAD of Epic 5 on Guy's decision** (epics.md:1312): the
corpus byte-fidelity theme had accumulated three unowned entries in `deferred-work.md`. It comes
first because **Epic 5 bumps the corpus**, and hardening after the bump means replaying every entry
against artefacts that have moved. **No corpus bytes change in this story** — it is test-side work
only, plus one `derive` (see AC3). If a task appears to require re-authoring a committed artefact or
re-hashing `MANIFEST.toml`, STOP: that is a finding, not a task (see Dev Notes → "The corpus does
not move here").

## Acceptance Criteria

1. **AC1 — the `obs_id` ↔ line binding is pinned on the two byte-pins that read purely by index.**
   **Given** `the_dhcp_churn_stream_moves_the_address_only_through_observed_at`
   (`crates/opencmdb-bin/src/fixtures.rs:1789`) and
   `the_vrrp_stream_shares_one_virtual_mac_and_moves_its_uplink` (`fixtures.rs:1866`), which read
   observations by INDEX while `dhcp-churn.toml` / `vrrp-virtual-mac.toml` judge by `obs_id`
   **when** the pins are strengthened **then** each pinned observation asserts its `obs_id`:
   - `dhcp-churn.jsonl` — 3 lines, `adadadad-0000-4000-8000-000000000001` … `…0003`;
   - `vrrp-virtual-mac.jsonl` — 4 lines, `aeaeaeae-0000-4000-8000-000000000001` … `…0004`.

   **And** the assertion is expressed ONCE. Six call sites of the same loop is accidental
   duplication, which the DRY rule forbids: introduce a single test helper in `fixtures.rs`'s test
   module —

   ```rust
   /// Pin the `obs_id` ↔ LINE binding of a committed stream: line `n` (0-indexed) carries
   /// `{prefix}-0000-4000-8000-{n+1:012}`. The traps judge by `obs_id` while every byte-pin
   /// reads by index; without this, a deliberate re-authoring that swaps two lines' ids (with a
   /// re-hashed manifest) would invert what a family's traps judge while every byte-level
   /// assertion stayed green (registered under story 4.15's review).
   fn assert_obs_ids(observations: &[Observation], prefix: &str) { … }
   ```

   — and fold the four existing copies into it: `fixtures.rs:2093-2103` (hostname-collision, 4.15,
   prefix `afafafaf`), `:2245-2252` (docker-veth, 4.16, `babababa`), `:2397-2404`
   (hostname-absence, 4.17, `bcbcbcbc`), `:2700-2711` (the wire spec, 4.18, `bdbdbdbd` — its loop
   is FUSED with its placeholder-context pins; the `obs_id` half is `:2707-2711`: call the helper
   and drop only that assertion from the loop). If you leave a call site inline, the story record
   must say which and why.

   **And the DRY verdict is argued, not asserted.** The house rule protects deliberate redundancy
   — but all four existing loops already COMPUTE their ids with `format!("…{suffix:012}")` rather
   than restating them, so what is being removed is mechanical duplication, not a second oracle
   (unlike `expected()`, which restates VALUES and must survive). The helper therefore encodes two
   corpus conventions in one place: the fixed `-0000-4000-8000-` middle segment, and sequential
   numbering from 1 — both true of all 13 committed streams as of this story. Say that in its doc,
   and say that a future stream numbered otherwise gets its own assertion rather than being
   re-authored to satisfy a helper.

2. **AC2 — every committed replay stream is admissible to the connector, not merely parseable.**
   **Given** that the 4.4 admissibility layer (foreign `connector_id`, uncovered scope, undeclared
   fact kind, repeated `obs_id`, scripted cancellation, capability ordering) is exercised against
   `minimal.jsonl`, `partial-then-failed.jsonl` and `capability-downgrade.jsonl` only — every
   FAMILY stream since 4.9 is gated for parseability by the fixtures walks and by nobody else
   (registered under story 4.12's review) **when** the corpus walk runs **then** every `.jsonl`
   under `fixtures/scenario/replay/` is loaded through `FixtureConnector::load` and must return
   `Ok`, so corpus-level parseability and connector-level admissibility stop being two different
   claims.

   **And** the walk is driven by a HAND-AUTHORED per-stream table of `(relative path,
   ConnectorId, scopes_covered, initial Capabilities)` — a second, independent statement of each
   stream's context, in the `expected()` idiom — **never derived from the observations**. Derivation
   would make `ForeignConnectorId` and `UncoveredScope` vacuous by construction and would be the
   move `fixture_connector.rs`'s module doc refuses ("a capability read off what was seen cannot
   express *capable of hostnames, saw none*").

   **And** the table is checked in BOTH directions, mirroring the corpus lock's own rule: a stream
   the walk finds with no table entry is RED (the orphan direction), and a table entry naming a
   file that does not exist is RED. A new committed stream must therefore state its context, or
   the suite reds.

   **And** the test's doc comment states plainly what the walk does NOT prove. The walk shows every
   committed stream is ADMISSIBLE; it never observes `UndeclaredFactKind` FIRING. Where the check is
   non-vacuous: `partial-then-failed.jsonl` (`partial_caps()` declares exactly the four kinds the
   stream emits) and `capability-downgrade.jsonl` on BOTH sides of its capability record
   (`downgrade_initial_caps()` declares four; the record narrows to three). Where it is vacuous: the
   eleven `corpus_*` streams, because `corpus_caps()` declares all seven `FactKind`s — deliberately
   wider than what is emitted, which is the whole reason the descriptor exists. Write that; do not
   write "the walk proves fact-kind coverage corpus-wide", and do not write "the one place it
   bites".

3. **AC3 — the round-trip byte-shape witness covers every committed stream, not `minimal.jsonl`
   alone.** **Given** `re_serializing_reproduces_the_committed_bytes` (`fixtures.rs:794`), which
   round-trips `minimal.jsonl` only, so no other stream — and no CONTROL record at all — has its
   exact serialized byte-shape (field order, `MacAddr` array encoding, `Uplink` field names, the
   internally-tagged `record` marker) pinned by a parse→re-serialize→compare test (registered under
   story 4.10's review) **when** the corpus walk runs **then** every `.jsonl` under
   `fixtures/scenario/replay/` is read with `read_records`, re-serialized record by record, and
   compared to the committed bytes LINE BY LINE, the failure naming the file and its 1-indexed line
   number.

   **And** `re_serializing_reproduces_the_committed_bytes` SURVIVES unchanged, with a comment
   saying why the two are not duplicates: it starts from `expected()` — an independently authored
   Rust literal — so it pins the VALUES as well as the shape; the corpus-wide witness starts from
   the file, so it pins the shape only. This is deliberate redundancy of the kind the DRY rule
   protects, and a future DRY pass must not collapse it.

   **And** the control-record shape is covered, which today it is not: `ControlRecord`
   (`fixtures.rs:122-135`) is `Deserialize`-only, so it gains `Serialize` and the round-trip renders
   a `Record::Failure` / `Record::Capability` back to its committed line. **Measured on the pinned
   serde/chrono:** the derive compiles under `tag` + `rename_all` + `deny_unknown_fields` + a
   `#[serde(flatten)] Capabilities`, emits the tag FIRST, and reproduces both committed control
   lines byte-exactly. The `#[cfg(test)]` mirror-struct fallback should therefore NOT be needed; if
   you end up taking it, say why.

   **And** note the mapping the story would otherwise leave implicit: `Record` (`fixtures.rs:88-104`)
   holds `ConnectorError` / `Capabilities` directly, **not** `ControlRecord`. Rendering means
   `Record::Failure(e)` → `ControlRecord::Failure { error: e.clone() }` and `Record::Capability(c)`
   → `ControlRecord::Capability { capabilities: c.clone() }`. Both inner types are `Clone`.

4. **AC4 — every strengthened guard is proven to red before it passes, and the mutation is
   recorded** (house rule, story 1.3). One recorded mutation per AC, minimum:
   - **AC1:** swap two `obs_id` values between two lines of `dhcp-churn.jsonl`, run
     `cargo test -p opencmdb-bin the_dhcp_churn_stream`, observe the named red, `git checkout` the
     file. Record the exact message. (`cargo test` does not consult `MANIFEST.toml`, so no re-hash
     is needed for the observation — and none must be committed.)
   - **AC2:** point one table entry's `ConnectorId` at a different UUID → expect `ForeignConnectorId`
     naming the stream and the offending `obs_id`; then DELETE one table entry → expect the orphan
     direction to red naming the unlisted stream. Both mutations are test-side; the corpus is not
     touched.
   - **AC3:** insert one space after a colon on line 1 of **`dhcp-churn.jsonl`** → expect the
     round-trip red naming that file and line 1; `git checkout` the file. **Not `minimal.jsonl`:**
     an edit there also reds `the_committed_fixture_reads_back_exactly` (`fixtures.rs:784`) and
     `re_serializing_reproduces_the_committed_bytes` (`:794`), so the new guard would be observed
     for the wrong reason — the same care AC1's mutation takes. Then do the same on the CONTROL
     line of `capability-downgrade.jsonl` (line 3), so the control-record half is proven too — a
     round-trip that silently skipped control records would pass the first mutation.

5. **AC5 — the local gate is green and the corpus is byte-identical.** `cargo fmt --all`,
   `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and
   `cargo xtask ci` all pass, and `git status` shows no change under `fixtures/`. The
   `views-hash STALE` line from `xtask ci` is expected and exits 0 — **do not regenerate
   `architecture-views.md` in this story** (it is a milestone task, project-context.md:127-130).

6. **AC6 — the register is updated by APPENDING, never by rewriting.** The three
   `deferred-work.md` entries this story closes are marked closed in place, in the file's
   established idiom (`✅ **CLOSED by story 5.1.** ~~old text struck~~ …`), keeping the original
   text readable:
   - "Deferred from: code review of story-4.10" — the round-trip byte-shape entry (AC3);
   - "Deferred from: code review of story-4.12" — the `FixtureConnector::load` entry (AC2);
   - "Deferred from: code review of story-4.15" — the `obs_id` ↔ line binding entry (AC1).

   **And** the FOURTH byte-fidelity entry is annotated in place as **still open**, not silently
   left behind: `deferred-work.md:365-378` (story-4.13's review — *"the dhcp-churn byte-pin test
   pins MAC/hostname values relationally, never by value"*), whose owner line names this very
   theme. **This story does not close it:** AC3's round-trip starts from the FILE, so it pins the
   byte-SHAPE and never the authored values; AC1 pins `obs_id`s only. Append a one-line
   `↺ **STILL OPEN after story 5.1** — …` note saying exactly that; do NOT strike it. *A register
   that loses an item is worse than no register* (the file's own line 16).

   **And** a new `## Deferred from: story-5.1` section records the finding this story surfaces but
   does NOT close (see Dev Notes → "Found while scoping, deliberately out of scope"): four
   committed family streams — `randomized-mac.jsonl`, `multi-nic.jsonl`,
   `shared-hardware-vm.jsonl`, `cloned-mac.jsonl` — have **no byte-pin test at all**, so their
   `obs_id` ↔ line binding and their authored values are asserted by nothing narrower than the
   corpus walks and the sha256 lock. AC1 cannot cover them: there is no test to strengthen.

   **And** the entry names its OWNER, because the work is already scheduled: **story 5.2b**,
   inserted on 2026-07-26 in Epic 5's debt block, immediately after 5.2 and ahead of the L1 join
   (epics.md, "Story 5.2b"). The entry says so — a register item with a named owner and a slot is
   not a deferral, and writing it as one would misstate the plan.

   **And** this stays in `deferred-work.md` and does NOT become a GitHub issue. The register is the
   established home for review-surfaced corpus debt (every entry since 4.1); an issue is reserved
   for scope that MOVES between epics — the 4.19b precedent (#34). Nothing moves here: the work
   stays in Epic 5, three stories later.

## Tasks / Subtasks

- [ ] **Task 1 — the `obs_id` pin helper (AC1)**
  - [ ] Add `assert_obs_ids(observations: &[Observation], prefix: &str)` to `fixtures.rs`'s test
        module, with the doc comment stating WHY (traps judge by `obs_id`, byte-pins read by index).
  - [ ] Call it from `the_dhcp_churn_stream_moves_the_address_only_through_observed_at` (`adadadad`)
        and `the_vrrp_stream_shares_one_virtual_mac_and_moves_its_uplink` (`aeaeaeae`).
  - [ ] Fold the four existing copies (`:2093-2103`, `:2245-2252`, `:2397-2404`, `:2700-2711`) into
        the helper, and record the DRY argument from AC1 in its doc.
  - [ ] Prove-to-red per AC4, record the message.

- [ ] **Task 2 — share the replay walk across the two test modules (AC2, AC3)**
  - [ ] Hoist `walk_replay_streams` (`fixtures.rs:1389`) out of `mod tests` to module scope as
        `#[cfg(test)] pub(crate) fn`, placed immediately BEFORE `#[cfg(test)] mod tests`. Its doc
        comment is kept and extended with the new callers.
  - [ ] Confirm the `file-size` gate stays green — and do NOT claim a reduction. The gate counts the
        lines before the FIRST line whose `trim_start()` begins with `#[cfg(test)]`
        (`xtask/src/main.rs:72-81`), which today is line 699, giving **698** code lines (ceiling
        2000). The hoisted block's doc comment sits ABOVE its `#[cfg(test)]` attribute, so the first
        marker moves later and the count **RISES** by roughly the doc comment's length — expected,
        harmless, and the completion record must say "rose to N, far under 2000", never "lowered".
  - [ ] Do not touch `trap_gate.rs`'s `discover_trap_files` — it is a deliberately separate walk
        over a different tree and says so (`trap_gate.rs:291-301`). Its sentence *"promoting either
        would move its callers for no gain here"* stays TRUE after the hoist: `walk_replay_streams`
        remains `#[cfg(test)]` and `trap_gate` gains no caller. Leave it verbatim; do not "fix" it.

- [ ] **Task 3 — the connector admissibility walk (AC2)**
  - [ ] In `fixture_connector.rs`'s test module, build the per-stream table from the THREE contexts
        that already exist there — `corpus_id/corpus_scope/corpus_caps` (`:358-386`),
        `partial_id/partial_scope/partial_caps` (`:454-475`),
        `downgrade_id/downgrade_scope/downgrade_initial_caps` (`:683-706`). Reuse them; do not
        author a fourth copy of the same UUIDs.
  - [ ] 13 entries, exactly (the measured mapping is in Dev Notes → "The corpus as measured").
  - [ ] **Convert the walked path correctly.** `walk_replay_streams` yields ABSOLUTE paths (rooted
        at `<manifest>/../../fixtures/…`, `..` components included) while `load` takes the
        corpus-RELATIVE string — and `fixture_path` (`fixtures.rs:60-77`) REFUSES an absolute path
        or one carrying `..`. Derive it with `path.strip_prefix(fixtures_dir())`, never by writing
        the `fixtures/` prefix again: `the_fixtures_path_is_expressed_once` (`fixtures.rs:1734`)
        counts occurrences of `/../../fixtures` and reds with a path-discipline message that would
        say nothing about what you actually did. Key the table by that same relative string, so
        `ForeignConnectorId`'s `origin` names the stream the way `MANIFEST.toml` spells it.
  - [ ] Walk `scenario/replay/`, `load` each stream with its table entry, `expect` `Ok`.
  - [ ] Assert both directions: no stream without an entry, no entry without a file.
  - [ ] Doc comment states the honest limit (AC2's last clause) AND that the table's `as_of` values
        are not validated for the INITIAL descriptor — `from_records` compares only capability
        RECORDS against preceding observations (`fixture_connector.rs:187-206`), so re-using
        `corpus_caps()`'s `2026-01-01T00:00:10Z` across streams dated as late as 2026-01-11 is
        admissible and means nothing.
  - [ ] **Update two stale doc comments.** `corpus_id()` (`:350-357`) opens *"the … set the committed
        `minimal.jsonl` needs"* and `corpus_caps()` (`:371-386`) says *"deliberately WIDER than what
        `minimal.jsonl` emits"*. After this story they are the declared context of ELEVEN streams,
        and `corpus_caps()`'s wideness is exactly why the fact-kind check is vacuous for those
        eleven. Say so. "Preserve the helpers" means their VALUES and call sites, not stale prose —
        a doc narrower than the code is the defect class this project's reviews keep catching.
  - [ ] Prove-to-red per AC4 (both mutations).

- [ ] **Task 4 — the corpus-wide round-trip witness (AC3)**
  - [ ] Add `Serialize` to `ControlRecord`'s derive (`fixtures.rs:122`); measured to compile and to
        reproduce both committed control lines byte-exactly, so the mirror-struct fallback should
        not be needed — if you take it, say why.
  - [ ] Add the render path (`Record` → committed line, via the `ControlRecord` mapping in AC3) and
        the walking test in `fixtures.rs`'s test module — this is a fixtures-layer claim, so it
        lives beside `re_serializing_reproduces_the_committed_bytes`.
  - [ ] **Get the line number right.** `read_records` (`fixtures.rs:483`) returns `Vec<Record>` with
        line numbers DISCARDED, and it skips truly empty lines while still counting them
        (`:493-500`: *"a blank line still occupies its number, so the message points at what an
        editor shows"*). Zip its output against
        `text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty())` so the reported number is
        the raw 1-indexed file line, matching what `read_records`'s own errors report — and assert
        the two iterators exhaust together (a length mismatch is itself a finding). No committed
        stream carries a blank line today, so a positional zip would be invisibly wrong.
  - [ ] Compare line by line; the panic message names file + 1-indexed line + both strings.
  - [ ] Leave `re_serializing_reproduces_the_committed_bytes` in place and add the
        "not-a-duplicate" comment (AC3).
  - [ ] Prove-to-red per AC4 (observation line AND control line).

- [ ] **Task 5 — the register (AC6)**
  - [ ] Mark the three entries closed in place, appending, never rewriting.
  - [ ] Add the `## Deferred from: story-5.1` section with the four-unpinned-families finding.

- [ ] **Task 6 — the gate and the branch (AC5)**
  - [ ] Full local gate: `cargo fmt --all` · `cargo clippy --workspace --all-targets -- -D warnings`
        · `cargo test --workspace` · `cargo xtask ci`.
  - [ ] `git status` clean under `fixtures/`; `MANIFEST.toml` unchanged.
  - [ ] Update `_bmad-output/implementation-artifacts/sprint-status.yaml`:
        `5-1-corpus-pins-obs-id-binding: done` plus `last_updated` — with a comment saying what was
        delivered AND what moved, never a bare `done` (4.18's rule). Name the fourth byte-fidelity
        entry left open.
  - [ ] Branch → PR → green CI → squash merge. **Never push straight to `master`** — the discipline
        has held since PR #15 (22 PRs; `git log` shows one post-#15 commit, story 4.13 `85860fd`,
        with no PR marker, unverified either way).

## Dev Notes

### The corpus does not move here

This story adds tests and one `derive`. It re-authors nothing. `fixtures/` is a SPEC locked in both
directions by `MANIFEST.toml` (edited AND orphan), and a bump reads in review as *"I am changing the
spec"*. **The measurement says no bump is needed:** every one of the 13 committed streams is already
compact JSON (`,`/`:` separators, no spaces) with top-level keys in `Observation`'s declaration
order (`obs_id, connector_id, observed_at, scope, facts, raw`), nested orders matching
(`scope: {l2_domain, vantage}`, `Mac: {addr, locally_administered}`, `Uplink: {peer_mac, peer_port}`,
`Hostname: {name, source}`), and the one capability line's `kinds` in `BTreeSet`/declaration order
(`["Mac","IpV4","Hostname"]`). *How this was checked:* each line was re-emitted with a compact JSON
dump preserving the file's own key order and compared to the raw line, and the key orders were
enumerated across all 13 files — a whitespace/order check outside Rust, **not** a run of the actual
serde serializer. A second, independent pass re-canonicalized all 13 streams (51 observation lines +
2 control lines) plus the wire file against the real Rust declaration orders and confirmed **zero
mismatches**; it also established that no committed stream carries a blank line, every file is
newline-terminated, and every `observed_at`/`as_of` is second-precision `Z`-suffixed (so chrono's
`AutoSi` round-trips exactly). The Rust round-trip remains the real check and it is Task 4's job to
observe it. If it reds on a committed line, that is a genuine finding: report it and stop rather
than re-authoring the corpus to fit the test.

### The corpus as measured (drives Task 3's table)

Every stream carries exactly ONE `connector_id` and ONE `Scope`. Three contexts cover all 13:

| Context (existing helpers) | `connector_id` | `l2_domain` / `vantage` | Streams |
|---|---|---|---|
| `corpus_*` | `3333…` | `1111…` / `2222…` | `minimal`, `example-traps`, `randomized-mac`, `multi-nic`, `shared-hardware-vm`, `cloned-mac`, `dhcp-churn`, `vrrp-virtual-mac`, `hostname-collision`, `docker-veth`, `hostname-absence` (11) |
| `partial_*` | `4444…` | `5555…` / `6666…` | `partial-then-failed` |
| `downgrade_*` | `7777…` | `8888…` / `9999…` | `capability-downgrade` |

`corpus_caps()` declares all seven `FactKind`s, so it admits every family stream's kinds
(`Mac`, `IpV4`, `Hostname`, `Uplink`, `OuiVendor`, `Rtt` are what the corpus actually emits;
`DhcpLease` is declared and never emitted — deliberately, per its doc). `partial_caps()` declares
`Mac/IpV4/Rtt/Hostname`, which is exactly what `partial-then-failed.jsonl` emits.
`downgrade_initial_caps()` declares `Mac/IpV4/Rtt/Hostname` and the file's own capability record
narrows to `Mac/IpV4/Hostname` mid-stream; the two observations after it emit `{IpV4, Hostname}` and
`{Mac}` — inside the narrowed set.

**Where the fact-kind check is non-vacuous:** `partial-then-failed.jsonl` (tight descriptor — a
fifth kind added to that stream would red) and `capability-downgrade.jsonl` on both sides of its
record. **Where it is vacuous:** the eleven `corpus_*` streams, since `corpus_caps()` declares all
seven kinds. And "non-vacuous" is not "fires": the walk asserts every stream is admissible, so
`UndeclaredFactKind` is never observed. Keep those three sentences distinct — collapsing them is
how a false doc comment gets written.

Control records exist in exactly two streams: a terminal `failure` (last line of
`partial-then-failed.jsonl`) and one `capability` (line 3 of `capability-downgrade.jsonl`). Task 4's
round-trip must render both, or the control half is untested — hence AC4's second AC3 mutation.

### Scope boundary: `scenario/replay/` only

`fixtures/scenario/wire/unifi-clients.expected.jsonl` is a `.jsonl` of `Observation`s but sits
OUTSIDE `scenario/replay/`, is `CONSUMER PENDING` until Epic 11's parser (issue #34), and already
has its own test which is also its privacy coverage
(`the_wire_spec_encodes_the_measured_field_behaviours`, `fixtures.rs:2584`, and see its doc: *"this
directory sits outside every corpus walk"*). **It is out of scope for AC2 and AC3.** Do not widen
the walk to reach it; do not move it.

### Found while scoping — out of scope HERE, and owned by story 5.2b

`randomized-mac.jsonl` (4.9), `multi-nic.jsonl` (4.10), `shared-hardware-vm.jsonl` (4.11) and
`cloned-mac.jsonl` (4.12) are named by **no test in the tree** — verified by
`grep -rn "<name>.jsonl" --include=*.rs crates xtask`, which returns nothing for all four. They have
no byte-pin test, so AC1 has nothing to strengthen there, and their `obs_id` ↔ line binding is
weaker than dhcp-churn's was: `read_traps`'s cross-check only asserts that a trap's `obs_id`s EXIST
in the stream, so swapping two ids inside one of those four files inverts what its traps judge and
nothing reds. AC2 and AC3 give them admissibility and byte-shape coverage — **not** value pins.
Register it (AC6); do not fix it here.

**It is already scheduled: story 5.2b**, inserted on 2026-07-26 in Epic 5's debt block (after 5.2,
ahead of the L1 join at 5.5). Two consequences for THIS story: the `assert_obs_ids` helper AC1
introduces is what 5.2b builds on, so give it a shape that four more families can call without
change; and do not pre-empt 5.2b by adding value pins here — its ACs are written and its
prove-to-red budget is its own.

### What this touches, and what it must not break

- **`crates/opencmdb-bin/src/fixtures.rs`** (UPDATE) — 2795 lines, of which ~698 are CODE (the
  `#[cfg(test)] mod tests` opens at line 699).
  *Today:* the corpus reader (`read_records:483`, `read_jsonl:640`, `read_traps:658`), the error
  taxonomy, and the corpus's inline test module holding `expected():724`, the privacy walk
  (`assert_text_is_synthetic:848`), the corpus validity walks, `walk_replay_streams:1389`, and every
  byte-pin. *This story changes:* one derive
  on the private `ControlRecord:122`; the hoist of `walk_replay_streams` to `#[cfg(test)]
  pub(crate)`; a new `assert_obs_ids` helper; a new round-trip walk; two byte-pins gain their pins;
  four gain a call to the helper. *Must be preserved:* `expected()` as the second independent oracle;
  `re_serializing_reproduces_the_committed_bytes` as the only value-pinning round-trip; the privacy
  walk's exhaustive `match` with no `_` arm (a new `Record` or `Fact` variant must still break it);
  `walk_replay_streams`'s symlink refusal, `README.md` exemption at any depth, and non-swallowed
  read errors (*"walks that quietly see less"* was the recurring defect of 4.1/4.3).
- **`crates/opencmdb-bin/src/fixture_connector.rs`** (UPDATE) — 1475 lines, of which ~337 are CODE
  (its test module opens at line 338).
  *Today:* `load:106` (reads the file, then `from_records`) and `from_records:153` (every
  admissibility invariant, positional capability containment). *This story changes:* the test module
  only — a per-stream table and one walking test. *Must be preserved:* the three existing context
  helper groups and every test that uses them; `load` and `from_records` are NOT modified.
- **`_bmad-output/implementation-artifacts/deferred-work.md`** (UPDATE) — append-only discipline.
- **`_bmad-output/implementation-artifacts/sprint-status.yaml`** (UPDATE) — the live source of truth
  (CLAUDE.md). It is `ready-for-dev` today and must reach `done` in this story's File List, with a
  comment saying what moved.
- **Nothing under `fixtures/`.** No bytes, no `MANIFEST.toml`.

### House rules that bind this story

- **Prove-to-red is not optional** (story 1.3): a guard is observed failing before it passes, and
  the mutation is recorded. AC4 names one per AC. *"Where possible" excuses the genuinely untestable
  — it does not excuse a new guard shipping without a test that reds when it is removed.*
- **Name the test behind every claim.** Four consecutive completion records over-claimed and every
  review caught it. In this story the temptation is exact and named: writing *"connector-level
  admissibility now holds corpus-wide"* when what holds is *"every committed stream loads with its
  declared context; fact-kind coverage is proven only where a capability record narrows it."* Write
  the weaker true sentence.
- **A doc comment must be TRUE.** Three reviews caught docs asserting behaviour the code did not
  have. The new test docs make claims about what the walk proves — hold them to the code.
- **DRY, with deliberate redundancy protected.** Six copies of the `obs_id` loop is accidental
  duplication → extract (AC1). `expected()` vs the committed bytes, and the minimal round-trip vs
  the corpus-wide one, are deliberate second oracles → keep both, and label them (AC3).
- **Document every public item.** Nothing new becomes `pub` here; `walk_replay_streams` becomes
  `pub(crate)` under `#[cfg(test)]` and keeps its doc.
- **File-size gate:** ≤2000 CODE lines, tests excluded, counted to the first `#[cfg(test)]`.
  `fixtures.rs` counts **698** today; the hoist raises it by the hoisted doc comment's length. Far
  under the ceiling either way — and it is a rise, not a reduction.
- **Dependency frontier (D47):** all work here is in `opencmdb-bin`. `opencmdb-core` is untouched;
  do not reach for `anyhow`/`axum`/`sqlx`/`askama` anywhere near it.
- **`DATABASE_URL` is usually unset locally** and the MariaDB-backed tests `return` early either
  way — a green suite says nothing about the database. Irrelevant to this story's guards, but do not
  cite a green suite as evidence of anything it did not run.
- **Known local flakiness (issue #38):** the suite has shown unexplained non-determinism (8 failures
  across 5 runs, identical sha256, clean `git status`, then 15+ green — stories 4.15/4.17). CI on a
  clean checkout has never reproduced it and **the "Synology Drive" explanation is REFUTED by
  measurement — do not re-adopt it**. If a corpus test reds unexpectedly, re-run and check
  `git status` before diagnosing; if it persists, that is data for #38, not a cause you may write
  down. *A cause needs a check, not a plausible story.*

### Testing standards

Tests live inline in the trailing `#[cfg(test)] mod tests` (D56b, one per file). Test names are
sentences (`the_corpus_carries_no_real_network_data`). Assertion messages name the offending FILE —
with a corpus walk, *"a stream is inadmissible"* is not actionable unless it says which. Prefer
`#[should_panic(expected = "…")]` with a substring when pinning a panic, so a pass-for-the-wrong-panic
is impossible (4.14's idiom, reused by 4.17).

### Project Structure Notes

Paths follow the established layout with no variance: corpus at the workspace root in `fixtures/`
(D56 — a file under `tests/` reads as the property of the test); the fixture reader and
`FixtureConnector` in `crates/opencmdb-bin/src` beside `arp_ping.rs`, deliberately in the shipped
crate so *"zero mocks"* is a gate rather than a slogan; `FIXTURES_DIR` expressed exactly once
(`fixtures.rs:48`) — take the path from `fixtures_dir()`/`fixture_path()`, never re-write the string
(there is a test for this: `the_fixtures_path_is_expressed_once`, `fixtures.rs:1734`, and Task 3's
path conversion is where a dev is most likely to trip it).

One judgment call to be aware of: the connector walk (AC2) lives in `fixture_connector.rs` and the
round-trip walk (AC3) in `fixtures.rs`, each at its own layer, sharing ONE walker via the Task 2
hoist. The alternative — both tests in `fixtures.rs` — avoids the hoist but makes the lower layer's
tests depend on the higher one. The hoist is the recommendation; if you take the alternative, say
why in the completion record.

### References

- Story source: [Source: _bmad-output/planning-artifacts/epics.md#Story 5.1] (epics.md:1316-1334);
  Epic 5 framing and build order, epics.md:1306-1314; the inherited-debt-at-the-head decision,
  epics.md:1312.
- Deferred entries: [Source: _bmad-output/implementation-artifacts/deferred-work.md] — CLOSED by
  this story: story-4.10 review (line 336), story-4.12 review (line 353), story-4.15 review
  (line 398). **Annotated still-open:** story-4.13 review (lines 365-378). The closure idiom and the
  *"a register that loses an item"* lesson are at lines 14, 16, 17, 126, 131.
- Corpus doctrine (fixture-as-spec, fixture-as-connector, obs_id as the anchor, no clock):
  [Source: _bmad-output/planning-artifacts/architecture.md#D19] and the module docs of
  `fixtures.rs:1-30` and `fixture_connector.rs:1-68`.
- Descriptor travels with the batch, positional containment, *"the past would change status"*:
  [Source: architecture.md#D34 §1] via `fixture_connector.rs:153-260` (the capability-ordering
  checks specifically at `:187-206`, the fact-kind check at `:236-247`).
- Corpus lock, both directions: `fixtures/MANIFEST.toml:1-13`; gate in `xtask/src/main.rs`.
- File-size rule and its counting method: `xtask/src/main.rs:72-81` [Source: CLAUDE.md#Engineering
  conventions].
- Story flow, PR discipline, local gate, `architecture-views.md` staleness:
  [Source: docs/project-context.md] §"Working conventions" and lines 127-130.
- The two previous stories' shape (byte-pin idiom, prove-to-red records):
  `_bmad-output/implementation-artifacts/4-17-trap-hostname-absent-or-empty.md`,
  `…/4-18-wire-format-traps.md`.

### Git intelligence

Last five commits: `8c81386` (Epic 5 decomposition, PR #40) · `b568815` (Epic 4 retrospective, #39)
· `b4ceca3` (Epic 4 report, #37) · `199ba79` (Epic 4 closure, #36) · `7b93c20` (Story 4.18, #35).
`fixtures.rs` was last touched by 4.18, 4.17, 4.16, 4.15, 4.14 — each adding one byte-pin test and,
from 4.15 onward, its `obs_id` pins. **This story is the back-fill of the two families that predate
that rule.** Every one of those commits went through a branch → PR → green CI → squash merge; hold
the same.

### Toolchain

No new dependency, no version to research. Rust 1.96+, edition 2024; `serde`/`serde_json` already in
`opencmdb-bin`'s tree and already used by both files. Pin nothing new — and never invent a version;
`Cargo.lock` is committed and every build is `--locked`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
