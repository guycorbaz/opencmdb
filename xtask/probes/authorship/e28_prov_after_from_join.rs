fn evasion_join() {
    sqlx::query("SELECT o.a FROM other o JOIN declared_attribute d ON d.entity_id = o.id WHERE d.origin = 'manual'");
}
