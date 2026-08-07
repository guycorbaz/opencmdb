fn read_escaped() {
    sqlx::query("SELECT entity_id FROM declared_attribute WHERE note = \"n\" AND actor_id = ?");
}
