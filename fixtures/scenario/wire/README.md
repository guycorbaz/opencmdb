# fixtures/scenario/wire/

D35 layer B's **spec half**, written before the parser it judges (story 4.18, after the epic-4
party-mode decision of 2026-07-25). This directory holds a SYNTHETIC wire body whose every field
behaviour is a **measurement** of the real UniFi payload (architecture.md's measurement table,
2026-07-16), plus the Observations the future parser MUST produce from it. It proves the
**parser**, never the engine — but unlike `../../capture/` it does not rot: it is a spec, right
or wrong, deliberately placed here rather than under `capture/` (whose charter is *real*
payloads, version-tagged, re-captured; the architecture tree placed wire content there because
it assumed a committable real body — the privacy rule forbids that, so the spec half lives here
and the capture half stays empty until Epic 11).

**Nothing runs these bytes under a parser today.** The shape test
(`the_wire_spec_encodes_the_measured_field_behaviours`, fixtures.rs) guards the pair — the
body's measured behaviours, the expected stream's values, the DERIVATION between them, and the
privacy rules (this directory sits outside every corpus walk; that test is its privacy
coverage). The first real red arrives with Epic 11's harness: see **GitHub issue #34**, which
carries this directory's execution and 4.19b as Epic-11 acceptance criteria, and the epic-4
correct-course record ("Epic 4 closure — 4.18 delivered as spec, 4.19 split").

## The measured behaviours the body encodes

Per the measurement table (architecture.md:4311-4326): `mac` lowercase colon-separated (100%) ·
`last_seen` a 10-digit SECONDS epoch (not ms) · `oui` present and EMPTY on a large share ·
`vlan` MISSING (100%) · `network_id` fixed-length 24, one distinct value · `hostname` MISSING
and EMPTY both occurring, **null NEVER** (story 4.17's shapes) · `is_wired` bool (100%; both
values occur — the one admitted inference: the split being total implies it) · `sw_port` an
integer of 1–2 digits.

## The named holes (never guessed — D45's red line)

An expectation below is either FIRM (derivable from the frozen Observation schema, the
measurement, or committed corpus doctrine) or a HOLE, named here and revisable only by a
deliberate corpus bump:

- **The `meta`/`rc`/`data` envelope** and **the `ip` key** (name/presence/format): vendor API
  convention, NOT covered by the measurement — confirmed or bumped at Epic 11's first real
  capture.
- **`Hostname.source: Dhcp`**: the source attribution is a parser design choice not yet taken;
  `Dhcp` is provisional (the corpus's uniform value).
- **`OuiVendor` empty-vs-absent**: the expected stream maps wire `""` → a fact carrying `""`,
  by 4.17's committed doctrine (empty ≡ no signal, but the byte-present shape is preserved);
  the measurement's own wording ("a named Fact that is usually absent") leaves the other
  mapping arguable — a recorded-bump candidate for Epic 11.
- **No `Uplink`**: the measurement never covered `sw_mac`; expecting an uplink would be belief.
- **`sw_port` on wireless**: presence rate unmeasured; the body encodes the certain case only
  (present on wired).
- **`obs_id` / `connector_id` / `scope`**: HARNESS CONTEXT, not expectations. The `bdbdbdbd`
  ids and the corpus's standard UUIDs are placeholders (pinned by the shape test so they
  cannot drift); Epic 11's runner injects its own context and compares **facts +
  `observed_at`** only. The `bdbdbdbd` obs_id prefix is RESERVED by this directory — the
  cross-stream uniqueness walk covers `scenario/replay/` only and cannot see these files.

## 4.19a — the drift surface, and the binding layer charter (delivered here; 4.19b → Epic 11)

**The drift-surface record (a measurement):** the real payload carries **127 distinct keys**
where the `Fact` enum names **7**. That ratio IS the drift surface D35's mutation fixtures
exist to cover — the failure that fails silently, not the one that fails loudly.

**Binding constraints on Epic 11's parser (not beliefs — requirements):**

1. A renamed field must produce an **explicit error**, never a silently empty collection.
2. `#[serde(default)]` is **FORBIDDEN** on any collection feeding presence.
3. Layer-A drift injection is theatre: "it tests nothing — it asserts the engine handles an
   error you handed it, without proving the parser produces one. That is the most insidious
   theatre of all, because it looks like fault injection" (epics.md's 4.19 AC, restating D35 —
   architecture.md:2032-2034 carries D35's own wording).

**4.19b — deferred to Epic 11, deliberately:** the mutation GENERATOR (first use of the
MANIFEST's `generator` field), the ~30 generated fixtures (deleted / nulled / retyped /
renamed) at `capture/mutations/`, and their expected parse outcomes. Deferred because expected
outcomes for an error taxonomy that does not exist would be "written from belief" (D45 — the
exact defect this corpus exists to refuse), and a generator has no test that reds without the
parser it attacks. The promise is held by issue #34 and the epic-4 correct-course record — not
by this paragraph alone.
