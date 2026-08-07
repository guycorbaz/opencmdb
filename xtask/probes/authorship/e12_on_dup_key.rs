fn evasion_ondup() {
    sqlx::query("INSERT INTO declared_attribute (entity_id) VALUES (?) ON DUPLICATE KEY UPDATE attr_value = ?");
}
