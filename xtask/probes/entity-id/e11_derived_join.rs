// 🔴 The review's HIGH 3: measured re-pointing the testimony, and green before the widening,
// because the derived table's `select` ends after `update`.
sqlx::query("UPDATE (SELECT 1 AS one) s JOIN declared_attribute d ON 1 = 1 SET d.entity_id = ?")
    .bind(target)
    .execute(&pool)
    .await?;
