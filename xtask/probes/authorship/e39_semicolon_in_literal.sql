-- Story 6.3's code review: a provenance READ hidden behind a `;` inside a quoted literal.
-- Measured GREEN before the shared statement-bound fix; the control (same line, no semicolon)
-- reddened all along, which is what made the hole a finding rather than a guess.
SELECT attr_value FROM declared_attribute WHERE attr_value = 'a;b' AND actor_id = 'operator';
