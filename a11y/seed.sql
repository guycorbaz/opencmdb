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
-- ⚠️ Both named causes appear, deliberately: they carry DIFFERENT sentences (a better source
-- against a doubt to lift), so a seed with only one would let a gate pass over a section where
-- the two had been fused. Every row is an ABSTENTION — a `match` would need an `interface`
-- row, and inventing an identity for a harness is more fixture than this needs.
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
