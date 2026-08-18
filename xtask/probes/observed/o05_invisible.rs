fn evasion_invisible() {
    // A zero-width space sits INSIDE the verb, not at a token boundary — the shape story 5.12
    // measured its first `is_invisible` enumeration missing.
    let q = "UPD​ATE observation_record SET raw = ?";
    sqlx::query(q);
}
