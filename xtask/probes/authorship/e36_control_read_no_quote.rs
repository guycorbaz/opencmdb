fn read_control() {
    sqlx::query("SELECT entity_id FROM declared_attribute WHERE origin = ?");
}
