-- Milestone 3: Relations & Connections

CREATE TABLE relation_types (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    inverse_name TEXT,
    color TEXT,
    is_builtin BOOLEAN DEFAULT FALSE,
    created_at TEXT NOT NULL
);

CREATE TABLE relations (
    id TEXT PRIMARY KEY,
    source_page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    target_page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    relation_type_id TEXT NOT NULL REFERENCES relation_types(id) ON DELETE CASCADE,
    description TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_relations_source ON relations(source_page_id);
CREATE INDEX idx_relations_target ON relations(target_page_id);
CREATE INDEX idx_relations_type ON relations(relation_type_id);

-- Seed built-in relation types
INSERT INTO relation_types (id, name, inverse_name, color, is_builtin, created_at) VALUES
    ('rel-leader-of',   'Leader of',   'Led by',       '#C8A55C', TRUE, '2026-01-01T00:00:00Z'),
    ('rel-member-of',   'Member of',   'Has member',   '#8B5CB8', TRUE, '2026-01-01T00:00:00Z'),
    ('rel-resides-at',  'Resides at',  'Home of',      '#4A8C6A', TRUE, '2026-01-01T00:00:00Z'),
    ('rel-located-in',  'Located in',  'Contains',     '#4A8C6A', TRUE, '2026-01-01T00:00:00Z'),
    ('rel-ally-of',     'Ally of',     'Ally of',      '#5C8A5C', TRUE, '2026-01-01T00:00:00Z'),
    ('rel-enemy-of',    'Enemy of',    'Enemy of',     '#B85C5C', TRUE, '2026-01-01T00:00:00Z'),
    ('rel-mentor-of',   'Mentor of',   'Mentored by',  '#5C7AB8', TRUE, '2026-01-01T00:00:00Z'),
    ('rel-parent-of',   'Parent of',   'Child of',     '#B8955C', TRUE, '2026-01-01T00:00:00Z'),
    ('rel-owns',        'Owns',        'Owned by',     '#B8955C', TRUE, '2026-01-01T00:00:00Z'),
    ('rel-created-by',  'Created by',  'Created',      '#5CB8A8', TRUE, '2026-01-01T00:00:00Z');
