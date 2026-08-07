fn evasion_vercomment() {
    sqlx::query("/*!50000 INSERT INTO declared_attribute (entity_id) VALUES (?) */");
}
