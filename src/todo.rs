#[derive(Debug, Clone)]
pub struct Todo {
    pub name: String,
    pub tags: String,
    pub is_completed: bool,
    pub index: usize,
}
