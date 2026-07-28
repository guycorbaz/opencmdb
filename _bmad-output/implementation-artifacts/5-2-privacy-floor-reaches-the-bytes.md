# Story 5.2: The privacy floor reaches the bytes it always claimed to cover

Status: review

<!-- Validation is MANDATORY here (Guy's decision, Epic 4 retrospective 2026-07-26): two
     fresh-context agents (fact-check + gap-hunt) before dev-story. The template banner saying
     validation is optional does not apply to this project.
     DONE 2026-07-28 — 6 HIGH, 7 MED, 10 LOW, all applied. See the Change Log. -->

## Story

As the owner of the corpus,
I want trap-file text and the `Observation.raw` field routed through the synthetic-text scanner, and
the scanner's three named evasions closed,
so that the privacy rule covers the COMMITTED BYTES rather than only the fields a decision happens
to read.

**This is the second of Epic 5's three inherited-debt stories** (epics.md:1313, Guy's decision at
Epic 4's retrospective). It closes the privacy theme the way 5.1 closed the byte-fidelity theme:
before Epic 5 bumps the corpus, not after. D19 makes a real capture in a public repository
disqualifying — and today the rule that enforces it cannot see two of the three places an author
types free text.

**No ARTEFACT bytes change in this story.** Every strengthening below was modelled against the
committed corpus before the story was written, and **all of them are green** (see Dev Notes → "What
was measured"). If a task appears to require re-authoring a committed artefact or re-hashing
`MANIFEST.toml`, STOP: that is a finding, not a task. The two `README.md` files under `fixtures/`
are the deliberate exception — orphan-exempt, unlisted, re-hashing nothing, and made false by this
very change (AC6).

## Acceptance Criteria

1. **AC1 — trap-file text is scanned, comments included, before TOML parsing discards them.**
   **Given** `assert_text_is_synthetic` (`crates/opencmdb-bin/src/fixtures.rs:1060`), whose only
   call site over the REPLAY tree is the `Record::Failure` arm (`:1044`), so the headers, comments
   and `reason` strings of `fixtures/scenario/traps/*.toml` reach it never (registered under story
   4.14's review — 4.14's own no-octets-in-reasons rule is held by review, not by a gate) **when**
   the trap corpus is walked **then** each `.toml` file's RAW TEXT — read with
   `std::fs::read_to_string`, before `read_traps`/`toml::from_str` — goes through
   `assert_text_is_synthetic`, and the failure names the file.

   **The shape already exists — reuse it, do not invent it.** `assert_text_is_synthetic` has a
   SECOND corpus call site the register never caught up with: `fixtures.rs:2793`, inside
   `the_wire_spec_encodes_the_measured_field_behaviours` (story 4.18), which does exactly this —
   `read_to_string` on a committed file, then the scanner on the raw bytes, because
   `scenario/wire/` sits outside every corpus walk. Two consequences, both binding:
   - AC1's mechanism is that idiom applied to a tree, not a new invention;
   - the 4.14 wiring test's doc (`fixtures.rs:2272-2275`) still says *"no committed trap text ever
     reaches `assert_text_is_synthetic` (its only call site is the `Record::Failure` walk)"*, and
     `deferred-work.md:419-420` says the same. Both were true when written (4.18 landed after) and
     are false today. **Fix the doc in this story** — "a doc comment must be TRUE" is a review
     criterion, and this story is where the sentence is being read.

   **And this walk is NOT vacuous, which the story states as a measurement rather than a hope:**
   the committed trap text already carries **4 distinct MACs** (`00:00:5e:00:01:0a`,
   `02:00:5e:00:53:10`, `02:00:5e:00:53:20`, `02:00:5e:00:53:78`) and **3 distinct IPs**
   (`192.0.2.1`, `192.0.2.120`, `192.0.2.121`), spread over four files — `dhcp-churn.toml` (1 MAC,
   2 IPs), `example.toml` (1 MAC), `randomized-mac.toml` (1 MAC), `vrrp-virtual-mac.toml` (1 MAC,
   1 IP). The other six carry none. All seven values pass.

   **That measurement becomes an ASSERTION, not a doc sentence.** The walk is non-vacuous only
   because four trap `reason` strings happen to quote addresses; a re-authoring that drops them —
   and story 5.2b touches these very families — would leave the scan green, empty and undetected.
   `walk_trap_files`'s own `checked > 0` does not help: it counts FILES, not addresses. So the new
   test counts the address-shaped tokens the scan actually inspected and asserts
   `macs_seen >= 4 && ips_seen >= 3`, with a message naming the counts. State the uneven
   distribution (six of the ten files contribute none) in the doc **beside** that assertion —
   an inventory in a doc comment has no guard behind it, which is the register's own lesson at
   `walk_replay_streams:715-718`.

   **And** the `README.md` under `scenario/traps/` is exempt at any depth, exactly as every other
   walk in this corpus exempts it.

2. **AC2 — `Observation.raw` is scanned by the same rule, and its VACUITY today is written down.**
   **Given** `raw: Option<String>` (`crates/opencmdb-core/src/observation/mod.rs:246`) — the FIELD
   carries no `///` of its own (it is part of core's outstanding doc sweep); the description lives
   on the `Observation` struct doc (`:234-237`), *"opaque provenance … that NO decision ever reads
   (D19)"*, and the phrase *"never read by a decision"* is the JSONL payload VALUE in
   `minimal.jsonl:3`, not a doc — which the corpus walk `the_corpus_carries_no_real_network_data`
   (`fixtures.rs:1033`) never inspects — its `Record::Observation` arm (`:1038-1040`) passes only
   `facts` to `assert_facts_are_synthetic` (registered under story 4.16's review) **when** the walk
   runs **then** a `Some(raw)` payload goes through `assert_text_is_synthetic` in that same arm.

   **And the doc says plainly what this proves today: almost nothing.** Measured across all 13
   replay streams and the wire artefact, **exactly ONE committed observation carries a non-null
   `raw`** — `minimal.jsonl` line 3, `{"provenance":"never read by a decision"}` — and it contains
   no address, so the new call site is **vacuous on the committed corpus**. It is worth adding
   anyway (the charter is the bytes, and `raw` is the obvious landing place for a pasted capture),
   but AC4's prove-to-red for it therefore CANNOT come from the corpus. Write *"no committed `raw`
   currently exercises this"*; do not write *"`raw` is now covered corpus-wide"*.

   **And because it is vacuous, the call site does not defend itself — so this AC ships a
   PERMANENT guard alongside it.** A mutation record is not a guard: with no committed `raw` to
   break, deleting the new line after merge reds nothing. This is exactly why
   `the_text_scanner_admits_the_vrrp_range` (`:2277`) exists, and its doc says so. Ship the same
   remedy here: a test that builds an `Observation` whose `raw` names a non-documentation address
   and drives the same per-record logic the walk arm calls. `opencmdb-core` is still not touched —
   the field's missing `///` is core's doc sweep, not this story's.

3. **AC3 — the three named evasions close, and the scanner's own doc stops calling them unnamed.**
   **Given** `assert_text_is_synthetic`'s tokenizer
   (`text.split(|c: char| !(c.is_ascii_hexdigit() || c == '.' || c == ':'))`), whose doc admits it
   is *"a floor, not a proof"* (`:1057-1058`), and `is_synthetic_mac` (`:1089`), whose doc states
   the separate rule that *"the list is CLOSED and 5-octet exact"* (`:1077-1091`) — while the
   register names the three specific holes (story 4.14's review) **when** they are closed **then**
   all six of these hold, each with its own assertion:

   | # | Input shape | Required behaviour |
   |---|---|---|
   | a | `…seen at 198.18.0.1.` (trailing `.`) | the IP is SEEN → red |
   | b | `…answers as 00:11:22:33:44:55:` (trailing `:`) | the MAC is SEEN → red |
   | c | `…reachable at 198.18.0.1:8080…` (INTERIOR punctuation) | the IP is SEEN → red |
   | d | `…the port learned 00-11-22-33-44-55…` (dash form) | the MAC is SEEN → red |
   | e | `…33:33:ff:00:60:0a…` (IPv6-multicast shape) | REFUSED → red |
   | f | `…01:00:5e:00:00:0a…` (IPv4-multicast shape) | REFUSED → red, and now for a STATED reason |

   **Row (c) is the one an edge-trim does not reach — and the obvious fix is MEASURED WRONG.**
   Trimming `.`/`:`/`-` off a token's ends closes (a) and (b), reaches nothing of (d) (`-` is a
   SEPARATOR today, so the dash form has already shattered into six two-character tokens before any
   trim runs), and leaves `198.18.0.1:8080` unparseable — the token keeps its interior `:`.

   The obvious next move — enumerate every substring of a token and feed each to the parsers —
   **reds the committed corpus.** `Ipv4Addr::from_str` rejects only LEADING ZEROS, so `92.0.2.120`
   and `2.0.2.120` both parse, and every documentation IP in the corpus contains two
   non-documentation IPs inside it. Measured 2026-07-28 at `e846836`: 8 false positives in
   `dhcp-churn.toml`, 2 in `vrrp-virtual-mac.toml`, and **10 in `scenario/wire/unifi-clients.json`,
   which the scanner ALREADY reads today** (`fixtures.rs:2793`). A dev who takes that route meets a
   red corpus mid-task, and the tempting repair — loosening `assert_documentation_ip` — is the
   worst available move. Do not take it.

   **The required shape is boundary-anchored longest-match.** Normalise `-` → `:`; split into
   maximal runs of `[0-9a-fA-F.:]`; inside a run, a candidate may start only at the run start or
   immediately after a `.` or `:`; at each such start take the LONGEST prefix that parses as
   `Ipv4Addr` or `MacAddr`, check it, and resume after it. Measured on the committed corpus this
   yields exactly the same 4 MACs / 3 IPs in trap text and leaves the wire body's 4 IPs / 4 MACs
   unchanged — **zero new reds** — while seeing all of rows (a)–(d). Its residual limit must be
   written into the scanner's doc rather than quietly dropped: an address glued to a hex-letter
   prefix (`ab198.18.0.1`) is still invisible.

   Row (c) is an acceptance criterion, not an optional extra. If you take a different shape than
   the one above, re-run it against `the_wire_spec_encodes_the_measured_field_behaviours` before
   claiming anything — that test scans the largest body of text the scanner sees.

   **Row (d) must NOT be closed in `opencmdb-core`.** `MacAddr::from_str`
   (`observation/mod.rs:106-123`) is colon-only and STAYS colon-only: widening a domain parser for
   a test's convenience is a frontier violation (D47) and would change what the shipped connectors
   accept off the wire. Normalise `-` → `:` inside the scanner, in `opencmdb-bin`'s test module.

   **Rows (e)/(f) — the rule that makes them consistent.** Today `is_synthetic_mac` reads
   *locally administered OR the VRRP block*. `33:33:…` has its U/L bit set, so it is ADMITTED —
   and an IPv6 solicited-node multicast MAC embeds the low four bytes of a real IPv6 address, i.e.
   real interface-identifier bytes. `01:00:5e:…` has U/L clear, so it is refused — for an unrelated
   reason. The consistent rule is that a MULTICAST address (I/G bit, `addr.0[0] & 1`) is not an
   interface address at all:

   ```rust
   fn is_synthetic_mac(addr: MacAddr) -> bool {
       (addr.is_locally_administered() && addr.0[0] & 1 == 0) || addr.0[..5] == [0, 0, 94, 0, 1]
   }
   ```

   **Measured against every committed MAC — 39 distinct addresses across `Fact::Mac` and
   `Uplink::peer_mac` in all 14 committed `.jsonl` files: not one has the I/G bit set, so ZERO
   would red.** The VRRP block (`00:00:5e:00:01:xx`, I/G clear) is unaffected. Record that
   measurement in the story; do not re-derive it under implementation pressure.

   **The MESSAGE must change with the rule — and only in the multicast branch.**
   `assert_text_is_synthetic:1069` and `assert_synthetic_mac:1161` both say the address is
   *"neither locally administered nor in the IANA VRRP virtual-router range"*. After the tightening
   that sentence is FALSE for `33:33:…`, which IS locally administered and is now refused for being
   multicast. Emit a distinct sentence when the I/G bit is set (*"is a MULTICAST address and names
   no interface"*) and keep the existing wording verbatim for the non-multicast case, so
   `the_text_scanner_still_refuses_a_mac_outside_the_block`'s `should_panic(expected = …)` at
   `:2287` still matches and is not quietly re-pointed.

   **And rows (e)/(f) are pinned where the RULE lives, not only through the tokenizer.**
   `is_synthetic_mac`'s closed list is pinned by `the_vrrp_allowance_is_five_octets_exact`
   (`:2243`) — six assertions on raw `MacAddr` byte tuples. Add a locally-administered-but-multicast
   row there (`MacAddr([0x33, 0x33, 0xff, 0, 0x60, 0x0a])`): it is the only place the new rule's two
   conjuncts can be pinned independently of the tokenizer. Note also that row (f)
   (`01:00:5e:00:00:0a`) is **already refused today** (U/L clear) — its prove-to-red is therefore
   NOT a "the hole existed" observation; only the stated reason changes, so its `expected` substring
   must name the NEW multicast sentence or the test proves nothing.

   **And** the register's own words become the doc: `is_synthetic_mac` and
   `assert_text_is_synthetic` say which evasions are CLOSED and which floor remains. Four things
   remain, and all four are named rather than elided: a hostname in prose still cannot be recognised
   mechanically; `Fact::OuiVendor { vendor }` and `Fact::Uplink { peer_port }` are free author-typed
   text that `assert_facts_are_synthetic` discards by construction (`:1133` and `:1122`); and every
   `README.md` is exempt at any depth by AC1's own rule — `fixtures/scenario/traps/README.md` is
   6 KB of prose and the largest un-scanned text in the corpus. None of the four is closed here and
   none may be claimed; the last three go under `## Deferred from: story-5.2` (AC7) with an owner.

4. **AC4 — every strengthened guard is proven to red before it passes, and the mutation is
   recorded** (house rule, story 1.3). One recorded observation per row of AC3's table, plus:
   - **AC1:** put a vendor-shaped MAC (`00:11:22:33:44:55`) into a trap file's HEADER COMMENT, run
     the new walk, observe the named red, `git checkout` the file. A comment, not a `reason` — the
     AC's whole point is that the scan happens before TOML parsing throws comments away.
     (`cargo test` does not consult `MANIFEST.toml`, so no re-hash is needed, and none must be
     committed.)
   - **AC2:** the corpus cannot supply this one (see AC2), **and a `scratch_dir` tree cannot be
     walked into it**: `walk_replay_streams` (`:730`) takes no root parameter — it hardcodes
     `fixture_path("scenario/replay")`, and that fixed root is load-bearing for 5.1's callers. **Do
     not parameterise it.** Take the mutation on a hand-built `Record::Observation` whose `raw`
     names `198.18.0.1`, driven through the same per-record logic the walk arm calls, and ship it
     as the PERMANENT guard AC2 requires (name it for what it proves, e.g.
     `an_observations_raw_payload_is_scanned`). If wiring that needs the arm's body extracted into
     a helper (`fn assert_record_is_synthetic(record: &Record, path: &Path)`), do that and say so
     in the record. Say also that the mutation is record-side BECAUSE the corpus has no `raw` to
     break.
   - **AC3:** rows (a)–(f), each as an in-memory string against `Path::new("in-memory")` — the
     idiom `the_text_scanner_admits_the_vrrp_range` (`:2276`) and
     `the_text_scanner_still_refuses_a_mac_outside_the_block` (`:2286`) already use. Prefer
     `#[should_panic(expected = "…")]` with a substring, so a pass-for-the-wrong-panic is
     impossible (4.14's idiom). **Each `expected` names the EXACT address the row asserts**
     (`"198.18.0.1 is not in an RFC 5737"`, `"00:11:22:33:44:55"`), never the generic tail of the
     message: a scanner that finds `98.18.0.1` instead of `198.18.0.1` also panics, and a loose
     substring would credit that bug with a pass — which is precisely how AC3's measured-wrong
     substring scan would slip past row (c) and only surface on `cargo test --workspace`.
   - Rows (a)–(d) must be observed **GREEN before the fix** as well, or the record cannot claim the
     hole existed. That is the two-sided observation story 5.1's mutation 12 established for a
     guard whose "red" is a counterfactual.

5. **AC5 — the trap walk stops descending into dot-directories, and the story does not author a
   third copy of it.** *(This AC has no counterpart in `epics.md`'s Story 5.2 and is a deliberate
   addition, on the same grounds 5.1 fixed the replay side inside its own story: the class is one
   line of code, and leaving half of it open across a story boundary is how the register
   accumulates. Say so — an Acceptance Auditor traces story ACs to epic ACs.)*
   **Given** that story 5.1 closed exactly this class for `scenario/replay/` and **only** there,
   while `fixtures/scenario/traps/.claude/.cc-writes` EXISTS in the working tree today **when** any
   tool writes a file under it **then** the corpus is accused of a defect it does not have.
   **This is measured, not argued:** a single `probe.txt` written there reds
   `every_trap_file_in_the_corpus_is_valid` with *"only .toml trap files belong under
   scenario/traps/"* **and SIX `trap_gate` tests** — the harness walk `discover_trap_files`
   (`trap_gate.rs:302`) has no dot-entry exemption either, and it returns `Err` before
   `score_corpus` ever validates answers, so even
   `an_answer_for_an_unknown_trap_is_refused` (which expects an error, just a different one) falls
   to its `other => panic!` arm. **Stated precisely, because the story must not overclaim:**
   `trap_gate` is `#![allow(dead_code)]` and wired into no runtime path (its own module doc), so
   these are six TESTS in the future gate's harness, not a red `cargo xtask ci`. **`xtask`'s own
   corpus walk already skips dot-entries** (`xtask/src/main.rs:626-629`, decided 2026-07-21) — the
   corpus lock and the orphan check need no change and stay consistent with both post-story walks.
   Both walks skip dot-entries, file or directory, with the cost
   named as 5.1 named it (a `.hidden.toml` would no longer be seen; acceptable because the corpus
   never hides an artefact and `MANIFEST.toml` lists every one by its visible name).

   **And** AC1's new scan REUSES a walk rather than adding one. There are already two walks over
   `scenario/traps/` — the production `discover_trap_files` and the inline test
   `every_trap_file_in_the_corpus_is_valid` (`fixtures.rs:1625`). A third is accidental duplication
   and the DRY rule forbids it. The recommendation is to hoist the test walk beside
   `walk_replay_streams` as `#[cfg(test)] pub(crate) fn walk_trap_files`, exactly as 5.1 hoisted
   the replay walk, and to leave `discover_trap_files` in place as the production path (its doc at
   `trap_gate.rs:291-301` already argues why the two are separate, and its paragraph at `:298-301`
   names both other walks explicitly — **update that doc if the shape changes**). If you take a
   different shape, say why in the record.

   **The hoist inherits three OPEN register items, and each must be disposed of explicitly.**
   `deferred-work.md#Deferred from: code review of story-5.1` records them against
   `walk_replay_streams` with the rationale that *the hoist made one function the shared definition
   across two test modules, so each item's blast radius grew even though its code did not*. The
   trap walk (`fixtures.rs:1625-1675`) has all three, and this hoist grows them the same way:
   - **(i)** the ROOT is never symlink-checked (`fixture_path("scenario/traps")` goes straight onto
     the stack, `:1628`), so a doc claiming "refuses symlinks" would not cover `scenario/traps/`
     itself;
   - **(ii)** there is no `is_file()` check, so a FIFO named `x.toml` makes `read_to_string` block
     and the suite HANG rather than fail — the register calls this *"the one failure mode with no
     diagnostic at all"*, and the local precedent is one condition at `fixtures.rs:1872`;
   - **(iii)** the walk yields unsorted `read_dir` order while its sibling sorts
     (`trap_gate.rs:355`), so with two broken files WHICH one panics varies per run.

   Close **(ii)** and **(iii)** in the hoist — both are one line, and (iii) is what makes the two
   trap walks actually agree, which is the whole point of AC5 — and either close (i) or record all
   of it under `## Deferred from: story-5.2` naming the trap walk. ⚠️ (iii) is a thing to RULE OUT
   for issue #38, **never** to record as its cause.

6. **AC6 — the local gate is green and every MANIFEST-listed artefact is byte-identical.**
   `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, and `cargo xtask ci` all pass, and `git status` shows **no change to
   any MANIFEST-listed artefact under `fixtures/`**. The `views-hash STALE` line from `xtask ci` is
   expected and exits 0 — **do not regenerate `architecture-views.md` in this story** (milestone
   task, project-context.md).

   **Two `README.md` files under `fixtures/` MUST change, and that is not a corpus edit.** READMEs
   are orphan-exempt by exact name (`xtask/src/main.rs:735`) and appear in none of `MANIFEST.toml`'s
   25 entries, so editing them re-hashes nothing. They must change because this story makes their
   text false, and docs-current-before-push (CLAUDE.md) puts that in this story rather than a
   follow-up:
   - `fixtures/README.md:44-51` states that the privacy check covers *"every fact of every
     observation, and any IPv4 or MAC literal appearing in a control record's free text"*, and that
     it *"does not cover, and knowingly: an observation's opaque `raw` payload"* — **both sentences
     become false here**;
   - `fixtures/README.md:60` and `fixtures/scenario/traps/README.md:96-101` state the MAC rule as
     "locally-administered" and the trap-text rule as review-enforced — the multicast tightening
     narrows the first, AC1 converts the second to test-enforced.

   Confirm with `git diff --stat fixtures/` that ONLY `README.md` files moved and `MANIFEST.toml`
   did not.

7. **AC7 — the register is updated by APPENDING, never by rewriting.** The two `deferred-work.md`
   entries this story closes are marked closed in place, in the file's established idiom
   (`✅ **CLOSED by story 5.2.** ~~old text struck~~ …`), keeping the original readable:
   - `## Deferred from: code review of story-4.14` — BOTH bullets ("Trap-file text is scanned by no
     privacy rule", AC1; "The free-text scanner is a floor with named evasions", AC3);
   - `## Deferred from: code review of story-4.16` — the `Observation.raw` bullet (AC2).

   **And** the `## Deferred from: code review of story-5.1` section's dot-entry theme is settled
   rather than left half-done: 5.1's section does not carry a dot-entry bullet (it fixed the replay
   side inside the story), so AC5's trap-side closure is recorded as a one-line note under this
   story's own heading, naming that 5.1 closed the class on one tree only.

   **And** the story-4.14 bullet's own wording is corrected as it is closed, not struck silently:
   it says *"its only call site is the `Record::Failure` walk"* (`deferred-work.md:419-420`), which
   story 4.18 falsified when it added `fixtures.rs:2793`. A register that closes an item on a false
   premise is the *"check that its own commit falsifies"* failure 5.1's review named.

   **And** anything this story SURFACES and does not fix gets a `## Deferred from: story-5.2`
   section with a named owner — not a GitHub issue. The register is the established home for
   review-surfaced corpus debt; an issue is reserved for scope that MOVES between epics (the 4.19b
   precedent, #34). At minimum this story surfaces: the three unscanned surfaces named in AC3
   (`OuiVendor.vendor`, `Uplink.peer_port`, every `README.md`), and whichever of AC5's three
   inherited walk items is not closed in the hoist.

## Tasks / Subtasks

- [x] **Task 1 — close the three evasions in the scanner (AC3, AC4)**
  - [x] Observe rows (a)–(d) **GREEN** first, with the scanner as it ships today. Record it: this is
        what makes "the hole existed" a measurement rather than a claim.
  - [x] Rewrite the tokenizer as **boundary-anchored longest-match** (AC3) — including the INTERIOR
        case `198.18.0.1:8080`, which an edge-trim does not reach. **Do NOT enumerate all
        substrings**: it is measured to red `dhcp-churn.toml`, `vrrp-virtual-mac.toml` and the wire
        test on committed bytes.
  - [x] Normalise dash-form MACs inside the scanner. **`opencmdb-core` is not touched**;
        `MacAddr::from_str` stays colon-only (D47). (`MacAddr.0` is `pub`, so `addr.0[0] & 1`
        compiles from `opencmdb-bin` with no core change — verified.)
  - [x] Tighten `is_synthetic_mac` to refuse MULTICAST addresses, keeping the VRRP block. Update its
        doc — it is the one place the "closed list" rule is written. Add the
        locally-administered-multicast row to `the_vrrp_allowance_is_five_octets_exact` (`:2243`).
  - [x] Split the refusal MESSAGE: a multicast sentence for the I/G case, the existing wording
        verbatim otherwise (both at `:1069` and `:1161`), so `:2287`'s `expected` still matches.
  - [x] Six prove-to-red observations, one per row, each recorded with its exact message, each
        `expected` naming the exact address.
  - [x] Re-run `the_wire_spec_encodes_the_measured_field_behaviours` (`:2790`) — it is the scanner's
        second call site and the largest text it sees. A tokenizer change that is green on trap text
        can still red there.
  - [x] Confirm the two existing wiring tests (`:2276`, `:2286`) still pass unchanged — they are the
        4.14 amendment's only guard and must not be quietly re-pointed. Fix the now-false sentence
        in the first one's doc (`:2272-2275`).

- [x] **Task 2 — the dot-entry skip on the trap tree (AC5, AC4)**
  - [x] Reproduce the failure first: write `fixtures/scenario/traps/.claude/.cc-writes/probe.txt`,
        run `cargo test -p opencmdb-bin`, and record WHICH tests red (expected: 1 in `fixtures.rs`,
        **6 in `trap_gate`** — including `an_answer_for_an_unknown_trap_is_refused`, which expects an
        error but a different one). Delete the probe. `.claude/` is git-ignored, so `git status`
        stays clean. `cargo xtask ci` stays GREEN throughout — its walk already skips dot-entries.
  - [x] Skip dot-entries in the test walk AND in `discover_trap_files`, naming the cost.
  - [x] Re-run with the probe in place → green; without the skip → red. Two-sided, as 5.1's
        mutation 12.

- [x] **Task 3 — hoist one trap walk and wire the text scan onto it (AC1, AC5)**
  - [x] Hoist `every_trap_file_in_the_corpus_is_valid`'s inline walk to `#[cfg(test)] pub(crate) fn
        walk_trap_files`, placed beside `walk_replay_streams`, asserting its own non-emptiness
        (5.1's shape — the five-verbatim-copies lesson).
  - [x] **Do not** author a third walk. **Do not** promote `discover_trap_files`: it is the
        production path and its doc argues the separation. If its doc's sentence about the two walks
        stops being true, update the doc in the same change.
  - [x] Close the hoisted walk's inherited items (ii) `is_file()` and (iii) `sort()`; dispose of
        (i) the root symlink check explicitly — fix or register (AC5). *(All three CLOSED in the
        hoist, including (i); the replay-side twins stay open and the divergence is registered.)*
  - [x] Add the text scan over each trap file's raw bytes, comments included, before `read_traps`,
        reusing the `read_to_string`-then-scan shape already at `fixtures.rs:2793`.
  - [x] Assert the coverage (`macs_seen >= 4 && ips_seen >= 3`) with the counts in the message; the
        doc states the uneven distribution beside it, never instead of it.
  - [x] Prove-to-red per AC4 (a vendor MAC in a header COMMENT).
  - [x] Report the file-size number. **Expect it to be UNCHANGED at 728**: the gate counts to the
        FIRST `#[cfg(test)]` at any indentation (`xtask/src/main.rs:67-76`), which is
        `walk_replay_streams`'s own attribute at `:729`; placing `walk_trap_files` beside it moves
        the count by zero. Say why it did not move — do not chase 5.1's 698 → 720 → 728 trail.

- [x] **Task 4 — `raw` through the scanner (AC2, AC4)**
  - [x] Add the call in `the_corpus_carries_no_real_network_data`'s `Record::Observation` arm.
  - [x] Doc says it is vacuous on today's corpus and why — one `raw` value, no address.
  - [x] Ship the PERMANENT guard: a `should_panic` test over a hand-built `Record::Observation`
        whose `raw` names a non-documentation address. **`walk_replay_streams` cannot be pointed at
        a scratch tree** — it hardcodes its root and must not be parameterised. Extract the arm's
        body into a helper if that is what wiring it takes.
  - [x] Say in the record that the mutation is record-side BECAUSE the corpus has no `raw` to break.

- [x] **Task 5 — the register and the docs it makes stale (AC7, AC6)**
  - [x] Close the two story-4.14 bullets and the story-4.16 bullet in place, appending — correcting
        the 4.14 bullet's "only call site" wording rather than striking it silently.
  - [x] Add the one-line note that 5.1 closed the dot-entry class on the replay tree only.
  - [x] Open `## Deferred from: story-5.2` for anything surfaced and not fixed, with an owner —
        at minimum the three unscanned surfaces and any unclosed inherited walk item.
  - [x] Update `fixtures/README.md` (`:41-60`) and `fixtures/scenario/traps/README.md` (`:96-101`).
        Confirm with `git diff --stat fixtures/` that ONLY `README.md` files moved.
  - [x] Update `docs/project-context.md:62-65` — the `opencmdb-bin` test count, and the sentence
        *"two deliberate privacy-walk amendments"*, which this story makes THREE (the multicast
        refusal joins 4.14's VRRP range and 4.17's honestly-empty hostname).

- [x] **Task 6 — the gate and the branch (AC6)**
  - [x] Full local gate: all four commands, all green.
  - [x] `git status` under `fixtures/` shows README changes only; `MANIFEST.toml` unchanged.
  - [x] Update `sprint-status.yaml` with what was delivered AND what moved. **Set `review`, not
        `done`** — `done` is the merge's business (5.1 established this and its review confirmed it).
  - [ ] Branch → `code-review` → push → PR → green CI → squash merge. **In that order**: 5.1 proved
        it works and its review found three substantive defects, one of which was a tree that did
        not compile. *(Branch `story-5.2-privacy-floor-reaches-the-bytes` created; `code-review`,
        push and PR are the next steps after this workflow, deliberately not taken by it.)*

### Review Findings

_Code review held 2026-07-28 — three parallel layers (Blind Hunter, diff only · Edge Case Hunter,
diff + read access · Acceptance Auditor, diff + spec + context docs). Every claim below was
re-verified against the tree by the reviewer rather than taken from a layer's word; the two that
required a run are marked **MEASURED BY THE REVIEW**. AC verdict: **AC1 PARTIAL · AC2 MET ·
AC3 PARTIAL · AC4 MET · AC5 PARTIAL · AC6 MET · AC7 PARTIAL**. The gate was reproduced
independently: `cargo fmt --check` clean, `clippy --all-targets -D warnings` clean, 258 tests
(130+86+42), `xtask ci` all gates green with `views-hash STALE` informational, `MANIFEST.toml`
untouched and all 25 artefacts matching their sha256, only the two orphan-exempt `README.md` files
moved under `fixtures/`, `opencmdb-core` and `xtask` untouched (D47), `fixtures.rs` still 728 code
lines. The story's own two self-reported findings were both checked and are both CORRECT._

_The theme of this review is not the code — the mechanism is sound and the six evasion rows are
genuinely closed. It is that **four documents this story wrote assert more than the tree does**, in
a story whose stated bar is "a doc comment must be TRUE" and whose own register closes a bullet for
having been written on a false premise._

- [x] [Review][Decision → RESOLVED as (b), Guy's call 2026-07-28] **The FIFO hang is NOT closed, and the register says the class was disposed of** — `discover_trap_files` (`trap_gate.rs:306`, the PRODUCTION walk) gained the dot-entry skip but NOT the `is_file()` guard, and `read_traps` reads with `read_to_string` (`fixtures.rs:666`). Six `trap_gate` tests drive that walk against the committed root, so a FIFO named `x.toml` under `scenario/traps/` still blocks and the SUITE STILL HANGS. **MEASURED BY THE REVIEW** (Acceptance Auditor, reproduced twice): with the guard in place, `timeout 90 cargo test -p opencmdb-bin` returns **143 (SIGTERM), no output**; only a filtered run surfaces the named failure. The Debug Log's two-sided "WITH the guard, a named failure … WITHOUT it, the suite HUNG" therefore compares a filtered run to a full one, and `deferred-work.md`'s disposal section — *"a check that was run"* — plus `walk_trap_files`'s own doc (*"a FIFO … would HANG the suite … is refused by name"*) both read as if the suite no longer hangs. Two defensible ways out, and the choice is scope, not correctness: **(a)** add the `is_file()` refusal to `discover_trap_files` too (the foreign-extension arm three lines below is the `FixtureError` idiom to copy), or **(b)** leave production alone and correct the record — narrow the register and the two doc sentences to `walk_trap_files`, and register the production walk's hang under `## Deferred from: story-5.2` with an owner. **Resolved as (b):** production code stays untouched (widening the story to it would need its own prove-to-red and is outside every AC), and the three false-by-scope sentences were narrowed — `walk_trap_files`'s doc, its inline `is_file()` comment, the register's disposal bullet — plus this story's own Debug Log entry, which compared a filtered run to a full one. The production walk's hang is registered with an owner.
- [x] [Review][Patch] **`the_text_scanner_admits_the_vrrp_range`'s replacement doc is falsified by this very commit** — the old false "only call site" sentence was correctly removed, but the sentence written in its place says *"no COMMITTED text exercises the VRRP allowance through the scanner, so without this test the scanner's MAC leg could silently keep a different rule … and nothing would red."* Both halves are now false, and this story is what made them false. **MEASURED BY THE REVIEW:** `fixtures/scenario/traps/vrrp-virtual-mac.toml:37` carries `00:00:5e:00:01:0a` in a header COMMENT (and nowhere else in the file), which `the_committed_trap_text_carries_no_real_network_data` now reads and scans; `0x00 & 0x02 == 0`, so that address is admitted ONLY by the VRRP leg. Dropping `|| addr.0[..5] == [0, 0, 94, 0, 1]` reds that test — `vrrp-virtual-mac.toml: free text names 00:00:5e:00:01:0a, which is neither locally administered nor in the IANA VRRP virtual-router range`. This is exactly the *"a check that its own commit falsifies"* failure the story quotes twice and the register closes the 4.14 bullet for. [crates/opencmdb-bin/src/fixtures.rs:2540]
- [x] [Review][Patch] **The scanner's doc contradicts itself about the anchor, and cites two wrong mutations** — three defects in one doc block. (1) *"Removing the anchor ALONE leaves all 127 tests green"* is stated in the present tense and is false of the tree that ships it: the suite is **130** tests, and the story's OWN mutation 7 records that dropping the anchor with the new blindness guard present is RED. (2) Four lines later the same doc says *"removing the anchor reds it"* — a direct self-contradiction. (3) The citations are off by the same offset: the doc says *"(story 5.2, mutation 4)"* for the `92.0.2.90` wire-test red (the table numbers it **6**) and *"(story 5.2, mutation 3)"* for the anchor-only-green (the table numbers it **5**); mutations 3 and 4 are the `is_multicast_mac` drop and the message collapse. The weaker true sentence was available: *"measured at 127 tests, before the blindness guard existed; in the delivered tree removing the anchor reds that guard and nothing else."* [crates/opencmdb-bin/src/fixtures.rs:1237-1244]
- [x] [Review][Patch] **The coverage doc overstates twice: `reason` strings, and a per-file distribution the assertion does not enforce** — (a) both `the_committed_trap_text_carries_no_real_network_data`'s doc (*"a re-authoring that drops those four reasons"*) and `fixtures/scenario/traps/README.md` (*"pinned to the 4 distinct MACs and 3 distinct IPs the committed reasons carry today"*) attribute the coverage to `reason` strings, but `00:00:5e:00:01:0a` lives ONLY in a header comment — which understates the guard and contradicts AC1's whole point, that the scan runs before TOML parsing discards comments. (b) The doc enumerates the per-file distribution (`dhcp-churn` 1+2, `example` 1, `randomized-mac` 1, `vrrp-virtual-mac` 1+1) and then asserts only `macs.len() >= 4 && ips.len() >= 3` over a GLOBAL `BTreeSet` union: story 5.2b dropping `randomized-mac.toml`'s MAC while adding two elsewhere keeps the count at 5 and passes. Prose asserting what the executable code does not check — the register's own lesson at `walk_replay_streams`. Weaken the doc to the global floor the assertion actually is, or pin per file. [crates/opencmdb-bin/src/fixtures.rs:1918-1934]
- [x] [Review][Patch] **`fixtures/README.md` claims every uncovered surface has an owner; the register says one does not** — the README lists the hostname-in-prose hole among the uncovered surfaces and closes with *"All are recorded in `deferred-work.md` with an owner."* `deferred-work.md` says the opposite in as many words: *"A fourth hole is named but has no owner because it is not mechanically closable: a hostname in prose."* The register's own 4.14 closure agrees (*"three of its four remaining holes now have an owner"*). [fixtures/README.md:51-54]
- [x] [Review][Patch] **The "what remains a floor" enumeration is presented as complete and is not** — AC3 requires the residual limit written into the doc rather than quietly dropped, and the register hands out *"one owner each"* against this list. Five shapes pass the new scanner clean and none is named (all MEASURED by the Edge Case Hunter against the shipped tokenizer, two independently re-derived by the Blind Hunter): **IPv6 literals entirely** — no branch attempts them, they are pure hex-and-colon so they are collected as runs and discarded, and `Observation.raw` (the surface this story just wired) is exactly where a pasted capture's global-unicast IPv6 lands; the story's own multicast rationale invokes real IPv6 interface-identifier bytes as the thing worth refusing. **Zero-padded IPv4** (`010.001.002.003`) — `Ipv4Addr::from_str` rejects leading zeros, which the doc cites only as the reason substring enumeration would red the corpus, never as the hole it also opens. **The Cisco dotted MAC** (`0011.2233.4455`) and the bare form (`001122334455`) — the same address row (d) was closed for, in the notation every IOS/Aruba/HP CLI emits. **The glue limit is any HEXDIGIT, not just a hex letter** — the doc and the guard's own name say *"glued to a hex-letter prefix (`ab198.18.0.1`)"*, but `1198.18.0.1` (a VLAN id, a port number) is equally invisible, which is far likelier and makes the admitted limit read more exotic than it is. **The resume can swallow a real address adjacent to an accepted one** — `0a:00:11:22:33:44:55` matches the synthetic `0a:00:11:22:33:44` and skips the vendor MAC at offset 3; `192.0.2.110.0.0.1` matches the documentation `192.0.2.110` and skips `10.0.0.1`. That last one is a limit of the mechanism this story introduced, so it is owed a sentence in the same paragraph as the anchor's. Name them; closing any is not this story's scope. [crates/opencmdb-bin/src/fixtures.rs:1246-1251]
- [x] [Review][Patch] **`the_text_scanner_is_blind_to_an_address_glued_to_hex` asserts nothing** — it calls `assert_text_is_synthetic` and discards the returned `ScannedText`, so replacing the whole scanner body with `ScannedText::default()` leaves it green. Its own doc claims it is *"the only place the anchor's behaviour is observable"*; as written it cannot distinguish "the anchor blocked the candidate" from "the tokenizer found nothing anywhere, ever". The machinery to fix it is already returned and the fix is one line: put a findable control address in the same string and assert the scan saw it and only it (e.g. `"the label reads ab198.18.0.1 next to 192.0.2.1"` → `assert_eq!(seen.ips, vec![Ipv4Addr::new(192,0,2,1)])`). This is the story's own theme — a guard that passes on nothing — applied to the guard the story added to close it. [crates/opencmdb-bin/src/fixtures.rs:2648-2654]
- [x] [Review][Patch] **Two small doc-truth slips in the MAC rule** — (a) `is_synthetic_mac`'s doc says of `33:33:…` and `01:00:5e:…` that *"the two are now refused by one rule"*; they are not. `01:00:5e:00:00:0a` has its U/L bit clear, so `&&` short-circuits and `is_multicast_mac` is never evaluated — it is refused by the old leg exactly as before, and only the MESSAGE was unified. The story itself says so in AC3 (*"already refused today … only the stated reason changes"*). (b) Row (e)'s test doc says a solicited-node MAC's *"low FOUR bytes are the low four bytes of a real IPv6 address"* while `is_synthetic_mac`'s doc says *"low three bytes"* — three is right (the fourth from the end is the constant `0xff`), so two docs in one file disagree. [crates/opencmdb-bin/src/fixtures.rs:1310-1316, 2657-2659]
- [x] [Review][Patch] **`sprint-status.yaml` ships the pre-correction probe count** — its contexting block still says the probe reds `every_trap_file_in_the_corpus_is_valid` *"AND five trap_gate tests"*, while the delivery block of the same file, `deferred-work.md`, `trap_gate.rs`'s new comment and the story all say **six** (mutation 8 measured 7 red = 1 + 6). The story's Change Log explicitly records *"the probe count (five → six)"* as a correction that was applied; it was not applied here. [_bmad-output/implementation-artifacts/sprint-status.yaml:180-182]
- [x] [Review][Defer] **The `Record::Failure` scan call site is as vacuous as `raw` and got no permanent guard** [crates/opencmdb-bin/src/fixtures.rs:1179] — deferred, pre-existing. The story's own argument is explicit: *"Because it is vacuous it does not defend itself, so it ships with a PERMANENT guard rather than only a mutation record."* **MEASURED BY THE REVIEW:** the corpus holds exactly one failure text — `"the documentation subnet stopped answering mid-sweep"` — and it carries no address, so emptying the `Record::Failure` arm three lines below the `raw` arm reds nothing. The wiring is undefended by the same test the story applied to its sibling. The vacuity predates this story (4.x wired the arm); this story only relocated it into `assert_record_is_synthetic`.
- [x] [Review][Defer] **The dot-entry skip is evaluated AFTER the symlink refusal in all three walks** [crates/opencmdb-bin/src/fixtures.rs:59-74, trap_gate.rs:324-356] — deferred, pre-existing. A tooling scratch directory materialised as a SYMLINK (`fixtures/scenario/traps/.cache` → elsewhere) panics *"the corpus must contain its own bytes, not a symlink"* before the `continue` is ever reached, so the class the skip exists to close — *"tooling scratch is not corpus"* — is closed only for real directories. The ordering copies story 5.1's replay walk verbatim, so fixing it in one walk would create a fourth divergence; it belongs with the replay-side twins already registered.
- [x] [Review][Defer] **A non-UTF-8 filename bypasses the dot-entry skip** [crates/opencmdb-bin/src/fixtures.rs:286-292, trap_gate.rs:894-900] — deferred, pre-existing. `entry.file_name().to_str().is_some_and(|n| n.starts_with('.'))` yields `false` for a non-UTF-8 name (legal on Linux), so `.cache-\xFF` is not skipped, falls through, and is accused as a foreign extension — the exact class the skip exists to prevent, on the kind of name a tool is most likely to produce. `as_encoded_bytes().starts_with(b".")` is the byte-level test the intent calls for. Same idiom in all three walks, so it is one decision, not three.
- [x] [Review][Defer] **`Fact::Hostname { name }` is prefix-checked, never text-scanned** [crates/opencmdb-bin/src/fixtures.rs:1385] — deferred, pre-existing. The arm asserts `name.is_empty() || name.starts_with("doc-")`, so `doc-192.168.1.1` satisfies the hostname rule and its address is never seen by `assert_text_is_synthetic`. It is the same shape as the `OuiVendor.vendor` / `Uplink.peer_port` surfaces this story registered, and it belongs beside them; the rule predates the story.

_Dismissed as noise (6), with the reason each was refuted: **MAC letter-case** — `MacAddr::from_str` uses `u8::from_str_radix(part, 16)` and `observation/mod.rs:267` already pins `AA:BB:CC:DD:EE:FF`; no hole. **`LONGEST_ADDRESS` documented as perf only** — its doc states the correctness derivation first (`00:11:22:33:44:55` is 17 bytes, `255.255.255.255` is 15). **`mac_refusal_reason`'s multicast sentence is false of broadcast/LLDP** — the primary clause (*"is a MULTICAST address and names no interface"*) is true universally; the IPv6 clause reads as the rationale, not as a claim about the address at hand. **AC4's per-row granularity** — rows (a)–(e) are recorded as one grouped observation, but every substantive thing AC4 asked for is present, including each `expected` naming the exact address. **`walk_replay_streams` keeps all three defects** — already registered under `## Deferred from: story-5.2` with an owner, deliberately and with the divergence named. **Every `README.md` is un-scanned prose** — already registered with an owner and stated in both READMEs._

## Dev Notes

### What was measured, before the story was written

Every number below comes from a run against the committed corpus on 2026-07-28, at
`e846836` (master, story 5.1 merged). They exist so the dev does not re-derive them, and so a
surprise during implementation reads as a FINDING rather than as an expected cost.

- **Trap text is worth scanning.** 4 distinct MACs and 3 distinct IPs live in
  `fixtures/scenario/traps/*.toml` today; all 7 pass the current rule. Distribution:
  `dhcp-churn.toml` 1 MAC + 2 IPs · `example.toml` 1 MAC · `randomized-mac.toml` 1 MAC ·
  `vrrp-virtual-mac.toml` 1 MAC + 1 IP · the other six files, nothing.
- **The boundary-anchored tokenizer finds nothing NEW in the committed trap text** — the same 4 MACs
  and 3 IPs, no more, and the wire body's 4 IPs / 4 MACs unchanged. So AC3 closes a hole; it does not
  reveal a leak. That is why AC4's mutations use crafted inputs.
- **The all-substrings tokenizer, by contrast, REDS the corpus.** `Ipv4Addr::from_str` refuses only
  leading zeros, so `92.0.2.120` and `2.0.2.120` parse: 8 false positives in `dhcp-churn.toml`, 2 in
  `vrrp-virtual-mac.toml`, 10 in `scenario/wire/unifi-clients.json`. This is the story's single
  highest-risk turn and it is measured, not feared.
- **Dash-normalisation is safe against the corpus's hyphenated identifiers** — UUIDs
  (`aeaeaeae-0000-4000-8000-000000000004` normalises to colon groups of 8/4/4/4/12, never a MAC's
  3-apart spacing), `doc-network-…`, `doc-host-…`, `swport-11`, `must-not-merge`, and the 5-octet
  HSRP literal `00:00:0c:07:ac` (six octets required) all stay unparsed. Zero false MACs or IPs.
- **`raw` is one value.** Across 13 replay streams + the wire artefact, exactly one non-null:
  `minimal.jsonl:3`, `{"provenance":"never read by a decision"}`. No address.
- **The multicast tightening is free.** 39 distinct MACs across `Fact::Mac` and `Uplink::peer_mac`
  in all 14 committed `.jsonl` files; **zero** have the I/G bit set. The VRRP address
  `00:00:5e:00:01:0a` (2 sites) has I/G clear and is admitted by the explicit block regardless.
- **The trap tree's dot-entry hole is real, and it reaches the harness, not the gate.** With
  `fixtures/scenario/traps/.claude/.cc-writes/probe.txt` present:
  `every_trap_file_in_the_corpus_is_valid` panics at `fixtures.rs:1661` with *"only .toml trap files
  belong under scenario/traps/"*, and **SIX `trap_gate` tests fail** —
  `the_committed_corpus_is_discovered_and_scored_by_nothing` (`:384`),
  `the_report_says_plainly_that_nothing_was_scored` (`:409`),
  `a_trap_with_no_answer_is_discovered_but_not_scored` (`:428`),
  `passed_is_the_failures_gate_with_a_discovered_floor` (`:587`),
  `an_answer_for_an_unknown_trap_is_refused` (`:744`, the easy one to miss: it expects an error, and
  gets `Io` instead of `AnswerForUnknownTrap`) and
  `replaying_the_same_corpus_twice_yields_identical_verdicts` (`:1050`). All six drive
  `discover_trap_files`. `cargo xtask ci` stays GREEN — its own corpus walk already skips
  dot-entries (`xtask/src/main.rs:626-629`).
  ⚠️ **This is NOT a cause for issue #38 and must not be recorded as one.** The directories were
  created 2026-07-26, they are empty, and #38's failures predate them. *A cause needs a check, not a
  plausible story.*
- **`assert_text_is_synthetic` has TWO corpus call sites, not one.** The `Record::Failure` arm
  (`:1044`) and `fixtures.rs:2793`, where `the_wire_spec_encodes_the_measured_field_behaviours`
  (story 4.18) scans `scenario/wire/unifi-clients.json` whole — that directory sits outside every
  corpus walk, so the test is also its privacy coverage. Both `deferred-work.md:419-420` and the
  4.14 wiring test's doc (`:2272-2275`) still say "only call site"; they were written before 4.18
  landed and are false today.

### The scanner as it stands, and exactly what is wrong with it

```rust
fn assert_text_is_synthetic(text: &str, path: &Path) {          // fixtures.rs:1060
    for token in text.split(|c: char| !(c.is_ascii_hexdigit() || c == '.' || c == ':')) {
        if let Ok(addr) = token.parse::<Ipv4Addr>() { assert_documentation_ip(addr, path); }
        if let Ok(mac) = MacAddr::from_str(token) { assert!(is_synthetic_mac(mac), …); }
    }
}
fn is_synthetic_mac(addr: MacAddr) -> bool {                    // fixtures.rs:1089
    addr.is_locally_administered() || addr.0[..5] == [0, 0, 94, 0, 1]
}
```

- `.` and `:` are KEPT inside tokens, so `198.18.0.1.` and `00:11:22:33:44:55:` are single tokens
  that parse as neither. `198.18.0.1:8080` is worse — the punctuation is interior.
- `-` is a SEPARATOR, so `00-11-22-33-44-55` shatters into six two-character tokens, none of which
  is a MAC. `MacAddr::from_str` is colon-only by construction (`observation/mod.rs:111`). An
  edge-trim reaches NOTHING of this row: the shattering happens before any trim could run.
- `is_locally_administered` is `self.0[0] & 0b0000_0010 != 0` (`observation/mod.rs:78-80`) — the U/L
  bit alone. `0x33` has it set; `0x01` does not. Hence the inconsistency AC3(e)/(f) closes.
- The *"a floor, not a proof"* sentence is `assert_text_is_synthetic`'s doc (`:1057-1058`).
  `is_synthetic_mac`'s doc (`:1077-1091`) carries the different claim that *"the list is CLOSED and
  5-octet exact"*. Both change here; do not merge them.

**What stays a floor after this story, and must be said rather than quietly dropped:** a hostname in
prose still cannot be recognised mechanically (the scanner's doc says so today — keep the sentence);
`Fact::OuiVendor { vendor }` and `Fact::Uplink { peer_port }` are free author-typed text that
`assert_facts_are_synthetic` drops by construction (`:1133`, `:1122`); and every `README.md` is
exempt at any depth, `fixtures/scenario/traps/README.md` being 6 KB of the corpus's largest
un-scanned prose. Four residual holes, all four named — the story's title is a direction, not a
claim of completeness.

### What this touches, and what it must not break

- **`crates/opencmdb-bin/src/fixtures.rs`** (UPDATE) — **728 CODE lines** (the first `#[cfg(test)]`
  is at line 729; the ceiling is 2000 and the largest file in the tree is 884).
  *Today:* the corpus reader, `walk_replay_streams` (`:729`, hoisted by 5.1), and the test module
  holding `the_corpus_carries_no_real_network_data:1033`, `assert_text_is_synthetic:1060`,
  `is_synthetic_mac:1089`, `assert_facts_are_synthetic:1096`, `assert_documentation_ip:1143`,
  `assert_synthetic_mac:1157`, `scratch_dir:1234`, `every_trap_file_in_the_corpus_is_valid:1625`,
  `the_vrrp_allowance_is_five_octets_exact:2243` (the closed list's real pin), the two 4.14 wiring
  tests `:2272-2293`, and `the_wire_spec_encodes_the_measured_field_behaviours:2790` — **the
  scanner's second call site (`:2793`), and the largest body of text it sees. Every tokenizer change
  must be re-run against it.**
  *This story changes:* the scanner's tokenizer, MAC rule and refusal message; one new call site in
  the `Record::Observation` arm plus its permanent guard; the trap walk hoisted with a text scan
  wired onto it; the now-false doc at `:2272-2275`.
  *Must be preserved:* the exhaustive `match` with no `_` arm in `the_corpus_carries_no_real_network_data`
  (a new `Record` variant must still break it — load-bearing, 4.5b proved it). ⚠️ Note that
  `assert_facts_are_synthetic` DOES end in a catch-all `other => panic!(…)` (`:1134`) — required
  because `Fact` is `#[non_exhaustive]`. Do not "fix" it into a compile error. Also preserved:
  `expected()` as the second independent oracle; the two 4.14 wiring tests (`:2276`, `:2286`) passing
  unchanged; `walk_replay_streams`'s symlink refusal, `README.md` exemption at any depth, dot-entry
  skip, its own non-emptiness assertion, and its FIXED root.
- **`crates/opencmdb-bin/src/trap_gate.rs`** (UPDATE) — `discover_trap_files:302` gains the
  dot-entry skip. Its doc at `:291-301` names both other walks explicitly at `:298-301`; if the
  shape changes, the doc changes with it. **Its `found.sort()` (`:355`) stays** — it is why a
  discovery run is deterministic, and `deferred-work.md:529-537` records the replay walk's LACK of
  it as open. The module is `#![allow(dead_code)]` and wired into no runtime path: its tests are the
  future gate's harness, not the gate.
- **`crates/opencmdb-core/`** — **NOT touched.** `MacAddr::from_str` stays colon-only (D47), and
  `MacAddr.0` is already `pub` (`observation/mod.rs:71`), so every byte test AC3 needs compiles from
  `opencmdb-bin` with no core change.
- **`xtask/src/main.rs`** — **NOT touched.** Its corpus walk already skips dot-entries (`:626-629`)
  and its orphan rule exempts `README.md` by exact name (`:735`), so the lock stays consistent with
  both post-story walks in both directions.
- **`_bmad-output/implementation-artifacts/deferred-work.md`** (UPDATE) — append-only discipline.
- **`_bmad-output/implementation-artifacts/sprint-status.yaml`** (UPDATE) — live source of truth.
- **`docs/project-context.md`** (UPDATE) — `:62-65` carries the bin test count and the sentence
  *"two deliberate privacy-walk amendments"*; this story makes it three.
- **Under `fixtures/`: the two `README.md` files ONLY** (AC6). No artefact bytes, no `MANIFEST.toml`.

### Inherited from story 5.1 — read this before writing a doc comment

5.1's code review found three defects worth carrying forward as habits, not as trivia:

1. **A tree that does not compile is not a deliverable.** Run the crate, not just the diff, before
   claiming anything.
2. **A helper that closes a vacuity can re-introduce it.** `assert_obs_ids` shipped asserting the
   ids of *whatever it was handed* and nothing about the length, so an empty slice passed. Ask of
   every new guard here: what does it do when handed nothing? A text scan over ZERO trap files is
   the same failure mode — hence AC5's non-emptiness assertion on `walk_trap_files`. And one level
   deeper, which is where 5.1's version of this bug actually lived: a scan over ten files carrying
   zero addresses is *also* vacuous, and counting files does not catch it — hence AC1's
   `macs_seen >= 4 && ips_seen >= 3`. The same question is owed to the `raw` call site (AC2's
   permanent guard) and to the dot-entry skip.
3. **A check that its own commit falsifies is worse than no check.** 5.1's register entry cited a
   `grep` its own diff broke, and the same sentence had reached `epics.md` as the NEXT story's
   premise. When this story writes "verified by …", re-run the verification after the last edit.

Also inherited: `assert_obs_ids(observations, prefix, expected_len)` now takes a length. Story 5.2b,
not this story, is what calls it for four more families — **do not pre-empt 5.2b by adding value
pins here.**

### House rules that bind this story

- **Prove-to-red is not optional** (story 1.3). AC4 names one per row, and rows (a)–(d) need the
  two-sided observation because a tokenizer hole's "red" is a counterfactual.
- **Name the test behind every claim.** The temptation here is exact: writing *"the corpus privacy
  rule now covers the committed bytes"* when what holds is *"trap text and `raw` are scanned by the
  same address-shaped rule; a hostname in prose, `OuiVendor.vendor`, `Uplink.peer_port` and every
  `README.md` are still invisible, and no committed `raw` exercises the new call site."* Write the
  weaker true sentence — the story's TITLE is a direction, not a completion claim.
- **A doc comment must be TRUE.** Three reviews, plus 5.1's, caught docs asserting behaviour the
  code did not have — and this story starts with two live examples (`fixtures.rs:2272-2275` and
  `deferred-work.md:419-420`, both saying "only call site"). `is_synthetic_mac`'s doc matters most:
  it is where the closed list of admitted byte shapes is written down. **Assertion messages are held
  to the same bar** — see the multicast branch in AC3.
- **DRY, with deliberate redundancy protected.** A third trap walk is accidental duplication →
  hoist (AC5). The production/test walk split is DELIBERATE and argued in `trap_gate.rs`'s doc →
  keep it. `scratch_dir`'s per-module duplication (`fixtures.rs:1234`, `trap_gate.rs:373`) is the
  deliberate redundancy CLAUDE.md names — do not collapse it.
- **Dependency frontier (D47):** all work is in `opencmdb-bin`. Do not widen a core parser.
- **File-size gate:** ≤2000 CODE lines, tests excluded, counted to the first `#[cfg(test)]` at any
  indentation. That first attribute is `walk_replay_streams`'s at `:729`, so this story's hoist
  moves the number by ZERO — expect 728, and say why.
- **`DATABASE_URL` is usually unset locally** and the MariaDB-backed tests `return` early — a green
  suite says nothing about the database. Irrelevant here, but do not cite it as evidence.
- **Known local flakiness (issue #38):** unexplained non-determinism, 8 failures across 5 runs
  (stories 4.15/4.17). CI on a clean checkout has never reproduced it and **the "Synology Drive"
  explanation is REFUTED by measurement — do not re-adopt it.** If a corpus test reds unexpectedly,
  re-run and check `git status` before diagnosing.

### Testing standards

Tests live inline in the trailing `#[cfg(test)] mod tests` (D56b, one per file). Test names are
sentences. Assertion messages name the offending FILE — with a corpus walk, *"a real MAC is in the
corpus"* is not actionable unless it says which. Prefer `#[should_panic(expected = "…")]` with a
substring when pinning a panic (4.14's idiom, reused by 4.17 and by the two scanner wiring tests
this story must not break) — with the `expected` naming the exact address, never the generic tail of
the message. Scratch fixtures go through `scratch_dir` (`fixtures.rs:1234`), which keys on
`(process id, tag)` and so gives each TEST its own directory; the helper is duplicated per module
(`trap_gate.rs:373` has its own), and that duplication is the deliberate redundancy the DRY rule
protects. ⚠️ `scratch_dir` produces a path under `std::env::temp_dir()`, which no corpus walk can be
pointed at — see AC4's AC2 bullet.

### Project Structure Notes

Paths follow the established layout with no variance: corpus at the workspace root in `fixtures/`
(D56); the fixture reader, `FixtureConnector` and `trap_gate` in `crates/opencmdb-bin/src`;
`FIXTURES_DIR` expressed exactly once (`fixtures.rs:48`) — take the path from
`fixtures_dir()`/`fixture_path()`, never re-write the string (there is a test:
`the_fixtures_path_is_expressed_once`, and 5.1's Task 3 is where a dev last nearly tripped it).

One judgment call to be aware of: the trap-text scan could live in `trap_gate.rs` beside the
production walk instead of in `fixtures.rs` beside the scanner. The recommendation is `fixtures.rs`
— the scanner and every other privacy assertion live there, and `trap_gate.rs` is the SCORING
harness, not the corpus-integrity layer. If you take the alternative, say why.

### References

- Story source: [Source: _bmad-output/planning-artifacts/epics.md#Story 5.2] (epics.md:1339-1359);
  Epic 5 framing and build order, epics.md:1307-1313; inherited-debt-at-the-head, epics.md:1313.
- Deferred entries CLOSED by this story:
  [Source: _bmad-output/implementation-artifacts/deferred-work.md] — story-4.14 review (both
  bullets, lines 419-434), story-4.16 review (the `raw` bullet, lines 457-463). The closure idiom is
  the file's own `✅ **CLOSED by story X.** ~~struck~~` shape (e.g. line 14), and the
  *"a register that loses an item is worse than no register"* lesson is at line 16.
- Privacy doctrine (D19, *"real captures are a privacy liability in a public repo … disqualifying.
  Not debatable"*): [Source: _bmad-output/planning-artifacts/architecture.md#D19] §`:1267`, sentence
  at `:1319-1320`. Separately, the corpus-is-a-SPEC framing (*"changing one is a commit that says
  'I am changing the spec'"*) is the module doc of `fixtures.rs:1-31` — which says nothing about
  privacy. Two different sources; do not merge the citation.
- The corpus's own privacy prose, which this story falsifies: `fixtures/README.md:41-60` and
  `fixtures/scenario/traps/README.md:96-101` (AC6).
- The VRRP amendment and its closed-list rule (story 4.14): `is_synthetic_mac`'s doc,
  `fixtures.rs:1077-1091`; the closed-list pin `the_vrrp_allowance_is_five_octets_exact:2243`; and
  the two wiring tests at `:2272-2293`.
- Corpus lock, both directions: `fixtures/MANIFEST.toml`; gate in `xtask/src/main.rs` (walk `:626`,
  README exemption `:735`, file-size counter `:67-80`).
- Story flow, PR discipline, local gate: [Source: docs/project-context.md] §"Working conventions"
  (`:186`+). The `architecture-views.md` staleness rule is separate, at `:130-133`.
- The immediately previous story, for the byte-pin idiom, the prove-to-red record shape and the
  review lessons above:
  `_bmad-output/implementation-artifacts/5-1-corpus-pins-obs-id-binding.md`.

### Git intelligence

Last five commits: `e846836` (5.1 bookkeeping, PR #42) · `c84e14a` (**story 5.1**, PR #41) ·
`8c81386` (Epic 5 decomposition, PR #40) · `b568815` (Epic 4 retrospective, #39) · `b4ceca3` (Epic 4
report, #37). `fixtures.rs` was last touched by 5.1, which hoisted `walk_replay_streams`, added the
dot-entry skip on the replay tree and the corpus-wide round-trip — **this story is the same two
moves on the TRAP tree, plus the scanner pass Epic 4 deferred twice.** Before 5.1, `fixtures.rs` was
last touched by `7b93c20` (story 4.18) — which is what added the scanner's second call site. Every
one of those commits went through branch → PR → green CI → squash merge; hold the same. (Do not
quote a running "N/N green" tally in the completion notes: `project-context.md:200` and
`sprint-status.yaml:146` both record 47/47 at 22 PRs, four PRs have merged since, and the repo's own
run history is 100 success / 3 failure / 1 cancelled over 104 runs. The claim is not reconstructible
from any of them — say "green CI on every PR", which is.)

### Toolchain

No new dependency, no version to research, **and no regex crate** — the tokenizer stays hand-rolled.
`regex` IS already in the tree transitively (`pnet_macros` ← `pnet_packet` ← `surge-ping`, and
`rust-i18n` as a build-dependency; `Cargo.lock:1700` pins 1.13.1), so a dev who checks will find it
compiled on every build — but it is **not a direct dependency of any workspace crate**, and
promoting it to one for a test helper would need approval under the story's own rules. Rust 1.96+,
edition 2024. `Cargo.lock` is committed and every build is `--locked`. Never invent a version.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), `bmad-dev-story`, 2026-07-28.

### Debug Log References

Every observation below was run on this branch. Line numbers are pre-change unless stated.

**Baseline (before any edit), master `e846836` + the story file:** `cargo test --workspace` →
121 (bin) + 86 (core) + 42 (xtask) = **249**, green.

**Prove-to-red log — 12 observations.** Rows (a)–(f) are AC3's table; each `expected` names the
exact address, so a scanner that found a *different* address would not be credited with a pass.

| # | Mutation / state | Observed |
|---|---|---|
| 1 | **AC3 rows (a)–(e), scanner AS IT SHIPPED.** The six new tests added before any fix. | Rows (a)–(e) FAILED with *"test did not panic as expected"* — the scanner was GREEN on `198.18.0.1.`, `00:11:22:33:44:55:`, `198.18.0.1:8080`, `00-11-22-33-44-55` and `33:33:ff:00:60:0a`. **The holes existed; this is the two-sided half a counterfactual red needs.** |
| 2 | **AC3 row (f), scanner as it shipped.** | PANICKED, but with the OLD wording: *"…names 01:00:5e:00:00:0a, which is neither locally administered nor in the IANA VRRP…"*. Confirms the story's warning: row (f) was already refused, only its stated REASON changes, so its `expected` had to name the new sentence or the test would prove nothing. |
| 3 | Drop `&& !is_multicast_mac(addr)` from `is_synthetic_mac`. | 2 red: `the_vrrp_allowance_is_five_octets_exact` (the RULE-level pin, `MacAddr([0x33,0x33,0xff,0,0x60,0x0a])`) and `the_text_scanner_refuses_an_ipv6_multicast_mac` (the tokenizer-level row). Row (f) stayed green — correct, its U/L bit is clear. |
| 4 | Collapse the split refusal message (`if is_multicast_mac(addr)` → `if false`). | 2 red: rows (e) and (f), each reporting the OLD sentence against the new `expected`. The message is pinned, not just the rule. |
| 5 | **Drop the boundary anchor ONLY**, keeping longest-match-and-resume. | **ALL 127 GREEN.** ⚠️ A finding, recorded rather than smoothed over: the anchor does NOT keep the corpus safe. See Completion Notes. |
| 6 | Drop the anchor AND the resume (`i += 1` always) — the "all substrings" tokenizer the story warns about. | `the_wire_spec_encodes_the_measured_field_behaviours` RED: *"…unifi-clients.json: 92.0.2.90 is not in an RFC 5737 documentation range"*. The story's measured trap **reproduced**; the RESUME is what averts it. |
| 7 | Drop the anchor, with the new blindness guard in the tree. | `the_text_scanner_is_blind_to_an_address_glued_to_hex` RED (*"in-memory: 198.18.0.1 is not in an RFC 5737…"*). The anchor's one observable consequence now has a falsifiable check. |
| 8 | **AC5 reproduction.** `echo probe > fixtures/scenario/traps/.claude/.cc-writes/probe.txt`, no code change. | **7 red — exactly the story's prediction**: `every_trap_file_in_the_corpus_is_valid` plus SIX `trap_gate` tests (`the_committed_corpus_is_discovered_and_scored_by_nothing`, `the_report_says_plainly_that_nothing_was_scored`, `a_trap_with_no_answer_is_discovered_but_not_scored`, `passed_is_the_failures_gate_with_a_discovered_floor`, `an_answer_for_an_unknown_trap_is_refused`, `replaying_the_same_corpus_twice_yields_identical_verdicts`). `cargo xtask ci` GREEN throughout. |
| 9 | Same probe, WITH the dot-entry skip in both trap walks. | Green. Two-sided, as story 5.1's mutation 12. |
| 10 | Same probe, skip removed from the hoisted `walk_trap_files`. | 2 red (both callers of the hoisted walk), each naming `…/.claude/.cc-writes/probe.txt`. The blast-radius growth a hoist causes, observed. |
| 11 | **AC1 prove-to-red.** `00:11:22:33:44:55` prepended to `example.toml` as a **header COMMENT**, then `git checkout`. | `the_committed_trap_text_carries_no_real_network_data` RED naming the file. A comment, not a `reason` — the scan runs before TOML parsing discards it. `git status fixtures/` clean afterwards; no `MANIFEST.toml` re-hash (`cargo test` does not consult it). |
| 12 | **AC2 prove-to-red.** Delete the `if let Some(raw)` call site. | Exactly ONE test red: `an_observations_raw_payload_is_scanned` (*"did not panic as expected"*). The corpus walk stayed green — **which IS the vacuity, measured.** The mutation is record-side BECAUSE the corpus has no `raw` to break. |

**Mutation 13, added by the code review (2026-07-28)** — for the assertion the review put into
`the_text_scanner_is_blind_to_an_address_glued_to_hex`, which until then asserted nothing:

| # | Mutation / state | Observed |
|---|---|---|
| 13 | Gut the scanner — `assert_text_is_synthetic` returns `ScannedText::default()` before scanning anything. | **Before the patch: GREEN** — the test called the scanner and discarded its result, so a dead tokenizer satisfied it. **After: RED**, `assertion left == right failed … left: [], right: [192.0.2.1]`, with the message *"finding nothing would mean the tokenizer is dead, not that the anchor is working"*. The control address is what turns "this does not panic" into a check. Tree restored, md5 verified, 258 green. |

**Two further measurements, run rather than cited:**

- **The trap-text coverage numbers.** With the assertion temporarily raised to `>= 99`, the message
  read *"the trap-text scan inspected **4** distinct MAC(s) and **3** distinct IP(s)"* — the
  boundary-anchored tokenizer reproduces the story's pre-written measurement exactly, finding
  nothing new and nothing less.
- **The `is_file()` hang, which the register calls "the one failure mode with no diagnostic at
  all".** `mkfifo fixtures/scenario/traps/fifo.toml`, then: WITH the guard, a named failure —
  *"…/fifo.toml: only regular files belong under scenario/traps/"*; WITHOUT it, `timeout 60 cargo
  test` returned **143 (SIGTERM)** with no output — the suite HUNG. FIFO removed; `git status
  fixtures/` clean.
  ⚠️ **Corrected by this story's code review: those two halves are not the same run.** The named
  failure came from a FILTERED invocation; the full suite still returns 143 WITH the guard in
  place, because `discover_trap_files` — the production walk, driven by six `trap_gate` tests —
  never gained the guard. The guard closes the class in `walk_trap_files` only. Reproduced twice by
  the review; the fix was deliberately scoped out (Guy's call) and registered with an owner.

**File-size gate: 728 code lines in `fixtures.rs`, UNCHANGED.** The gate counts to the first
`#[cfg(test)]` at any indentation; that attribute is `walk_replay_streams`'s at `:729` and
`walk_trap_files` was placed after it (the second attribute is now at `:823`), so the count moved
by zero. Confirmed by `awk '/^#\[cfg\(test\)\]/{print NR-1; exit}'`.

**Final gate, re-run in full on the finished tree:** `cargo fmt --all` · `cargo clippy --workspace
--all-targets -- -D warnings` clean · `cargo test --workspace` → **130 + 86 + 42 = 258**, green ·
`cargo xtask ci` → all gates green (`views-hash STALE` informational, exit 0, deliberately not
regenerated) · `git diff --stat fixtures/` → `README.md` ×2 only, `MANIFEST.toml` untouched.

### Completion Notes List

**What holds, in the weaker true sentence rather than the story's title.** Trap-file text (header
comments included, scanned before TOML parsing discards them) and `Observation.raw` now go through
the same address-shaped rule as everything else, and the scanner's three named evasions are closed.
A hostname in prose, `Fact::OuiVendor { vendor }`, `Fact::Uplink { peer_port }` and every
`README.md` remain invisible, and **no committed `raw` exercises the new call site**. The privacy
floor reaches further; it does not cover the bytes.

**Findings — things measured that the story had wrong or had not asked about:**

1. **The boundary anchor is not what keeps the corpus green; the RESUME is** (mutations 5 and 6).
   AC3 specifies "boundary-anchored longest-match" and warns that the substring route reds the
   corpus. Both halves were run: removing the anchor alone leaves all 127 tests green, because with
   longest-match-and-resume an interior start inside an address is never reached anyway. Only when
   the resume ALSO goes does the corpus red — reproducing the story's `92.0.2.90` exactly, in
   `unifi-clients.json`. **The specified shape was kept** (it bounds the scan to address-shaped
   positions rather than sliding through arbitrary hex), but its contribution is not claimed: the
   doc now says which conjunct earns the safety and that the anchor's own red is unobservable here.
   Its one observable consequence — `ab198.18.0.1` stays invisible — was converted from a doc
   sentence into a guard, `the_text_scanner_is_blind_to_an_address_glued_to_hex`, so an admitted
   limit cannot rot into a false claim of coverage and removing the anchor now reds something.
2. **AC6's second README target does not say what the story says it says.** AC6 cites
   `fixtures/scenario/traps/README.md:96-101` as stating "the trap-text rule as review-enforced".
   That sentence is not in that file — it is in `deferred-work.md`'s 4.14 bullet. What IS there and
   IS made false is the MAC rule ("locally-administered MACs"), narrowed by the multicast
   tightening. Both were fixed: the MAC rule corrected, and the now-true "this is a TEST, not a
   review habit" stated explicitly, along with the fact that this README is itself the largest
   un-scanned text in the corpus.
3. **All three inherited walk items were closed, including (i).** AC5 allowed registering the root
   symlink check instead of fixing it; it is one `symlink_metadata` call and was taken. The
   consequence is named rather than hidden: `walk_trap_files` and `walk_replay_streams` now DIFFER
   on three points, because the replay-side twins stay open. That divergence is registered under
   `## Deferred from: story-5.2` with an owner — widening this story to a second tree would have
   made the diff unreviewable, which is the same argument AC5 makes for closing the trap side here.

**Design decisions worth naming for the review:**

- **The trap-text scan is a test of its own** (`the_committed_trap_text_carries_no_real_network_data`),
  sibling to `the_corpus_carries_no_real_network_data`, rather than folded into
  `every_trap_file_in_the_corpus_is_valid`. Both drive the one hoisted walk, so "every trap file"
  cannot mean two sets; each test name says what it proves. It lives in `fixtures.rs`, beside the
  scanner and every other privacy assertion, per the story's recommendation — `trap_gate.rs` is the
  scoring harness, not the corpus-integrity layer.
- **`assert_text_is_synthetic` now RETURNS what it saw** (`ScannedText { ips, macs }`). AC1 needs
  the coverage counted, and the alternative — a second pass with a second tokenizer — would be two
  definitions of "an address in text" that could disagree. The vacuity question story 5.1's review
  raised is answered at BOTH levels: `walk_trap_files` refuses to pass over zero files, and the
  caller refuses to pass having inspected fewer than 4 MACs / 3 IPs.
- **The `Record::Observation` arm was extracted into `assert_record_is_synthetic`.** The permanent
  `raw` guard must drive the code the walk drives, not a copy that could agree with a bug, and the
  walk's root is hardcoded and must stay so. The `match` is still exhaustive with no `_` arm, so a
  new `Record` variant still breaks it and forces a privacy decision (4.5b's lesson, preserved).
- **The refusal message was split through one `mac_refusal_reason` helper**, shared by the free-text
  and structured-fact sites, so they cannot drift. The non-multicast wording is verbatim from
  before, so `the_text_scanner_still_refuses_a_mac_outside_the_block`'s `expected` matches unchanged
  and was not quietly re-pointed.
- **`opencmdb-core` was not touched.** `MacAddr::from_str` stays colon-only (D47); `-` → `:`
  normalisation happens inside the scanner, in `opencmdb-bin`'s test module.

**Two false doc sentences removed rather than re-enumerated.** The 4.14 wiring test's doc and the
`deferred-work.md` 4.14 bullet both said the scanner's *"only call site is the `Record::Failure`
walk"* — true when written, falsified by story 4.18 one day before the register entry was written,
and falsified twice more by this story. The doc's sentence is gone (an inventory in a doc comment
has no guard behind it); the register's is struck with the correction stated, because closing an
item on a premise its own tree falsifies is the failure story 5.1's review named.

**No corpus bytes changed.** `MANIFEST.toml` untouched, all 25 artefacts match their recorded
sha256. The only files that moved under `fixtures/` are the two orphan-exempt, unlisted `README.md`
files that this story's own change made false.

⚠️ **Not recorded as a cause for issue #38**, deliberately: the `.claude/` directories postdate the
flaky runs and are empty, and `walk_trap_files`'s new `sort()` is a thing to RULE OUT, never a
diagnosis. *A cause needs a check, not a plausible story.*

### File List

- `crates/opencmdb-bin/src/fixtures.rs` (MODIFIED) — scanner rewritten (boundary-anchored
  longest-match, dash normalisation, `ScannedText` return, `LONGEST_ADDRESS`); `is_synthetic_mac`
  tightened to refuse multicast; `is_multicast_mac` and `mac_refusal_reason` added;
  `assert_synthetic_mac` routed through the shared reason; `walk_trap_files` hoisted beside
  `walk_replay_streams` with the root symlink check, dot-entry skip, `is_file()` guard and `sort()`;
  `assert_record_is_synthetic` extracted with the `raw` call site; `every_trap_file_in_the_corpus_is_valid`
  rewired onto the hoist; 9 tests added (6 AC3 rows, the trap-text scan, the `raw` guard, the
  anchor-blindness guard); the multicast row added to `the_vrrp_allowance_is_five_octets_exact`;
  the false "only call site" doc sentence removed.
- `crates/opencmdb-bin/src/trap_gate.rs` (MODIFIED) — `discover_trap_files` gains the dot-entry
  skip; its doc updated to name `fixtures::walk_trap_files` and the rules the two walks share.
- `fixtures/README.md` (MODIFIED) — trap-text and `raw` coverage stated, with the `raw` vacuity;
  the three uncovered surfaces named; dot-entry skip and its cost documented; the MAC rule narrowed
  to locally-administered UNICAST plus the VRRP block. Orphan-exempt, unlisted, re-hashes nothing.
- `fixtures/scenario/traps/README.md` (MODIFIED) — same MAC-rule correction; the trap-text rule
  restated as test-enforced rather than a review habit; this file's own exemption named. Same
  exemption status.
- `docs/project-context.md` (MODIFIED) — test count 249 → 258; *"two deliberate privacy-walk
  amendments"* → three, with the multicast refusal distinguished as the one that REFUSES a byte
  shape with no family behind it.
- `_bmad-output/implementation-artifacts/deferred-work.md` (MODIFIED) — three bullets closed in
  place (4.14 ×2, 4.16), the 4.14 bullet's false premise corrected as it closed; new
  `## Deferred from: story-5.2` section recording the inherited-item disposal, the replay-side
  divergence, the both-trees dot-entry closure and the three unscanned surfaces with owners.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` (MODIFIED) — story 5.2 → `review`.
- `_bmad-output/implementation-artifacts/5-2-privacy-floor-reaches-the-bytes.md` (MODIFIED) — this
  record.

## Change Log

| Date | Change |
|---|---|
| 2026-07-28 | Story contexted. Measurements taken against `e846836` before writing: trap text carries 4 MACs / 3 IPs across 4 of 10 files (the scan is non-vacuous); `raw` has exactly one committed value and no address (the new call site is vacuous, and says so); 39 distinct committed MACs, zero multicast, so the tightening reds nothing; and a probe file under `fixtures/scenario/traps/.claude/` reds one fixtures test plus six `trap_gate` tests — the dot-entry class story 5.1 closed on the replay tree only. |
| 2026-07-28 | **Implemented** on branch `story-5.2-privacy-floor-reaches-the-bytes`. 249 → 258 tests (130 bin + 86 core + 42 xtask); 12 mutations proven to red and recorded, plus two run measurements (the 4/3 coverage counts, and the FIFO hang killed at 60s by SIGTERM). Corpus bytes UNCHANGED — only the two orphan-exempt `README.md` files moved under `fixtures/`, `MANIFEST.toml` untouched, `fixtures.rs` still 728 code lines. Three register bullets closed, one of them with its own false premise corrected as it closed. **One finding:** AC3's boundary anchor was measured NOT to be what keeps the corpus green — the RESUME is; the specified shape was kept but the doc now says which conjunct earns the safety, and the anchor's only observable consequence gained a guard. **One correction:** AC6 cites a "review-enforced" sentence in `scenario/traps/README.md` that is not in that file (it is in `deferred-work.md`); what IS false there is the MAC rule, and that was fixed. |
| 2026-07-28 | **Code review held** — three parallel layers, every claim re-verified against the tree by the reviewer. Verdict: AC1 PARTIAL · AC2 MET · AC3 PARTIAL · AC4 MET · AC5 PARTIAL · AC6 MET · AC7 PARTIAL. **9 patches applied, 4 items deferred, 6 dismissed as refuted.** The mechanism was sound and the corpus never moved; what the review caught is that four documents this story wrote asserted more than the tree does. The two load-bearing ones, both MEASURED rather than argued: (1) the sentence written to REPLACE the removed "only call site" claim — *"no COMMITTED text exercises the VRRP allowance through the scanner"* — is falsified by this very commit, because `vrrp-virtual-mac.toml:37` carries `00:00:5e:00:01:0a` in a header comment that the story's own new trap-text scan reads; dropping the VRRP leg reds that scan. Two rotted inventories in one doc block, one story apart. (2) The FIFO hang is closed in `walk_trap_files` ONLY — `discover_trap_files` never gained the guard, six `trap_gate` tests drive it, and `timeout 90 cargo test -p opencmdb-bin` still returns 143 with no output; the Debug Log's two-sided claim compared a filtered run to a full one. Resolved as (b) on Guy's call: production untouched, the four sentences narrowed, the hang registered with an owner. Also fixed: a doc that contradicted itself about the anchor two paragraphs apart plus two wrong mutation citations and a stale "127 tests"; the coverage doc attributing to `reason` strings a MAC that lives only in a comment, and claiming a per-file distribution the global floor does not enforce; `fixtures/README.md` promising an owner the register denies; and the residual-floor enumeration, which was presented as complete and missed five measured shapes — IPv6 entirely unscanned, zero-padded IPv4, the Cisco dotted and bare MAC forms, the glue limit being any hexdigit rather than a hex letter, and the resume swallowing an address adjacent to an accepted one. One new guard earned a 13th mutation: `the_text_scanner_is_blind_to_an_address_glued_to_hex` asserted nothing and passed on a gutted scanner; it now carries a control address and reds. Gate re-run whole and green after the patches: fmt, clippy `-D warnings`, 258 tests, `xtask ci` all gates, `fixtures.rs` still 728 code lines, `MANIFEST.toml` untouched, only the two READMEs moved under `fixtures/`. |
| 2026-07-28 | Validated by two fresh-context agents (fact-check + gap-hunt), per the Epic 4 retrospective. Six HIGH findings applied. The load-bearing one: the "obvious" all-substrings tokenizer for AC3 row (c) is MEASURED to red the committed corpus (`Ipv4Addr::from_str` accepts `92.0.2.120` inside `192.0.2.120`), so AC3 now specifies boundary-anchored longest-match and names the trap it replaces. Both agents independently found that `assert_text_is_synthetic` has a second corpus call site (`fixtures.rs:2793`, story 4.18) that AC1's premise, the 4.14 wiring doc and `deferred-work.md` all still deny. Also corrected: AC4's `raw` mutation was not executable (`walk_replay_streams` takes no root) and shipped no permanent guard; the multicast tightening makes two live assertion messages false; the hoist inherits three open register items, one of which HANGS the suite; AC6 forbade two `README.md` updates that docs-current-before-push requires. Plus the probe count (five → six), the `regex` claim (it is in the tree transitively), and thirteen smaller corrections of citation and premise. |
