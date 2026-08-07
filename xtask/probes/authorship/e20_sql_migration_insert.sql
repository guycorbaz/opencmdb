-- a migration
INSERT INTO declared_attribute (entity_id, attr_key, attr_value, origin, actor_id, updated_at)
  VALUES ('x','hostname','nas','manual','engine',NOW(6));
