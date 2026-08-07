fn evasion_load() {
    sqlx::query("LOAD DATA INFILE '/tmp/x.csv' INTO TABLE declared_attribute");
}
