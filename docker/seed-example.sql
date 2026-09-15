-- opencmdb — example seed (DEMO data, safe to delete).
--
-- Purpose: make a real GAP appear on the page immediately, before you have any real declared
-- data or a live scan. It inserts ONE declared entity plus ONE synthetic observation that
-- DISAGREES on the hostname — so the page shows a drift gap end to end.
--
-- Why synthetic? The ping-only scanner (v0.1.0) observes just an IPv4 (and RTT), so on real data
-- it can only ever produce a "clear" match on the IP plus "no observed value" reach for fields it
-- cannot see (like hostname). To actually SHOW a drift on the hostname, this seed writes an
-- observation as if a richer connector had seen a different host — clearly demo data.
--
-- Values use the RFC 5737 documentation range (192.0.2.0/24); no real network is referenced.
--
-- HOW TO RUN (after `docker compose up -d` has created the tables once):
--   mysql -u <USER> -p opencmdb < seed-example.sql
--   (or paste it into phpMyAdmin / Adminer on your DSM MariaDB)
-- Then refresh http://<nas>:8080/ — you should see a gap on `hostname`.
--
-- The tables are created automatically by opencmdb on startup (sqlx migrations); run this AFTER
-- the container has started at least once.

-- ⚠️ Stop HERE, before anything is deleted, on a store the current opencmdb has not started on:
-- the sighting table below arrives with the binary's migrations. Without this line the two DELETEs
-- below ran and the first INSERT then failed (`ERROR 1146`), leaving no demo at all.
SELECT 1 FROM address_sighting LIMIT 0;

-- Idempotent: clear any prior copy of this demo entity/observation first.
DELETE FROM declared_attribute WHERE entity_id = '00000000-0000-0000-0000-0000000000aa';
DELETE FROM observation_record WHERE id       = 'dddddddd-0000-0000-0000-0000000000aa';
-- ...and the address sighting that observation implies. Without it a re-run's INSERT below meets
-- the first run's row and stops on a duplicate key (`ERROR 1062`), after the observation above has
-- already been replaced. This is the ONE row this seed owns: a documentation address, no MAC.
DELETE FROM address_sighting
  WHERE addr = '192.000.002.010' AND l2_domain = '00000000-0000-0000-0000-000000000000' AND mac = '-';

-- One instant for the observation and its sighting: NOW(6) differs from one statement to the next.
SET @t = NOW(6);

-- ── Declared side: entity 192.0.2.10 is documented as "nas-01" ───────────────────────────────
-- One row per field (attributes-per-row, D3). origin = manual, actor is a human (never 'scanner').
INSERT INTO declared_attribute (entity_id, attr_key, attr_value, origin, actor_id, updated_at) VALUES
  ('00000000-0000-0000-0000-0000000000aa', 'ipv4',     '192.0.2.10', 'manual', 'operator', NOW(6)),
  ('00000000-0000-0000-0000-0000000000aa', 'hostname', 'nas-01',     'manual', 'operator', NOW(6));

-- ── Observed side: the network answered at 192.0.2.10 with a DIFFERENT hostname ───────────────
-- Immutable observation; facts are serialized JSON (the engine compares them in Rust, never SQL).
-- Facts: an IPv4 (in perimeter) + a Hostname that disagrees with the declared "nas-01".
INSERT INTO observation_record (id, connector_id, observed_at, l2_domain, vantage, facts, raw) VALUES
  ('dddddddd-0000-0000-0000-0000000000aa',
   '00000000-0000-0000-0000-000000000000',
   @t,
   '00000000-0000-0000-0000-000000000000',
   '00000000-0000-0000-0000-000000000000',
   '[{"IpV4":{"addr":"192.0.2.10"}},{"Hostname":{"name":"unknown-host","source":"Dns"}}]',
   NULL);

-- ── The address sighting summary: this seed runs AFTER opencmdb has started, so the one-time
-- backfill at boot never saw the observation above — the seed records its sighting itself.
-- Canonical padded address; no hardware address was seen, hence '-'.
INSERT INTO address_sighting (addr, l2_domain, mac, first_seen_at, last_seen_at) VALUES
  ('192.000.002.010', '00000000-0000-0000-0000-000000000000', '-', @t, @t);

-- Result on the page: Entity 192.0.2.10 · gap on `hostname`: declared "nas-01" -> observed
-- "unknown-host" · Reach 0.
--
-- ── To remove this demo later ────────────────────────────────────────────────────────────────
-- DELETE FROM declared_attribute WHERE entity_id = '00000000-0000-0000-0000-0000000000aa';
-- DELETE FROM observation_record WHERE id       = 'dddddddd-0000-0000-0000-0000000000aa';
-- DELETE FROM address_sighting
--   WHERE addr = '192.000.002.010' AND l2_domain = '00000000-0000-0000-0000-000000000000' AND mac = '-';
