fn evasion_raw() {
    let q = r#"INSERT INTO declared_attribute (entity_id) VALUES (?)"#;
    sqlx::query(q);
}
