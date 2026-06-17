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
    eprintln!("  show [list] <id>      Show notes for a todo by identifier");
    eprintln!("  view [list] <id>      Alias for show");
    eprintln!("  log [days]            Show logbook entries (defaults to 1 day)");
    eprintln!("  defer <id>            Defer todo from today to tomorrow");
    eprintln!("  untagged              Show all untagged todos");
    eprintln!("  soonest               Show the todo with the shortest time tag");
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

fn resolve_id(todos: &[Todo], id_str: &str, list_name: &str) -> usize {
    if let Ok(n) = id_str.parse::<usize>() {
        if n > 0 {
            return n;
        }
    }

    let id_upper = id_str.to_uppercase();
    for todo in todos {
        if todo.identifier == id_upper {
            return todo.index;
        }
    }

    eprintln!("Error: No todo found with identifier or number '{}'", id_str);
    eprintln!("Use 'thingy {}' to see available todos", list_name.to_lowercase());
    std::process::exit(1);
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

    (list_name, resolve_id(todos, id_str, list_name))
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
    let num = resolve_id(&todos, id_str, from_list);

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
            print_todo_line(todo);
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
            print_todo_line(&todo);
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
            print_todo_line(&todo);
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
                // No on-deck todos found, fall back to showing the top item:
                let todos = fetch_todos_for_list("Today");
                if todos.is_empty() {
                    println!("No todos in Today list");
                } else {
                    let first = &todos[0];
                    println!("{}", todo_display_text(first));
                }
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
    let todos = fetch_todos_for_list("Today");

    if todos.is_empty() {
        println!("No todos in Today list");
        std::process::exit(0);
    }

    let mut rng = rand::thread_rng();
    let random_idx = rng.gen_range(0..todos.len());
    let selected_todo = &todos[random_idx];

    let todo_name = mark_todo_inprogress("Today", selected_todo.index);

    println!("You are working on:\n");
    println!("    [{}] {}\n", selected_todo.identifier, todo_name.trim());
    println!("Either:\n");
    println!("- do it now");
    println!("- spend five minutes on it and schedule it later");
    println!("- delete the todo");
    println!("- move it out of today into the \"whenever\" bucket");
}

pub fn fetch_todo_notes(list_name: &str, todo_num: usize) -> Result<String, String> {
    let script = format!(
        r#"
tell application "Things3"
    set listToQuery to list "{}"
    {}
    if (count of listTodos) < {} then
        error "Todo number {} is out of range"
    end if
    set todoItem to item {} of listTodos
    return notes of todoItem
end tell
"#,
        list_name, FILTER_COMPLETED, todo_num, todo_num, todo_num
    );

    run_applescript(&script)
}

pub fn show_todo_notes(args: &[String]) {
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

    match fetch_todo_notes(list_name, todo_num) {
        Ok(notes) => {
            let todo = &todos.iter().find(|t| t.index == todo_num).unwrap();
            println!("{}", todo.name);
            let trimmed_notes = notes.trim();
            if !trimmed_notes.is_empty() {
                println!();
                println!("{}", trimmed_notes);
            } else {
                println!();
                println!("(no notes)");
            }
        }
        Err(error) => {
            eprintln!("Error fetching notes: {}", error);
            std::process::exit(1);
        }
    }
}

fn todo_display_text(todo: &Todo) -> String {
    if !todo.tags.is_empty() {
        format!("{} [{}]", todo.name, todo.tags)
    } else {
        todo.name.clone()
    }
}

fn print_todo_line(todo: &Todo) {
    println!(" {} {}", todo.identifier, todo_display_text(todo));
}

fn parse_time_seconds(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let num_end = s.find(|c: char| !c.is_ascii_digit())?;
    if num_end == 0 {
        return None;
    }
    let num: u64 = s[..num_end].parse().ok()?;
    let unit = s[num_end..].trim().to_lowercase();
    match unit.as_str() {
        "s" | "sec" | "secs" | "second" | "seconds" => Some(num),
        "m" | "min" | "mins" | "minute" | "minutes" => Some(num * 60),
        "h" | "hr" | "hrs" | "hour" | "hours" => Some(num * 3600),
        "d" | "day" | "days" => Some(num * 86400),
        _ => None,
    }
}

fn todo_time_secs(todo: &Todo) -> Option<u64> {
    if todo.tags.is_empty() {
        return None;
    }
    for tag in todo.tags.split(',') {
        if let Some(secs) = parse_time_seconds(tag.trim()) {
            return Some(secs);
        }
    }
    None
}

pub fn show_untagged() {
    let todos = fetch_todos_for_list("Today");
    let untagged: Vec<&Todo> = todos.iter()
        .filter(|t| t.tags.is_empty())
        .collect();

    if untagged.is_empty() {
        println!("No untagged todos");
    } else {
        for todo in untagged {
            print_todo_line(todo);
        }
    }
}

pub fn soonest_todo() {
    let todos = fetch_todos_for_list("Today");

    if todos.is_empty() {
        println!("No todos in Today list");
        return;
    }

    let untagged: Vec<&Todo> = todos.iter()
        .filter(|t| todo_time_secs(t).is_none())
        .collect();

    if !untagged.is_empty() {
        for todo in &untagged {
            print_todo_line(todo);
        }
        return;
    }

    // All todos have time tags - find the soonest:
    let mut tagged: Vec<(&Todo, u64)> = todos.iter()
        .filter_map(|t| todo_time_secs(t).map(|secs| (t, secs)))
        .collect();

    tagged.sort_by(|a, b| {
        a.1.cmp(&b.1).then(a.0.name.len().cmp(&b.0.name.len()))
    });

    if let Some((todo, _)) = tagged.first() {
        print_todo_line(todo);
    }
}

pub fn defer_todo(args: &[String]) {
    let todos = fetch_todos_for_list("Today");
    let (_, num) = parse_list_and_identifier(args, &todos);

    let script = format!(
        r#"
tell application "Things3"
    set listToQuery to list "Today"
    {}
    if (count of listTodos) < {} then
        error "Todo number {} is out of range"
    end if
    set todoToDefer to item {} of listTodos
    set todoName to name of todoToDefer
    schedule todoToDefer for (current date) + (1 * days)
    return todoName
end tell
"#,
        FILTER_COMPLETED, num, num, num
    );

    match run_applescript(&script) {
        Ok(todo_name) => {
            println!("Deferred to tomorrow: {}", todo_name.trim());
        }
        Err(error) => {
            eprintln!("Error deferring todo: {}", error);
            std::process::exit(1);
        }
    }
}

pub fn show_log(args: &[String]) {
    let days: i32 = if args.is_empty() {
        1
    } else {
        match args[0].parse() {
            Ok(n) if n > 0 => n,
            _ => {
                eprintln!("Error: days must be a positive number");
                eprintln!("Usage: thingy log [days]");
                std::process::exit(1);
            }
        }
    };

    let script = format!(
        r#"
tell application "Things3"
    set targetDate to (current date) - ({} * days)
    set logbookTodos to to dos of list "Logbook"
    set output to ""
    set oldDelimiters to AppleScript's text item delimiters

    repeat with i from 1 to (count of logbookTodos)
        set todo to item i of logbookTodos
        set completionDate to completion date of todo
        if completionDate is not missing value then
            if completionDate ≥ targetDate then
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
            else if completionDate < targetDate then
                exit repeat
            end if
        end if
    end repeat

    set AppleScript's text item delimiters to oldDelimiters
    return output
end tell
"#,
        days
    );

    match run_applescript(&script) {
        Ok(result) => {
            let trimmed = result.trim();
            if trimmed.is_empty() {
                println!("No logbook entries in the last {} day{}", days, if days == 1 { "" } else { "s" });
            } else {
                for line in trimmed.lines() {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 1 {
                        let name = parts[0];
                        if parts.len() >= 2 && !parts[1].is_empty() {
                            println!("  {} [{}]", name, parts[1]);
                        } else {
                            println!("  {}", name);
                        }
                    }
                }
            }
        }
        Err(error) => {
            eprintln!("Error fetching logbook: {}", error);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_todo(name: &str, identifier: &str, index: usize) -> Todo {
        Todo {
            name: name.to_string(),
            tags: String::new(),
            is_completed: false,
            index,
            identifier: identifier.to_string(),
        }
    }

    #[test]
    fn test_resolve_id_numeric() {
        let todos = vec![make_todo("Buy milk", "BUY", 1)];
        assert_eq!(resolve_id(&todos, "3", "Today"), 3);
    }

    #[test]
    fn test_resolve_id_by_identifier() {
        let todos = vec![
            make_todo("Buy milk", "BUY", 1),
            make_todo("Call mom", "CAL", 2),
        ];
        assert_eq!(resolve_id(&todos, "CAL", "Today"), 2);
    }

    #[test]
    fn test_resolve_id_case_insensitive() {
        let todos = vec![make_todo("Buy milk", "BUY", 1)];
        assert_eq!(resolve_id(&todos, "buy", "Today"), 1);
    }

    #[test]
    fn test_parse_time_seconds_minutes() {
        assert_eq!(parse_time_seconds("5m"), Some(300));
        assert_eq!(parse_time_seconds("2min"), Some(120));
        assert_eq!(parse_time_seconds("1minute"), Some(60));
        assert_eq!(parse_time_seconds("3minutes"), Some(180));
    }

    #[test]
    fn test_parse_time_seconds_hours() {
        assert_eq!(parse_time_seconds("1h"), Some(3600));
        assert_eq!(parse_time_seconds("2hr"), Some(7200));
        assert_eq!(parse_time_seconds("3hours"), Some(10800));
    }

    #[test]
    fn test_parse_time_seconds_days() {
        assert_eq!(parse_time_seconds("1d"), Some(86400));
        assert_eq!(parse_time_seconds("2days"), Some(172800));
    }

    #[test]
    fn test_parse_time_seconds_seconds() {
        assert_eq!(parse_time_seconds("30s"), Some(30));
        assert_eq!(parse_time_seconds("10sec"), Some(10));
        assert_eq!(parse_time_seconds("60seconds"), Some(60));
    }

    #[test]
    fn test_parse_time_seconds_invalid() {
        assert_eq!(parse_time_seconds(""), None);
        assert_eq!(parse_time_seconds("abc"), None);
        assert_eq!(parse_time_seconds("5x"), None);
        assert_eq!(parse_time_seconds("  "), None);
    }
}
