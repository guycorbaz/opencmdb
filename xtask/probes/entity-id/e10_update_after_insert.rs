sqlx::query("INSERT INTO interface (id) VALUES (?)").execute(&mut *tx).await?;
sqlx::query("UPDATE declared_attribute SET attr_value = ?").execute(&mut *tx).await?;
