---
stepsCompleted: ['step-01-init', 'step-02-discovery', 'step-02b-vision', 'step-02c-executive-summary', 'step-03-success', 'step-04-journeys', 'step-05-domain', 'step-06-innovation', 'step-07-project-type', 'step-08-scoping', 'step-09-functional', 'step-10-nonfunctional', 'step-11-polish', 'step-12-complete', 'step-e-01-discovery', 'step-e-02-review', 'step-e-03-edit']
workflow: 'edit'
lastEdited: '2026-07-17'
editHistory:
  - date: '2026-07-17'
    source: 'architecture D57-scope (party mode: Winston, Murat, Amelia, John — decided by Guy). Items F55-F58. The impact graph leaves the MVP.'
    changes: |
      THE DECISION (Guy, on John's split): the MVP ships DECLARATIONS, never BELIEFS.
        THE RULE, and it is binding: a relationship the product may one day OBSERVE is a DECLARATION.
        A relationship nothing will ever observe is a BELIEF. Growth ships beliefs the day an observer
        exists to contradict them.

      FR26/FR27 STAY at MVP - software instances on a device, applications with owner + criticality.
        Two tables, two FKs. Growth's port/service scanning gives FR26 its observed half, so it is not
        an orphan.
      FR28 SPLITS - `hosts`/`exposes` MVP (they are CONTAINMENTS, FR26 restated, and they are the two
        the service scan will one day OBSERVE) · `depends_on`/`connects_to` -> GROWTH, sequenced with
        the port/service scanning: "a coherence dependency, not an effort cut" (the PRD already knew
        how to write this sentence - it wrote it for the service fingerprint).
      FR29 at MVP = FR39, WHICH WAS ALREADY VOTED AND NOBODY HAD READ IT: "view its full record
        (declared attributes, observation history, connection point, HOSTED APPLICATIONS)". Marc's
        answer was already in the MVP. One containment hop, no traversal.
      🔴 AND THE WORD "IMPACT" IS REFUSED ON IT. It is named "Hosted here". A screen called Impact that
        does one hop and concludes "nothing else is affected" is FR7's `satisfaction` trap with a nav
        entry - present, well-typed and wrong is more dangerous than missing; the missing one fails
        loudly. And WORSE than `satisfaction`: that one is observed, so a connector can contradict it.
        depends_on is the only field in the product with no loop behind it. It can only rot, and
        nothing is watching. The true impact view -> Growth, beside the interactive topology, where
        the UX spec had already put it.
      FR26/FR27 AT MVP ARE THE PROBE, and that is what makes this free: in six months we COUNT what was
        actually typed. 11 instances -> the graph is a SELECT and D57 never existed. 300 -> it is a real
        decision, taken on a fact. Same gesture as the cardinality probe: it measures OUR terrain, the
        only one of the two we own. Nobody had to be overridden - shipping FR26/27 funds the decision.

      SITES: FR26 (+ the D15 anchor and AC-M-04) · FR28 (split) · FR29 (= FR39, renamed) · section title
        "Applications & Impact Analysis" -> "Applications & Hosted Software" · Journey 6 (Marc no longer
        declares depends_on; "Hosted here" answers him) · NFR2 · Technical/Business success lines · the
        MVP scope list (10) · the riskiest-first order.

      🔴 THE DEBATE'S OWN FINDING, recorded because it is the third in 24h: the brief that convened the
        room said "Journey 4 (binding): Marc creates Paperless". Journey 4 is "When the source goes
        blind" - the CRITICAL PATH, and the one actually promoted to binding. Paperless is Journey 6,
        titled "Sunday documentation (CONSULTATIVE)". The binding label was carried onto the
        consultative journey - and it was the ONLY procedural argument protecting FR26-29. All four
        agents opened with the correction. Murat: "this document now has enough decisions that its own
        authors cannot hold them all in their heads, and it discovers holes where its own rules are
        already standing." -> F56.
  - date: '2026-07-17'
    source: 'architecture D64 (party mode: Winston, Murat, John — decided by Guy). Items F52-F54. The storage engine narrows.'
    changes: |
      THE DECISION: SQLite is OUT for good. MySQL is OUT. MariaDB 10.11+ is the only supported engine.
        The locked non-negotiable "SQLite + MySQL/MariaDB" is REVOKED. "SQLite later" is banned in
        writing (it pays the full re-entry cost AND gives up the certainty of the removal). See D64.

      THE MOTIVE, and it is deliberately NOT either of the two first offered - both were refused:
        (a) "I will use MariaDB and most users will too" - REFUSED. This is D59, one day old: Guy IS a
            user, he is NOT the reference user. Not probe-able before there are users, so the lesson is
            not "measure", it is "do not found the decision on it". A motive that cannot be checked
            collapses the first time someone writes "I just wanted a docker run".
        (b) "an SMB of 200-300 employees approaches a thousand devices - SQLite won't hold" - REFUSED,
            checkable and FALSE. SQLite's ceiling is hundreds of GB; what bounds it is write concurrency,
            and D21/D25 already decided single-writer. The observation is true (the target is an SMB, not
            a Raspberry Pi); only the inference is wrong.
        RECORDED INSTEAD: "We do not need most users to PREFER MariaDB. We need no target user to be
            BLOCKED by it." Verifiable against our own journeys - Journey 6's Marc runs Paperless-ngx +
            PostgreSQL + Redis in his basement and documents it as an asset. Plus: the measured ledger is
            one-sided (DSM ships MariaDB 10.11.11 natively and backs it up; the one-line `docker run` has
            not existed since D27; the "SQLite, small installs" population never had one shred of evidence
            in any document). D59 disqualifies BOTH camps - SQLite was an unmeasured belief too.

      1 STRIKE (number retained, as with FR52):
        NFR16 - SQLite WAL / busy_timeout / 50 concurrent writers. It described the failure mode of an
          engine we no longer support. The serialized writer survives in NFR1 - it is a property of our
          write path, not of SQLite's locking.
      6 UPDATES:
        NFR21 - MariaDB 10.11+ only. MySQL is a DIFFERENT PRODUCT (collation defaults, RETURNING,
          recursive-CTE history) and we do not claim what has no CI - that rule is D52's and it is ours.
          "MariaDB/MySQL only" was never one engine; it was two, on a WORSE axis: cousins diverge rarely,
          so the differential's yield collapses while its cost stays whole. PostgreSQL stays out at MVP,
          and its future addition is recorded WITH its price, never as a free option.
        NFR15 - the dual-parity clause is gone; what survives and is PROMOTED is D10: the engine never
          decides a comparison. This is a CORRECTNESS requirement, not a portability one - identity is
          the product. New enforcement (D64 condition 1): explicit binary collation on every text column
          + a CI grep over the DDL. The second engine was the only thing making that red before.
        NFR17 - migrations verified on MariaDB; resumability comes from the BACKUP, not the transaction.
        NFR20 - the binary is one process and REQUIRES a MariaDB alongside. No document may describe the
          try-it path as a single `docker run` - it would be a gap between declared and observed, shipped
          by the product whose thesis is that such gaps matter (NFR9's error class, in our own window).
        Technical Success - dual-backend parity AC replaced by the engine pin (dev = CI = prod at
          mariadb:10.11.11) + the identity-comparison AC.
        Scope / complexity / risk lines (4 sites) - the 2x test matrix is gone.

      RENUNCIATION, named rather than buried (D64): the driver-lie detector is gone with D46b, in the
        same month we adopt sqlx =0.9.0 (8 weeks old, no 0.9.1, repo moved orgs) under a "prefer recent"
        rule carried by "if your CI is good, age is a superstition" - and half of what made the CI good
        was two drivers. We know, we accept it, we do not pretend to cover it. AND: Guy re-opened
        PostgreSQL as a possible future, which cashes in D51 point 4 (amended by its own author this
        session: "the discounted cost is zero" was false).

      DISSENT: Murat dissented on TIMING, not on the decision - story 1 is already funded as dual and
        returns four numbers this room is guessing. Guy overrode and closed the debate. The numbers will
        be observed anyway; what they can no longer do is re-open it.
  - date: '2026-07-16'
    source: 'architecture workflow (steps 5-8) + the cardinality probe run against the reference device — items F42-F51. A SHORT, targeted pass: F1-F37 were already applied earlier the same day.'
    changes: |
      1 ADDITION:
        FR16b - Abstention is DISPLAYED, COUNTED and GROUPED BY CAUSE, and is never a reproach.
          FR16 already made abstention first-class in the ENGINE, per item. Nothing covered the AGGREGATE:
          the operator does not experience one abstention, he experiences their total. Each cause is one
          line and one gesture, not N failures. The counter measures the product's REACH, not the
          operator's debt.
      6 UPDATES:
        NFR8  - the version matrix was bounded on the WRONG AXIS. The controller runs two independent
          release trains: UniFi OS console (a 5.x build) and Network application (a 10.x build). The connector
          speaks to the Network application. A fixture tagged with a console version lies about its own
          provenance - and the re-capture job diffs against that tag. Tested matrix = Network application
          10.4.x, growing BY EVIDENCE. Outside it the guarantee changes in KIND: the product does not
          claim to work on a version it has never seen, it claims not to LIE about one.
          ("Supports future versions" is a wish, not a requirement: there is no fixture for a version that
          does not exist, and the claim would be discovered false at a user's site rather than ours.)
        NFR30 - 300 hosts / 36 subnets is the TARGET AUDIENCE's profile, not the author's install (the developer's own network
          a few hundred clients on a single VLAN, measured). It travelled through four documents
          unqualified. Consequence: the dimension the target is DEFINED by - segmentation - is the one
          dimension the author's network cannot exercise. And it lands on the seeded generator.
        NFR baseline - the load-bearing assumption (the gap is the CONTENT, abstention the EXCEPTION) is
          now MEASURED and HOLDS: single-interface comfortably above the 80% threshold. Bounds, not a number.
          Three reservations recorded, because a favourable answer is when one stops looking.
        FR9   - signal availability is measured and bounds the outcome BEFORE the engine does: hostname is
          unusable on nearly half of known clients. The abstention rate is bounded below by DATA, not by engine
          quality.
        FR38b - dormant is not an edge case: a large share of the known inventory is locally administered. The
          feature that keeps the central indicator from drifting upward forever carries half the data.
        FR7   - the first measured capability descriptor confirms per-source difference is legal, not a
          defect (UniFi: topology yes at the large majority of wired clients, latency NO - latency is a scanner fact).
          Plus the silent trap: `satisfaction` is present on nearly all clients and is NOT a latency. A field that is
          present, well-typed and wrong is more dangerous than a missing one - the missing one fails loudly.
      1 GROWTH ADDITION:
        The narrator - the six-month test ("does the product become more unpleasant?") is a LOWER BOUND;
          an empty text file passes it. Its missing symmetric: does the product TEACH him something when he
          comes back? The rule "only a NEW FACT may trigger a notification" is its left half - a new fact IS
          a reason to speak. Recorded beside the bans on purpose: they look alike from a distance and only
          one is a reproach. The line: the narrator reports facts about the NETWORK, never elapsed time
          about the operator.
      1 CLOSED WITHOUT EDIT:
        F43 asked for "the number that defines stable". NFR4 already carries it: "the gate is truth-table
          failures = 0". Asking for a second threshold would have been exactly the theatre NFR4 refuses.
      NOT DONE HERE - the UX-targeted half: `Live + Reduced` must be GREEN with a scope label, never orange
        (blind is an incident and deserves a colour; reduced is a property and deserves a sentence - and the
        UX spec currently has NO source-state screen at all, so this is a design to make); `ignore` ->
        `exclude` (already flagged "not settled" in the UX glossary; the reason is structural, not tonal:
        `ignore` names the operator's ATTITUDE while every other verb names the object's STATE); plus the UX
        halves of the abstention counter, the narrator, dormant and hostname. Second pass.
  - date: '2026-07-16'
    source: 'architecture workflow (steps 1-4) — 33 feedback items F1-F37 raised while testing every requirement against its actual testability'
    changes: |
      8 REWRITES (the requirement as written was false, unenforceable or untestable):
        FR5  - three states -> two orthogonal axes (liveness x capability); full/degraded/offline is a derived
               presentation. The missing cell was "live but reduced" (a source that answers and sees less).
        FR19 - "unavailable" was undefined at the heart of the guarantee that defines the product; now mechanical.
        NFR4 - the statistical gate is theatre at n=300 (the only measurable threshold is zero); replaced by a
               binary adversarial trap suite with THREE columns, incl. must-merge (the anti-cowardice column:
               a gate that rewards abstention is trivially gamed by an engine that abstains on everything).
        NFR8 - "degrades gracefully (no crash)" is passed by the most dangerous product imaginable; replaced by
               four falsifiable assertions, chiefly monotone honesty: a faulted run cannot invent a single fact.
        NFR9 - "the key is separate" is unenforceable (backup tools select per shared folder); split into
               9a/9b/9c, where 9b is a behaviour of our binary (refuse to start) and 9c is an explicit
               NON-guarantee. A broad fictional guarantee destroys the vigilance it replaces.
        FR52 - removed from MVP (see below).
        Success Criteria x2 - north-star aggregation and the "honest signal" line.
      5 ADDITIONS (the requirement was missing):
        FR13 re-discovery case (the dominant long-run case had no requirement); FR14 sixth gesture (accept-gap);
        FR38b ephemeral-interface lifecycle (without it the divergence metric drifts upward forever);
        NFR7b silent schema drift (a bigger risk than the loud 3 a.m. failure); Journey 4 promoted to a named,
        binding requirement (a story cannot fail a build).
      3 REMOVALS:
        FR52 opt-in telemetry - no user job; n_opt-in ~ 2 at 3 months with a ~3x selection bias that MORE opt-in
        does not fix; the receiver is a target list; a receiver outage disguises itself as signal.
        Cross-instance north-star aggregation - a percentage whose denominator is user-defined has no referent.
        Telemetry as the "retention (honest signal)" - replaced by Docker pulls tag-by-tag + public artefacts.
      20 CLARIFICATIONS: FR7 (dated capability), FR9 (two-level identity), FR16 (ambiguity is persisted data),
        FR20 (only capable sources conflict), FR38 (observations only), FR48 (envelope, master-key rotation only),
        NFR1/16 (read/write path separation), NFR5 (extended), NFR12 (non-regenerable oracle, failure paths),
        NFR15 (identical invariants, backend-specific triggers), scan reach as an L2/L3 assumption, threat model,
        MariaDB day-1, open-divergence excludes pre-baseline, 3-month goal in integers.
      1 ADDITION - a canonical vocabulary section (binding). accept-as-declared -> document / accept-gap.
        FR UI "Merger" is the fixed translation of EN "document"; "merge" is retired in English because the
        founding pillar is "linked, never merged". "revert" is retired: nothing is ever destroyed.
      NOT DONE HERE - 4 items target ux-design-specification.md (bootstrap as a mode; doubt granularity per
        regime; testable backlog bans; the Tailwind note describing an API that no longer exists). Second pass.
releaseMode: phased
inputDocuments:
  - _bmad-output/planning-artifacts/product-brief-opencmdb.md
  - _bmad-output/planning-artifacts/product-brief-opencmdb-distillate.md
  - _bmad-output/planning-artifacts/competitive-analysis.md
  - docs/brief-initial-bmad.md
  - _bmad-output/planning-artifacts/architecture.md
documentCounts:
  briefs: 2
  research: 1
  brainstorming: 0
  projectDocs: 1
workflowType: 'prd'
project: opencmdb
classification:
  projectType: 'self-hosted data-reconciliation engine (event-driven, data-centric) — web UI + read-only JSON API + Prometheus /metrics as surfaces'
  domain: 'infrastructure / network management (IPAM/CMDB) — general, no regulatory compliance'
  complexity: high
  projectContext: greenfield
  panelNotes:
    - 'Panel (Winston/John/Amelia) reframed away from "web_app": the core is a reconciliation engine with temporal state; UI/API are surfaces. PRD must treat discovery ingestion, data freshness, conflict/identity semantics as first-rank features.'
    - 'Complexity raised medium -> high: dual SQLite/MariaDB backend (2x test matrix), unofficial UniFi API unstable across UniFi OS versions (needs contract tests + recorded fixtures + graceful degradation), MAC reconciliation as a state machine (randomization, shared-HW VMs, multi-NIC).'
    - 'Open questions to carry into PRD: (a) at MVP does a gap trigger an ACTION/alert or just display? (b) is "install-and-forget" operability a first-rank MVP feature? (c) is ARP scan (NET_RAW/host networking) in automated CI scope or out?'
    - 'Test-orienting ACs suggested: identical suite green on both backends; versioned UniFi fixtures; reconciliation spec as a truth table (input -> decision).'
visionDecisions:
  - 'MVP soul: both reactive (event-driven alerts) AND consultative (browse current state); event-driven backbone. Fork = option 3.'
  - 'Alerts must be actionable in one click via a DEEP LINK to the exact object. Deep-link (stable canonical URLs) is an MVP deliverable on in-app + generic webhook.'
  - 'Email notifications stay v2 (deliverability/SMTP cost; audience lives in ntfy/Discord/webhook, not email). Generic webhook is the MVP notification backbone; email is just one v2 transport among others.'
  - 'Deep-link architecture required at MVP: stable canonical URI per entity/alert (e.g. /entity/{uuid}, /alert/{id}); external_base_url configurable; authenticated resolution that handles deleted/stale objects.'
  - 'external_base_url default (assumed, pending confirmation): tolerant with fallback (auto-detect LAN IP:port) + visible warning until set; does NOT block startup (protects <15min first-value goal).'
  - 'Focused object views must be responsive for BOTH mobile and desktop (alert click can originate from either).'
---

# Product Requirements Document - opencmdb

**Author:** Guy
**Date:** 2026-07-15

## Executive Summary

**Your network documentation lies — opencmdb tells you where.** Scanning and
documenting have always been separate acts: scanners see what's on the wire but
forget it; source-of-truth tools record what you declared but never look. opencmdb
is the bridge. It is a self-hosted, single-binary **reconciliation engine** that
continuously compares the network's **observed** state (auto-discovered) with its
**declared** state (documented by you), and makes every discrepancy actionable.

It serves operators of small-but-nontrivial networks — an advanced home-lab or an
SMB without a dedicated IT team, typically up to a few hundred hosts across several
VLANs. Two moves define it: point it at a UniFi controller and a full inventory
populates itself in one polling cycle (an ARP/ping scanner covers everything
non-UniFi); then a discovered device becomes documentation in one gesture — the
machine proposes, the human ratifies. When a conflict appears, it names **which**
physical device is involved — not merely that a conflict exists.

The incumbents leave the middle unserved:

| Class | Example | Auto-discovers? | Models declared intent? |
|---|---|---|---|
| Source-of-truth | NetBox, iTop | No | Yes (heavy stack) |
| Scanner | Angry IP, phpIPAM | Yes | No |
| **opencmdb** | — | **Yes** | **Yes (one-click)** |

The MVP is both **reactive** (event-driven alerts: unknown device, IP unseen for N
days, IP conflict — each a one-click deep link onto the exact object) and
**consultative** (live inventory, per-subnet IP occupancy, hosted applications).
It ships as one container on a Synology NAS; the **target** — to be validated beyond
the author's own network — is a working install in under 30 minutes and visible
value in under 15.

### What Makes This Special

- **Reconciliation, unpaywalled.** Continuous observed-vs-declared drift detection
  exists today only at the top of the market (NetBox Assurance — paid, heavy stack).
  A free, self-hosted, single-binary version is an open wedge.
- **Scanning that documents.** One-click *documenting* keeps the declared
  side alive with near-zero effort — the discipline every static tool fails at.
- **Zero-privilege UniFi integration.** Reads clients, switch ports, SSIDs, VLANs,
  and DHCP leases from the controller with no SNMP, SSH, or admin credentials — a
  differentiator and a safety property. (A dependency risk too: the connector rides
  an unofficial API that shifts across UniFi OS versions.)

_Also in scope and expanded later in this PRD: single-binary footprint (one binary +
your MariaDB — no Redis, no workers, no queue, no proxy) and blast-radius awareness
("if this host dies, what breaks?")._

## Project Classification

- **Project Type:** Self-hosted, event-driven data-reconciliation engine; the
  discovery scheduler and observed-vs-declared diff are the core, with web UI, JSON
  API, and Prometheus `/metrics` as surfaces.
- **Domain:** Infrastructure / network management (IPAM + lightweight application
  CMDB + topology). No regulatory or compliance burden.
- **Complexity:** High (technical) — reconciliation/device-identity under MAC
  randomization, and an unofficial UniFi API shifting across versions. _(The dual-backend
  2× test matrix was removed by D64, 2026-07-17: MariaDB 10.11+ is the only engine.)_
- **Project Context:** Greenfield; solo, AI-assisted development. Author's own
  all-UniFi, Synology-hosted, multi-building network is the live proving ground (N=1).

## Success Criteria

### North-Star & Guiding Metrics

- **Declared coverage %** — the share of the observed network (devices/IPs seen)
  that is under declared intent, and its trend over time. Rising largely unassisted
  = the reconciliation bet holds; stagnating = the product isn't sticking. This is
  THE product metric.
- **Open-divergence count trends down** — unreconciled gaps are detected *and*
  resolved over time, not a perpetually growing inbox. **It never counts pre-baseline items.** Before
  the operator has baselined, there IS no divergence: there is nothing declared to diverge from. *A
  first run is an inventory, not a backlog* — "we found 340 things; is this your infrastructure?" is one
  gesture, not 340 decisions. **Counting them would poison the north-star from day one — and worse, it
  would tempt us to nag.** The count starts at zero, by construction, after baselining.
- **These metrics judge US, never the operator.** If open divergences rise across installs, **our** loop
  is broken — the triage is badly ordered, the questions badly posed. **The backlog is the operator's
  decision; the trend is our responsibility.** The success measure is not *"is the backlog emptied"* but
  *"is it TREATABLE the day he decides to treat it"*.
- **Both are computed locally per instance, shown to the user, and aggregated nowhere.** Cross-instance
  aggregation is rejected on principle, not on privacy grounds: **declared coverage % is a percentage
  whose denominator is defined by the user** — one operator declares 3 hosts, another 300; one declares
  everything, another only what matters. **"68% average coverage" has no referent: it is a category
  error, not a statistic.** The north-star is locally instructive and globally meaningless. It still
  judges us — through issues, through what people ask for, through what they say. **The loop closes by
  conversation, not by a server.**

### User Success

_Locally instrumented and surfaced as a self-diagnostic dashboard; these live in the
operator's own database and **leave it under no circumstances**. Targets are to be
calibrated against 5-10 real installs **through conversation with those operators** —
no invented thresholds on N=1._

- **Time-to-first-value (UniFi network):** populated inventory + first reconciliation
  findings, target < 15 min (hypothesis, instrumented).
- **Declaration sticks:** users keep promoting newly discovered devices to declared
  over time (not just once) via one-click *documenting* — the real signal the
  bet holds.
- **The gap stays actionable, not noisy:** the triage inbox trends toward resolved,
  and the "unknown device" false-positive rate stays low despite MAC randomization.
- **Concrete jobs succeed in seconds:** find a free IP in a subnet; identify *which*
  device owns a conflicting IP; **see what is hosted on a device before powering it down**
  (FR39, "Hosted here" — one hop). _(D57-scope: the traversal-based blast radius is Growth.)_

### Business Success

_Open-source, self-hosted, no revenue model — success = adoption, retention, and
community. Directional, to be calibrated after launch._

- **Vanity (not retention):** GitHub stars/forks. A star is a single, free, one-off event.
- **Retention, measured with what already exists — free, holding no data on anyone, with no failure
  mode of its own:**
  - **Docker Hub pulls TAG BY TAG over time.** Not vanity, and the PRD previously mis-classified them:
    **a pull of version N+1 by someone who already pulled version N is an act of maintenance on a
    running system** — literally the behaviour we want to measure. Noisy and aggregate, but its bias is
    publicly known and therefore socially corrected.
  - **Issues, discussions and PRs describing real usage.** *An issue saying "3 months in, 412 IPs, this
    divergence isn't resolving" cannot be written without having kept the product.* It yields retention
    **and** context **and** the use case **and** a human to talk to — **strictly superior to a
    heartbeat, which only proves a container is running somewhere. A forgotten container heartbeats for
    two years: a heartbeat counts zombies and calls it retention.**
  - **Asking.** The audience is a few hundred people, not a few million. *Telemetry is the instrument of
    products that cannot talk to their users. This one can.*
- **3-month goal — stated in integers, because at this scale integers are honest and percentages lie.**
  A percentage implies a denominator, hence a population, hence a sample, hence a bias; a count asserts
  nothing it cannot prove:
  > **At least 3 distinct people other than the author have run opencmdb on their own infrastructure and
  > produced a public artefact proving it** (issue, discussion, PR, or a post containing real output),
  > **of whom at least 1 returned ≥ 14 days after the first.**
  > **Plus, worth its weight: at least 1 bug report that could only have been produced by running the
  > software on infrastructure the author does not own** — impossible to fake, because it costs the
  > sender something.
  > *Instrument: issue/discussion authors ≠ the author, timestamped, counted by hand. Cost: nothing.
  > Falsifiable at D+90: it is 2 → failed; it is 4 → passed.*
  > *The threshold of 3 is an openly arbitrary bet. That is the point: **an arbitrary threshold on an
  > exact instrument can be debated honestly between humans; a biased instrument cannot — it does not
  > warn you that it is biased.***
- **12-month:** a growing base of *retained* self-hosted installs (same instruments), and the first
  external contribution (a community connector or substantive PR) — the leading indicator the
  connector-as-plugin bet is working.

### Technical Success

_Deterministic, CI pass/fail._

- **Engine (firm AC): MariaDB 10.11+ is the only supported engine, and it is a day-1 production
  target.** The first vertical slice must be green on MariaDB before any feature is stacked. The CI
  engine is pinned to `mariadb:10.11.11` — the exact DSM 7 package — so that **dev, CI and production
  are the same engine at the same version.** _(D64, 2026-07-17: SQLite and MySQL are both out. The
  claimed/tested gap is 0 — we do not claim what has no CI.)_
- **Identity comparison never descends into the engine (firm AC, and it is a CORRECTNESS gate, not a
  portability one):** value comparison and normalization happen in application code, never in SQL; time
  is computed in application code and bound as a parameter, never taken from the database; identifiers
  are generated by the application. **Held by: explicit binary collation on every text column + a CI grep
  over the DDL.** *Without it, `utf8mb4_general_ci` silently settles hostname equality — an identity
  anchor — and identity is the product.* _(D64 condition 1: this replaced the second engine, which was
  the only thing enforcing it before.)_
- **Reconciliation correctness:** the **binary trap suite of NFR4 passes with zero failures** across its
  three columns (must-not-merge / must-merge / must-abstain), at the **device** level. **False-merge and
  false-split do not carry equal weight**: a false merge is catastrophic and has no clean undo; a false
  split is benign and additive (two records instead of one, correctable). Statistical measures are
  published as observability, not as gates. Reconciliation cycles are idempotent and independent of
  ingestion order.
- **UniFi connector resilience:** the four assertions of NFR8 hold under fault injection — **monotone
  honesty above all: a faulted run cannot invent a single fact.** Verified at two layers, including
  **raw-byte replay through the real parser**, against a bounded, version-tagged, re-captured fixture
  matrix.
- **Bounded discovery performance:** a discovery cycle + diff for a few-hundred-host
  network completes within an explicit time budget without blocking the UI.
- **Security baseline:**
  - *In transit:* TLS is a **deployment concern** — opencmdb serves HTTP and is
    expected to run behind a reverse proxy that terminates HTTPS (documented
    recommendation, not an app feature).
  - *At rest:* the app encrypts/hashes stored secrets (UniFi API key, passwords) —
    a reverse proxy does **not** solve this.
  - *Auth:* the app owns authentication for the web UI, JSON API, and `/metrics`
    endpoint (scrape token).

### Measurable Outcomes

- **Externally verifiable:** Docker Hub pulls **tag by tag over time**; public artefacts of real usage
  (issues/discussions/PRs, counted by hand); the invariant suite green on both backends; **the trap
  suite at zero failures** on the labeled fixture; UniFi zero-manual-entry inventory in one polling
  cycle (reproducible on a reference rig); non-UniFi devices discovered by scan.
- **Locally instrumented — shown to the user, aggregated nowhere:** install < 30 min; first value
  < 15 min; declared coverage %; device-promotion cadence; triage-inbox resolution; abstention rate —
  all "to calibrate," with no hard N=1 numbers. **These are the operator's numbers, not ours: they exist
  to inform him, not to report to us.**

> Product scope (MVP / Growth / Vision) is defined canonically in **Project Scoping &
> Phased Development** below, alongside build sequencing and risk mitigation.

## User Journeys

_Unified human persona: "Marc", a technically confident operator of a
small-but-nontrivial network (advanced home-lab or SMB without dedicated IT). The
thread through every journey: **"I thought I knew what was running on my network."**
Each one demolishes a false certainty. A final journey covers a non-human consumer.
Read-only human stakeholders are a v2 concern (see Scope)._

### Journey 1 — First light, UniFi (happy path)

Marc pulls the container onto his Synology and pastes his UniFi controller URL and
API key. One polling cycle later the empty screen fills — 84 devices, each with its
switch port or SSID, VLAN, DHCP lease, none typed by hand. For a beat he freezes:
*84 lines — which ones matter?* Then the triage inbox reframes it. He hits
**document** on the ten that count — NAS, UDM, the studio printer. In under
fifteen minutes: a living inventory and three flagged gaps. He didn't document his
network; opencmdb did, and he confirmed it.

*Reveals:* setup wizard, UniFi connector, discovery scheduler, triage inbox,
documenting, occupancy view, first-value instrumentation.

### Journey 2 — First light, no UniFi (happy path for the majority)

Nadia runs a Proxmox + Mikrotik home-lab; no UniFi controller. She declares her
subnets and opencmdb runs an ARP + ping sweep. The inventory that returns is honest
but partial: MACs, IPs, some hostnames from mDNS/reverse-DNS — but no switch ports,
no SSIDs, no VLAN truth. opencmdb says so plainly rather than faking confidence. She
enriches the twenty that matter by hand, accepts them as declared, and still gets the
core value: occupancy, conflicts, stale-IP alerts. The UniFi user gets more for free;
the non-UniFi user is a first-class citizen, not an afterthought.

*Reveals:* subnet declaration, generic scanner enrichment (mDNS/reverse-DNS),
explicit source-capability limits shown to the user, manual device enrichment,
parity of core value without UniFi.

### Journey 3 — The 11 p.m. conflict (edge case, recovery)

A ntfy notification: *IP conflict on 10.0.20.0/24* — a deep link. Marc taps it and
lands, on his phone, precisely on the conflict: two MACs on `.47` — his declared NAS
and an unknown device from yesterday with a randomized MAC (`android-8f2c`).
opencmdb couldn't auto-match it, so it didn't guess and didn't merge — it surfaced
the ambiguity with evidence (first seen, the AP joined, IP history). Marc recognizes
a colleague's new phone, excludes it (out of scope, remembered), and the false alarm never returns.

*Reveals:* deep-linked mobile alerts, conflict detection with device identification,
reconciliation under MAC randomization, ambiguity + evidence, remembered decisions.

### Journey 4 — When the source goes blind (critical failure path)

At 3 a.m. Marc's UniFi API key is rotated by a firmware update. opencmdb's next poll
fails. The dangerous product would silently mark 84 devices "disappeared" and fire 84
stale-IP alerts. opencmdb doesn't: it detects that a *source*, not the network,
went dark. The dashboard turns a source-health indicator red — "UniFi unreachable
since 03:12" — pauses observation-derived alerts for that source, and holds the last
known state instead of fabricating divergences. Marc wakes to one honest alert, not a
panic wall. He re-issues the key; the source comes back green. Trust survives because
the engine knew the difference between "I'm blind" and "they left."

*Reveals:* per-source health/liveness detection, source-availability-aware alerting
(suppress divergence/stale alerts when a source is down), last-known-state retention,
a single "source unavailable" alert, clear recovery.

> **This journey is promoted to a named, binding requirement — it is the trust thesis in one line, and
> a story cannot fail a build:**
> **An authentication rejection from a source must produce exactly ONE source alert, naming the credential
> rejection and its remedy, and ZERO device alerts.**
> It must be a test that **fails if "the source refused us" is merged with "the source did not answer"** —
> two conditions with the same liveness verdict and **opposite operator actions**. Merging them tells the
> operator "something is wrong"; separating them tells him "your key was rotated by the firmware." *That
> is the difference between a product and a dashboard.*

### Journey 5 — Back after three weeks (re-engagement)

Marc was heads-down on a deadline; he hasn't opened opencmdb in three weeks. The
inbox has 61 items. The old tool would guilt-trip him into abandonment. Instead the
dashboard leads with *what changed and matters*: 4 genuinely new devices, 2 conflicts,
the rest routine DHCP churn he can bulk-document or bulk-exclude in two clicks. Ten
minutes and he's caught up, not buried. The tool respects that he has a life.

*Reveals:* "what changed since last visit" view, bulk triage (document/exclude),
change prioritization (new/conflict vs routine churn), absence-tolerant UX.

### Journey 6 — Sunday documentation (consultative)

Sunday coffee. The self-diagnostic dashboard shows declared coverage at 72%, up from
last week; three open divergences. Marc needs a free IP for a new Pi — the per-subnet
occupancy view hands him `.53` in two clicks. Then he documents the studio's
document-management app: creates **"Paperless"**, attaches Paperless-ngx + PostgreSQL
+ Redis. Before powering the NAS down for maintenance, he opens the NAS record and
**"Hosted here"** lists the three of them, with the application's owner and criticality.
He shuts it down with confidence, not dread.

*Reveals:* self-diagnostic dashboard, free-IP lookup, application/software CRUD,
containment relationships, the device record's "Hosted here" panel.

_(D57-scope, 2026-07-17: this journey previously had Marc declare `depends_on`/`connects_to` and read an
**impact view**. Both are GROWTH now. **His question — "what breaks if I power this off?" — is answered at
MVP by one containment hop, and FR39 already carried it.** The journey is titled "Sunday documentation
(**consultative**)", and it was mistaken for the binding one during the D57 debate — the binding journey is
Journey 4, "when the source goes blind".)_

### Journey 7 — The green panel by the bed (integration consumer, non-human)

Marc doesn't want more dashboards to babysit — he wants to sleep. He points Prometheus
at opencmdb's token-authenticated `/metrics`; a Grafana panel now shows free IPs per
subnet, open-divergence count, and inbox size on the same wall as everything else. A
Prometheus rule pages him only if divergences climb. A small script pulls the
read-only JSON API to cross-check devices against his backup inventory. opencmdb earns
its keep by staying quiet and green — surfacing only when something genuinely needs him.

*Reveals:* authenticated Prometheus `/metrics`, opencmdb-specific gauges, read-only
JSON API, generic webhook, "quiet until it matters" positioning.

### Journey Requirements Summary

Collectively the journeys require: (1) setup wizard + UniFi connector + generic
scanner with subnet declaration; (2) discovery scheduler; (3) reconciliation with
triage inbox, documenting, ambiguity surfacing, remembered decisions, and
**bulk triage**; (4) IPAM occupancy + free-IP lookup; (5) conflict detection with
device identification; (6) **per-source health/liveness detection and
source-availability-aware alerting** (never fabricate divergences when a source is
down); (7) explicit, user-visible **source-capability limits** (non-UniFi is partial
but honest); (8) deep-linked mobile+desktop alerts + generic webhook; (9) a
**"what changed since last visit"** view; (10) applications/software with **containment
relationships + "Hosted here"**; (11) self-diagnostic dashboard; (12) read-only JSON API
+ authenticated Prometheus `/metrics`; (13) observation history + last-known-state
retention.

## Domain-Specific Requirements

_Domain is technically demanding but carries no regulatory regime of its own. These
are technical/operational constraints intrinsic to a self-hosted network-discovery
tool._

### Network Discovery Constraints
- The generic ARP scan requires `NET_RAW` (or host networking) in a container; the
  UniFi connector needs no privilege and is preferred where available. Scanner
  topology is an architecture decision; **MVP default = single container**, with the
  scanner built as a **separable component** (a privileged sidecar remains a later
  architecture option).
- **If `NET_RAW` is unavailable, the scanner degrades to ping-only and reports a
  capability-downgrade status** — it does not crash and does not fail-fast.
- Discovery cadence is configurable within explicit **min/max bounds**; overlapping
  scans of the same source are **coalesced, not stacked**; a per-host probe timeout
  and a bounded total sweep time per subnet are specified.
- Local segments use ARP+ping, routed subnets use ping sweep; hostname enrichment via
  mDNS/reverse-DNS; IPv6 documented, not actively scanned at MVP.
- **Scan reach is a layer-2/layer-3 TOPOLOGY assumption, not merely a privilege question.** ARP is
  layer-2: a NAS on one VLAN **cannot ARP 36 subnets** unless it is trunked onto each. The ping fallback
  crosses routed subnets and depends on the router. **The real axis is reachability, not the NET_RAW
  toggle** — and NFR1's time budget is unattainable for many deployments if this is left implicit.
  **Documented as a deployment prerequisite, with the reachability the reference figures assume.**
- **Source-capability transparency:** each source publishes a machine-readable
  capability descriptor (e.g. `{ports, ssids, vlans, dhcp, mac, ip, hostname}`
  booleans) that drives what the UI claims to know; non-UniFi is partial but honest.

### Self-Hosted Operational Constraints
- **One binary + your MariaDB. No Redis, no workers, no queue, no proxy.** MariaDB 10.11+ is the only
  supported engine (Synology-native and included in Synology's automatic backups); SQLite and MySQL are
  not supported; PostgreSQL is not supported at MVP. **The database is a prerequisite, stated as one:**
  the reference deployment is a two-service compose, and the product does not claim otherwise anywhere.
  All connector writes go through a serialized writer, with the read and write paths separated so that
  reads never contend for the write path (NFR1).
- Docker (Synology Container Manager) is the priority deployment; native binary also
  supported. 12-factor config (file + environment variables). `external_base_url` is
  configurable (tolerant fallback + visible warning). TLS termination is a deployment
  concern (reverse proxy); the app serves HTTP. Internal tokio scheduler — no external
  cron, Redis, or worker services.

### Data Sensitivity & Threat Model
- Stored data is a full network map plus infrastructure credentials — sensitive
  regardless of any compliance regime.
- **Threat model (named, and split — see NFR9a/9b/9c):** opencmdb protects data and credentials
  against a stolen or exfiltrated **database file** and against unauthenticated network access. It does
  **not** defend against a local root attacker. **It does NOT claim to protect a backup that contains
  both the database and the key — that is stated as an explicit non-guarantee, because the claim cannot
  be enforced and an unenforceable claim destroys the vigilance it pretends to replace.**
  The key is supplied out-of-band from a **file in a separate top-level shared folder**, mounted
  read-only, auto-generated on first start. **An environment variable is rejected** on two independent
  grounds: it lands in a compose file that backup tools carry away, and it leaks through process
  inspection and host logs. **A startup prompt is rejected as the default** — it would break
  install-and-forget, leaving a dead service after every power cut — but remains available as an option.
  **The application refuses to start if the key path resolves inside the data volume**: we cannot prevent
  a bad backup configuration, but we can prevent the bad configuration we can see.
- Credentials (UniFi API key, passwords) encrypted/hashed at rest; all HTTP surfaces
  authenticated; configurable observation retention (default 90 days).
- **API-key rotation and encrypted secret backup/restore are MVP.** A full
  change-audit log is v2, but the MVP logs security-sensitive events (authentication,
  secret access/rotation) to the application log. Targeted per-host export/deletion is
  covered by CRUD at MVP; bulk export is a growth feature.

### External Dependency Risk
- The UniFi local API is unofficial and shifts across UniFi OS versions — the
  connector is isolated behind a trait, contract-tested against version-tagged
  fixtures, and **degrades to a defined partial/last-known state** (not a crash) on
  version drift.

### Compliance & Regulatory
- No regulated data class is targeted, and opencmdb is **neither a data controller nor
  a processor — it is a software provider**. Where the operator (e.g. an SMB) is subject
  to GDPR because device/host identifiers relate to people, opencmdb is **designed to
  support that compliance** (export, targeted deletion, retention control, secret
  handling) rather than claim exemption.

## Innovation & Novel Patterns

### The Core Contradiction It Resolves

A network must be documented to be governed, yet must **not** be documented *manually*,
because the effort of hand-maintenance is exactly what makes documentation rot. opencmdb
resolves this by pairing two innovations — one structural, one interactional:

- **Observed and declared as a linked, never-merged model (structural).** Hand-entered
  intent and timestamped observation are kept as distinct truths; reconciliation is an
  explicit *link* layer, not a destructive merge. This is what makes drift analysable over
  time and what makes source-availability-aware reasoning possible — the engine can tell
  "a source went blind" from "a device actually left," and refuse to fabricate divergences.
  It is the thesis; without it, opencmdb is just another scanner.
- **Documenting in one gesture (interactional).** One click promotes a discovery into declared
  documentation, collapsing declaration effort toward zero. This is the resolution of the
  effort-vs-upkeep contradiction: the machine observes, the human ratifies. It is what makes
  the structural model usable by an audience that would never maintain a manual source of truth.

Together: the never-merged model is *why reconciliation is possible*; documenting is
*why anyone will actually keep it true*.

### Defensibility / Moat

The durable edge is **not** a feature — an incumbent could clone reconciliation or
documenting in a sprint. It is **radical simplicity as the product itself**. A
competitor's binary is still bound to its DNA: even if NetBox shipped a reconciling binary
tomorrow, it would carry NetBox's complexity — its enterprise data model, its learning
curve, its operational weight. That complexity is intrinsic to serving the enterprise, and
it is precisely what opencmdb's audience is fleeing. opencmdb's moat is that *simplicity is
the design center*, not a packaging decision — a home-lab-native, single-binary, zero-friction
footprint an enterprise vendor cannot adopt without abandoning its value anchor. Zero install
friction is also the adoption engine for a revenue-free open-source project.

### The Key Unsolved Leverage Point: Device Identity

MAC as a primary key is unstable (randomization, cloned/virtual MACs, NIC swaps). Until
identity is anchored on a **composite invariant** — MAC + hostname + IP/DHCP history +
connection topology (switch port / AP) + optional service fingerprint — the "gap" degrades
into noise and documenting risks ratifying phantoms. This is the highest-leverage
design problem in the product and directly gates reconciliation precision/recall (see
Success Criteria). Detailed strategy is deferred to the Architecture phase; **the PRD commits
to a composite-identity approach, not raw MAC.**

### Market Context

Only NetBox Assurance markets continuous reconciliation (paywalled, heavy stack); source-of-
truth tools don't auto-discover; discovery tools don't model declared intent; only NetAlertX
touches UniFi (presence only). The lightweight middle — discover *and* reconcile — is unserved.

### Validation Approach

- Reconciliation precision/recall on a labeled fixture dataset (MAC randomization, multi-NIC,
  shared-hardware VMs) gates release — and is the concrete test of the identity strategy.
- Dogfooding on the author's live network + light community validation tests the declaration
  hypothesis; the declared-coverage % north-star is the live signal per install.

### Innovation Risk Mitigation

- **If device identity can't be stabilised:** reconciliation quality caps out; the precision
  target gates release and forces the composite-identity work before scale.
- **If documenting doesn't drive declaration:** the product degrades gracefully into a
  competent discovery/IPAM tool; the north-star flags it early.
- **If the moat is contested (incumbent ships a free binary):** compete on footprint, UniFi-
  first depth, and UX discipline — not on feature parity.

## Web & API Surfaces

### Overview

opencmdb is an event-driven reconciliation engine exposed through two surfaces: a
self-hosted **web UI** (an authenticated admin tool, not a public site) and an **HTTP
API layer** (read-only JSON + a Prometheus `/metrics` endpoint). SEO, public
multi-tenancy, and app-store concerns do not apply.

### Frontend Architecture

- **Open decision (surfaced from the brief):** HTMX + server-side templates **(Askama
  preferred — Jinja-like, compile-time-checked, readable/maintainable)** vs an SPA
  (Svelte/Vue). **PRD lean: HTMX + Askama templates for MVP** — one Rust binary, almost no
  JS build toolchain, fits the "simplicity is the product" moat. The v2 interactive topology
  map, if it needs rich client-side rendering, is added as an **isolated JS island** on an
  HTMX page rather than forcing an SPA now. Final call deferred to Architecture, but the PRD
  assumes HTMX + Askama unless overridden.
- Server-rendered pages with progressive enhancement; **responsive for mobile + desktop**
  (deep-link targets must render well on a phone, per the journeys); no SEO (auth-gated
  internal tool).
- Browser support: current evergreen browsers (Chrome, Firefox, Safari, Edge); no legacy.
- Accessibility: WCAG 2.1 AA on the key views (see NFR25), with semantic HTML, keyboard
  navigation, and adequate contrast throughout — not a full-app formal certification.

### Real-Time / Reactivity

- The product is event-driven; the triage inbox, alerts, and self-diagnostic dashboard
  benefit from live updates. **MVP: HTMX fragment polling** (e.g. `hx-trigger="every 10s"`)
  for those views — near-zero cost, no persistent connections, and tolerant of an
  imperfectly configured reverse proxy. **SSE is a clean later upgrade** (the HTMX SSE
  extension drops in without a rewrite) if true push is wanted; WebSockets are not required.

### API & Integration Surface

- **Endpoints (MVP):** read-only JSON over core entities (devices, interfaces, subnets,
  IPs, applications, alerts, observations) + the Prometheus `/metrics` endpoint. Write
  access is v2.
- **Data format:** JSON with a consistent envelope; ISO-8601 timestamps; stable entity
  **UUIDs** (the same identifiers that power deep links).
- **Auth:** session cookie for the UI; bearer token for the JSON API; a separate scrape
  token for `/metrics` (all authenticated, per the threat model).
- **Rate limits:** not needed at MVP (single-operator, self-hosted); a sane per-token cap
  arrives with the stable public API (v2).
- **Versioning:** unversioned / best-effort at MVP; `/v1` contract + OpenAPI docs at v2.
- **No SDK at MVP** — JSON suffices; the audience scripts directly against it.

### Implementation Considerations

- **API-first backend:** the UI (HTMX now, possibly an SPA later) is a client of the same
  API — which structurally protects the deferred frontend decision.
- Deep-link routes resolve to authenticated, focused object views keyed by entity UUID.
- i18n: externalized EN/FR strings across templates from day one.

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

- **Approach: a problem-solving MVP.** The single goal is to prove the reconciliation loop
  — discover → reconcile → document → actionable gap — works and earns *trust* on
  a real network. The boundary is the smallest surface a real operator would trust while
  proving the thesis (observed vs declared, never merged).
- **Validated learning:** the declared-coverage % north-star per install + dogfooding on the
  author's live network + light community validation.
- **Delivery capacity:** developer + AI-assisted (Claude Code). MVP scope is therefore bounded
  by **product coherence and trust, not solo-developer effort** — effort-based de-scoping is
  explicitly rejected.

### MVP Feature Set (Phase 1)

**Core journeys supported:** all seven (J1–J7) — together they define the trust surface.

**Must-have capabilities:**
- Device inventory (multi-interface; **composite identity**, not raw MAC).
- IPAM: subnets, VLANs, DHCP ranges, per-subnet occupancy, free-IP lookup.
- Discovery: zero-privilege **UniFi connector** + generic **ARP/ping scanner** (NET_RAW→
  ping-only fallback), scheduler (bounded/coalesced cadence) + on-demand.
- Reconciliation: composite matching at two levels (interface identity, then device grouping);
  triage inbox with **six gestures — document / accept-gap / create / attach / exclude / snooze**
  (remembered); ambiguity surfacing with candidates and evidence; bulk triage; "what changed since
  last visit".
- Source health/liveness + source-availability-aware alerting (never fabricate divergences).
- Alerts (unknown device, stale IP, IP conflict *with device identification*) → in-app +
  generic webhook, each a stable deep link (mobile + desktop).
- Applications/software + **containment relationships (`hosts`, `exposes`) only** + **"Hosted here" on the
  device record (FR39)**. _(D57-scope: `depends_on`/`connects_to` and the impact traversal are GROWTH.)_
- Topology as a structured list/table (auto for UniFi, manual otherwise).
- Self-diagnostic dashboard (declared coverage, open divergences, inbox health).
- Read-only JSON API + authenticated Prometheus `/metrics`.
- Auth (session + tokens); credentials encrypted at rest; API-key rotation + encrypted secret
  backup/restore; security-event logging.
- Configurable observation retention (default 90d) + permanent first/last-seen + IP-history
  rollups.
- Bilingual EN/FR UI (HTMX + Askama, polling); IPv6 documented (not scanned).
- **Ephemeral-interface lifecycle (FR38b)** — without it the divergence metric drifts upward forever.

### Build Sequencing (riskiest-first)

Scope is bounded by product coherence and trust, not effort (see MVP Strategy); what remains
capacity-independent is build ORDER:

- **Milestone 0 — thin vertical slice (for early real feedback):** UniFi connector →
  composite-identity + linked-never-merged schema → document → gap view. This is
  the smallest thing that proves the thesis on a live network.
- **Riskiest-first order:** (1) composite identity + linked schema; (2) UniFi → resolution;
  (3) reconciliation/triage engine; (4) periphery (IPAM, applications, topology, alerting,
  dashboard, i18n). _(D64: "on a portable SQL abstraction" struck — there is one engine.
  D57-scope: the impact traversal left the MVP.)_
- **Engineering discipline mandated from day 1 (not deferrable):** a portable SQL layer with no
  backend-specific dialect, **and MariaDB green from the first vertical slice — not activated later**
  (portability is paid while the schema is tiny; deferring it means migrating when it costs most, and
  the dual-backend suite is also the differential test that catches persistence-layer defects); the
  UniFi connector isolated behind a source interface with contract-tests against version-tagged
  fixtures. **The labeled identity fixture is written before the engine** — it defines the shape of the
  source interface, so deferring it means designing that interface twice. **The measurement harness is
  written before the engine too: a metric written afterwards is bent to fit the engine.**
- **Composite identity:** the framework is composite/extensible from day 1, **and it operates at two
  levels** (interface identity, then device grouping — see FR9). The MVP signal set is hardware address
  + hostname + IP/lease history + connection topology, with **service fingerprint sequenced alongside
  Growth's service scanning** (a coherence dependency, not an effort cut). **Structural facts about an
  address are read at ingestion, never scored** — scoring is spent only on what is genuinely uncertain.

### Post-MVP Features

**Phase 2 (Growth):** email + ntfy/Telegram channels (reusing deep links); CSV / NetBox /
phpIPAM import; active IPv6 scanning; read-only user role; stable **versioned public API +
write access + OpenAPI**; additional connectors (Omada, Mikrotik, OPNsense/pfSense, Proxmox,
Docker) via a documented plug-in contract; full change-audit log; bulk export; **port/service
scanning + service-fingerprint identity signal**.

**Phase 3 (Vision/Expansion):** interactive **graphical topology map** (isolated JS island);
SNMP/LLDP topology for non-UniFi gear; consuming Prometheus/monitoring data for reverse
enrichment; a community-connector ecosystem.

### Risk Mitigation Strategy

- **Technical:** composite device identity is the riskiest core → precision/recall gate on a
  labeled fixture set before scale; comparison and normalization kept out of SQL and enforced by a
  DDL collation gate from day 1; the UniFi connector isolated behind a trait,
  contract-tested, and degrades gracefully.
- **Market:** N=1 adoption risk → light community validation + the declared-coverage
  north-star; documenting lowers the declaration barrier the whole bet rests on.
- **Resource:** delivery is developer + AI-assisted (Claude Code); the primary control is
  **riskiest-first sequencing and a stable foundation, not scope reduction**.

## Functional Requirements

_The binding capability contract for the MVP: any capability not listed here will not be
built unless explicitly added. Multi-verb FRs are atomized into acceptance criteria at story
time, not renumbered here._

### Discovery & Data Sources
- FR1: The operator can connect a UniFi controller as a discovery source via its URL and an API key.
- FR2: The system can discover devices, IPs, switch ports, SSIDs, VLANs, and DHCP leases from a UniFi controller without elevated network privileges.
- FR3: The operator can declare subnets to be scanned by a generic ARP/ping source.
- FR4: The system can discover active devices on declared subnets and enrich them with hostnames where available.
- FR5: The system represents each source's state along **two independent axes** — **liveness** (`live` / `blind`, with a named cause) and **capability** (the descriptor currently in force) — per **(source, scope)**, where a *scope* is the smallest set that can go blind independently (one scope per subnet for the generic scanner; one scope for a UniFi controller). `full / degraded / offline` is a **derived presentation** of the two axes (`blind`→offline; `live`+reduced→degraded; `live`+full→full), not a stored state. The system reports capability downgrades as notifiable events.
- FR6: The operator can configure per-source discovery cadence within defined bounds and can trigger an on-demand scan.
- FR7: Each source exposes a capability descriptor of what it can and cannot observe, surfaced to the operator. **The descriptor is dated and travels with each batch of observations — it is not a static property of the source**, because capability can shrink at runtime (a scanner losing its raw-socket privilege keeps answering, it simply sees less). Consequently: **a capability reduction is a notifiable event** carrying its cause and its remedy; **observations are always interpreted under the descriptor in force when they were collected** (re-interpreting yesterday's observation under today's descriptor would make the past change status); and the descriptor is what prevents the engine from confusing *"this field is absent because there is nothing"* with *"this field is absent because this source is blind to it"* — **the second, read as the first, is a false merge.**
  **The descriptor is what makes a per-connector capability difference legal rather than a defect, and the first measurement confirms it is needed** (reference device, 2026-07-16): the UniFi source exposes addresses, addresses-to-switch-port and addresses-to-access-point **connection topology (present on the large majority of wired clients — the topology signal L2 depends on), vendor and lease data — but NO round-trip latency at all.** Latency is a **scanner** fact, not a UniFi fact. **No single source emits every observable; the descriptor is the mechanism that already says so, and it must be filled from measurement, never from the union of what we wish existed.**
  > **🔴 The trap this creates, named because it is silent:** the UniFi payload carries a **`satisfaction` score on nearly all of clients. It is not a latency.** An implementer looking for round-trip time will find a plausible number at near-total coverage and map it. **It compiles, it is populated, and it is meaningless.** *A field that is present, well-typed and wrong is more dangerous than one that is missing — the missing one fails loudly.* **Descriptor entries are filled from what a source MEASURES, never from what its payload happens to contain.**
- FR8: The system distinguishes a source outage from genuine device disappearance.

### Reconciliation & Device Identity
- FR9: The system stably identifies a device across changing IP and MAC (composite identity), not MAC alone. **Identity is established at TWO levels, because a hardware address identifies an INTERFACE, not a device** (FR43 already treats interfaces as first-class, and the MVP is explicitly multi-interface):
  - **(L1) Interface identity** — "is this network interface the same as yesterday?" Primary signal: the hardware address, scoped to its layer-2 domain. Attacked by address randomization, cloned/spoofed addresses, and ephemeral container interfaces.
  - **(L2) Device grouping** — "are these three interfaces the same host?" Signals: hostname, connection topology, lease history. Attacked by multi-NIC hosts, VMs on shared hardware, and shared virtual addresses (a redundancy protocol gives **one address to two physical routers**).
  This is not added complexity: the reference NAS is two bonded ports plus container interfaces; an access point exposes one wired and two radio addresses; a laptop has wired, wireless and a randomized address per network. **These are the median case, not the exotic one.** Without the split, a laptop counts as two devices — the operator sees 340 where he has 300, **stops trusting the inventory, and therefore stops trusting the gap.**
  **Both levels are inferred**: an interface is not equal to an address (a per-network randomized address produces two distinct observations of the same physical interface). **Structural facts about an address — that it is locally administered, or reserved by a redundancy protocol — are known at ingestion and never scored.**
  **Signal availability is measured, and it bounds the outcome before the engine does** (reference device, 2026-07-16): **hostname — one of L2's three signals — is unusable on nearly half of known clients** (absent on 71, empty string on 11, and **never null**: two of the three plausible shapes occur, the third does not, so a trap encoding it would test a case the source cannot produce). ⇒ **The abstention rate is bounded below by DATA AVAILABILITY, not by the engine's quality.** *An engine cannot group by a hostname that is not there — and no amount of correctness recovers a signal the source never sent.* A grouping quality target that ignores this measures the network, not the engine.
- FR10: The system reconciles observed data against declared records by identity.
- FR11: The system keeps declared and observed data as distinct, linked records; observations never overwrite declared data.
- FR12: The operator can review unreconciled discoveries in a triage inbox.
- FR13: The operator can **document** a discovery in one action — writing the observed values into the declared record — in the two distinct cases the loop actually produces:
  - **(a) New discovery, no declared record:** the whole record is documented at once (`document-all`). This is the day-one case.
  - **(b) Re-discovery of an ALREADY-DECLARED device that has drifted:** documenting is **field-selective** (`document-field`) — *"the gap becomes the norm, and the operator decides which fields become the documentation."* **This is the dominant long-run case** — once a network is documented, most inbox items are re-discoveries, not new devices — and a single blanket action here would destroy operator-authored knowledge (name, owner, notes) that no scan can reproduce.
  - **Invariant, both cases:** the observed record is **never modified**, the link is preserved, and only the declared record is written, by an explicit human action. Documenting composes the declared record field by field; it never fuses the two records (see *Innovation & Novel Patterns*). A field documented today can diverge again tomorrow — the divergence computation never consults how a declared value was obtained.
  - **UI terminology (FR/EN):** the French interface names this gesture **« Merger »**; English documentation, API and code use **`document`**. One term, one translation — the pair is fixed and neither language carries two meanings.
- FR14: The operator can, from the inbox: create a new record, attach a discovery to an existing device, exclude it (out of scope), snooze it (defer until a chosen time), or **accept the gap**.
  - **`accept-gap` — "I have seen it, I have not yet decided what to do."** The gap **stays OPEN and keeps counting**; the item leaves the inbox and **re-surfaces only when the observed value CHANGES — never on a clock.** *Snooze expires on a date; accept-gap expires when reality moves: one is a timer, the other a sensor.* A **note is mandatory** (a gesture that costs ten seconds of thought is never the lazy default, and the note is the only thing that will help in six months). **No penalty attaches to it**: it silences nothing — it says "tell me if reality changes", i.e. *I am still listening*.
  - It fills a job no other gesture covers: *when a divergence cannot be resolved now, take note of it without lying, and move on without losing it.* Accepting would close what was not decided; ignoring says "I don't want to know" when the operator does; snoozing is a timer when he wants a note.
  - **`accept-gap` is a waypoint, not a terminal state: accepting can lead to documenting.**
  - **Metric integrity:** `accept-gap` does not raise the open-divergence count — the item was already open. **Folding this case into "document" would make the count go DOWN: a gap that exists, recorded as closed. That is the poison this gesture prevents.**
  - **UI terminology (FR/EN):** French **« Accepter l'écart »**, English **`accept the gap`**. Canonical definition, verbatim: *"accepter une divergence qui devra être résolue."*
- FR15: The system remembers triage decisions (including exclusions) so resolved items do not reappear.
- FR16: When identity is ambiguous, the system presents the candidate matches with supporting evidence and marks the item unresolved, without guessing or merging. **"Unresolved" is a persisted record carrying its candidates and their evidence — not an absence.** Presenting candidates with their evidence is impossible if the candidates are stored nowhere. **Abstention is therefore a first-class outcome of the matching engine, not an error path**: it must be possible to say *"I don't know"* and to show why. *"I don't know" does not erode trust; "I know" followed by "actually, no" erodes it irreversibly.*
- FR16b: **Abstention is displayed, counted, and grouped by CAUSE — and it is never a reproach.** FR16 makes abstention a first-class outcome of the *engine* and a persisted record *per item*. That is not enough: the operator does not experience one abstention, he experiences their **total**. The system therefore displays, beside the evaluated population, **the count of devices it did NOT evaluate, broken down by the reason it could not** — no live source on the scope, address grouping unresolved, a signal absent from every capable source. **Each cause is one line and one gesture, not N failures.** *96 multi-interface devices is not 96 failures — it is ONE question.* This is the same rule as the regime-dependent question granularity (UX spec — the motif in bootstrap/migration, the field in steady state) applied to abstention rather than to doubt: the number of records is unchanged, the number of human gestures drops by two orders of magnitude. Bulk triage (FR17) is the gesture.
  **The counter is bound by the backlog bans (UX spec — *Dignity*: no error counters, no "X problems" badges, no health gauge, no age brandished as reproach):** it does not redden, it does not grow bold, it does not age visibly, it carries no gauge. **It measures the product's REACH, not the operator's debt** — if the operator does nothing for six months it still reads the same number, in the same grey, with the same dignity. *A product that says "I don't know" and says WHY is a map. One that shows a blank field is an admission of failure. The difference is not the number — it is whether the cause is actionable.*
  **Measured, not assumed** (reference device, 2026-07-16): the abstention floor is set by **data availability, not by engine quality** — see FR9 and NFR30.
- FR17: The operator can triage multiple discoveries in bulk.
- FR18: The operator can view what changed since their last visit, prioritized (new / conflict vs routine churn).
- FR19: The system suppresses observation-derived alerts and divergences for a **(source, scope)** whose **liveness is `blind`** — whatever the cause — retaining last-known state. "Unavailable" is defined mechanically: **every source-error condition sets `blind` or leaves `live`-with-reduced-capability, and in neither case does an absent observation produce a divergence or a change event.** A capability reduction suppresses divergences **only on the fields that capability covered**, not globally.
- FR20: When sources report conflicting observations for the same entity, the system surfaces the disagreement with both values tagged by source — it does not silently pick one, and the disagreement never propagates to declared data. **The conflict only fires when TWO SOURCES CAPABLE OF OBSERVING THAT FIELD disagree**: the capability descriptor (FR7) filters first. *A controller that says "not in my table" and a scanner that says "it answers a ping" are not in conflict — they measure different things; that is a difference of scope, not a contradiction.* **A source that cannot know contributes nothing to the arbitration; it does not lose it.** (Configurable source-precedence/trust ordering is a Growth feature — and deliberately so: **a precedence rule is a merge rule in disguise** — *"when they disagree, believe that one and discard the other"* — which is what "linked, never merged" forbids, applied to sources instead of fields.)

### IP Address Management
- FR21: The operator can manage subnets, VLANs, and DHCP ranges.
- FR22: The operator can view per-subnet IP occupancy (used / free / declared / observed).
- FR23: The operator can find a free IP address within a subnet.
- FR24: The system detects IP conflicts (same IP on two MACs, or a static-declared IP inside a DHCP range) and identifies the devices involved.
- FR25: The operator can document IPv6 subnets and addresses (observation-only; active IPv6 scanning is out of MVP scope).

### Applications & Hosted Software
_(Section renamed 2026-07-17 by D57-scope: "Impact Analysis" named a traversal that is now Growth. **The
word Impact does not go on a one-hop screen** — FR7's `satisfaction` rule.)_
- FR26: The operator can record software instances (name, version, listening ports) hosted on a device.
  **The anchor to the device follows D15 in full** — the edge is a human testimony about a referent the
  engine migrates, so `entity_id` is never rewritten (**AC-M-04**: split a device hosting 3 software ⇒ the
  answer does not shrink in silence).
- FR27: The operator can group software into applications with an owner and criticality.
- FR28: **SPLIT by D57-scope (2026-07-17), and the line is not arbitrary.**
  - **MVP — `hosts` and `exposes`:** the operator can declare these. **They are containments, not edges**
    (FR26 restated), and they are **the two that Growth's port/service scanning will one day OBSERVE.**
  - **GROWTH — `depends_on` and `connects_to`:** sequenced with the port/service scanning, **a coherence
    dependency, not an effort cut.** Nothing observes them today, and *"a declaration nothing will ever
    re-read is the only field in this product with no loop behind it: it can only rot, and nothing is
    watching."*
  > **THE RULE, binding: a relationship the product may one day OBSERVE is a DECLARATION. A relationship
  > nothing will ever observe is a BELIEF. The MVP ships declarations. Growth ships beliefs the day an
  > observer exists to contradict them.**
- FR29: **At MVP this is FR39, and it is named "Hosted here" — never "Impact".** The device record answers
  Marc's real question (*what runs on this machine, whose is it, how critical*) with **one containment hop,
  no traversal**. **The word "Impact" is refused on a one-hop screen:** a view called *Impact* that concludes
  *"nothing else is affected"* is FR7's `satisfaction` trap with a nav entry — *present, well-typed and
  wrong is more dangerous than missing; the missing one fails loudly.* **The true impact view (traversal
  over `depends_on`) is GROWTH**, beside the interactive topology where the UX spec had already placed it.
  _(Decided by Guy after the D57 party mode. FR26/FR27 at MVP are also **the probe**: in six months we count
  what was actually typed — 11 instances and the graph is a `SELECT`; 300 and it is a real decision, taken
  on a fact.)_

### Alerts & Notifications
- FR30: The system raises alerts for an unknown device appearing, a documented IP unseen for N days, and an IP conflict.
- FR31: The operator can receive alerts in-app and via a generic outbound webhook.
- FR32: Every alert carries a stable deep link that opens the exact related object in a focused view.
- FR33: The operator can act on an alert (resolve / accept the gap / exclude) from the object it links to.
- FR34: The operator can configure alert thresholds (e.g. stale-IP window, unknown-device criteria) and can mute or snooze specific alerts.
- FR35: The operator can configure which alert types are delivered through which channel (in-app; generic webhook at MVP).

### Insight, History & Reporting
- FR36: The operator can view a self-diagnostic dashboard reporting source health, reconciliation lag/queue, declared coverage, open-divergence trend, and inbox health.
- FR37: The system records timestamped observation history per device (first seen, last seen, IP↔MAC history).
- FR38: The operator can configure **observation** retention (default 90 days); first/last-seen and IP-history rollups are retained indefinitely. **This governs OBSERVATIONS only — the lifecycle of inferred entities is FR38b. (Without that cross-reference the reader concludes, wrongly, that inferred entities are eternal.)**
- FR38b: **Ephemeral-interface lifecycle.** An interface whose hardware address is locally administered — i.e. randomized by the device's operating system, a structural fact readable at ingestion, not an inference — and which has not been observed for a configurable window (default 30 days) moves to a **`dormant`** state: **excluded from divergence metrics and from automatic candidate generation, still queryable, retaining first/last-seen and address history indefinitely** (consistent with FR38), and **returning to active if the address is observed again — same entity, not a new one**.
  **This is not an edge case, and the measurement says so** (reference device, 2026-07-16): **a large share of known clients carry a locally administered address — nearly half.** *The feature that keeps the product's central indicator from drifting upward forever carries nearly half the inventory.* Size the retention policy, the sweep and the operator-facing views for a population of that weight, not for a footnote. **And the asymmetry stays load-bearing: only locally administered addresses go dormant.** A server powered off for six months has a universal address — **it is ABSENT, not dormant, and its absence is information the operator wants.** The absence of a randomized address is the protocol behaving normally.
  - **Why this is a correctness requirement, not housekeeping:** under composite identity every rotation legitimately creates a distinct interface. Left active, dead ephemeral interfaces count as unreconciled assets **forever** — **the divergence count, the number the operator reads every morning, drifts monotonically upward and never comes back down. Within a year the product's central indicator is noise.**
  - **Only locally-administered addresses.** A server powered off for six months has a globally-unique address: it is **absent**, not dormant. **The absence of a universally-administered address is information the operator wants; the absence of a randomized one is the protocol behaving normally.** That asymmetry rests on a structural fact, not a heuristic.
  - **An interface grouped by an explicit human affirmation never goes dormant automatically** — the system does not revoke a human affirmation on a timer.
  - **Configuration constraint: the dormancy window must be shorter than the observation retention window**, or the system would mark interfaces dormant with no surviving observation to justify the decision — **an unauditable decision, and auditability is this product's foundation.** Violation is a **startup failure naming both settings**, not a warning.
- FR39: The operator can search by IP, MAC, hostname, or device and view its full record (declared attributes, observation history, connection point, hosted applications).

### Data Lifecycle & Editing
- FR40: The operator can edit the declared attributes of a record (e.g. name, tags, owner, role, notes).
- FR41: The operator can decommission, archive, or delete a declared device, subnet, or application, and reconciliation reflects the change.
- FR42: The operator can back up and restore the full dataset (export/import) for disaster recovery and migration.

### Integration & API
- FR43: An integration consumer can read core entities (devices, interfaces, subnets, IPs, applications, alerts, observations) via a read-only JSON API.
- FR44: A monitoring system can scrape opencmdb-specific metrics from an authenticated Prometheus `/metrics` endpoint.

### Administration, Security & Operations
- FR45: A first-run setup wizard guides the operator from empty state through connecting a source (or declaring a subnet), the first scan, and initial triage.
- FR46: The operator can authenticate with a local login and session.
- FR47: The system stores source credentials and passwords encrypted/hashed at rest.
- FR48: The operator can rotate a source's API key and can back up/restore secrets in encrypted form. **Secrets are protected by an envelope: the out-of-volume master key encrypts a single data key, which encrypts the credential fields.** Consequences that make the requirement achievable rather than aspirational: **rotating the master key rewrites one record and never touches the credential fields** (without the envelope, rotation is a crash-resistant multi-row migration across two backends — *the cost of testing it would exceed the feature*); **the secret backup is a short blob** the operator can keep in a password manager, not a database dump; **restoring on a new machine** unwraps under a passphrase and re-wraps under the new master key, and the data never moves. **The backup carries an identifier of the key that protects it** — without it, restoring a pre-rotation backup is indistinguishable from restoring a corrupt archive. **Master-key rotation ships at MVP; data-key rotation does not** (see Deferred).
- FR49: The system logs security-sensitive events (authentication, secret access/rotation).
- FR50: The operator can use the interface in English or French.
- FR51: The operator can configure the external base URL used to build deep links, with a tolerant fallback and a warning when unset.
- ~~FR52: opt-in anonymous telemetry.~~ **REMOVED FROM MVP** (number retained, not reused). Rationale and the conditions under which it could ever return are in *Deferred to Growth / Vision* below.

### Topology
- FR53: The operator can view network topology as a structured list/table — connections auto-populated for UniFi (client↔switch-port, client↔AP/SSID) and manually entered otherwise.

### Architecture Constraint (not a user-facing FR)
- The data model and auth are **multi-user-ready** from day one, even though a single admin is provisioned at MVP (a read-only/multi-user role is a Growth feature).

### Canonical Vocabulary (binding — one term, one translation)

_Vocabulary is architecture: **if a gesture is named after an operation we forbid, someone eventually
implements the operation.** These pairs are fixed. Neither language carries two meanings for one word,
and no "elegant" synonyms are permitted. The pair need not share a root — only a meaning._

| Concept | EN (docs, API, code) | FR (UI) | Meaning |
|---|---|---|---|
| The auto-discovered state | **observed** | **observé** | Factual, source-tagged, timestamped, **never editable, never modified** |
| The operator-documented state | **declared** | **déclaré** | Chosen intent — *"the state the operator has **documented**"* |
| The difference between them | **gap** | **écart** | The core object; **the product** |
| Link observed to declared | **reconcile** | **réconcilier** | A process, **never a button** |
| **Close the gap** — write observed values into the declared record, field by field | **document** (`document-field` / `document-all`) | **« Merger »** | The gap **closes**. The observed record is untouched; the link holds |
| **Keep the gap open** — record that a divergence is real and known, without deciding | **accept the gap** (`accept-gap`) | **« Accepter l'écart »** | *"Accepter une divergence qui devra être résolue."* The gap **stays open and keeps counting**; wakes on observed change, never on a clock |
| Defer until a chosen time | **snooze** | **mettre en veille** | A **timer** — distinct from `accept-gap`, which is a **sensor** |
| **Put an object outside the frame of the question** | **exclude** | **exclure** | **Replaces `ignore`.** The object is out of scope: factual, reversible, no verdict. **Deliberately the same term as an out-of-capability field** (FR7) — one concept, one word |
| Resolving inbox items | **triage** | **triage** | document / accept-gap / create / attach / **exclude** / snooze |
| A discovery origin | **source** | **source** | UniFi controller, generic scan, manual |

**Retired, and not to be reintroduced:** `accept-as-declared` (it named only the new-discovery case and
hid the dominant re-discovery case) · `merge` **in English** (it names the forbidden operation — the
founding pillar is *linked, never merged*; the French UI verb « Merger » is the fixed translation of
`document` and carries no such claim) · `revert` (**there is nothing to revert: nothing is ever
destroyed. Undoing an adoption, unlinking, and purging a decision are three distinct ADDITIVE
operations. The word imports a destructive-reversible intuition that is false here, and it would
eventually produce code that deletes.**) · **`ignore` / « ignorer »** — **not for its tone.** *"The tone
disdains"* is true and soft. **The structural reason: `ignore` names the OPERATOR'S ATTITUDE, while every
other verb in this grammar names the OBJECT'S STATE or the relation between two records** — `document`,
`attach`, `snooze`, `accept-gap` all say what happens to the thing. **It is the only one that names a human
feeling: it is out of grammar.** And it contradicts the product's own invariant — **a tool that may not
have an opinion about the backlog cannot carry a verb meaning *"I don't care"* in its action bar.**
**`exclude` unifies rather than merely replaces:** an excluded object and an out-of-capability field (FR7)
are the same idea — ***outside the frame of the question*. One concept, one term, two coherent uses.**

**Grammar that disambiguates in one line: you document a VALUE; you accept a GAP.** The object settles
which gesture is meant.

### Deferred to Growth / Vision (non-binding for MVP)

**The narrator — and the ban it must not be confused with.** Recorded at Growth rather than left unsaid,
because **the ban and the permission look alike from a distance and only one of them is a reproach** —
whoever implements the bans without this beside them will implement silence.

The backlog rules are deliberately negative: no nag, no badge, no growing counter, no health gauge, no age
as reproach, no gamification, no degradation. **Their test — *"if the operator does nothing for six months,
does the product become more unpleasant?"* — is a LOWER BOUND.** It guarantees the product does not get
worse. **It guarantees nothing upward: an empty text file passes it perfectly, and so does a product nobody
opens.** The missing symmetric:

> **If the operator does nothing for six months and comes back — does the product TEACH him something he
> did not know?**

The existing rule *"a notification may only be triggered by a **new fact**; 'it's been a while' is not a
fact"* is only its **left half. Turn it over: a new fact IS a reason to speak.** *"This device has not been
seen for four months"* is a fact. **It is not "you have not done anything in a while" — the first speaks
about the operator, the second speaks about the NETWORK.** The product has no right to comment on the
operator. **It has every right — and a duty — to report on the network.**

**What the backlog lacks is not a screen. It is a NARRATOR.** The journal already holds the regimes and the
history; nobody has asked it to tell them. **Product bet, stated as a bet:** *a backlog of divergences is
something you avoid; a story of what happened on your network while you weren't looking is something you
open — and the divergence is inside it, without anyone having had to brandish it.*
**The line, so the implementation cannot blur it: the narrator reports FACTS about the network, never
elapsed time about the operator. Any sentence whose subject is the operator is a nag and is banned.**

**Other Growth / Vision items:**
- Email + ntfy/Telegram channels; CSV / NetBox / phpIPAM import; active IPv6 scanning;
  read-only user role; versioned public API + write access + OpenAPI; additional connectors
  (Omada, Mikrotik, OPNsense/pfSense, Proxmox, Docker); full change-audit log; bulk export;
  port/service scanning + service-fingerprint identity signal; configurable inter-source
  precedence/trust ordering; **learned auto-acceptance of recurring expected discoveries**
  (pattern-learning that keeps the triage queue to the genuinely-new); interactive graphical
  topology map; SNMP/LLDP; consuming Prometheus for reverse enrichment; **data-key rotation**
  (source-credential key rotation ships at MVP; rotating the underlying data key re-encrypts every
  field and its failure paths are not yet all testable — shipping an operation whose failure paths
  cannot all be tested is shipping a time bomb with a button on it).

**Removed from the MVP — FR52, opt-in anonymous telemetry.** Not deferred out of timidity; removed for
three independent reasons, and it may only return under the conditions below.

1. **No user job.** *Which user wakes up with the problem FR52 solves? None.* Stated honestly the job is
   *"as the AUTHOR, I want to know whether anyone kept it."* That is real and legitimate — **it is not
   the product's job, and the author is not the user.** FR52 asks the operator to accept that the binary
   holding **the complete map of his network and his infrastructure credentials** opens an outbound
   socket to a server the author controls, **to do the author a favour**. In return he gets *"the project
   may survive better"* — an expectation, not a consideration.
   **And the cost is not the packet: the cost of an outbound channel, even disabled, even opt-in, is that
   the user starts wondering what ELSE we send. We would not be adding a measurement — we would be
   injecting doubt into the only asset this product has. Doubt, unlike telemetry, is not opt-in.**
   The prompt also lands at the worst possible moment: first run, inside the <15-minute promise, at the
   user's point of maximum vigilance — teaching him that the code can talk outward, that someone is
   interested, and that he now has to verify. **The trust cost multiplies by every image pull; the
   informational value saturates after a few hundred instances.**
2. **The instrument cannot produce a true sentence at this scale.** A few dozen installs at 3 months, at
   an opt-in rate that in a self-hosting audience is charitably 5% — *the act of self-hosting is itself
   a refusal of telemetry* — yields **n ≈ 2**. The resulting confidence interval spans "catastrophic" to
   "excellent". Worse, **the opt-in decision and the retention event are driven by the same latent
   variable — engagement — so the sample is biased toward the retained user by roughly a factor of three,
   and RAISING the opt-in rate does not fix it: more data tightens the interval around the wrong number.
   It makes the author confident. That is worse than ignorance.**
   Which inverts the PRD's own classification: **Docker pulls are a bad instrument everyone knows is bad,
   and an instrument whose flaw is public gets socially corrected. Opt-in telemetry is a bad instrument
   that looks like a good one — it has identifiers, timestamps, a denominator, a percentage. It looks like
   science. At n≈2 it is vanity in a lab coat — and where pulls mislead the reader, telemetry misleads the
   author, who is the only person whose decisions depend on the number.**
3. **The receiver is an operational commitment disguised as a checkbox — and a liability.** It is the only
   requirement in this PRD that **does not run on the user's machine**: it describes what the author does,
   forever. The endpoint is 40 lines; the cost is everything around them that is not code — a domain, a
   certificate, a host, a retention policy, a privacy notice, a contact address that answers, a backup
   (*lose the data and the retention signal is a retroactive lie*) — paid in **attention**, the project's
   scarcest resource, as a permanent on-call for a monthly glance.
   **And it is not a telemetry database: it is a list of installations of a network-mapping tool — a target
   list.** An attacker who compromises it obtains a directory of people known to hold a structured inventory
   of their own network. **A receiver a solo maintainer cannot defend is a receiver he should not own; a
   receiver that does not exist is the only one certain to be defensible.**
   It also breaks the doctrine behind NFR27 even while satisfying its letter (it is the author's
   infrastructure, not the user's): **an always-on, legally exposed, security-sensitive component that
   nobody tests and whose lying goes undetected. If the receiver silently drops 30% of pings, the author
   does not see an outage — he sees a declining retention curve, and draws product conclusions. The outage
   disguises itself as signal.** And *"anonymous"* is a claim that must survive contact with an IP address
   in an access log: a stable install identifier does exactly what a cookie does, and an erasure request
   against an anonymous identifier cannot be honoured — which is either a controller obligation or a false
   claim not yet discovered.

**If it ever returns** — never before trust is earned and demonstrated, and never below roughly a thousand
plausible installs (below that the instrument cannot produce a true sentence):
**never at first run** · **never as dormant outbound code** — absent from the binary by default, behind a
compile-time flag, because *the only credible opt-in for this audience is that the code does not exist until
it is asked for*, and no string in an inactive binary may reveal an endpoint · **never a persistent
identifier**, which turns a counter into a dossier · **never anything derived from the data** — *the number
of devices on a network IS information about that network, and aggregating it does not make it less true* ·
**never re-ask** — a refusal is final · **never share the channel** with update checks, crash reports or
news — *one channel, one use, or no channel* · **never a database, a queue, a spool or a retry** (a spool is
a local telemetry database, and a spool that flushes at once is indistinguishable from beaconing) · **never
opaque** — the exact payload in cleartext in the documentation and in the application's own log, readable
without packet capture. **The only moment it may ask is when the user comes to us**: available in settings
or via a documented flag, never surfaced, never announced. *The product does not ask; the product makes
available.* Which yields an even rarer, even more biased signal — **the admission that it was never a
measuring instrument. It is a gift. Call it a gift, and stop budgeting it as a metric.**

## Non-Functional Requirements

_Only categories that matter for this product. **Reference environment** for performance/scale
NFRs: an x86 Synology Plus-class NAS (≈4-core / 8 GB) with a synthetic dataset of **300 hosts /
36 subnets** unless stated (**a TARGET profile, not any measured install — see NFR30**). Docker
deployment targets x86 Synology (Container Manager is x86-only); ARM is supported best-effort via
the native binary. Thresholds marked "calibrate" are provisional pending real measurement._

**One load-bearing assumption has now been measured, and it holds.** The whole product — the screen
hierarchy, the journeys, the emphasis of these requirements — assumes **the gap is the CONTENT and
abstention the EXCEPTION.** That assumption was never measured, and **no specification and no test can
derive it: what fraction of a real network falls into each case is a property of the terrain, not of the
logic.** It was measured on the reference device (2026-07-16, read-only, aggregates only):

> **Over the stable, universally-administered addresses: multi-interface is a modest share, single-interface a large majority.** Bounds, not a number — the
> number is the output of the grouping engine, and structural facts are all a probe may legitimately read.
> **The threshold was 80%. The assumption HOLDS: the gap is the content; abstention is the exception; the
> grouped view stays a bootstrap mode and does not become the product.**

**Three reservations, recorded because a favourable answer is when one stops looking:**
1. **That majority is an OVER-estimate by an unknown margin.** Hostname — one of grouping's three signals — is
   unusable on nearly half of clients (FR9). **You cannot demonstrate a grouping you have no signal for**, so the
   abstention floor is itself a floor.
2. **a large share of the inventory is locally administered** (FR38b) — the ephemeral-interface lifecycle is half the
   data, not a footnote.
3. **It was measured at a few hundred clients / 1 VLAN, not at 300 / 36** (NFR30). **Infrastructure grows with
   segmentation and infrastructure is the multi-interface population** — the infrastructure devices each
   carry several radio addresses. *The direction of the bias is knowable even where its size is not.*

### Performance
- NFR1: A full discovery cycle + reconciliation diff over the reference dataset completes at
  **p95 ≤ 120 s** while the UI stays responsive (< 2 s) during the scan (measured via job
  timestamps). **The tension with NFR16 is reframed: it is not "write throughput vs scan speed".**
  The cycle is dominated by network I/O across the subnets, not by the database — a few hundred upserts
  in one batched transaction is milliseconds. A serialized writer only hurts if writes are per-host.
  **The requirement is therefore a separation of the read and write paths**: the write path is owned
  exclusively, everything else (UI, diff computation) reads concurrently and never contends for it; the
  diff computes against a read snapshot while the previous batch commits; bulk writes are batched and
  yield, so interactive writes are never starved behind a scan. **Instrument where the 120 s actually
  goes before hardening anything — the expectation is that the overwhelming majority is network.**
- NFR2: Primary UI views (inventory, triage inbox, per-subnet occupancy, **the device record incl. "Hosted
  here"**, deep-linked object) render at **p95 ≤ 1.5 s** on the reference NAS **while a discovery cycle is
  writing** (measured via Playwright timing). _("Primary" is a LATENCY class — this screen is on the human
  path — not a claim about hierarchy; the UX spec ranks the same screen as "supporting" and the two do not
  conflict. D57-scope replaced the impact/blast-radius view here: it is Growth. **When it lands, note that
  NFR2 cannot police it — the measured failure mode of a recursive CTE is FAST and wrong: 36.5 ms, 40× inside
  budget, 199 331 rows instead of 200.**)_ _(D64: "under WAL load" named a SQLite mechanism. The
  requirement was never about WAL — it is about reads not contending with the write path (NFR1) — so it
  is restated in terms of the load that actually exists.)_
- NFR3: Time-to-first-value (instrumented hypothesis): populated UniFi inventory + first
  findings in < 15 min; install < 30 min — validated with ~5 testers on a guided protocol
  (median), not asserted.

### Reliability & Reconciliation Correctness
- NFR4: **The release gate is a binary adversarial trap suite, not a statistical threshold.** At the
  reference scale (300 hosts) the only measurable threshold is zero: with n=300 and zero observed
  events the 95% upper bound on the true rate is already 1%, so a `≤ 0.01` gate cannot distinguish
  0.5% from 2% — **any fraction is theatre**. The gate is: **truth-table failures = 0**, at the
  **device** level, over ~50 adversarial scenarios (randomized MAC, multi-NIC, shared-hardware VM,
  **cloned/spoofed MAC**, DHCP churn, **VRRP/HSRP shared virtual MAC**, hostname collision, ephemeral
  container interfaces), **each in positive AND negative form**. The truth table has **three columns**,
  and all three gate:
  - **must-not-merge** — a merge is a failure (the false merge: two hosts fused; the operator loses
    trust and uninstalls).
  - **must-merge** — an **abstention** is a failure. This is the anti-cowardice column: *a gate that
    rewards abstention and punishes false-merge is trivially gamed — an engine that abstains on
    everything scores zero false merges.* The distinction is precise: abstaining **because there is
    not enough signal** is honesty (measured, not gated); abstaining **when the signal is there**
    (identical anchors, a single candidate) is cowardice (gated).
  - **must-abstain** — a decision is a failure (the honestly ambiguous case; see FR16).
  **Bulk statistical metrics (cluster contamination, pairwise precision/recall, abstention rate) are
  OBSERVABILITY, published per release with confidence intervals; they gate nothing.** Their purpose is
  discovery: **any anomaly surviving investigation becomes a new trap — the gate grows only by proof.**
  Stated limit: a trap suite proves nothing about what was not imagined; at v0.1 the gate is **weak and
  honest rather than strong and false**.
  The labeled fixture is an **architecture deliverable** (it defines the shape of the source interface
  and of the engine), **synthetic and seeded** — a generator constructs the devices *then* emits the
  observations, so truth is an **input to generation, not an interpretation of an output**. Real network
  captures are used only for a distributional representativeness diff, **never as a gate, and never
  committed to the public repository** (they carry real MACs, hostnames and home topology).
  **The oracle is the fixture author, explicit and versioned, with a mandatory one-sentence `reason` on
  every expectation: if the reason cannot be written in one sentence, the case is genuinely ambiguous
  and becomes a `must-abstain`.**
- NFR5: The never-overwrite invariant is enforced and covered by explicit anti-regression tests. **The
  grammatical subject of the invariant is the SCANNER**: ingesting an observation must never alter a
  declared field. An operator writing a declared value through an explicit action is a normal
  declarative write with a human author, and is not covered by this prohibition. Three assertions:
  - Ingesting an observation that contradicts a declared field leaves that field **unchanged** and opens
    a divergence.
  - Documenting a field (FR13) sets the declared value **and leaves the observation record
    bit-for-bit unchanged, with the link intact** — the invariant is not only "declared is not
    overwritten" but also "**the observed side is never touched by the act of documenting**". *That is
    where a merge would silently reintroduce itself.*
  - **No code path writes a declared field with a non-human author** — a structural constraint, not a
    convention.
  And the corollary that keeps drift detection alive: **the divergence computation never consults how a
  declared value was obtained.** A field documented from an observation yesterday is simply a declared
  field today, and can diverge again tomorrow. **A declared field that automatically follows its source
  is a merge in disguise: it makes divergence structurally impossible on that field. It is forbidden.**
- NFR6: Reconciliation cycles and inter-source precedence are idempotent and independent of
  ingestion order (verified by fuzzing arrival order).
- NFR7: **0 false "device-gone" events — enforced by the type system, validated by fault injection.**
  The requirement is unachievable as a test alone; it is a design constraint:
  - **An observation must be structurally INCAPABLE of expressing "gone".** It states only what was
    seen. **Absence is DERIVED by the engine, and only when liveness is `live`.** The property is then
    provable by exhaustion over a closed set of error conditions: *for every source-error condition,
    zero change events are emitted.* **Fault injection validates it; it does not discover it.**
  - **Presence requires explicit hysteresis.** A device that misses one probe has not left: the engine
    must distinguish **absence of proof from proof of absence**. Presence flips to "gone" only after N
    failures over a window T. **Under any fault, state degrades toward "stale/unknown", never toward
    "changed/gone".**
  - **Every observed datum carries freshness and the capability under which it was collected**, so that
    a shrinking sensor is never read as a changing world.
  Validated under fault injection (10% packet loss over N cycles; source down/degraded/timeout/
  auth-rejected) against the assertions of NFR8.
- NFR7b: **Silent schema drift.** The loud failure (an authentication rejection at 3 a.m.) is the easy
  one. **The dangerous one is a source that renames a field: a tolerant parser returns an empty
  collection, and the engine is live, confident and wrong.** Two defences are required:
  - **Collections feeding presence must never default to empty on a parse miss** — a missing required
    field is an error condition, not an empty result. (Statically checkable.)
  - **A population collapse is a SOURCE event, not a device event.** An implausible response — the vast
    majority of a scope's population vanishing in one cycle — is classified as a source fault and
    suppresses divergences, rather than being reported as mass departure. The plausibility threshold is
    a product decision, to be calibrated; **it is the only defence against a drift that fails silently.**
- NFR8: **"Degrades gracefully (no crash)" is not a requirement — the most dangerous product passes it.**
  An engine that marks 84 devices "disappeared" at 3 a.m. and fires 84 alerts has not crashed: it ran
  perfectly and lied with an impeccable uptime. **A requirement the worst case satisfies is not a
  requirement.** The claim is replaced by four falsifiable assertions, verified under fault injection:
  - **(a) Monotone honesty:** for any injected fault, `device_facts(faulted_run) ⊆ device_facts(clean_run)`
    on the same fixture. **A fault may only REMOVE knowledge, never ADD an assertion.** The clean run is
    the oracle — differential, no magic number. Source-facts may grow.
  - **(b) Bounded blast radius:** a fault scoped to one (source, scope) modifies no other scope's state.
  - **(c) Convergence after recovery:** once the fault lifts, state converges to the clean-run state
    within N cycles — no stuck `blind`, no hysteresis one-way door.
  - **(d) Exactly one actionable notification**, naming the cause and the scope.
  **Fault injection has two layers, and only the second tests version drift:** replaying error *outcomes*
  tests the engine; **replaying raw recorded bytes through the real parser tests that the parser PRODUCES
  the right error when a field is renamed, retyped, nulled or removed.** Injecting a drift verdict
  directly asserts only that our own match arm compiles. Fixtures are **version-tagged, dated, and
  re-captured by a job that diffs the schema** — a contract test against a permanently frozen fixture is
  a snapshot test in disguise.
  **The supported version matrix is bounded as a product decision, and it is bounded on the axis the
  connector actually speaks to.** The controller exposes two independent release trains: the **UniFi OS
  console** (measured a 5.x build on the reference device) and the **Network application** (measured a 10.x build,
  previously a prior 10.x build). **The connector speaks to the Network application. Tagging a fixture with a console
  version makes it lie about its own provenance — and the re-capture job diffs against that tag.**
  - **Tested matrix: Network application 10.4.x**, the version measured on the reference device
    (2026-07-16). Version-tagged, dated, re-captured. **The matrix grows by EVIDENCE, never by intention:
    a version enters it when a capture proves it, not when we hope it.**
  - **Outside the matrix the guarantee changes in KIND and does not vanish. The product does not claim to
    work on a version it has never seen — it claims not to LIE about one.** An unparseable or implausible
    response blinds the source, **suppresses its divergences (NFR19), and raises exactly one notification
    naming the version and aimed at the maintainer.** *A version we have never seen cannot be supported.
    It can be refused honestly.*
  - **The upgrade path is the re-capture job, and it is what makes "future versions" a testable idea rather
    than an unenforceable promise:** when the vendor ships a new version, the job diffs the live schema
    against the fixture. **Green → the matrix extends by proof. Red → a fixture and a decision — and we
    know before the engine lies, not after.**
  > **"Supports future versions" is not a requirement, it is a wish: there is no fixture for a version that
  > does not exist, and the claim would be discovered false at a user's site rather than ours. What is
  > enforceable is the pair — a bounded tested matrix, and a loud refusal outside it.**

### Security
- NFR9: **Threat model, split into three claims — because a claim that cannot be tested is not a
  requirement, it is a slogan.** The previous wording ("the key is kept separate from the database
  volume") is **not enforceable**: the primary deployment is Docker on a Synology NAS, backup tools
  select **per shared folder**, and the #1 exfiltration vector on a consumer NAS is the backup itself.
  The key would be "separate" in the container's mount namespace and **in the same archive** in the
  artefact that actually leaves the house. **A control that satisfies its own wording and nothing else
  is theatre — and a broad fictional guarantee is worse than a narrow real one, because it destroys the
  vigilance the user already had. That negative delta multiplies by every image pull; the truth of the
  claim does not.**
  - **NFR9a — GUARANTEED, testable.** Source credentials are never present in plaintext in the database
    file, a dump, the write-ahead log/journal, the application logs, or API responses. **An attacker
    holding only the database cannot read them.** Verified by a byte-level scan of the artefacts.
  - **NFR9b — GUARANTEED, testable.** The application never writes the master key into the data volume.
    Key path and data path are distinct parameters, and **startup FAILS if the key path resolves inside
    the data directory** (after symlink resolution) or if the key file is group/other-readable. *The
    move that matters: from "the key IS separate" — a state of the world we do not control and cannot
    test — to "we REFUSE to start where it is not" — a behaviour of our binary, fully testable. We
    cannot measure the world; we can measure our reaction to it.*
  - **NFR9c — NOT GUARANTEED, documented as a non-guarantee.** If the operator's backup tool copies
    both the key and the database, encryption at rest no longer protects them. **opencmdb cannot and
    will not prevent this.** Stated in the quickstart, before installation — not in a FAQ. The reference
    deployment places secrets in a **separate top-level shared folder** precisely because backup tools
    select at that granularity: it makes the exclusion **visible and atomic instead of drowned**. Same
    class of decision as NFR13 (TLS): a cooperative, documented guarantee.
  - **Out of scope, unchanged:** a local root attacker.
- NFR10: Credentials and passwords are never stored in plaintext (verified: the stored blob is
  not plaintext).
- NFR11: All HTTP surfaces (UI, JSON API, `/metrics`) require authentication.
- NFR12: The secret round-trip (rotate → backup → restore → decrypt) is verified end-to-end, **and the
  majority of its assertions are failure paths — a security suite containing only happy paths does not
  measure security, it measures that the demo works.** Note what this is NOT: asserting that
  `encrypt` then `decrypt` returns the input tests someone else's library — **that is not our test.**
  - **Non-regenerable oracle:** the expected plaintext is a **literal constant in the test**, never a
    value produced by the code under test; **backup fixtures are frozen, committed artefacts produced by
    an earlier version.** Otherwise we do not test restore compatibility — we test that the code agrees
    with itself.
  - **Full cycle:** seed a credential → rotate the master key → back up → **destroy the instance
    entirely** (new data directory, new process) → restore → the credential decrypts to the literal.
  - **The backup is opaque:** a byte-level scan of the whole archive finds neither the credential nor
    any key material.
  - **Wrong key on restore:** a distinct typed failure — not a panic, not a corrupt value, not a success
    with garbage — **and the target instance is left intact.** (*The real risk is the operator restoring
    with the wrong key and losing what he still had.*)
  - **Pre-rotation backup:** a defined, documented outcome, and **"wrong key" is distinguishable from
    "corrupt archive"** (see FR48's key identifier).
  - **Interrupted rotation — the assertion that justifies the envelope architecture:** the process is
    killed at each enumerated cut point (an abrupt kill, not a graceful unwind — *a graceful unwind does
    not model a NAS losing power, and losing power is the real scenario*). After restart, **every
    credential is decryptable, by either the old key or the new one — never a state where part of the
    data is unreadable.** *This assertion is not negotiable downward when the schedule tightens: it is
    the one that bought the architecture. Cut it and we paid the complexity without buying the guarantee.*
  - **An orphan data key** (wrapped by a key that no longer exists) is detected **at startup**, not on
    the first credential read hours later during a reconciliation. **Fail fast, fail loud** — a silent
    failure here turns a configuration problem into data loss found too late.
  - **Truncated or altered backup:** rejected on integrity **before anything is written to the target** —
    an assertion about the order of our operations, not about the cipher.
  - **Verified on both backends**, on every PR. If a cross-backend restore is unsupported, **the refusal
    is explicit and tested**, not a failure on the third credential.
- NFR13: TLS in transit is a documented deployment responsibility (reverse proxy), not provided
  by the app.

### Data Integrity & Durability
- NFR14: Full dataset backup/restore round-trips with no loss or corruption — verified by equal
  SHA-256 and row counts before/after on a CI dataset.
- NFR15: **The invariant suite runs on MariaDB 10.11.11 — the exact DSM 7 package — on every PR, never
  nightly** (an engine tested nightly is an engine broken for a day). **The engine never decides a
  comparison, and this survives D64 as a CORRECTNESS requirement, not a portability one:** value
  comparison and normalization happen in application code, never in SQL; time is computed in application
  code and bound as a parameter, never taken from the database; identifiers are generated by the
  application, not by the engine. **Enforced by explicit binary collation on every text column plus a CI
  grep over the DDL** — one dialect, one file, one red, one repair. *Without that gate, `utf8mb4_general_ci`
  settles hostname equality on our behalf: an IDENTITY bug, not a cosmetic test failure. The second engine
  used to be what made this red; it is gone, and the gate replaces it (D64 condition 1).*
- NFR16: ~~SQLite WAL mode, `busy_timeout`, 50 concurrent writers~~ — **STRUCK by D64 (number retained).**
  The requirement described a failure mode of an engine the product no longer supports. The serialized
  writer survives in NFR1 and in the Self-Hosted Operational Constraints, where it belongs: it is a
  property of our write path, not of SQLite's locking.

### Upgrade, Migration & Footprint
- NFR17: Schema migrations are versioned, idempotent, and **resumable after interruption** (e.g.
  a NAS power loss mid-migration); a backup is taken automatically before migration; migrations
  are verified on a populated MariaDB with a zero-loss invariant, and a documented rollback path
  (via backup). **Resumability comes from the BACKUP, not from the transaction** — DDL is not
  transactional on MariaDB.
- NFR18: Resident memory is **≤ ~512 MB at rest** on the reference NAS (calibrate); cold start
  is < 5 s; binary and container-image size are bounded and tracked in CI.
- NFR19: An update incurs a **bounded downtime (target < 30 s)** — not zero-downtime — and the
  app resumes cleanly with no data loss.

### Compatibility & Portability
- NFR20: Runs as a single binary and as a Docker container (Synology Container Manager priority,
  x86); ARM via native binary is best-effort. **The binary is one process — no Redis, no workers, no
  queue, no required reverse proxy — and it requires a MariaDB alongside it.** The reference deployment
  is a two-service compose, and **no document, tagline or image description may describe the try-it path
  as a single `docker run`** (D64 condition 2): it would be a gap between declared and observed, shipped
  by the product whose thesis is that such gaps matter.
- NFR21: **MariaDB 10.11+ is the only supported engine.** SQLite and MySQL are not supported —
  MySQL is a different product from MariaDB (collation defaults, `RETURNING`, recursive-CTE history) and
  **we do not claim what has no CI.** PostgreSQL is not supported at MVP. **If PostgreSQL is ever added,
  it is not free and the price is already named:** with the second engine gone, nothing forces the
  repository trait to stay dialect-neutral, so **the trait is audited BEFORE any such port, not during**
  (D64 renunciation 3, D51 point 4 as amended).
- NFR22: UI supports current evergreen browsers (Chrome, Firefox, Safari, Edge).
- NFR23: The UniFi connector supports a stated minimum UniFi OS version matrix (defined in
  architecture) and is tested against it.

### Usability & Accessibility
- NFR24: The UI is responsive (breakpoints 360 / 768 / 1280 px; no horizontal overflow; touch
  targets ≥ 44 px), verified by visual snapshot; deep-linked object views are usable on a phone.
- NFR25: Accessibility — **WCAG 2.1 AA on the key views** (inbox, occupancy grid, deep-linked
  object, login/setup, dashboard — and, by rule, any view in a critical path). axe-core (0
  violations, per theme) is a **blocking floor, not a ceiling**; because it misses focus/announcement
  behaviour, two **manual gates are also blocking**: a scripted keyboard checklist on any PR touching
  the inbox/grid (focus visible and **never lost after an optimistic swap**), and a screen-reader
  pass (NVDA+Firefox / VoiceOver+Safari) per release with recorded proof. Keyboard operability and
  adequate contrast throughout.
- NFR26: The UI is available in English and French; all user-facing strings are externalized.

### Operability & Maintainability
- NFR27: 12-factor configuration (file + environment variables); no external services (cron,
  Redis, workers) required.
- NFR28: Installation on a clean environment following the documentation is achievable in
  **≤ 30 min wall-clock** (measured on a clean Ubuntu 22.04 VM with the provided script;
  Synology target validated separately).
- NFR29: The self-diagnostic dashboard and authenticated `/metrics` provide operational
  visibility **to the operator, on his own instance**. No operational data leaves the deployment.
  Our own retention signal comes from public artefacts and pull statistics (see *Business Success*).

### Scalability (bounded)
- NFR30: Designed for a single operator and a small-but-nontrivial network — a reference target
  of **300 hosts across 36 subnets/VLANs**; not enterprise-scale or many concurrent users at MVP.
  **300 hosts / 36 subnets is the TARGET AUDIENCE's profile. It is not the author's install, and it has
  never been measured on a real network of that shape** — it describes who the product is for, and it is
  the size the seeded generator produces. **The author's own network, measured 2026-07-16: a few hundred known
  clients, single-VLAN, one site.** The phrase travelled through four documents
  unqualified; it is qualified here.
  **The consequence, and it is not cosmetic:**
  > **The dimension this target is DEFINED by — segmentation — is the one dimension the author's network
  > cannot exercise.** With a single VLAN, the layer-2 domain that scopes interface identity has exactly
  > one value, so the scoped key degenerates to the address alone. **The foundation ships with zero contact
  > with a segmented reality until a segmented user appears; it is adopted on evidence of shape, not of
  > behaviour.** *Developing against it is testing a network we do not target.*
  **And it lands on the generator:** the seeded dataset must produce an interface-per-device distribution
  at 300/36 with **exactly one empirical anchor — a single-VLAN home lab.** The bias direction is
  knowable even where its size is not: **infrastructure grows with segmentation, and infrastructure is the
  multi-interface population** (measured: 10 infrastructure devices carrying 26 radio entries — 3.6
  addresses per box). **The generator must not encode a measurement taken at a single-VLAN home lab as if it held at 300/36.**
