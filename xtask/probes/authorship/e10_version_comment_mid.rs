fn evasion_vercomment2() {
    sqlx::query("INSERT /*!50000 IGNORE */ INTO declared_attribute (entity_id) VALUES (?)");
}
