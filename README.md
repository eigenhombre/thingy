# thingy

A command-line tool for managing Things3 todos without opening the app.

Heavily vibe-coded with Claude, though I do use this in daily use
(Jan., 2026).

## Installation

```bash
make install
```

This installs the binary to `~/bin/thingy` by default.

To install to a different location, set the `BINDIR` environment variable:

```bash
BINDIR=/usr/local/bin make install
```

## Usage

```
Usage: thingy [command] [args]

Commands:
  (no args)             Show today's todos
  help, -h              Show this help message
  add [list] <text>     Add a new todo (defaults to inbox)
  inbox                 Show current inbox todos
  today                 Show current today todos
  inprog                Show in-progress todos from today
  completed             Show completed todos from today
  finished              Alias for completed
  count                 Show count of non-completed today todos
  total                 Alias for count
  rm [list] <num>       Remove todo (defaults to today)
  complete [list] [num...] Mark todo(s) complete (defaults to today #1)
  done [list] [num...]    Alias for complete
  finish [list] [num...]  Alias for complete
  mv <num>              Move todo from inbox to today
  mv <from> <num> [to]  Move todo between lists (defaults to today)
  workon [list] <num>   Tag todo as in-progress (defaults to today)
  next [list] <num>     Tag todo as on-deck (defaults to today)
  next                  Show the on-deck todo
  ondeck                Alias for next
  interactive           Interactive mode with keyboard navigation
  i                     Alias for interactive
```

### Interactive Mode

`thingy i` or `thingy interactive` enters an interactive mode with keyboard navigation:

- **↑/↓** - Navigate between todos
- **Space/x** - Toggle completion status
- **/** - Toggle in-progress tag
- **r** - Refresh from Things3
- **q/Esc** - Exit

## Development

Update this README's usage section:
```bash
make readme
```

## TODOs

1. Enhance string escaping to handle newlines, carriage returns, and other special characters
2. Extract AppleScript generation into helper functions to reduce duplication
3. Create reusable functions for tag manipulation operations
4. Add validation that Things3 is installed and available before operations
5. Add unit tests for parsing and validation logic
6. Improve error messages when Things3 is unavailable
7. Refactor tag filtering logic to eliminate duplication
