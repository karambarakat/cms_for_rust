use cms_for_rust::collections::Collection;

#[derive(Collection)]
struct Todo {
    #[field(Regex(r#"^.{1,100}$"#))]
    pub title: String,
    pub done: bool,
    pub description: Option<String>,
}

impl Collection for Todo {
    fn table_name() -> &'static str {
        "Todo"
    }
}
