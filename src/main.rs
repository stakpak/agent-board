mod cli;
mod db;
mod models;
mod output;

use clap::Parser;
use cli::{Cli, Commands, CreateCommands, DeleteCommands, ListCommands, UpdateCommands};
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
    // Handle version command before loading DB
    if matches!(cli.command, Commands::Version) {
        println!("agent-board {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let db = db::Database::load(&cli).await?;
    let default_format = cli.format.clone();
    let quiet = cli.quiet;
    let agent_id_result = cli.get_agent_id();

    match cli.command {
        Commands::Version => unreachable!(), // Handled above

        Commands::Get { id, format } => {
            let fmt = format.unwrap_or(default_format);
            if id.starts_with("agent_") {
                let agent = db.get_agent(&id).await?;
                output::print_agent(&agent, fmt);
            } else if id.starts_with("board_") {
                let board = db.get_board(&id).await?;
                if fmt == models::OutputFormat::Pretty {
                    let cards = db.list_cards(&id, None, None, &[], false).await?;
                    let card_ids: Vec<String> = cards.iter().map(|c| c.id.clone()).collect();
                    let comment_counts = db.get_comment_counts(&card_ids).await?;
                    output::print_kanban(&board, &cards, &comment_counts);
                } else {
                    let summary = db.get_board_summary(&id).await?;
                    output::print_board(&board, &summary, fmt);
                }
            } else if id.starts_with("card_") {
                let card = db.get_card(&id).await?;
                let comments = db.list_comments(&id).await?;
                output::print_card(&card, &comments, fmt);
            } else {
                return Err(AgentBoardError::InvalidArgs(format!(
                    "Unknown ID prefix: {}. Expected agent_, board_, or card_",
                    id
                )));
            }
        }

        Commands::Mine {
            board,
            status,
            format,
        } => {
            let agent_id = agent_id_result?;
            let cards = db
                .get_cards_by_assignee(&agent_id, board.as_deref(), status)
                .await?;
            output::print_cards(&cards, format.unwrap_or(default_format));
        }

        Commands::Whoami => {
            let agent_id = agent_id_result?;
            let agent = db.get_agent(&agent_id).await?;
            let cwd = std::env::current_dir()
                .map_err(|e| {
                    AgentBoardError::General(format!("Failed to get current directory: {}", e))
                })?
                .to_string_lossy()
                .to_string();
            output::print_agent_whoami(&agent, &cwd);
        }

        // ====================================================================
        // LIST commands
        // ====================================================================
        Commands::List { command } => match command {
            ListCommands::Boards {
                include_deleted,
                format,
            } => {
                let boards = db.list_boards(include_deleted).await?;
                output::print_boards(&boards, format.unwrap_or(default_format));
            }
            ListCommands::Cards {
                board_id,
                status,
                assigned_to,
                tag,
                include_deleted,
                format,
            } => {
                let cards = db
                    .list_cards(
                        &board_id,
                        status,
                        assigned_to.as_deref(),
                        &tag,
                        include_deleted,
                    )
                    .await?;
                output::print_cards(&cards, format.unwrap_or(default_format));
            }
            ListCommands::Agents {
                include_inactive,
                format,
            } => {
                let agents = db.list_agents(include_inactive).await?;
                output::print_agents(&agents, format.unwrap_or(default_format));
            }
            ListCommands::Comments { card_id, format } => {
                let comments = db.list_comments(&card_id).await?;
                output::print_comments(&comments, format.unwrap_or(default_format));
            }
        },

        // ====================================================================
        // CREATE commands
        // ====================================================================
        Commands::Create { command } => match command {
            CreateCommands::Board { name, description } => {
                let board = db.create_board(name, description).await?;
                if !quiet {
                    println!("Created board: {}", board.id);
                }
            }
            CreateCommands::Card {
                board_id,
                name,
                description,
                status,
            } => {
                let card = db.create_card(&board_id, name, description, status).await?;
                if !quiet {
                    println!("Created card: {}", card.id);
                }
            }
            CreateCommands::Agent {
                command,
                name,
                description,
            } => {
                let cwd = std::env::current_dir()
                    .map_err(|e| {
                        AgentBoardError::General(format!("Failed to get current directory: {}", e))
                    })?
                    .to_string_lossy()
                    .to_string();
                let agent = db
                    .register_agent(name, command, cwd.clone(), description)
                    .await?;
                if !quiet {
                    println!("Created agent: {} (Name: {})", agent.id, agent.name);
                    println!("Working directory: {}", cwd);
                    println!();
                    println!("To use this agent, run:");
                    println!("  export AGENT_BOARD_AGENT_ID={}", agent.id);
                }
            }
            CreateCommands::Checklist { card_id, item } => {
                let items = db.add_checklist_items(&card_id, item).await?;
                if !quiet {
                    println!("Added {} checklist item(s)", items.len());
                }
            }
            CreateCommands::Comment {
                card_id,
                text,
                file,
            } => {
                let content = if let Some(path) = file {
                    std::fs::read_to_string(&path).map_err(|e| {
                        AgentBoardError::General(format!("Failed to read file: {}", e))
                    })?
                } else {
                    text.ok_or(AgentBoardError::InvalidArgs(
                        "Either text or --file required".into(),
                    ))?
                };
                let agent_id = std::env::var("AGENT_BOARD_AGENT_ID").ok();
                let comment = db.add_comment(&card_id, content, agent_id).await?;
                if !quiet {
                    println!("Added comment: {}", comment.id);
                }
            }
        },

        // ====================================================================
        // UPDATE commands
        // ====================================================================
        Commands::Update { command } => match command {
            UpdateCommands::Card {
                card_id,
                name,
                description,
                status,
                assign,
                assign_to_me,
                add_tag,
                remove_tag,
            } => {
                let agent_id = match (&assign, assign_to_me) {
                    (Some(s), _) if s == "null" => Some(None), // explicit unassign
                    (Some(s), _) => Some(Some(s.clone())),     // explicit assign
                    (None, true) => {
                        // --assign-to-me flag: use current agent ID or auto-create
                        let id = match std::env::var("AGENT_BOARD_AGENT_ID") {
                            Ok(id) => id,
                            Err(_) => {
                                // Auto-create agent identity
                                let cwd = std::env::current_dir()
                                    .map_err(|e| {
                                        AgentBoardError::General(format!(
                                            "Failed to get current directory: {}",
                                            e
                                        ))
                                    })?
                                    .to_string_lossy()
                                    .to_string();
                                let agent = db
                                    .register_agent(None, "stakpak".to_string(), cwd, None)
                                    .await?;
                                eprintln!(
                                    "Note: Created agent identity {} for this session.",
                                    agent.id
                                );
                                eprintln!("      To persist: export AGENT_BOARD_AGENT_ID={}", agent.id);
                                agent.id
                            }
                        };
                        Some(Some(id))
                    }
                    (None, false) => {
                        // Use env var agent ID if status is being changed to in_progress
                        if status == Some(models::Status::InProgress) {
                            let id = match std::env::var("AGENT_BOARD_AGENT_ID") {
                                Ok(id) => id,
                                Err(_) => {
                                    // Auto-create agent identity
                                    let cwd = std::env::current_dir()
                                        .map_err(|e| {
                                            AgentBoardError::General(format!(
                                                "Failed to get current directory: {}",
                                                e
                                            ))
                                        })?
                                        .to_string_lossy()
                                        .to_string();
                                    let agent = db
                                        .register_agent(None, "stakpak".to_string(), cwd, None)
                                        .await?;
                                    eprintln!(
                                        "Note: Created agent identity {} for this session.",
                                        agent.id
                                    );
                                    eprintln!(
                                        "      To persist: export AGENT_BOARD_AGENT_ID={}",
                                        agent.id
                                    );
                                    agent.id
                                }
                            };
                            Some(Some(id))
                        } else {
                            None // no change to assignment
                        }
                    }
                };
                let update = models::CardUpdate {
                    name,
                    description,
                    status,
                    session_id: agent_id,
                    add_tags: add_tag,
                    remove_tags: remove_tag,
                };
                db.update_card(&card_id, update).await?;
                if !quiet {
                    println!("Updated card: {}", card_id);
                }
            }
            UpdateCommands::Board {
                board_id,
                name,
                description,
            } => {
                db.update_board(&board_id, name, description).await?;
                if !quiet {
                    println!("Updated board: {}", board_id);
                }
            }
            UpdateCommands::Agent {
                agent_id,
                name,
                command,
                description,
                workdir,
            } => {
                let working_directory = match workdir {
                    Some(w) if w == "." => Some(
                        std::env::current_dir()
                            .map_err(|e| {
                                AgentBoardError::General(format!(
                                    "Failed to get current directory: {}",
                                    e
                                ))
                            })?
                            .to_string_lossy()
                            .to_string(),
                    ),
                    Some(w) => Some(w),
                    None => None,
                };
                let update = models::AgentUpdate {
                    name,
                    command,
                    description,
                    working_directory,
                };
                db.update_agent(&agent_id, update).await?;
                if !quiet {
                    println!("Updated agent: {}", agent_id);
                }
            }
            UpdateCommands::ChecklistItem {
                item_id,
                check,
                uncheck,
            } => {
                // Require explicit --check or --uncheck flag
                if !check && !uncheck {
                    return Err(AgentBoardError::InvalidArgs(
                        "Must specify --check or --uncheck".into(),
                    ));
                }
                db.check_item(&item_id, check).await?;
                if !quiet {
                    println!(
                        "{} item: {}",
                        if check { "Checked" } else { "Unchecked" },
                        item_id
                    );
                }
            }
        },

        // ====================================================================
        // DELETE commands
        // ====================================================================
        Commands::Delete { command } => match command {
            DeleteCommands::Board { board_id } => {
                db.delete_board(&board_id).await?;
                if !quiet {
                    println!("Deleted board: {}", board_id);
                }
            }
            DeleteCommands::Card { card_id } => {
                db.delete_card(&card_id).await?;
                if !quiet {
                    println!("Deleted card: {}", card_id);
                }
            }
            DeleteCommands::Agent { agent_id } => {
                db.unregister_agent(&agent_id).await?;
                if !quiet {
                    println!("Deleted agent: {}", agent_id);
                }
            }
            DeleteCommands::Comment { comment_id } => {
                db.delete_comment(&comment_id).await?;
                if !quiet {
                    println!("Deleted comment: {}", comment_id);
                }
            }
            DeleteCommands::ChecklistItem { item_id } => {
                db.delete_checklist_item(&item_id).await?;
                if !quiet {
                    println!("Deleted checklist item: {}", item_id);
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
