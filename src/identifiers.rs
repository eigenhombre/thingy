use std::collections::HashMap;
use crate::todo::Todo;

pub fn assign_identifiers(todos: &mut [Todo]) {
    let mut seen_ids: HashMap<String, usize> = HashMap::new();

    for todo in todos.iter_mut() {
        let base_id = Todo::generate_base_identifier(&todo.name);
        let collision_count = seen_ids.entry(base_id.clone()).or_insert(0);

        if *collision_count > 999 {
            panic!("Too many collisions for identifier {}", base_id);
        }

        todo.identifier = if *collision_count == 0 {
            base_id
        } else {
            resolve_collision(&base_id, *collision_count)
        };

        *collision_count += 1;
    }
}

fn resolve_collision(base_id: &str, n: usize) -> String {
    let base_len = base_id.chars().count();
    match n {
        1..=9 if base_len >= 2 => {
            let prefix: String = base_id.chars().take(base_len - 1).collect();
            format!("{}{}", prefix, n)
        }
        1..=9 => format!("{}", n),
        10..=99 if base_len >= 2 => {
            let prefix: String = base_id.chars().take(base_len.saturating_sub(2).max(0)).collect();
            format!("{}{}", prefix, n)
        }
        10..=99 => format!("{}", n),
        100..=999 => format!("{}", n),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_todo(name: &str, index: usize) -> Todo {
        Todo {
            name: name.to_string(),
            tags: String::new(),
            is_completed: false,
            index,
            identifier: String::new(),
        }
    }

    #[test]
    fn test_basic_identifier_generation() {
        let mut todos = vec![
            make_todo("fooitem", 1),
            make_todo("Buy milk", 2),
            make_todo("Call mom", 3),
        ];

        assign_identifiers(&mut todos);

        assert_eq!(todos[0].identifier, "FOO");
        assert_eq!(todos[1].identifier, "BUY");
        assert_eq!(todos[2].identifier, "CAL");
    }

    #[test]
    fn test_short_names() {
        let mut todos = vec![
            make_todo("hi", 1),
            make_todo("a", 2),
            make_todo("ab", 3),
        ];

        assign_identifiers(&mut todos);

        assert_eq!(todos[0].identifier, "HI");
        assert_eq!(todos[1].identifier, "A");
        assert_eq!(todos[2].identifier, "AB");
    }

    #[test]
    fn test_empty_and_whitespace() {
        let mut todos = vec![
            make_todo("", 1),
            make_todo("  ", 2),
            make_todo("   \t\n", 3),
        ];

        assign_identifiers(&mut todos);

        assert_eq!(todos[0].identifier, "___");
        assert_eq!(todos[1].identifier, "__1");
        assert_eq!(todos[2].identifier, "__2");
    }

    #[test]
    fn test_numbers() {
        let mut todos = vec![
            make_todo("123 test", 1),
            make_todo("456", 2),
        ];

        assign_identifiers(&mut todos);

        assert_eq!(todos[0].identifier, "123");
        assert_eq!(todos[1].identifier, "456");
    }

    #[test]
    fn test_basic_collision() {
        let mut todos = vec![
            make_todo("Hello world", 1),
            make_todo("Hello there", 2),
            make_todo("Hello again", 3),
        ];

        assign_identifiers(&mut todos);

        assert_eq!(todos[0].identifier, "HEL");
        assert_eq!(todos[1].identifier, "HE1");
        assert_eq!(todos[2].identifier, "HE2");
    }

    #[test]
    fn test_many_collisions() {
        let mut todos: Vec<Todo> = (0..15)
            .map(|i| make_todo(&format!("Hello {}", i), i + 1))
            .collect();

        assign_identifiers(&mut todos);

        assert_eq!(todos[0].identifier, "HEL");
        assert_eq!(todos[1].identifier, "HE1");
        assert_eq!(todos[9].identifier, "HE9");
        assert_eq!(todos[10].identifier, "H10");
        assert_eq!(todos[11].identifier, "H11");
        assert_eq!(todos[14].identifier, "H14");
    }

    #[test]
    fn test_three_digit_collisions() {
        let mut todos: Vec<Todo> = (0..105)
            .map(|i| make_todo(&format!("Hello {}", i), i + 1))
            .collect();

        assign_identifiers(&mut todos);

        assert_eq!(todos[0].identifier, "HEL");
        assert_eq!(todos[99].identifier, "H99");
        assert_eq!(todos[100].identifier, "100");
        assert_eq!(todos[101].identifier, "101");
        assert_eq!(todos[104].identifier, "104");
    }

    #[test]
    fn test_special_characters() {
        let mut todos = vec![
            make_todo("#!/bin/bash", 1),
            make_todo("call Jeremy", 2),
            make_todo("@mention", 3),
        ];

        assign_identifiers(&mut todos);

        assert_eq!(todos[0].identifier, "#!/");
        assert_eq!(todos[1].identifier, "CAL");
        assert_eq!(todos[2].identifier, "@ME");
    }

    #[test]
    fn test_unicode() {
        let mut todos = vec![
            make_todo("café", 1),
            make_todo("naïve", 2),
            make_todo("日本語", 3),
        ];

        assign_identifiers(&mut todos);

        assert_eq!(todos[0].identifier, "CAF");
        assert_eq!(todos[1].identifier, "NAÏ");
        assert_eq!(todos[2].identifier, "日本語");
    }

    #[test]
    fn test_short_name_collisions() {
        let mut todos = vec![
            make_todo("ab", 1),
            make_todo("AB", 2),
            make_todo("Ab", 3),
        ];

        assign_identifiers(&mut todos);

        assert_eq!(todos[0].identifier, "AB");
        assert_eq!(todos[1].identifier, "A1");
        assert_eq!(todos[2].identifier, "A2");
    }

    #[test]
    fn test_single_char_collisions() {
        let mut todos = vec![
            make_todo("a", 1),
            make_todo("A", 2),
            make_todo("a ", 3),
        ];

        assign_identifiers(&mut todos);

        assert_eq!(todos[0].identifier, "A");
        assert_eq!(todos[1].identifier, "1");
        assert_eq!(todos[2].identifier, "2");
    }

    #[test]
    fn test_uniqueness() {
        let mut todos = vec![
            make_todo("Buy groceries", 1),
            make_todo("Call dentist", 2),
            make_todo("Buy tickets", 3),
            make_todo("Buy coffee", 4),
            make_todo("Finish report", 5),
        ];

        assign_identifiers(&mut todos);

        let ids: Vec<String> = todos.iter().map(|t| t.identifier.clone()).collect();
        let unique_ids: std::collections::HashSet<_> = ids.iter().collect();

        assert_eq!(
            ids.len(),
            unique_ids.len(),
            "All identifiers must be unique"
        );
    }

    #[test]
    #[should_panic(expected = "Too many collisions")]
    fn test_collision_limit() {
        let mut todos: Vec<Todo> = (0..1001)
            .map(|i| make_todo(&format!("Hello {}", i), i + 1))
            .collect();

        assign_identifiers(&mut todos);
    }
}
