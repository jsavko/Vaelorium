-- Milestone 1: Wiki Engine Schema

-- Pages table: the core content unit
CREATE TABLE pages (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    icon TEXT,
    featured_image_path TEXT,
    parent_id TEXT REFERENCES pages(id) ON DELETE SET NULL,
    sort_order INTEGER DEFAULT 0,
    entity_type_id TEXT,
    visibility TEXT DEFAULT 'private',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by TEXT,
    updated_by TEXT
);

-- Page content stored as Yjs binary for CRDT sync
CREATE TABLE page_content (
    page_id TEXT PRIMARY KEY REFERENCES pages(id) ON DELETE CASCADE,
    yjs_state BLOB NOT NULL,
    yjs_version INTEGER DEFAULT 0
);

-- Backing table for FTS content
CREATE TABLE pages_fts_content (
    rowid INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL UNIQUE REFERENCES pages(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    text_content TEXT NOT NULL
);

-- Full-text search index
CREATE VIRTUAL TABLE pages_fts USING fts5(
    title,
    text_content,
    content='pages_fts_content',
    content_rowid='rowid'
);

-- Tags
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT
);

-- Page-tag associations
CREATE TABLE page_tags (
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (page_id, tag_id)
);

-- Wiki links (cross-references between pages)
CREATE TABLE wiki_links (
    source_page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    target_page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    link_text TEXT,
    PRIMARY KEY (source_page_id, target_page_id)
);

-- Version history snapshots
CREATE TABLE page_versions (
    id TEXT PRIMARY KEY,
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    yjs_snapshot BLOB NOT NULL,
    version_number INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT,
    summary TEXT
);

-- Indexes
CREATE INDEX idx_pages_parent ON pages(parent_id);

CREATE INDEX idx_pages_entity_type ON pages(entity_type_id);

CREATE INDEX idx_pages_updated ON pages(updated_at DESC);

CREATE INDEX idx_wiki_links_target ON wiki_links(target_page_id);

CREATE INDEX idx_page_versions_page ON page_versions(page_id, version_number DESC);

CREATE INDEX idx_pages_fts_content_page ON pages_fts_content(page_id);
