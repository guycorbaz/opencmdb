# Story 14.6: IPv6, observation-only — the plan holds what the scanner will never see

Status: drafting — **NOT validated. `bmad-create-story validate` by two fresh-context agents is
mandatory here and has not run.**

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

### 0.1 — ✅ (a) THE PLAN HOLDS IT, AND NOTHING ELSE CHANGES

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
occupancy line renders a count. ✅ **TAKEN: saturate**, which is enough for the COMPARISON that decides whether the grid is drawn —
the only use that decides anything. ⚠️ **And the cost is carried rather than hidden**: a saturated
value is a lie as a COUNT, and the occupancy line renders a count, so that line must be SILENT for
IPv6 rather than show a saturated number. **REFUSED**: `u128` throughout, honest everywhere and
paid at the ceiling comparison, the occupancy and their tests.

### 0.5 — A migration is owed, and the schema's reservation is narrower than it reads

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

🔴 **And the canonical-spelling decision must be RE-TAKEN for IPv6.** D-14.1(b) stores IPv4
zero-padded so that lexicographic order is numeric order. The IPv6 equivalent is the fully expanded,
zero-padded, lower-case form (`2001:0db8:0000:…`) — 39 characters, which is exactly why the column is
39 wide. ⚠️ But `ip_range_same_family`'s `LENGTH(first) = LENGTH(last)` then stops being vacuous and
becomes the family check it was written to be — **the day 14.1 predicted in writing**, and the test
pinning its vacuity must flip.

### 0.6 — Two registered rows are this story's by name

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

⚠️ **STILL NOT WRITTEN, and the reason has changed.** §0 is settled; **§0.5 is not** — the canonical
IPv6 spelling and migration `0011`'s shape are a schema decision this story may not take alone, and
the mandatory validation has not run. Criteria written now would be criteria written against an
unvalidated design, which is the one thing this project's process refuses.

## Tasks / Subtasks

- [x] **T0** §0.1–§0.4 settled with Guy on 2026-09-21, each recorded with the option refused.
- [ ] **T0b** §0.5: the canonical IPv6 spelling, and whether `0011` widens the CHECKS or replaces them.
- [ ] **T1** Run `bmad-create-story validate` — two fresh-context layers, MANDATORY here.
- [ ] **T2..** Written after T0b and T1, from the answers.

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
- ⚠️ **This story cannot be verified on the reference LAN** unless it has IPv6 — to be measured, not
  assumed, before a criterion claims a browser pass over real data.
