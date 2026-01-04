use std::process::Command;

pub const FILTER_COMPLETED: &str = r#"
    set allTodos to to dos of listToQuery
    set listTodos to {}
    repeat with todo in allTodos
        if status of todo is not completed then
            set end of listTodos to todo
        end if
    end repeat
"#;

pub fn run_applescript(script: &str) -> Result<String, String> {
    let result = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        }
        Err(e) => Err(format!("Error executing osascript: {}", e)),
    }
}

pub fn parse_list_name(name: &str) -> Result<&'static str, String> {
    match name.to_lowercase().as_str() {
        "inbox" => Ok("Inbox"),
        "today" => Ok("Today"),
        _ => Err(format!("Unknown list '{}'. Valid lists: inbox, today", name)),
    }
}
