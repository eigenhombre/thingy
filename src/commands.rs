use crate::applescript::{run_applescript, parse_list_name, FILTER_COMPLETED};
use crate::todo::Todo;
use rand::Rng;

pub fn show_help() {
    eprintln!("Usage: thingy [command] [args]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  (no args)             Show today's todos");
    eprintln!("  help, -h              Show this help message");
    eprintln!("  add [list] <text>     Add a new todo (defaults to today)");
    eprintln!("  inbox                 Show current inbox todos");
    eprintln!("  today                 Show current today todos");
    eprintln!("  inprog                Show in-progress todos from today");
    eprintln!("  completed             Show completed todos from today");
    eprintln!("  finished              Alias for completed");
    eprintln!("  count                 Show count of non-completed today todos");
    eprintln!("  total                 Alias for count");
    eprintln!("  rm [list] <id>        Remove todo by identifier (defaults to today)");
    eprintln!("  complete [list] <id...> Mark todo(s) complete by identifier");
    eprintln!("  done [list] <id...>     Alias for complete");
    eprintln!("  finish [list] <id...>   Alias for complete");
    eprintln!("  mv <id>               Move todo from inbox to today by identifier");
    eprintln!("  mv <from> <id> [to]   Move todo between lists (defaults to today)");
    eprintln!("  workon [list] <id>    Tag todo as in-progress by identifier");
    eprintln!("  rand                  Pick a random todo from today and mark it in-progress");
    eprintln!("  next [list] <id>      Tag todo as on-deck by identifier");
    eprintln!("  next                  Show the on-deck todo");
    eprintln!("  ondeck                Alias for next");
    eprintln!("  interactive           Interactive mode with keyboard navigation");
    eprintln!("  i                     Alias for interactive");
}

pub fn add_todo(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: 'add' command requires todo text");
        eprintln!("Usage: thingy add [list] <todo text>");
        std::process::exit(1);
    }

    let (list_name, text_args) = match args[0].to_lowercase().as_str() {
        "inbox" => {
            if args.len() < 2 {
                eprintln!("Error: 'add' command requires todo text");
                eprintln!("Usage: thingy add [list] <todo text>");
                std::process::exit(1);
            }
            ("Inbox", &args[1..])
        }
        "today" => {
            if args.len() < 2 {
                eprintln!("Error: 'add' command requires todo text");
                eprintln!("Usage: thingy add [list] <todo text>");
                std::process::exit(1);
            }
            ("Today", &args[1..])
        }
        _ => ("Today", args)
    };

    let todo_text = text_args.join(" ");
    let escaped_text = todo_text.replace("\\", "\\\\").replace("\"", "\\\"");

    let script = format!(
        r#"
tell application "Things3"
    set newTodo to make new to do with properties {{name:"{}"}}
    move newTodo to list "{}"
    return name of newTodo
end tell
"#,
        escaped_text, list_name
    );

    match run_applescript(&script) {
        Ok(result) => {
            println!("Added to {}: {}", list_name, result.trim());
        }
        Err(error) => {
            eprintln!("Error adding todo: {}", error);
            std::process::exit(1);
        }
    }
}

fn parse_list_and_identifier(args: &[String], todos: &[Todo]) -> (&'static str, usize) {
    if args.is_empty() {
        eprintln!("Error: Missing todo identifier or number");
        std::process::exit(1);
    }

    let (list_name, id_str) = if args.len() == 1 {
        ("Today", &args[0])
    } else {
        match args[0].to_lowercase().as_str() {
            "inbox" => ("Inbox", &args[1]),
            "today" => ("Today", &args[1]),
            _ => {
                eprintln!("Error: Unknown list '{}'", args[0]);
                eprintln!("Valid lists: inbox, today");
                std::process::exit(1);
            }
        }
    };

    if let Ok(n) = id_str.parse::<usize>() {
        if n > 0 {
            return (list_name, n);
        }
    }

    let id_upper = id_str.to_uppercase();
    for todo in todos {
        if todo.identifier == id_upper {
            return (list_name, todo.index);
        }
    }

    eprintln!("Error: No todo found with identifier or number '{}'", id_str);
    eprintln!("Use 'thingy {}' to see available todos", list_name.to_lowercase());
    std::process::exit(1);
}

fn fetch_todos_for_list(list_name: &str) -> Vec<Todo> {
    let script = format!(
        r#"
tell application "Things3"
    set listToQuery to list "{}"
    {}
    set output to ""
    set oldDelimiters to AppleScript's text item delimiters
    repeat with todo in listTodos
        set todoName to name of todo
        set todoTags to tag names of todo
        if (count of todoTags) > 0 then
            set AppleScript's text item delimiters to ", "
            set tagString to todoTags as string
            set AppleScript's text item delimiters to oldDelimiters
            set output to output & todoName & "|" & tagString & "\n"
        else
            set output to output & todoName & "|\n"
        end if
    end repeat
    set AppleScript's text item delimiters to oldDelimiters
    return output
end tell
"#,
        list_name, FILTER_COMPLETED
    );

    match run_applescript(&script) {
        Ok(result) => {
            let mut todos = Vec::new();
            for (idx, line) in result.trim().lines().enumerate() {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 1 {
                    let name = parts[0].to_string();
                    let tags = if parts.len() >= 2 && !parts[1].is_empty() {
                        parts[1].to_string()
                    } else {
                        String::new()
                    };
                    todos.push(Todo {
                        name,
                        tags,
                        is_completed: false,
                        index: idx + 1,
                        identifier: String::new(),
                    });
                }
            }
            crate::identifiers::assign_identifiers(&mut todos);
            todos
        }
        Err(error) => {
            eprintln!("Error fetching todos: {}", error);
            std::process::exit(1);
        }
    }
}

fn fetch_completed_todos() -> Vec<Todo> {
    let script = r#"
tell application "Things3"
    set listToQuery to list "Today"
    set allTodos to to dos of listToQuery
    set output to ""
    set oldDelimiters to AppleScript's text item delimiters
    repeat with todo in allTodos
        if status of todo is completed then
            set todoName to name of todo
            set todoTags to tag names of todo
            if (count of todoTags) > 0 then
                set AppleScript's text item delimiters to ", "
                set tagString to todoTags as string
                set AppleScript's text item delimiters to oldDelimiters
                set output to output & todoName & "|" & tagString & "\n"
            else
                set output to output & todoName & "|\n"
            end if
        end if
    end repeat
    set AppleScript's text item delimiters to oldDelimiters
    return output
end tell
"#;

    match run_applescript(script) {
        Ok(result) => {
            let mut todos = Vec::new();
            for (idx, line) in result.trim().lines().enumerate() {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 1 {
                    let name = parts[0].to_string();
                    let tags = if parts.len() >= 2 && !parts[1].is_empty() {
                        parts[1].to_string()
                    } else {
                        String::new()
                    };
                    todos.push(Todo {
                        name,
                        tags,
                        is_completed: true,
                        index: idx + 1,
                        identifier: String::new(),
                    });
                }
            }
            crate::identifiers::assign_identifiers(&mut todos);
            todos
        }
        Err(error) => {
            eprintln!("Error fetching completed todos: {}", error);
            std::process::exit(1);
        }
    }
}

pub fn remove_todo(args: &[String]) {
    let list_name = if args.len() >= 2 {
        match args[0].to_lowercase().as_str() {
            "inbox" => "Inbox",
            "today" => "Today",
            _ => "Today",
        }
    } else {
        "Today"
    };

    let todos = fetch_todos_for_list(list_name);
    let (list_name, todo_num) = parse_list_and_identifier(args, &todos);

    let script = format!(
        r#"
tell application "Things3"
    set listToQuery to list "{}"
    {}
    if (count of listTodos) < {} then
        error "Todo number {} is out of range"
    end if
    set todoToDelete to item {} of listTodos
    set todoName to name of todoToDelete
    delete todoToDelete
    return todoName
end tell
"#,
        list_name, FILTER_COMPLETED, todo_num, todo_num, todo_num
    );

    match run_applescript(&script) {
        Ok(todo_name) => {
            println!("Removed from {}: {}", list_name, todo_name.trim());
        }
        Err(error) => {
            eprintln!("Error removing todo: {}", error);
            std::process::exit(1);
        }
    }
}

pub fn complete_todo(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: 'complete' command requires todo identifier");
        eprintln!("Usage: thingy complete [list] <id...>");
        std::process::exit(1);
    }

    let (list_name, id_args) = match args[0].to_lowercase().as_str() {
        "inbox" => {
            if args.len() < 2 {
                eprintln!("Error: Missing todo identifier after list name");
                std::process::exit(1);
            }
            ("Inbox", &args[1..])
        }
        "today" => {
            if args.len() < 2 {
                eprintln!("Error: Missing todo identifier after list name");
                std::process::exit(1);
            }
            ("Today", &args[1..])
        }
        _ => ("Today", args)
    };

    let todos = fetch_todos_for_list(list_name);
    let mut todo_nums: Vec<usize> = Vec::new();

    for id_str in id_args {
        if let Ok(n) = id_str.parse::<usize>() {
            if n > 0 {
                todo_nums.push(n);
                continue;
            }
        }

        let id_upper = id_str.to_uppercase();
        let mut found = false;
        for todo in &todos {
            if todo.identifier == id_upper {
                todo_nums.push(todo.index);
                found = true;
                break;
            }
        }

        if !found {
            eprintln!("Error: No todo found with identifier or number '{}'", id_str);
            eprintln!("Use 'thingy {}' to see available todos", list_name.to_lowercase());
            std::process::exit(1);
        }
    }

    if todo_nums.is_empty() {
        eprintln!("Error: No valid todo identifiers provided");
        std::process::exit(1);
    }

    todo_nums.sort_by(|a, b| b.cmp(a));

    for todo_num in todo_nums {
        complete_single_todo(list_name, todo_num);
    }
}

fn complete_single_todo(list_name: &str, todo_num: usize) {
    let script = format!(
        r#"
tell application "Things3"
    set listToQuery to list "{}"
    {}
    if (count of listTodos) < {} then
        error "Todo number {} is out of range"
    end if
    set todoToComplete to item {} of listTodos
    set todoName to name of todoToComplete

    set currentTags to tag names of todoToComplete
    if currentTags contains "in-progress" then
        set oldDelimiters to AppleScript's text item delimiters
        set AppleScript's text item delimiters to ", "
        set tagList to text items of currentTags
        set newTagList to {{}}
        repeat with i from 1 to (count of tagList)
            set tagItem to (item i of tagList) as text
            if tagItem is not "in-progress" then
                set end of newTagList to tagItem
            end if
        end repeat
        if (count of newTagList) > 0 then
            set AppleScript's text item delimiters to ", "
            set tag names of todoToComplete to (newTagList as text)
        else
            set tag names of todoToComplete to ""
        end if
        set AppleScript's text item delimiters to oldDelimiters
    end if

    set status of todoToComplete to completed
    return todoName
end tell
"#,
        list_name, FILTER_COMPLETED, todo_num, todo_num, todo_num
    );

    match run_applescript(&script) {
        Ok(todo_name) => {
            println!("Completed: {}", todo_name.trim());
        }
        Err(error) => {
            eprintln!("Error completing todo: {}", error);
            std::process::exit(1);
        }
    }
}

pub fn move_todo(args: &[String]) {
    if args.is_empty() {
        eprintln!("Error: 'mv' command requires todo identifier or number");
        eprintln!("Usage: thingy mv <id>");
        eprintln!("       thingy mv <from> <id> [to]");
        std::process::exit(1);
    }

    let (from_list, id_str, to_list) = if args.len() == 1 {
        ("Inbox", &args[0], "Today")
    } else if args.len() == 2 {
        let from = parse_list_name(&args[0]).unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });
        (from, &args[1], "Today")
    } else {
        let from = parse_list_name(&args[0]).unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });
        let to = parse_list_name(&args[2]).unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });
        (from, &args[1], to)
    };

    let todos = fetch_todos_for_list(from_list);

    let num: usize = if let Ok(n) = id_str.parse::<usize>() {
        if n > 0 {
            n
        } else {
            eprintln!("Error: Invalid todo number '{}'", id_str);
            std::process::exit(1);
        }
    } else {
        let id_upper = id_str.to_uppercase();
        let mut found_index = None;
        for todo in &todos {
            if todo.identifier == id_upper {
                found_index = Some(todo.index);
                break;
            }
        }

        match found_index {
            Some(idx) => idx,
            None => {
                eprintln!("Error: No todo found with identifier or number '{}'", id_str);
                eprintln!("Use 'thingy {}' to see available todos", from_list.to_lowercase());
                std::process::exit(1);
            }
        }
    };

    let script = format!(
        r#"
tell application "Things3"
    set fromList to list "{}"
    set toList to list "{}"
    set listToQuery to fromList
    {}
    if (count of listTodos) < {} then
        error "Todo number {} is out of range"
    end if
    set todoToMove to item {} of listTodos
    set todoName to name of todoToMove
    move todoToMove to toList
    return todoName
end tell
"#,
        from_list, to_list, FILTER_COMPLETED, num, num, num
    );

    match run_applescript(&script) {
        Ok(todo_name) => {
            println!(
                "Moved from {} to {}: {}",
                from_list,
                to_list,
                todo_name.trim()
            );
        }
        Err(error) => {
            eprintln!("Error moving todo: {}", error);
            std::process::exit(1);
        }
    }
}

pub fn show_inbox() {
    show_list("Inbox");
}

pub fn show_today() {
    show_list("Today");
}

pub fn count_todos() {
    let script = format!(
        r#"
tell application "Things3"
    set listToQuery to list "Today"
    {}
    return count of listTodos
end tell
"#,
        FILTER_COMPLETED
    );

    match run_applescript(&script) {
        Ok(count_str) => {
            let count = count_str.trim();
            println!("{} todo{}", count, if count == "1" { "" } else { "s" });
        }
        Err(error) => {
            eprintln!("Error counting todos: {}", error);
            std::process::exit(1);
        }
    }
}

pub fn show_inprog() {
    let todos = fetch_todos_for_list("Today");
    let inprog_todos: Vec<&Todo> = todos
        .iter()
        .filter(|t| t.tags.contains("in-progress"))
        .collect();

    if inprog_todos.is_empty() {
        println!("No in-progress todos");
    } else {
        println!("In-progress todos:");
        for todo in inprog_todos {
            let todo_text = if !todo.tags.is_empty() {
                format!("{} [{}]", todo.name, todo.tags)
            } else {
                todo.name.clone()
            };
            println!(" {} {}", todo.identifier, todo_text);
        }
    }
}

pub fn show_completed() {
    let todos = fetch_completed_todos();

    if todos.is_empty() {
        println!("No completed todos today");
    } else {
        println!("Completed today:");
        for todo in todos {
            let todo_text = if !todo.tags.is_empty() {
                format!("{} [{}]", todo.name, todo.tags)
            } else {
                todo.name.clone()
            };
            println!(" {} {}", todo.identifier, todo_text);
        }
    }
}

fn show_list(list_name: &str) {
    let todos = fetch_todos_for_list(list_name);

    if todos.is_empty() {
        println!("{} is empty", list_name);
    } else {
        println!("{} todos:", list_name);
        for todo in todos {
            let todo_text = if !todo.tags.is_empty() {
                format!("{} [{}]", todo.name, todo.tags)
            } else {
                todo.name.clone()
            };
            println!(" {} {}", todo.identifier, todo_text);
        }
    }
}

fn mark_todo_inprogress(list_name: &str, todo_num: usize) -> String {
    let script = format!(
        r#"
tell application "Things3"
    set listToQuery to list "{}"
    {}
    if (count of listTodos) < {} then
        error "Todo number {} is out of range"
    end if
    set todoToTag to item {} of listTodos
    set todoName to name of todoToTag

    set inProgressTag to missing value
    try
        set inProgressTag to tag "in-progress"
    on error
        set inProgressTag to make new tag with properties {{name:"in-progress"}}
    end try

    set currentTags to tag names of todoToTag
    if currentTags is "" then
        set tag names of todoToTag to "in-progress"
    else if currentTags does not contain "in-progress" then
        set tag names of todoToTag to currentTags & ", in-progress"
    end if
    return todoName
end tell
"#,
        list_name, FILTER_COMPLETED, todo_num, todo_num, todo_num
    );

    match run_applescript(&script) {
        Ok(todo_name) => todo_name,
        Err(error) => {
            eprintln!("Error tagging todo: {}", error);
            std::process::exit(1);
        }
    }
}

pub fn workon_todo(args: &[String]) {
    let list_name = if args.len() >= 2 {
        match args[0].to_lowercase().as_str() {
            "inbox" => "Inbox",
            "today" => "Today",
            _ => "Today",
        }
    } else {
        "Today"
    };

    let todos = fetch_todos_for_list(list_name);
    let (list_name, todo_num) = parse_list_and_identifier(args, &todos);
    let todo_name = mark_todo_inprogress(list_name, todo_num);
    println!("Working on: {}", todo_name.trim());
}

pub fn next_todo(args: &[String]) {
    if args.is_empty() {
        show_next_todo();
    } else {
        tag_next_todo(args);
    }
}

fn tag_next_todo(args: &[String]) {
    let list_name = if args.len() >= 2 {
        match args[0].to_lowercase().as_str() {
            "inbox" => "Inbox",
            "today" => "Today",
            _ => "Today",
        }
    } else {
        "Today"
    };

    let todos = fetch_todos_for_list(list_name);
    let (list_name, todo_num) = parse_list_and_identifier(args, &todos);

    let script = format!(
        r#"
tell application "Things3"
    set listToQuery to list "{}"
    {}
    if (count of listTodos) < {} then
        error "Todo number {} is out of range"
    end if
    set todoToTag to item {} of listTodos
    set todoName to name of todoToTag

    set onDeckTag to missing value
    try
        set onDeckTag to tag "on-deck"
    on error
        set onDeckTag to make new tag with properties {{name:"on-deck"}}
    end try

    set currentTags to tag names of todoToTag
    if currentTags is "" then
        set tag names of todoToTag to "on-deck"
    else if currentTags does not contain "on-deck" then
        set tag names of todoToTag to currentTags & ", on-deck"
    end if
    return todoName
end tell
"#,
        list_name, FILTER_COMPLETED, todo_num, todo_num, todo_num
    );

    match run_applescript(&script) {
        Ok(todo_name) => {
            println!("Tagged as next: {}", todo_name.trim());
        }
        Err(error) => {
            eprintln!("Error tagging todo: {}", error);
            std::process::exit(1);
        }
    }
}

fn show_next_todo() {
    let script = format!(
        r#"
tell application "Things3"
    set listToQuery to list "Today"
    {}
    set output to ""
    set oldDelimiters to AppleScript's text item delimiters
    repeat with todo in listTodos
        set todoTags to tag names of todo
        if todoTags contains "on-deck" then
            set todoName to name of todo
            if (count of todoTags) > 0 then
                set AppleScript's text item delimiters to ", "
                set tagString to todoTags as string
                set AppleScript's text item delimiters to oldDelimiters
                set output to output & todoName & " [" & tagString & "]" & "\n"
            else
                set output to output & todoName & "\n"
            end if
        end if
    end repeat
    set AppleScript's text item delimiters to oldDelimiters
    return output
end tell
"#,
        FILTER_COMPLETED
    );

    match run_applescript(&script) {
        Ok(todos) => {
            let trimmed = todos.trim();
            if trimmed.is_empty() {
                println!("No on-deck todos");
            } else {
                for todo in trimmed.lines() {
                    println!("{}", todo);
                }
            }
        }
        Err(error) => {
            eprintln!("Error querying Things: {}", error);
            std::process::exit(1);
        }
    }
}

pub fn rand_todo() {
    let count_script = format!(
        r#"
tell application "Things3"
    set listToQuery to list "Today"
    {}
    return count of listTodos
end tell
"#,
        FILTER_COMPLETED
    );

    let count: usize = match run_applescript(&count_script) {
        Ok(count_str) => match count_str.trim().parse() {
            Ok(n) => n,
            Err(_) => {
                eprintln!("Error parsing todo count");
                std::process::exit(1);
            }
        },
        Err(error) => {
            eprintln!("Error counting todos: {}", error);
            std::process::exit(1);
        }
    };

    if count == 0 {
        println!("No todos in Today list");
        std::process::exit(0);
    }

    let mut rng = rand::thread_rng();
    let random_num = rng.gen_range(1..=count);

    let todo_name = mark_todo_inprogress("Today", random_num);

    println!("You are working on:\n");
    println!("    {}\n", todo_name.trim());
    println!("Either:\n");
    println!("- do it now");
    println!("- spend five minutes on it and schedule it later");
    println!("- delete the todo");
    println!("- move it out of today into the \"whenever\" bucket");
}
