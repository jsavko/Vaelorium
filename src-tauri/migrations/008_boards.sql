-- Milestone 6: Boards (Whiteboards)

CREATE TABLE boards (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE board_cards (
    id TEXT PRIMARY KEY,
    board_id TEXT NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    page_id TEXT REFERENCES pages(id) ON DELETE SET NULL,
    content TEXT,
    x REAL NOT NULL DEFAULT 0,
    y REAL NOT NULL DEFAULT 0,
    width REAL NOT NULL DEFAULT 200,
    height REAL NOT NULL DEFAULT 120,
    color TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE board_connectors (
    id TEXT PRIMARY KEY,
    board_id TEXT NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    source_card_id TEXT NOT NULL REFERENCES board_cards(id) ON DELETE CASCADE,
    target_card_id TEXT NOT NULL REFERENCES board_cards(id) ON DELETE CASCADE,
    label TEXT,
    color TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_board_cards_board ON board_cards(board_id);
CREATE INDEX idx_board_connectors_board ON board_connectors(board_id);
