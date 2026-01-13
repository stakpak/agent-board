mod cli;
mod db;
mod models;
mod output;

use cli::{Cli, Commands, CardCommands, ChecklistCommands, CommentCommands, BoardCommands};
use clap::Parser;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");
    
    match rt.block_on(run(cli)) {
        Ok(()) => ExitCode::from(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            e.exit_code()
        }
    }
}

async fn run(cli: Cli) -> Result<(), AgentBoardError> {
    let db = db::Database::load(&cli).await?;
    let default_format = cli.format.clone();
    let quiet = cli.quiet;
    let session_id_result = cli.get_session_id();
    
    match cli.command {
        Commands::Mine { board, status, format } => {
            let session_id = session_id_result?;
            let cards = db.get_cards_by_assignee(&session_id, board.as_deref(), status).await?;
            output::print_cards(&cards, format.unwrap_or(default_format));
        }
        Commands::Card { command } => match command {
            CardCommands::Get { card_id, format } => {
                let card = db.get_card(&card_id).await?;
                let comments = db.list_comments(&card_id).await?;
                output::print_card(&card, &comments, format.unwrap_or(default_format));
            }
            CardCommands::List { board_id, status, assigned_to, include_deleted, format } => {
                let cards = db.list_cards(&board_id, status, assigned_to.as_deref(), include_deleted).await?;
                output::print_cards(&cards, format.unwrap_or(default_format));
            }
            CardCommands::Create { board_id, name, description, status } => {
                let card = db.create_card(&board_id, name, description, status).await?;
                if !quiet {
                    println!("Created card: {}", card.id);
                }
            }
            CardCommands::Update { card_id, name, description, status, agent_session_id, assign_to_me, add_tag, remove_tag } => {
                let session_id = match (&agent_session_id, assign_to_me) {
                    (Some(s), _) if s == "null" => Some(None), // explicit unassign
                    (Some(s), _) => Some(Some(s.clone())),     // explicit assign
                    (None, true) => {
                        // --assign-to-me flag: use current session ID
                        Some(Some(std::env::var("AGENT_BOARD_SESSION_ID")
                            .map_err(|_| AgentBoardError::InvalidArgs("AGENT_BOARD_SESSION_ID environment variable not set".into()))?))
                    }
                    (None, false) => {
                        // Use env var session ID if status is being changed to in_progress
                        if status == Some(models::Status::InProgress) {
                            Some(Some(std::env::var("AGENT_BOARD_SESSION_ID")
                                .map_err(|_| AgentBoardError::InvalidArgs("AGENT_BOARD_SESSION_ID environment variable not set".into()))?))
                        } else {
                            None // no change to assignment
                        }
                    }
                };
                let update = models::CardUpdate {
                    name,
                    description,
                    status,
                    session_id,
                    add_tags: add_tag,
                    remove_tags: remove_tag,
                };
                db.update_card(&card_id, update).await?;
                if !quiet {
                    println!("Updated card: {}", card_id);
                }
            }
            CardCommands::Delete { card_id } => {
                db.delete_card(&card_id).await?;
                if !quiet {
                    println!("Deleted card: {}", card_id);
                }
            }
        },
        Commands::Checklist { command } => match command {
            ChecklistCommands::Add { card_id, name, item } => {
                let checklist = db.add_checklist(&card_id, name, item).await?;
                if !quiet {
                    println!("Added checklist: {}", checklist.id);
                }
            }
            ChecklistCommands::Check { item_id, uncheck } => {
                db.check_item(&item_id, !uncheck).await?;
                if !quiet {
                    println!("{} item: {}", if uncheck { "Unchecked" } else { "Checked" }, item_id);
                }
            }
        },
        Commands::Comment { command } => match command {
            CommentCommands::Add { card_id, text, file } => {
                let content = if let Some(path) = file {
                    std::fs::read_to_string(&path)
                        .map_err(|e| AgentBoardError::General(format!("Failed to read file: {}", e)))?
                } else {
                    text.ok_or(AgentBoardError::InvalidArgs("Either text or --file required".into()))?
                };
                let session_id = std::env::var("AGENT_BOARD_SESSION_ID").ok();
                let comment = db.add_comment(&card_id, content, session_id).await?;
                if !quiet {
                    println!("Added comment: {}", comment.id);
                }
            }
            CommentCommands::List { card_id, format } => {
                let comments = db.list_comments(&card_id).await?;
                output::print_comments(&comments, format.unwrap_or(default_format));
            }
        },
        Commands::Board { command } => match command {
            BoardCommands::Get { board_id, format } => {
                let board = db.get_board(&board_id).await?;
                let summary = db.get_board_summary(&board_id).await?;
                output::print_board(&board, &summary, format.unwrap_or(default_format));
            }
            BoardCommands::List { include_deleted, format } => {
                let boards = db.list_boards(include_deleted).await?;
                output::print_boards(&boards, format.unwrap_or(default_format));
            }
            BoardCommands::Create { name, description } => {
                let board = db.create_board(name, description).await?;
                if !quiet {
                    println!("Created board: {}", board.id);
                }
            }
            BoardCommands::Delete { board_id } => {
                db.delete_board(&board_id).await?;
                if !quiet {
                    println!("Deleted board: {}", board_id);
                }
            }
        },
    }
    
    db.save().await?;
    Ok(())
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentBoardError {
    #[error("{0}")]
    General(String),
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Session conflict: {0}")]
    SessionConflict(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl AgentBoardError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            AgentBoardError::General(_) => ExitCode::from(1),
            AgentBoardError::InvalidArgs(_) => ExitCode::from(2),
            AgentBoardError::NotFound(_) => ExitCode::from(4),
            AgentBoardError::PermissionDenied(_) => ExitCode::from(5),
            AgentBoardError::SessionConflict(_) => ExitCode::from(6),
            AgentBoardError::Io(_) => ExitCode::from(1),
            AgentBoardError::Json(_) => ExitCode::from(1),
        }
    }
}
