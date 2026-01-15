use clap::{Parser, Subcommand};
use crate::models::{OutputFormat, Status};
use crate::AgentBoardError;

#[derive(Parser, Debug)]
#[command(name = "taskboard")]
#[command(about = "CLI for managing task boards", long_about = None)]
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
    pub fn get_session_id(&self) -> Result<String, AgentBoardError> {
        std::env::var("AGENT_BOARD_SESSION_ID")
            .map_err(|_| AgentBoardError::InvalidArgs("AGENT_BOARD_SESSION_ID environment variable not set".into()))
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get all cards assigned to current user
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

    /// Card operations
    Card {
        #[command(subcommand)]
        command: CardCommands,
    },

    /// Checklist operations
    Checklist {
        #[command(subcommand)]
        command: ChecklistCommands,
    },

    /// Comment operations
    Comment {
        #[command(subcommand)]
        command: CommentCommands,
    },

    /// Board operations
    Board {
        #[command(subcommand)]
        command: BoardCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum CardCommands {
    /// Retrieve full card details
    Get {
        /// Card ID
        card_id: String,

        /// Output format
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// Query cards on a board
    List {
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

    /// Create a new card on a board
    Create {
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

    /// Update card fields
    Update {
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

        /// Set session ID to assign card (use 'null' to unassign)
        #[arg(long, conflicts_with = "assign_to_me")]
        agent_session_id: Option<String>,

        /// Assign card to current session (uses AGENT_BOARD_SESSION_ID)
        #[arg(long, conflicts_with = "agent_session_id")]
        assign_to_me: bool,

        /// Add tag (repeatable)
        #[arg(long)]
        add_tag: Vec<String>,

        /// Remove tag (repeatable)
        #[arg(long)]
        remove_tag: Vec<String>,
    },

    /// Delete a card (soft delete)
    Delete {
        /// Card ID
        card_id: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ChecklistCommands {
    /// Add checklist items to a card
    Add {
        /// Card ID
        card_id: String,

        /// Name for the checklist
        #[arg(long, default_value = "Tasks")]
        name: String,

        /// Checklist item text (repeatable)
        #[arg(long, required = true)]
        item: Vec<String>,
    },

    /// Check or uncheck a checklist item
    Check {
        /// Item ID
        item_id: String,

        /// Uncheck the item instead of checking it
        #[arg(long)]
        uncheck: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum CommentCommands {
    /// Add a comment to a card
    Add {
        /// Card ID
        card_id: String,

        /// Comment text
        text: Option<String>,

        /// Read comment text from file
        #[arg(long)]
        file: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum BoardCommands {
    /// Get board overview and summary
    Get {
        /// Board ID
        board_id: String,

        /// Output format
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// List all accessible boards
    List {
        /// Include soft-deleted boards
        #[arg(long)]
        include_deleted: bool,

        /// Output format
        #[arg(long)]
        format: Option<OutputFormat>,
    },

    /// Create a new board
    Create {
        /// Board name
        name: String,

        /// Board description
        #[arg(long)]
        description: Option<String>,
    },

    /// Delete a board (soft delete)
    Delete {
        /// Board ID
        board_id: String,
    },
}
