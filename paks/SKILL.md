---
name: agent-board-cli
description: |
  Command-line task management tool for AI coding agents and humans. Provides local SQLite-based task boards for tracking work items, checklists, and comments without requiring an external API.
  
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
  version: "0.1.2"
---

# Agent-Board CLI

## Quick Reference

```bash
# Board operations
agent-board board create "Project Name" --description "Description"
agent-board board list [--include-deleted]
agent-board board get <board_id>
agent-board board delete <board_id>

# Card operations
agent-board card create <board_id> "Task name" --description "Details" --status todo
agent-board card list <board_id> [--status todo|in-progress|pending-review|done] [--include-deleted]
agent-board card list <board_id> --tag blocked --tag needs-human  # Filter by tags (AND logic)
agent-board card get <card_id>
agent-board card update <card_id> --status in-progress --assign-to-me
agent-board card update <card_id> --status pending-review
agent-board card update <card_id> --status done
agent-board card delete <card_id>

# Checklist operations
agent-board checklist add <card_id> --name "Subtasks" --item "Step 1" --item "Step 2"
agent-board checklist check <item_id>
agent-board checklist check <item_id> --uncheck

# Comment operations
agent-board comment add <card_id> "Progress update or notes"

# Get my assigned cards
agent-board mine [--status todo|in-progress|pending-review|done]
```

## Installation

```bash
# Homebrew (recommended)
brew tap stakpak/stakpak && brew install agent-board

# Direct download (replace PLATFORM: darwin-aarch64, darwin-x86_64, linux-x86_64, linux-aarch64)
curl -L https://github.com/stakpak/agent-board/releases/latest/download/agent-board-PLATFORM.tar.gz | tar xz
sudo mv agent-board /usr/local/bin/

# Build from source
cargo build --release  # Binary at ./target/release/agent-board
```

## Environment Setup

Set your session ID to enable card assignment:

```bash
export AGENT_BOARD_SESSION_ID=agent_session_001
```

Data is stored in `~/.agent-board/data.db` (SQLite) by default. Override with:

```bash
export AGENT_BOARD_DB_PATH=/custom/path/data.db
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

# 4. Claim the card and start working
agent-board card update card_xyz789 --status in-progress --assign-to-me

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

# Mark done (assignment preserved for history)
agent-board card update card_xyz789 --status done

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
| Pending Review | `pending-review` | Work complete, awaiting review |
| Done | `done` | Completed and reviewed |

**Note:** Use hyphens on the command line (e.g., `in-progress`, `pending-review`), not underscores.

## Card Assignment

Claim cards with `--assign-to-me`:
```bash
# Claim a card when starting work
agent-board card update card_123 --status in-progress --assign-to-me
```

Assignment is preserved after completion for history/accountability.

Explicit assignment control:
```bash
# Assign to specific session
agent-board card update card_123 --agent-session-id session_abc

# Unassign card (only if needed)
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

## Human Review (Optional)

Cards can flow `in-progress` → `done` directly. Use these only when human input is needed:

| Scenario | Mechanism | When |
|----------|-----------|------|
| **Blocked** | `--add-tag blocked --add-tag needs-human` | Cannot continue without human input |
| **Review** | `--status pending-review` | Work done, want human verification |

```bash
# Blocked: need human help to proceed
agent-board card update card_123 --add-tag blocked --add-tag needs-human
agent-board comment add card_123 "BLOCKED: Need cost approval before provisioning"

# Review: work complete, want verification
agent-board card update card_123 --status pending-review
agent-board comment add card_123 "Ready for review: verify terraform plan"
```

**Common tags:** `blocked`, `needs-human`, `expedite`, `security-review`, `cost-approval`

## Best Practices for Agents

1. **Always set AGENT_BOARD_SESSION_ID** before starting work
2. **Use `--assign-to-me`** when claiming a card to start work
3. **Think Kanban** - Cards represent discrete, deliverable work items that flow through the board
4. **Add comments** when starting, making progress, or completing work
5. **Use descriptive card names** that capture the task intent
6. **Keep assignment on completion** - Don't unassign when marking done (preserves history)
7. **Check `agent-board mine`** at session start to see pending work
8. **Use `--format json`** when you need to parse output programmatically
9. **Use `blocked` + `needs-human` tags** when you cannot continue without human input
10. **Use `pending-review` status** when work is done but you want human verification (optional)

### Cards vs Checklists

| Use Cards | Use Checklists |
|-----------|----------------|
| Parallelizable work | Sequential steps in one deliverable |
| Different dependencies | Shared dependencies |
| Independent status tracking | Linear progress |

**Simple task:** "Add Docker support" → One card with checklist (Dockerfile, compose, test, docs)

**Complex task:** "Migrate to microservices" → Multiple cards (auth service, payment service, API gateway, testing)

## Data Location

All data persists in a local SQLite database:
- Default: `~/.agent-board/data.db`
- Override: Set `AGENT_BOARD_DB_PATH` environment variable

The database is created automatically on first use.
