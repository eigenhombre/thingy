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

{{USAGE}}

### Todo Identifiers

Each todo is automatically assigned a unique identifier based on the first three non-whitespace characters of its name (uppercase). For example:

- "Buy groceries" → **BUY**
- "Call dentist" → **CAL**
- "Fix bug #123" → **FIX**
- "hi" → **HI**
- "a" → **A**

When multiple todos have the same base identifier, they're distinguished with numeric suffixes:
- "Hello world" → **HEL**
- "Hello there" → **HE1**
- "Hello again" → **HE2**

Identifiers are **case-insensitive** - you can use `buy`, `BUY`, or `Buy` interchangeably.

**Examples:**
```bash
# Complete a todo by identifier
thingy done BUY

# Remove a todo
thingy rm inbox CAL

# Work on a todo
thingy workon HEL

# Complete multiple todos
thingy done CAL PIN FLU
```

**Backward Compatibility:** Numeric positions still work (e.g., `thingy done 1`).

### Interactive Mode

`thingy i` or `thingy interactive` enters an interactive mode with keyboard navigation:

- **↑/↓** - Navigate between todos
- **Space/x** - Toggle completion status
- **/** - Toggle in-progress tag
- **+** - Add new todo
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
