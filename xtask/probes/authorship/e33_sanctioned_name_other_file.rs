fn insert_declared_attribute(pool: &Pool) {
    sqlx::query("INSERT INTO declared_attribute (entity_id, attr_value) VALUES (?, ?)");
}
