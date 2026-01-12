---
name: taskboard-cli
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

# Taskboard CLI for AI Agents

## Overview

`taskboard` is a command-line task management tool designed for AI coding agents and humans. It provides a local JSON-based task board for tracking work items, checklists, and comments without requiring an external API.

## Quick Reference

```bash
# Board operations
taskboard board create "Project Name" --description "Description"
taskboard board list
taskboard board get <board_id>

# Card operations
taskboard card create <board_id> "Task name" --description "Details" --status todo
taskboard card list <board_id> [--status todo|in-progress|done]
taskboard card get <card_id>
taskboard card update <card_id> --status in-progress
taskboard card update <card_id> --status done --agent-session-id null

# Checklist operations
taskboard checklist add <card_id> --name "Subtasks" --item "Step 1" --item "Step 2"
taskboard checklist check <item_id>
taskboard checklist check <item_id> --uncheck

# Comment operations
taskboard comment add <card_id> "Progress update or notes"
taskboard comment list <card_id>

# Get my assigned cards
taskboard mine [--status todo|in-progress|done]
```

## Environment Setup

Set your session ID to enable card assignment:

```bash
export TASKBOARD_SESSION_ID=agent_session_001
```

Data is stored in `~/.taskboard/data.json` by default. Override with:

```bash
export TASKBOARD_DB_PATH=/custom/path/data.json
```

## Workflow Patterns

### Starting a New Task

```bash
# 1. Find or create a board
taskboard board list
taskboard board create "Feature Development" --description "Q1 features"

# 2. Create a card for the task
taskboard card create board_abc123 "Implement user authentication" \
  --description "Add OAuth2 login flow with Google and GitHub providers"

# 3. Break down into subtasks
taskboard checklist add card_xyz789 \
  --name "Implementation Steps" \
  --item "Set up OAuth client credentials" \
  --item "Create auth endpoints" \
  --item "Add session management" \
  --item "Write integration tests" \
  --item "Update documentation"

# 4. Start working (auto-assigns to your session)
taskboard card update card_xyz789 --status in-progress

# 5. Document that you're starting
taskboard comment add card_xyz789 "Beginning OAuth implementation"
```

### Tracking Progress

```bash
# Check off completed subtasks
taskboard checklist check item_001
taskboard checklist check item_002

# Add progress notes
taskboard comment add card_xyz789 "OAuth endpoints complete. Starting session management."

# View current state
taskboard card get card_xyz789
```

### Completing Work

```bash
# Check remaining items
taskboard checklist check item_003
taskboard checklist check item_004
taskboard checklist check item_005

# Mark done and unassign
taskboard card update card_xyz789 --status done --agent-session-id null

# Add completion summary
taskboard comment add card_xyz789 "Implementation complete. All tests passing."
```

### Reviewing Work

```bash
# See all your assigned cards
taskboard mine

# Filter by status
taskboard mine --status in-progress

# Get board overview
taskboard board get board_abc123

# List all cards on a board
taskboard card list board_abc123 --status todo
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
CARD_ID=$(taskboard card create board_123 "New task" --format simple)

# Get full JSON for parsing
taskboard card get card_xyz789 --format json | jq '.status'
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
- You update status to `in-progress` (uses `TASKBOARD_SESSION_ID`)

Explicit assignment control:
```bash
# Assign to specific session
taskboard card update card_123 --agent-session-id session_abc

# Unassign card
taskboard card update card_123 --agent-session-id null
```

## Tags

Organize cards with tags:

```bash
# Add tags
taskboard card update card_123 --add-tag urgent --add-tag backend

# Remove tags
taskboard card update card_123 --remove-tag urgent
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

1. **Always set TASKBOARD_SESSION_ID** before starting work
2. **Create checklists** for multi-step tasks to track progress
3. **Add comments** when starting, making progress, or completing work
4. **Use descriptive card names** that capture the task intent
5. **Mark cards done and unassign** when completing work
6. **Check `taskboard mine`** at session start to see pending work
7. **Use `--format json`** when you need to parse output programmatically

## Data Location

All data persists in a local JSON file:
- Default: `~/.taskboard/data.json`
- Override: Set `TASKBOARD_DB_PATH` environment variable

The database is created automatically on first use.
