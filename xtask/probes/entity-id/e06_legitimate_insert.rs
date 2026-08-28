// The sanctioned write. `authorship` guards this one; D15 says nothing against it.
sqlx::query("INSERT INTO declared_attribute (entity_id, attr_key) VALUES (?, ?)")
    .bind(entity_id)
    .bind(key)
    .execute(&mut *tx)
    .await?;
