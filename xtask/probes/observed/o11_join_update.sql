UPDATE identity_link l
  JOIN observation_record o ON o.id = l.observation_id
  SET o.raw = '{}';
