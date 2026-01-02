# thingy

A command-line tool for managing Things3 todos without opening the app.

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
