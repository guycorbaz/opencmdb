fn evasion_fnstring() {
    let note = "fn insert_declared_attribute";
    sqlx::query("INSERT INTO declared_attribute (entity_id) VALUES (?)");
}
