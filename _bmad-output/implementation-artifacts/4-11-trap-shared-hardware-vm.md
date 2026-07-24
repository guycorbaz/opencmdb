# Story 4.11: Trap family — shared-hardware VM

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As the author of the trap corpus,
I want the shared-hardware VM family committed in positive AND negative form, plus its honestly-ambiguous edge,
so that virtual interfaces sharing one physical host are **neither fused nor split wrongly** — the engine
must group two virtual NICs of one VM, must NOT merge two different VMs just because they share the
hypervisor's uplink, and must ABSTAIN where the signal genuinely cannot tell the two apart.

## Acceptance Criteria

1. **Given** two virtual interfaces that are one VM's NICs (two locally-administered MACs, the SAME
   synthetic hostname, sharing the hypervisor's single physical uplink), **when** the primary-form trap is
   scored, **then** L2 **must GROUP them** — the trap is a `must-merge` naming the L2 rule that FIRES on
   the agreeing hostname. The expectation states explicitly, in one sentence, WHICH grouping is correct and
   WHY (epic AC1: *"the expectation states explicitly which grouping is correct and why, in one sentence"*).
   The shared hardware is **not** what groups them — the hostname is; a distinct virtual MAC is no reason
   to split one VM into two devices.

2. **Given** the inverse form — two virtual interfaces that are DIFFERENT VMs co-tenant on ONE hypervisor
   (two locally-administered MACs, DIFFERENT synthetic hostnames, sharing the SAME physical uplink) —
   **when** it is scored, **then** it asserts they are **NOT grouped**: a `must-not-merge` naming the L2
   rule that OPPOSES the merge (the distinct hostnames). This is the family's whole point: shared hardware
   (identical `Uplink.peer_mac`) **tempts** a false merge, and the corpus proves the gate can refuse it. The
   refusal names the rule that OPPOSES the join (distinct hostnames), **not** the one that was tempting
   (shared uplink) — trap.rs:69-73; example.toml's *"opposes the merge rather than merely failing to
   support it"*.

3. **And** the honestly-ambiguous variant is labelled **`must-abstain` rather than argued** (epic AC2:
   *"the ambiguous variant is labelled `must-abstain` rather than argued"*): a virtual interface that shares
   only the hypervisor's uplink and reports **no hostname** cannot be told apart from a second NIC of an
   existing VM or a new co-tenant VM — the signal is genuinely insufficient, so the expectation is
   `must-abstain` naming an [`AbstentionCause`], never a `RuleId`. This makes shared-hardware-vm the **first
   three-column family** in the committed corpus (merge + not-merge + abstain of ONE scenario), which DR1
   explicitly permits (*"a 3-column family … is expressible and useful"*).

4. **And** every expectation in the family carries its mandatory **one-sentence `reason`** — the oracle,
   since there is no external truth to appeal to (D19; `Trap::validate`, 20–300 chars, single line). Each
   reason states plainly WHY the column is correct in the family's own vocabulary (two virtual NICs of one
   VM / two VMs co-tenant on one hypervisor / a hostless virtual interface behind a shared uplink), so a
   later reader can disagree on the record.

5. **And** the three traps all declare `family = "shared-hardware-vm"`, so the corpus-completeness check
   (story 4.7b, `incomplete_families`) sees the family present in **BOTH decision poles** (≥1 `must-merge`
   AND ≥1 `must-not-merge`) and the gate stays green — the `must-abstain` is grouped with the family but
   satisfies **NEITHER** pole (DR1: it is D18's orthogonal third column, not a decision form). A family that
   were only `must-abstain`, or only one pole, would red `passed()` on its own.

6. **And** the family is committed as **two NEW locked corpus artefacts** —
   `fixtures/scenario/replay/shared-hardware-vm.jsonl` (the four presences it judges) and
   `fixtures/scenario/traps/shared-hardware-vm.toml` (the three traps) — **both added to
   `fixtures/MANIFEST.toml` with their sha256** in the same commit (a **deliberate corpus bump**, exactly
   like stories 4.9 and 4.10). After this story `cargo xtask ci`'s `fixtures` gate reports **11 artefacts,
   all sha256 match, no orphan** (9 existing + 2 new). Omitting either from the manifest is an ORPHAN
   finding (red); a wrong sha256 is an EDITED finding (red).

7. **And** the committed replay stream carries **synthetic values only** — locally-administered MACs (byte
   0 has the 0x02 bit set, `locally_administered: true`, INCLUDING every `Uplink.peer_mac`), RFC 5737
   documentation IPs (`192.0.2.0/24`), and **hostnames that begin `doc-`** — so the existing privacy walk
   `the_corpus_carries_no_real_network_data` passes over the new stream unchanged (D19;
   [[no-private-data-in-artifacts]]). `assert_facts_are_synthetic` (fixtures.rs:867-913) checks each
   `Hostname.name` with `name.starts_with("doc-")`, each `Mac`/`Uplink.peer_mac` for the U/L bit, and each
   `IpV4`/`DhcpLease` for an RFC 5737 range; a real vendor MAC, a non-5737 IP, or a hostname not prefixed
   `doc-` reds the walk.

8. **And** the three committed-corpus assertions that hard-code the trap count are updated from **7 to 10**
   in the same change (`example.toml`'s 3 + `randomized-mac.toml`'s 2 + `multi-nic.toml`'s 2 + this
   family's **3**): `report.discovered()` at `trap_gate.rs:390` and `:426`, and the `"7 trap(s) discovered"`
   render string at `:408`. The scratch-corpus counts elsewhere in `trap_gate.rs`
   (`each_column_can_be_driven_red`, `a_one_sided_family_reddens_the_gate_on_its_own`, and the other scratch
   tests) are **NOT** touched — they assert their own scratch dirs, not the committed root. At v0.1
   `scored()` and `failures()` stay **0** for the whole committed corpus: no answer producer exists (Epic 5),
   so the family is discovered, parsed and validated (obs_ids resolve, reasons present, family complete in
   both poles) but scored by nothing.

## Tasks / Subtasks

> **⚠️ ONE REVIEW DECISION IS BAKED IN — read the "Rule vocabulary" Dev Note before writing the `.toml`.**
> This family CANNOT reuse `l2-uplink-agrees`/`l2-different-switch`: the VMs share the hypervisor's uplink,
> so an uplink rule would assert the exact false merge the family exists to refuse. It therefore COINS two
> hostname-based L2 rule ids — `l2-hostname-agrees` (merge) and `l2-different-hostname` (oppose) —
> architecturally sanctioned (hostname is the named L2 signal, architecture.md:891) and recorded as the
> Epic-5 contract. If a reviewer prefers different names or a different framing, raise it in review; do NOT
> silently reframe the family onto different hypervisors (that would duplicate multi-nic and drop the
> shared-hardware point).

- [x] **Task 1 — write the replay stream** `fixtures/scenario/replay/shared-hardware-vm.jsonl` (AC: 1, 2, 3, 7) — pure data, no code
  - [x] Author **four observations** in one stream, JSONL, matching the EXACT shape of `multi-nic.jsonl`
        (fields in order: `obs_id`, `connector_id`, `observed_at`, `scope` {`l2_domain`, `vantage`},
        `facts`, `raw`). Copy `multi-nic.jsonl`'s first line as the structural template (see the exact line
        quoted in the References). Use a **fresh `obs_id` prefix `abababab`** — the cross-stream rule forbids
        one `obs_id` appearing in two committed streams (`fixtures.rs`
        `no_obs_id_is_shared_across_replay_streams`, :1306), and `aaaa…`/`bbbb…`/`cccc…`/`dddd…`/`eeee…`/`ffff…`
        are ALL taken by the six existing committed streams (verified: one prefix per stream). **`abababab`
        is free** in every committed stream. **Pin the four obs_ids as full valid UUIDs** and reuse the SAME
        spelling in the traps' `observations` lists (a non-UUID reds `ObsId`'s parse; a stream↔trap spelling
        mismatch reds `read_traps` with `DanglingObservation`). **Before committing, `grep -r abababab
        fixtures/scenario/replay/` and confirm the ONLY hits are this new file.**
        - W1 = `abababab-0000-4000-8000-000000000001`
        - W2 = `abababab-0000-4000-8000-000000000002`
        - W3 = `abababab-0000-4000-8000-000000000003`
        - W4 = `abababab-0000-4000-8000-000000000004`
  - [x] Keep `connector_id`, `l2_domain` and `vantage` **identical across all four** (the within-stream
        provenance/scope checks, stories 4.5a/4.5b): reuse the synthetic UUIDs already in the corpus —
        `connector_id` `33333333-3333-4333-8333-333333333333`, `l2_domain`
        `11111111-1111-4111-8111-111111111111`, `vantage` `22222222-2222-4222-8222-222222222222`.
  - [x] **The shared-hardware signal is a SHARED `Uplink` fact (identical across all four) + a HOSTNAME
        discriminator.** All four observations carry the SAME `Uplink.peer_mac` **and** the same `peer_port`
        — the hypervisor's single physical port into one switch, through which every VM's traffic egresses.
        `Uplink { peer_mac: MacAddr, peer_port: String }` serializes exactly as in `multi-nic.jsonl`:
        `{"Uplink":{"peer_mac":[2,0,94,0,96,10],"peer_port":"swport-1"}}` — **identical bytes on all four
        lines** (this is the point: shared hardware tempts the merge). `peer_mac` MUST be
        locally-administered (byte 0 = `0x02`); `peer_port` is a free `String`, keep it innocuous.
  - [x] **The `Hostname` fact carries the discriminator** — `Fact::Hostname { name: String, source:
        HostnameSource }` (`observation/mod.rs:154-158, 130-136`). It is ALREADY used in committed streams,
        so copy the exact shape: `{"Hostname":{"name":"doc-vm-alpha","source":"Dhcp"}}`
        (`HostnameSource` serializes PascalCase: `Dhcp`/`Dns`/`Mdns`/`Netbios`/`Other`; `Dhcp` matches the
        other committed streams). **Every `name` MUST begin `doc-`** — `assert_facts_are_synthetic` asserts
        `name.starts_with("doc-")` (fixtures.rs:879), so a hostname without that prefix reds the privacy
        walk regardless of plausibility.
  - [x] **W1 and W2 — one VM's two virtual NICs (feed the `must-merge`, AC1).** Two DIFFERENT
        locally-administered MACs, the **SAME hostname** `doc-vm-alpha`, the SAME shared `Uplink`. Facts per
        observation, in this order: one `Mac`, one `IpV4`, one `Hostname`, one `Uplink`.
        - W1: `{"Mac":{"addr":[2,0,94,0,83,80],"locally_administered":true}}` (`02:00:5e:00:53:50`),
          `{"IpV4":{"addr":"192.0.2.80"}}`, `{"Hostname":{"name":"doc-vm-alpha","source":"Dhcp"}}`,
          `{"Uplink":{"peer_mac":[2,0,94,0,96,10],"peer_port":"swport-1"}}`.
        - W2: `{"Mac":{"addr":[2,0,94,0,83,81],"locally_administered":true}}` (`02:00:5e:00:53:51`),
          `{"IpV4":{"addr":"192.0.2.81"}}`, `{"Hostname":{"name":"doc-vm-alpha","source":"Dhcp"}}`,
          `{"Uplink":{"peer_mac":[2,0,94,0,96,10],"peer_port":"swport-1"}}`.
  - [x] **W3 — a different co-tenant VM (feeds the `must-not-merge`, AC2).** A DIFFERENT
        locally-administered MAC, a DIFFERENT hostname `doc-vm-beta`, the SAME shared `Uplink` (identical
        `peer_mac` AND `peer_port` as W1/W2 — same hypervisor):
        `{"Mac":{"addr":[2,0,94,0,83,82],"locally_administered":true}}` (`02:00:5e:00:53:52`),
        `{"IpV4":{"addr":"192.0.2.82"}}`, `{"Hostname":{"name":"doc-vm-beta","source":"Dhcp"}}`,
        `{"Uplink":{"peer_mac":[2,0,94,0,96,10],"peer_port":"swport-1"}}`.
  - [x] **W4 — the hostless virtual interface (feeds the `must-abstain`, AC3).** A DIFFERENT
        locally-administered MAC, the SAME shared `Uplink`, and **NO `Hostname` fact at all** — the
        discriminator is absent, so the grouping question has no answer. Facts: one `Mac`, one `IpV4`, one
        `Uplink` (NO hostname):
        `{"Mac":{"addr":[2,0,94,0,83,83],"locally_administered":true}}` (`02:00:5e:00:53:53`),
        `{"IpV4":{"addr":"192.0.2.83"}}`, `{"Uplink":{"peer_mac":[2,0,94,0,96,10],"peer_port":"swport-1"}}`.
  - [x] **`observed_at` — distinct, plausible RFC3339 timestamps.** No ordering guard applies to plain
        observations (`CapabilityOutOfOrder` governs only capability records, of which this stream has none),
        so any valid values parse. Give the two NICs of one VM (W1/W2) the SAME scan time and W3/W4 later
        ones, e.g. W1/W2 = `2026-01-04T00:00:00Z`, W3 = `2026-01-04T00:05:00Z`, W4 = `2026-01-04T00:10:00Z`.
        Timestamps carry no assertion in this family — the hostname (and its absence) does.
  - [x] **`raw` is `null`** on every line (as in every committed stream). **Synthetic-only, non-negotiable
        (AC7):** every MAC (host AND `Uplink.peer_mac`) locally-administered; every IP `192.0.2.x`; every
        hostname `doc-*`. A real vendor MAC, a non-5737 IP, or a non-`doc-` hostname reds
        `the_corpus_carries_no_real_network_data`. Never a real capture (D19).

- [x] **Task 2 — write the family trap file** `fixtures/scenario/traps/shared-hardware-vm.toml` (AC: 1, 2, 3, 4, 5) — pure data, no code
  - [x] Open with a short header in the voice of `multi-nic.toml`/`randomized-mac.toml`: *the shared-hardware
        VM family — several VMs on one hypervisor share the physical uplink, so shared hardware TEMPTS a
        false merge while the VMs are genuinely different logical hosts (architecture.md:891, D12: L2 groups
        by hostname/topology/DHCP, attacked by shared-hardware VMs). This L2 family is distinguished by
        HOSTNAME, not uplink — the uplink is shared by construction. All three traps below judge
        `scenario/replay/shared-hardware-vm.jsonl`; together they say: a shared physical host is no reason to
        fuse distinct logical hosts, and a distinct virtual MAC is no reason to split one VM.*
  - [x] **The `must-merge` (primary form, AC1).** Judges `[W1, W2]`. `family = "shared-hardware-vm"`.
        `expect = { must-merge = { rule = "l2-hostname-agrees" } }`. `reason` (one sentence, names the L2
        rule that FIRES on the agreeing hostname): *"these two virtual interfaces are one VM's NICs — their
        locally-administered MACs differ but they share the hostname doc-vm-alpha, so L2 groups them by the
        agreeing hostname, and the hypervisor's shared uplink is not what makes them one device."*
  - [x] **The `must-not-merge` (inverse form, AC2).** Judges `[W1, W3]`. `family = "shared-hardware-vm"`.
        `expect = { must-not-merge = { rule = "l2-different-hostname" } }`. `reason` (one sentence, names the
        OPPOSING rule): *"these two virtual interfaces are different VMs co-tenant on one hypervisor — they
        share the physical uplink but report distinct hostnames doc-vm-alpha and doc-vm-beta, so the distinct
        hostnames oppose the L2 join and grouping them by shared hardware would be a false merge."*
  - [x] **The `must-abstain` (ambiguous edge, AC3).** Judges `[W1, W4]`. `family = "shared-hardware-vm"`.
        `expect = { must-abstain = { cause = "NoObservedValue" } }` — `cause` is an [`AbstentionCause`]
        variant, serialized PascalCase as a bare string EXACTLY like `example.toml`'s
        `{ must-abstain = { cause = "NoObservedValue" } }` (the established precedent — copy that spelling).
        `reason` (one sentence, states why the signal is insufficient — **do NOT use the story-internal
        "W4" label in the committed record; name the interface plainly**): *"this hostless virtual interface
        shares only the hypervisor's uplink with doc-vm-alpha and reports no hostname, so no observed value
        distinguishes a second NIC of that VM from a new co-tenant VM, and the engine must abstain rather
        than guess."*
  - [x] **Rule ids — COINED, deliberately (read the "Rule vocabulary" Dev Note).** Use `l2-hostname-agrees`
        (the merge rule) and `l2-different-hostname` (the opposing rule). These are NEW to the corpus — no
        sibling `.toml` to pattern-copy from — because `score.rs` only models the uplink pair
        (`l2-uplink-agrees`/`l2-different-switch`, `score.rs:620,625,…`), which this family cannot use (the
        uplink is shared). The names mirror the existing pair's convention (`l2-<signal>-agrees` for the
        merge; `l2-different-<signal>` for the opposition, as in `l2-different-switch`). At v0.1 nothing
        validates rule-id membership (`RuleId` is a `String`, trap.rs:31-38), so a typo ships silently now
        and becomes an unhonourable `Other(String)` when Epic 5 closes `RuleId` into an enum — **type the two
        ids carefully and consistently between the `.toml` and this story.** Do NOT use an L1 id or an
        `l2-uplink-*` id here.
  - [x] **Pick trap ids unique across the whole corpus:** `shared-hardware-vm-must-merge`,
        `shared-hardware-vm-must-not-merge`, `shared-hardware-vm-must-abstain` (`example.toml` uses
        `example-must-*`, `randomized-mac.toml` `randomized-mac-must-*`, `multi-nic.toml` `multi-nic-must-*`
        — no clash; `read_traps` reds a `TrapId` seen in two files, case-folded and trimmed).

- [x] **Task 3 — lock the two new artefacts** `fixtures/MANIFEST.toml` (AC: 6) — deliberate corpus bump
  - [x] Append TWO `[[artefact]]` entries (paths `scenario/replay/shared-hardware-vm.jsonl` and
        `scenario/traps/shared-hardware-vm.toml`), each with a one-line comment naming the story and what the
        artefact is, mirroring the 4.9/4.10 entries' style. Compute each `sha256` from the committed bytes —
        `sha256sum fixtures/scenario/replay/shared-hardware-vm.jsonl` and `…/traps/shared-hardware-vm.toml`
        — and paste the hex. **Author Task 1/Task 2 FIRST, then hash** — any later byte edit (even a trailing
        newline) invalidates the sha256, which reds the gate with a confusing "edited" finding on a brand-new
        file.
  - [x] Do NOT touch the nine existing entries or their sha256 (`example.toml`, `randomized-mac.toml`,
        `multi-nic.toml`, and the six streams stay byte-for-byte — any edit there is an unrelated EDITED
        finding).

- [x] **Task 4 — update the committed-count assertions** `crates/opencmdb-bin/src/trap_gate.rs` (AC: 8) — three edits, tests only
  - [x] `the_committed_corpus_is_discovered_and_scored_by_nothing` (`:390`): `report.discovered()` `7` → `10`.
        Keep `scored()` and `failures()` at `0` (no producer exists). Update the comment above it (`:389`, "…
        seven in the committed corpus") that names
        the corpus breakdown ("… `multi-nic.toml` (story 4.10) two more — seven in the committed corpus") to
        add shared-hardware-vm's **three** and say **ten**.
  - [x] `the_report_says_plainly_that_nothing_was_scored` (`:408`): `"7 trap(s) discovered"` →
        `"10 trap(s) discovered"`. Leave `"0 scored"` and `"0 truth-table failure(s)"` unchanged.
  - [x] `a_trap_with_no_answer_is_discovered_but_not_scored` (`:426`): `report.discovered()` `7` → `10`
        (still answers ONE trap — `example-must-abstain` — so `scored()` stays `1`; the other nine are
        unanswered). **Also fix the stale comment inside the SAME test at `:418`** — "so `scored` is 1 while
        `discovered` stays 7" → `stays 10`; editing only the `:426` assertion would leave `:418` lying.
  - [x] **Do NOT touch the scratch-corpus counts** — `each_column_can_be_driven_red`,
        `a_one_sided_family_reddens_the_gate_on_its_own`, and the other scratch tests build their OWN
        temp-dir corpora; their counts are unrelated to the committed root. Changing them would be wrong.
  - [x] **Prove the count coupling is real (record it).** Before adding the family to the manifest, run
        `cargo test -p opencmdb-bin --locked the_committed_corpus_is_discovered_and_scored_by_nothing` with
        the assertions still at `7` and `shared-hardware-vm.toml` on disk: it should RED with `left: 10,
        right: 7` (the walk discovers 10), confirming the family is genuinely discovered on the live corpus
        root — AND that all three traps parsed, validated, and resolved their obs_ids against the committed
        stream before the count was even checked. Then update to `10`. Record this in the Debug Log. (No new
        test needed: `discover_trap_files` walks `.toml` under `scenario/traps/` automatically.)

- [x] **Task 5 — gates and the corpus lock** (AC: 5, 6, 7, 8)
  - [x] `cargo fmt --all` · `cargo clippy --workspace --all-targets --locked -- -D warnings` ·
        `cargo test --workspace --locked` · `cargo run -p xtask --locked -- ci` — all green.
  - [x] `cargo xtask ci`'s `fixtures` gate must report **11 artefacts, all sha256 match, and no orphan
        finding** (9 existing + 2 new). Confirm the new stream and trap file are BOTH listed and BOTH match.
        `file-size` gate: `.toml`/`.jsonl` are not counted (the gate measures `.rs` before the first
        `#[cfg(test)]`); `trap_gate.rs` grows only by three literals inside `#[cfg(test)]`.
  - [x] Confirm the whole committed corpus still `passed()` at v0.1: `discovered() == 10`, `scored() == 0`,
        `failures() == 0`, `rule_mismatches` empty, `incomplete_families` empty (randomized-mac, multi-nic
        AND shared-hardware-vm each carry BOTH poles → all complete; the shared-hardware-vm `must-abstain`
        counts for neither pole but does not make it incomplete). If any test that renders the report
        elsewhere hard-codes `7`, catch it here (grep `"7 trap"` / `discovered(), 7` across `crates/`).
  - [x] `Cargo.lock` unchanged (no dependency touched — two data files, one manifest bump, three test
        literals). `architecture-views.md` is NOT regenerated in this story (`ℹ views-hash STALE` is by
        design — regenerate at the Epic-4 milestone, not here). `example.toml`, `randomized-mac.toml`,
        `multi-nic.toml` and the six existing streams are byte-for-byte unchanged (measured via
        `git status --short`).

### Review Findings

- [x] [Review][Patch] The coined id `l2-different-hostname` is hyphen-broken across two comment lines in the trap-file header (`l2-different-` / `hostname`), so a grep on the id misses the very prose that coins it — rewrap the sentence so the id stays whole (byte change → re-hash + MANIFEST bump) [fixtures/scenario/traps/shared-hardware-vm.toml:10-11] — **FIXED**: header rewrapped, id whole at :11 (grep-verified); file re-hashed `47055cb7…54a4e6`, MANIFEST updated, `fixtures` gate green (11 match).
- [x] [Review][Patch] `fixtures/scenario/traps/README.md:8` still says "today only `example.toml` lives here" while the directory now holds four `.toml` files — pre-existing (4.9/4.10 missed it) but docs-current-before-push makes a doc contradicting the corpus a push blocker; README is orphan-exempt so no re-hash [fixtures/scenario/traps/README.md:8] — **FIXED**: sentence now reads "each in its own `.toml` beside `example.toml`" (stays true as families accumulate; README orphan-exempt, no manifest change).

## Dev Notes

### The shape of this story in one paragraph

The shared-hardware VM family, and — like 4.9 and 4.10 — it is almost entirely DATA. Two new committed
artefacts (a **four**-observation replay stream and a **three**-trap family file), a deliberate
`MANIFEST.toml` bump to lock them, and three test-literal updates to keep the committed-corpus count honest
(**7 → 10**). There is **no engine and no new harness code**: at v0.1 the corpus is discovered, parsed and
validated but scored by nothing — the `(verdict, rule)` scoring arrives in Epic 5. Two things are genuinely
NEW versus 4.9/4.10: (1) it is the **first three-column family** — merge + not-merge + `must-abstain` of ONE
scenario, exercising DR1's explicit allowance that an abstain trap MAY carry a family; and (2) it is the
first family whose **discriminator is the hostname, not the topology** — the hypervisor's uplink is SHARED
by every VM, so the family must state the answer on the hostname signal (architecture.md:891), which forces
the two coined L2 rule ids below.

### Rule vocabulary — this family COINS two L2 rule ids, and it must (AC1/AC2, the review decision)

**Read this before writing the `.toml`.** Stories 4.9 and 4.10 both said "reuse the rule ids, do not coin"
— because the rules they needed (`l1-exact-mac`/`l1-distinct-mac`, `l2-uplink-agrees`/`l2-different-switch`)
**already existed** in `score.rs`'s vocabulary. Shared-hardware VM is the first family where that is NOT
true, and the reason is structural, not an oversight:

- A VM cluster's whole signature is a **shared physical uplink**: every VM on one hypervisor egresses
  through the same physical port, so all four observations carry an **identical `Uplink`** (same `peer_mac`,
  same `peer_port`). The topology signal is therefore the SAME for two NICs of one VM (W1/W2) AND for two
  different VMs (W1/W3) — it cannot distinguish them.
- Using `l2-uplink-agrees` as the merge rule would assert *"the shared uplink groups them"* — which is
  **exactly the false merge this family exists to refuse** (it would also merge W1 and W3, two different
  VMs). Using `l2-different-switch` as the opposing rule would be a flat lie: the switch/port is the SAME.
- The discriminator that actually decides is the **hostname** — the architecture's named L2 signal
  (architecture.md:891: L2 main signals are *"hostname, topology, DHCP"*; D12 lists *"shared-hardware VMs"*
  among the attacks on L2 grouping). So the family COINS:
  - `l2-hostname-agrees` — the `must-merge` rule that FIRES (W1/W2 share `doc-vm-alpha`).
  - `l2-different-hostname` — the `must-not-merge` rule that OPPOSES (W1/W3 report distinct hostnames).
  The names **mirror the existing pair's convention** (`l2-uplink-agrees` → `l2-hostname-agrees`;
  `l2-different-switch` → `l2-different-hostname`), so the vocabulary reads consistently and the later
  hostname-collision family (story 4.15) can reuse it.
- **Why coining is safe here.** `RuleId` is a plain `String` at v0.1 (trap.rs:31-38) and the committed
  corpus SCORES NOTHING (no engine), so a rule id is a **recorded Epic-5 contract**, exactly as 4.10's
  `l2-uplink-agrees` was. Coining `l2-hostname-agrees`/`l2-different-hostname` records *"Epic 5's L2 cascade
  will have hostname-agreement and hostname-conflict rules"* — which the architecture already predicts. When
  Epic 5 closes `RuleId` into an enum (architecture.md:2652: *a decision on every variant*), these two ids
  must be honoured with real rules; that is the contract, and it is faithful.
- **This is a REVIEW DECISION, surfaced not hidden.** If a reviewer prefers different names
  (`l2-hostname-match`/`l2-hostname-conflict`, say) or disputes coining at all, raise it in review — the
  choice is documented here and in the story's closing questions. **Do NOT** "resolve" it by reframing the
  family onto DIFFERENT hypervisors (different uplinks) to reuse `l2-uplink-agrees`/`l2-different-switch`:
  that drops the shared-hardware essence entirely and merely duplicates multi-nic (4.10).

### The three columns, at L2 — and which observations feed which (AC1/AC2/AC3)

| Column | Rule / cause | Judges | The point |
|---|---|---|---|
| **`must-merge`** (primary) | `l2-hostname-agrees` (the L2 rule that FIRES) | `[W1, W2]` — same hostname `doc-vm-alpha`, shared uplink, different vMACs | L2 groups one VM's two NICs by the agreeing hostname; a distinct virtual MAC is no reason to SPLIT one VM |
| **`must-not-merge`** (inverse) | `l2-different-hostname` (the OPPOSING L2 rule) | `[W1, W3]` — distinct hostnames (`doc-vm-alpha` / `doc-vm-beta`), SAME shared uplink | shared hardware TEMPTS the merge; distinct hostnames oppose it — fusing two VMs is a false MERGE |
| **`must-abstain`** (ambiguous edge) | `cause = NoObservedValue` | `[W1, W4]` — W4 has NO hostname, only the shared uplink | the discriminator is absent; the engine must ABSTAIN, not guess (FR16) |

D18's three columns (trap.rs:55-76): `must-not-merge` guards the false merge (operator loses trust,
uninstalls), `must-merge` guards **cowardice / the false split** (an engine that abstains on everything
scores false-merge = 0 and is demolished by the middle column), `must-abstain` guards **guessing on the
honestly ambiguous case** (FR16). This family is the first in the corpus to exercise ALL THREE at once —
the merge and not-merge poles keep the family complete (AC5), and the abstain is the honestly-insufficient
edge of the SAME scenario that DR1 explicitly welcomes.

### Why the `must-abstain` cause is `NoObservedValue` (and not `ConflictingObservations`)

`AbstentionCause` (gap/mod.rs:24-34) has three variants: `OutOfPerimeter`, `NoObservedValue`,
`ConflictingObservations`. The W4 case is **absence of the discriminating signal**, not a conflict: W4
reports no hostname at all, so nothing contradicts `doc-vm-alpha` — there is simply no value to decide on.
That is `NoObservedValue`, and it is the SAME cause `example.toml`'s `example-must-abstain` uses for its
*"no identifying fact supports any conclusion"* case (the established precedent — reuse its spelling
verbatim). `ConflictingObservations` would be wrong: it means *"two in-perimeter observations disagree on a
field"*, and W4 disagrees with nothing. (If a reviewer reads the ambiguity as a genuine conflict — e.g. by
making W4 carry a THIRD hostname — that is a different trap; this story deliberately keeps the ambiguity as
pure absence, the cleanest honest-insufficiency case.)

### Why shared-hardware VM is an L2 problem, and the "one VM" is the author's oracle (AC1/AC4)

- The identity model splits into two layers (architecture.md:888-895, D12): **L1 = interface identity**
  (main signal MAC, attacked by MAC randomization / cloned MAC / Docker veth) and **L2 = device grouping**
  ("are these interfaces the same host?", main signals **hostname / topology / DHCP**, attacked by multi-NIC
  / **shared-hardware VMs** / VRRP/HSRP). Shared-hardware VM sits squarely in L2, on the hostname signal.
- Per D19 and the 4.9/4.10 precedent, the ground truth — "W1 and W2 are ONE VM", "W1 and W3 are DIFFERENT
  VMs" — is the author's ORACLE, stated in the `reason`, not a fact the bytes alone prove. The shared
  hostname is the plausible carrier for the merge; the distinct hostnames are the decisive negative for the
  refusal; the ABSENT hostname is the honest insufficiency for the abstain. No single L2 signal decides by
  fiat (D13: L2 is a rule cascade, *"floats may RANK, never DECIDE"*) — which is exactly why the corpus
  states the answer + reason and Epic 5/6 build the rule that reaches it.
- **The asymmetry is the same shape as multi-nic's, named so a reviewer sees it was a choice.** The
  `must-not-merge` pole is byte-aligned (distinct hostnames directly carried by the bytes — a decisive
  negative). The `must-merge` pole rests on the author's "one VM" oracle: two observations sharing a
  hostname is *strong* evidence but D13 keeps hostname a RANKING signal, not a decider, so the "one device"
  claim is the reason's, stated on the record. This is not a weakness — it is the L2 problem's nature (L2 is
  the same record-linkage problem as L1 but on the HARD signals; a false split/merge is born precisely
  because a single signal under-determines grouping).

### At v0.1 there is no engine — what "scored" means here (AC8)

`score_corpus` (`trap_gate.rs`) discovers `.toml` files, reads and validates each trap through `read_traps`,
and scores each trap **only if `answers` carries an `Outcome` for its `TrapId`**. At v0.1 the answers map is
empty for a real run (no producer — Epic 5 builds it), so:

- `discovered()` counts every trap the walk opened → **10** after this story (3 in `example.toml` + 2 in
  `randomized-mac.toml` + 2 in `multi-nic.toml` + **3** in `shared-hardware-vm.toml`).
- `scored()` and `failures()` stay **0** for the committed corpus.
- `incomplete_families()` runs answer-INDEPENDENTLY over corpus SHAPE (4.7b): shared-hardware-vm carries
  both poles (the abstain is neutral, DR1), so it is complete and the bucket stays empty; `passed()` stays
  green.

The three committed-count assertions (`trap_gate.rs:390`, `:408`, `:426` — line numbers may drift, the
function names in Task 4 are authoritative) are the only tests coupled to the count — they exist precisely
so the zeros stay HONEST. Updating `7 → 10` is mandatory and is itself the
evidence the family was discovered. The scratch-corpus tests build their own temp dirs and must NOT be
touched.

### How a trap is validated (so the family stays green) — the checks the new files must survive

`read_traps` / `Trap::validate` (trap.rs:250-315) enforce, per trap:
- **obs_id resolution** — every id in `observations` must exist in the trap's named `replay` stream, or
  `read_traps` fails naming the trap and the missing observation. So `shared-hardware-vm.toml`'s
  `replay = "scenario/replay/shared-hardware-vm.jsonl"` and every `obs_id` it lists (W1/W2 for must-merge;
  W1/W3 for must-not-merge; W1/W4 for must-abstain) must be in that committed stream. `read_traps` resolves
  `replay` against the BAKED corpus root, so the stream must be a committed file (it is, once Task 1 lands).
- **`reason` non-empty, 20–300 chars, single line, no control char** (`trap.rs:258-312`). All three
  prescribed reasons are single-line and inside the bound — **measure and record the exact char counts**
  (they read ~200–260; if any exceeds 300, tighten it, never split it into two sentences).
- **a decision names a rule; an abstention names a cause** — the `Expectation` enum makes "a merge that
  also names an abstention cause" unrepresentable, and `Trap::validate` reds a decision whose `rule` is
  blank (trap.rs:288-292). `must-abstain` carries a `cause`, no `rule` — the type enforces it.
- **unique trap ids across files** (`DuplicateTrapId`) — the three `shared-hardware-vm-must-*` ids are
  unique in the whole corpus.
- **family present + clean token** (`FamilyEmpty`/`FamilyMalformed`) — `"shared-hardware-vm"` is a clean
  token (internal hyphens are fine; the guard refuses surrounding whitespace and control chars,
  trap.rs:300-310).
- **family completeness** — `incomplete_families` groups by case-folded family name and requires ≥1
  `must-merge` AND ≥1 `must-not-merge` (DR1: a `must-abstain` satisfies neither pole). The merge + not-merge
  traps satisfy this; the abstain rides along.

And `the_corpus_carries_no_real_network_data` / `every_trap_file_in_the_corpus_is_valid` /
`every_replay_stream_in_the_corpus_is_valid` walk the new files automatically — no harness change; a
malformed file reds the walk by name.

### The corpus lock is a DELIBERATE bump (AC6) — same mechanism as stories 4.9/4.10

Two real artefacts that MUST be locked. The `fixtures` gate is two-directional (MANIFEST.toml header): an
artefact under `fixtures/` absent from the manifest is an ORPHAN (red); a listed artefact whose bytes
changed is EDITED (red). Adding the two entries with correct sha256 turns both green. This is an "I am
changing the spec" commit by design — the review SHOULD see the manifest move. Order of operations: **write
the data files, THEN hash, THEN paste the sha256.** READMEs stay exempt (do NOT add any README to the
manifest — orphan-exempt by name).

### Project Structure Notes

- **NEW (locked):** `fixtures/scenario/replay/shared-hardware-vm.jsonl` (4 observations, first family to use
  `Hostname` as the L2 discriminator with a shared `Uplink`), `fixtures/scenario/traps/shared-hardware-vm.toml`
  (3 traps, `family = "shared-hardware-vm"`, first three-column family). Both listed in
  `fixtures/MANIFEST.toml` with sha256.
- **Updated:** `fixtures/MANIFEST.toml` (two `[[artefact]]` entries — the deliberate bump);
  `crates/opencmdb-bin/src/trap_gate.rs` (three `#[cfg(test)]` count literals, 7 → 10, plus the adjacent
  comments — tests only, no production logic).
- **Unchanged, expected:** `example.toml`, `randomized-mac.toml`, `multi-nic.toml` and the six existing
  replay streams (byte-for-byte — any edit is an unrelated EDITED finding); `trap.rs` / `score.rs` /
  `gap/mod.rs` / `trap_gate.rs` production paths (domain and harness frozen since 4.6a/4.7a/4.7b — this
  story adds no rule, no scoring, no engine); all READMEs (orphan-exempt); `Cargo.lock`; every other `.rs`.
  `discover_trap_files` / `incomplete_families` / `read_traps` are used AS-IS.
- **Out of scope, deliberately:** any engine or rule producer, and the real hostname/topology rule that
  `l2-hostname-agrees`/`l2-different-hostname` name (Epic 5); the actual L2 grouping algorithm and its
  blocking key (Epic 6+); a fourth trap or a second abstain (the family is complete with three);
  `docs/project-context.md` and `CLAUDE.md` (a new locked family is milestone/push-level under
  docs-current-before-push — fold in at the Epic-4 milestone with the `architecture-views.md`
  regeneration, not per-story); the cloned/spoofed-MAC family (story 4.12) and every later family.

### Traps (mistakes this story must not make)

1. **Reusing `l2-uplink-agrees` / `l2-different-switch`.** The uplink is SHARED across every VM — an uplink
   rule asserts the exact false merge the family refuses. Use the coined hostname rules
   `l2-hostname-agrees` / `l2-different-hostname` (see the Rule-vocabulary Dev Note). Do NOT use an L1 id.
2. **Making the `Uplink` differ between poles.** Unlike multi-nic (where the switch DIFFERED for the
   not-merge), here the `Uplink` is IDENTICAL on ALL FOUR observations (same hypervisor). The discriminator
   is the hostname, not the uplink — that is the whole point. Same `peer_mac` AND same `peer_port` on every
   line.
3. **A hostname not prefixed `doc-`.** `assert_facts_are_synthetic` asserts `name.starts_with("doc-")`
   (fixtures.rs:879). `doc-vm-alpha` / `doc-vm-beta` pass; `vm-alpha` reds the privacy walk.
4. **Giving W4 a hostname.** W4 is the abstain case precisely BECAUSE it has no `Hostname` fact. Adding one
   would collapse the ambiguity and break AC3. W4 carries `Mac` + `IpV4` + shared `Uplink` only.
5. **Using `ConflictingObservations` for the abstain cause.** Nothing conflicts — the discriminator is
   ABSENT. Use `NoObservedValue`, `example.toml`'s established cause for "no identifying fact." (Review
   decision — documented.)
6. **A real MAC in an `Uplink` or `Mac`.** Both are privacy-checked (`fixtures.rs:874-877`) — every MAC MUST
   be locally-administered, byte 0 = `0x02`.
7. **Forgetting the manifest bump.** The two new files are LOCKED artefacts. Both need a `[[artefact]]`
   entry with the correct sha256, or the `fixtures` gate reds as an orphan.
8. **Hashing before the final byte.** Compute sha256 only after the files are final; a later trailing
   newline invalidates it.
9. **Leaving the count assertions at 7.** Three committed-corpus tests hard-code `7`; the family (THREE
   traps) makes it `10`. Update `trap_gate.rs:390/:408/:426` (and their comments) — and do NOT touch the
   scratch counts. Note this family adds THREE, not two (it has an abstain).
10. **Reusing an existing stream's `obs_id` prefix.** New stream → fresh ids (`abababab-…`); aaaa–ffff are
    taken by the six committed streams. Grep to confirm `abababab` is unused before committing.
11. **Recording a real hostname/IP.** Synthetic-only: locally-administered MACs, RFC 5737 IPs, `doc-*`
    hostnames. The privacy walk enforces it; a real capture is disqualifying (D19;
    [[no-private-data-in-artifacts]]).
12. **Inconsistent `connector_id`/`scope` within the stream.** All four observations share one
    `connector_id`, `l2_domain` and `vantage` (the within-stream provenance/scope checks, 4.5a/4.5b).
13. **Regenerating `architecture-views.md`.** Its `ℹ views-hash STALE` is by design — Epic-4 milestone, not
    this story.
14. **Claiming more than measured.** Name the command behind every count; record the discovery-count
    coupling proved (10 discovered) and the measured reason char counts. Write the weaker true sentence.
    [[claims-must-match-verification]]

### Latest technical specifics

No new crate, no version bump, no domain or harness code. Rust 1.96+, edition 2024. Two data files (one
`.jsonl` — first family to use `Hostname` as the L2 discriminator alongside a shared `Uplink` — one
`.toml`), one `MANIFEST.toml` bump, three `#[cfg(test)]` literals in `opencmdb-bin`. **Never invent a
version — pin from the committed `Cargo.lock`, which does not move here.**

### References

- [Source: _bmad-output/planning-artifacts/epics.md:1121-1132 — Story 4.11 "Trap family — shared-hardware
  VM": the two ACs (several virtual interfaces on one physical host → state explicitly which grouping is
  correct and why in one sentence; the ambiguous variant labelled `must-abstain` rather than argued) and the
  story sentence "neither fused nor split wrongly"]
- [Source: _bmad-output/planning-artifacts/architecture.md:888-908 — D12 the three-level model
  `observation → interface → device`; the L1/L2 table (:888-891) naming **hostname/topology/DHCP** as L2
  signals and **shared-hardware VMs** as an L2 attack; "the product is dead" economic test (:906-908)]
- [Source: _bmad-output/planning-artifacts/architecture.md:929-1002 — D13: L2 = the FORM of C with A's
  decision function, "named rules"; the verdict algebra (Decisive/Supports/Neutral/Opposes/Disqualifying),
  "abstention EMERGES from conflict"; "floats may RANK, never DECIDE"]
- [Source: _bmad-output/planning-artifacts/architecture.md:1208-1266 — D18: the three columns table
  (`must-not-merge` / `must-merge` / `must-abstain`, :1232-1234); the honest-vs-cowardly distinction
  (:1241-1249) — abstain because the signal is absent is honest and gated only as measurement, not failure]
- [Source: crates/opencmdb-core/src/trap.rs:55-124, 258-315 — `Expectation` (the three columns; `MustMerge
  {rule}` / `MustNotMerge {rule}` / `MustAbstain {cause}`, the "name the OPPOSING rule" contract at :66-73),
  `RuleId`/`TrapId`/`FamilyId` (:31-50), `Trap`/`Trap::validate` (reason bounds, rule-present, family token)]
- [Source: crates/opencmdb-core/src/gap/mod.rs:24-34 — `AbstentionCause` (`OutOfPerimeter` /
  `NoObservedValue` / `ConflictingObservations`), serialized PascalCase; `NoObservedValue` is the cause the
  W4 abstain reuses, matching `example.toml`]
- [Source: crates/opencmdb-core/src/observation/mod.rs:130-136, 146-172 — `HostnameSource`
  (`Dhcp`/`Dns`/`Mdns`/`Netbios`/`Other`), the `Fact` enum; `Hostname { name, source }` (:154-158) and
  `Uplink { peer_mac, peer_port }` (:164-168); `MacAddr` serializes as a 6-byte array]
- [Source: crates/opencmdb-core/src/score.rs:599-627, 704-710, 905-943 — the EXISTING rule vocabulary
  (`l1-exact-mac`/`l1-distinct-mac`, `l2-uplink-agrees`/`l2-different-switch`); this family COINS
  `l2-hostname-agrees`/`l2-different-hostname` because the uplink pair cannot express a shared uplink]
- [Source: crates/opencmdb-bin/src/fixtures.rs:864-913 — `assert_facts_are_synthetic` (the `Hostname` →
  `starts_with("doc-")` check at :879, the `Uplink.peer_mac`/`Mac` U/L-bit check, the RFC 5737 IP check) /
  `assert_synthetic_mac` / `assert_documentation_ip`; :1306 the cross-stream obs_id guard]
- [Source: crates/opencmdb-bin/src/trap_gate.rs:382-428 — the three committed-count assertions (7 → 10);
  `a_trap_with_no_answer_is_discovered_but_not_scored` answers `example-must-abstain` with
  `Outcome::Abstained { cause: NoObservedValue }`, so `scored()` stays 1]
- [Source: fixtures/scenario/traps/example.toml:40-44 — `example-must-abstain`, the established
  `{ must-abstain = { cause = "NoObservedValue" } }` shape and single-observation reason this family follows]
- [Source: fixtures/scenario/traps/multi-nic.toml + fixtures/scenario/replay/multi-nic.jsonl — story 4.10,
  the immediately prior L2 family and the exact structural model (fresh obs_id block, shared-vs-different
  `Uplink`, manifest bump, count update); note the CONTRAST: 4.10's discriminator is the switch (uplink
  DIFFERS for not-merge), 4.11's is the hostname (uplink SHARED throughout)]
- [Source: _bmad-output/implementation-artifacts/4-7b-trap-corpus-positive-and-negative.md:241-247 — DR1 (a
  `must-abstain` MAY carry a family but counts for NEITHER pole; a 3-column family is expressible and
  useful) and DR2; the completeness contract this family satisfies]

## Dev Agent Record

### Agent Model Used

claude-opus-4-8[1m] (dev-story workflow, 2026-07-24).

### Debug Log References

- **Count-coupling proved red (Task 4).** With `shared-hardware-vm.toml` on disk but the assertions
  still at `7`, `cargo test -p opencmdb-bin --locked the_committed_corpus_is_discovered_and_scored_by_nothing`
  RED with `assertion left == right failed … left: 10, right: 7` at `trap_gate.rs:390` — confirming the
  three new traps were discovered, parsed, validated and their obs_ids resolved against the committed
  stream before the count was even checked. Updated `7 → 10` at `:390/:408/:426` (+ the `:389` and `:418`
  comments) and the test went green.
- **sha256 of the two final artefacts** (`sha256sum`): stream
  `2981d3c1195bace9afde526a36ef270b590de7da1c73f543ac5d7f577e881a07`, trap file
  `75f0b947c210982df23b845f486f3cdc6e1297515877cdf58d5cf04217bc825c`. Hashed only after both files were
  final; pasted verbatim into `MANIFEST.toml`. **Post-review:** the trap file's header was rewrapped
  (review patch 1) → re-hashed to `47055cb70aac7b9b1b995d4bb3c45003f633299d4c8c2682b5fe58120054a4e6`
  and MANIFEST updated; the stream is byte-unchanged. `fixtures` gate re-run green (11 match).
- **Reason char counts measured** (`wc -m`, 20–300 bound): must-merge **243**, must-not-merge **273**,
  must-abstain **241** — all single-line, all inside the bound.
- **`abababab` prefix verified free** before authoring (`grep -r abababab fixtures/scenario/replay/`) and
  after (`grep -rl abababab fixtures/scenario/` → only the new `.jsonl` and its `.toml`, as expected —
  the trap file references the stream's obs_ids).

### Completion Notes List

- **First three-column family in the corpus** — `must-merge` (`l2-hostname-agrees`, `[W1,W2]`) +
  `must-not-merge` (`l2-different-hostname`, `[W1,W3]`) + `must-abstain` (`NoObservedValue`, `[W1,W4]`) of
  ONE scenario. Both decision poles present → `incomplete_families` empty, `passed()` green; the abstain
  rides along satisfying neither pole (DR1). Verified green:
  `passed_is_the_failures_gate_with_a_discovered_floor`.
- **First family whose discriminator is the hostname, not the topology.** All four observations carry an
  IDENTICAL shared `Uplink` (`peer_mac [2,0,94,0,96,10]`, `peer_port "swport-1"`) — the hypervisor's
  single physical port. W1/W2 share hostname `doc-vm-alpha`, W3 is `doc-vm-beta`, W4 has NO hostname.
- **Two L2 rule ids COINED (review decision, surfaced not hidden):** `l2-hostname-agrees` and
  `l2-different-hostname`. The uplink pair (`l2-uplink-agrees`/`l2-different-switch`) cannot express a
  shared uplink, so this records the Epic-5 contract that L2's cascade gains hostname-agreement and
  hostname-conflict rules (architecture.md:891). A reviewer preferring different names should raise it.
- **Gates all green** (`cargo run -p xtask -- ci`): `fixtures` reports **11 fixtures match, 0 orphan**
  (9 existing + 2 new); `file-size` largest 884 lines (the `.toml`/`.jsonl` are not counted, and
  `trap_gate.rs` grew only inside `#[cfg(test)]`). `views-hash STALE` is by design (ℹ, exits 0 — not
  regenerated in a story). `cargo fmt --all`, `cargo clippy --workspace --all-targets --locked -D warnings`,
  and `cargo test --workspace --locked` (107 bin / 86 core / 42 xtask, all pass) clean.
- **Privacy + corpus-validity walks pass over the new files automatically** — no harness change:
  `the_corpus_carries_no_real_network_data`, `every_replay_stream_in_the_corpus_is_valid`,
  `every_trap_file_in_the_corpus_is_valid`, `no_obs_id_is_shared_across_replay_streams` all green.
- **`scored()` / `failures()` stay 0** for the committed corpus at v0.1 — no answer producer exists
  (Epic 5). The family is discovered (10), parsed and validated, but scored by nothing.
- **Existing artefacts byte-for-byte unchanged** (`git status --short`): only the two new fixtures, the
  three-literal `trap_gate.rs` test bump, and the `MANIFEST.toml` two-entry bump. `Cargo.lock` untouched.

### File List

- **New (locked):** `fixtures/scenario/replay/shared-hardware-vm.jsonl` (4 observations)
- **New (locked):** `fixtures/scenario/traps/shared-hardware-vm.toml` (3 traps, `family = "shared-hardware-vm"`)
- **Modified:** `fixtures/MANIFEST.toml` (two `[[artefact]]` entries with sha256 — the deliberate bump)
- **Modified:** `crates/opencmdb-bin/src/trap_gate.rs` (three `#[cfg(test)]` count literals 7 → 10, plus the `:389`/`:418` comments — tests only, no production logic)
- **Modified:** `_bmad-output/implementation-artifacts/sprint-status.yaml` (4.11 → in-progress → review → done)
- **Modified (review patch):** `fixtures/scenario/traps/README.md` (one stale sentence — orphan-exempt, no manifest entry)

## Change Log

| Date       | Change                                                                 |
|------------|------------------------------------------------------------------------|
| 2026-07-24 | Story 4.11 drafted (create-story): shared-hardware VM trap family — first three-column L2 family (must-merge/`l2-hostname-agrees` + must-not-merge/`l2-different-hostname` + must-abstain/`NoObservedValue`), shared `Uplink` throughout, hostname discriminator, committed-count 7 → 10. Coins two hostname L2 rule ids (review decision, flagged). Status → ready-for-dev. |
| 2026-07-24 | Validated (three fresh-context agents: fact-check + gap-hunt during create, then a checklist-compliance pass on `validate`). All clean — READY-FOR-DEV, 0 HIGH/MED defects; AC↔task traceability complete, simulated file-assembly passes every gate (fixtures/privacy/validate/count), the family reports complete. Fixes applied: off-by-one `trap_gate.rs` line numbers (:389→:390, :407→:408, :425→:426, :417→:418); `fixtures.rs:877`→`:879` for the `doc-` hostname check; rewrote the committed `must-abstain` reason to drop the story-internal "W4" label ("this hostless virtual interface…"). |
| 2026-07-24 | Implemented (dev-story): wrote `shared-hardware-vm.jsonl` (4 obs, prefix `abababab`, shared `Uplink` + hostname discriminator, W4 hostless) and `shared-hardware-vm.toml` (3 traps, coined `l2-hostname-agrees`/`l2-different-hostname`, abstain on `NoObservedValue`); locked both in `MANIFEST.toml` with sha256; bumped `trap_gate.rs` committed-count assertions 7 → 10 (count-coupling proved red at left:10/right:7 first). Reasons 243/273/241 chars. All gates green: `fixtures` 11 match / 0 orphan, clippy/fmt/test/xtask ci clean, privacy + validity walks pass, existing artefacts + `Cargo.lock` unchanged. Status → review. |
| 2026-07-24 | Code review (3 fresh-context layers: Blind Hunter / Edge Case Hunter / Acceptance Auditor). Auditor: PASS on all 8 AC + all 14 anti-mistakes, every Dev Agent Record claim re-measured true. 0 HIGH/MED. Findings: 2 LOW patches APPLIED (trap-file header rewrapped so `l2-different-hostname` greps whole → re-hashed `47055cb7…` + MANIFEST bump; traps README.md:8 stale "only example.toml lives here" → "each in its own .toml", pre-existing since 4.9), 1 dismissed ("first three-column family" is exact — example.toml declares no family). Coined rule ids uncontested by all three layers. Gates re-run green (fixtures 11 match, 107+86+42 tests pass). Status → done. |
