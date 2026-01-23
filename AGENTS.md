# Agent-Board Codebase Guide

## Project Overview

A Rust CLI tool for task management with SQLite storage (via libsql). Built for AI agents and humans to track work items, checklists, and comments.

## Architecture

```
src/
├── main.rs      # Entry point, command dispatch, error handling
├── cli.rs       # Clap-based CLI definitions (Commands, subcommands, args)
├── models.rs    # Data structures (Agent, Board, Card, ChecklistItem, Comment, Status)
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
- `Commands` enum: `Version`, `Get`, `List`, `Create`, `Update`, `Delete`, `Mine`, `Whoami`
- Subcommand enums: `ListCommands`, `CreateCommands`, `UpdateCommands`, `DeleteCommands`
- Uses clap derive macros

### models.rs
- `Agent` struct: id, name, command, working_directory, description, timestamps, deactivated_at
- `AgentUpdate` struct for agent update operations
- `Status` enum: `Todo`, `InProgress`, `PendingReview`, `Done` (serde snake_case)
- `OutputFormat` enum: `Json`, `Table`, `Simple`, `Pretty`
- `CardUpdate` struct for update operations (avoids too-many-args clippy warning)
- `Board`, `Card`, `ChecklistItem`, `Comment` structs
- `Card.checklist: Vec<ChecklistItem>` - single checklist per card (simplified model)
- `Board` and `Card` have `deleted_at: Option<DateTime<Utc>>` for soft delete
- `AgentBoardData` holds all entities for JSON serialization

### db.rs
- `Database` struct with `conn: Connection` (libsql)
- `load()` opens SQLite at `~/.agent-board/data.db` or `AGENT_BOARD_DB_PATH`
- Auto-initializes schema from `schema.sql`
- Async CRUD methods for agents, boards, cards, checklist items, comments
- `generate_id(prefix)` creates IDs like `agent_abc123def456`, `card_abc123def456`
- `generate_agent_name()` uses `names` crate for random adjective-noun names

### schema.sql
- SQLite schema with tables: `agents`, `boards`, `cards`, `card_tags`, `checklist_items`, `comments`
- `agents` table: id, name (unique), command, working_directory, description, timestamps, deactivated_at
- `boards` and `cards` tables have `deleted_at TEXT` column for soft delete
- `checklist_items` table references `card_id` directly (simplified - one checklist per card)
- Foreign keys with `ON DELETE CASCADE`
- Indexes for common queries (board_id, status, assigned_to, card_id)

### output.rs
- `print_agents()`, `print_agent()`, `print_agent_whoami()` for agent output
- `print_cards()`, `print_card()`, `print_boards()`, `print_board()`, `print_kanban()`
- `print_comments()`, `print_checklist_items()` for listing comments/checklist items
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
./target/debug/agent-board create board "Test" --description "Test board"
./target/debug/agent-board create card board_xxx "Task name"
./target/debug/agent-board --help
```

## Agent Identity System

Agent identity is tracked via the `AGENT_BOARD_AGENT_ID` environment variable. Identity is **auto-created** when needed, or can be registered explicitly.

### Auto-Creation (Recommended)

When using `--assign-to-me` or `--status in-progress` without an agent ID set, one is automatically created:

```bash
# Just start working - agent identity created automatically
agent-board update card card_xxx --status in-progress
# Note: Created agent identity agent_abc123 for this session.
#       To persist: export AGENT_BOARD_AGENT_ID=agent_abc123
```

### Explicit Registration

```bash
# Register a new agent (auto-generated name)
./target/debug/agent-board create agent

# Register with explicit name
./target/debug/agent-board create agent code-reviewer --command claude --description "Reviews PRs"

# Set identity for session
export AGENT_BOARD_AGENT_ID=agent_abc123
```

### Agent Model
- **id**: Unique identifier (e.g., `agent_abc123def456`)
- **name**: Human-readable name (unique, auto-generated if not provided)
- **command**: The CLI command used to invoke the agent (e.g., `stakpak`, `claude`, `aider`)
- **working_directory**: Directory where the agent was registered (used for context validation)
- **description**: Optional description of the agent's purpose

### Agent Commands
```bash
# Show current agent identity
export AGENT_BOARD_AGENT_ID=agent_xxx
./target/debug/agent-board whoami

# List all agents
./target/debug/agent-board list agents
./target/debug/agent-board list agents --include-inactive

# Get agent details (uses top-level get command)
./target/debug/agent-board get <agent_id>

# Update agent
./target/debug/agent-board update agent <agent_id> --name new-name --workdir .

# Delete agent (soft delete)
./target/debug/agent-board delete agent <agent_id>
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
./target/debug/agent-board delete card <card_id>

# Delete a board (soft delete, cascades to all cards in board)
./target/debug/agent-board delete board <board_id>

# Delete an agent (soft delete)
./target/debug/agent-board delete agent <agent_id>

# Delete a comment (hard delete)
./target/debug/agent-board delete comment <comment_id>

# Delete a checklist item (hard delete)
./target/debug/agent-board delete checklist-item <item_id>
```

### Viewing Deleted/Inactive Items
```bash
# List boards including deleted ones
./target/debug/agent-board list boards --include-deleted

# List cards including deleted ones (works even on deleted boards)
./target/debug/agent-board list cards <board_id> --include-deleted

# List agents including inactive ones
./target/debug/agent-board list agents --include-inactive
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
./target/debug/agent-board get <board_id> --format pretty
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

## CLI Command Reference

### List Commands
```bash
list boards [--include-deleted]
list cards <board_id> [--status STATUS] [--assigned-to ID] [--tag TAG] [--include-deleted]
list agents [--include-inactive]
list comments <card_id>
```

### Create Commands
```bash
create board <name> [--description DESC]
create card <board_id> <name> [--description DESC] [--status STATUS]
create agent [name] [--command CMD] [--description DESC]
create checklist <card_id> --item "text" [--item "text"...]   # adds items to card's checklist
create comment <card_id> <text> | --file PATH
```

### Update Commands
```bash
update board <board_id> [--name NAME] [--description DESC]
update card <card_id> [--name NAME] [--description DESC] [--status STATUS] [--assign ID|--assign-to-me] [--add-tag TAG] [--remove-tag TAG]
update agent <agent_id> [--name NAME] [--command CMD] [--description DESC] [--workdir PATH]
update checklist-item <item_id> --check|--uncheck
```

### Delete Commands
```bash
delete board <board_id>           # soft delete
delete card <card_id>             # soft delete
delete agent <agent_id>           # soft delete
delete comment <comment_id>       # hard delete
delete checklist-item <item_id>   # hard delete
```

## Future Improvements

- [ ] Add `--filter` for more flexible queries
- [x] Add `delete card` command
- [x] Add `delete board` command
- [x] Add `--format pretty` for visual kanban board
- [x] Add agent identity system
- [x] Add top-level `get` command (auto-detects entity type from ID prefix)
- [x] Add `version` subcommand
- [x] Simplify CLI to `<action> <entity>` pattern (get/list/create/update/delete)
- [x] Add `list comments` command
- [x] Add `update board` command
- [x] Add `delete comment`, `delete checklist-item` commands
- [x] Simplify checklist model (single checklist per card, items shown when viewing card)
- [ ] Add restore commands for soft-deleted entities
- [ ] Add shell completions (`clap_complete`)
- [ ] Add `--dry-run` for mutations
