fn evasion_nested() {
    fn insert_declared_attribute() {}
    sqlx::query("INSERT INTO declared_attribute (entity_id) VALUES (?)");
}
