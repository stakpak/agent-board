-- Agent Board SQLite Schema
-- Migrated from JSON file storage to libsql

-- Boards table
CREATE TABLE IF NOT EXISTS boards (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Cards table
CREATE TABLE IF NOT EXISTS cards (
    id TEXT PRIMARY KEY NOT NULL,
    board_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'todo',
    assigned_to TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE
);

-- Card tags (many-to-many via separate table)
CREATE TABLE IF NOT EXISTS card_tags (
    card_id TEXT NOT NULL,
    tag TEXT NOT NULL,
    PRIMARY KEY (card_id, tag),
    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
);

-- Checklists table
CREATE TABLE IF NOT EXISTS checklists (
    id TEXT PRIMARY KEY NOT NULL,
    card_id TEXT NOT NULL,
    name TEXT NOT NULL,
    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
);

-- Checklist items table
CREATE TABLE IF NOT EXISTS checklist_items (
    id TEXT PRIMARY KEY NOT NULL,
    checklist_id TEXT NOT NULL,
    text TEXT NOT NULL,
    checked INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (checklist_id) REFERENCES checklists(id) ON DELETE CASCADE
);

-- Comments table
CREATE TABLE IF NOT EXISTS comments (
    id TEXT PRIMARY KEY NOT NULL,
    card_id TEXT NOT NULL,
    author TEXT,
    text TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (card_id) REFERENCES cards(id) ON DELETE CASCADE
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_cards_board_id ON cards(board_id);
CREATE INDEX IF NOT EXISTS idx_cards_status ON cards(status);
CREATE INDEX IF NOT EXISTS idx_cards_assigned_to ON cards(assigned_to);
CREATE INDEX IF NOT EXISTS idx_checklists_card_id ON checklists(card_id);
CREATE INDEX IF NOT EXISTS idx_checklist_items_checklist_id ON checklist_items(checklist_id);
CREATE INDEX IF NOT EXISTS idx_comments_card_id ON comments(card_id);
CREATE INDEX IF NOT EXISTS idx_card_tags_card_id ON card_tags(card_id);
