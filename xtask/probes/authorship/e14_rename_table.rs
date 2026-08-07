fn evasion_rename() {
    sqlx::query("RENAME TABLE declared_attribute TO declared_attribute_old");
}
