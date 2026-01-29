#[derive(Debug, Clone)]
pub struct Todo {
    pub name: String,
    pub tags: String,
    pub is_completed: bool,
    pub index: usize,
    pub identifier: String,
}

impl Todo {
    pub fn generate_base_identifier(name: &str) -> String {
        let chars: Vec<char> = name
            .chars()
            .filter(|c| !c.is_whitespace())
            .take(3)
            .map(|c| c.to_uppercase().next().unwrap_or('_'))
            .collect();

        if chars.is_empty() {
            "___".to_string()
        } else {
            chars.iter().collect()
        }
    }
}
