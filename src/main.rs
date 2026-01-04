use std::env;

mod applescript;
mod commands;
mod interactive;
mod todo;

use commands::*;
use interactive::interactive_mode;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        show_today();
        return;
    }

    let command = &args[0];

    match command.as_str() {
        "help" | "-h" | "--help" => show_help(),
        "add" => add_todo(&args[1..]),
        "inbox" => show_inbox(),
        "today" => show_today(),
        "inprog" => show_inprog(),
        "completed" | "finished" => show_completed(),
        "count" | "total" => count_todos(),
        "rm" => remove_todo(&args[1..]),
        "complete" | "done" | "finish" => complete_todo(&args[1..]),
        "mv" | "move" => move_todo(&args[1..]),
        "workon" => workon_todo(&args[1..]),
        "next" | "ondeck" => next_todo(&args[1..]),
        "i" | "interactive" => interactive_mode(),
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            eprintln!();
            show_help();
            std::process::exit(1);
        }
    }
}
