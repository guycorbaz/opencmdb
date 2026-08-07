#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        sqlx::query("INSERT INTO declared_attribute (entity_id) VALUES (?)");
    }
}
