use crate::cli::Cli;
use crate::models::*;
use crate::AgentBoardError;
use chrono::{DateTime, Utc};
use libsql::{Builder, Connection};
use std::path::PathBuf;
use uuid::Uuid;

const SCHEMA: &str = include_str!("schema.sql");

pub struct Database {
    conn: Connection,
}

impl Database {
    pub async fn load(_cli: &Cli) -> Result<Self, AgentBoardError> {
        let path = Self::get_db_path()?;
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db = Builder::new_local(&path)
            .build()
            .await
            .map_err(|e| AgentBoardError::General(format!("Failed to open database: {}", e)))?;
        
        let conn = db.connect()
            .map_err(|e| AgentBoardError::General(format!("Failed to connect: {}", e)))?;

        // Initialize schema
        conn.execute_batch(SCHEMA)
            .await
            .map_err(|e| AgentBoardError::General(format!("Failed to initialize schema: {}", e)))?;

        Ok(Self { conn })
    }

    pub async fn save(&self) -> Result<(), AgentBoardError> {
        // SQLite auto-commits, nothing to do here
        Ok(())
    }

    fn get_db_path() -> Result<PathBuf, AgentBoardError> {
        // Check for custom path in env
        if let Ok(custom_path) = std::env::var("AGENT_BOARD_DB_PATH") {
            return Ok(PathBuf::from(custom_path));
        }

        // Default to ~/.agent-board/data.db
        let home = dirs::home_dir()
            .ok_or_else(|| AgentBoardError::General("Could not determine home directory".into()))?;
        Ok(home.join(".agent-board").join("data.db"))
    }

    fn generate_id(prefix: &str) -> String {
        format!("{}_{}", prefix, &Uuid::new_v4().to_string().replace("-", "")[..12])
    }

    fn parse_datetime(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now())
    }

    fn status_from_str(s: &str) -> Status {
        match s {
            "in_progress" => Status::InProgress,
            "pending_review" => Status::PendingReview,
            "done" => Status::Done,
            _ => Status::Todo,
        }
    }

    fn generate_agent_name() -> String {
        let mut generator = names::Generator::default();
        generator.next().unwrap_or_else(|| "unnamed-agent".to_string())
    }

    // Agent operations
    pub async fn register_agent(
        &self,
        name: Option<String>,
        command: String,
        working_directory: String,
        description: Option<String>,
    ) -> Result<Agent, AgentBoardError> {
        let agent_name = name.unwrap_or_else(Self::generate_agent_name);
        let id = Self::generate_id("agent");
        let now = Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO agents (id, name, command, working_directory, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                libsql::params![id.as_str(), agent_name.as_str(), command.as_str(), working_directory.as_str(), description.clone().unwrap_or_default().as_str(), now.as_str(), now.as_str()],
            )
            .await
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    AgentBoardError::InvalidArgs(format!("Agent name '{}' already exists", agent_name))
                } else {
                    AgentBoardError::General(format!("Insert failed: {}", e))
                }
            })?;

        self.get_agent(&id).await
    }

    pub async fn get_agent(&self, agent_id: &str) -> Result<Agent, AgentBoardError> {
        let mut rows = self.conn
            .query(
                "SELECT id, name, command, working_directory, description, created_at, updated_at, deactivated_at FROM agents WHERE id = ?1 AND deactivated_at IS NULL",
                [agent_id],
            )
            .await
            .map_err(|e| AgentBoardError::General(format!("Query failed: {}", e)))?;

        if let Some(row) = rows.next().await.map_err(|e| AgentBoardError::General(format!("Row fetch failed: {}", e)))? {
            Ok(Agent {
                id: row.get::<String>(0).unwrap_or_default(),
                name: row.get::<String>(1).unwrap_or_default(),
                command: row.get::<String>(2).unwrap_or_default(),
                working_directory: row.get::<String>(3).unwrap_or_default(),
                description: row.get::<Option<String>>(4).ok().flatten(),
                created_at: Self::parse_datetime(&row.get::<String>(5).unwrap_or_default()),
                updated_at: Self::parse_datetime(&row.get::<String>(6).unwrap_or_default()),
                deactivated_at: row.get::<Option<String>>(7).ok().flatten().map(|s| Self::parse_datetime(&s)),
            })
        } else {
            Err(AgentBoardError::NotFound(format!("Agent not found: {}", agent_id)))
        }
    }

    pub async fn list_agents(&self, include_inactive: bool) -> Result<Vec<Agent>, AgentBoardError> {
        let query = if include_inactive {
            "SELECT id, name, command, working_directory, description, created_at, updated_at, deactivated_at FROM agents ORDER BY created_at DESC"
        } else {
            "SELECT id, name, command, working_directory, description, created_at, updated_at, deactivated_at FROM agents WHERE deactivated_at IS NULL ORDER BY created_at DESC"
        };
        let mut rows = self.conn
            .query(query, ())
            .await
            .map_err(|e| AgentBoardError::General(format!("Query failed: {}", e)))?;

        let mut agents = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AgentBoardError::General(format!("Row fetch failed: {}", e)))? {
            agents.push(Agent {
                id: row.get::<String>(0).unwrap_or_default(),
                name: row.get::<String>(1).unwrap_or_default(),
                command: row.get::<String>(2).unwrap_or_default(),
                working_directory: row.get::<String>(3).unwrap_or_default(),
                description: row.get::<Option<String>>(4).ok().flatten(),
                created_at: Self::parse_datetime(&row.get::<String>(5).unwrap_or_default()),
                updated_at: Self::parse_datetime(&row.get::<String>(6).unwrap_or_default()),
                deactivated_at: row.get::<Option<String>>(7).ok().flatten().map(|s| Self::parse_datetime(&s)),
            });
        }
        Ok(agents)
    }

    pub async fn update_agent(&self, agent_id: &str, update: AgentUpdate) -> Result<(), AgentBoardError> {
        // Verify agent exists
        self.get_agent(agent_id).await?;

        let now = Utc::now().to_rfc3339();

        if let Some(n) = update.name {
            self.conn
                .execute("UPDATE agents SET name = ?1, updated_at = ?2 WHERE id = ?3", [&n, &now, agent_id])
                .await
                .map_err(|e| {
                    if e.to_string().contains("UNIQUE constraint failed") {
                        AgentBoardError::InvalidArgs(format!("Agent name '{}' already exists", n))
                    } else {
                        AgentBoardError::General(format!("Update failed: {}", e))
                    }
                })?;
        }
        if let Some(c) = update.command {
            self.conn
                .execute("UPDATE agents SET command = ?1, updated_at = ?2 WHERE id = ?3", [&c, &now, agent_id])
                .await
                .map_err(|e| AgentBoardError::General(format!("Update failed: {}", e)))?;
        }
        if let Some(d) = update.description {
            self.conn
                .execute("UPDATE agents SET description = ?1, updated_at = ?2 WHERE id = ?3", [&d, &now, agent_id])
                .await
                .map_err(|e| AgentBoardError::General(format!("Update failed: {}", e)))?;
        }
        if let Some(w) = update.working_directory {
            self.conn
                .execute("UPDATE agents SET working_directory = ?1, updated_at = ?2 WHERE id = ?3", [&w, &now, agent_id])
                .await
                .map_err(|e| AgentBoardError::General(format!("Update failed: {}", e)))?;
        }
        Ok(())
    }

    pub async fn unregister_agent(&self, agent_id: &str) -> Result<(), AgentBoardError> {
        // Verify agent exists
        self.get_agent(agent_id).await?;

        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE agents SET deactivated_at = ?1, updated_at = ?1 WHERE id = ?2",
                [&now, agent_id],
            )
            .await
            .map_err(|e| AgentBoardError::General(format!("Unregister failed: {}", e)))?;

        Ok(())
    }

    // Board operations
    pub async fn list_boards(&self, include_deleted: bool) -> Result<Vec<Board>, AgentBoardError> {
        let query = if include_deleted {
            "SELECT id, name, description, created_at, updated_at, deleted_at FROM boards ORDER BY created_at DESC"
        } else {
            "SELECT id, name, description, created_at, updated_at, deleted_at FROM boards WHERE deleted_at IS NULL ORDER BY created_at DESC"
        };
        let mut rows = self.conn
            .query(query, ())
            .await
            .map_err(|e| AgentBoardError::General(format!("Query failed: {}", e)))?;

        let mut boards = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AgentBoardError::General(format!("Row fetch failed: {}", e)))? {
            boards.push(Board {
                id: row.get::<String>(0).unwrap_or_default(),
                name: row.get::<String>(1).unwrap_or_default(),
                description: row.get::<Option<String>>(2).ok().flatten(),
                created_at: Self::parse_datetime(&row.get::<String>(3).unwrap_or_default()),
                updated_at: Self::parse_datetime(&row.get::<String>(4).unwrap_or_default()),
                deleted_at: row.get::<Option<String>>(5).ok().flatten().map(|s| Self::parse_datetime(&s)),
            });
        }
        Ok(boards)
    }

    pub async fn get_board(&self, board_id: &str) -> Result<Board, AgentBoardError> {
        let mut rows = self.conn
            .query("SELECT id, name, description, created_at, updated_at, deleted_at FROM boards WHERE id = ?1 AND deleted_at IS NULL", [board_id])
            .await
            .map_err(|e| AgentBoardError::General(format!("Query failed: {}", e)))?;

        if let Some(row) = rows.next().await.map_err(|e| AgentBoardError::General(format!("Row fetch failed: {}", e)))? {
            Ok(Board {
                id: row.get::<String>(0).unwrap_or_default(),
                name: row.get::<String>(1).unwrap_or_default(),
                description: row.get::<Option<String>>(2).ok().flatten(),
                created_at: Self::parse_datetime(&row.get::<String>(3).unwrap_or_default()),
                updated_at: Self::parse_datetime(&row.get::<String>(4).unwrap_or_default()),
                deleted_at: row.get::<Option<String>>(5).ok().flatten().map(|s| Self::parse_datetime(&s)),
            })
        } else {
            Err(AgentBoardError::NotFound(format!("Board not found: {}", board_id)))
        }
    }

    pub async fn delete_board(&self, board_id: &str) -> Result<(), AgentBoardError> {
        // Verify board exists
        self.get_board(board_id).await?;

        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE boards SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
                [&now, board_id],
            )
            .await
            .map_err(|e| AgentBoardError::General(format!("Delete failed: {}", e)))?;

        // Soft delete all cards in this board
        self.conn
            .execute(
                "UPDATE cards SET deleted_at = ?1, updated_at = ?1 WHERE board_id = ?2 AND deleted_at IS NULL",
                [&now, board_id],
            )
            .await
            .map_err(|e| AgentBoardError::General(format!("Delete cards failed: {}", e)))?;

        Ok(())
    }

    pub async fn create_board(&self, name: String, description: Option<String>) -> Result<Board, AgentBoardError> {
        let id = Self::generate_id("board");
        let now = Utc::now().to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO boards (id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                libsql::params![id.as_str(), name.as_str(), description.clone().unwrap_or_default().as_str(), now.as_str(), now.as_str()],
            )
            .await
            .map_err(|e| AgentBoardError::General(format!("Insert failed: {}", e)))?;

        self.get_board(&id).await
    }

    pub async fn get_board_summary(&self, board_id: &str) -> Result<BoardSummary, AgentBoardError> {
        // Verify board exists
        self.get_board(board_id).await?;

        let mut rows = self.conn
            .query(
                "SELECT status, COUNT(*) as cnt FROM cards WHERE board_id = ?1 AND deleted_at IS NULL GROUP BY status",
                [board_id],
            )
            .await
            .map_err(|e| AgentBoardError::General(format!("Query failed: {}", e)))?;

        let mut summary = BoardSummary::default();
        while let Some(row) = rows.next().await.map_err(|e| AgentBoardError::General(format!("Row fetch failed: {}", e)))? {
            let status: String = row.get(0).unwrap_or_default();
            let count: i64 = row.get(1).unwrap_or(0);
            match status.as_str() {
                "todo" => summary.todo_count = count as usize,
                "in_progress" => summary.in_progress_count = count as usize,
                "pending_review" => summary.pending_review_count = count as usize,
                "done" => summary.done_count = count as usize,
                _ => {}
            }
        }
        summary.total_cards = summary.todo_count + summary.in_progress_count + summary.pending_review_count + summary.done_count;
        Ok(summary)
    }

    // Card operations - helper to load full card with tags and checklists
    async fn load_card_full(&self, card_id: &str) -> Result<Card, AgentBoardError> {
        self.load_card_full_with_deleted(card_id, false).await
    }

    async fn load_card_full_with_deleted(&self, card_id: &str, include_deleted: bool) -> Result<Card, AgentBoardError> {
        let query = if include_deleted {
            "SELECT id, board_id, name, description, status, assigned_to, created_at, updated_at, deleted_at FROM cards WHERE id = ?1"
        } else {
            "SELECT id, board_id, name, description, status, assigned_to, created_at, updated_at, deleted_at FROM cards WHERE id = ?1 AND deleted_at IS NULL"
        };
        let mut rows = self.conn
            .query(query, [card_id])
            .await
            .map_err(|e| AgentBoardError::General(format!("Query failed: {}", e)))?;

        let row = rows.next().await
            .map_err(|e| AgentBoardError::General(format!("Row fetch failed: {}", e)))?
            .ok_or_else(|| AgentBoardError::NotFound(format!("Card not found: {}", card_id)))?;

        let id: String = row.get(0).unwrap_or_default();
        let board_id: String = row.get(1).unwrap_or_default();
        let name: String = row.get(2).unwrap_or_default();
        let description: Option<String> = row.get::<Option<String>>(3).ok().flatten();
        let status = Self::status_from_str(&row.get::<String>(4).unwrap_or_default());
        let assigned_to: Option<String> = row.get::<Option<String>>(5).ok().flatten();
        let created_at = Self::parse_datetime(&row.get::<String>(6).unwrap_or_default());
        let updated_at = Self::parse_datetime(&row.get::<String>(7).unwrap_or_default());
        let deleted_at: Option<DateTime<Utc>> = row.get::<Option<String>>(8).ok().flatten().map(|s| Self::parse_datetime(&s));

        // Load tags
        let mut tag_rows = self.conn
            .query("SELECT tag FROM card_tags WHERE card_id = ?1", [id.as_str()])
            .await
            .map_err(|e| AgentBoardError::General(format!("Query failed: {}", e)))?;
        let mut tags = Vec::new();
        while let Some(tag_row) = tag_rows.next().await.map_err(|e| AgentBoardError::General(format!("Row fetch failed: {}", e)))? {
            tags.push(tag_row.get::<String>(0).unwrap_or_default());
        }

        // Load checklists
        let checklists = self.load_checklists_for_card(&id).await?;

        Ok(Card {
            id,
            board_id,
            name,
            description,
            status,
            assigned_to,
            tags,
            checklists,
            created_at,
            updated_at,
            deleted_at,
        })
    }

    async fn load_checklists_for_card(&self, card_id: &str) -> Result<Vec<Checklist>, AgentBoardError> {
        let mut checklist_rows = self.conn
            .query("SELECT id, name FROM checklists WHERE card_id = ?1", [card_id])
            .await
            .map_err(|e| AgentBoardError::General(format!("Query failed: {}", e)))?;

        let mut checklists = Vec::new();
        while let Some(cl_row) = checklist_rows.next().await.map_err(|e| AgentBoardError::General(format!("Row fetch failed: {}", e)))? {
            let cl_id: String = cl_row.get(0).unwrap_or_default();
            let cl_name: String = cl_row.get(1).unwrap_or_default();

            // Load items for this checklist
            let mut item_rows = self.conn
                .query("SELECT id, text, checked FROM checklist_items WHERE checklist_id = ?1", [cl_id.as_str()])
                .await
                .map_err(|e| AgentBoardError::General(format!("Query failed: {}", e)))?;

            let mut items = Vec::new();
            while let Some(item_row) = item_rows.next().await.map_err(|e| AgentBoardError::General(format!("Row fetch failed: {}", e)))? {
                items.push(ChecklistItem {
                    id: item_row.get::<String>(0).unwrap_or_default(),
                    text: item_row.get::<String>(1).unwrap_or_default(),
                    checked: item_row.get::<i64>(2).unwrap_or(0) != 0,
                });
            }

            checklists.push(Checklist {
                id: cl_id,
                name: cl_name,
                items,
            });
        }
        Ok(checklists)
    }

    pub async fn get_card(&self, card_id: &str) -> Result<Card, AgentBoardError> {
        self.load_card_full(card_id).await
    }

    pub async fn list_cards(&self, board_id: &str, status: Option<Status>, assigned_to: Option<&str>, tags: &[String], include_deleted: bool) -> Result<Vec<Card>, AgentBoardError> {
        // Verify board exists (allow deleted boards when include_deleted is true)
        if include_deleted {
            // Check if board exists at all (including deleted)
            let mut rows = self.conn
                .query("SELECT id FROM boards WHERE id = ?1", [board_id])
                .await
                .map_err(|e| AgentBoardError::General(format!("Query failed: {}", e)))?;
            if rows.next().await.map_err(|e| AgentBoardError::General(format!("Row fetch failed: {}", e)))?.is_none() {
                return Err(AgentBoardError::NotFound(format!("Board not found: {}", board_id)));
            }
        } else {
            self.get_board(board_id).await?;
        }

        let deleted_filter = if include_deleted { "" } else { " AND deleted_at IS NULL" };
        
        // Build tag filter using subquery for AND logic (card must have ALL specified tags)
        let tag_filter = if tags.is_empty() {
            String::new()
        } else {
            let tag_conditions: Vec<String> = tags.iter()
                .map(|t| format!("EXISTS (SELECT 1 FROM card_tags WHERE card_id = cards.id AND tag = '{}')", t))
                .collect();
            format!(" AND {}", tag_conditions.join(" AND "))
        };
        
        let query = match (&status, &assigned_to) {
            (Some(s), Some(a)) => {
                format!(
                    "SELECT id FROM cards WHERE board_id = '{}' AND status = '{}' AND assigned_to = '{}'{}{}",
                    board_id, s, a, deleted_filter, tag_filter
                )
            }
            (Some(s), None) => {
                format!(
                    "SELECT id FROM cards WHERE board_id = '{}' AND status = '{}'{}{}",
                    board_id, s, deleted_filter, tag_filter
                )
            }
            (None, Some(a)) => {
                format!(
                    "SELECT id FROM cards WHERE board_id = '{}' AND assigned_to = '{}'{}{}",
                    board_id, a, deleted_filter, tag_filter
                )
            }
            (None, None) => {
                format!("SELECT id FROM cards WHERE board_id = '{}'{}{}",  board_id, deleted_filter, tag_filter)
            }
        };

        let mut rows = self.conn
            .query(&query, ())
            .await
            .map_err(|e| AgentBoardError::General(format!("Query failed: {}", e)))?;

        let mut cards = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AgentBoardError::General(format!("Row fetch failed: {}", e)))? {
            let card_id: String = row.get(0).unwrap_or_default();
            cards.push(self.load_card_full_with_deleted(&card_id, include_deleted).await?);
        }
        Ok(cards)
    }

    pub async fn get_cards_by_assignee(&self, session_id: &str, board_id: Option<&str>, status: Option<Status>) -> Result<Vec<Card>, AgentBoardError> {
        let query = match (&board_id, &status) {
            (Some(b), Some(s)) => {
                format!(
                    "SELECT id FROM cards WHERE assigned_to = '{}' AND board_id = '{}' AND status = '{}' AND deleted_at IS NULL",
                    session_id, b, s
                )
            }
            (Some(b), None) => {
                format!(
                    "SELECT id FROM cards WHERE assigned_to = '{}' AND board_id = '{}' AND deleted_at IS NULL",
                    session_id, b
                )
            }
            (None, Some(s)) => {
                format!(
                    "SELECT id FROM cards WHERE assigned_to = '{}' AND status = '{}' AND deleted_at IS NULL",
                    session_id, s
                )
            }
            (None, None) => {
                format!("SELECT id FROM cards WHERE assigned_to = '{}' AND deleted_at IS NULL", session_id)
            }
        };

        let mut rows = self.conn
            .query(&query, ())
            .await
            .map_err(|e| AgentBoardError::General(format!("Query failed: {}", e)))?;

        let mut cards = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AgentBoardError::General(format!("Row fetch failed: {}", e)))? {
            let card_id: String = row.get(0).unwrap_or_default();
            cards.push(self.load_card_full(&card_id).await?);
        }
        Ok(cards)
    }

    pub async fn create_card(&self, board_id: &str, name: String, description: Option<String>, status: Status) -> Result<Card, AgentBoardError> {
        // Verify board exists
        self.get_board(board_id).await?;

        let id = Self::generate_id("card");
        let now = Utc::now().to_rfc3339();
        let status_str = status.to_string();

        self.conn
            .execute(
                "INSERT INTO cards (id, board_id, name, description, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                [&id, board_id, &name, &description.clone().unwrap_or_default(), &status_str, &now, &now],
            )
            .await
            .map_err(|e| AgentBoardError::General(format!("Insert failed: {}", e)))?;

        self.get_card(&id).await
    }

    pub async fn update_card(
        &self,
        card_id: &str,
        update: crate::models::CardUpdate,
    ) -> Result<(), AgentBoardError> {
        // Verify card exists
        self.get_card(card_id).await?;

        let now = Utc::now().to_rfc3339();

        if let Some(n) = update.name {
            self.conn
                .execute("UPDATE cards SET name = ?1, updated_at = ?2 WHERE id = ?3", [&n, &now, card_id])
                .await
                .map_err(|e| AgentBoardError::General(format!("Update failed: {}", e)))?;
        }
        if let Some(d) = update.description {
            self.conn
                .execute("UPDATE cards SET description = ?1, updated_at = ?2 WHERE id = ?3", [&d, &now, card_id])
                .await
                .map_err(|e| AgentBoardError::General(format!("Update failed: {}", e)))?;
        }
        if let Some(s) = update.status {
            self.conn
                .execute("UPDATE cards SET status = ?1, updated_at = ?2 WHERE id = ?3", [&s.to_string(), &now, card_id])
                .await
                .map_err(|e| AgentBoardError::General(format!("Update failed: {}", e)))?;
        }
        if let Some(sid) = update.session_id {
            match sid {
                Some(s) => {
                    self.conn
                        .execute("UPDATE cards SET assigned_to = ?1, updated_at = ?2 WHERE id = ?3", [&s, &now, card_id])
                        .await
                        .map_err(|e| AgentBoardError::General(format!("Update failed: {}", e)))?;
                }
                None => {
                    self.conn
                        .execute("UPDATE cards SET assigned_to = NULL, updated_at = ?1 WHERE id = ?2", [&now, card_id])
                        .await
                        .map_err(|e| AgentBoardError::General(format!("Update failed: {}", e)))?;
                }
            }
        }
        for tag in update.add_tags {
            self.conn
                .execute("INSERT OR IGNORE INTO card_tags (card_id, tag) VALUES (?1, ?2)", [card_id, &tag])
                .await
                .map_err(|e| AgentBoardError::General(format!("Insert tag failed: {}", e)))?;
        }
        for tag in update.remove_tags {
            self.conn
                .execute("DELETE FROM card_tags WHERE card_id = ?1 AND tag = ?2", [card_id, &tag])
                .await
                .map_err(|e| AgentBoardError::General(format!("Delete tag failed: {}", e)))?;
        }

        Ok(())
    }

    pub async fn delete_card(&self, card_id: &str) -> Result<(), AgentBoardError> {
        // Verify card exists
        self.get_card(card_id).await?;

        let now = Utc::now().to_rfc3339();
        self.conn
            .execute(
                "UPDATE cards SET deleted_at = ?1, updated_at = ?1 WHERE id = ?2",
                [&now, card_id],
            )
            .await
            .map_err(|e| AgentBoardError::General(format!("Delete failed: {}", e)))?;

        Ok(())
    }

    // Checklist operations
    pub async fn add_checklist(&self, card_id: &str, name: String, items: Vec<String>) -> Result<Checklist, AgentBoardError> {
        // Verify card exists
        self.get_card(card_id).await?;

        let checklist_id = Self::generate_id("checklist");
        
        self.conn
            .execute(
                "INSERT INTO checklists (id, card_id, name) VALUES (?1, ?2, ?3)",
                [&checklist_id, card_id, &name],
            )
            .await
            .map_err(|e| AgentBoardError::General(format!("Insert checklist failed: {}", e)))?;

        let mut checklist_items = Vec::new();
        for item_text in items {
            let item_id = Self::generate_id("item");
            self.conn
                .execute(
                    "INSERT INTO checklist_items (id, checklist_id, text, checked) VALUES (?1, ?2, ?3, 0)",
                    libsql::params![item_id.as_str(), checklist_id.as_str(), item_text.as_str()],
                )
                .await
                .map_err(|e| AgentBoardError::General(format!("Insert item failed: {}", e)))?;
            checklist_items.push(ChecklistItem {
                id: item_id,
                text: item_text,
                checked: false,
            });
        }

        // Update card's updated_at
        let now = Utc::now().to_rfc3339();
        self.conn
            .execute("UPDATE cards SET updated_at = ?1 WHERE id = ?2", [&now, card_id])
            .await
            .map_err(|e| AgentBoardError::General(format!("Update failed: {}", e)))?;

        Ok(Checklist {
            id: checklist_id,
            name,
            items: checklist_items,
        })
    }

    pub async fn check_item(&self, item_id: &str, checked: bool) -> Result<(), AgentBoardError> {
        let checked_val = if checked { 1 } else { 0 };
        
        let result = self.conn
            .execute(
                "UPDATE checklist_items SET checked = ?1 WHERE id = ?2",
                libsql::params![checked_val, item_id],
            )
            .await
            .map_err(|e| AgentBoardError::General(format!("Update failed: {}", e)))?;

        if result == 0 {
            return Err(AgentBoardError::NotFound(format!("Checklist item not found: {}", item_id)));
        }

        // Update the card's updated_at via the checklist
        self.conn
            .execute(
                "UPDATE cards SET updated_at = ?1 WHERE id = (SELECT card_id FROM checklists WHERE id = (SELECT checklist_id FROM checklist_items WHERE id = ?2))",
                [&Utc::now().to_rfc3339(), item_id],
            )
            .await
            .map_err(|e| AgentBoardError::General(format!("Update card timestamp failed: {}", e)))?;

        Ok(())
    }

    // Comment operations
    pub async fn add_comment(&self, card_id: &str, text: String, author: Option<String>) -> Result<Comment, AgentBoardError> {
        // Verify card exists
        self.get_card(card_id).await?;

        let id = Self::generate_id("comment");
        let now = Utc::now();
        let now_str = now.to_rfc3339();

        self.conn
            .execute(
                "INSERT INTO comments (id, card_id, author, text, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                [&id, card_id, &author.clone().unwrap_or_default(), &text, &now_str],
            )
            .await
            .map_err(|e| AgentBoardError::General(format!("Insert comment failed: {}", e)))?;

        Ok(Comment {
            id,
            card_id: card_id.to_string(),
            author,
            text,
            created_at: now,
        })
    }

    pub async fn list_comments(&self, card_id: &str) -> Result<Vec<Comment>, AgentBoardError> {
        // Verify card exists
        self.get_card(card_id).await?;

        let mut rows = self.conn
            .query(
                "SELECT id, card_id, author, text, created_at FROM comments WHERE card_id = ?1 ORDER BY created_at ASC",
                [card_id],
            )
            .await
            .map_err(|e| AgentBoardError::General(format!("Query failed: {}", e)))?;

        let mut comments = Vec::new();
        while let Some(row) = rows.next().await.map_err(|e| AgentBoardError::General(format!("Row fetch failed: {}", e)))? {
            comments.push(Comment {
                id: row.get::<String>(0).unwrap_or_default(),
                card_id: row.get::<String>(1).unwrap_or_default(),
                author: row.get::<Option<String>>(2).ok().flatten(),
                text: row.get::<String>(3).unwrap_or_default(),
                created_at: Self::parse_datetime(&row.get::<String>(4).unwrap_or_default()),
            });
        }
        Ok(comments)
    }
}
