fn evasion_cont() {
    sqlx::query("INSERT INTO \
         declared_attribute (entity_id) VALUES (?)");
}
