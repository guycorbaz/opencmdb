-- An append is how observations come into being; it is not an overwrite.
INSERT INTO observation_record (id, connector_id, observed_at, l2_domain, vantage, facts)
  VALUES ('x','y',NOW(6),'z','w','[]');
