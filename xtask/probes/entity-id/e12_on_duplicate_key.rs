// 🔴 The review's HIGH 4: measured re-pointing through `declared_one_adoption_per_field`, at the
// SANCTIONED site, with all ten gates green before the widening.
sqlx::query(
    "INSERT INTO declared_attribute (entity_id, attr_key, attr_value) VALUES (?, ?, ?) \
     ON DUPLICATE KEY UPDATE entity_id = VALUES(entity_id)",
)
.execute(&mut *tx)
.await?;
