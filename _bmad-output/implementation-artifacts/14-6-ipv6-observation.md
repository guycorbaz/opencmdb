# Story 14.6: IPv6, observation-only — the plan holds what the scanner will never see

Status: **ready-for-dev** — contexted and VALIDATED 2026-09-21 by two fresh-context layers, with
nine decisions taken by Guy and each recorded with the option refused.

🔑 **In NO epic file.** `epics.md`'s Epic 14 body stops at story 14.4; `:474` carries the epic's FR scope and **`:476`** is
what names 14.5 and 14.6, added by the partial retrospective of 2026-09-19. So this story inherits
**FR25's one sentence** — *"The operator can document IPv6 subnets and addresses (observation-only;
active IPv6 scanning is out of MVP scope)"* (`prd.md:910`) — and Epic 14's six measured constraints,
which it is downstream of and may not re-decide. Precedent for an epic-less story: 6.4b, 14.4c, 14.5.

Baseline: `master` at `bf4b71e` — **1 032 tests** (731 bin + 191 core + 110 xtask), ten gates, axe 0
violation nodes over the two passes, kbd-probe 61 checks.

## Story

**As** an operator whose network has IPv6,
**I want** to write my IPv6 subnets and addresses into the plan,
**so that** the plan describes the network I have rather than the half of it the scanner can reach.

## §0 — What Guy settled on 2026-09-21, each with the option refused

✅ **All four taken, the recommendation in all four** — and the recommendations were built on
measurements taken first, which is why they are recorded with what they refused rather than as
preferences that happened to win.

### 0.1 — ✅ (a) THE PLAN HOLDS IT, AND THE REFUSAL IS KEYED ON THE **FAMILY**

FR25 says *document*, observation-only. Two readings, and the difference is most of the story:

- **✅ TAKEN — (a) THE PLAN HOLDS IT.** The operator defines an IPv6 subnet, a range and an address;
  the selector lists them; the rail corrects and removes them. **No grid, no offer, no findings** —
  and the screen SAYS why rather than rendering an empty audit.

  ⚠️ **The clause *"and nothing else changes"* is STRUCK rather than softened: it is already false for
  the write path.** Since `0010` — story 14.5, merged the day this story was contexted —
  `ip_subnet.vlan` is `SMALLINT UNSIGNED **NOT NULL** DEFAULT 0` and the uniqueness key is
  `(base, prefix_len, vlan)`, so the validation's first IPv6 insert failed with
  `ERROR 1048: Column 'vlan' cannot be null` **before it ever reached the canonical CHECK**. Every
  IPv6 subnet write goes through 14.5's VLAN-carrying form, and *the rail corrects and removes them*
  now inherits 14.5's TENTH write route as well.

  ⚠️ **Measured: TWO THIRDS of it are nearly free on the screen and the third is not.**
  `MAX_DRAWN_ADDRESSES = 1024` (shipped by 14.2's code review) already routes any subnet larger than
  an IPv6 **/118** to the *too large to draw* branch, which shows the declared RANGES — a /64 holds
  18 446 744 073 709 551 616 addresses. **No grid** and **no offer** are inherited there, and the
  branch even says the second (`s.too_large_no_offer`).

  🔴 **BUT THE CEILING DOES NOT DELIVER THIS DECISION, AND THE VALIDATION CAUGHT IT.** *Every IPv6
  subnet an operator really has is larger than a /118* is an assumption about DEPLOYMENTS, not a
  property of the product: nothing refuses `2001:db8::/120`, which is **256** addresses, under the
  ceiling, and takes the **grid** branch. So (a) needs an explicit **FAMILY** test — *is this subnet
  IPv6* — and not a size test.

  🔴 **AND *"no findings"* IS FALSE OF THAT BRANCH, BY A SHIPPED DECISION, IN WRITING.**
  `_ipam.html:88` reads *"Decision 14: a subnet too large to draw offers nothing, says so, and **still
  shows its findings**"*, and includes `_ipam_audit.html`. For an IPv6 subnet the audit returns an
  empty vector, and an empty findings list renders `ipam.findings.none`: **"Nothing the network has
  shown contradicts this subnet's plan."** ⚠️ That is *precisely* the sentence §0.2 exists to forbid —
  an active, positive, FALSE claim about a subnet nothing looked at. **So (a) requires changing
  Decision 14's behaviour for one address family; it is not inherited.**

  ⚠️ **And §0.1 is COUPLED to §0.4, which this story treated as independent**: the occupancy line is
  ALREADY silent in the too-large branch (`ipam_page.rs:1639`, `strings(None, …)`), so §0.4's cost
  bites only in the GRID branch — the branch (a) asserts IPv6 never reaches and a `/120` does.
- **REFUSED — (b) PARITY.** A small IPv6 subnet draws a grid, the offer proposes a free address, the audit runs.
  ⚠️ The offer over a /64 is meaningless and the audit has nothing to audit against (see §0.2), so
  (b) buys parity for subnet sizes nobody deploys.

### 0.2 — ✅ A SENTENCE IN PLACE OF THE AUDIT, and no new noun

🔴 **Observation-only means the observed side is EMPTY for IPv6, permanently and by design.** The
connector emits IPv4 only; `address_sighting` therefore holds no IPv6 row, ever. So for an IPv6
subnet every declared address is *declared and never seen*, and the audit's three words —
`gap`, `undeclared`, « Conflit d'adresse » — are each about a COMPARISON that cannot happen.

⚠️ **Rendering an empty findings list is the wrong answer**: an empty list reads as *nothing is
wrong*, where the truth is *nothing was checked*. This is `AXE_REQUIRE_*`'s own distinction — *the
gate could not run* is not a pass — applied to the operator's screen. The fork:

- **✅ TAKEN — (a)** the IPv6 plan carries a sentence in place of the audit, saying the scanner does
  not reach it. 🔑 *An empty list reads as **nothing is wrong** where the truth is **nothing was
  checked***, which is `AXE_REQUIRE_*`'s own distinction — *the gate could not run* is not a pass —
  applied to the operator's screen for the first time.
- **REFUSED — (b)** a fourth state joins the audit's vocabulary — ⚠️ which is a **binding-table act and
  therefore Guy's**, not a story's (epic constraint 5, PR #166's precedent), and `undeclared`'s own
  widening of 2026-09-11 shows the table resists new nouns.

### 0.3 — ✅ THE PLAN SPEAKS `IpAddr`, THE SIGHTINGS STAY IPv4

🔴 **MEASURED — and the first version of this table was REFUTED by the validation, which is why the
command is written down now.** Throw-away worktree off `master`, `cargo check -p opencmdb-bin
--message-format short`, counting the files each error MENTIONS:

| change | compiler errors | where |
|---|---|---|
| `sed -i '101s/base: Ipv4Addr,/base: std::net::IpAddr,/'` — **the field alone** | **5** | THREE inside `Subnet`'s impl (`:123`, `:141`, `:147`) and **TWO in `insert_subnet`** (`:276`, `:282`) |
| `sed -i 's/Ipv4Addr/IpAddr/g' ipam_repo.rs` | **44** | `ipam_repo` 25 · `ipam_page` 13 · `ipam_audit` 11 · `ipam_write` 6 · **`sighting_repo` 2** · `ipam_rail` 2 |

⚠️ **The story first said 5 *"all inside `Subnet`'s own impl"* and **45** with a different
distribution, and neither was reproducible.** The escape matters more than the count: `:282` is
`canonical(subnet.base)` — **the IPv4 zero-padding function** — so the field does not stay contained,
and the one place it escapes to is §0.5's un-taken decision. ⚠️ Under `--all-targets` the second row
is **112** (bin 44 + test 68), which is the number a developer actually meets.

🔑 **The last column is the finding, and it survives both corrections.** Widening the shared address
type drags **`sighting_repo.rs`** along — the OBSERVED side — at `:106` and `:331`, and FR25 says the
observed side never sees IPv6. So the naive widening
makes an IPv6 path in code that nothing can ever produce, which is this project's dominant defect
class written into the type system: *a guard placed where the defect cannot occur*.

✅ **TAKEN**: the PLAN speaks `IpAddr` and the SIGHTINGS keep `Ipv4Addr`, so the audit joins them
only where both are IPv4 — **observation-only expressed in the TYPES rather than in a comment**.
**REFUSED**: one address type throughout, which reads more simply and makes the observed half carry
a case it will never meet. It is not free either: `Plan` holds `defined: BTreeSet<Ipv4Addr>` and `ranges:
Vec<(Ipv4Addr, Ipv4Addr, IpPolicy)>`, both read by the audit and by the grid.

### 0.4 — ✅ SATURATE IN `u64`, AND NEVER RENDER IT FOR IPv6

`Subnet::size() -> u64` is `u64::from(u32::from(last) - u32::from(network)) + 1` — IPv4 arithmetic
in its body and in its return type. An IPv6 /0 has 2^128 addresses, which no `u64` holds, and
`MAX_DRAWN_ADDRESSES` is compared against it — the comparison that decides whether the grid is drawn
at all. ⚠️ A saturating `u64` answer is enough for the COMPARISON and is a lie as a COUNT, and the
occupancy line renders a count. ✅ **TAKEN: saturate**, which is enough for the COMPARISON that decides whether the grid is drawn.
**REFUSED**: `u128` throughout, honest everywhere and paid at the ceiling comparison, the occupancy
and their tests.

🔴 **THE DECISION STANDS AND ITS STATED MECHANISM WAS WRONG — the gap-hunt measured both halves.**
*(a)* **The occupancy line does not read `size()` at all**: `PlanView::counts()` walks `self.cells`,
and `size()` has exactly ONE production caller — `ipam_page.rs:1009`, the ceiling comparison. So the
danger this decision named, *a saturated number in the occupancy line*, cannot occur; the danger that
DOES occur is §0.1's — a **true** count over a subnet the product should not be auditing at all.
*Right conclusion, wrong reason*, and the reason is corrected rather than the conclusion.

🔴 **(b) THE OBVIOUS SPELLING IS A RELEASE-ONLY HANG, and this is the sharpest thing the validation
found.** `1_u64 << (128 - 64)` is `1 << 64`. Reproduced by me on this machine:

```
$ rustc -O shift.rs && ./shift_rel      $ rustc shift.rs && ./shift_dbg
v4 /24  -> 256                          v4 /24  -> 256
v6 /120 -> 256                          v6 /120 -> 256
v6 /64  -> 1                            panicked: attempt to shift left with overflow  (exit 101)
```

The workspace declares **no `[profile]` section**, so the shipped image runs with
`overflow-checks = false`. An IPv6 `/64` would therefore report `size() == 1`, sail under
`MAX_DRAWN_ADDRESSES`, and send `PlanView::derive` looping over 2^64 addresses — **story 14.2's
2.08 GB denial of service, unbounded, and INVISIBLE in the debug suite, where the same code panics
instead.** 🔑 *The debug build turns this defect into a crash and the release build turns it into a
hang; a test suite that only runs debug measures the crash and ships the hang.* `size()` must be
family-aware and written so it cannot shift by 64 or more, with `/0`, `/64` and `/128` pinned.

### 0.5 — ✅ EXPANDED, ZERO-PADDED, **LOWER-CASE**, 39 CHARACTERS · and `0011` WIDENS IN PLACE, KEEPING THE FOUR NAMES

⚠️ **`VARCHAR(39)` and `prefix_len <= 128` are reserved and the CHECKS REFUSE IPv6 anyway** — verified
live on a migrated store, three refusals each naming its own constraint. **SIX** address columns carry
an IPv4 dotted-quad `RLIKE`, across **THREE** migrations, where this paragraph first said five in one:

| migration | constraint | column | side |
|---|---|---|---|
| `0007:102` | `ip_subnet_base_canonical` | `ip_subnet.base` | PLAN |
| `0007:133` | `ip_range_first_canonical` | `ip_range.first_addr` | PLAN |
| `0007:135` | `ip_range_last_canonical` | `ip_range.last_addr` | PLAN |
| `0007:154` | `ip_address_canonical` | `ip_address.addr` | PLAN |
| `0008:72` | `address_sighting_addr_canonical` | `address_sighting.addr` | **OBSERVED** |
| `0009:59` | `address_release_addr_canonical` | `address_release.addr` | **OBSERVED** |

🔴 **AND *"`0011` must widen those CHECKS"* CONTRADICTED §0.3 — the validation caught it before an
implementer could.** Read as *the six*, it widens the observed side, which §0.3 decided must stay
`Ipv4Addr`: the story would have built the IPv6 path the decision exists to prevent. **`0011` widens
the FOUR plan-side checks and `0008`/`0009` stay IPv4, said here in writing** so a dev agent meeting
§0.5 alone cannot get it wrong. And 14.5's lesson applies: **a shipped migration cannot be corrected
in place** (sqlx checksums it), so `0007`'s prose stays and `0011` carries the correction.

✅ **TAKEN (Guy, 2026-09-21), and it was BUILT before it was taken — the gap-hunt applied both
variants to a store already at `0010` and pressed them.**

**The spelling: fully expanded, zero-padded, LOWER-CASE, 39 characters.** Within a family
lexicographic order IS numeric order (`::1 < ::a < ::10 < ::ff < ::1:0`, read back from an
`ascii_bin` column), so D-14.1(b)'s whole reason survives verbatim and 39 is exactly the width
`0007` reserved. ⚠️ **Across families the order INTERLEAVES** — `0000:…` sorts before `009.0.0.1`
sorts before `2001:db8:…` sorts before `255.255.255.255` — so the plan-wide order is per-family and
not global. Nothing depends on a global one today; saying so beats letting someone find it.

**`0011` widens the FOUR plan-side checks IN PLACE, keeping their names** (drop-then-add).
⚠️ **That is the order story 14.5 refused, and it is taken here on a measurement rather than in
forgetfulness**: the window is real at the SQL level (`192.0.2.9` and `999.999.999.999` both insert
between the DROP and the ADD) and **unreachable through the product** — an interrupted `0011` leaves
`_sqlx_migrations` at `success = 0` and the binary refuses to boot, naming its own remedy
(`migration 11 is partially applied; fix and remove row from _sqlx_migrations`). **REFUSED**: a new
name then dropping the old, which is safe at the SQL level and costs §0.9's inherited prove-to-red
outright — the guard reads the constraint BY NAME and would panic on `RowNotFound` at `.expect()`,
never reaching either of the two sentences written to guide this story. *A guard first seen red by an
absent row has not been seen red.*

🔴 **AND AN INSTRUMENT FINDING THAT NEARLY BECAME A FALSE HIGH, which is the transferable half.**
`SELECT 'literal' RLIKE pattern` over two utf8mb4 literals is **case-insensitive**; the same pattern
on an `ascii_bin` COLUMN takes the column's collation and refuses the upper-case twin. The gap-hunt
measured `1` for `2001:0DB8:…` with literals and was a minute from reporting *"the case is not
imposed"* — against the column it is `ERROR 4025`. ⚠️ **`0007`'s own header records its probe results
in the literal form**, which is sound for IPv4 (no letters) and WRONG for IPv6. *Every probe of the
canonical pattern runs THROUGH THE COLUMN, and `0011`'s header must say why.*

### 0.6 — ✅ NO EDGES AT ALL FOR IPv6

`is_edge`'s shipped arm is `prefix_len >= 31`. Measured on a `2001:db8:0:43::/120` with a control,
**both plausible readings compile, both look right, and they give two different occupancy lines** —
`0 infrastructure` under the shipped rule, `2` under a family-aware `>= width - 1`. Nothing in the
code decides between them.

✅ **TAKEN: `is_edge` is FALSE for every IPv6 address.** IPv6 has no broadcast address; RFC 4291
§2.6.1 reserves the all-zeros interface identifier as the *Subnet-Router anycast* and the last
address is an ordinary host — so painting two cells `infrastructure` would state something about the
network that nothing supports, which is what `0010`'s sentinel decision refused one story ago.
**REFUSED**: the mechanical `/127`-`/128` transposition, which compiles and is a translation where a
decision was owed. ⚠️ A `/120`'s occupancy figures are PINNED so the two readings cannot both pass.

### 0.7 — ✅ THE *OUTSIDE* LIST STAYS, AND THE SENTENCE BOUNDS ITS SCOPE

🔴 **Measured on the running prototype**: `Plan::outside` is plan-wide and `audit_render` renders it
in BOTH branches, so an IPv6 subnet's page carried **observed IPv4 addresses with a LIVE release
control on each**, immediately under a heading and the all-clear sentence of §0.2. *Claim and
refutation in one viewport* — story 14.4's third round paid for that class.

✅ **TAKEN: the list keeps rendering**, because it belongs to the PLAN and not to the subnet, and
hiding it would cost the operator a true finding for having selected another tab. **REFUSED**:
dropping it on an IPv6 page, which is simpler to state and makes the product quieter than it needs
to be. ⚠️ **The cost is carried by the SENTENCE**: §0.2's text must say it speaks of THIS SUBNET
only, or the two contradict each other on one screen.

### 0.8 — ✅ THE CANONICAL CODEC BECOMES FAMILY-GENERIC, AND THE OBSERVED SIDE NARROWS

🔑 **The seam §0.3 creates is FOUR LINES in TWO places, and only two of them are the audit** — the
gap-hunt built the widening until the production build stopped: `ipam_audit.rs:101-102`
(`merge_sightings`, one honest `IpAddr::V4`) and **`sighting_repo.rs:106`, `:331`, which are not
comparisons at all but the SHARED canonical codec** `ipam_repo::{canonical, from_canonical}`.

✅ **TAKEN: one codec that knows both families; `sighting_repo` narrows its result back to
`Ipv4Addr` with an EXPLICIT arm for *an IPv6 row in `address_sighting`*** — a state only a write that
went around the connector can produce, which the product therefore NAMES rather than assumes
impossible. **REFUSED**: a second IPv4-only codec for the observed side, which needs no arm for an
impossible case and duplicates the padding logic, where this project's DRY rule tolerates duplication
only when a test pins it and a comment names it as deliberate.

⚠️ **And `ip_range_same_family`'s `LENGTH(first) = LENGTH(last)` stops being vacuous the moment two
widths exist** — the day story 14.1 predicted in writing. It becomes the family check it was written
to be, and the test pinning its vacuity must flip. ⚠️ It is the **SECOND** carrier and not the only
one: the gap-hunt measured that a straddling pair is refused FIRST by the adapter's containment check
(`POST /ipam/range first=192.0.2.1 last=2001:db8:0:42::ffff` → **422 "That range falls outside its
subnet."**), so the DDL CHECK never sees it through the product.

### 0.9 — Registered rows, and what they are really worth

- ⚠️ **`/ipam/release` accepts any well-formed IPv4** and `plan_releases` reads the whole table per
  audit read. An IPv6 plan makes the first question live. ⚠️ **The row itself names Epic 14's
  RETROSPECTIVE as owner** (`deferred-work.md:5981`); this story's ownership lives in
  `epic-14-retro-2026-09-19.md:114` instead, so an implementer reading only the register finds the
  wrong name.
- ⚠️ **`ip_range_same_family` is pinned as vacuous** by a test that reds *"the day FR25 adds a second
  width"* (story 14.1's own words) — ⚠️ **and it is NOT a register row**: `same_family` appears zero
  times in `deferred-work.md`, the vacuity being recorded in code (`ipam_repo.rs:2032`) and in a
  migration header (`0007:18-19`). The paragraph above said *"two registered rows"* and there is one.
  🔑 **The prove-to-red is REAL — the validation ran it with a control** — but it is *free* under three
  conditions it does not advertise: it reads **only** `ip_range_first_canonical` by name, so widening
  the subnet and address checks alone leaves it GREEN; it returns early without `DATABASE_URL`, so it
  is free in CI and silent locally; and if `0011` RENAMES the constraint instead of replacing it in
  place, the red becomes `.expect()`-carried, which this project counts separately.

## §1 — What the tree says today, measured 2026-09-21 on `bf4b71e`

- **`Subnet` is IPv4 by construction**: `base: Ipv4Addr`, `prefix_len: u8`, and `is_edge` already
  carries a `/31`-and-`/32` special case the edge layer found wrong once.
- **`Ipv4Addr` OCCURRENCES** (not call sites — 2 of `ipam_page`'s are in doc comments):
  `ipam_audit` 46 · `ipam_write` 37 · `ipam_repo` 36 · `ipam_page` 32 · **`sighting_repo` 6** ·
  `ipam_rail` 3 · `opencmdb-core::ipam` **0**. ⚠️ The MODULE holds no address type; the CRATE does —
  `observation/mod.rs:300` is `Fact::IpV4 { addr: Ipv4Addr }`, a production variant. The conclusion
  survives untouched: `std::net` is std either way, and D47's frontier is about `anyhow`/`axum`/
  `sqlx`/`askama`.
- 🔑 **`Fact` HAS NO IPv6 VARIANT AT ALL** (`Mac`, `IpV4`, `Hostname`, `DhcpLease`, `Uplink`,
  `OuiVendor`, `Rtt`), which is a STRONGER carrier for §0.2 than *"the connector emits IPv4 only"*:
  the observed side is empty for IPv6 because **the domain type cannot represent it**, not because
  one implementation declines to.
- **`MAX_DRAWN_ADDRESSES = 1024`**, checked BEFORE the cells are materialised (14.2's review, after a
  `10.0.0.0/8` served 2.08 GB at status 200).
- **The audit's inputs are plan-wide** (decision 2, 2026-09-10) and `Subnet` carries no VLAN — story
  14.5's blindness — so an IPv6 subnet would join the same plan-wide reads.
- **`ip_subnet.vlan` exists** since `0010`; an IPv6 subnet inherits the VLAN axis for free, and
  whether that means anything for IPv6 is a question nobody has asked.

## Acceptance Criteria

⚠️ **Every criterion below is DERIVED from §0 and from what the two validation layers measured, not
from FR25's one sentence.** Where a criterion exists because a layer refuted something, it says so.

**AC1 — migration `0011` widens the FOUR PLAN-SIDE checks, in place, keeping their names.**
`ip_subnet.base`, `ip_range.first_addr`, `ip_range.last_addr`, `ip_address.addr`. 🔴
**`address_sighting.addr` (`0008:72`) and `address_release.addr` (`0009:59`) STAY NARROW, and a guard
REDS if either is ever widened** — that is §0.3 written into the schema and its second carrier. The
pattern accepts the expanded, zero-padded, lower-case form and refuses the upper-case twin, the
compressed form, nine groups, a trailing newline (`\z`, never `$` — story 14.1's trap) and
`::ffff:192.000.002.009`. Re-runnable, applied twice; the header carries the `success = 0` recovery
recipe and **states drop-then-add as a DECISION with the boot refusal as its reason**, so the next
migration story does not read it as 14.5's lesson forgotten.

**AC2 — every probe of the canonical pattern runs THROUGH THE COLUMN**, never against a literal, and
`0011`'s header says why: `RLIKE` over two utf8mb4 literals is case-insensitive and reports `1` for
the upper-case IPv6 twin the column refuses with `ERROR 4025`. ⚠️ `0007`'s header records its probes
in the literal form — sound for IPv4, wrong here — and that is named rather than inherited.

**AC3 — the refusal is keyed on the FAMILY, never on the size.** An IPv6 subnet gets **no grid, no
occupancy line, no offer and no audit of its own**, *whatever its prefix length* — with tests on a
`/120`, a `/124` and a `/128`. 🔴 The size ceiling delivers none of the three: the gap-hunt built the
widening and `/ipam` drew a 256-cell grid, an occupancy line and **an offer** (`Next address the plan
can offer 2001:db8:0:43::10`) on a `2001:db8:0:43::/120`. ⚠️ And the *too large to draw* branch's own
sentence is about SIZE, which is a true sentence about the wrong reason for a `/64`.

**AC4 — §0.2's sentence REPLACES `ipam.findings.none` for an IPv6 subnet**, in both locales, and
**governs the whole audit region**. 🔴 Not *beside* it: measured on the running prototype, an IPv6
page asserted *"Nothing the network has shown contradicts this subnet's plan."* — the product
claiming concordance about a plan its only connector will never look at. ⚠️ Per §0.7 the plan-wide
*outside* list keeps rendering, so the sentence must say it speaks of THIS SUBNET only, or the two
contradict each other in one viewport.

**AC5 — `Subnet::size()` is family-aware and CANNOT shift by 64 or more**, with `/0`, `/64`, `/120`
and `/128` pinned in both families. 🔴 The obvious spelling `1_u64 << (128 - prefix)` **panics in
debug and returns 1 in release** — reproduced — and the workspace declares no `[profile]`, so the
shipped image has `overflow-checks = false`: an IPv6 `/64` would report `1`, sail under the ceiling
and loop over 2^64 addresses. *The debug build turns it into a crash and the release build into a
hang; a suite that runs debug measures the crash and ships the hang.*

**AC6 — `is_edge` is FALSE for every IPv6 address** (§0.6), and a `/120`'s occupancy figures are
PINNED so the mechanical `/127`-`/128` reading cannot also pass. The reason — IPv6 has no broadcast
address — is written at the site.

**AC7 — every write route answers an IPv6 argument BY NAME, before the store.** 🔴
`POST /ipam/release` answered **500** with *"The write did not go through, or its answer was lost"*
under the naive widening, because `0009`'s check (which AC1 keeps narrow) refuses it — on a route
reachable by hand-editing a URL, for a gesture §0 makes meaningless for IPv6. A keyed refusal with
its own status, on `ReleaseOfADefinedAddress`'s shape; and the other nine routes are enumerated with
what each answers.

**AC8 — the migration and the widened READER are ONE commit**, with a test pinning that an IPv6 row
is read back rather than skipped. 🔴 Measured: with `0011` applied and the reader not widened,
`list_subnets`, `plan_ranges` and `plan_addresses` SKIP the rows and NAME them in a `warn` — the page
answers **200** with the operator's IPv6 subnet silently absent from the selector.

**AC9 — §0.3's promise is NARROWED in writing, because it is carried on one half only.**
`Plan.defined: BTreeSet<IpAddr>` refuses a mixed lookup at the type (`Borrow`); `Plan.ranges` does
NOT — `Ipv4Addr` and `IpAddr` are cross-comparable, so `covering`'s interval test compiles silently
and answers correctly only because `IpAddr`'s order puts every V4 below every V6. **A tripwire on one
half**, on story 5.12's precedent, never *observation-only expressed in the types* without that
qualifier.

**AC10 — `ip_range_same_family` gets the test and the refusal row its own message demands**, and the
story records that it is the **SECOND** carrier: a straddling pair is refused first by the adapter's
containment check (measured, 422). ⚠️ Its inherited prove-to-red is free only under three conditions
it does not advertise — it reads ONE constraint by name, it returns early without `DATABASE_URL`, and
a rename turns its red `.expect()`-carried. AC1's name-keeping is what buys it.

**AC11 — the screen speaks the right family.** The forms' example placeholders on an IPv6 subnet's
page are IPv6 (measured: *"First address — for example 192.0.2.10"* renders there today), and the
selector tab names remain distinct per story 14.5's `tab_labels`. ⚠️ No test and neither browser gate
can see a resolvable key rendering a correct string in the WRONG CONTEXT — story 6b.6's `role_key`
family — so the carrier is named rather than assumed.

**AC12 — both browser gates REACH an IPv6 page.** `a11y/seed.sql` is IPv4-only, so an unseeded IPv6
surface is *the gate could not run* rather than a pass — this project's own `AXE_REQUIRE_*`
distinction. Any new key joins `ipam_page.rs`'s non-blank guard, and **a `.rs` is touched** because
`app.yml` is invisible to Cargo's incremental build.

**AC13 — THE LIVE COUNT lives in this story's `## Record` block**, checked by `cargo xtask record`,
with the DDL pass recorded into `deferred-work.md:5164`'s row as that row asks of the next migration
story — story 14.5 was the third to route around the driver.

**AC14 — no regression**: ten gates, `clippy --all-targets`, `RUSTFLAGS="-D warnings"`, fmt,
`cargo deny`, both store conditions, both browser gates, both manuals — and the user manual's IPAM
chapter gains IPv6 with `IPv6` leaving its `\planned` block, which no other criterion owns.
⚠️ **And this story cannot be verified against real IPv6**: measured on the developer machine, ten
link-local addresses, **no global address and no default IPv6 route**. Every IPv6 behaviour is
exercised by fixtures, exactly as story 14.5's VLAN was, and saying so is honest rather than a gap
to hide.

## Tasks / Subtasks

- [x] **T0** §0.1–§0.4 settled with Guy 2026-09-21; **§0.5–§0.8 settled the same day** after the
      validation, each recorded with the option refused.
- [x] **T1** `bmad-create-story validate` — two fresh-context layers. **The fact-check refuted seven
      claims and the gap-hunt BUILT the design and refuted its central premise on a running product.**
- [x] **T2** (AC1, AC2) `0011`, its probes through the column, and the guard keeping `0008`/`0009` narrow.
- [x] **T3** (AC5, AC6, AC9) `Subnet` and `Plan` widened; `size()`, `is_edge`, and the narrowed promise.
- [x] **T4** (AC8, AC10) The reader in the same commit; `ip_range_same_family`'s own test.
- [x] **T5** (AC3, AC4, AC11) The family refusal, the sentence that replaces the all-clear, the copy.
- [x] **T6** (AC7) Every write route's answer to an IPv6 argument, enumerated.
- [x] **T7** (AC12–AC14) The gates, the record, the manuals, the twins; the DDL pass recorded in the register.

## Mutation table

⚠️ **Written as the pass ran, never reconstructed afterwards** — story 14.5's review found that table
missing entirely, living in three commit messages and a scratchpad directory that does not survive
the session. Every cargo-side row was driven by `cargo xtask mutate`, which prints the prediction
beside the measurement and exits 1 when they disagree.

| id | mutation | predicted | measured | carrier |
|---|---|---|---|---|
| **D1** | `0011` widens `address_sighting.addr` too | red | 🔴 red 1 | `the_observed_side_stays_narrow` — decision §0.3 written into the schema |
| **D2** | `0011` narrows `ip_range_first_canonical` back to IPv4 | red | 🔴 red 1 | `the_family_check_is_live_now_that_a_second_width_exists` |
| **D3** | `$` instead of `\z` in `ip_address_canonical` | red | 🔴 red 1 | `one_address_has_exactly_one_spelling_in_the_store` — story 14.1's trap, still carried after the widening |
| T1 | `is_edge` loses its IPv6 arm | red | 🔴 red 1 | `an_ipv6_subnet_has_no_edges_at_any_prefix_length` |
| T2 | `size()` back to `1_u64 << (width - prefix)` | red | 🔴 red 1 | `a_subnets_size_saturates_and_never_shifts_out_of_range` — the release-only hang |
| T3 | `from_canonical` loses its IPv6 arm | red | 🔴 red 2 | the read-back test and the codec round-trip |
| T4 | the release drops its `is_ipv6` refusal | red | 🔴 red 1 | `every_write_route_says_what_it_does_with_an_ipv6_argument` |
| T5 | `family_examples` is not called | red:1 | 🔴 red 1 | `an_ipv6_subnets_page_offers_ipv6_examples` |
| **T6** | `plan_data` drops the family check | red | ✅ **GREEN — the pass's own finding** | **nothing.** The test called the RENDERER, so the branch that decides which renderer runs was carried by no test at all |
| T6-bis | the same, after the test went through `plan_data` | red | 🔴 red 1 | `the_family_and_not_the_size_decides_what_an_ipv6_subnet_renders` |
| **B1** | the seed's IPv6 subnet removed | axe 2 | 🔴 axe **2** (*the gate could not run*) | `AXE_REQUIRE_V6` |

**Eleven rows: ten reds and one GREEN that is the finding.** Carriers named per row; no *"every red
assertion-carried"* headline is claimed.

🔴 **`cargo xtask mutate` STILL CANNOT DRIVE DDL — this is the FOURTH story to measure it**, and
`deferred-work.md:5164` asks each one to record its pass there. With `--baseline` on a virgin store
it reported **162 reds**, because changing a migration breaks sqlx's checksum for the store the
baseline had just migrated. D1–D3 were driven by the purpose-built script instead. *A debt four
stories route around is a practice.*

⚠️ **Three instrument defects of mine, each caught by disbelieving a result.** Two browser runs
answered `ERR_INVALID_AUTH_CREDENTIALS` because the credentials were not exported in that shell — I
was one line from recording a refusal as the proof that `AXE_REQUIRE_V6` works. A `sed` renaming
`from_bits` missed the one call written `me.from_bits(...)`, and the compiler named it. And an
apostrophe in *"this subnet's plan"* is escaped by Askama, so a `contains` assertion could not see
the sentence it was written for — **the third time this project has paid for that**.

## Dev Notes

### Traps this project has paid for, and which apply here

- 🔴 **A guard placed where the defect cannot occur** — §0.3's whole subject: widening the type into
  `sighting_repo.rs` creates an IPv6 path nothing can produce.
- 🔴 **An empty list reads as *nothing is wrong*** where the truth is *nothing was checked* — §0.2.
- 🔴 **A shipped migration cannot be corrected in place** (story 14.5's finding): sqlx checksums it.
- ⚠️ **`cargo xtask mutate` cannot drive DDL** (`deferred-work.md:5164`) and **story 14.5 is the third
  to route around it with a purpose-built script** — that row asks the next migration story to record
  its pass there too, which is this one.
- ⚠️ **The suite is non-deterministic against a REUSED store** (story 6.6's row, met twice in 14.5):
  drop the database and pass `--baseline`, or a store's red is read as a mutation's.
- ⚠️ **`ipam_page.rs` has ~106 lines of headroom** and the next split is MEASURED and registered: the
  three `GET` checks, ~550 lines.
- ✅ **This story cannot be verified against real IPv6, MEASURED rather than assumed**: the developer
  machine carries ten link-local addresses, **no global address and no default IPv6 route**
  (`ip -6 addr show scope global` empty, `ip -6 route show default` empty). Fixtures throughout, as
  story 14.5's VLAN was — and AC14 says so rather than leaving a browser criterion to imply otherwise.
- 🔴 **A probe against a LITERAL is not a probe of the column** — §0.5's near-miss: `RLIKE` over two
  utf8mb4 literals is case-insensitive and reports `1` for a value the `ascii_bin` column refuses.
- 🔴 **The debug build and the release build disagree about `<<`** — §0.4(b): overflow panics in one
  and wraps in the other, and this workspace declares no `[profile]`, so the suite measures the panic
  and the image ships the wrap.
- 🔴 **`git checkout -- crates/` restores from the INDEX, not from HEAD** — the gap-hunt hit it and
  was saved only by having committed its prototype first. That gesture has destroyed uncommitted work
  five times in this project; this would have been the sixth.
- 🔴 **`ipam_repo.rs`'s first `#[cfg(test)]` is on LINE 7**, inside the module doc explaining that the
  `file-size` gate stops at the first one — a script cutting the file there replaces nothing and the
  unchanged build reads as a result. The gap-hunt hit it; story 14.2's review hit it in the same file.

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (1M context), 2026-09-21/22.

### Completion Notes List

**What shipped.** The plan holds IPv6. `0011` widens the four PLAN-side canonical CHECKs to the
expanded, zero-padded, lower-case 39-character form and leaves the two OBSERVED-side ones narrow;
`Subnet` carries an `IpAddr` and does its arithmetic on `u128`; the SIGHTINGS stay `Ipv4Addr`, so
the audit joins the two only where both are IPv4. An IPv6 subnet's page draws no grid, no occupancy
line and no offer **whatever its prefix length**, and says *nothing here was checked* in place of the
all-clear. **1 032 → 1 043 tests** (742 bin + 191 core + 110 xtask), ten gates, both browser gates.

🔴 **THE VALIDATION EARNED ITS COST TWICE, AND THE SECOND TIME IT REFUTED THE STORY'S CENTRE.** The
gap-hunt BUILT the widening and measured that a `2001:db8:0:43::/120` — 256 addresses, under the
ceiling — drew a 256-cell grid, an occupancy line and an OFFER. *Every IPv6 subnet an operator really
has is larger than a /118* was a claim about deployments, not a property of the product. The refusal
is keyed on the FAMILY.

🔴 **AND MY OWN GUARD WAS PLACED WHERE THE DEFECT CANNOT OCCUR.** Mutation T6 — dropping the family
check from `plan_data` — came back GREEN: my test called `render_unobservable` directly, so the
branch that decides which renderer runs was carried by nothing, in the story whose central criterion
it is. *A test that calls the renderer measures the renderer.*

🔴 **THE SHARPEST THING THE STORY AVOIDED IS A RELEASE-ONLY HANG.** `1_u64 << (128 - 64)` panics in
debug and returns **1** in release; the workspace declares no `[profile]`, so the shipped image runs
with `overflow-checks = false`. An IPv6 `/64` would have reported a size of 1, sailed under the
ceiling and looped over 2⁶⁴ addresses — story 14.2's 2.08 GB denial of service, unbounded, and
invisible in the debug suite where the same code panics. *The debug build turns it into a crash and
the release build into a hang.*

⚠️ **`ipam_page.rs` crossed the ceiling mid-implementation** (2015 lines, the gate RED) and the right
cut was taken in ONE gesture because story 14.5's review had measured and registered it: the three
`GET` checks, 596 lines, now `ipam_checks.rs`. ⚠️ The split then produced a finding of its own —
`ipam_page`'s key guard reads `include_str!("ipam_page.rs")`, so nineteen keys left its population
**without reddening it**. *A guard keyed on a file measures the file, not the concept.*

⚠️ **This story could not be verified against real IPv6, measured rather than assumed**: the
developer machine carries ten link-local addresses, no global address and no default IPv6 route.
Every IPv6 behaviour is exercised by fixtures, as story 14.5's VLAN was.

## Record

- live-count: bin=742 core=191 xtask=110
- base: a254de354fb679c54edd6cc467bbefcc20e31887
- registered: is the tightest file in the tree at 1954 code lines
- registered: is LIVE since `0011`, and it is the SECOND carrier
- registered: The plan-wide address order is PER-FAMILY, not global
- registered: is NOT carried by the type, where `Plan.defined` is
- registered: The workspace declares no `[profile]`
- file: .github/workflows/ci.yml
- file: _bmad-output/implementation-artifacts/14-6-ipv6-observation.md
- file: _bmad-output/implementation-artifacts/deferred-work.md
- file: a11y/axe-gate.mjs
- file: a11y/seed.sql
- file: crates/opencmdb-bin/assets/app.css
- file: crates/opencmdb-bin/locales/app.yml
- file: crates/opencmdb-bin/migrations/0011_ipv6_canonical.sql
- file: crates/opencmdb-bin/src/ipam_audit.rs
- file: crates/opencmdb-bin/src/ipam_checks.rs
- file: crates/opencmdb-bin/src/ipam_page.rs
- file: crates/opencmdb-bin/src/ipam_rail.rs
- file: crates/opencmdb-bin/src/ipam_repo.rs
- file: crates/opencmdb-bin/src/ipam_write.rs
- file: crates/opencmdb-bin/src/main.rs
- file: crates/opencmdb-bin/src/sighting_repo.rs
- file: crates/opencmdb-bin/templates/_ipam_audit.html
- file: docs/manuals/user-manual/user-manual.tex

### File List

The `## Record` block's `file:` lines are this story's File List (checked by `cargo xtask record`).
