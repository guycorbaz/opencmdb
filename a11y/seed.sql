-- The accessibility harness's OWN seed — story 6b.11's code review, 2026-08-23.
--
-- 🔴 **IT EXISTS BECAUSE THE SHIPPED DEMO SEED IS ONE ROW SHORT, AND THAT WAS FOUND BY
-- REPRODUCING CI FROM A TRUNCATED STORE RATHER THAN FROM A SESSION'S OWN.** `kbd-probe.mjs`
-- needs at least TWO queue rows — a single row cannot witness an arrow STEP, cannot show the
-- highlight and `aria-current` agreeing on which of several rows is current, and gives the
-- two end-of-list controls nothing to be at either end of. `docker/seed-example.sql` inserts
-- ONE entity with ONE disagreeing field, which renders exactly ONE queue row: measured on a
-- virgin database, `0` rows before it and `1` after.
--
-- ⚠️ **The CI run that passed was green on RESIDUE, not on the seed.** The `Tests` step runs
-- against the same database immediately before, and what it leaves behind happened to carry
-- the queue over the floor (`queue: 3 row(s)`). Nothing guarantees that, and the accessibility
-- step run on its own would have exited **2**. So this file TRUNCATES first: the gate must
-- measure a state the repository can reproduce, not one the pipeline happened to accumulate.
--
-- 🔑 **NOT a widening of `docker/seed-example.sql`.** That file is the operator's demonstration
-- data, shipped in the image and run deliberately by a human; bending it to a harness's floor
-- would make the product's demo answer to CI's needs. Two files, two audiences.
--
-- ⚠️ **This file is inside the `authorship` gate's perimeter, by decision.** `AUTHORSHIP_ROOTS`
-- gained `a11y` in the same commit and `SANCTIONED_SITES` names this path, because a seed
-- writing `origin='manual'` / `actor_id='operator'` is a machine writing as a human — exactly
-- NFR5's subject. Story 5.12's own doc says to reopen the perimeter the day a new file carries
-- SQL rather than to let it sit outside; opening the roots WITHOUT naming the site would simply
-- turn the gate red, so the two are one act.
--
-- Addresses are RFC 5737 documentation range; no real network is referenced.

-- Idempotent: this file may be re-run, and it starts from a known state on purpose.
-- ⚠️ The plan's three tables are truncated in DEPENDENCY ORDER, children first: `ip_range` and
-- `ip_address` both carry a foreign key to `ip_subnet`, so deleting the parent first answers
-- `ERROR 1451` and the whole seed stops. Story 14.2 added them.
DELETE FROM ip_address;
DELETE FROM ip_range;
DELETE FROM ip_subnet;
DELETE FROM link_candidate;
DELETE FROM identity_link;
DELETE FROM observation_record;
DELETE FROM declared_attribute;

-- ── Four documented entities, three of which the network contradicts ──────────────────────
INSERT INTO declared_attribute (entity_id, attr_key, attr_value, origin, actor_id, updated_at) VALUES
  ('00000000-0000-0000-0000-0000000000b1', 'ipv4',     '192.0.2.11', 'manual', 'operator', NOW(6)),
  ('00000000-0000-0000-0000-0000000000b1', 'hostname', 'nas-01',     'manual', 'operator', NOW(6)),
  ('00000000-0000-0000-0000-0000000000b2', 'ipv4',     '192.0.2.12', 'manual', 'operator', NOW(6)),
  ('00000000-0000-0000-0000-0000000000b2', 'hostname', 'switch-core','manual', 'operator', NOW(6)),
  ('00000000-0000-0000-0000-0000000000b3', 'ipv4',     '192.0.2.13', 'manual', 'operator', NOW(6)),
  ('00000000-0000-0000-0000-0000000000b3', 'hostname', 'printer-hp', 'manual', 'operator', NOW(6)),
  ('00000000-0000-0000-0000-0000000000b4', 'ipv4',     '192.0.2.14', 'manual', 'operator', NOW(6)),
  ('00000000-0000-0000-0000-0000000000b4', 'hostname', 'vm-billing', 'manual', 'operator', NOW(6));

-- ── The observed side: each answers at its address with a DIFFERENT hostname ───────────────
INSERT INTO observation_record (id, connector_id, observed_at, l2_domain, vantage, facts, raw) VALUES
  ('dddddddd-0000-0000-0000-0000000000b1',
   '00000000-0000-0000-0000-000000000000', NOW(6),
   '00000000-0000-0000-0000-000000000000', '00000000-0000-0000-0000-000000000000',
   '[{"IpV4":{"addr":"192.0.2.11"}},{"Hostname":{"name":"nas-01.lan","source":"Dns"}}]', NULL),
  ('dddddddd-0000-0000-0000-0000000000b2',
   '00000000-0000-0000-0000-000000000000', NOW(6),
   '00000000-0000-0000-0000-000000000000', '00000000-0000-0000-0000-000000000000',
   '[{"IpV4":{"addr":"192.0.2.12"}},{"Hostname":{"name":"sw-core-1","source":"Dns"}}]', NULL),
  ('dddddddd-0000-0000-0000-0000000000b3',
   '00000000-0000-0000-0000-000000000000', NOW(6),
   '00000000-0000-0000-0000-000000000000', '00000000-0000-0000-0000-000000000000',
   '[{"IpV4":{"addr":"192.0.2.13"}},{"Hostname":{"name":"hp-laserjet","source":"Dns"}}]', NULL),
  -- ── The UNDECLARED sighting: an address no declared entity claims ────────────────────────
  --
  -- 🔴 **STORY 6.4 ADDED THIS ROW, and without it neither browser gate can see the product's
  -- only live gesture.** Measured before it existed: this seed produced ZERO `Nouveau` rows,
  -- because every observation above answers at an address one of the four entities declares.
  -- The documenting gesture belongs to `undeclared` — *observed, and no declared record claims
  -- it* — so a store with none of that population offers the gesture nowhere, and a DOM-level
  -- guard walking it measures the absence of a control that had no reason to be there.
  --
  -- ⚠️ It carries a HOSTNAME as well as an address, deliberately: `POST /document-all` adopts
  -- the WHOLE record, so a single-fact sighting would let a gate pass over a gesture that
  -- documents one field — which is FR13(b), Epic 7's, and a different gesture.
  ('dddddddd-0000-0000-0000-0000000000b9',
   '00000000-0000-0000-0000-000000000000', NOW(6),
   '00000000-0000-0000-0000-000000000000', '00000000-0000-0000-0000-000000000000',
   '[{"IpV4":{"addr":"192.0.2.99"}},{"Hostname":{"name":"unknown-99","source":"Dns"}}]', NULL);

-- ── The identity engine's reach, so the section that reports it is not empty ────────────────
--
-- 🔴 **STORY 6.4 ADDED THIS TOO, and without it the reach section renders "Nothing observed
-- yet" over a queue of five rows dated a minute ago** — a claim and its refutation in one
-- viewport, which is story 6b.5's registered contradiction reappearing on this very screen.
-- The rows above are written straight into the store, so the identity pass never ran over them
-- and `identity_link` was empty; `has_any` is then false and the section says nothing was seen.
--
-- 🔑 **And it is what puts story 6.4's OWN copy on a page a browser gate walks.** Each cause
-- line says why it carries no documenting gesture — the gesture belongs to `undeclared`, which
-- is the queue's `Nouveau` row and not this section — and with no abstention seeded, those
-- sentences were rendered nowhere the gates could see them.
--
-- ⚠️ Both named causes appear so the screen shows the two DIFFERENT sentences (a better source
-- against a doubt to lift) to anyone who looks. ⚠️ **Not because a gate would catch their fusion**:
-- the probe check reads that each line HAS a sentence, never which one — its own doc says so — and
-- the fusion case is caught in Rust, by `every_identity_cause_line_says_why_it_carries_no_gesture`.
-- That justification stood here until story 6.4's code review refuted it against the check's own
-- stated limit. Every row is an ABSTENTION — a `match` would need an `interface` row, and inventing
-- an identity for a harness is more fixture than this needs.
INSERT INTO identity_link
  (id, observation_id, interface_id, current_subject, outcome, rule_id, abstention_cause,
   evidence, ruleset_version, decided_by, valid_from, valid_to) VALUES
  ('11111111-0000-0000-0000-0000000000b1', 'dddddddd-0000-0000-0000-0000000000b1',
   NULL, '00000000-0000-0000-0000-000000000000', 'abstained', NULL, 'absence_of_proof',
   '[]', 1, 'ENGINE', NOW(6), '9999-12-31 23:59:59.999999'),
  ('11111111-0000-0000-0000-0000000000b2', 'dddddddd-0000-0000-0000-0000000000b2',
   NULL, '00000000-0000-0000-0000-000000000000', 'abstained', NULL, 'absence_of_proof',
   '[]', 1, 'ENGINE', NOW(6), '9999-12-31 23:59:59.999999'),
  ('11111111-0000-0000-0000-0000000000b3', 'dddddddd-0000-0000-0000-0000000000b3',
   NULL, '00000000-0000-0000-0000-000000000000', 'abstained', NULL, 'ambiguous',
   '[]', 1, 'ENGINE', NOW(6), '9999-12-31 23:59:59.999999');

-- ── The addressing plan (story 14.2) ────────────────────────────────────────
--
-- 🔴 WITHOUT THESE ROWS THE AXE GATE WOULD WALK AN EMPTY `/ipam` AND REPORT SUCCESS. That is the
-- `AXE_REQUIRE_QUEUE` shape exactly: the screen's whole content — cells each carrying their own
-- accessible name, a selector, a legend — exists only when a subnet does, so a gate run against a
-- virgin store measures the empty-plan sentence and nothing else, and passes. `AXE_REQUIRE_PLAN=1`
-- turns that into *the gate could not run*; these rows are what let it run.
--
-- 🔴 **THE PREFIX LENGTHS ARE /25, AND THAT IS NOT COSMETIC.** `ip_subnet_cidr` is UNIQUE on
-- `(base, prefix_len)`, and `ipam_repo.rs`'s store-backed tests own the three RFC 5737 /24s
-- (`192.0.2.0/24`, `198.51.100.0/24`, `203.0.113.0/24`) — one CIDR per test, which is story 14.1's
-- own rule after a panicking test poisoned its successor and inflated every mutation count. Seeding
-- a /24 here took the name a test needs: measured, FOUR tests went red with
-- `the subnet: Constraint("unique")` the moment this file had run against a developer's store.
-- ⚠️ CI never saw it — its seed step runs AFTER the tests — so the failure would have been local
-- only, which is precisely where this project measures its mutations. *A fixture and a test that
-- share a namespace are one collision apart, and the store does not forget between them.*
--
-- ⚠️ TWO subnets, because one cannot show a selector narrowing: with a single tab, `aria-current`
-- is satisfied by the only thing there is. Story 6b.4's ordering finding is the same shape — a
-- fixture that seeded ONE entity made an ordering structurally unobservable.
--
-- 🔑 The ranges are chosen so the grid draws EVERY cell state at once: a `static` range, a
-- `dhcp-pool` range, one individually-defined address inside the static range, and a gap between
-- the two ranges that no range covers — which is the blank cell, the one state no legend entry
-- names, and half of the pair the gate compares in the browser. A gate that walks a grid showing
-- three of four states measures three of four.
--
-- Addresses are RFC 5737 documentation blocks, as everything in this file is.
INSERT INTO ip_subnet (id, base, prefix_len, label) VALUES
  ('22222222-0000-0000-0000-00000000a001', '192.000.002.000', 25, 'Office'),
  ('22222222-0000-0000-0000-00000000a002', '198.051.100.128', 25, 'Workshop');

INSERT INTO ip_range (id, subnet_id, first_addr, last_addr, policy, label) VALUES
  ('33333333-0000-0000-0000-00000000b001', '22222222-0000-0000-0000-00000000a001',
   '192.000.002.001', '192.000.002.040', 'static', 'Servers and gear'),
  ('33333333-0000-0000-0000-00000000b002', '22222222-0000-0000-0000-00000000a001',
   '192.000.002.080', '192.000.002.126', 'dhcp-pool', 'Laptops'),
  ('33333333-0000-0000-0000-00000000b003', '22222222-0000-0000-0000-00000000a002',
   '198.051.100.129', '198.051.100.200', 'reserved', 'Held for the new line');

INSERT INTO ip_address (id, subnet_id, addr, label) VALUES
  ('44444444-0000-0000-0000-00000000c001', '22222222-0000-0000-0000-00000000a001',
   '192.000.002.009', 'nas-01');
