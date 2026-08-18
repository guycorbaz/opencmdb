fn make_ingest_idempotent() {
    // The ordinary gesture, and an overwrite: the governing keyword is `insert into`.
    let q = "INSERT INTO observation_record (id, raw) VALUES (?, ?) \
             ON DUPLICATE KEY UPDATE raw = VALUES(raw)";
    sqlx::query(q);
}
