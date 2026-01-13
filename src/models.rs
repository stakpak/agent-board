use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Todo,
    InProgress,
    PendingReview,
    Done,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Todo => write!(f, "todo"),
            Status::InProgress => write!(f, "in_progress"),
            Status::PendingReview => write!(f, "pending_review"),
            Status::Done => write!(f, "done"),
        }
    }
}

/// Parameters for updating a card
#[derive(Debug, Default)]
pub struct CardUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<Status>,
    pub session_id: Option<Option<String>>, // None = no change, Some(None) = unassign, Some(Some(x)) = assign
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, clap::ValueEnum, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Json,
    #[default]
    Table,
    Simple,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub board_id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: Status,
    pub assigned_to: Option<String>,
    pub tags: Vec<String>,
    pub checklists: Vec<Checklist>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checklist {
    pub id: String,
    pub name: String,
    pub items: Vec<ChecklistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: String,
    pub text: String,
    pub checked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub card_id: String,
    pub author: Option<String>,
    pub text: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BoardSummary {
    pub todo_count: usize,
    pub in_progress_count: usize,
    pub pending_review_count: usize,
    pub done_count: usize,
    pub total_cards: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentBoardData {
    pub boards: Vec<Board>,
    pub cards: Vec<Card>,
    pub comments: Vec<Comment>,
}
