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

{{USAGE}}

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
