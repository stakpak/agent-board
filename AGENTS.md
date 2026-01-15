# Agent-Board Codebase Guide

## Project Overview

A Rust CLI tool for task management with SQLite storage (via libsql). Built for AI agents and humans to track work items, checklists, and comments.

## Architecture

```
src/
├── main.rs      # Entry point, command dispatch, error handling
├── cli.rs       # Clap-based CLI definitions (Commands, subcommands, args)
├── models.rs    # Data structures (Agent, Board, Card, Checklist, Comment, Status)
├── db.rs        # SQLite database operations (CRUD for all entities)
├── output.rs    # Output formatting (table, json, simple)
└── schema.sql   # SQLite schema definitions
```

## Key Files

### main.rs
- Entry point with `main() -> ExitCode`
- `run(cli: Cli)` dispatches commands to db operations
- `AgentBoardError` enum with exit codes (0-6)
- Extracts `default_format`, `quiet`, `agent_id_result` before match to avoid borrow issues

### cli.rs
- `Cli` struct with global options (`--format`, `--quiet`, `--verbose`)
- `Commands` enum: `Mine`, `Agent`, `Card`, `Checklist`, `Comment`, `Board`
- Subcommand enums: `AgentCommands`, `CardCommands`, `ChecklistCommands`, `CommentCommands`, `BoardCommands`
- Uses clap derive macros

### models.rs
- `Agent` struct: id, name, command, working_directory, description, timestamps, deactivated_at
- `AgentUpdate` struct for agent update operations
- `Status` enum: `Todo`, `InProgress`, `PendingReview`, `Done` (serde snake_case)
- `OutputFormat` enum: `Json`, `Table`, `Simple`, `Pretty`
- `CardUpdate` struct for update operations (avoids too-many-args clippy warning)
- `Board`, `Card`, `Checklist`, `ChecklistItem`, `Comment` structs
- `Board` and `Card` have `deleted_at: Option<DateTime<Utc>>` for soft delete
- `AgentBoardData` holds all entities for JSON serialization

### db.rs
- `Database` struct with `conn: Connection` (libsql)
- `load()` opens SQLite at `~/.agent-board/data.db` or `AGENT_BOARD_DB_PATH`
- Auto-initializes schema from `schema.sql`
- Async CRUD methods for agents, boards, cards, checklists, comments
- `generate_id(prefix)` creates IDs like `agent_abc123def456`, `card_abc123def456`
- `generate_agent_name()` uses `names` crate for random adjective-noun names

### schema.sql
- SQLite schema with tables: `agents`, `boards`, `cards`, `card_tags`, `checklists`, `checklist_items`, `comments`
- `agents` table: id, name (unique), command, working_directory, description, timestamps, deactivated_at
- `boards` and `cards` tables have `deleted_at TEXT` column for soft delete
- Foreign keys with `ON DELETE CASCADE`
- Indexes for common queries (board_id, status, assigned_to)

### output.rs
- `print_agents()`, `print_agent()`, `print_agent_whoami()` for agent output
- `print_cards()`, `print_card()`, `print_boards()`, `print_board()`, `print_kanban()`, `print_comments()`
- Uses `tabled` crate for table output
- JSON output via `serde_json::to_string_pretty`
- Simple output: just IDs, one per line
- Pretty output: visual kanban board with colored columns (board get only)
- Deleted items show `[DELETED]` suffix, inactive agents show `[INACTIVE]`

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
libsql = { version = "0.9", features = ["core"] }  # SQLite database
tokio = { version = "1.29", features = ["rt", "macros"] }  # Async runtime
colored = "2.1"                                         # Terminal colors
names = { version = "0.14.0", default-features = false }  # Random name generation
```

## Build & Test

```bash
# Build
cargo build
cargo build --release

# Run clippy
cargo clippy

# Test manually
./target/debug/agent-board board create "Test" --description "Test board"
./target/debug/agent-board card create board_xxx "Task name"
./target/debug/agent-board --help
```

## Agent Identity System

Agents must register an identity before working on tasks. Identity is tracked via the `AGENT_BOARD_AGENT_ID` environment variable.

### Agent Model
- **id**: Unique identifier (e.g., `agent_abc123def456`)
- **name**: Human-readable name (unique, auto-generated if not provided)
- **command**: The CLI command used to invoke the agent (e.g., `stakpak`, `claude`, `aider`)
- **working_directory**: Directory where the agent was registered (used for context validation)
- **description**: Optional description of the agent's purpose

### Agent Commands
```bash
# Register a new agent (auto-generated name)
./target/debug/agent-board agent register --command stakpak

# Register with explicit name
./target/debug/agent-board agent register --command claude --name code-reviewer --description "Reviews PRs"

# Show current agent identity
export AGENT_BOARD_AGENT_ID=agent_xxx
./target/debug/agent-board agent whoami

# List all agents
./target/debug/agent-board agent list
./target/debug/agent-board agent list --include-inactive

# Get agent details
./target/debug/agent-board agent get <agent_id>

# Update agent
./target/debug/agent-board agent update <agent_id> --name new-name --workdir .

# Unregister (soft delete)
./target/debug/agent-board agent unregister <agent_id>
```

### Agent Workflow
```bash
# 1. Register once
agent-board agent register --command stakpak
# Output: Created agent: agent_abc123 (Name: swift-falcon)
#         To use this agent, run:
#           export AGENT_BOARD_AGENT_ID=agent_abc123

# 2. Set env var
export AGENT_BOARD_AGENT_ID=agent_abc123

# 3. Work on tasks
agent-board card update card_xxx --status in-progress --assign-to-me
agent-board mine
```

### Working Directory Warning
`whoami` warns if current directory doesn't match registered working directory:
```
WARNING: Current directory (/tmp) does not match registered working directory
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
- Agent ID is fetched before match to avoid partial move issues

### Status CLI Values

- Use `in-progress` (hyphen) on CLI, stored as `in_progress` (underscore) in JSON
- Clap's `ValueEnum` handles the conversion

## Data Storage

- Default: `~/.agent-board/data.db` (SQLite)
- Override: `AGENT_BOARD_DB_PATH` env var
- Auto-creates parent directories and initializes schema on first run
- Uses libsql for SQLite operations

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `AGENT_BOARD_AGENT_ID` | Current agent identity for `mine`, `whoami`, `--assign-to-me` |
| `AGENT_BOARD_DB_PATH` | Override default database path |

## Exit Codes

| Code | Constant | Meaning |
|------|----------|---------|
| 0 | - | Success |
| 1 | `General` | General error |
| 2 | `InvalidArgs` | Invalid arguments |
| 4 | `NotFound` | Entity not found |
| 5 | `PermissionDenied` | Permission denied |
| 6 | `SessionConflict` | Session conflict |

## Soft Delete

Boards, cards, and agents support soft delete - records are marked with `deleted_at` or `deactivated_at` timestamp rather than being permanently removed.

### Delete Commands
```bash
# Delete a card (soft delete)
./target/debug/agent-board card delete <card_id>

# Delete a board (soft delete, cascades to all cards in board)
./target/debug/agent-board board delete <board_id>

# Unregister an agent (soft delete)
./target/debug/agent-board agent unregister <agent_id>
```

### Viewing Deleted/Inactive Items
```bash
# List boards including deleted ones
./target/debug/agent-board board list --include-deleted

# List cards including deleted ones (works even on deleted boards)
./target/debug/agent-board card list <board_id> --include-deleted

# List agents including inactive ones
./target/debug/agent-board agent list --include-inactive
```

Deleted items display with `[DELETED]` suffix, inactive agents show `[INACTIVE]`.

### Implementation Notes
- All list/get queries filter `WHERE deleted_at IS NULL` or `WHERE deactivated_at IS NULL` by default
- Board deletion cascades: soft-deletes all cards in that board
- `--include-deleted` / `--include-inactive` flags bypass the filter to show all records
- Data is preserved in DB for potential recovery (restore not yet implemented)

## Visual Kanban Board

Display a board as a visual kanban with `--format pretty`:

```bash
./target/debug/agent-board board get <board_id> --format pretty
```

Features:
- 4-column layout: TODO → IN PROGRESS → PENDING REVIEW → DONE
- Color-coded headers and card names by status
- Cards display: title (2 lines), ID, assignee, tags (2 lines), comment count
- Tags shown in blue, IDs and comments dimmed

Example output:
```
┌───────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│ My Board - board_abc123                                                                                           │
├────────────────────────────┬────────────────────────────┬────────────────────────────┬────────────────────────────┤
│ TODO                       │ IN PROGRESS                │ PENDING REVIEW             │ DONE                       │
│ (2 cards)                  │ (1 cards)                  │ (0 cards)                  │ (3 cards)                  │
├────────────────────────────┼────────────────────────────┼────────────────────────────┼────────────────────────────┤
│ ┌────────────────────────┐ │ ┌────────────────────────┐ │                            │ ┌────────────────────────┐ │
│ │ Task name here         │ │ │ Another task           │ │                            │ │ Completed task         │ │
│ │                        │ │ │                        │ │                            │ │                        │ │
│ │ card_xyz789            │ │ │ card_def456            │ │                            │ │ card_ghi012            │ │
│ │ @agent_session_id      │ │ │ @agent_session_id      │ │                            │ │ @agent_session_id      │ │
│ │ #tag1 #tag2            │ │ │ #urgent                │ │                            │ │                        │ │
│ │                        │ │ │                        │ │                            │ │                        │ │
│ │ [2 comments]           │ │ │ [0 comments]           │ │                            │ │ [5 comments]           │ │
│ └────────────────────────┘ │ └────────────────────────┘ │                            │ └────────────────────────┘ │
└────────────────────────────┴────────────────────────────┴────────────────────────────┴────────────────────────────┘
```

## Future Improvements

- [ ] Add `--filter` for more flexible queries
- [x] Add `card delete` command
- [x] Add `board delete` command
- [x] Add `--format pretty` for visual kanban board
- [x] Add agent identity system
- [ ] Add `card restore` / `board restore` / `agent reactivate` commands
- [ ] Add shell completions (`clap_complete`)
- [ ] Add `--dry-run` for mutations
