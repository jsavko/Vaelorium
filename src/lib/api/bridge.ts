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
  nextSortOrder: 1,
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
      mockDb.wikiLinks.set(args.source_page_id, args.links)
      return null
    }

    case 'get_backlinks': {
      const backlinks: any[] = []
      for (const [sourceId, links] of mockDb.wikiLinks) {
        for (const link of links) {
          if (link.target_page_id === args.pageId) {
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

    default:
      console.warn(`Mock: unhandled command '${command}'`, args)
      return null
  }
}
