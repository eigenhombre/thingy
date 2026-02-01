use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal, ExecutableCommand,
};

use crate::applescript::run_applescript;
use crate::commands::fetch_todo_notes;
use crate::todo::Todo;

fn exit_with_error(message: &str) -> ! {
    let _ = terminal::disable_raw_mode();
    eprintln!("\n{}", message);
    std::process::exit(1);
}

fn remove_inprogress_tag_from_local_state(todo: &mut Todo) {
    if todo.tags.contains("in-progress") {
        todo.tags = todo.tags
            .split(", ")
            .filter(|t| *t != "in-progress")
            .collect::<Vec<_>>()
            .join(", ");
    }
}

fn clear_and_redraw_all_todos(todos: &[Todo], selected_idx: usize) -> usize {
    let mut stdout = io::stdout();
    stdout.execute(terminal::Clear(terminal::ClearType::All)).unwrap();
    stdout.execute(cursor::MoveTo(0, 0)).unwrap();

    for (idx, todo) in todos.iter().enumerate() {
        let line = render_todo_line(todo, idx == selected_idx);
        print!("{}\r\n", line);
    }
    stdout.flush().unwrap();
    todos.len()
}

fn fetch_all_todos() -> Result<Vec<Todo>, String> {
    let script = r#"
tell application "Things3"
    set listToQuery to list "Today"
    set allTodos to to dos of listToQuery
    set output to ""
    set oldDelimiters to AppleScript's text item delimiters
    repeat with todo in allTodos
        set todoName to name of todo
        set todoStatus to status of todo
        set todoTags to tag names of todo

        if todoStatus is completed then
            set statusFlag to "COMPLETED"
        else
            set statusFlag to "NOTCOMPLETED"
        end if

        if (count of todoTags) > 0 then
            set AppleScript's text item delimiters to ", "
            set tagString to todoTags as string
            set AppleScript's text item delimiters to oldDelimiters
            set output to output & statusFlag & "|" & todoName & "|" & tagString & "\n"
        else
            set output to output & statusFlag & "|" & todoName & "|\n"
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
                if parts.len() >= 2 {
                    let is_completed = parts[0] == "COMPLETED";
                    let name = parts[1].to_string();
                    let tags = if parts.len() >= 3 {
                        parts[2].to_string()
                    } else {
                        String::new()
                    };
                    todos.push(Todo {
                        name,
                        tags,
                        is_completed,
                        index: idx + 1,
                        identifier: String::new(),
                    });
                }
            }
            crate::identifiers::assign_identifiers(&mut todos);
            Ok(todos)
        }
        Err(e) => Err(e),
    }
}

fn toggle_todo_completion(todo: &Todo) -> Result<(), String> {
    let script = if todo.is_completed {
        format!(
            r#"
tell application "Things3"
    set listToQuery to list "Today"
    set allTodos to to dos of listToQuery
    if (count of allTodos) < {} then
        error "Todo index {} is out of range"
    end if
    set todoToUpdate to item {} of allTodos
    set status of todoToUpdate to open
end tell
"#,
            todo.index, todo.index, todo.index
        )
    } else {
        format!(
            r#"
tell application "Things3"
    set listToQuery to list "Today"
    set allTodos to to dos of listToQuery
    if (count of allTodos) < {} then
        error "Todo index {} is out of range"
    end if
    set todoToUpdate to item {} of allTodos

    set currentTags to tag names of todoToUpdate
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
            set tag names of todoToUpdate to (newTagList as text)
        else
            set tag names of todoToUpdate to ""
        end if
        set AppleScript's text item delimiters to oldDelimiters
    end if

    set status of todoToUpdate to completed
end tell
"#,
            todo.index, todo.index, todo.index
        )
    };

    run_applescript(&script).map(|_| ())
}

fn toggle_inprogress_tag(todo: &Todo) -> Result<String, String> {
    let has_tag = todo.tags.split(", ").any(|t| t == "in-progress");

    let script = if has_tag {
        format!(
            r#"
tell application "Things3"
    set listToQuery to list "Today"
    set allTodos to to dos of listToQuery
    if (count of allTodos) < {} then
        error "Todo index {} is out of range"
    end if
    set todoToUpdate to item {} of allTodos

    set currentTags to tag names of todoToUpdate
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
        set tag names of todoToUpdate to (newTagList as text)
    else
        set tag names of todoToUpdate to ""
    end if
    set AppleScript's text item delimiters to oldDelimiters
    return tag names of todoToUpdate
end tell
"#,
            todo.index, todo.index, todo.index
        )
    } else {
        format!(
            r#"
tell application "Things3"
    set listToQuery to list "Today"
    set allTodos to to dos of listToQuery
    if (count of allTodos) < {} then
        error "Todo index {} is out of range"
    end if
    set todoToUpdate to item {} of allTodos

    set inProgressTag to missing value
    try
        set inProgressTag to tag "in-progress"
    on error
        set inProgressTag to make new tag with properties {{name:"in-progress"}}
    end try

    set currentTags to tag names of todoToUpdate
    if currentTags is "" then
        set tag names of todoToUpdate to "in-progress"
    else if currentTags does not contain "in-progress" then
        set tag names of todoToUpdate to currentTags & ", in-progress"
    end if
    return tag names of todoToUpdate
end tell
"#,
            todo.index, todo.index, todo.index
        )
    };

    run_applescript(&script)
}

fn render_todo_line(todo: &Todo, is_selected: bool) -> String {
    let prefix = if is_selected { "> " } else { "  " };
    let todo_text = if !todo.tags.is_empty() {
        format!("{} [{}]", todo.name, todo.tags)
    } else {
        todo.name.clone()
    };

    if todo.is_completed {
        format!("{}{} \x1b[9m{}\x1b[29m", prefix, todo.identifier, todo_text)
    } else {
        format!("{}{} {}", prefix, todo.identifier, todo_text)
    }
}

fn redraw_list(todos: &[Todo], selected_idx: usize, displayed_count: usize) {
    let mut stdout = io::stdout();

    // Move cursor up by the number of items currently displayed
    stdout.execute(cursor::MoveUp(displayed_count as u16)).unwrap();

    // Clear from cursor down to remove old list
    stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();

    // Redraw each line
    for (idx, todo) in todos.iter().enumerate() {
        let line = render_todo_line(todo, idx == selected_idx);
        print!("{}\r\n", line);
    }
    stdout.flush().unwrap();
}

fn clear_line_and_print(stdout: &mut io::Stdout, text: &str) {
    stdout.execute(cursor::MoveToColumn(0)).unwrap();
    stdout.execute(terminal::Clear(terminal::ClearType::CurrentLine)).unwrap();
    print!("{}", text);
    stdout.flush().unwrap();
}

fn redraw_from_top(todos: &[Todo], selected_idx: usize) {
    let mut stdout = io::stdout();
    stdout.execute(cursor::MoveToColumn(0)).unwrap();
    stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();

    for (idx, todo) in todos.iter().enumerate() {
        let line = render_todo_line(todo, idx == selected_idx);
        print!("{}\r\n", line);
    }
    stdout.flush().unwrap();
}

fn log_completed_and_refresh(
    todos: &mut Vec<Todo>,
    selected_idx: &mut usize,
    displayed_count: &mut usize,
) {
    let script = r#"
tell application "Things3"
    log completed now
end tell
"#;
    if let Err(e) = run_applescript(script) {
        exit_with_error(&format!("Error logging completed: {}", e));
    }

    match fetch_all_todos() {
        Ok(new_todos) => {
            *todos = new_todos;
            if *selected_idx >= todos.len() && todos.len() > 0 {
                *selected_idx = todos.len() - 1;
            }
            if todos.is_empty() {
                let _ = terminal::disable_raw_mode();
                let _ = io::stdout().execute(cursor::Show);
                println!("\nNo todos in Today list");
                std::process::exit(0);
            }
            redraw_list(&todos, *selected_idx, *displayed_count);
            *displayed_count = todos.len();
        }
        Err(e) => {
            exit_with_error(&format!("Error refreshing todos: {}", e));
        }
    }
}

fn add_new_todo(todos: &[Todo], displayed_count: usize) -> Result<Option<String>, String> {
    let mut stdout = io::stdout();
    let mut input = String::new();

    // Move cursor to top of list:
    if displayed_count > 0 {
        stdout.execute(cursor::MoveUp(displayed_count as u16)).unwrap();
    }

    // Clear from cursor down to remove the existing list:
    stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();

    // Show input prompt at top:
    print!("+ ___ ");
    stdout.flush().unwrap();

    // Print all existing todos below:
    for todo in todos {
        print!("\r\n{}", render_todo_line(todo, false));
    }
    stdout.flush().unwrap();

    // Move cursor back to input line:
    if todos.len() > 0 {
        stdout.execute(cursor::MoveUp(todos.len() as u16)).unwrap();
        stdout.execute(cursor::MoveToColumn(6)).unwrap();
    }
    stdout.flush().unwrap();

    loop {
        if let Ok(Event::Key(KeyEvent {
            code,
            modifiers: _,
            kind: _,
            state: _,
        })) = event::read()
        {
            let should_update_display = match code {
                KeyCode::Char(c) => {
                    input.push(c);
                    true
                }
                KeyCode::Backspace => {
                    if !input.is_empty() {
                        input.pop();
                        true
                    } else {
                        false
                    }
                }
                KeyCode::Enter => {
                    if !input.trim().is_empty() {
                        // Escape backslashes and quotes for AppleScript string literal:
                        let escaped_text = input.trim()
                            .replace("\\", "\\\\")
                            .replace("\"", "\\\"");

                        let script = format!(
                            r#"
tell application "Things3"
    set newTodo to make new to do with properties {{name:"{}"}}
    move newTodo to list "Today"
    return name of newTodo
end tell
"#,
                            escaped_text
                        );

                        match run_applescript(&script) {
                            Ok(name) => return Ok(Some(name.trim().to_string())),
                            Err(e) => return Err(e),
                        }
                    } else {
                        return Ok(None);
                    }
                }
                KeyCode::Esc => {
                    return Ok(None);
                }
                _ => false,
            };

            if should_update_display {
                let identifier = if input.is_empty() {
                    "___".to_string()
                } else {
                    Todo::generate_base_identifier(&input)
                };
                clear_line_and_print(&mut stdout, &format!("+ {} {}", identifier, input));
            }
        }
    }
}

fn show_notes_view(todo: &Todo, displayed_count: usize) {
    let mut stdout = io::stdout();

    stdout.execute(cursor::MoveUp(displayed_count as u16)).unwrap();
    stdout.execute(terminal::Clear(terminal::ClearType::FromCursorDown)).unwrap();

    println!("{}\r", todo.name);
    println!("\r");

    match fetch_todo_notes("Today", todo.index) {
        Ok(notes) => {
            let trimmed_notes = notes.trim();
            if !trimmed_notes.is_empty() {
                println!("{}\r", trimmed_notes);
            } else {
                println!("(no notes)\r");
            }
        }
        Err(e) => {
            println!("Error fetching notes: {}\r", e);
        }
    }

    stdout.flush().unwrap();

    loop {
        if let Ok(Event::Key(KeyEvent {
            code,
            modifiers: _,
            kind: _,
            state: _,
        })) = event::read()
        {
            match code {
                KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') => {
                    return;
                }
                _ => {}
            }
        }
    }
}

pub fn interactive_mode() {
    let mut todos = match fetch_all_todos() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error fetching todos: {}", e);
            std::process::exit(1);
        }
    };

    if todos.is_empty() {
        println!("No todos in Today list");
        return;
    }

    let mut selected_idx = 0;

    for (idx, todo) in todos.iter().enumerate() {
        let line = render_todo_line(todo, idx == 0);
        println!("{}", line);
    }

    let mut displayed_count = todos.len();

    if let Err(e) = terminal::enable_raw_mode() {
        eprintln!("Error enabling raw mode: {}", e);
        return;
    }

    let mut stdout = io::stdout();
    let _ = stdout.execute(cursor::Hide);

    loop {
        if let Ok(Event::Key(KeyEvent {
            code,
            modifiers,
            kind: _,
            state: _,
        })) = event::read()
        {
            match (code, modifiers) {
                (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => {
                    break;
                }
                (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                    displayed_count = clear_and_redraw_all_todos(&todos, selected_idx);
                }
                (KeyCode::Enter, _) => {
                    let todo = &todos[selected_idx];
                    show_notes_view(todo, displayed_count);
                    displayed_count = clear_and_redraw_all_todos(&todos, selected_idx);
                }
                (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                    if selected_idx > 0 {
                        selected_idx -= 1;
                        redraw_list(&todos, selected_idx, displayed_count);
                    }
                }
                (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                    if selected_idx < todos.len() - 1 {
                        selected_idx += 1;
                        redraw_list(&todos, selected_idx, displayed_count);
                    }
                }
                (KeyCode::Char(' '), _) | (KeyCode::Char('x'), _) => {
                    let todo = &todos[selected_idx];
                    let was_completed = todo.is_completed;
                    if let Err(e) = toggle_todo_completion(todo) {
                        exit_with_error(&format!("Error toggling todo: {}", e));
                    }

                    todos[selected_idx].is_completed = !was_completed;

                    if !was_completed {
                        remove_inprogress_tag_from_local_state(&mut todos[selected_idx]);
                    }

                    redraw_list(&todos, selected_idx, displayed_count);
                }
                (KeyCode::Char('/'), _) => {
                    let todo = &todos[selected_idx];
                    match toggle_inprogress_tag(todo) {
                        Ok(new_tags) => {
                            todos[selected_idx].tags = new_tags.trim().to_string();
                            redraw_list(&todos, selected_idx, displayed_count);
                        }
                        Err(e) => {
                            exit_with_error(&format!("Error toggling in-progress tag: {}", e));
                        }
                    }
                }
                (KeyCode::Char('r'), _) | (KeyCode::Char('g'), _) => {
                    match fetch_all_todos() {
                        Ok(new_todos) => {
                            todos = new_todos;
                            if selected_idx >= todos.len() && todos.len() > 0 {
                                selected_idx = todos.len() - 1;
                            }
                            if todos.is_empty() {
                                let _ = terminal::disable_raw_mode();
                                let _ = io::stdout().execute(cursor::Show);
                                println!("\nNo todos in Today list");
                                return;
                            }
                            redraw_list(&todos, selected_idx, displayed_count);
                            displayed_count = todos.len();
                        }
                        Err(e) => {
                            exit_with_error(&format!("Error refreshing todos: {}", e));
                        }
                    }
                }
                (KeyCode::Char('L'), _) => {
                    log_completed_and_refresh(&mut todos, &mut selected_idx, &mut displayed_count);
                }
                (KeyCode::Char('X'), _) => {
                    let todo = &todos[selected_idx];
                    let was_completed = todo.is_completed;

                    if let Err(e) = toggle_todo_completion(todo) {
                        exit_with_error(&format!("Error toggling todo: {}", e));
                    }

                    if !was_completed {
                        todos[selected_idx].is_completed = true;
                        remove_inprogress_tag_from_local_state(&mut todos[selected_idx]);
                        log_completed_and_refresh(&mut todos, &mut selected_idx, &mut displayed_count);
                    } else {
                        todos[selected_idx].is_completed = false;
                        redraw_list(&todos, selected_idx, displayed_count);
                    }
                }
                (KeyCode::Char('+'), _) => {
                    match add_new_todo(&todos, displayed_count) {
                        Ok(Some(_)) => {
                            match fetch_all_todos() {
                                Ok(new_todos) => {
                                    todos = new_todos;
                                    selected_idx = 0;
                                    redraw_from_top(&todos, selected_idx);
                                    displayed_count = todos.len();
                                }
                                Err(e) => {
                                    exit_with_error(&format!("Error refreshing todos: {}", e));
                                }
                            }
                        }
                        Ok(None) => {
                            redraw_from_top(&todos, selected_idx);
                        }
                        Err(e) => {
                            exit_with_error(&format!("Error adding todo: {}", e));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let _ = io::stdout().execute(cursor::Show);
    let _ = terminal::disable_raw_mode();
}
