/**
 * Bridge between frontend and backend.
 * In Tauri: uses invoke() to call Rust commands.
 * In browser: uses mock data for UI development/testing.
 */

let isTauri = false

try {
  // @ts-ignore
  isTauri = !!window.__TAURI_INTERNALS__
} catch {
  isTauri = false
}

export { isTauri }

export async function callCommand<T>(command: string, args?: Record<string, any>): Promise<T> {
  if (isTauri) {
    const { invoke } = await import('@tauri-apps/api/core')
    return invoke(command, args)
  } else {
    return mockCommand(command, args)
  }
}

// In-memory mock database for browser testing
const mockDb = {
  pages: new Map<string, any>(),
  pageContent: new Map<string, number[]>(),
  tags: new Map<string, any>(),
  pageTags: new Map<string, Set<string>>(),
  wikiLinks: new Map<string, any[]>(),
  versions: new Map<string, any[]>(),
  entityTypes: new Map<string, any>(),
  entityTypeFields: new Map<string, any>(),
  entityFieldValues: new Map<string, any>(),
  nextSortOrder: 1,
  images: new Map<string, any>(),
  relationTypes: new Map<string, any>(),
  relations: new Map<string, any>(),
  maps: new Map<string, any>(),
  mapPins: new Map<string, any>(),
  timelines: new Map<string, any>(),
  timelineEvents: new Map<string, any>(),
  boards: new Map<string, any>(),
  boardCards: new Map<string, any>(),
  boardConnectors: new Map<string, any>(),
  tomeOpen: true,
  tomeMeta: { name: 'Dev Tome', description: 'Browser development tome', created_at: '2026-01-01T00:00:00Z' } as any,
  recentTomes: [
    { path: '/mock/dev-tome.tome', name: 'Dev Tome', description: 'Browser development tome', last_opened: new Date().toISOString() },
  ] as any[],
}

function resetMockDb() {
  mockDb.pages.clear()
  mockDb.pageContent.clear()
  mockDb.tags.clear()
  mockDb.pageTags.clear()
  mockDb.wikiLinks.clear()
  mockDb.versions.clear()
  mockDb.entityTypes.clear()
  mockDb.entityTypeFields.clear()
  mockDb.entityFieldValues.clear()
  mockDb.nextSortOrder = 1
  seedBuiltinEntityTypes()
}

// Seed built-in entity types
function seedBuiltinEntityTypes() {
  const ts = '2026-01-01T00:00:00Z'
  const builtinTypes = [
    { id: 'builtin-character', name: 'Character', icon: 'shield', color: '#B85C5C', sort_order: 1 },
    { id: 'builtin-location', name: 'Location', icon: 'compass', color: '#4A8C6A', sort_order: 2 },
    { id: 'builtin-quest', name: 'Quest', icon: 'scroll', color: '#5C7AB8', sort_order: 3 },
    { id: 'builtin-organisation', name: 'Organisation', icon: 'users', color: '#8B5CB8', sort_order: 4 },
    { id: 'builtin-item', name: 'Item', icon: 'gem', color: '#B8955C', sort_order: 5 },
    { id: 'builtin-creature', name: 'Creature', icon: 'bug', color: '#5CB8A8', sort_order: 6 },
    { id: 'builtin-event', name: 'Event', icon: 'sparkles', color: '#B85C8B', sort_order: 7 },
    { id: 'builtin-journal', name: 'Journal', icon: 'notebook-pen', color: '#7A8C5C', sort_order: 8 },
  ]
  for (const t of builtinTypes) {
    mockDb.entityTypes.set(t.id, { ...t, is_builtin: true, created_at: ts, updated_at: ts })
  }

  const builtinFields: Array<{ id: string; entity_type_id: string; name: string; field_type: string; sort_order: number; options?: string; default_value?: string; reference_type_id?: string }> = [
    // Character
    { id: 'field-char-race', entity_type_id: 'builtin-character', name: 'Race', field_type: 'text', sort_order: 1 },
    { id: 'field-char-class', entity_type_id: 'builtin-character', name: 'Class', field_type: 'text', sort_order: 2 },
    { id: 'field-char-alignment', entity_type_id: 'builtin-character', name: 'Alignment', field_type: 'select', sort_order: 3, options: '["Lawful Good","Neutral Good","Chaotic Good","Lawful Neutral","True Neutral","Chaotic Neutral","Lawful Evil","Neutral Evil","Chaotic Evil"]' },
    { id: 'field-char-status', entity_type_id: 'builtin-character', name: 'Status', field_type: 'select', sort_order: 4, default_value: '"Alive"', options: '["Alive","Dead","Missing","Unknown"]' },
    { id: 'field-char-hp', entity_type_id: 'builtin-character', name: 'HP', field_type: 'number', sort_order: 5 },
    { id: 'field-char-location', entity_type_id: 'builtin-character', name: 'Location', field_type: 'page_reference', sort_order: 6, reference_type_id: 'builtin-location' },
    { id: 'field-char-organisation', entity_type_id: 'builtin-character', name: 'Organisation', field_type: 'page_reference', sort_order: 7, reference_type_id: 'builtin-organisation' },
    // Location
    { id: 'field-loc-type', entity_type_id: 'builtin-location', name: 'Type', field_type: 'select', sort_order: 1, options: '["City","Town","Village","Fortress","Temple","Wilderness","Other"]' },
    { id: 'field-loc-region', entity_type_id: 'builtin-location', name: 'Region', field_type: 'page_reference', sort_order: 2, reference_type_id: 'builtin-location' },
    { id: 'field-loc-population', entity_type_id: 'builtin-location', name: 'Population', field_type: 'number', sort_order: 3 },
    { id: 'field-loc-climate', entity_type_id: 'builtin-location', name: 'Climate', field_type: 'text', sort_order: 4 },
    // Quest
    { id: 'field-quest-status', entity_type_id: 'builtin-quest', name: 'Status', field_type: 'select', sort_order: 1, default_value: '"Active"', options: '["Active","Completed","Failed","Abandoned"]' },
    { id: 'field-quest-priority', entity_type_id: 'builtin-quest', name: 'Priority', field_type: 'select', sort_order: 2, default_value: '"Medium"', options: '["Low","Medium","High","Critical"]' },
    { id: 'field-quest-giver', entity_type_id: 'builtin-quest', name: 'Giver', field_type: 'page_reference', sort_order: 3, reference_type_id: 'builtin-character' },
    { id: 'field-quest-reward', entity_type_id: 'builtin-quest', name: 'Reward', field_type: 'text', sort_order: 4 },
    // Organisation
    { id: 'field-org-type', entity_type_id: 'builtin-organisation', name: 'Type', field_type: 'select', sort_order: 1, options: '["Guild","Order","Government","Criminal","Religious","Other"]' },
    { id: 'field-org-leader', entity_type_id: 'builtin-organisation', name: 'Leader', field_type: 'page_reference', sort_order: 2, reference_type_id: 'builtin-character' },
    { id: 'field-org-headquarters', entity_type_id: 'builtin-organisation', name: 'Headquarters', field_type: 'page_reference', sort_order: 3, reference_type_id: 'builtin-location' },
    { id: 'field-org-members', entity_type_id: 'builtin-organisation', name: 'Members', field_type: 'number', sort_order: 4 },
    // Item
    { id: 'field-item-type', entity_type_id: 'builtin-item', name: 'Type', field_type: 'select', sort_order: 1, options: '["Weapon","Armor","Potion","Scroll","Wondrous","Other"]' },
    { id: 'field-item-rarity', entity_type_id: 'builtin-item', name: 'Rarity', field_type: 'select', sort_order: 2, options: '["Common","Uncommon","Rare","Very Rare","Legendary","Artifact"]' },
    { id: 'field-item-value', entity_type_id: 'builtin-item', name: 'Value', field_type: 'text', sort_order: 3 },
    { id: 'field-item-owner', entity_type_id: 'builtin-item', name: 'Owner', field_type: 'page_reference', sort_order: 4, reference_type_id: 'builtin-character' },
    // Creature
    { id: 'field-creature-type', entity_type_id: 'builtin-creature', name: 'Type', field_type: 'select', sort_order: 1, options: '["Beast","Monstrosity","Undead","Fiend","Celestial","Dragon","Other"]' },
    { id: 'field-creature-cr', entity_type_id: 'builtin-creature', name: 'Challenge Rating', field_type: 'text', sort_order: 2 },
    { id: 'field-creature-habitat', entity_type_id: 'builtin-creature', name: 'Habitat', field_type: 'page_reference', sort_order: 3, reference_type_id: 'builtin-location' },
    { id: 'field-creature-alignment', entity_type_id: 'builtin-creature', name: 'Alignment', field_type: 'select', sort_order: 4, options: '["Lawful Good","Neutral Good","Chaotic Good","Lawful Neutral","True Neutral","Chaotic Neutral","Lawful Evil","Neutral Evil","Chaotic Evil"]' },
    // Event
    { id: 'field-event-date', entity_type_id: 'builtin-event', name: 'Date', field_type: 'text', sort_order: 1 },
    { id: 'field-event-duration', entity_type_id: 'builtin-event', name: 'Duration', field_type: 'text', sort_order: 2 },
    { id: 'field-event-location', entity_type_id: 'builtin-event', name: 'Location', field_type: 'page_reference', sort_order: 3, reference_type_id: 'builtin-location' },
    { id: 'field-event-significance', entity_type_id: 'builtin-event', name: 'Significance', field_type: 'select', sort_order: 4, options: '["Minor","Major","World-changing"]' },
    // Journal
    { id: 'field-journal-session', entity_type_id: 'builtin-journal', name: 'Session Number', field_type: 'number', sort_order: 1 },
    { id: 'field-journal-date', entity_type_id: 'builtin-journal', name: 'Date Played', field_type: 'text', sort_order: 2 },
    { id: 'field-journal-dm', entity_type_id: 'builtin-journal', name: 'DM', field_type: 'page_reference', sort_order: 3, reference_type_id: 'builtin-character' },
    { id: 'field-journal-location', entity_type_id: 'builtin-journal', name: 'Location', field_type: 'page_reference', sort_order: 4, reference_type_id: 'builtin-location' },
  ]
  for (const f of builtinFields) {
    mockDb.entityTypeFields.set(f.id, {
      ...f,
      is_required: false,
      default_value: f.default_value || null,
      options: f.options || null,
      reference_type_id: f.reference_type_id || null,
      created_at: ts,
    })
  }
}

seedBuiltinEntityTypes()

// Seed built-in relation types
const builtinRelTypes = [
  { id: 'rel-leader-of', name: 'Leader of', inverse_name: 'Led by', color: '#C8A55C' },
  { id: 'rel-member-of', name: 'Member of', inverse_name: 'Has member', color: '#8B5CB8' },
  { id: 'rel-resides-at', name: 'Resides at', inverse_name: 'Home of', color: '#4A8C6A' },
  { id: 'rel-located-in', name: 'Located in', inverse_name: 'Contains', color: '#4A8C6A' },
  { id: 'rel-ally-of', name: 'Ally of', inverse_name: 'Ally of', color: '#5C8A5C' },
  { id: 'rel-enemy-of', name: 'Enemy of', inverse_name: 'Enemy of', color: '#B85C5C' },
  { id: 'rel-mentor-of', name: 'Mentor of', inverse_name: 'Mentored by', color: '#5C7AB8' },
  { id: 'rel-parent-of', name: 'Parent of', inverse_name: 'Child of', color: '#B8955C' },
  { id: 'rel-owns', name: 'Owns', inverse_name: 'Owned by', color: '#B8955C' },
  { id: 'rel-created-by', name: 'Created by', inverse_name: 'Created', color: '#5CB8A8' },
]
for (const t of builtinRelTypes) {
  mockDb.relationTypes.set(t.id, { ...t, is_builtin: true, created_at: '2026-01-01T00:00:00Z' })
}

function uuid() {
  return crypto.randomUUID()
}

function now() {
  return new Date().toISOString()
}

async function mockCommand(command: string, args?: any): Promise<any> {
  switch (command) {
    case 'create_page': {
      const { input } = args
      const id = uuid()
      const page = {
        id,
        title: input.title,
        icon: input.icon || null,
        featured_image_path: null,
        parent_id: input.parent_id || null,
        sort_order: mockDb.nextSortOrder++,
        entity_type_id: input.entity_type_id || null,
        visibility: 'private',
        created_at: now(),
        updated_at: now(),
        created_by: null,
        updated_by: null,
      }
      mockDb.pages.set(id, page)
      mockDb.pageContent.set(id, [])
      return page
    }

    case 'get_page': {
      const page = mockDb.pages.get(args.id)
      if (!page) throw new Error('Page not found')
      return page
    }

    case 'update_page': {
      const page = mockDb.pages.get(args.id)
      if (!page) throw new Error('Page not found')
      const { input } = args
      if (input.title !== undefined) page.title = input.title
      if (input.icon !== undefined) page.icon = input.icon
      if (input.parent_id !== undefined) page.parent_id = input.parent_id
      if (input.sort_order !== undefined) page.sort_order = input.sort_order
      if (input.visibility !== undefined) page.visibility = input.visibility
      if (input.featured_image_path !== undefined) page.featured_image_path = input.featured_image_path
      if (input.entity_type_id !== undefined) page.entity_type_id = input.entity_type_id
      page.updated_at = now()
      return page
    }

    case 'delete_page': {
      mockDb.pages.delete(args.id)
      mockDb.pageContent.delete(args.id)
      return null
    }

    case 'list_pages': {
      return Array.from(mockDb.pages.values()).sort((a, b) => a.sort_order - b.sort_order)
    }

    case 'list_pages_by_type': {
      const typeId = args.entityTypeId || args.entity_type_id
      return Array.from(mockDb.pages.values())
        .filter((p) => p.entity_type_id === typeId)
        .sort((a: any, b: any) => a.title.localeCompare(b.title))
    }

    case 'get_page_tree': {
      const pages = Array.from(mockDb.pages.values())
      return pages.map((p) => {
        const childCount = pages.filter((c) => c.parent_id === p.id).length
        return {
          id: p.id,
          title: p.title,
          icon: p.icon,
          entity_type_id: p.entity_type_id,
          parent_id: p.parent_id,
          sort_order: p.sort_order,
          children_count: childCount,
        }
      }).sort((a, b) => a.sort_order - b.sort_order)
    }

    case 'save_page_content': {
      mockDb.pageContent.set(args.pageId, args.yjsState)
      const page = mockDb.pages.get(args.pageId)
      if (page) page.updated_at = now()
      return null
    }

    case 'get_page_content': {
      return mockDb.pageContent.get(args.pageId) || []
    }

    case 'reorder_pages': {
      for (const m of args.moves) {
        const page = mockDb.pages.get(m.id)
        if (page) {
          page.parent_id = m.parent_id
          page.sort_order = m.sort_order
        }
      }
      return null
    }

    case 'save_wiki_links': {
      const srcId = args.sourcePageId || args.source_page_id
      mockDb.wikiLinks.set(srcId, args.links)
      return null
    }

    case 'get_backlinks': {
      const targetId = args.pageId || args.page_id
      const backlinks: any[] = []
      for (const [sourceId, links] of mockDb.wikiLinks) {
        for (const link of links) {
          if (link.target_page_id === targetId) {
            const page = mockDb.pages.get(sourceId)
            if (page) {
              backlinks.push({ page_id: sourceId, title: page.title, entity_type_id: page.entity_type_id })
            }
          }
        }
      }
      return backlinks
    }

    case 'search_pages': {
      const q = (args.query || '').toLowerCase()
      if (!q) return []
      return Array.from(mockDb.pages.values())
        .filter((p) => p.title.toLowerCase().includes(q))
        .map((p) => ({ page_id: p.id, title: p.title, entity_type_id: p.entity_type_id, snippet: null }))
        .slice(0, 20)
    }

    case 'update_search_index':
      return null

    case 'update_page_title_in_links': {
      const { pageId, oldTitle, newTitle } = args
      // Find all pages that link to this page and update their content
      for (const [sourceId, links] of mockDb.wikiLinks) {
        let hasLink = false
        for (const link of links) {
          if (link.target_page_id === pageId) {
            // Update link_text in the wiki_links entry
            link.link_text = newTitle
            hasLink = true
          }
        }
        if (hasLink) {
          // Update the stored Yjs content for this source page
          // In the mock, content is stored as Yjs binary — we can't easily modify it
          // But we can mark it as needing update. For the mock, we'll store
          // a mapping of pending title updates that the editor checks on load.
          if (!(mockDb as any).pendingTitleUpdates) (mockDb as any).pendingTitleUpdates = new Map()
          const updates = (mockDb as any).pendingTitleUpdates.get(sourceId) || []
          updates.push({ pageId, oldTitle, newTitle })
          ;(mockDb as any).pendingTitleUpdates.set(sourceId, updates)
        }
      }
      return null
    }

    case 'create_tag': {
      const tag = { id: uuid(), name: args.name, color: args.color || null }
      mockDb.tags.set(tag.id, tag)
      return tag
    }

    case 'list_tags':
      return Array.from(mockDb.tags.values())

    case 'add_tag_to_page': {
      if (!mockDb.pageTags.has(args.page_id)) mockDb.pageTags.set(args.page_id, new Set())
      mockDb.pageTags.get(args.page_id)!.add(args.tag_id)
      return null
    }

    case 'remove_tag_from_page': {
      mockDb.pageTags.get(args.page_id)?.delete(args.tag_id)
      return null
    }

    case 'get_page_tags': {
      const tagIds = mockDb.pageTags.get(args.page_id) || new Set()
      return Array.from(tagIds).map((id) => mockDb.tags.get(id)).filter(Boolean)
    }

    case 'create_version': {
      const v = {
        id: uuid(),
        page_id: args.pageId,
        version_number: (mockDb.versions.get(args.pageId)?.length || 0) + 1,
        created_at: now(),
        created_by: null,
        summary: args.summary || null,
      }
      if (!mockDb.versions.has(args.pageId)) mockDb.versions.set(args.pageId, [])
      mockDb.versions.get(args.pageId)!.push(v)
      return v
    }

    case 'list_versions':
      return (mockDb.versions.get(args.pageId) || []).reverse()

    case 'get_version_snapshot':
      return []

    // ── Entity Types ──

    case 'list_entity_types': {
      return Array.from(mockDb.entityTypes.values()).sort(
        (a: any, b: any) => a.sort_order - b.sort_order,
      )
    }

    case 'get_entity_type': {
      const et = mockDb.entityTypes.get(args.id)
      if (!et) throw new Error('Entity type not found')
      return et
    }

    case 'create_entity_type': {
      const id = uuid()
      const maxSort = Array.from(mockDb.entityTypes.values()).reduce(
        (max: number, t: any) => Math.max(max, t.sort_order),
        0,
      )
      const et = {
        id,
        name: args.name,
        icon: args.icon || null,
        color: args.color || null,
        is_builtin: false,
        sort_order: maxSort + 1,
        created_at: now(),
        updated_at: now(),
      }
      mockDb.entityTypes.set(id, et)
      return et
    }

    case 'update_entity_type': {
      const et = mockDb.entityTypes.get(args.id)
      if (!et) throw new Error('Entity type not found')
      if (args.name !== undefined) et.name = args.name
      if (args.icon !== undefined) et.icon = args.icon
      if (args.color !== undefined) et.color = args.color
      et.updated_at = now()
      return et
    }

    case 'delete_entity_type': {
      const et = mockDb.entityTypes.get(args.id)
      if (!et) throw new Error('Entity type not found')
      // Clear entity_type_id from pages using this type
      for (const page of mockDb.pages.values()) {
        if (page.entity_type_id === args.id) page.entity_type_id = null
      }
      // Delete fields belonging to this type
      for (const [fId, field] of mockDb.entityTypeFields) {
        if (field.entity_type_id === args.id) {
          // Delete field values for this field
          for (const [vKey, val] of mockDb.entityFieldValues) {
            if (val.field_id === fId) mockDb.entityFieldValues.delete(vKey)
          }
          mockDb.entityTypeFields.delete(fId)
        }
      }
      mockDb.entityTypes.delete(args.id)
      return null
    }

    // ── Entity Fields ──

    case 'list_entity_type_fields': {
      const etId = args.entityTypeId || args.entity_type_id
      return Array.from(mockDb.entityTypeFields.values())
        .filter((f: any) => f.entity_type_id === etId)
        .sort((a: any, b: any) => a.sort_order - b.sort_order)
    }

    case 'create_entity_type_field': {
      const id = uuid()
      const etId = args.entityTypeId || args.entity_type_id
      const existing = Array.from(mockDb.entityTypeFields.values()).filter(
        (f: any) => f.entity_type_id === etId,
      )
      const maxSort = existing.reduce((max: number, f: any) => Math.max(max, f.sort_order), 0)
      const field = {
        id,
        entity_type_id: etId,
        name: args.name,
        field_type: args.fieldType || args.field_type,
        sort_order: maxSort + 1,
        is_required: args.isRequired || args.is_required || false,
        default_value: args.defaultValue || args.default_value || null,
        options: args.options || null,
        reference_type_id: args.referenceTypeId || args.reference_type_id || null,
        created_at: now(),
      }
      mockDb.entityTypeFields.set(id, field)
      return field
    }

    case 'update_entity_type_field': {
      const field = mockDb.entityTypeFields.get(args.id)
      if (!field) throw new Error('Field not found')
      if ((args.name) !== undefined) field.name = args.name
      if ((args.fieldType || args.field_type) !== undefined) field.field_type = args.fieldType || args.field_type
      if ((args.isRequired ?? args.is_required) !== undefined) field.is_required = args.isRequired ?? args.is_required
      if ((args.defaultValue || args.default_value) !== undefined) field.default_value = args.defaultValue || args.default_value
      if ((args.options) !== undefined) field.options = args.options
      if ((args.referenceTypeId || args.reference_type_id) !== undefined) field.reference_type_id = args.referenceTypeId || args.reference_type_id
      return field
    }

    case 'delete_entity_type_field': {
      // Delete field values for this field
      for (const [vKey, val] of mockDb.entityFieldValues) {
        if (val.field_id === args.id) mockDb.entityFieldValues.delete(vKey)
      }
      mockDb.entityTypeFields.delete(args.id)
      return null
    }

    case 'reorder_entity_type_fields': {
      for (const m of args.moves) {
        const field = mockDb.entityTypeFields.get(m.id)
        if (field) field.sort_order = m.sort_order
      }
      return null
    }

    // ── Field Values ──

    case 'get_page_field_values': {
      const pgId = args.pageId || args.page_id
      return Array.from(mockDb.entityFieldValues.values()).filter(
        (v: any) => v.page_id === pgId,
      )
    }

    case 'set_field_value': {
      const sfPageId = args.pageId || args.page_id
      const sfFieldId = args.fieldId || args.field_id
      const key = `${sfPageId}:${sfFieldId}`
      const existing = mockDb.entityFieldValues.get(key)
      const fv = {
        id: existing?.id || uuid(),
        page_id: sfPageId,
        field_id: sfFieldId,
        value: args.value ?? null,
      }
      mockDb.entityFieldValues.set(key, fv)
      return fv
    }

    case 'delete_field_value': {
      const dfPageId = args.pageId || args.page_id
      const dfFieldId = args.fieldId || args.field_id
      const key = `${dfPageId}:${dfFieldId}`
      mockDb.entityFieldValues.delete(key)
      return null
    }

    case 'query_pages_by_field': {
      const qFieldId = args.fieldId || args.field_id
      const matchingPageIds = Array.from(mockDb.entityFieldValues.values())
        .filter((v: any) => v.field_id === qFieldId && v.value === args.value)
        .map((v: any) => v.page_id)
      return Array.from(mockDb.pages.values())
        .filter((p: any) => matchingPageIds.includes(p.id))
        .map((p: any) => ({
          id: p.id,
          title: p.title,
          icon: p.icon,
          entity_type_id: p.entity_type_id,
        }))
        .sort((a: any, b: any) => a.title.localeCompare(b.title))
    }

    // ── Export/Import ──

    case 'export_tome_json': {
      const exp = {
        version: '1.0',
        pages: Array.from(mockDb.pages.values()).map((p: any) => ({ ...p, content_base64: null })),
        entity_types: Array.from(mockDb.entityTypes.values()),
        tags: Array.from(mockDb.tags.values()),
      }
      return JSON.stringify(exp, null, 2)
    }

    case 'export_tome_markdown': { return null }
    case 'import_markdown_folder': { return { pages_imported: 0, errors: ['Not supported in browser mock'] } }
    case 'import_json': {
      try {
        const data = JSON.parse(args.json)
        let count = 0
        if (data.pages) {
          for (const p of data.pages) {
            const id = uuid()
            mockDb.pages.set(id, { ...p, id, sort_order: mockDb.nextSortOrder++ })
            count++
          }
        }
        return { pages_imported: count, errors: [] }
      } catch (e: any) {
        return { pages_imported: 0, errors: [e.message] }
      }
    }

    // ── Boards ──

    case 'create_board': { const id = uuid(); const b = { id, name: args.name, sort_order: 0, created_at: now(), updated_at: now() }; mockDb.boards.set(id, b); return b }
    case 'list_boards': { return Array.from(mockDb.boards.values()).sort((a: any, b: any) => a.name.localeCompare(b.name)) }
    case 'delete_board': { mockDb.boards.delete(args.id); for (const [k, c] of mockDb.boardCards) { if (c.board_id === args.id) mockDb.boardCards.delete(k) } for (const [k, c] of mockDb.boardConnectors) { if (c.board_id === args.id) mockDb.boardConnectors.delete(k) } return null }
    case 'create_card': { const id = uuid(); const c = { id, board_id: args.boardId, page_id: args.pageId || null, content: args.content || null, x: args.x, y: args.y, width: 200, height: 120, color: args.color || null, created_at: now() }; mockDb.boardCards.set(id, c); return c }
    case 'update_card': { const c = mockDb.boardCards.get(args.id); if (!c) throw new Error('Card not found'); if (args.x !== undefined) c.x = args.x; if (args.y !== undefined) c.y = args.y; if (args.content !== undefined) c.content = args.content; if (args.pageId !== undefined) c.page_id = args.pageId; if (args.color !== undefined) c.color = args.color; if (args.width !== undefined) c.width = args.width; if (args.height !== undefined) c.height = args.height; return c }
    case 'delete_card': { mockDb.boardCards.delete(args.id); for (const [k, c] of mockDb.boardConnectors) { if (c.source_card_id === args.id || c.target_card_id === args.id) mockDb.boardConnectors.delete(k) } return null }
    case 'get_board_cards': { return Array.from(mockDb.boardCards.values()).filter((c: any) => c.board_id === args.boardId) }
    case 'create_connector': { const id = uuid(); const c = { id, board_id: args.boardId, source_card_id: args.sourceCardId, target_card_id: args.targetCardId, label: args.label || null, color: args.color || null, created_at: now() }; mockDb.boardConnectors.set(id, c); return c }
    case 'delete_connector': { mockDb.boardConnectors.delete(args.id); return null }
    case 'get_board_connectors': { return Array.from(mockDb.boardConnectors.values()).filter((c: any) => c.board_id === args.boardId) }

    // ── Timelines ──

    case 'create_timeline': {
      const id = uuid()
      const tl = { id, name: args.name, description: args.description || null, sort_order: 0, created_at: now(), updated_at: now() }
      mockDb.timelines.set(id, tl)
      return tl
    }

    case 'list_timelines': {
      return Array.from(mockDb.timelines.values()).sort((a: any, b: any) => a.name.localeCompare(b.name))
    }

    case 'delete_timeline': {
      mockDb.timelines.delete(args.id)
      for (const [eId, evt] of mockDb.timelineEvents) {
        if (evt.timeline_id === args.id) mockDb.timelineEvents.delete(eId)
      }
      return null
    }

    case 'create_timeline_event': {
      const id = uuid()
      const evt = { id, timeline_id: args.timelineId, title: args.title, description: args.description || null, date: args.date, end_date: args.endDate || null, page_id: args.pageId || null, color: args.color || null, sort_order: 0, created_at: now() }
      mockDb.timelineEvents.set(id, evt)
      return evt
    }

    case 'update_timeline_event': {
      const evt = mockDb.timelineEvents.get(args.id)
      if (!evt) throw new Error('Event not found')
      if (args.title !== undefined) evt.title = args.title
      if (args.date !== undefined) evt.date = args.date
      if (args.description !== undefined) evt.description = args.description
      if (args.endDate !== undefined) evt.end_date = args.endDate
      if (args.pageId !== undefined) evt.page_id = args.pageId
      if (args.color !== undefined) evt.color = args.color
      return evt
    }

    case 'delete_timeline_event': {
      mockDb.timelineEvents.delete(args.id)
      return null
    }

    case 'get_timeline_events': {
      return Array.from(mockDb.timelineEvents.values())
        .filter((e: any) => e.timeline_id === args.timelineId)
        .sort((a: any, b: any) => a.date.localeCompare(b.date))
    }

    // ── Maps ──

    case 'create_map': {
      const id = uuid()
      const map = { id, title: args.title, image_id: args.imageId || null, parent_map_id: null, sort_order: 0, created_at: now(), updated_at: now() }
      mockDb.maps.set(id, map)
      return map
    }

    case 'list_maps': {
      return Array.from(mockDb.maps.values()).sort((a: any, b: any) => a.title.localeCompare(b.title))
    }

    case 'get_map': {
      const map = mockDb.maps.get(args.id)
      if (!map) throw new Error('Map not found')
      return map
    }

    case 'delete_map': {
      mockDb.maps.delete(args.id)
      for (const [pId, pin] of mockDb.mapPins) {
        if (pin.map_id === args.id) mockDb.mapPins.delete(pId)
      }
      return null
    }

    case 'create_pin': {
      const id = uuid()
      const pin = { id, map_id: args.mapId, page_id: args.pageId || null, label: args.label || null, x: args.x, y: args.y, icon: args.icon || 'map-pin', color: args.color || null, created_at: now() }
      mockDb.mapPins.set(id, pin)
      return pin
    }

    case 'update_pin': {
      const pin = mockDb.mapPins.get(args.id)
      if (!pin) throw new Error('Pin not found')
      if (args.x !== undefined) pin.x = args.x
      if (args.y !== undefined) pin.y = args.y
      if (args.pageId !== undefined) pin.page_id = args.pageId
      if (args.label !== undefined) pin.label = args.label
      if (args.color !== undefined) pin.color = args.color
      return pin
    }

    case 'delete_pin': {
      mockDb.mapPins.delete(args.id)
      return null
    }

    case 'get_map_pins': {
      return Array.from(mockDb.mapPins.values()).filter((p: any) => p.map_id === args.mapId)
    }

    // ── Relations ──

    case 'list_relation_types': {
      return Array.from(mockDb.relationTypes.values()).sort((a: any, b: any) => a.name.localeCompare(b.name))
    }

    case 'create_relation_type': {
      const id = uuid()
      const rt = { id, name: args.name, inverse_name: args.inverseName || null, color: args.color || null, is_builtin: false, created_at: now() }
      mockDb.relationTypes.set(id, rt)
      return rt
    }

    case 'create_relation': {
      const id = uuid()
      const rel = { id, source_page_id: args.sourcePageId, target_page_id: args.targetPageId, relation_type_id: args.relationTypeId, description: args.description || null, created_at: now() }
      mockDb.relations.set(id, rel)
      return rel
    }

    case 'delete_relation': {
      mockDb.relations.delete(args.id)
      return null
    }

    case 'get_page_relations': {
      const pgId = args.pageId
      const results: any[] = []
      for (const rel of mockDb.relations.values()) {
        if (rel.source_page_id === pgId) {
          const page = mockDb.pages.get(rel.target_page_id)
          const rt = mockDb.relationTypes.get(rel.relation_type_id)
          if (page && rt) {
            results.push({ id: rel.id, page_id: page.id, page_title: page.title, page_icon: page.icon, page_entity_type_id: page.entity_type_id, relation_type_id: rel.relation_type_id, relation_label: rt.name, description: rel.description, direction: 'outgoing' })
          }
        }
        if (rel.target_page_id === pgId) {
          const page = mockDb.pages.get(rel.source_page_id)
          const rt = mockDb.relationTypes.get(rel.relation_type_id)
          if (page && rt) {
            results.push({ id: rel.id, page_id: page.id, page_title: page.title, page_icon: page.icon, page_entity_type_id: page.entity_type_id, relation_type_id: rel.relation_type_id, relation_label: rt.inverse_name || rt.name, description: rel.description, direction: 'incoming' })
          }
        }
      }
      return results.sort((a: any, b: any) => a.page_title.localeCompare(b.page_title))
    }

    case 'list_all_relations': {
      return Array.from(mockDb.relations.values())
    }

    // ── Images ──

    case 'upload_image': {
      const id = uuid()
      const img = { id, filename: args.path?.split('/').pop() || 'image', mime_type: 'image/png', data: [], created_at: now() }
      mockDb.images.set(id, img)
      return { id: img.id, filename: img.filename, mime_type: img.mime_type, created_at: img.created_at }
    }

    case 'upload_image_data': {
      const id = uuid()
      const img = { id, filename: args.filename, mime_type: 'image/png', data: args.data || [], created_at: now() }
      mockDb.images.set(id, img)
      return { id: img.id, filename: img.filename, mime_type: img.mime_type, created_at: img.created_at }
    }

    case 'get_image': {
      const img = mockDb.images.get(args.id)
      if (!img) throw new Error('Image not found')
      return img
    }

    case 'delete_image': {
      mockDb.images.delete(args.id)
      return null
    }

    case 'list_images': {
      return Array.from(mockDb.images.values()).map((img: any) => ({
        id: img.id, filename: img.filename, mime_type: img.mime_type, created_at: img.created_at,
      }))
    }

    // ── Tomes ──

    case 'get_app_state': {
      return { recent_tomes: mockDb.recentTomes }
    }

    case 'create_tome': {
      mockDb.tomeOpen = true
      mockDb.tomeMeta = { name: args.name, description: args.description || null, created_at: now(), cover_image: null }
      resetMockDb()
      const entry = { path: args.path || '/mock/' + args.name.toLowerCase().replace(/\s+/g, '-') + '.vaelorium', name: args.name, description: args.description || null, last_opened: now() }
      mockDb.recentTomes = [entry, ...mockDb.recentTomes.filter((t: any) => t.path !== entry.path)].slice(0, 10)
      return { path: entry.path, name: args.name, description: args.description || null }
    }

    case 'open_tome': {
      mockDb.tomeOpen = true
      const existing = mockDb.recentTomes.find((t: any) => t.path === args.path)
      mockDb.tomeMeta = { name: existing?.name || 'Opened Tome', description: existing?.description || null, created_at: now(), cover_image: null }
      resetMockDb()
      if (existing) existing.last_opened = now()
      return { path: args.path, name: mockDb.tomeMeta.name, description: mockDb.tomeMeta.description }
    }

    case 'close_tome': {
      mockDb.tomeOpen = false
      return null
    }

    case 'get_tome_metadata': {
      if (!mockDb.tomeOpen) throw new Error('No Tome is currently open')
      return mockDb.tomeMeta
    }

    case 'update_tome_metadata': {
      if (!mockDb.tomeOpen) throw new Error('No Tome is currently open')
      if (args.key === 'name') mockDb.tomeMeta.name = args.value
      if (args.key === 'description') mockDb.tomeMeta.description = args.value
      if (args.key === 'cover_image') mockDb.tomeMeta.cover_image = args.value
      return null
    }

    default:
      console.warn(`Mock: unhandled command '${command}'`, args)
      return null
  }
}
