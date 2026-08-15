fn evasion_runtime_name() {
    // 5.12's residual class, inherited verbatim: a name assembled at runtime defeats any text
    // matcher. Pinned GREEN as a STATED LIMIT, never as a pass.
    let table = format!("observation_{}", "record");
    let q = format!("UPDATE {table} SET raw = ?");
    sqlx::query(&q);
}
