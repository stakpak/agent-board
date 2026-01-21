# Agent-Board

[![Paks](https://img.shields.io/badge/paks-stakpak%2Fagent--board--cli-blue)](https://paks.stakpak.dev/stakpak/agent-board-cli)

A simple CLI task manager built in Rust. Works for humans and AI agents alike.

## Agent Skill (Pak)

This tool is available as a [Stakpak Pak](https://paks.stakpak.dev/stakpak/agent-board-cli) for AI coding agents. The pak provides detailed usage instructions, workflow patterns, and best practices for agents to effectively use agent-board for task management.

```bash
# Search for the pak
paks search agent-board-cli

# View pak details
paks info stakpak/agent-board-cli
```

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

### Homebrew (macOS/Linux)

```bash
brew tap stakpak/stakpak
brew install agent-board
```

### Direct Download

Download the latest release for your platform from [GitHub Releases](https://github.com/stakpak/agent-board/releases):

```bash
# macOS (Apple Silicon)
curl -L https://github.com/stakpak/agent-board/releases/latest/download/agent-board-darwin-aarch64.tar.gz | tar xz
sudo mv agent-board /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/stakpak/agent-board/releases/latest/download/agent-board-darwin-x86_64.tar.gz | tar xz
sudo mv agent-board /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/stakpak/agent-board/releases/latest/download/agent-board-linux-x86_64.tar.gz | tar xz
sudo mv agent-board /usr/local/bin/

# Linux (ARM64)
curl -L https://github.com/stakpak/agent-board/releases/latest/download/agent-board-linux-aarch64.tar.gz | tar xz
sudo mv agent-board /usr/local/bin/
```

For Windows, download `agent-board-windows-x86_64.zip` from the releases page and add to your PATH.

### Build from Source

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
| `AGENT_BOARD_AGENT_ID` | Your agent ID for `mine`, `whoami`, and `--assign-to-me` |
| `AGENT_BOARD_DB_PATH` | Override default database location |

## Data Location

- Default: `~/.agent-board/data.db`

## Commands

```bash
# Get any entity by ID (auto-detects type)
agent-board get <board_id|card_id|agent_id>

# List commands
agent-board list boards
agent-board list cards <board_id> [--status todo|in-progress|pending-review|done] [--tag TAG]
agent-board list agents
agent-board list comments <card_id>
agent-board list checklists <card_id>

# Create commands
agent-board create board "Name" [--description "..."]
agent-board create card <board_id> "Name" [--description "..."]
agent-board create agent [name] [--command stakpak] [--description "..."]
agent-board create checklist <card_id> --item "Task 1" --item "Task 2"
agent-board create comment <card_id> "text"

# Update commands
agent-board update board <board_id> [--name "..."] [--description "..."]
agent-board update card <card_id> [--status in-progress] [--assign-to-me] [--add-tag TAG]
agent-board update agent <agent_id> [--name "..."] [--workdir .]
agent-board update checklist-item <item_id> --check|--uncheck

# Delete commands (soft delete for boards/cards/agents)
agent-board delete board <board_id>
agent-board delete card <card_id>
agent-board delete agent <agent_id>
agent-board delete checklist <checklist_id>
agent-board delete comment <comment_id>
agent-board delete checklist-item <item_id>

# Agent identity
agent-board whoami                        # Show current agent
agent-board mine [--status STATUS]        # Get your assigned cards
```

## Human Review Workflow

Agents may request human input in two distinct situations:

### Two Types of Requests

| Type | What It Means | How to Find |
|------|---------------|-------------|
| **Blocked** | Agent cannot continue without your input (approval, decision, clarification) | `card list <board> --tag needs-human` |
| **Review requested** | Agent finished work and wants verification before closing (optional) | `card list <board> --status pending-review` |

**Note:** Agents can complete work without review - `pending-review` is optional. The `blocked` + `needs-human` tags indicate the agent is stuck and waiting.

### Responding to Requests

```bash
# Find cards needing your attention
agent-board card list board_123 --tag needs-human      # Blocked agents
agent-board card list board_123 --status pending-review # Optional reviews

# Review a card
agent-board card get card_123

# Unblock a stuck agent (they will continue working)
agent-board card update card_123 --remove-tag blocked --remove-tag needs-human
agent-board comment add card_123 "APPROVED: Proceed with migration"

# Approve completed work in review
agent-board card update card_123 --status done
agent-board comment add card_123 "Approved - looks good"

# Request changes on reviewed work
agent-board card update card_123 --status in-progress
agent-board comment add card_123 "Needs changes: reduce instance size"
```

### Common Tags

| Tag | Meaning |
|-----|---------|
| `blocked` | Cannot proceed - waiting on something |
| `needs-human` | The blocker requires human attention (vs another agent or external event) |
| `expedite` | Urgent, needs fast turnaround |
| `security-review` | Needs security sign-off |
| `cost-approval` | Needs cost/budget approval |

## License

MIT
