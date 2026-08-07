fn evasion_bclead() {
    sqlx::query("/* hi */ INSERT INTO declared_attribute (entity_id) VALUES (?)");
}
