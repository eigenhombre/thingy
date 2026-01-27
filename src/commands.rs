use crate::applescript::{run_applescript, parse_list_name, FILTER_COMPLETED};
use rand::Rng;

pub fn show_help() {
    eprintln!("Usage: thingy [command] [args]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  (no args)             Show today's todos");
    eprintln!("  help, -h              Show this help message");
    eprintln!("  add [list] <text>     Add a new todo (defaults to inbox)");
    eprintln!("  inbox                 Show current inbox todos");
    eprintln!("  today                 Show current today todos");
    eprintln!("  inprog                Show in-progress todos from today");
    eprintln!("  completed             Show completed todos from today");
    eprintln!("  finished              Alias for completed");
    eprintln!("  count                 Show count of non-completed today todos");
    eprintln!("  total                 Alias for count");
    eprintln!("  rm [list] <num>       Remove todo (defaults to today)");
    eprintln!("  complete [list] [num...] Mark todo(s) complete (defaults to today #1)");
    eprintln!("  done [list] [num...]    Alias for complete");
    eprintln!("  finish [list] [num...]  Alias for complete");
    eprintln!("  mv <num>              Move todo from inbox to today");
    eprintln!("  mv <from> <num> [to]  Move todo between lists (defaults to today)");
    eprintln!("  workon [list] <num>   Tag todo as in-progress (defaults to today)");
    eprintln!("  rand                  Pick a random todo from today and mark it in-progress");
    eprintln!("  next [list] <num>     Tag todo as on-deck (defaults to today)");
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
        _ => ("Inbox", args)
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

fn parse_list_and_num(args: &[String]) -> (&'static str, usize) {
    if args.is_empty() {
        eprintln!("Error: Missing todo number");
        std::process::exit(1);
    }

    let (list_name, num_str) = if args.len() == 1 {
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

    let todo_num: usize = match num_str.parse() {
        Ok(n) if n > 0 => n,
        _ => {
            eprintln!("Error: Invalid todo number '{}'", num_str);
            eprintln!("Todo number must be a positive integer");
            std::process::exit(1);
        }
    };

    (list_name, todo_num)
}

pub fn remove_todo(args: &[String]) {
    let (list_name, todo_num) = parse_list_and_num(args);

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
        complete_single_todo("Today", 1);
        return;
    }

    let (list_name, num_args) = match args[0].to_lowercase().as_str() {
        "inbox" => {
            if args.len() < 2 {
                eprintln!("Error: Missing todo number after list name");
                std::process::exit(1);
            }
            ("Inbox", &args[1..])
        }
        "today" => {
            if args.len() < 2 {
                eprintln!("Error: Missing todo number after list name");
                std::process::exit(1);
            }
            ("Today", &args[1..])
        }
        _ => ("Today", args)
    };

    let mut todo_nums: Vec<usize> = Vec::new();
    for num_str in num_args {
        match num_str.parse::<usize>() {
            Ok(n) if n > 0 => todo_nums.push(n),
            _ => {
                eprintln!("Error: Invalid todo number '{}'", num_str);
                eprintln!("Todo number must be a positive integer");
                std::process::exit(1);
            }
        }
    }

    if todo_nums.is_empty() {
        eprintln!("Error: No valid todo numbers provided");
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
        eprintln!("Error: 'mv' command requires todo number");
        eprintln!("Usage: thingy mv <num>");
        eprintln!("       thingy mv <from> <num> [to]");
        std::process::exit(1);
    }

    let (from_list, todo_num, to_list) = if args.len() == 1 {
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

    let num: usize = match todo_num.parse() {
        Ok(n) if n > 0 => n,
        _ => {
            eprintln!("Error: Invalid todo number '{}'", todo_num);
            eprintln!("Todo number must be a positive integer");
            std::process::exit(1);
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
    let script = format!(
        r#"
tell application "Things3"
    set listToQuery to list "Today"
    {}
    set output to ""
    set oldDelimiters to AppleScript's text item delimiters
    repeat with todo in listTodos
        set todoTags to tag names of todo
        if todoTags contains "in-progress" then
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
                println!("No in-progress todos");
            } else {
                println!("In-progress todos:");
                for (i, todo) in trimmed.lines().enumerate() {
                    println!("  {}. {}", i + 1, todo);
                }
            }
        }
        Err(error) => {
            eprintln!("Error querying Things: {}", error);
            std::process::exit(1);
        }
    }
}

pub fn show_completed() {
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
                set output to output & todoName & " [" & tagString & "]" & "\n"
            else
                set output to output & todoName & "\n"
            end if
        end if
    end repeat
    set AppleScript's text item delimiters to oldDelimiters
    return output
end tell
"#;

    match run_applescript(script) {
        Ok(todos) => {
            let trimmed = todos.trim();
            if trimmed.is_empty() {
                println!("No completed todos today");
            } else {
                println!("Completed today:");
                for (i, todo) in trimmed.lines().enumerate() {
                    println!("  {}. {}", i + 1, todo);
                }
            }
        }
        Err(error) => {
            eprintln!("Error querying Things: {}", error);
            std::process::exit(1);
        }
    }
}

fn show_list(list_name: &str) {
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
            set output to output & todoName & " [" & tagString & "]" & "\n"
        else
            set output to output & todoName & "\n"
        end if
    end repeat
    set AppleScript's text item delimiters to oldDelimiters
    return output
end tell
"#,
        list_name, FILTER_COMPLETED
    );

    match run_applescript(&script) {
        Ok(todos) => {
            let trimmed = todos.trim();
            if trimmed.is_empty() {
                println!("{} is empty", list_name);
            } else {
                println!("{} todos:", list_name);
                for (i, todo) in trimmed.lines().enumerate() {
                    println!("  {}. {}", i + 1, todo);
                }
            }
        }
        Err(error) => {
            eprintln!("Error querying Things: {}", error);
            std::process::exit(1);
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
    let (list_name, todo_num) = parse_list_and_num(args);
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
    let (list_name, todo_num) = parse_list_and_num(args);

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
