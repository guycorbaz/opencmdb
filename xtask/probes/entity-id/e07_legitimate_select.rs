sqlx::query_as("SELECT entity_id, attr_key FROM declared_attribute ORDER BY entity_id")
    .fetch_all(&pool)
    .await?;
