fn evasion_orderby() {
    sqlx::query("SELECT entity_id FROM declared_attribute ORDER BY origin_obs_id");
}
