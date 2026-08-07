fn evasion_nbsp() {
    sqlx::query("INSERT INTO declared_attribute (entity_id) VALUES (?)");
}
