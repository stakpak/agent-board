use crate::cli::Cli;
use crate::models::*;
use crate::TaskboardError;
use chrono::Utc;
use std::path::PathBuf;
use uuid::Uuid;

pub struct Database {
    data: TaskboardData,
    path: PathBuf,
}

impl Database {
    pub fn load(_cli: &Cli) -> Result<Self, TaskboardError> {
        let path = Self::get_db_path()?;
        
        let data = if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            serde_json::from_str(&content)?
        } else {
            TaskboardData::default()
        };

        Ok(Self { data, path })
    }

    pub fn save(&self) -> Result<(), TaskboardError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.data)?;
        std::fs::write(&self.path, content)?;
        Ok(())
    }

    fn get_db_path() -> Result<PathBuf, TaskboardError> {
        // Check for custom path in env
        if let Ok(custom_path) = std::env::var("TASKBOARD_DB_PATH") {
            return Ok(PathBuf::from(custom_path));
        }

        // Default to ~/.taskboard/data.json
        let home = dirs::home_dir()
            .ok_or_else(|| TaskboardError::General("Could not determine home directory".into()))?;
        Ok(home.join(".taskboard").join("data.json"))
    }

    fn generate_id(prefix: &str) -> String {
        format!("{}_{}", prefix, &Uuid::new_v4().to_string().replace("-", "")[..12])
    }

    // Board operations
    pub fn list_boards(&self) -> &[Board] {
        &self.data.boards
    }

    pub fn get_board(&self, board_id: &str) -> Result<&Board, TaskboardError> {
        self.data.boards
            .iter()
            .find(|b| b.id == board_id)
            .ok_or_else(|| TaskboardError::NotFound(format!("Board not found: {}", board_id)))
    }

    pub fn create_board(&mut self, name: String, description: Option<String>) -> Result<&Board, TaskboardError> {
        let now = Utc::now();
        let board = Board {
            id: Self::generate_id("board"),
            name,
            description,
            created_at: now,
            updated_at: now,
        };
        self.data.boards.push(board);
        Ok(self.data.boards.last().unwrap())
    }

    pub fn get_board_summary(&self, board_id: &str) -> Result<BoardSummary, TaskboardError> {
        // Verify board exists
        self.get_board(board_id)?;

        let cards: Vec<_> = self.data.cards.iter().filter(|c| c.board_id == board_id).collect();
        
        Ok(BoardSummary {
            todo_count: cards.iter().filter(|c| c.status == Status::Todo).count(),
            in_progress_count: cards.iter().filter(|c| c.status == Status::InProgress).count(),
            done_count: cards.iter().filter(|c| c.status == Status::Done).count(),
            total_cards: cards.len(),
        })
    }

    // Card operations
    pub fn get_card(&self, card_id: &str) -> Result<&Card, TaskboardError> {
        self.data.cards
            .iter()
            .find(|c| c.id == card_id)
            .ok_or_else(|| TaskboardError::NotFound(format!("Card not found: {}", card_id)))
    }

    pub fn list_cards(&self, board_id: &str, status: Option<Status>, assigned_to: Option<&str>) -> Result<Vec<&Card>, TaskboardError> {
        // Verify board exists
        self.get_board(board_id)?;

        Ok(self.data.cards
            .iter()
            .filter(|c| c.board_id == board_id)
            .filter(|c| status.as_ref().is_none_or(|s| &c.status == s))
            .filter(|c| assigned_to.is_none_or(|a| c.assigned_to.as_deref() == Some(a)))
            .collect())
    }

    pub fn get_cards_by_assignee(&self, session_id: &str, board_id: Option<&str>, status: Option<Status>) -> Result<Vec<&Card>, TaskboardError> {
        Ok(self.data.cards
            .iter()
            .filter(|c| c.assigned_to.as_deref() == Some(session_id))
            .filter(|c| board_id.is_none_or(|b| c.board_id == b))
            .filter(|c| status.as_ref().is_none_or(|s| &c.status == s))
            .collect())
    }

    pub fn create_card(&mut self, board_id: &str, name: String, description: Option<String>, status: Status) -> Result<&Card, TaskboardError> {
        // Verify board exists
        self.get_board(board_id)?;

        let now = Utc::now();
        let card = Card {
            id: Self::generate_id("card"),
            board_id: board_id.to_string(),
            name,
            description,
            status,
            assigned_to: None,
            tags: Vec::new(),
            checklists: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        self.data.cards.push(card);
        Ok(self.data.cards.last().unwrap())
    }

    pub fn update_card(
        &mut self,
        card_id: &str,
        update: crate::models::CardUpdate,
    ) -> Result<(), TaskboardError> {
        let card = self.data.cards
            .iter_mut()
            .find(|c| c.id == card_id)
            .ok_or_else(|| TaskboardError::NotFound(format!("Card not found: {}", card_id)))?;

        if let Some(n) = update.name {
            card.name = n;
        }
        if let Some(d) = update.description {
            card.description = Some(d);
        }
        if let Some(s) = update.status {
            card.status = s;
        }
        if let Some(sid) = update.session_id {
            card.assigned_to = sid;
        }
        for tag in update.add_tags {
            if !card.tags.contains(&tag) {
                card.tags.push(tag);
            }
        }
        for tag in update.remove_tags {
            card.tags.retain(|t| t != &tag);
        }
        card.updated_at = Utc::now();

        Ok(())
    }

    // Checklist operations
    pub fn add_checklist(&mut self, card_id: &str, name: String, items: Vec<String>) -> Result<&Checklist, TaskboardError> {
        let card = self.data.cards
            .iter_mut()
            .find(|c| c.id == card_id)
            .ok_or_else(|| TaskboardError::NotFound(format!("Card not found: {}", card_id)))?;

        let checklist = Checklist {
            id: Self::generate_id("checklist"),
            name,
            items: items.into_iter().map(|text| ChecklistItem {
                id: Self::generate_id("item"),
                text,
                checked: false,
            }).collect(),
        };
        card.checklists.push(checklist);
        card.updated_at = Utc::now();

        Ok(card.checklists.last().unwrap())
    }

    pub fn check_item(&mut self, item_id: &str, checked: bool) -> Result<(), TaskboardError> {
        for card in &mut self.data.cards {
            for checklist in &mut card.checklists {
                if let Some(item) = checklist.items.iter_mut().find(|i| i.id == item_id) {
                    item.checked = checked;
                    card.updated_at = Utc::now();
                    return Ok(());
                }
            }
        }
        Err(TaskboardError::NotFound(format!("Checklist item not found: {}", item_id)))
    }

    // Comment operations
    pub fn add_comment(&mut self, card_id: &str, text: String, author: Option<String>) -> Result<&Comment, TaskboardError> {
        // Verify card exists
        self.get_card(card_id)?;

        let comment = Comment {
            id: Self::generate_id("comment"),
            card_id: card_id.to_string(),
            author,
            text,
            created_at: Utc::now(),
        };
        self.data.comments.push(comment);
        Ok(self.data.comments.last().unwrap())
    }

    pub fn list_comments(&self, card_id: &str) -> Result<Vec<&Comment>, TaskboardError> {
        // Verify card exists
        self.get_card(card_id)?;

        Ok(self.data.comments
            .iter()
            .filter(|c| c.card_id == card_id)
            .collect())
    }
}
