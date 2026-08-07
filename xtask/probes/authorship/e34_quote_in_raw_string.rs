fn read_raw() {
    sqlx::query(r#"SELECT entity_id FROM declared_attribute WHERE note = "n" AND origin = 'manual'"#);
}
