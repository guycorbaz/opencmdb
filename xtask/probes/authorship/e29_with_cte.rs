fn evasion_cte() {
    sqlx::query("WITH x AS (SELECT origin FROM declared_attribute) SELECT * FROM x");
}
