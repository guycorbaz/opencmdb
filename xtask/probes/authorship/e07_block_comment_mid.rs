fn evasion_bcmid() {
    sqlx::query("INSERT /* hi */ INTO declared_attribute (entity_id) VALUES (?)");
}
