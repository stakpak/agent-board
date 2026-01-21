use crate::AgentBoardError;
use crate::models::{OutputFormat, Status};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "agent-board")]
#[command(about = "CLI for managing task boards", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Override API key (unused in local mode)
    #[arg(long, global = true)]
    pub api_key: Option<String>,

    /// Override API endpoint (unused in local mode)
    #[arg(long, global = true)]
    pub api_url: Option<String>,

    /// Output format
    #[arg(long, global = true, default_value = "table")]
    pub format: OutputFormat,

    /// Suppress non-essential output
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Show detailed debug output
    #[arg(long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub fn get_agent_id(&self) -> Result<String, AgentBoardError> {
        std::env::var("AGENT_BOARD_AGENT_ID").map_err(|_| {
            AgentBoardError::InvalidArgs("AGENT_BOARD_AGENT_ID environment variable not set".into())
        })
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show version information
    Version,

    /// Get any entity by ID (auto-detects type from prefix: agent_, board_, card_)
    Get {
        /// Entity ID (e.g., board_xxx, card_xxx, agent_xxx)
        id: String,

        /// Output format
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// List entities (boards, cards, agents)
    List {
        #[command(subcommand)]
        command: ListCommands,
    },

    /// Create entities (boards, cards, agents, checklists, comments)
    Create {
        #[command(subcommand)]
        command: CreateCommands,
    },

    /// Update entities (boards, cards, agents, checklist items)
    Update {
        #[command(subcommand)]
        command: UpdateCommands,
    },

    /// Delete entities (boards, cards, agents)
    Delete {
        #[command(subcommand)]
        command: DeleteCommands,
    },

    /// Get all cards assigned to current agent
    Mine {
        /// Filter by board
        #[arg(long)]
        board: Option<String>,

        /// Filter by status
        #[arg(long)]
        status: Option<Status>,

        /// Output format
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// Show current agent identity (from AGENT_BOARD_AGENT_ID)
    Whoami,
}

// ============================================================================
// LIST subcommands
// ============================================================================

#[derive(Subcommand, Debug)]
pub enum ListCommands {
    /// List all boards
    Boards {
        /// Include soft-deleted boards
        #[arg(long)]
        include_deleted: bool,

        /// Output format
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// List cards on a board
    Cards {
        /// Board ID
        board_id: String,

        /// Filter by status
        #[arg(long)]
        status: Option<Status>,

        /// Filter by assignee
        #[arg(long)]
        assigned_to: Option<String>,

        /// Filter by tag (repeatable, cards must have ALL specified tags)
        #[arg(long)]
        tag: Vec<String>,

        /// Include soft-deleted cards
        #[arg(long)]
        include_deleted: bool,

        /// Output format
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// List all registered agents
    Agents {
        /// Include deactivated agents
        #[arg(long)]
        include_inactive: bool,

        /// Output format
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// List comments on a card
    Comments {
        /// Card ID
        card_id: String,

        /// Output format
        #[arg(long)]
        format: Option<OutputFormat>,
    },
}

// ============================================================================
// CREATE subcommands
// ============================================================================

#[derive(Subcommand, Debug)]
pub enum CreateCommands {
    /// Create a new board
    Board {
        /// Board name
        name: String,

        /// Board description
        #[arg(long)]
        description: Option<String>,
    },

    /// Create a new card on a board
    Card {
        /// Board ID
        board_id: String,

        /// Card name
        name: String,

        /// Card description
        #[arg(long)]
        description: Option<String>,

        /// Initial status
        #[arg(long, default_value = "todo")]
        status: Status,
    },

    /// Register a new agent identity
    Agent {
        /// Command to invoke this agent (e.g., stakpak, claude, aider)
        #[arg(long)]
        command: String,

        /// Agent name (auto-generated if not provided)
        #[arg(long)]
        name: Option<String>,

        /// Agent description
        #[arg(long)]
        description: Option<String>,
    },

    /// Add a checklist to a card
    Checklist {
        /// Card ID
        card_id: String,

        /// Name for the checklist
        #[arg(long, default_value = "Tasks")]
        name: String,

        /// Checklist item text (repeatable)
        #[arg(long, required = true)]
        item: Vec<String>,
    },

    /// Add a comment to a card
    Comment {
        /// Card ID
        card_id: String,

        /// Comment text
        text: Option<String>,

        /// Read comment text from file
        #[arg(long)]
        file: Option<String>,
    },
}

// ============================================================================
// UPDATE subcommands
// ============================================================================

#[derive(Subcommand, Debug)]
pub enum UpdateCommands {
    /// Update card fields
    Card {
        /// Card ID
        card_id: String,

        /// Update card name
        #[arg(long)]
        name: Option<String>,

        /// Update description
        #[arg(long)]
        description: Option<String>,

        /// Update status
        #[arg(long)]
        status: Option<Status>,

        /// Assign card to agent ID (use 'null' to unassign)
        #[arg(long, conflicts_with = "assign_to_me")]
        assign: Option<String>,

        /// Assign card to current agent (uses AGENT_BOARD_AGENT_ID)
        #[arg(long, conflicts_with = "assign")]
        assign_to_me: bool,

        /// Add tag (repeatable)
        #[arg(long)]
        add_tag: Vec<String>,

        /// Remove tag (repeatable)
        #[arg(long)]
        remove_tag: Vec<String>,
    },

    /// Update agent details
    Agent {
        /// Agent ID
        agent_id: String,

        /// Update agent name
        #[arg(long)]
        name: Option<String>,

        /// Update command
        #[arg(long)]
        command: Option<String>,

        /// Update description
        #[arg(long)]
        description: Option<String>,

        /// Update working directory (use "." for current directory)
        #[arg(long)]
        workdir: Option<String>,
    },

    /// Check or uncheck a checklist item
    ChecklistItem {
        /// Item ID
        item_id: String,

        /// Check the item (mark as complete)
        #[arg(long, conflicts_with = "uncheck")]
        check: bool,

        /// Uncheck the item (mark as incomplete)
        #[arg(long, conflicts_with = "check")]
        uncheck: bool,
    },
}

// ============================================================================
// DELETE subcommands
// ============================================================================

#[derive(Subcommand, Debug)]
pub enum DeleteCommands {
    /// Delete a board (soft delete)
    Board {
        /// Board ID
        board_id: String,
    },

    /// Delete a card (soft delete)
    Card {
        /// Card ID
        card_id: String,
    },

    /// Unregister an agent (soft delete)
    Agent {
        /// Agent ID
        agent_id: String,
    },
}
