use crate::models::*;
use colored::Colorize;
use tabled::{Table, Tabled, settings::Style};

#[derive(Tabled)]
struct AgentRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Command")]
    command: String,
    #[tabled(rename = "Working Directory")]
    working_directory: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

#[derive(Tabled)]
struct CardRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Assigned To")]
    assigned_to: String,
    #[tabled(rename = "Board")]
    board_id: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

#[derive(Tabled)]
struct BoardRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

pub fn print_cards(cards: &[Card], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&cards).unwrap());
        }
        OutputFormat::Table => {
            if cards.is_empty() {
                println!("No cards found.");
                return;
            }
            let rows: Vec<CardRow> = cards
                .iter()
                .map(|c| {
                    let deleted_marker = if c.deleted_at.is_some() {
                        " [DELETED]"
                    } else {
                        ""
                    };
                    CardRow {
                        id: c.id.clone(),
                        name: format!("{}{}", truncate(&c.name, 35), deleted_marker),
                        status: c.status.to_string(),
                        assigned_to: c.assigned_to.clone().unwrap_or_else(|| "-".to_string()),
                        board_id: c.board_id.clone(),
                        created_at: c.created_at.format("%Y-%m-%d %H:%M").to_string(),
                    }
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{}", table);
        }
        OutputFormat::Simple => {
            for card in cards {
                println!("{}", card.id);
            }
        }
        OutputFormat::Pretty => {
            // Pretty format doesn't apply to card lists, fall back to table
            print_cards(cards, OutputFormat::Table);
        }
    }
}

pub fn print_card(card: &Card, comments: &[Comment], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "card": card,
                "comments": comments
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        OutputFormat::Table => {
            println!("Card: {}", card.id);
            println!("Name: {}", card.name);
            println!("Board: {}", card.board_id);
            println!("Status: {}", card.status);
            println!(
                "Assigned To: {}",
                card.assigned_to.as_deref().unwrap_or("-")
            );
            if let Some(desc) = &card.description {
                println!("Description: {}", desc);
            }
            if !card.tags.is_empty() {
                println!("Tags: {}", card.tags.join(", "));
            }
            for checklist in &card.checklists {
                println!("\nChecklist: {} ({})", checklist.name, checklist.id);
                for item in &checklist.items {
                    let check = if item.checked { "x" } else { " " };
                    println!("  [{}] {} ({})", check, item.text, item.id);
                }
            }
            if !comments.is_empty() {
                println!("\nComments:");
                for comment in comments {
                    let author = comment.author.as_deref().unwrap_or("anonymous");
                    let time = comment.created_at.format("%Y-%m-%d %H:%M");
                    println!("  [{}] {} ({})", author, time, comment.id);
                    for line in comment.text.lines() {
                        println!("    {}", line);
                    }
                }
            }
        }
        OutputFormat::Simple => {
            println!("{}", card.id);
        }
        OutputFormat::Pretty => {
            // Pretty format doesn't apply to single card, fall back to table
            print_card(card, comments, OutputFormat::Table);
        }
    }
}

pub fn print_boards(boards: &[Board], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&boards).unwrap());
        }
        OutputFormat::Table => {
            if boards.is_empty() {
                println!("No boards found.");
                return;
            }
            let rows: Vec<BoardRow> = boards
                .iter()
                .map(|b| {
                    let deleted_marker = if b.deleted_at.is_some() {
                        " [DELETED]"
                    } else {
                        ""
                    };
                    BoardRow {
                        id: b.id.clone(),
                        name: format!("{}{}", b.name, deleted_marker),
                        description: b.description.clone().unwrap_or_else(|| "-".to_string()),
                        created_at: b.created_at.format("%Y-%m-%d %H:%M").to_string(),
                    }
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{}", table);
        }
        OutputFormat::Simple => {
            for board in boards {
                println!("{}", board.id);
            }
        }
        OutputFormat::Pretty => {
            // Pretty format doesn't apply to board list, fall back to table
            print_boards(boards, OutputFormat::Table);
        }
    }
}

pub fn print_board(board: &Board, summary: &BoardSummary, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            let output = serde_json::json!({
                "board": board,
                "summary": summary
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        OutputFormat::Table => {
            println!("Board: {}", board.id);
            println!("Name: {}", board.name);
            if let Some(desc) = &board.description {
                println!("Description: {}", desc);
            }
            println!("\nSummary:");
            println!("  Todo: {}", summary.todo_count);
            println!("  In Progress: {}", summary.in_progress_count);
            println!("  Pending Review: {}", summary.pending_review_count);
            println!("  Done: {}", summary.done_count);
            println!("  Total: {}", summary.total_cards);
        }
        OutputFormat::Simple => {
            println!("{}", board.id);
        }
        OutputFormat::Pretty => {
            // Pretty is handled separately in main.rs with print_kanban
            // This shouldn't be reached, but fall back to table
            print_board(board, summary, OutputFormat::Table);
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Print a visual kanban board with cards organized by status columns
pub fn print_kanban(
    board: &Board,
    cards: &[Card],
    comment_counts: &std::collections::HashMap<String, usize>,
) {
    const COL_WIDTH: usize = 28;
    const CARD_INNER: usize = COL_WIDTH - 4; // Account for borders and padding

    // Group cards by status
    let todo: Vec<_> = cards.iter().filter(|c| c.status == Status::Todo).collect();
    let in_progress: Vec<_> = cards
        .iter()
        .filter(|c| c.status == Status::InProgress)
        .collect();
    let pending_review: Vec<_> = cards
        .iter()
        .filter(|c| c.status == Status::PendingReview)
        .collect();
    let done: Vec<_> = cards.iter().filter(|c| c.status == Status::Done).collect();

    // Board header
    println!();
    println!("┌{}┐", "─".repeat(COL_WIDTH * 4 + 3));
    let title = format!("{} - {}", board.name, board.id);
    println!("│ {:<width$} │", title, width = COL_WIDTH * 4 + 1);
    if let Some(desc) = &board.description {
        println!(
            "│ {:<width$} │",
            truncate(desc, COL_WIDTH * 4 - 1),
            width = COL_WIDTH * 4 + 1
        );
    }
    println!(
        "├{}┬{}┬{}┬{}┤",
        "─".repeat(COL_WIDTH),
        "─".repeat(COL_WIDTH),
        "─".repeat(COL_WIDTH),
        "─".repeat(COL_WIDTH)
    );

    // Column headers with colors
    let header_colors = [
        "TODO".white(),
        "IN PROGRESS".yellow(),
        "PENDING REVIEW".cyan(),
        "DONE".green(),
    ];
    print!("│");
    for header in &header_colors {
        print!(" {:<width$}│", header, width = COL_WIDTH - 1);
    }
    println!();

    // Counts
    print!("│");
    for count in [
        todo.len(),
        in_progress.len(),
        pending_review.len(),
        done.len(),
    ] {
        let count_str = format!("({} cards)", count);
        print!(" {:<width$}│", count_str, width = COL_WIDTH - 1);
    }
    println!();

    println!(
        "├{}┼{}┼{}┼{}┤",
        "─".repeat(COL_WIDTH),
        "─".repeat(COL_WIDTH),
        "─".repeat(COL_WIDTH),
        "─".repeat(COL_WIDTH)
    );

    // Find max cards in any column
    let max_cards = [
        todo.len(),
        in_progress.len(),
        pending_review.len(),
        done.len(),
    ]
    .into_iter()
    .max()
    .unwrap_or(0);

    let columns = [&todo, &in_progress, &pending_review, &done];

    // Print cards row by row (each card takes 5 lines)
    for i in 0..max_cards {
        // Card top border
        print!("│");
        for col in &columns {
            if i < col.len() {
                print!(" ┌{}┐ │", "─".repeat(CARD_INNER));
            } else {
                print!("{:width$}│", "", width = COL_WIDTH);
            }
        }
        println!();

        // Card name line 1 (colored by status)
        print!("│");
        for (col_idx, col) in columns.iter().enumerate() {
            if i < col.len() {
                let card = col[i];
                let max_chars = CARD_INNER - 2;
                let name_line1 = if card.name.len() > max_chars {
                    &card.name[..max_chars]
                } else {
                    &card.name
                };
                let colored_name = match col_idx {
                    1 => format!("{:<width$}", name_line1, width = max_chars).yellow(),
                    2 => format!("{:<width$}", name_line1, width = max_chars).cyan(),
                    3 => format!("{:<width$}", name_line1, width = max_chars).green(),
                    _ => format!("{:<width$}", name_line1, width = max_chars).white(),
                };
                print!(" │ {} │ │", colored_name);
            } else {
                print!("{:width$}│", "", width = COL_WIDTH);
            }
        }
        println!();

        // Card name line 2 (continuation, colored by status)
        print!("│");
        for (col_idx, col) in columns.iter().enumerate() {
            if i < col.len() {
                let card = col[i];
                let max_chars = CARD_INNER - 2;
                let name_line2 = if card.name.len() > max_chars {
                    truncate(&card.name[max_chars..], max_chars)
                } else {
                    String::new()
                };
                let colored_name = match col_idx {
                    1 => format!("{:<width$}", name_line2, width = max_chars).yellow(),
                    2 => format!("{:<width$}", name_line2, width = max_chars).cyan(),
                    3 => format!("{:<width$}", name_line2, width = max_chars).green(),
                    _ => format!("{:<width$}", name_line2, width = max_chars).white(),
                };
                print!(" │ {} │ │", colored_name);
            } else {
                print!("{:width$}│", "", width = COL_WIDTH);
            }
        }
        println!();

        // Card ID line (dimmed)
        print!("│");
        for col in &columns {
            if i < col.len() {
                let card = col[i];
                let id_short = if card.id.len() > CARD_INNER - 2 {
                    format!("{}...", &card.id[..CARD_INNER - 5])
                } else {
                    card.id.clone()
                };
                let dimmed_id = format!("{:<width$}", id_short, width = CARD_INNER - 2).dimmed();
                print!(" │ {} │ │", dimmed_id);
            } else {
                print!("{:width$}│", "", width = COL_WIDTH);
            }
        }
        println!();

        // Card assignee line
        print!("│");
        for col in &columns {
            if i < col.len() {
                let card = col[i];
                let assignee = card.assigned_to.as_deref().unwrap_or("-");
                let assignee_display = format!("@{}", truncate(assignee, CARD_INNER - 4));
                print!(
                    " │ {:<width$} │ │",
                    assignee_display,
                    width = CARD_INNER - 2
                );
            } else {
                print!("{:width$}│", "", width = COL_WIDTH);
            }
        }
        println!();

        // Card tags line 1
        print!("│");
        for col in &columns {
            if i < col.len() {
                let card = col[i];
                if card.tags.is_empty() {
                    print!(" │ {:<width$} │ │", "", width = CARD_INNER - 2);
                } else {
                    let tags_str = card
                        .tags
                        .iter()
                        .map(|t| format!("#{}", t))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let line1 = if tags_str.len() > CARD_INNER - 2 {
                        &tags_str[..CARD_INNER - 2]
                    } else {
                        &tags_str
                    };
                    let tags_part = format!("{:<width$}", line1, width = CARD_INNER - 2);
                    print!(" │ {} │ │", tags_part.blue());
                }
            } else {
                print!("{:width$}│", "", width = COL_WIDTH);
            }
        }
        println!();

        // Card tags line 2
        print!("│");
        for col in &columns {
            if i < col.len() {
                let card = col[i];
                if card.tags.is_empty() {
                    print!(" │ {:<width$} │ │", "", width = CARD_INNER - 2);
                } else {
                    let tags_str = card
                        .tags
                        .iter()
                        .map(|t| format!("#{}", t))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let line2 = if tags_str.len() > CARD_INNER - 2 {
                        let remaining = &tags_str[CARD_INNER - 2..];
                        truncate(remaining, CARD_INNER - 2)
                    } else {
                        String::new()
                    };
                    let tags_part = format!("{:<width$}", line2, width = CARD_INNER - 2);
                    print!(" │ {} │ │", tags_part.blue());
                }
            } else {
                print!("{:width$}│", "", width = COL_WIDTH);
            }
        }
        println!();

        // Card comments line
        print!("│");
        for col in &columns {
            if i < col.len() {
                let card = col[i];
                let comment_count = comment_counts.get(&card.id).copied().unwrap_or(0);
                let comment_text = if comment_count == 1 {
                    "[1 comment]".to_string()
                } else {
                    format!("[{} comments]", comment_count)
                };
                let padded = format!("{:<width$}", comment_text, width = CARD_INNER - 2);
                print!(" │ {} │ │", padded.dimmed());
            } else {
                print!("{:width$}│", "", width = COL_WIDTH);
            }
        }
        println!();

        // Card bottom border
        print!("│");
        for col in &columns {
            if i < col.len() {
                print!(" └{}┘ │", "─".repeat(CARD_INNER));
            } else {
                print!("{:width$}│", "", width = COL_WIDTH);
            }
        }
        println!();
    }

    // If no cards at all
    if max_cards == 0 {
        print!("│");
        for _ in 0..4 {
            print!(" {:<width$}│", "(empty)", width = COL_WIDTH - 1);
        }
        println!();
    }

    // Bottom border
    println!(
        "└{}┴{}┴{}┴{}┘",
        "─".repeat(COL_WIDTH),
        "─".repeat(COL_WIDTH),
        "─".repeat(COL_WIDTH),
        "─".repeat(COL_WIDTH)
    );
    println!();
}

pub fn print_agents(agents: &[Agent], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&agents).unwrap());
        }
        OutputFormat::Table => {
            if agents.is_empty() {
                println!("No agents found.");
                return;
            }
            let rows: Vec<AgentRow> = agents
                .iter()
                .map(|a| {
                    let inactive_marker = if a.deactivated_at.is_some() {
                        " [INACTIVE]"
                    } else {
                        ""
                    };
                    AgentRow {
                        id: a.id.clone(),
                        name: format!("{}{}", a.name, inactive_marker),
                        command: a.command.clone(),
                        working_directory: truncate(&a.working_directory, 40),
                        created_at: a.created_at.format("%Y-%m-%d %H:%M").to_string(),
                    }
                })
                .collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{}", table);
        }
        OutputFormat::Simple => {
            for agent in agents {
                println!("{}", agent.id);
            }
        }
        OutputFormat::Pretty => {
            // Pretty format doesn't apply to agent list, fall back to table
            print_agents(agents, OutputFormat::Table);
        }
    }
}

pub fn print_agent(agent: &Agent, format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&agent).unwrap());
        }
        OutputFormat::Table => {
            println!("Agent: {}", agent.id);
            println!("Name: {}", agent.name);
            println!("Command: {}", agent.command);
            println!("Working Directory: {}", agent.working_directory);
            if let Some(desc) = &agent.description {
                println!("Description: {}", desc);
            }
            println!("Created: {}", agent.created_at.format("%Y-%m-%d %H:%M"));
            if let Some(deactivated) = agent.deactivated_at {
                println!("Deactivated: {}", deactivated.format("%Y-%m-%d %H:%M"));
            }
        }
        OutputFormat::Simple => {
            println!("{}", agent.id);
        }
        OutputFormat::Pretty => {
            // Pretty format doesn't apply to single agent, fall back to table
            print_agent(agent, OutputFormat::Table);
        }
    }
}

pub fn print_agent_whoami(agent: &Agent, current_dir: &str) {
    println!("Agent: {}", agent.id);
    println!("Name: {}", agent.name);
    println!("Command: {}", agent.command);
    println!("Working Directory: {}", agent.working_directory);
    if let Some(desc) = &agent.description {
        println!("Description: {}", desc);
    }

    // Check if current directory matches
    if current_dir != agent.working_directory {
        eprintln!(
            "WARNING: Current directory ({}) does not match registered working directory",
            current_dir
        );
    }
}

pub fn print_comments(comments: &[Comment], format: OutputFormat) {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&comments).unwrap());
        }
        OutputFormat::Table => {
            if comments.is_empty() {
                println!("No comments found.");
                return;
            }
            for comment in comments {
                let author = comment.author.as_deref().unwrap_or("anonymous");
                let time = comment.created_at.format("%Y-%m-%d %H:%M");
                println!("─────────────────────────────────────────────────────────────");
                println!("[{}] {} ({})", author, time, comment.id);
                println!();
                println!("{}", comment.text);
                println!();
            }
        }
        OutputFormat::Simple => {
            for comment in comments {
                println!("{}", comment.id);
            }
        }
        OutputFormat::Pretty => {
            // Pretty format doesn't apply to comments, fall back to table
            print_comments(comments, OutputFormat::Table);
        }
    }
}
