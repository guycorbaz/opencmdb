fn evasion_where() {
    sqlx::query("SELECT entity_id, attr_value FROM declared_attribute WHERE actor_id = 'scanner' AND origin = 'manual'");
}
