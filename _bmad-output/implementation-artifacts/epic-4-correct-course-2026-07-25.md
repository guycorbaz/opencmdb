# Sprint change record — Epic 4 closure: story 4.19 split, 4.18 delivered as spec

Date: 2026-07-25
Trigger: the autonomous run reached the two stories epic planning had flagged as
"authored here but only executable in Epic 11 (UniFi connector)".
Decided by: party-mode roundtable (Winston — architect, Murat — test architect, John — PM),
run under Guy's standing mandate for this session ("work to the end of the epic; on a problem,
convene a party mode and decide"). Guy arbitrates after the fact via the epic report.

## The problem, stated exactly

Stories 4.18 (wire-format traps) and 4.19 (mutation fixtures) both name deliverables whose
consumer is Epic 11's UniFi parser, not Epic 5's identity engine. Three constraints collided:

1. **The real captured body can never be committed.** It is Guy's live network in a public
   repository; the corpus rule is absolute. `fixtures/capture/` therefore stays empty of real
   payloads until Epic 11 scrubs one.
2. **The parser does not exist.** Its serde types, its error taxonomy and its normalization
   choices are all undecided.
3. **D45 forbids writing a trap from belief** — "a gate on a false truth whose red will never
   arrive". That is the exact defect this corpus exists to refuse, so "write it anyway and fix
   it later" was not available.

## The decision

**4.18 — delivered NOW, in full, as a spec.** Its content is *measurement*, not belief: every
field behaviour it encodes is a recorded observation of the real payload, carried by a
SYNTHETIC body (shapes and conclusions, never the values). Its "expected variant" is expressed
as **expected Observations** — D19's frozen schema is the parser's output contract, so the
artefact CONSTRAINS Epic 11 instead of letting the parser certify itself ("the worst oracle is
a test green from birth" — Murat). Murat's red line was adopted as the story's stop criterion
and honoured: every expectation is derivable from the frozen schema, the measurement or
committed corpus doctrine; everything else is a NAMED HOLE in the charter, never a guess. The
validation pass then caught two beliefs that had slipped in as measurement — the
`meta`/`rc`/`data` envelope and the `ip` key — and they became named holes too.

**4.19 — SPLIT.**

- **4.19a, delivered with 4.18** (in `fixtures/scenario/wire/README.md`): the drift-surface
  record (the payload carries **127 distinct keys** where `Fact` names **7** — a measurement),
  and the layer charter as **binding constraints on Epic 11**: a renamed field must produce an
  explicit error, never a silently empty collection; `#[serde(default)]` is forbidden on any
  collection feeding presence; layer-A drift injection is theatre. These are not beliefs about
  a future parser — they are requirements imposed on it, and a requirement written early is
  architecture, not speculation.
- **4.19b, re-scoped to Epic 11**: the mutation GENERATOR, the ~30 generated fixtures, and
  their expected parse outcomes. Two reasons, both structural: expected outcomes for an error
  taxonomy that does not exist would be belief (D45), and a generator has no test that reds
  without the parser it attacks (the house rule "no guard ships without a red"). Murat argued
  for building the generator now against structural assertions; Winston's objection carried.

**Epic 4 closes DONE with the re-scope recorded, not silent.** John's argument decided it: an
epic left "in progress" for months while Epics 5–10 proceed is a lie in the burndown, and "the
day one line of the status file is known to be false, nobody believes the others". Epic 4's
promise to its actual consumer — Epic 5 needs the replay corpus, the trap corpus and the
metrics harness — is fully met.

## Where the promise now lives (four places, none of them a note in passing)

1. **GitHub issue #34** — "Epic 11: run the 4.18 wire spec under the real parser + implement
   4.19b (mutation generator)", carrying the inherited acceptance criteria. This is the living
   promise; issues are this project's single source of truth outside the story flow.
2. **This record.**
3. **`fixtures/scenario/wire/README.md`** — the charter, the named holes, the 4.19a
   deliverables and the 4.19b deferral, committed *inside the corpus* and pointing at #34.
4. **`fixtures/MANIFEST.toml`** — both wire entries carry `CONSUMER PENDING: Epic 11 (issue
   #34)`, so the corpus lock itself states that these artefacts have no reader yet.

Plus the dated note in `epics.md` and the sprint-status entries, which say what was delivered
and what moved rather than a bare "done".

## The lesson, for the next decomposition

John named it and it belongs in the retrospective: **a story belongs to the epic of its
CONSUMER, not the epic of its theme.** 4.18 and 4.19 were filed under Epic 4 because "traps"
was the theme; their only consumer was always Epic 11. The epic planning half-knew it — the
front-matter clause "authored here but only become executable in Epic 11" is the aveu à moitié
fait. Finishing that admission is what this record does.
