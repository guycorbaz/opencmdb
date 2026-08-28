// ⚠️ RED BY DECISION, and this probe is where that decision is measured rather than stated.
// The statement UPDATEs another table and only READS this one, so D15 is not breached — but the
// gate refuses any UPDATE whose statement names `declared_attribute`, because telling the two
// apart needs a parser, and a matcher that must parse is wrong in both directions.
// Measured on the committed tree: no such statement exists, so the breadth costs nothing today.
sqlx::query("UPDATE interface SET last_seen_at = (SELECT MAX(updated_at) FROM declared_attribute)")
    .execute(&pool)
    .await?;
