-- Milestone 2: Entity Type System

-- Entity type definitions
CREATE TABLE entity_types (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    icon TEXT,
    color TEXT,
    is_builtin BOOLEAN DEFAULT FALSE,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Field definitions for entity types
CREATE TABLE entity_type_fields (
    id TEXT PRIMARY KEY,
    entity_type_id TEXT NOT NULL REFERENCES entity_types(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    field_type TEXT NOT NULL,
    sort_order INTEGER DEFAULT 0,
    is_required BOOLEAN DEFAULT FALSE,
    default_value TEXT,
    options TEXT,
    reference_type_id TEXT,
    created_at TEXT NOT NULL
);

-- Field values for specific pages
CREATE TABLE entity_field_values (
    id TEXT PRIMARY KEY,
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    field_id TEXT NOT NULL REFERENCES entity_type_fields(id) ON DELETE CASCADE,
    value TEXT,
    UNIQUE(page_id, field_id)
);

-- Indexes
CREATE INDEX idx_entity_type_fields_type ON entity_type_fields(entity_type_id, sort_order);
CREATE INDEX idx_entity_field_values_page ON entity_field_values(page_id);
CREATE INDEX idx_entity_field_values_field ON entity_field_values(field_id);

-- ============================================================
-- Seed built-in entity types (deterministic IDs)
-- ============================================================

INSERT INTO entity_types (id, name, icon, color, is_builtin, sort_order, created_at, updated_at) VALUES
    ('builtin-character',    'Character',    'shield',       '#B85C5C', TRUE, 1, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('builtin-location',     'Location',     'compass',      '#4A8C6A', TRUE, 2, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('builtin-quest',        'Quest',        'scroll',       '#5C7AB8', TRUE, 3, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('builtin-organisation', 'Organisation', 'users',        '#8B5CB8', TRUE, 4, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('builtin-item',         'Item',         'gem',          '#B8955C', TRUE, 5, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('builtin-creature',     'Creature',     'bug',          '#5CB8A8', TRUE, 6, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('builtin-event',        'Event',        'sparkles',     '#B85C8B', TRUE, 7, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z'),
    ('builtin-journal',      'Journal',      'notebook-pen', '#7A8C5C', TRUE, 8, '2026-01-01T00:00:00Z', '2026-01-01T00:00:00Z');

-- ============================================================
-- Seed default fields for each built-in type
-- ============================================================

-- Character fields
INSERT INTO entity_type_fields (id, entity_type_id, name, field_type, sort_order, is_required, default_value, options, reference_type_id, created_at) VALUES
    ('field-char-race',         'builtin-character', 'Race',         'text',           1, FALSE, NULL, NULL, NULL, '2026-01-01T00:00:00Z'),
    ('field-char-class',        'builtin-character', 'Class',        'text',           2, FALSE, NULL, NULL, NULL, '2026-01-01T00:00:00Z'),
    ('field-char-alignment',    'builtin-character', 'Alignment',    'select',         3, FALSE, NULL, '["Lawful Good","Neutral Good","Chaotic Good","Lawful Neutral","True Neutral","Chaotic Neutral","Lawful Evil","Neutral Evil","Chaotic Evil"]', NULL, '2026-01-01T00:00:00Z'),
    ('field-char-status',       'builtin-character', 'Status',       'select',         4, FALSE, '"Alive"', '["Alive","Dead","Missing","Unknown"]', NULL, '2026-01-01T00:00:00Z'),
    ('field-char-hp',           'builtin-character', 'HP',           'number',         5, FALSE, NULL, NULL, NULL, '2026-01-01T00:00:00Z'),
    ('field-char-location',     'builtin-character', 'Location',     'page_reference', 6, FALSE, NULL, NULL, 'builtin-location', '2026-01-01T00:00:00Z'),
    ('field-char-organisation', 'builtin-character', 'Organisation', 'page_reference', 7, FALSE, NULL, NULL, 'builtin-organisation', '2026-01-01T00:00:00Z');

-- Location fields
INSERT INTO entity_type_fields (id, entity_type_id, name, field_type, sort_order, is_required, default_value, options, reference_type_id, created_at) VALUES
    ('field-loc-type',       'builtin-location', 'Type',       'select',         1, FALSE, NULL, '["City","Town","Village","Fortress","Temple","Wilderness","Other"]', NULL, '2026-01-01T00:00:00Z'),
    ('field-loc-region',     'builtin-location', 'Region',     'page_reference', 2, FALSE, NULL, NULL, 'builtin-location', '2026-01-01T00:00:00Z'),
    ('field-loc-population', 'builtin-location', 'Population', 'number',         3, FALSE, NULL, NULL, NULL, '2026-01-01T00:00:00Z'),
    ('field-loc-climate',    'builtin-location', 'Climate',    'text',           4, FALSE, NULL, NULL, NULL, '2026-01-01T00:00:00Z');

-- Quest fields
INSERT INTO entity_type_fields (id, entity_type_id, name, field_type, sort_order, is_required, default_value, options, reference_type_id, created_at) VALUES
    ('field-quest-status',   'builtin-quest', 'Status',   'select',         1, FALSE, '"Active"', '["Active","Completed","Failed","Abandoned"]', NULL, '2026-01-01T00:00:00Z'),
    ('field-quest-priority', 'builtin-quest', 'Priority', 'select',         2, FALSE, '"Medium"', '["Low","Medium","High","Critical"]', NULL, '2026-01-01T00:00:00Z'),
    ('field-quest-giver',    'builtin-quest', 'Giver',    'page_reference', 3, FALSE, NULL, NULL, 'builtin-character', '2026-01-01T00:00:00Z'),
    ('field-quest-reward',   'builtin-quest', 'Reward',   'text',           4, FALSE, NULL, NULL, NULL, '2026-01-01T00:00:00Z');

-- Organisation fields
INSERT INTO entity_type_fields (id, entity_type_id, name, field_type, sort_order, is_required, default_value, options, reference_type_id, created_at) VALUES
    ('field-org-type',         'builtin-organisation', 'Type',         'select',         1, FALSE, NULL, '["Guild","Order","Government","Criminal","Religious","Other"]', NULL, '2026-01-01T00:00:00Z'),
    ('field-org-leader',       'builtin-organisation', 'Leader',       'page_reference', 2, FALSE, NULL, NULL, 'builtin-character', '2026-01-01T00:00:00Z'),
    ('field-org-headquarters', 'builtin-organisation', 'Headquarters', 'page_reference', 3, FALSE, NULL, NULL, 'builtin-location', '2026-01-01T00:00:00Z'),
    ('field-org-members',      'builtin-organisation', 'Members',      'number',         4, FALSE, NULL, NULL, NULL, '2026-01-01T00:00:00Z');

-- Item fields
INSERT INTO entity_type_fields (id, entity_type_id, name, field_type, sort_order, is_required, default_value, options, reference_type_id, created_at) VALUES
    ('field-item-type',   'builtin-item', 'Type',   'select',         1, FALSE, NULL, '["Weapon","Armor","Potion","Scroll","Wondrous","Other"]', NULL, '2026-01-01T00:00:00Z'),
    ('field-item-rarity', 'builtin-item', 'Rarity', 'select',         2, FALSE, NULL, '["Common","Uncommon","Rare","Very Rare","Legendary","Artifact"]', NULL, '2026-01-01T00:00:00Z'),
    ('field-item-value',  'builtin-item', 'Value',  'text',           3, FALSE, NULL, NULL, NULL, '2026-01-01T00:00:00Z'),
    ('field-item-owner',  'builtin-item', 'Owner',  'page_reference', 4, FALSE, NULL, NULL, 'builtin-character', '2026-01-01T00:00:00Z');

-- Creature fields
INSERT INTO entity_type_fields (id, entity_type_id, name, field_type, sort_order, is_required, default_value, options, reference_type_id, created_at) VALUES
    ('field-creature-type',      'builtin-creature', 'Type',             'select',         1, FALSE, NULL, '["Beast","Monstrosity","Undead","Fiend","Celestial","Dragon","Other"]', NULL, '2026-01-01T00:00:00Z'),
    ('field-creature-cr',        'builtin-creature', 'Challenge Rating', 'text',           2, FALSE, NULL, NULL, NULL, '2026-01-01T00:00:00Z'),
    ('field-creature-habitat',   'builtin-creature', 'Habitat',          'page_reference', 3, FALSE, NULL, NULL, 'builtin-location', '2026-01-01T00:00:00Z'),
    ('field-creature-alignment', 'builtin-creature', 'Alignment',        'select',         4, FALSE, NULL, '["Lawful Good","Neutral Good","Chaotic Good","Lawful Neutral","True Neutral","Chaotic Neutral","Lawful Evil","Neutral Evil","Chaotic Evil"]', NULL, '2026-01-01T00:00:00Z');

-- Event fields
INSERT INTO entity_type_fields (id, entity_type_id, name, field_type, sort_order, is_required, default_value, options, reference_type_id, created_at) VALUES
    ('field-event-date',         'builtin-event', 'Date',         'text',           1, FALSE, NULL, NULL, NULL, '2026-01-01T00:00:00Z'),
    ('field-event-duration',     'builtin-event', 'Duration',     'text',           2, FALSE, NULL, NULL, NULL, '2026-01-01T00:00:00Z'),
    ('field-event-location',     'builtin-event', 'Location',     'page_reference', 3, FALSE, NULL, NULL, 'builtin-location', '2026-01-01T00:00:00Z'),
    ('field-event-significance', 'builtin-event', 'Significance', 'select',         4, FALSE, NULL, '["Minor","Major","World-changing"]', NULL, '2026-01-01T00:00:00Z');

-- Journal fields
INSERT INTO entity_type_fields (id, entity_type_id, name, field_type, sort_order, is_required, default_value, options, reference_type_id, created_at) VALUES
    ('field-journal-session',  'builtin-journal', 'Session Number', 'number',         1, FALSE, NULL, NULL, NULL, '2026-01-01T00:00:00Z'),
    ('field-journal-date',     'builtin-journal', 'Date Played',    'text',           2, FALSE, NULL, NULL, NULL, '2026-01-01T00:00:00Z'),
    ('field-journal-dm',       'builtin-journal', 'DM',             'page_reference', 3, FALSE, NULL, NULL, 'builtin-character', '2026-01-01T00:00:00Z'),
    ('field-journal-location', 'builtin-journal', 'Location',       'page_reference', 4, FALSE, NULL, NULL, 'builtin-location', '2026-01-01T00:00:00Z');
