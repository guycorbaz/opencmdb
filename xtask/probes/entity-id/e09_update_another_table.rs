// Another table's UPDATE is another gate's business, or nobody's.
sqlx::query("UPDATE interface SET last_seen_at = ? WHERE id = ?")
    .execute(&pool)
    .await?;
