use crate::models::*;
use tabled::{Table, Tabled, settings::Style};

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
            let rows: Vec<CardRow> = cards.iter().map(|c| {
                let deleted_marker = if c.deleted_at.is_some() { " [DELETED]" } else { "" };
                CardRow {
                    id: c.id.clone(),
                    name: format!("{}{}", truncate(&c.name, 35), deleted_marker),
                    status: c.status.to_string(),
                    assigned_to: c.assigned_to.clone().unwrap_or_else(|| "-".to_string()),
                    board_id: c.board_id.clone(),
                    created_at: c.created_at.format("%Y-%m-%d %H:%M").to_string(),
                }
            }).collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{}", table);
        }
        OutputFormat::Simple => {
            for card in cards {
                println!("{}", card.id);
            }
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
            println!("Assigned To: {}", card.assigned_to.as_deref().unwrap_or("-"));
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
            let rows: Vec<BoardRow> = boards.iter().map(|b| {
                let deleted_marker = if b.deleted_at.is_some() { " [DELETED]" } else { "" };
                BoardRow {
                    id: b.id.clone(),
                    name: format!("{}{}", b.name, deleted_marker),
                    description: b.description.clone().unwrap_or_else(|| "-".to_string()),
                    created_at: b.created_at.format("%Y-%m-%d %H:%M").to_string(),
                }
            }).collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{}", table);
        }
        OutputFormat::Simple => {
            for board in boards {
                println!("{}", board.id);
            }
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
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
