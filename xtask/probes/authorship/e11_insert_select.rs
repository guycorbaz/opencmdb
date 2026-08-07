fn evasion_insel() {
    sqlx::query("INSERT INTO declared_attribute SELECT * FROM staging");
}
