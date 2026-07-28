# Story 5.2b: The four unpinned families — and dhcp-churn's authored values — state their premise in a test, not only in prose

Status: review

<!-- Validation is MANDATORY here (Guy's decision, Epic 4 retrospective 2026-07-26): two
     fresh-context agents (fact-check + gap-hunt) before dev-story. The template banner saying
     validation is optional does not apply to this project.
     DONE 2026-07-28 — 3 HIGH, 6 MED, 3 LOW, all applied. The load-bearing one added AC4b.
     See the Change Log. -->

## Story

As the owner of the corpus,
I want a byte-pin test for the `randomized-mac`, `multi-nic`, `shared-hardware-vm` and `cloned-mac`
families, and the authored values of `dhcp-churn` pinned by value,
so that no committed family can state a premise its own bytes contradict, and no trap can pass for
the wrong reason.

**This is the LAST of Epic 5's three inherited-debt stories** (epics.md:1317, Guy's decision at Epic
4's retrospective). 5.1 closed byte-fidelity's *binding* theme, 5.2 closed the *privacy* theme; this
one closes byte-fidelity's *value* theme.

⚠️ **The epic's closing promise is STALE and this story must not repeat it.** epics.md:1397 says that
after 5.2b *"the corpus byte-fidelity theme carries no open item"* — true when written on 2026-07-26,
falsified the next day by story 5.1's own code review, which registered
`scenario/wire/unifi-clients.expected.jsonl` as having **no round-trip byte-shape pin at all**
(`deferred-work.md:590`, owner **Epic 11's wire parser**, issue #34 — it sits outside
`scenario/replay/` and gains a pin when it gains a consumer). What this story may claim is the
narrower true sentence: **every register entry whose owner is *"whoever hardens corpus
byte-fidelity"* is closed.** See AC7.

It sits ahead of 5.5 deliberately (epics.md:1363): the corpus is the oracle the L1 join is about to
be judged against, and hardening an oracle after its first consumer exists means bending the engine
to fit whatever the corpus happens to say.

**No ARTEFACT bytes change in this story.** Every value below was read off the committed corpus on
2026-07-28 at `95d4d69`. If a pin appears to require re-authoring an artefact or re-hashing
`MANIFEST.toml`, STOP: that is a finding, not a task.

## Acceptance Criteria

1. **AC1 — `randomized-mac.jsonl` is pinned, and the one octet its whole family rests on is pinned
   by VALUE.**
   **Given** a stream named by no value test — its only mention is the per-stream context table
   story 5.1 added (`fixture_connector.rs:1519`), which states the stream's declared context and
   asserts nothing about its contents — while `read_traps` checks only that a trap's `obs_id`s
   EXIST, never which line they name **when** the stream is pinned **then** all of the following
   hold, each as its own assertion:
   - exactly **3** observations, `obs_id` prefix `eeeeeeee`, via `assert_obs_ids(&obs, "eeeeeeee", 3)`;
   - **every line carries exactly 2 facts** (`Mac` + `IpV4`) — uniform here, which is what lets the
     `find()`-one-of-each extraction be exact;
   - N1 and N2 carry the byte-identical MAC **`02:00:5e:00:53:20`** = `[2, 0, 94, 0, 83, 32]`,
     **value-pinned on each line, not merely asserted equal to one another**;
   - N3 carries **`02:00:5e:00:53:21`** = `[2, 0, 94, 0, 83, 33]`, value-pinned;
   - the three addresses are **`192.0.2.30` / `.31` / `.32`**, value-pinned;
   - the three instants are the authored vector `2026-01-02T00:00:00Z` / `01:00:00Z` / `02:00:00Z`.

   **Why value-pinning rather than `assert_eq!(mac(0), mac(1))`:** the family's `must-merge` trap
   (`l1-exact-mac`, judging obs **001+002**) and its `must-not-merge` trap (`l1-distinct-mac`,
   judging **001+003**) differ by ONE octet. A relational pin stays green if BOTH N1 and N2 are
   re-authored to `…:21` — at which point the `must-not-merge` pair becomes a same-MAC pair and the
   corpus demands the opposite decision, silently. That is the exact failure this story exists to
   stop, and it is why 4.13's relational-only pin is an open register item.

2. **AC2 — `multi-nic.jsonl` is pinned, and BOTH halves of the `Uplink` fact are pinned.**
   **Given** a stream whose premise is entirely geometric and which the harness validates nowhere
   (the VRRP byte-pin's own doc says uplink geometry is pinned *"here or nowhere"* — true precisely
   because VRRP HAS a byte-pin) **when** it is pinned **then**:
   - exactly **3** observations, prefix `ffffffff`; **3 facts on every line** (`Mac` + `IpV4` +
     `Uplink`);
   - M1 and M2 carry the **same** `peer_mac` `02:00:5e:00:60:0a` = `[2, 0, 94, 0, 96, 10]` and
     **different** `peer_port` — `swport-1` and `swport-2` — with **the whole `Uplink` fact
     value-pinned on each line, both halves**;
   - M3 carries a **different** `peer_mac` `02:00:5e:00:60:0b` = `[2, 0, 94, 0, 96, 11]` with
     `swport-7`, value-pinned;
   - the host MACs `02:00:5e:00:53:40` / `:41` / `:42` and the addresses `192.0.2.40` / `.41` / `.42`
     are value-pinned.

   The two traps are `must-merge`/`l2-uplink-agrees` on **001+002** and
   `must-not-merge`/`l2-different-switch` on **001+003**. Pinning both halves is what stops *"same
   switch, different port = agrees"* and *"different switch = opposes"* being silently exchanged.
   **And collapsing the two ports into one must red** — that edit would turn this family into the
   shared-hardware-vm shape, where an identical uplink is exactly what does NOT discriminate.

3. **AC3 — `shared-hardware-vm.jsonl` is pinned, and W4's ABSENT hostname is an assertion rather
   than an accident.**
   **Given** a trap header declaring the uplink *"shared by construction (the same `peer_mac` and
   `peer_port` on every observation)"* — prose that no test asserts **when** it is pinned **then**:
   - exactly **4** observations, prefix `abababab`;
   - **all four carry the byte-identical `Uplink`** — peer `02:00:5e:00:60:0a` = `[2, 0, 94, 0, 96,
     10]`, port `swport-1` — value-pinned on each line;
   - W1 and W2 carry `doc-vm-alpha`, W3 carries `doc-vm-beta`, all value-pinned;
   - **W4 carries NO `Hostname` fact**, asserted directly in story 4.17's idiom —
     `assert!(observations[3].facts.iter().all(|f| !matches!(f, Fact::Hostname { .. })))` — not
     inferred from a fact count;
   - the four MACs `02:00:5e:00:53:50` / `:51` / `:52` / `:53` are distinct and value-pinned, and the
     addresses are `192.0.2.80` / `.81` / `.82` / `.83`.

   Traps: `must-merge`/`l2-hostname-agrees` on **001+002**, `must-not-merge`/`l2-different-hostname`
   on **001+003**, `must-abstain`/`NoObservedValue` on **001+004**. Pinning the identical uplink is
   what keeps the discriminator the HOSTNAME: were the uplink allowed to drift, the `must-merge` pole
   could start passing for a topological reason the family explicitly denies.

   ⚠️ **The fact count here is NOT uniform — 4, 4, 4, 3.** A blanket per-line
   `assert_eq!(facts.len(), 4)` copied from the dhcp-churn idiom REDS on W4. Assert the **exact
   per-line vector** `[(0, 4), (1, 4), (2, 4), (3, 3)]`, in the idiom
   `the_hostname_absence_stream_encodes_empty_and_missing_and_never_null` already uses
   (`fixtures.rs:3053-3059`) and the VRRP pin too.

   **`>= 3` is NOT an acceptable alternative, and the choice an earlier draft offered is withdrawn —
   measured.** Adding a SECOND, contradicting `Uplink` to W4 (`peer_mac [2,0,94,0,96,11]`,
   `swport-99`) reds nothing on today's tree, and would still red nothing under a `>= 3` shape: the
   one-of-each `find()` returns the FIRST `Uplink`, which is the authored one, so the value pin
   passes while the family's *"shared by construction"* premise is false on the very pole that
   depends on it. Only the exact vector reds. The non-uniform count is the REASON the exact vector is
   needed, not a reason to relax it — and it is the same vacuity the dhcp-churn idiom's own comment
   names (*"without it, `find()` takes the FIRST match of each kind and a duplicated or extra fact
   would pass every assertion unnoticed"*).

4. **AC4 — `cloned-mac.jsonl` is pinned on every line, not pairwise.**
   **Given** the corpus's ONLY pre-release guard against the false merge — D21 refuses a unique index
   on `interface.mac_canon` deliberately, so the schema cannot be one, and D10 calls the false merge
   catastrophic and asymmetric **when** it is pinned **then**:
   - exactly **3** observations, prefix `acacacac`; **3 facts on every line**;
   - **all three** carry the one byte-identical MAC **`02:00:5e:00:53:70`** = `[2, 0, 94, 0, 83,
     112]`, **value-pinned on each of the three lines, not pairwise**;
   - K1 and K3 carry `doc-host-echo`, K2 carries `doc-host-foxtrot`, value-pinned;
   - the three addresses `192.0.2.112` / `.113` / `.114` are pinned and distinct;
   - the three `obs_id`s are pinned (`assert_obs_ids`).

   Traps: `must-not-merge`/`l2-different-hostname` on **001+002**, `must-merge`/`l1-exact-mac` on
   **001+003**. Two edits must be caught and neither is caught today: a **one-octet edit** to any
   line's MAC would turn the `must-not-merge` into a tautology any engine passes; an **`obs_id`
   permutation** swapping 002 and 003 would make the corpus DEMAND the false merge — pairing the two
   `doc-host-echo` presences under `must-not-merge` and the echo/foxtrot pair under `must-merge`.
   Pinning per line rather than pairwise is what reaches the first.

   ⚠️ **The second is reached by `assert_obs_ids` ONLY on the stream side, and the same inversion is
   available from the TOML side where nothing reaches it — see AC4b.** An earlier draft of this AC
   claimed `assert_obs_ids` closed it outright; that was wrong, and the correction is measured.

4b. **AC4b — each family's TRAP→`obs_id` binding is pinned, because every pin above lives on the
   `.jsonl` and the inversion is available entirely from the `.toml`.**
   *(Inserted 2026-07-28 by this story's mandatory validation pass — the house letter-suffix idiom,
   D56b, so AC5–AC7 keep their numbers.)*

   **Given** that AC1–AC4 pin only stream bytes, while a trap file declares WHICH pair of `obs_id`s
   it judges and under WHICH column and rule — a mapping the Dev Notes tabulate for all nine traps
   and that **no test asserts** (`read_traps` builds a `BTreeSet<Uuid>` per stream and only tests
   membership; `trap_gate`'s completeness check only asks that both poles of a family exist, which
   any exchange preserves) **when** each family is pinned **then** the test also reads the family's
   trap file and pins, per trap id, **the exact `observations` vector and the exact
   `(column, rule)` pair** — for `cloned-mac`: `cloned-mac-must-not-merge` judges
   `[…0001, …0002]` under `must-not-merge`/`l2-different-hostname`, and `cloned-mac-must-merge`
   judges `[…0001, …0003]` under `must-merge`/`l1-exact-mac`.

   **This is MEASURED, and it is the reason the AC exists.** Exchanging the two `observations`
   vectors in `fixtures/scenario/traps/cloned-mac.toml` — three characters, no `.jsonl` byte touched
   — makes the corpus **demand the false merge**: the echo/foxtrot pair (two real hosts, one wearing
   a clone of the other's MAC) becomes `must-merge`/`l1-exact-mac`, and the two genuine
   `doc-host-echo` presences become `must-not-merge`. `cargo test --workspace` was run in that state
   and reported **130 + 86 + 42, zero failures** (2026-07-28, story validation; file restored, md5
   verified, `git status fixtures/` empty). Every pin AC1–AC4 asks for stays green, because the
   stream did not move. This is D10's catastrophic-and-asymmetric direction, in the family the corpus
   itself calls *"the corpus's ONLY pre-release guard against the false merge"*.

   The sha256 lock is not the backstop: this corpus's stated threat model (`fixtures.rs:1085`) is
   *"a DELIBERATE re-authoring, which refreshes `MANIFEST.toml` by definition."*

   **Scope:** all five families, i.e. all nine traps in the Dev Notes table (the four families' eight
   plus `dhcp-churn`'s two — nine rows there, ten traps counting dhcp-churn's; pin what the table
   records for the family you are working on). Reuse `read_traps`; do not re-parse TOML by hand.

5. **AC5 — `dhcp-churn.jsonl`'s three authored values are pinned by VALUE, extending the existing
   test rather than adding a second one.**
   **Given** `the_dhcp_churn_stream_moves_the_address_only_through_observed_at`
   (`fixtures.rs:2311`), which asserts its MACs and hostnames only RELATIONALLY
   (`assert_eq!(mac(0), mac(1))`, `assert_ne!(hostname(2), hostname(0))`) while the trap file's
   `reason` strings cite concrete values no test asserts (registered under story 4.13's review — one
   of the TWO entries this story closes, not "the last open one"; see AC7) **when** the existing test
   is extended **then**
   `02:00:5e:00:53:78` = `[2, 0, 94, 0, 83, 120]`, `doc-host-golf` and `doc-host-hotel` are pinned by
   value, so a re-authored stream carrying different synthetic values can no longer strand its own
   reasons.

   **CORRECTION to the epic's wording, measured.** epics.md:1391 says *"both `reason` strings cite
   `02:00:5e:00:53:78`, `doc-host-golf` and `doc-host-hotel`"*. Counted on the committed file: the
   MAC appears in **one** reason (`dhcp-churn.toml:39`), `doc-host-hotel` in **one** (`:28`), and
   `doc-host-golf` in **both**. The UNION of the two reasons cites all three; neither reason cites
   all three. The conclusion is unchanged — those three values are cited by prose and asserted by no
   test — but the story must not repeat the false distribution. *(Same class of correction as story
   5.2's AC6 finding; recorded here rather than discovered mid-task.)*

   **The same false sentence is in the register bullet AC7 tells you to close** —
   `deferred-work.md:400` opens *"The constants both `reason` strings cite (`02:00:5e:00:53:78`,
   `doc-host-golf`, `doc-host-hotel`)"*. Because closure is append-and-strike, the wrong sentence
   survives in the file unless the closure note carries the counted distribution. Put it there.

   **Extend, do not duplicate.** A second dhcp-churn test would make "the dhcp-churn premise" mean
   two sets, which is the accidental-duplication the DRY rule forbids and exactly what AC5 of story
   5.2 hoisted a walk to avoid.

6. **AC6 — every pin is proven to red before it passes, one recorded mutation per family, each
   aimed at a stream no OTHER value test reads.** (House rule, story 1.3; the aiming rule is story
   5.1's lesson — a red observed in a stream that three tests read cannot tell you WHICH guard
   caught it.) Five mutations, at minimum:
   - **randomized-mac:** flip N3's MAC last octet `33` → `32`, observe the new pin red naming the
     line. Restore.
   - **multi-nic:** change M2's `peer_port` `swport-2` → `swport-1` (the port-collapse AC2 names),
     observe red. Restore.
   - **shared-hardware-vm:** change W3's `peer_port` on one line only, observe red; **and
     separately** REPLACE W4's `Uplink` fact with exactly
     `{"Hostname":{"name":"doc-vm-delta","source":"Dhcp"}}`, observe the absence assertion red. Two
     observations for this family — the shared-uplink pin and the absence pin fail independently.
     ⚠️ **REPLACE, not add, and those exact contents — all three constraints are measured:**
     *adding* a fourth fact takes W4's count 3 → 4, so the per-line count vector reds FIRST and the
     recorded red names the count, not the absence; a name without the `doc-` prefix also reds
     `the_corpus_carries_no_real_network_data` (`fixtures.rs:1422`), giving two reds from one
     mutation; and a lowercase `"dhcp"` fails `deny_unknown_fields` and reds every corpus walk. With
     `doc-vm-delta` + `Dhcp`, zero existing tests red and the new absence pin is the sole red.
   - **cloned-mac:** flip K2's MAC last octet, observe red. **And a second, on the TOML side (AC4b):**
     exchange the two `observations` vectors in `cloned-mac.toml`, observe AC4b's binding pin red.
     Measured during validation: without AC4b that exchange leaves the whole suite green while the
     corpus demands the false merge.
   - **dhcp-churn:** change `doc-host-hotel` to another `doc-` name — the existing relational
     `assert_ne!` stays GREEN and only the new value pin reds. **Record both halves**: that
     green-then-red pair is what demonstrates the value pin adds something the relational one did
     not, and it is the whole justification for story 4.13's register entry.

   **Every mutation edits a committed artefact.** `cargo test` does not consult `MANIFEST.toml`
   (measured: the workspace suite is fully green under each of these mutations), so no re-hash is
   needed and none must be committed — but `git checkout` the file after EACH one and verify
   `git status fixtures/` is clean before moving on.

   ⚠️ **`cargo xtask ci` DOES consult `MANIFEST.toml`, and its `fixtures` gate reds with
   `sha256 mismatch` on any mutated artefact.** Never run the gate with a mutation in place: the red
   is real, unrelated to your change, and this story elsewhere primes you to suspect issue #38. AC7's
   gate run is the LAST thing, after the last `git checkout`, with `git status fixtures/` verified
   empty first.

7. **AC7 — the local gate is green, no corpus byte moves, and the register is closed by APPENDING.**
   `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, `cargo xtask ci` all pass; `git status` shows **no change under
   `fixtures/` at all** (this story, unlike 5.2, does not even touch a `README.md` — if one turns out
   to need a change, say so and do it, but it is not expected). `views-hash STALE` is expected and
   exits 0 — **do not regenerate `architecture-views.md`**.

   Two register entries are marked closed in place, in the file's established idiom
   (`✅ **CLOSED by story 5.2b.** ~~old text struck~~ …`):
   - `## Deferred from: code review of story-4.13` (`deferred-work.md:397`) — **BOTH bullets**: the
     relational-pin entry at `:399-409` **and** its `↺ **STILL OPEN after story 5.1**` follow-up at
     `:410-415`, whose *"Owner unchanged: whoever pins the authored VALUES corpus-wide"* is this
     story. Closing only the first leaves the section ending on a bullet that asserts the item is
     open and names an unfilled owner — the register-contradicts-the-code failure this AC exists to
     prevent;
   - `## Deferred from: story-5.1` (`:516`) — the *"Four committed family streams are named by no
     VALUE test"* bullet at `:521`, whose own text names **story 5.2b as its owner**.
     ⚠️ **That heading has no "code review of" in it, and the register says why in as many words:**
     5.1 raised this while SCOPING itself, not in its review. The sibling section
     `## Deferred from: code review of story-5.1` (`:556`) is a different list and is **not** what
     this story closes. Do not strike a bullet in the wrong section.

   **What may be claimed, and what may not.** Write the narrow true sentence: *"every register entry
   whose owner is 'whoever hardens corpus byte-fidelity' is now closed."* Do **not** write *"the
   corpus byte-fidelity theme carries no open item"* — epics.md:1397 says it and it is stale. Two
   things stay open and both must still read as open after this story:
   - **`scenario/wire/unifi-clients.expected.jsonl` has no round-trip byte-shape pin at all**
     (`deferred-work.md:590`). It sits outside `scenario/replay/`, so 5.1's corpus-wide witness
     cannot reach it by design. Owner: **Epic 11's wire parser** (`CONSUMER PENDING`, issue #34).
     Not this story's.
   - **`example-traps.jsonl` is a SIXTH committed stream named by no value test** — surfaced by this
     story's validation pass, not previously registered. Its only mentions in `crates/` are the
     context table (`fixture_connector.rs:1518`), a `score.rs` unit-test string, and a prose comment;
     and `example.toml:26`'s reason cites `02:00:5e:00:53:10` — verbatim the 4.13 shape AC5 closes.
     **Decide explicitly and say which you took:** pin it as a sixth test (it is cheap — 3 lines,
     2 facts each, prefix `bbbbbbbb`, MACs `[2,0,94,0,83,16]` ×2 then `[…,17]`, IPs
     `192.0.2.20/.21/.22`), or register it under `## Deferred from: story-5.2b` with a named owner.
     What is NOT acceptable is closing the theme without mentioning it.

   **Verify the claim by re-reading the register AFTER the last edit, not before** — story 5.1's
   review found a citation its own diff had falsified, and 5.2's replaced a false sentence with
   another its own commit falsified. Anything else surfaced and not fixed goes under
   `## Deferred from: story-5.2b` with a named owner.

## Tasks / Subtasks

- [x] **Task 1 — read the idiom before writing a line of it (AC1–AC5)**
  - [x] Read `the_dhcp_churn_stream_moves_the_address_only_through_observed_at`
        (`fixtures.rs:2305-2380`) end to end. It is the shape to copy: `read_jsonl` + `fixture_path`,
        a per-line fact-count loop, `assert_obs_ids`, the `fact`/`mac`/`ip`/`hostname` closure
        triple, then value assertions, then the instants vector.
  - [x] Note what NOT to copy: its strictly-increasing instants assertion. **Three of the four new
        families do not have strictly increasing instants** (Dev Notes). Copying that line reds.
  - [x] Confirm `assert_obs_ids(&observations, prefix, expected_len)` takes a LENGTH (story 5.1)
        and that its `{:012}` decimal rendering is safe for streams of ≤ 9 lines — all five here are
        3 or 4.

- [x] **Task 2 — `randomized-mac` (AC1, AC6)**
  - [x] New test beside the existing byte-pins, named for what it proves (e.g.
        `the_randomized_mac_stream_rests_on_one_octet`).
  - [x] Per-line fact count 2; `assert_obs_ids(…, "eeeeeeee", 3)`; MAC value-pinned on EACH of the
        three lines; three addresses; the instants vector.
  - [x] Prove to red: N3's last octet `33` → `32`. Record the exact message. `git checkout`.

- [x] **Task 3 — `multi-nic` (AC2, AC6)**
  - [x] Per-line fact count 3; `assert_obs_ids(…, "ffffffff", 3)`; the whole `Uplink` fact
        value-pinned per line (both `peer_mac` and `peer_port`); host MACs and addresses pinned.
  - [x] ⚠️ M1 and M2 share the instant `2026-01-03T00:00:00Z`. Pin the vector as authored; do NOT
        assert strict increase.
  - [x] Prove to red: M2's `swport-2` → `swport-1`. Record. `git checkout`.

- [x] **Task 4 — `shared-hardware-vm` (AC3, AC6)**
  - [x] ⚠️ Fact counts are **4, 4, 4, 3** — assert the EXACT per-line vector
        `[(0,4),(1,4),(2,4),(3,3)]`. A uniform `assert_eq!(len, 4)` reds on W4; `>= 3` is measured
        vacuous and is not an option (AC3).
  - [x] `assert_obs_ids(…, "abababab", 4)`; the identical `Uplink` value-pinned on all FOUR lines;
        the three hostnames; W4's hostname ABSENCE via 4.17's `.iter().all(|f| !matches!(…))`; the
        four MACs and four addresses.
  - [x] Prove to red TWICE: (a) one line's `peer_port` changed; (b) a `Hostname` fact added to W4.
        Record both. `git checkout` after each.

- [x] **Task 5 — `cloned-mac` (AC4, AC6)**
  - [x] Per-line fact count 3; `assert_obs_ids(…, "acacacac", 3)`; the ONE MAC value-pinned on each
        of the three lines (**not** `assert_eq!(mac(1), mac(0))`); hostnames echo/foxtrot/echo;
        three distinct addresses.
  - [x] Prove to red: K2's MAC last octet flipped. Record. `git checkout`.

- [x] **Task 5b — pin each family's TRAP→`obs_id` binding (AC4b, AC6)**
  - [x] For every family, read its trap file with `read_traps(&fixture_path("scenario/traps/<name>.toml").unwrap())`
        and pin, per trap id, the exact `observations` vector and the exact `(column, rule)` pair.
        The Dev Notes table is the authority for all nine rows.
  - [x] Decide and say where this lives: folded into each family's byte-pin test (one test per
        family proves one family end to end), or one test over all five. **Recommendation: fold it
        in** — the AC exists because stream-side and TOML-side pins defend the SAME premise, and
        splitting them re-opens the gap where "the family" means two sets.
  - [x] Prove to red: exchange the two `observations` vectors in `cloned-mac.toml`. Without this AC
        the whole suite stays green while the corpus demands the false merge — that was measured
        during validation and is the reason the AC exists. `git checkout` after.

- [x] **Task 6 — extend `dhcp-churn` (AC5, AC6)**
  - [x] EXTEND the existing test — do not author a second one. Add value pins for
        `02:00:5e:00:53:78`, `doc-host-golf`, `doc-host-hotel`; keep every existing assertion.
  - [x] Prove to red with the TWO-SIDED observation: rename `doc-host-hotel`, confirm the existing
        `assert_ne!` stays GREEN, confirm the new value pin REDS. Record both halves — this pair IS
        the justification for closing 4.13's register entry.

- [x] **Task 7 — the register and the gate (AC7)**
  - [x] Close **both** story-4.13 bullets (`:399-409` and the `↺ STILL OPEN` follow-up at
        `:410-415`) and the story-5.1 four-families bullet (`:521`) in place, appending. Carry the
        counted `reason` distribution into the 4.13 closure note — the bullet's own first sentence
        repeats the epic's false "both reason strings cite" (AC5).
  - [x] Re-read the register AFTER the last code edit and write the NARROW claim (AC7): every entry
        owned by *"whoever hardens corpus byte-fidelity"* is closed; `unifi-clients.expected.jsonl`
        (`:590`, owner Epic 11) still reads open.
  - [x] Decide `example-traps.jsonl` explicitly — pin it as a sixth test, or register it with a
        named owner. Do not close the theme without mentioning it (AC7).
  - [x] Open `## Deferred from: story-5.2b` for anything surfaced and not fixed, with an owner.
  - [x] Full local gate, all four commands. `git status` under `fixtures/` **empty**. Report the
        `fixtures.rs` file-size number and why it did not move (all of this lands after the first
        `#[cfg(test)]`).
  - [x] Update `sprint-status.yaml` and `docs/project-context.md`'s test count. **Set `review`, not
        `done`** — `done` is the merge's business (5.1 established it; 5.2's review confirmed it,
        and 5.2's own code-review workflow would have set `done` wrongly).
  - [ ] Branch → `code-review` → push → PR → green CI → squash merge, **in that order**.
        *(NOT done by `dev-story` — the work is still on `master` in the working tree, uncommitted.
        This is the next workflow's business, and `done` is the merge's. Left unchecked rather than
        marked complete.)*

## Dev Notes

### What was measured, before the story was written

Every value below was read off the committed corpus on 2026-07-28 at `95d4d69`. They exist so the
dev does not re-derive them, and so a surprise during implementation reads as a FINDING.

**The five streams, verbatim.** MAC byte arrays are as committed; the hex form is what
`MacAddr::Display` renders.

| Stream | Lines | `obs_id` prefix | Facts/line | The premise, in bytes |
|---|---|---|---|---|
| `randomized-mac` | 3 | `eeeeeeee` | 2, 2, 2 | MAC `[2,0,94,0,83,32]` = `02:00:5e:00:53:20` on N1 **and** N2; `[…,33]` = `:21` on N3. IPs `.30/.31/.32` |
| `multi-nic` | 3 | `ffffffff` | 3, 3, 3 | `Uplink` peer `[2,0,94,0,96,10]` = `02:00:5e:00:60:0a` port `swport-1` (M1) / **same peer**, `swport-2` (M2) / peer `[…,96,11]` = `:0b`, `swport-7` (M3). Host MACs `:40/:41/:42`, IPs `.40/.41/.42` |
| `shared-hardware-vm` | 4 | `abababab` | **4, 4, 4, 3** | `Uplink` `[2,0,94,0,96,10]` + `swport-1` on **all four**. Hostnames `doc-vm-alpha`, `doc-vm-alpha`, `doc-vm-beta`, **none**. MACs `:50/:51/:52/:53`, IPs `.80/.81/.82/.83` |
| `cloned-mac` | 3 | `acacacac` | 3, 3, 3 | MAC `[2,0,94,0,83,112]` = `02:00:5e:00:53:70` on **all three**. Hostnames `doc-host-echo`, `doc-host-foxtrot`, `doc-host-echo`. IPs `.112/.113/.114` |
| `dhcp-churn` | 3 | `adadadad` | 3, 3, 3 | MAC `[…,120]` = `:78` on D1+D2, `[…,121]` = `:79` on D3. Hostnames `golf`, `golf`, `hotel`. IPs `.120`, `.121`, `.120` |

**Which `obs_id`s each trap actually judges** — measured, because a pin that ignores this can be
green while the family is inverted:

| Family | Trap | Judges |
|---|---|---|
| randomized-mac | `must-not-merge` / `l1-distinct-mac` | 001 + 003 |
| randomized-mac | `must-merge` / `l1-exact-mac` | 001 + 002 |
| multi-nic | `must-merge` / `l2-uplink-agrees` | 001 + 002 |
| multi-nic | `must-not-merge` / `l2-different-switch` | 001 + 003 |
| shared-hardware-vm | `must-merge` / `l2-hostname-agrees` | 001 + 002 |
| shared-hardware-vm | `must-not-merge` / `l2-different-hostname` | 001 + 003 |
| shared-hardware-vm | `must-abstain` / `NoObservedValue` | 001 + 004 |
| cloned-mac | `must-not-merge` / `l2-different-hostname` | 001 + 002 |
| cloned-mac | `must-merge` / `l1-exact-mac` | 001 + 003 |

### Every MAC this story pins, in the DECIMAL bytes the assertion needs

Assertions go through `MacAddr([u8; 6])`, which is decimal. Do not convert by hand — **position 5 is
the literal decimal `83`, which is `0x53`, and only the LAST octet varies.** Reading `02:00:5e:00:53:40`
as `[2,0,94,0,83,40]` is the single most likely transcription slip, and it produces a red pin that
this story has primed you to read as a corpus FINDING. It would not be one.

| Stream | Decimal arrays |
|---|---|
| `randomized-mac` | hosts `[2,0,94,0,83,32]` (N1, N2) · `[2,0,94,0,83,33]` (N3) |
| `multi-nic` | hosts `[2,0,94,0,83,64]` / `[…,65]` / `[…,66]` · peers `[2,0,94,0,96,10]` (M1, M2) · `[2,0,94,0,96,11]` (M3) |
| `shared-hardware-vm` | hosts `[2,0,94,0,83,80]` / `[…,81]` / `[…,82]` / `[…,83]` · shared peer `[2,0,94,0,96,10]` on all four |
| `cloned-mac` | `[2,0,94,0,83,112]` on all three |
| `dhcp-churn` | `[2,0,94,0,83,120]` (D1, D2) · `[2,0,94,0,83,121]` (D3) |

`Fact::Uplink { peer_mac: MacAddr, peer_port: String }`, so the pin is
`Fact::Uplink { peer_mac: MacAddr([2,0,94,0,96,10]), peer_port: "swport-1".into() }`. `Fact` is
`#[non_exhaustive]` **at the enum level only** — constructing a variant from `opencmdb-bin` is legal
(every existing pin does it); an exhaustive `match` on it is not, so use `matches!`.

### What STOP means, procedurally

The Story section says a pin that requires re-authoring an artefact is a FINDING, not a task. What to
do when it happens: **(a)** do not edit the artefact or the manifest; **(b)** write the pin against
the committed bytes AS THEY ARE, and say in its doc comment what the trap prose claims and what the
bytes actually hold; **(c)** record the contradiction under `## Deferred from: story-5.2b` with a
named owner and the trap file and line; **(d)** carry on with the remaining families — one
contradicted family does not block the other four. Only a contradiction that makes a pin impossible
to write at all sets the story `blocked`.

### ⚠️ THREE traps measured for the dev, each of which would cost an hour

1. **`shared-hardware-vm`'s fact count is NOT uniform: 4, 4, 4, 3.** W4 has no `Hostname`, which is
   the family's `must-abstain` pole. The dhcp-churn idiom's per-line loop
   (`assert_eq!(observation.facts.len(), 3)`) is uniform because that stream is; copying it here
   with `4` REDS on W4 and reads as a corpus defect. It is not one — it is the premise.

2. **Instants are NOT strictly increasing in three of the four families.** Measured:
   - `randomized-mac`: `00:00`, `01:00`, `02:00` (2026-01-02) — strictly increasing;
   - `multi-nic`: `00:00:00`, **`00:00:00`**, `00:05:00` (2026-01-03) — M1 and M2 share an instant;
   - `shared-hardware-vm`: `00:00`, **`00:00`**, `00:05`, `00:10` (2026-01-04) — W1 and W2 share one;
   - `cloned-mac`: `00:00`, **`00:00`**, `01:00` (2026-01-05) — K1 and K2 share one.

   dhcp-churn's byte-pin asserts strict increase *because the churn lives in time alone*; that is
   specific to it. Here, two NICs of one host seen in the same sweep SHOULD share an instant. Pin the
   authored vector; do not assert an ordering the corpus deliberately does not have.

3. **The epic's dhcp-churn clause is wrong about distribution.** epics.md:1391 says *"both `reason`
   strings cite"* all three values. Counted: the MAC is in one reason, `doc-host-hotel` in one,
   `doc-host-golf` in both. See AC5 — the conclusion holds, the sentence does not.

### The idiom to reuse, and where it lives

`fixtures.rs:2305-2380`, `the_dhcp_churn_stream_moves_the_address_only_through_observed_at`. Its
shape, in order: `read_jsonl(&fixture_path("scenario/replay/<name>.jsonl").unwrap())` → length
assertion → per-line fact-count loop (with the comment explaining WHY: without it, `find()` takes the
first match of each kind and a duplicated fact passes unnoticed) → `assert_obs_ids` → the
`fact`/`mac`/`ip`/`hostname` closure triple → value assertions → the instants vector.

`assert_obs_ids(&observations, prefix, expected_len)` is at `fixtures.rs:2290` and takes a length
since story 5.1 — a helper that asserted the ids of *whatever it was handed* let an empty slice pass,
which is the vacuity 5.1's review caught. Its `format!("{prefix}-0000-4000-8000-{:012}", n + 1)`
renders a DECIMAL sequence into a hexadecimal field (registered under story 5.1's review); harmless
below ten lines, and every stream here is 3 or 4.

Sibling byte-pins to read for tone and doc shape: `the_vrrp_stream_shares_one_virtual_mac_and_moves_its_uplink`
(`:2391`), `the_hostname_collision_stream_shares_one_name_across_two_boxes` (`:2753`),
`the_docker_veth_stream_replaces_its_veth_without_replacing_its_host` (`:2900`),
`the_hostname_absence_stream_encodes_empty_and_missing_and_never_null` (`:3046` — **the source of
AC3's absence idiom**).

### What this touches, and what it must not break

- **`crates/opencmdb-bin/src/fixtures.rs`** (UPDATE) — **728 code lines**, 3450 total. Everything
  this story adds lands in the trailing test module, i.e. AFTER the first `#[cfg(test)]` at `:729`,
  so **the file-size gate number must not move. Expect 728 and say why.**
  *Must be preserved:* `expected()` as the second independent oracle; `assert_obs_ids`'s length
  parameter; the existing dhcp-churn assertions (extend, never replace); `walk_replay_streams`'s
  fixed root; the exhaustive `match` with no `_` arm in `assert_record_is_synthetic`.
- **`crates/opencmdb-bin/src/fixture_connector.rs`** — **NOT expected to change.** Its
  `committed_stream_contexts()` table (`:1509-1527`) already lists all five streams; it states
  declared CONTEXT and deliberately asserts nothing about contents. Do not fold value pins into it —
  it answers a different question.
- **`crates/opencmdb-core/`** and **`xtask/`** — **NOT touched.** No new parser, no gate change.
- **Under `fixtures/`: NOTHING.** No artefact bytes, no `MANIFEST.toml`, and — unlike story 5.2 — not
  even a `README.md` is expected to move.
- **`_bmad-output/implementation-artifacts/deferred-work.md`** (UPDATE) — append-only.
- **`_bmad-output/implementation-artifacts/sprint-status.yaml`**, **`docs/project-context.md`**
  (UPDATE) — test count and the Epic 5 line.

### Inherited from stories 5.1 and 5.2 — read before writing a doc comment

1. **A helper that closes a vacuity can re-introduce it.** Ask of every new pin: what does it assert
   when handed nothing? Length first, values second.
2. **A check that its own commit falsifies is worse than no check.** 5.1 cited a `grep` its own diff
   broke; 5.2 replaced a false doc sentence with another one that the same commit falsified. When
   this story writes *"verified by …"*, **re-run the verification after the last edit.**
3. **An inventory in a doc comment has no guard behind it.** State what THIS test proves; let the
   register count.
4. **Name the test behind every claim.** The temptation here is *"the corpus is now pinned"*. What
   will hold is *"five families assert their authored values and their `obs_id` ↔ line binding; the
   trap files' `reason` prose is still not mechanically tied to the values it cites."*

### House rules that bind this story

- **Prove-to-red is not optional** (story 1.3). AC6 names five mutations, six observations, each
  aimed at a stream no other value test reads.
- **DRY, with deliberate redundancy protected.** These pins ARE the deliberate redundancy CLAUDE.md
  names — a second independent oracle over bytes `expected()` also describes. Do not collapse them
  into a loop over a table: a table-driven pin restates the corpus in one place and stops being
  independent of it. Five explicit tests, one per family.
- **File-size gate:** ≤ 2000 CODE lines, tests excluded, counted to the first `#[cfg(test)]`.
- **`DATABASE_URL` is usually unset locally** and the MariaDB-backed tests `return` early — a green
  suite says nothing about the database. Irrelevant here; do not cite it as evidence.
- **Known local flakiness (issue #38):** unexplained non-determinism; the "Synology Drive"
  explanation is **REFUTED by measurement — do not re-adopt it**. If a corpus test reds unexpectedly,
  re-run and check `git status` before diagnosing — especially in this story, where every mutation
  edits a committed artefact and a forgotten `git checkout` looks exactly like flakiness.

### Testing standards

Tests live inline in the trailing `#[cfg(test)] mod tests` (D56b, one per file). Test names are
sentences that say what they prove. Assertion messages name the offending LINE — with five streams
pinned, *"the MAC is wrong"* is not actionable unless it says which observation. Value comparisons go
through the parsed `Fact` (`assert_eq!(mac(0), Fact::Mac { addr: MacAddr([2,0,94,0,83,32]),
locally_administered: true })`) rather than string-matching the JSON, exactly as the dhcp-churn pin
compares `Fact::IpV4 { addr: Ipv4Addr::new(…) }`.

### Project Structure Notes

Paths follow the established layout with no variance: corpus at the workspace root in `fixtures/`
(D56); `FIXTURES_DIR` expressed exactly once (`fixtures.rs:48`) — take the path from
`fixtures_dir()`/`fixture_path()`, never re-write the string (`the_fixtures_path_is_expressed_once`
pins it, and 5.1's Task 3 is where a dev last nearly tripped it).

### References

- Story source: [Source: _bmad-output/planning-artifacts/epics.md#Story 5.2b] (epics.md:1361-1399);
  the insertion rationale, epics.md:1315 and :1363; build order, epics.md:1317.
- Register entries CLOSED by this story:
  [Source: _bmad-output/implementation-artifacts/deferred-work.md] — `## Deferred from: code review
  of story-4.13` (`:397`, bullet at `:399-409` and `:410-415`) and **`## Deferred from: story-5.1`** (`:516`, bullet
  at `:521`). The second heading carries no *"code review of"*, deliberately; the similarly-named
  `## Deferred from: code review of story-5.1` at `:556` is a different list this story does not
  close. The closure idiom is the file's own `✅ **CLOSED by story X.** ~~struck~~` shape.
- The idiom: `fixtures.rs:2305-2380` (dhcp-churn), `:2290` (`assert_obs_ids`), `:3046`
  (hostname-absence, the ABSENCE assertion).
- D21 (no unique index on `interface.mac_canon`) and D10 (the false merge is catastrophic and
  asymmetric): [Source: _bmad-output/planning-artifacts/architecture.md] — start at the Decision
  Index near the top, per F56.
- The two immediately previous stories, for the prove-to-red record shape and the review lessons:
  `5-1-corpus-pins-obs-id-binding.md` and `5-2-privacy-floor-reaches-the-bytes.md`.

### Git intelligence

Last five commits: `95d4d69` (5.2 bookkeeping, PR #45) · `4d2044b` (**story 5.2**, PR #44) ·
`e846836` (5.1 bookkeeping, #42) · `c84e14a` (**story 5.1**, #41) · `8c81386` (Epic 5 decomposition,
#40). `fixtures.rs` was last touched by 5.2, which rewrote the free-text scanner and hoisted
`walk_trap_files` — **none of that is in this story's path**: 5.2 worked on the privacy rule, this
one works on value pins, and they meet only in the file they share. Before 5.2, 5.1 hoisted
`walk_replay_streams` and gave `assert_obs_ids` its length parameter, which IS load-bearing here.
Every one of those commits went branch → PR → green CI → squash merge; hold the same. Do not quote a
running "N/N green" tally — say "green CI on every PR", which is reconstructible.

### Toolchain

No new dependency, no version to research. Rust 1.96+, edition 2024, `Cargo.lock` committed, every
build `--locked`. Never invent a version.

## Dev Agent Record

### Agent Model Used

`claude-opus-5[1m]` (Claude Opus 5, 1M context), via `bmad-dev-story`, 2026-07-28.

### Debug Log References

**Baseline before any edit:** `cargo test --workspace` → **130 + 86 + 42**, zero failures.
**After implementation:** **135 + 86 + 42** (+5 bin tests; the dhcp-churn pin was extended, not
added). `fixtures.rs`: 3450 → 4253 total lines, first `#[cfg(test)]` still at **line 729** ⇒
**728 code lines, unchanged** — every line added lands in the trailing test module, exactly as the
story predicted. The `file-size` gate's largest file is 884 (a different file); fixtures.rs is not
the offender and did not move.

**The seven recorded mutations (AC6 asked for six).** Each was the SOLE failure, each named the
offending line, and after each: `git checkout <artefact>` then `git status --porcelain fixtures/`
verified EMPTY before the next one. `cargo xtask ci` was never run with a mutation in place.

| # | Mutation | Result |
|---|---|---|
| 1 | `randomized-mac.jsonl` N3 MAC `[…,83,33]` → `[…,83,32]` | RED, sole — *"N3 wears 02:00:5e:00:53:21 — the ONE octet the must-not-merge pole rests on"*, `left: MacAddr([2,0,94,0,83,32])` |
| 2 | `multi-nic.jsonl` M2 `swport-2` → `swport-1` (the port collapse AC2 names) | RED, sole — *"M2 hangs off the SAME switch on a DIFFERENT port — the uplink that agrees"* |
| 3 | `shared-hardware-vm.jsonl` W3's `peer_port` → `swport-9`, **line 3 only** | RED, sole — *"observation 2 hangs off the shared hypervisor uplink, by construction"* |
| 4 | `shared-hardware-vm.jsonl` W4's `Uplink` **REPLACED** by `{"Hostname":{"name":"doc-vm-delta","source":"Dhcp"}}` | RED, sole — the ABSENCE assertion, `panicked at fixtures.rs:3660`. Zero other tests red, confirming the story's three measured constraints (replace-not-add, `doc-` prefix, exact `Dhcp` spelling) |
| 5 | `cloned-mac.jsonl` K2 MAC last octet `112` → `113` | RED, sole — *"observation 1 wears 02:00:5e:00:53:70 — the one cloned MAC, byte-identical"* |
| 6 | **`cloned-mac.toml`: the two poles' `observations` vectors EXCHANGED** (no stream byte touched) | RED, sole — *"trap `cloned-mac-must-not-merge` judges exactly these observations, in this order"*. **This is the AC4b hole; before this story it left the whole suite green while the corpus DEMANDED the false merge.** |
| 7 | `example-traps.jsonl` E3 final octet `17` → `16` | RED, sole — *"E3 differs in the FINAL octet, exactly as the must-not-merge reason claims"* |

**The dhcp-churn TWO-SIDED observation (AC5/AC6), the justification for closing 4.13.** With
`doc-host-hotel` → `doc-host-india` in the committed stream:
- **Half A** — the PRE-STORY tree (`git stash push crates/opencmdb-bin/src/fixtures.rs`):
  **130 + 86 + 42, ZERO failures.** The existing relational `assert_ne!(hostname(2), hostname(0))`
  stayed green, because golf ≠ india is still true. The relational pin never noticed.
- **Half B** — `git stash pop`, same mutated stream: **RED**, naming N3 —
  *"N3 answers to doc-host-hotel, the name the must-not-merge reason cites"*,
  `left: Hostname { name: "doc-host-india" }`.

That green-then-red pair is what demonstrates the value pin adds something the relational one did
not. Artefact restored; `git status fixtures/` empty.

**Final gate, run last, with `fixtures/` verified clean first:** `cargo fmt --all` ✅ ·
`cargo clippy --workspace --all-targets -- -D warnings` ✅ (zero warnings) · `cargo test --workspace`
✅ 135 + 86 + 42 · `cargo xtask ci` ✅ frontier / ddl-collation / vocabulary / fixtures (25 match
sha256) / file-size (20 files, largest 884), with `ℹ views-hash STALE` — expected, exits 0, and
`architecture-views.md` was NOT regenerated.

### Completion Notes List

**What holds now, in the narrow true sentence** (Dev Notes lesson 4): *five families and the
example stream assert their authored values and their `obs_id` ↔ line binding, and every committed
trap's `observations` vector and `Expectation` are pinned; the trap files' `reason` prose is still
not mechanically tied to the values it cites.* That last clause is registered, not glossed.

1. **The four unpinned families each got a byte-pin test**, in the `dhcp-churn` idiom and in that
   order: length → exact per-line fact count → `assert_obs_ids(prefix, len)` → the
   `fact`/`mac`/`ip`/`hostname`/`uplink` closures → authored values → the instants vector. Five
   explicit tests, deliberately NOT a table-driven loop: a table restates the corpus in one place
   and stops being independent of it, which is the deliberate redundancy CLAUDE.md protects.
2. **Every MAC is pinned by VALUE on each line, never pairwise** — `randomized-mac`'s three lines,
   `cloned-mac`'s three identical ones, `shared-hardware-vm`'s shared `Uplink` on all four. This is
   what reaches the one-octet edit a relational pin sails through.
3. **Three of the four families pin an instants vector that is NOT strictly increasing**, as
   measured. `dhcp-churn`'s strict-increase assertion was deliberately not copied — two NICs seen
   in one sweep should share an instant, and asserting an ordering the corpus does not have would
   red on the committed bytes.
4. **AC4b — `assert_trap_binds`.** Pins, per trap id, the exact `observations` vector *in order*
   and the whole `Expectation` (which covers column, rule, and the abstain pole's cause in one
   comparison). Each call site also pins the file's trap COUNT first, so an added trap cannot slip
   past — length first, values second, the vacuity lesson from 5.1. All **eleven** committed traps
   are bound: 9 from the story's table + `dhcp-churn`'s 2, plus `example.toml`'s 3.
   Order is pinned rather than membership, and the doc comment says why.
5. **AC7's open question was DECIDED, and the branch taken is the pin.** `example-traps.jsonl` —
   surfaced by validation as a SIXTH unpinned stream — is now pinned by
   `the_example_trap_stream_carries_the_values_its_reasons_cite` rather than registered with an
   owner. Closing the byte-fidelity theme while leaving a known instance of exactly that theme open
   would have made the closure claim narrower than it reads. Its doc comment states why it is not a
   duplicate of `the_committed_trap_file_reads_and_cross_checks` (that one asserts the FORMAT is
   exercised; this one asserts the VALUES) — different questions over one file, which is deliberate
   redundancy, not accidental duplication.
6. **One test-internal ORDERING decision, reasoned in the code.** W4's hostname-absence assertion
   sits AHEAD of the shared-uplink loop. The loop's one-of-each `find()` PANICS on a line whose
   `Uplink` was replaced, so with the loop first, AC6's prescribed mutation would have red the
   uplink pin and never reached the absence pin — the recorded red would have named the wrong
   guard. Ordered this way the mutation reds the absence assertion and nothing else, which is what
   proves it has teeth. The test comment states this.
7. **Two corrections carried, neither discovered mid-task.** The epic's *"both `reason` strings
   cite"* distribution is false (`epics.md:1391`) and **the same false sentence is in the register
   bullet** — so the counted distribution (MAC in one reason, hotel in one, golf in both; the UNION
   cites all three, neither reason does) travels IN the closure note, because append-and-strike
   would otherwise leave the wrong sentence standing.
8. **The register claim was verified by re-reading AFTER the last edit, and the first draft of it
   was falsified by this story's own commit.** The narrow claim initially read *"every entry owned
   by 'whoever hardens corpus byte-fidelity' is now closed"* — while the same edit opened a NEW
   entry with owner *"whoever next hardens corpus byte-fidelity"*. Corrected before commit: the
   claim is now scoped to entries owned **when this story opened**, and names both things that stay
   open (`unifi-clients.expected.jsonl`, owner Epic 11 / issue #34; and the new item). This is the
   exact failure 5.1's and 5.2's reviews each caught once.

**No artefact byte moved.** `git status --porcelain fixtures/` is empty; the `fixtures` gate reports
25 artefacts matching their recorded sha256. No `MANIFEST.toml` re-hash, no `README.md` change.
`opencmdb-core/`, `xtask/` and `fixture_connector.rs` were not touched.

**What was NOT done, deliberately:** `architecture-views.md` was not regenerated (`ℹ STALE` is
expected and exits 0; it is a milestone task, not a story task), and `epics.md:1397`'s stale
closing promise was not edited — it is outside this story's touch list, and the register now
carries the correction that supersedes it.

### File List

- `crates/opencmdb-bin/src/fixtures.rs` (UPDATE) — five new byte-pin tests, the `dhcp-churn` pin
  extended with three value assertions and its trap binding, and three new test helpers
  (`assert_trap_binds`, `merge`, `not_merge`). All inside the trailing `#[cfg(test)] mod tests`;
  code-line count unchanged at 728.
- `_bmad-output/implementation-artifacts/deferred-work.md` (UPDATE) — three bullets closed by
  append-and-strike (both story-4.13 bullets, the story-5.1 four-families bullet); new
  `## Deferred from: story-5.2b` section with one open item, the `example-traps.jsonl` decision on
  the record, and the narrow closing claim.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (UPDATE) — `5-2b` → `review`,
  `last_updated`, implementation summary.
- `docs/project-context.md` (UPDATE) — test count 258 → 263 and the Epic 5 row.
- `_bmad-output/implementation-artifacts/5-2b-four-families-byte-pins.md` (UPDATE) — this record.

**Unchanged, and verified so:** everything under `fixtures/`.

## Change Log

| Date | Change |
|---|---|
| 2026-07-28 | **Implemented — status `review`.** 258 → 263 tests (135 + 86 + 42); all four local gates green; **no byte moved under `fixtures/`**, verified empty before the gate ran. Five new byte-pin tests (`randomized-mac`, `multi-nic`, `shared-hardware-vm`, `cloned-mac`, `example-traps`) plus the `dhcp-churn` pin EXTENDED with its three cited values. `fixtures.rs` code-line count **unchanged at 728** — every line landed after the first `#[cfg(test)]`. **AC4b's `assert_trap_binds` binds all ELEVEN committed traps** (per trap id: the exact `observations` vector in order + the whole `Expectation`, with the file's trap count pinned first so an added trap cannot slip past). **Seven mutations recorded, AC6 asked for six** — each the SOLE red, each naming its line, `git checkout` + clean `git status fixtures/` between every one. The measured AC4b hole is closed: exchanging the two `observations` vectors in `cloned-mac.toml` now reds, where before it left the whole suite green while the corpus DEMANDED the false merge. The dhcp-churn observation is two-sided as required — the PRE-STORY tree reported **130 + 86 + 42 zero failures** under a renamed `doc-host-hotel` (the relational `assert_ne!` never noticed) while the extended test reds naming N3. **AC7's open question was DECIDED: `example-traps.jsonl` was PINNED, not registered.** One test-internal ordering change was necessary and is reasoned in the code — W4's absence assertion moved ahead of the shared-uplink loop, because that loop's `find()` panics on a line whose `Uplink` was replaced, so with it first AC6's mutation would have red the wrong guard. Register: both story-4.13 bullets and the story-5.1 four-families bullet closed by append-and-strike, with the counted `reason` distribution carried INTO the closure note (the false sentence was in the register, not only in `epics.md:1391`); new `## Deferred from: story-5.2b` opens ONE item (a trap's `reason` prose is still not mechanically tied to the values it cites). **The narrow claim's first draft was falsified by this story's own commit** — it said every entry owned by *"whoever hardens corpus byte-fidelity"* is closed while the same edit opened a new one with that owner; corrected before commit by scoping it to entries owned WHEN THIS STORY OPENED and naming both items that stay open. `architecture-views.md` NOT regenerated. |
| 2026-07-28 | **Validated by two fresh-context agents** (fact-check + gap-hunt), per the Epic 4 retrospective. **3 HIGH, 6 MED, 3 LOW — all applied.** The load-bearing one **added AC4b** and it is measured, not argued: every pin the story asked for lives on the `.jsonl`, but the inversion is available entirely from the `.toml`. Exchanging the two `observations` vectors in `cloned-mac.toml` — three characters, no stream byte touched — makes the corpus DEMAND the false merge (echo+foxtrot under `must-merge`/`l1-exact-mac`), and `cargo test --workspace` was run in that state and reported **130 + 86 + 42, zero failures**. AC4 had claimed `assert_obs_ids` closed that hole; it closes only the stream-side half. *(Reproduced independently by the reviewer before the AC was written; file restored, md5 verified, `git status fixtures/` empty.)* Second HIGH: the story's closing promise — inherited from `epics.md:1397` — is **stale**, because 5.1's own review registered `unifi-clients.expected.jsonl` as having no round-trip pin one day after the epic was written; and validation surfaced a SIXTH unpinned stream, `example-traps.jsonl`, whose `example.toml:26` reason cites `02:00:5e:00:53:10` — verbatim the 4.13 shape. Both are now named, with the narrow claim the story may make. Third HIGH: AC3 had offered a choice between the exact fact-count vector and `>= 3`; the second branch is **measured vacuous** (a second contradicting `Uplink` on W4 reds nothing under it) and was withdrawn. Also applied: the story-4.13 register section has TWO bullets and closing one leaves a "STILL OPEN" bullet standing; the false `reason`-distribution sentence is in the register too, not only in the epic; the W4 mutation must REPLACE not ADD (adding reds the count loop first) and needs a `doc-` prefix and the exact `HostnameSource` spelling or it reds two tests; `cargo xtask ci` DOES consult `MANIFEST.toml` and must never be run mid-mutation; every MAC is now given in decimal bytes with the `:40` = `64` trap named; and STOP now has a procedure. Three line ranges corrected. |
| 2026-07-28 | Story contexted. All five streams read verbatim off the committed corpus at `95d4d69` and tabulated, together with the `obs_id`s each of the nine traps judges. **Three traps measured for the dev:** `shared-hardware-vm`'s fact count is NOT uniform (4, 4, 4, 3 — W4 is the abstain pole and has no `Hostname`), instants are NOT strictly increasing in three of the four families (multi-nic, shared-hardware-vm and cloned-mac each have a shared instant, because two NICs seen in one sweep should share one), and dhcp-churn's byte-pin asserts strict increase only because that family's churn lives in time alone. **One correction to the epic:** epics.md:1391 says "both `reason` strings cite" all three dhcp-churn values; counted, the MAC appears in one reason, `doc-host-hotel` in one, `doc-host-golf` in both — the union cites all three, neither reason does. Conclusion unchanged. |
