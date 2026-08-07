fn evasion_utf8() {
    // précédent éééééééé
    let s = "café élève 🚀 中文";
    sqlx::query("INSERT INTO declared_attribute (entity_id) VALUES (?)");
}
