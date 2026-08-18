fn evasion_line_comment() {
    // The `--` lives INSIDE the literal: a naive stripper treats the rest of the line as a
    // comment and loses the write. Story 5.12 measured exactly this on its first draft.
    let q = "SELECT 1; -- housekeeping
             UPDATE observation_record SET facts = ? WHERE id = ?";
    sqlx::query(q);
}
