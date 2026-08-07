fn evasion_proc() {
    sqlx::query("CREATE TRIGGER t AFTER INSERT ON observation FOR EACH ROW INSERT INTO declared_attribute (entity_id) VALUES (NEW.id)");
}
