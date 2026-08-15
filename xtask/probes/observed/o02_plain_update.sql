-- a migration that rewrites history
UPDATE observation_record SET raw = '{}' WHERE id = 'x';
