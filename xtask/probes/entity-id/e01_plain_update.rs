// D15's own specimen, in the language a refactor writes it in.
sqlx::query("UPDATE declared_attribute SET entity_id = ? WHERE entity_id = ?")
    .bind(target)
    .bind(source)
    .execute(&pool)
    .await?;
