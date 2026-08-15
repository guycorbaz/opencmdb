fn read_it() {
    // LOAD-BEARING NEGATIVE: the guarded table name IS present, and the gate must stay green
    // because the governing keyword is `select`.
    let q = "SELECT id, facts FROM observation_record WHERE id = ?";
    sqlx::query(q);
}
