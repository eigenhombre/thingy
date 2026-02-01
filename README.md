# thingy

![build](https://github.com/eigenhombre/thingy/workflows/CI/badge.svg)

A command-line tool for managing Things3 todos without opening the
app.  Created leaning heavily on Claude (as my Rust skills are still
limited), though I do use this daily as of January, 2026.

## Installation

    make install

This installs the binary to `~/bin/thingy` by default.

To install to a different location, set the `BINDIR` environment variable:

    BINDIR=/usr/local/bin make install

## Usage

    Usage: thingy [command] [args]
    
    Commands:
      (no args)             Show today's todos
      help, -h              Show this help message
      add [list] <text>     Add a new todo (defaults to today)
      inbox                 Show current inbox todos
      today                 Show current today todos
      inprog                Show in-progress todos from today
      completed             Show completed todos from today
      finished              Alias for completed
      count                 Show count of non-completed today todos
      total                 Alias for count
      rm [list] <id>        Remove todo by identifier (defaults to today)
      complete [list] <id...> Mark todo(s) complete by identifier
      done [list] <id...>     Alias for complete
      finish [list] <id...>   Alias for complete
      mv <id>               Move todo from inbox to today by identifier
      mv <from> <id> [to]   Move todo between lists (defaults to today)
      workon [list] <id>    Tag todo as in-progress by identifier
      rand                  Pick a random todo from today and mark it in-progress
      next [list] <id>      Tag todo as on-deck by identifier
      next                  Show the on-deck todo
      ondeck                Alias for next
      show [list] <id>      Show notes for a todo by identifier
      view [list] <id>      Alias for show
      log [days]            Show logbook entries (defaults to 1 day)
      interactive           Interactive mode with keyboard navigation
      i                     Alias for interactive

### Todo Identifiers

Each todo is automatically assigned a unique identifier based on the
first three non-whitespace characters of its name (uppercase). For
example:

- "Buy groceries" → **BUY**
- "Call dentist" → **CAL**
- "Fix bug #123" → **FIX**
- "hi" → **HI**
- "a" → **A**

When multiple todos have the same base identifier, they're
distinguished with numeric suffixes:
- "Hello world" → **HEL**
- "Hello there" → **HE1**
- "Hello again" → **HE2**

Identifiers are **case-insensitive** - you can use `buy`, `BUY`, or
`Buy` interchangeably.

There's no great reason for this system, other than it helps keep me
from confusing to-dos with GitHub tickets, for which I have a [similar
command-line tool](https://github.com/eigenhombre/trish).

**Examples:**

    # Complete a todo by identifier
    thingy done BUY

    # Remove a todo
    thingy rm inbox CAL

    # Work on a todo
    thingy workon HEL

    # Complete multiple todos
    thingy done CAL PIN FLU

**Backward Compatibility:** Numeric positions still work (e.g., `thingy done 1`).

### Interactive Mode

`thingy i` or `thingy interactive` enters an interactive mode with keyboard navigation:

- **↑/↓** or **k/j** - Navigate between todos
- **Space/x** - Toggle completion status
- **X** - Mark complete and log to Logbook
- **/** - Toggle in-progress tag
- **Enter** - View todo notes (press Enter/Esc/q to return)
- **+** - Add new todo
- **L** - Log completed items to Logbook
- **r** - Refresh from Things3
- **Ctrl-L** - Clear screen and redraw
- **q/Esc** - Exit

## Development

Update this README's usage section (from `thingy -h`):

    make readme
