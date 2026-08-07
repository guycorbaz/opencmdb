fn evasion_semi() {
    sqlx::query("SET x=1; INSERT INTO declared_attribute (entity_id) VALUES (?)");
}
