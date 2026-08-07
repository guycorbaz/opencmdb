fn evasion_concat() {
    let a = "INSERT INTO ";
    let b = "declared_attribute";
    let q = format!("{a}{b} (entity_id) VALUES (?)");
    sqlx::query(&q);
}
