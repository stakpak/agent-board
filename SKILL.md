---
name: agent-board-cli
description: |
  Command-line task management tool for AI coding agents and humans. Provides local JSON-based task boards for tracking work items, checklists, and comments without requiring an external API.
  
  Use this skill when:
  - Managing multi-step coding tasks
  - Tracking progress on complex implementations  
  - Breaking down work into checkable subtasks
  - Documenting decisions and progress via comments
  - Coordinating work across agent sessions
license: MIT
tags:
  - task-management
  - cli
  - agent-tools
  - productivity
  - rust
metadata:
  author: Stakpak
  version: "0.1.0"
---

# Agent-Board CLI for AI Agents

## Overview

`agent-board` is a command-line task management tool designed for AI coding agents and humans. It provides a local JSON-based task board for tracking work items, checklists, and comments without requiring an external API.

## Quick Reference

```bash
# Board operations
agent-board board create "Project Name" --description "Description"
agent-board board list
agent-board board get <board_id>

# Card operations
agent-board card create <board_id> "Task name" --description "Details" --status todo
agent-board card list <board_id> [--status todo|in-progress|done]
agent-board card get <card_id>
agent-board card update <card_id> --status in-progress
agent-board card update <card_id> --status done --agent-session-id null

# Checklist operations
agent-board checklist add <card_id> --name "Subtasks" --item "Step 1" --item "Step 2"
agent-board checklist check <item_id>
agent-board checklist check <item_id> --uncheck

# Comment operations
agent-board comment add <card_id> "Progress update or notes"
agent-board comment list <card_id>

# Get my assigned cards
agent-board mine [--status todo|in-progress|done]
```

## Environment Setup

Set your session ID to enable card assignment:

```bash
export AGENT_BOARD_SESSION_ID=agent_session_001
```

Data is stored in `~/.agent-board/data.json` by default. Override with:

```bash
export AGENT_BOARD_DB_PATH=/custom/path/data.json
```

## Workflow Patterns

### Starting a New Task

```bash
# 1. Find or create a board
agent-board board list
agent-board board create "Feature Development" --description "Q1 features"

# 2. Create a card for the task
agent-board card create board_abc123 "Implement user authentication" \
  --description "Add OAuth2 login flow with Google and GitHub providers"

# 3. Break down into subtasks
agent-board checklist add card_xyz789 \
  --name "Implementation Steps" \
  --item "Set up OAuth client credentials" \
  --item "Create auth endpoints" \
  --item "Add session management" \
  --item "Write integration tests" \
  --item "Update documentation"

# 4. Start working (auto-assigns to your session)
agent-board card update card_xyz789 --status in-progress

# 5. Document that you're starting
agent-board comment add card_xyz789 "Beginning OAuth implementation"
```

### Tracking Progress

```bash
# Check off completed subtasks
agent-board checklist check item_001
agent-board checklist check item_002

# Add progress notes
agent-board comment add card_xyz789 "OAuth endpoints complete. Starting session management."

# View current state
agent-board card get card_xyz789
```

### Completing Work

```bash
# Check remaining items
agent-board checklist check item_003
agent-board checklist check item_004
agent-board checklist check item_005

# Mark done and unassign
agent-board card update card_xyz789 --status done --agent-session-id null

# Add completion summary
agent-board comment add card_xyz789 "Implementation complete. All tests passing."
```

### Reviewing Work

```bash
# See all your assigned cards
agent-board mine

# Filter by status
agent-board mine --status in-progress

# Get board overview
agent-board board get board_abc123

# List all cards on a board
agent-board card list board_abc123 --status todo
```

## Output Formats

Use `--format` to control output:

| Format | Use Case |
|--------|----------|
| `table` | Human-readable display (default) |
| `json` | Parsing with jq, programmatic access |
| `simple` | Just IDs, one per line, for scripting |

```bash
# Get card ID for scripting
CARD_ID=$(agent-board card create board_123 "New task" --format simple)

# Get full JSON for parsing
agent-board card get card_xyz789 --format json | jq '.status'
```

## Status Values

| Status | CLI Value | Meaning |
|--------|-----------|---------|
| Todo | `todo` | Not started |
| In Progress | `in-progress` | Currently being worked on |
| Done | `done` | Completed |

**Note:** Use `in-progress` (with hyphen) on the command line, not `in_progress`.

## Card Assignment

Cards are automatically assigned when:
- You update status to `in-progress` (uses `AGENT_BOARD_SESSION_ID`)

Explicit assignment control:
```bash
# Assign to specific session
agent-board card update card_123 --agent-session-id session_abc

# Unassign card
agent-board card update card_123 --agent-session-id null
```

## Tags

Organize cards with tags:

```bash
# Add tags
agent-board card update card_123 --add-tag urgent --add-tag backend

# Remove tags
agent-board card update card_123 --remove-tag urgent
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 4 | Not found (card, board, etc.) |
| 5 | Permission denied |
| 6 | Session conflict |

## Best Practices for Agents

1. **Always set AGENT_BOARD_SESSION_ID** before starting work
2. **Create checklists** for multi-step tasks to track progress
3. **Add comments** when starting, making progress, or completing work
4. **Use descriptive card names** that capture the task intent
5. **Mark cards done and unassign** when completing work
6. **Check `agent-board mine`** at session start to see pending work
7. **Use `--format json`** when you need to parse output programmatically

## Data Location

All data persists in a local JSON file:
- Default: `~/.agent-board/data.json`
- Override: Set `AGENT_BOARD_DB_PATH` environment variable

The database is created automatically on first use.
