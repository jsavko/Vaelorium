//! Schema registry for sync-tracked tables.
//!
//! Each entry tells the engine and the journal helpers which columns to
//! capture for op generation and how to apply incoming ops. Adding a new
//! table is a matter of an entry here plus a `journal::emit_for_row` call
//! at each mutation point — the engine's apply path is generic over this.
//!
//! Special cases (binary BLOBs, complex composite keys) bypass the registry
//! and use hand-written paths in `engine.rs`. As of Phase 4a only
//! `page_content.yjs_state` (BLOB) is handled that way.

pub struct TableSchema {
    pub name: &'static str,
    pub columns: &'static [&'static str],
    pub primary_key: &'static str,
    /// Columns that should never trigger a conflict when both sides change
    /// them (in addition to the global meta-field denylist in
    /// `engine::is_meta_field`). For per-table audit columns beyond the
    /// global ones.
    pub meta_fields: &'static [&'static str],
}

pub struct TableRegistry {
    pub pages: TableSchema,
    pub entity_types: TableSchema,
    pub relations: TableSchema,
    pub relation_types: TableSchema,
    pub tags: TableSchema,
    // page_tags (M:N pivot) deferred to Phase 4b — composite primary key
    // needs custom apply logic that bypasses `apply_op_via_schema`.
    pub maps: TableSchema,
    pub map_pins: TableSchema,
    pub timelines: TableSchema,
    pub timeline_events: TableSchema,
}

pub static TABLES: TableRegistry = TableRegistry {
    pages: TableSchema {
        name: "pages",
        columns: &[
            "id", "title", "icon", "featured_image_path", "parent_id",
            "sort_order", "entity_type_id", "visibility",
            "created_at", "updated_at", "created_by", "updated_by",
        ],
        primary_key: "id",
        meta_fields: &[],
    },
    entity_types: TableSchema {
        name: "entity_types",
        columns: &[
            "id", "name", "icon", "color", "is_builtin",
            "sort_order", "created_at", "updated_at",
        ],
        primary_key: "id",
        meta_fields: &[],
    },
    relations: TableSchema {
        name: "relations",
        columns: &[
            "id", "source_page_id", "target_page_id",
            "relation_type_id", "description", "created_at",
        ],
        primary_key: "id",
        meta_fields: &[],
    },
    relation_types: TableSchema {
        name: "relation_types",
        columns: &["id", "name", "inverse_name", "color", "is_builtin", "created_at"],
        primary_key: "id",
        meta_fields: &[],
    },
    tags: TableSchema {
        name: "tags",
        columns: &["id", "name", "color"],
        primary_key: "id",
        meta_fields: &[],
    },
    maps: TableSchema {
        name: "maps",
        columns: &[
            "id", "title", "image_id", "parent_map_id",
            "sort_order", "created_at", "updated_at",
        ],
        primary_key: "id",
        meta_fields: &[],
    },
    map_pins: TableSchema {
        name: "map_pins",
        columns: &[
            "id", "map_id", "page_id", "label",
            "x", "y", "icon", "color", "created_at",
        ],
        primary_key: "id",
        meta_fields: &[],
    },
    timelines: TableSchema {
        name: "timelines",
        columns: &[
            "id", "name", "description", "sort_order",
            "created_at", "updated_at",
        ],
        primary_key: "id",
        meta_fields: &[],
    },
    timeline_events: TableSchema {
        name: "timeline_events",
        columns: &[
            "id", "timeline_id", "title", "description",
            "date", "end_date", "page_id", "color",
            "sort_order", "created_at",
        ],
        primary_key: "id",
        meta_fields: &[],
    },
};

/// Lookup helper used by the engine apply path.
pub fn schema_by_name(name: &str) -> Option<&'static TableSchema> {
    Some(match name {
        "pages" => &TABLES.pages,
        "entity_types" => &TABLES.entity_types,
        "relations" => &TABLES.relations,
        "relation_types" => &TABLES.relation_types,
        "tags" => &TABLES.tags,
        "maps" => &TABLES.maps,
        "map_pins" => &TABLES.map_pins,
        "timelines" => &TABLES.timelines,
        "timeline_events" => &TABLES.timeline_events,
        _ => return None,
    })
}
