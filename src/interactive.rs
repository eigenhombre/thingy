use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    terminal, ExecutableCommand,
};

use crate::applescript::run_applescript;
use crate::todo::Todo;

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
                    });
                }
            }
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
        format!("{}{}. \x1b[9m{}\x1b[29m", prefix, todo.index, todo_text)
    } else {
        format!("{}{}. {}", prefix, todo.index, todo_text)
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
            modifiers: _,
            kind: _,
            state: _,
        })) = event::read()
        {
            match code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    break;
                }
                KeyCode::Up => {
                    if selected_idx > 0 {
                        selected_idx -= 1;
                        redraw_list(&todos, selected_idx, displayed_count);
                    }
                }
                KeyCode::Down => {
                    if selected_idx < todos.len() - 1 {
                        selected_idx += 1;
                        redraw_list(&todos, selected_idx, displayed_count);
                    }
                }
                KeyCode::Char(' ') | KeyCode::Char('x') => {
                    let todo = &todos[selected_idx];
                    if let Err(e) = toggle_todo_completion(todo) {
                        let _ = terminal::disable_raw_mode();
                        eprintln!("\nError toggling todo: {}", e);
                        std::process::exit(1);
                    }

                    todos[selected_idx].is_completed = !todos[selected_idx].is_completed;
                    redraw_list(&todos, selected_idx, displayed_count);
                }
                KeyCode::Char('/') => {
                    let todo = &todos[selected_idx];
                    match toggle_inprogress_tag(todo) {
                        Ok(new_tags) => {
                            todos[selected_idx].tags = new_tags.trim().to_string();
                            redraw_list(&todos, selected_idx, displayed_count);
                        }
                        Err(e) => {
                            let _ = terminal::disable_raw_mode();
                            eprintln!("\nError toggling in-progress tag: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                KeyCode::Char('r') => {
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
                            let _ = terminal::disable_raw_mode();
                            eprintln!("\nError refreshing todos: {}", e);
                            std::process::exit(1);
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
