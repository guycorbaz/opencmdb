fn mark_documented() {
    let q = "UPDATE observation_record SET raw = ? WHERE id = ?";
    sqlx::query(q);
}
