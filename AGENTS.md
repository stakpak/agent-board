# Taskboard Codebase Guide

## Project Overview

A Rust CLI tool for task management with local JSON storage. Built for AI agents and humans to track work items, checklists, and comments.

## Architecture

```
src/
├── main.rs      # Entry point, command dispatch, error handling
├── cli.rs       # Clap-based CLI definitions (Commands, subcommands, args)
├── models.rs    # Data structures (Board, Card, Checklist, Comment, Status)
├── db.rs        # JSON file database operations (CRUD for all entities)
└── output.rs    # Output formatting (table, json, simple)
```

## Key Files

### main.rs
- Entry point with `main() -> ExitCode`
- `run(cli: Cli)` dispatches commands to db operations
- `TaskboardError` enum with exit codes (0-6)
- Extracts `default_format`, `quiet`, `session_id_result` before match to avoid borrow issues

### cli.rs
- `Cli` struct with global options (`--format`, `--quiet`, `--verbose`)
- `Commands` enum: `Mine`, `Card`, `Checklist`, `Comment`, `Board`
- Subcommand enums: `CardCommands`, `ChecklistCommands`, `CommentCommands`, `BoardCommands`
- Uses clap derive macros

### models.rs
- `Status` enum: `Todo`, `InProgress`, `Done` (serde snake_case)
- `OutputFormat` enum: `Json`, `Table`, `Simple`
- `CardUpdate` struct for update operations (avoids too-many-args clippy warning)
- `Board`, `Card`, `Checklist`, `ChecklistItem`, `Comment` structs
- `TaskboardData` holds all entities for JSON serialization

### db.rs
- `Database` struct with `data: TaskboardData` and `path: PathBuf`
- `load()` reads from `~/.taskboard/data.json` or `TASKBOARD_DB_PATH`
- `save()` writes pretty JSON
- CRUD methods for boards, cards, checklists, comments
- `generate_id(prefix)` creates IDs like `card_abc123def456`

### output.rs
- `print_cards()`, `print_card()`, `print_boards()`, `print_board()`, `print_comments()`
- Uses `tabled` crate for table output
- JSON output via `serde_json::to_string_pretty`
- Simple output: just IDs, one per line

## Dependencies

```toml
clap = { version = "4.4", features = ["derive"] }  # CLI parsing
serde = { version = "1.0", features = ["derive"] }  # Serialization
serde_json = "1.0"                                  # JSON
chrono = { version = "0.4", features = ["serde"] }  # Timestamps
uuid = { version = "1.6", features = ["v4"] }       # ID generation
dirs = "5.0"                                        # Home directory
tabled = "0.15"                                     # Table output
thiserror = "1.0"                                   # Error handling
```

## Build & Test

```bash
# Build
cargo build
cargo build --release

# Run clippy
cargo clippy

# Test manually
./target/debug/taskboard board create "Test" --description "Test board"
./target/debug/taskboard card create board_xxx "Task name"
./target/debug/taskboard --help
```

## Common Patterns

### Adding a New Command

1. Add variant to appropriate enum in `cli.rs`
2. Add match arm in `main.rs` `run()` function
3. Add db method in `db.rs`
4. Add output function in `output.rs` if needed

### Borrow Checker Notes

- Extract values from `cli` before the `match cli.command` block
- Use `cli.format.clone()` since `OutputFormat` doesn't impl `Copy`
- Session ID is fetched before match to avoid partial move issues

### Status CLI Values

- Use `in-progress` (hyphen) on CLI, stored as `in_progress` (underscore) in JSON
- Clap's `ValueEnum` handles the conversion

## Data Storage

- Default: `~/.taskboard/data.json`
- Override: `TASKBOARD_DB_PATH` env var
- Auto-creates parent directories on save
- Pretty-printed JSON for human readability

## Exit Codes

| Code | Constant | Meaning |
|------|----------|---------|
| 0 | - | Success |
| 1 | `General` | General error |
| 2 | `InvalidArgs` | Invalid arguments |
| 4 | `NotFound` | Entity not found |
| 5 | `PermissionDenied` | Permission denied |
| 6 | `SessionConflict` | Session conflict |

## Future Improvements

- [ ] Add `--filter` for more flexible queries
- [ ] Add `card delete` command
- [ ] Add `board delete` command
- [ ] Add shell completions (`clap_complete`)
- [ ] Add `--dry-run` for mutations
- [ ] Consider SQLite for larger datasets
