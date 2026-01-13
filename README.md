# Agent-Board

A simple CLI task manager built in Rust. Works for humans and AI agents alike.

## What It Does

Track tasks, checklists, and comments across boards. Data stored locally in SQLite.

## Quick Start

```bash
# Create a board
agent-board board create "My Project"

# Add a card
agent-board card create board_xxx "Fix the thing"

# List your cards
agent-board mine
```

## Install

```bash
cargo build --release
# Binary at ./target/release/agent-board
```

## Output Formats

```bash
--format table   # Pretty tables (default)
--format json    # Machine-readable
--format simple  # Just IDs
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `AGENT_BOARD_SESSION_ID` | Your session ID for `mine` and `--assign-to-me` |
| `AGENT_BOARD_DB_PATH` | Override default database location |

## Data Location

- Default: `~/.agent-board/data.db`

## Commands

```bash
# Board commands
agent-board board list                    # List all boards
agent-board board get <board_id>          # Get board overview and card summary
agent-board board create "Name"           # Create a new board

# Card commands
agent-board mine                          # Get all cards assigned to you
agent-board card list <board_id>          # List cards on a board
agent-board card get <card_id>            # Get full card details
agent-board card create <board_id> "Name" # Create a new card
agent-board card update <card_id> ...     # Update card fields (name, status, tags, assignee)

# Checklist commands
agent-board checklist add <card_id> --item "Task 1" --item "Task 2"  # Add checklist items
agent-board checklist check <item_id>     # Mark item as done
agent-board checklist check <item_id> --uncheck  # Mark item as not done

# Comment commands
agent-board comment add <card_id> "text"  # Add a comment to a card
agent-board comment list <card_id>        # List all comments on a card
```

## License

MIT
