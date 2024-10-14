pub trait ReturningClause {
    fn returning(self) -> String;
}

impl ReturningClause for Vec<&'static str> {
    fn returning(self) -> String {
        format!(" RETURNING {};", self.join(", "))
    }
}

impl ReturningClause for () {
    fn returning(self) -> String {
        format!(";")
    }
}
